//! Flow formats (DOCX, HTML, Markdown, plain text): no geometry, so reading order is document order.

use crate::error::ExtractError;
use crate::integrity::{self, SignalSet, span_evidence};
use crate::model::{Document, Format, Line, MetadataField, Page, Region, Rgb, Span, SpanId};
use crate::signals::{HiddenReason, SignalKind};
use crate::thresholds::Thresholds;

mod html;
pub use html::html;

/// One styled run of a paragraph.
#[derive(Debug, Clone, Default)]
pub struct Run {
    pub text: String,
    pub font_size: Option<f32>,
    pub color: Option<Rgb>,
    /// Marked hidden by the format itself (DOCX `w:vanish`, HTML `hidden` or `display:none`).
    pub vanished: bool,
}

#[derive(Debug, Default)]
pub struct FlowBuilder {
    spans: Vec<Span>,
    lines: Vec<Line>,
    pub signals: SignalSet,
    pub images: u32,
}

impl FlowBuilder {
    /// Adds one paragraph as one line; empty paragraphs are dropped.
    pub fn paragraph(&mut self, runs: Vec<Run>, region: Region) {
        let runs: Vec<Run> = runs.into_iter().filter(|r| !r.text.is_empty()).collect();
        if runs.iter().all(|r| r.text.trim().is_empty()) {
            return;
        }
        let mut line = Line {
            text: String::new(),
            page: 1,
            bbox: None,
            spans: Vec::new(),
            region,
        };
        for run in runs {
            let id = SpanId(self.spans.len() as u32);
            let span = Span {
                id,
                text: run.text,
                page: 1,
                bbox: None,
                font_size: run.font_size,
                color: run.color,
                region,
            };
            if run.vanished {
                self.signals.add(
                    SignalKind::HiddenText(HiddenReason::Vanished),
                    Some(1),
                    span_evidence(&span),
                );
            }
            line.text.push_str(&span.text);
            line.spans.push(id);
            self.spans.push(span);
        }
        line.text = line.text.trim().to_owned();
        self.lines.push(line);
    }

    pub fn plain(&mut self, text: &str, region: Region) {
        self.paragraph(
            vec![Run {
                text: text.to_owned(),
                ..Run::default()
            }],
            region,
        );
    }

    pub fn finish(
        mut self,
        format: Format,
        metadata: Vec<MetadataField>,
        t: &Thresholds,
    ) -> Document {
        let mut signals = integrity::span_signals(&self.spans, t);
        signals.extend(integrity::metadata_signals(&metadata, &self.spans));
        for (region, kind) in [
            (Region::Table, SignalKind::Table),
            (Region::TextBox, SignalKind::TextBox),
        ] {
            for s in self.spans.iter().filter(|s| s.region == region) {
                signals.add(kind.clone(), Some(1), span_evidence(s));
            }
        }
        if self.spans.is_empty() && self.images > 0 {
            let evidence = crate::signals::Evidence {
                text: format!("{} image(s), no text", self.images),
                span: None,
                bbox: None,
            };
            signals.add(SignalKind::ImageOnlyPage, Some(1), evidence);
        }
        signals.extend(std::mem::take(&mut self.signals));
        let page = Page {
            number: 1,
            size: None,
            image_count: self.images,
            span_count: self.spans.len() as u32,
        };
        Document {
            format,
            pages: vec![page],
            spans: self.spans,
            lines: self.lines,
            metadata,
            signals: signals.into_vec(),
        }
    }
}

pub fn text(input: &str, t: &Thresholds) -> Result<Document, ExtractError> {
    let mut b = FlowBuilder::default();
    input.lines().for_each(|l| b.plain(l, Region::Body));
    Ok(b.finish(Format::Text, Vec::new(), t))
}

pub fn markdown(input: &str, t: &Thresholds) -> Result<Document, ExtractError> {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
    let mut b = FlowBuilder::default();
    let (mut line, mut in_table) = (String::new(), false);
    let flush = |b: &mut FlowBuilder, line: &mut String, table: bool| {
        b.plain(line, if table { Region::Table } else { Region::Body });
        line.clear();
    };
    for event in Parser::new_ext(input, Options::ENABLE_TABLES) {
        match event {
            Event::Start(Tag::Table(_)) => in_table = true,
            Event::End(TagEnd::Table) => in_table = false,
            Event::Start(Tag::Item) => {
                flush(&mut b, &mut line, in_table);
                line.push_str("\u{2022} ");
            }
            Event::Start(Tag::Image { .. }) => b.images += 1,
            Event::Text(s) | Event::Code(s) => line.push_str(&s),
            Event::SoftBreak | Event::HardBreak => flush(&mut b, &mut line, in_table),
            Event::End(TagEnd::TableCell) => line.push(' '),
            Event::End(
                TagEnd::Paragraph
                | TagEnd::Heading(_)
                | TagEnd::Item
                | TagEnd::TableRow
                | TagEnd::TableHead,
            ) => flush(&mut b, &mut line, in_table),
            _ => {}
        }
    }
    flush(&mut b, &mut line, in_table);
    Ok(b.finish(Format::Markdown, Vec::new(), t))
}
