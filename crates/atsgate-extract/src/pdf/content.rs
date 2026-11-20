//! A lopdf pass over each page's content stream for what text extraction does not report:
//! ruling paths, image draws, the fonts actually used, and where each text run starts.

use crate::layout::frames::Segment;
use lopdf::content::Operation;
use lopdf::{Dictionary, Document, Object, ObjectId};

/// A 2D affine matrix [a b c d e f], as PDF writes it.
type Matrix = [f32; 6];
const IDENTITY: Matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

fn mul(m: &Matrix, n: &Matrix) -> Matrix {
    [
        m[0] * n[0] + m[1] * n[2],
        m[0] * n[1] + m[1] * n[3],
        m[2] * n[0] + m[3] * n[2],
        m[2] * n[1] + m[3] * n[3],
        m[4] * n[0] + m[5] * n[2] + n[4],
        m[4] * n[1] + m[5] * n[3] + n[5],
    ]
}

fn apply(m: &Matrix, x: f32, y: f32) -> (f32, f32) {
    (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5])
}

/// A text-showing operation: the font resource it used and where it started.
#[derive(Debug, Clone)]
pub struct TextShow {
    pub font: Vec<u8>,
    pub origin: (f32, f32),
}

#[derive(Debug, Default)]
pub struct PageScan {
    pub segments: Vec<Segment>,
    pub images: u32,
    pub shows: Vec<TextShow>,
}

#[derive(Debug, Default)]
struct State {
    ctm_stack: Vec<Matrix>,
    ctm: Option<Matrix>,
    tm: Matrix,
    tlm: Matrix,
    leading: f32,
    font: Vec<u8>,
    pending: Vec<(f32, f32, f32, f32)>,
    cursor: (f32, f32),
    start: (f32, f32),
}

fn nums(op: &Operation) -> Vec<f32> {
    op.operands
        .iter()
        .filter_map(|o| o.as_float().ok())
        .collect()
}

/// Resource names of image XObjects, from inline and referenced (inherited) resource dictionaries.
fn image_names(doc: &Document, page: ObjectId) -> Result<Vec<Vec<u8>>, lopdf::Error> {
    let (inline, referenced) = doc.get_page_resources(page)?;
    let referenced = referenced
        .iter()
        .filter_map(|id| doc.get_dictionary(*id).ok());
    let mut out = Vec::new();
    for resources in inline.into_iter().chain(referenced) {
        let Ok(xobjects) = resources
            .get_deref(b"XObject", doc)
            .and_then(Object::as_dict)
        else {
            continue;
        };
        for (name, value) in xobjects.iter() {
            let stream = doc.dereference(value)?.1.as_stream()?;
            if stream.dict.get(b"Subtype").and_then(Object::as_name).ok()
                == Some(b"Image".as_slice())
            {
                out.push(name.clone());
            }
        }
    }
    Ok(out)
}

impl State {
    fn ctm(&self) -> Matrix {
        self.ctm.unwrap_or(IDENTITY)
    }
    fn next_line(&mut self, tx: f32, ty: f32) {
        self.tlm = mul(&[1.0, 0.0, 0.0, 1.0, tx, ty], &self.tlm);
        self.tm = self.tlm;
    }
    fn show(&mut self, out: &mut PageScan) {
        let origin = apply(&mul(&self.tm, &self.ctm()), 0.0, 0.0);
        out.shows.push(TextShow {
            font: self.font.clone(),
            origin,
        });
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let (a, b) = (
            apply(&self.ctm(), self.cursor.0, self.cursor.1),
            apply(&self.ctm(), x, y),
        );
        self.pending.push((a.0, a.1, b.0, b.1));
        self.cursor = (x, y);
    }
}

/// Scans one page. Form XObjects are not entered (a documented limitation).
pub fn scan(doc: &Document, page: ObjectId) -> Result<PageScan, lopdf::Error> {
    let content = doc.get_and_decode_page_content(page)?;
    let images = image_names(doc, page)?;
    let mut s = State::default();
    let mut out = PageScan::default();
    for op in &content.operations {
        let n = nums(op);
        match op.operator.as_str() {
            "q" => s.ctm_stack.push(s.ctm()),
            "Q" => s.ctm = s.ctm_stack.pop(),
            "cm" if n.len() == 6 => {
                s.ctm = Some(mul(&[n[0], n[1], n[2], n[3], n[4], n[5]], &s.ctm()))
            }
            "BT" => (s.tm, s.tlm) = (IDENTITY, IDENTITY),
            "Tf" => {
                s.font = op
                    .operands
                    .first()
                    .and_then(|o| o.as_name().ok())
                    .map(<[u8]>::to_vec)
                    .unwrap_or_default()
            }
            "Tm" if n.len() == 6 => {
                (s.tm, s.tlm) = (
                    [n[0], n[1], n[2], n[3], n[4], n[5]],
                    [n[0], n[1], n[2], n[3], n[4], n[5]],
                )
            }
            "Td" if n.len() == 2 => s.next_line(n[0], n[1]),
            "TD" if n.len() == 2 => {
                s.leading = -n[1];
                s.next_line(n[0], n[1]);
            }
            "TL" if n.len() == 1 => s.leading = n[0],
            "T*" => s.next_line(0.0, -s.leading),
            "Tj" | "TJ" => s.show(&mut out),
            "'" | "\"" => {
                s.next_line(0.0, -s.leading);
                s.show(&mut out);
            }
            "m" if n.len() == 2 => (s.cursor, s.start) = ((n[0], n[1]), (n[0], n[1])),
            "l" if n.len() == 2 => s.line_to(n[0], n[1]),
            "h" => s.line_to(s.start.0, s.start.1),
            "re" if n.len() == 4 => {
                let (x, y, w, h) = (n[0], n[1], n[2], n[3]);
                (s.cursor, s.start) = ((x, y), (x, y));
                [(x + w, y), (x + w, y + h), (x, y + h), (x, y)]
                    .iter()
                    .for_each(|&(px, py)| s.line_to(px, py));
            }
            "S" | "s" | "f" | "F" | "f*" | "B" | "B*" | "b" | "b*" => {
                let axis = |&(x0, y0, x1, y1): &(f32, f32, f32, f32)| {
                    (x0 - x1).abs() <= 1.0 || (y0 - y1).abs() <= 1.0
                };
                out.segments.extend(
                    s.pending
                        .drain(..)
                        .filter(axis)
                        .map(|(x0, y0, x1, y1)| Segment { x0, y0, x1, y1 }),
                );
            }
            "n" => s.pending.clear(),
            "Do" if op
                .operands
                .first()
                .and_then(|o| o.as_name().ok())
                .is_some_and(|n| images.iter().any(|i| i == n)) =>
            {
                out.images += 1;
            }
            "BI" => out.images += 1,
            _ => {}
        }
    }
    Ok(out)
}

/// The font dictionaries of a page, by resource name.
pub fn fonts(
    doc: &Document,
    page: ObjectId,
) -> Result<std::collections::BTreeMap<Vec<u8>, &Dictionary>, lopdf::Error> {
    doc.get_page_fonts(page)
}
