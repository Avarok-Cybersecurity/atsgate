//! Format-independent checks over spans, characters and metadata.

use crate::model::{MetadataField, Region, Span};
use crate::signals::{Evidence, HiddenReason, Signal, SignalKind};
use crate::thresholds::Thresholds;

pub const ZERO_WIDTH: [char; 5] = ['\u{200B}', '\u{200C}', '\u{200D}', '\u{2060}', '\u{FEFF}'];

/// Collects signals, merging evidence for the same kind on the same page, in first-seen order.
#[derive(Debug, Default)]
pub struct SignalSet(Vec<Signal>);

impl SignalSet {
    pub fn add(&mut self, kind: SignalKind, page: Option<u32>, evidence: Evidence) {
        match self.0.iter_mut().find(|s| s.kind == kind && s.page == page) {
            Some(s) => s.evidence.push(evidence),
            None => self.0.push(Signal {
                kind,
                page,
                evidence: vec![evidence],
            }),
        }
    }
    pub fn extend(&mut self, other: SignalSet) {
        for s in other.0 {
            for e in s.evidence {
                self.add(s.kind.clone(), s.page, e);
            }
        }
    }
    pub fn into_vec(self) -> Vec<Signal> {
        self.0
    }
}

pub fn span_evidence(s: &Span) -> Evidence {
    Evidence {
        text: s.text.clone(),
        span: Some(s.id),
        bbox: s.bbox,
    }
}

/// White, tiny, zero-width, ligature, unmapped and header/footer checks on spans.
pub fn span_signals(spans: &[Span], t: &Thresholds) -> SignalSet {
    let mut out = SignalSet::default();
    for s in spans.iter().filter(|s| !s.text.trim().is_empty()) {
        let page = Some(s.page);
        if s.color
            .is_some_and(|c| c.r.min(c.g).min(c.b) >= t.white_min_channel)
        {
            out.add(
                SignalKind::HiddenText(HiddenReason::White),
                page,
                span_evidence(s),
            );
        }
        if s.font_size.is_some_and(|f| f < t.tiny_font_pt) {
            out.add(
                SignalKind::HiddenText(HiddenReason::Tiny),
                page,
                span_evidence(s),
            );
        }
        for (kind, found) in text_checks(&s.text) {
            if found {
                out.add(kind, page, span_evidence(s));
            }
        }
        if matches!(s.region, Region::Header | Region::Footer) {
            out.add(SignalKind::HeaderFooterContent, page, span_evidence(s));
        }
    }
    out
}

fn text_checks(text: &str) -> [(SignalKind, bool); 3] {
    [
        (
            SignalKind::HiddenText(HiddenReason::ZeroWidth),
            text.contains(ZERO_WIDTH),
        ),
        (
            SignalKind::LigatureGlyphs,
            text.chars().any(|c| ('\u{FB00}'..='\u{FB06}').contains(&c)),
        ),
        (SignalKind::UnmappedGlyphs, text.contains('\u{FFFD}')),
    ]
}

fn terms(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() >= 3)
        .map(str::to_lowercase)
        .collect()
}

/// Terms that appear in a metadata field but nowhere in the visible text. A document with no
/// text at all is skipped: there is nothing to compare against, and it is flagged as image-only.
pub fn metadata_signals(metadata: &[MetadataField], spans: &[Span]) -> SignalSet {
    if spans.is_empty() {
        return SignalSet::default();
    }
    let visible: std::collections::HashSet<String> =
        spans.iter().flat_map(|s| terms(&s.text)).collect();
    let mut out = SignalSet::default();
    for field in metadata {
        let mut seen = Vec::new();
        for term in terms(&field.value) {
            if !visible.contains(&term) && !seen.contains(&term) {
                seen.push(term);
            }
        }
        if !seen.is_empty() {
            let evidence = Evidence {
                text: seen.join(" "),
                span: None,
                bbox: None,
            };
            out.add(
                SignalKind::MetadataOnlyText { field: field.name },
                None,
                evidence,
            );
        }
    }
    out
}
