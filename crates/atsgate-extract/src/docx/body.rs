//! Walks one WordprocessingML part (document, header or footer) into flow paragraphs.

use crate::error::ExtractError;
use crate::flow::{FlowBuilder, Run};
use crate::model::{Region, Rgb};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

fn attr(e: &BytesStart, key: &str) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key)
        .map(|a| a.value.to_string())
}

fn color(hex: &str) -> Option<Rgb> {
    let v = u32::from_str_radix(hex, 16)
        .ok()
        .filter(|_| hex.len() == 6)?;
    let ch = |shift: u32| ((v >> shift) & 0xFF) as f32 / 255.0;
    Some(Rgb {
        r: ch(16),
        g: ch(8),
        b: ch(0),
    })
}

fn on(e: &BytesStart) -> bool {
    !matches!(attr(e, "w:val").as_deref(), Some("0" | "false" | "off"))
}

#[derive(Debug, Default)]
struct Walk {
    paragraphs: Vec<Vec<Run>>,
    run: Option<Run>,
    tables: u32,
    text_boxes: u32,
    in_ppr: bool,
    in_text: bool,
    skip_text: bool,
}

impl Walk {
    fn region(&self, part: Region) -> Region {
        match (self.text_boxes, self.tables) {
            (1.., _) => Region::TextBox,
            (0, 1..) => Region::Table,
            _ => part,
        }
    }
    fn flush_run(&mut self) {
        if let (Some(run), Some(p)) = (self.run.take(), self.paragraphs.last_mut()) {
            p.push(run);
        }
    }
    fn push_text(&mut self, text: &str) {
        if let Some(run) = self.run.as_mut() {
            run.text.push_str(text);
        }
    }
}

/// Returns the section column count (`w:cols w:num`) seen in this part, if any.
pub fn walk(xml: &str, part: Region, out: &mut FlowBuilder) -> Result<Option<u32>, ExtractError> {
    let mut reader = Reader::from_str(xml);
    let mut w = Walk::default();
    let mut columns = None;
    loop {
        let event = reader
            .read_event()
            .map_err(|e| ExtractError::Xml(e.to_string()))?;
        match &event {
            Event::Start(e) | Event::Empty(e) => {
                let empty = matches!(event, Event::Empty(_));
                match e.name().as_ref() {
                    "w:p" if !empty => w.paragraphs.push(Vec::new()),
                    "w:tbl" => w.tables += 1,
                    "w:txbxContent" => w.text_boxes += 1,
                    "w:pPr" if !empty => w.in_ppr = true,
                    "w:r" if !empty && !w.in_ppr => w.run = Some(Run::default()),
                    "w:t" if !empty => w.in_text = true,
                    "w:instrText" | "w:delText" if !empty => w.skip_text = true,
                    "w:tab" if !w.in_ppr => w.push_text("\t"),
                    "w:color" if !w.in_ppr => {
                        let c = attr(e, "w:val").and_then(|v| color(&v));
                        w.run.iter_mut().for_each(|r| r.color = c);
                    }
                    "w:sz" if !w.in_ppr => {
                        let size = attr(e, "w:val")
                            .and_then(|v| v.parse::<f32>().ok())
                            .map(|hp| hp / 2.0);
                        w.run.iter_mut().for_each(|r| r.font_size = size);
                    }
                    "w:vanish" if !w.in_ppr => w.run.iter_mut().for_each(|r| r.vanished = on(e)),
                    "a:blip" => out.images += 1,
                    "w:cols" => columns = attr(e, "w:num").and_then(|n| n.parse().ok()),
                    _ => {}
                }
            }
            Event::Text(t) if w.in_text && !w.skip_text => {
                w.push_text(t.as_ref());
            }
            Event::GeneralRef(r) if w.in_text && !w.skip_text => {
                match r
                    .resolve_char_ref()
                    .map_err(|e| ExtractError::Xml(e.to_string()))?
                {
                    Some(c) => w.push_text(c.encode_utf8(&mut [0; 4])),
                    None => {
                        let name: &str = r.as_ref();
                        let resolved = quick_xml::escape::resolve_predefined_entity(name)
                            .ok_or_else(|| ExtractError::Xml(format!("unknown entity &{name};")))?;
                        w.push_text(resolved);
                    }
                }
            }
            Event::End(e) => match e.name().as_ref() {
                "w:p" => {
                    w.flush_run();
                    let runs = w.paragraphs.pop().unwrap_or_default();
                    out.paragraph(runs, w.region(part));
                }
                "w:r" => w.flush_run(),
                "w:t" => w.in_text = false,
                "w:instrText" | "w:delText" => w.skip_text = false,
                "w:pPr" => w.in_ppr = false,
                "w:tbl" => w.tables -= 1,
                "w:txbxContent" => w.text_boxes -= 1,
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(columns)
}
