//! Evidence: the rule that produced a point, and where in the document it came from.

use atsgate_extract::{BBox, Document};
use atsgate_resume::LineRef;
use atsgate_taxonomy::fold;
use schemars::JsonSchema;
use serde::Serialize;

/// A box in PDF user space (points, origin bottom-left): `[x0, y0, x1, y1]`.
pub type Box4 = [f32; 4];

fn box4(b: BBox) -> Box4 {
    [b.x0, b.y0, b.x1, b.y1]
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct Evidence {
    /// Stable rule id, e.g. `skill.required` or `layout.multi_column`.
    pub rule: String,
    /// 1-based page; `null` for text that belongs to no page (document metadata).
    pub page: Option<u32>,
    /// Present for PDF input; flow formats carry no geometry.
    pub bbox: Option<Box4>,
    /// Indexes into the extracted document's spans.
    pub spans: Vec<u32>,
    /// The text the rule read.
    pub text: String,
}

impl Evidence {
    /// Evidence that points at no document position (a computed or absent fact).
    pub fn note(rule: &str, text: impl Into<String>) -> Self {
        Self {
            rule: rule.to_owned(),
            page: None,
            bbox: None,
            spans: Vec::new(),
            text: text.into(),
        }
    }

    /// Evidence at one document line.
    pub fn at_line(rule: &str, doc: &Document, at: &LineRef, text: impl Into<String>) -> Self {
        let line = doc.lines.get(at.index);
        Self {
            rule: rule.to_owned(),
            page: Some(at.page),
            bbox: at.bbox.map(box4),
            spans: line
                .map(|l| l.spans.iter().map(|s| s.0).collect())
                .unwrap_or_default(),
            text: text.into(),
        }
    }

    /// Evidence for an extraction signal.
    pub fn of_signal(rule: &str, page: Option<u32>, e: &atsgate_extract::Evidence) -> Self {
        Self {
            rule: rule.to_owned(),
            page,
            bbox: e.bbox.map(box4),
            spans: e.span.iter().map(|s| s.0).collect(),
            text: e.text.clone(),
        }
    }
}

/// The line among `lines` whose text contains `needle` (after folding), else the first line.
pub fn locate<'a>(doc: &Document, lines: &'a [LineRef], needle: &str) -> Option<&'a LineRef> {
    let key = fold(needle);
    lines
        .iter()
        .find(|at| {
            doc.lines
                .get(at.index)
                .is_some_and(|l| format!(" {} ", fold(&l.text)).contains(&format!(" {key} ")))
        })
        .or_else(|| lines.first())
}

/// Evidence for `text` found somewhere in `lines`, or a note when the unit has no lines.
pub fn found(rule: &str, doc: &Document, lines: &[LineRef], needle: &str, text: &str) -> Evidence {
    match locate(doc, lines, needle) {
        Some(at) => Evidence::at_line(rule, doc, at, text),
        None => Evidence::note(rule, text),
    }
}
