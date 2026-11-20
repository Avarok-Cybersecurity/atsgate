//! The document a person sees: lines without the spans an integrity signal found hidden (white,
//! tiny, off-page or vanished text). Scoring reads only this, so hidden keywords earn nothing;
//! the flags themselves come from the original document.
//!
//! Zero-width characters are not removed here: they sit inside visible words, and resume
//! parsing already drops them.

use atsgate_extract::{Document, HiddenReason, SignalKind, SpanId};
use std::collections::BTreeSet;

fn hides_span(kind: &SignalKind) -> bool {
    matches!(
        kind,
        SignalKind::HiddenText(
            HiddenReason::White
                | HiddenReason::Tiny
                | HiddenReason::OffPage
                | HiddenReason::Vanished
        )
    )
}

/// Span ids that an integrity signal marks as hidden.
pub fn hidden_spans(doc: &Document) -> BTreeSet<SpanId> {
    doc.signals
        .iter()
        .filter(|s| hides_span(&s.kind))
        .flat_map(|s| s.evidence.iter().filter_map(|e| e.span))
        .collect()
}

/// `doc` with hidden spans removed from its lines. Lines without hidden spans are unchanged;
/// a line with some is rebuilt from its visible spans, and a line with only hidden spans goes.
pub fn visible(doc: &Document) -> Document {
    let hidden = hidden_spans(doc);
    if hidden.is_empty() {
        return doc.clone();
    }
    let mut out = doc.clone();
    out.lines = doc
        .lines
        .iter()
        .filter_map(|line| {
            if !line.spans.iter().any(|id| hidden.contains(id)) {
                return Some(line.clone());
            }
            let spans: Vec<SpanId> = line
                .spans
                .iter()
                .copied()
                .filter(|id| !hidden.contains(id))
                .collect();
            let text = spans
                .iter()
                .map(|id| doc.span(*id).text.trim())
                .filter(|t| !t.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            (!text.is_empty()).then(|| atsgate_extract::Line {
                text,
                spans,
                ..line.clone()
            })
        })
        .collect();
    out
}
