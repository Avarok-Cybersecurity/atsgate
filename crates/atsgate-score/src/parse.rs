//! Parse integrity F = max(0, 100 − Σ penalty·σ_vendor), integrity flags, and the
//! quantification heuristic Q.
//!
//! Base penalties (points at σ = 1). Where ats-screener's formatting dimension has the same
//! signal its value is kept (multi-column 15, tables 12, more than two pages 5, all-caps lines 3);
//! the others are ATSGate's, ranked by how much text a naive parser loses:
//!
//! | rule | points | why |
//! |---|---|---|
//! | `layout.image_only_page` | 100 | no text to parse at all |
//! | `layout.multi_column` | 15 | row-by-row parsers interleave the columns |
//! | `layout.reading_order_disagreement` | 15 | stored order scrambles blocks the same way |
//! | `glyphs.missing_tounicode` | 15 | glyphs extract as nothing or as garbage |
//! | `layout.table` | 12 | cells are skipped or flattened |
//! | `layout.text_box` | 12 | out-of-flow boxes are skipped or misplaced |
//! | `layout.header_footer_content` | 10 | many parsers never read the margins |
//! | `glyphs.unmapped` | 8 | replacement characters break words |
//! | `glyphs.ligature` | 8 | "ﬁ" breaks keyword matching |
//! | `parse.page_count` | 5 | more than two pages may be truncated |
//! | `parse.nonstandard_heading` | 3 each, at most 9 | sections may not be recognized |
//! | `parse.undated_role` | 3 each, at most 9 | a role without dates adds no years |
//!
//! Each signal kind is charged once, however many pages or spans carry it.

use crate::evidence::Evidence;
use crate::units;
use atsgate_extract::{Document, SignalKind};
use atsgate_resume::{Resume, SectionKind};
use schemars::JsonSchema;
use serde::Serialize;

pub const PAGE_LIMIT: usize = 2;
pub const PER_ITEM: f64 = 3.0;
pub const PER_ITEM_CAP: f64 = 9.0;

/// The base penalty of a non-integrity extraction signal.
pub fn signal_penalty(kind: &SignalKind) -> Option<f64> {
    match kind {
        SignalKind::ImageOnlyPage => Some(100.0),
        SignalKind::MultiColumn { .. } | SignalKind::ReadingOrderDisagreement { .. } => Some(15.0),
        SignalKind::MissingToUnicode { .. } => Some(15.0),
        SignalKind::Table | SignalKind::TextBox => Some(12.0),
        SignalKind::HeaderFooterContent => Some(10.0),
        SignalKind::UnmappedGlyphs | SignalKind::LigatureGlyphs => Some(8.0),
        SignalKind::HiddenText(_) | SignalKind::MetadataOnlyText { .. } => None,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct Penalty {
    pub rule: String,
    /// Points at σ = 1; each vendor charges points·σ.
    pub points: f64,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct IntegrityFlag {
    pub rule: String,
    pub evidence: Vec<Evidence>,
}

fn signal_evidence(code: &str, s: &atsgate_extract::Signal) -> Vec<Evidence> {
    s.evidence
        .iter()
        .map(|e| Evidence::of_signal(code, s.page, e))
        .collect()
}

/// Penalties in a fixed order: extraction signals as the extractor reported them, then
/// headings, undated roles and page count.
pub fn penalties(doc: &Document, resume: &Resume) -> Vec<Penalty> {
    let mut out: Vec<Penalty> = Vec::new();
    for s in &doc.signals {
        let Some(points) = signal_penalty(&s.kind) else {
            continue;
        };
        let code = s.kind.code();
        match out.iter_mut().find(|p| p.rule == code) {
            Some(p) => p.evidence.extend(signal_evidence(code, s)),
            None => out.push(Penalty {
                rule: code.to_owned(),
                points,
                evidence: signal_evidence(code, s),
            }),
        }
    }
    let headings: Vec<Evidence> = resume
        .unrecognized_headings
        .iter()
        .map(|at| {
            let text = doc.lines.get(at.index).map_or("", |l| l.text.as_str());
            Evidence::at_line("parse.nonstandard_heading", doc, at, text)
        })
        .collect();
    let undated: Vec<Evidence> = resume
        .sections
        .iter()
        .filter(|s| s.kind == SectionKind::Experience)
        .flat_map(|s| &s.entries)
        .filter(|e| e.dates.is_none())
        .map(|e| match e.evidence.first() {
            Some(at) => Evidence::at_line("parse.undated_role", doc, at, e.title.clone()),
            None => Evidence::note("parse.undated_role", e.title.clone()),
        })
        .collect();
    for (rule, evidence) in [
        ("parse.nonstandard_heading", headings),
        ("parse.undated_role", undated),
    ] {
        if !evidence.is_empty() {
            let points = (PER_ITEM * evidence.len() as f64).min(PER_ITEM_CAP);
            out.push(Penalty {
                rule: rule.to_owned(),
                points,
                evidence,
            });
        }
    }
    if doc.pages.len() > PAGE_LIMIT {
        let note = format!("{} pages", doc.pages.len());
        out.push(Penalty {
            rule: "parse.page_count".to_owned(),
            points: 5.0,
            evidence: vec![Evidence::note("parse.page_count", note)],
        });
    }
    out
}

/// F for a parser of strictness `sigma`, on a 0-100 scale.
pub fn parse_score(penalties: &[Penalty], sigma: f64) -> f64 {
    (100.0 - penalties.iter().map(|p| p.points * sigma).sum::<f64>()).max(0.0)
}

/// Text a reader cannot see but a parser reads. Reported, never scored up.
pub fn integrity_flags(doc: &Document) -> Vec<IntegrityFlag> {
    let mut out: Vec<IntegrityFlag> = Vec::new();
    for s in doc.signals.iter().filter(|s| s.kind.is_integrity()) {
        let code = s.kind.code();
        match out.iter_mut().find(|f| f.rule == code) {
            Some(f) => f.evidence.extend(signal_evidence(code, s)),
            None => out.push(IntegrityFlag {
                rule: code.to_owned(),
                evidence: signal_evidence(code, s),
            }),
        }
    }
    out
}

pub const QUANTIFICATION_LABEL: &str = "heuristic, no outcome evidence";

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct Quantification {
    /// Always "heuristic, no outcome evidence": no study links this ratio to hiring outcomes.
    pub label: String,
    /// Share of experience and project bullets that contain a number; `null` with no bullets.
    pub score: Option<f64>,
    pub quantified: usize,
    pub bullets: usize,
}

pub fn quantification(resume: &Resume) -> Quantification {
    let bullets = units::bullets(resume);
    let quantified = bullets
        .iter()
        .filter(|(b, _)| b.chars().any(|c| c.is_ascii_digit()))
        .count();
    Quantification {
        label: QUANTIFICATION_LABEL.to_owned(),
        score: (!bullets.is_empty()).then(|| quantified as f64 / bullets.len() as f64),
        quantified,
        bullets: bullets.len(),
    }
}
