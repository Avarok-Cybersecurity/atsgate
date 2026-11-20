//! Vendor profiles: a weight vector over (F, R, P, T, E, D) and a parsing strictness σ each.
//!
//! The starting weights and σ are ats-screener's (S. Patel, MIT licence;
//! `src/lib/engine/scorer/profiles/*.ts` at commit 4fd3009, opened 2026-11-20), re-mapped:
//! - formatting + section completeness → F (both are what ATSGate's parse integrity measures);
//! - keyword match → R and P, three quarters to required skills and one quarter to preferred;
//! - experience relevance → T and E, half each (title fit and years);
//! - education match → D;
//! - quantification is advisory in ATSGate (Q), so its weight is dropped and the rest is
//!   re-normalized to sum to 1.
//!
//! Every result is a **simulated parse and keyword risk**, never a vendor's score: no vendor
//! publishes one, and Greenhouse and Lever document no auto-scoring at all.

use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Vendor {
    Workday,
    Taleo,
    Icims,
    Greenhouse,
    Lever,
    SuccessFactors,
}

/// ats-screener's weights for one platform, as published.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SourceWeights {
    pub formatting: f64,
    pub keyword_match: f64,
    pub section_completeness: f64,
    pub experience_relevance: f64,
    pub education_match: f64,
    pub quantification: f64,
}

/// ATSGate's weights, summing to 1.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, JsonSchema)]
pub struct Weights {
    pub f: f64,
    pub r: f64,
    pub p: f64,
    pub t: f64,
    pub e: f64,
    pub d: f64,
}

pub const REQUIRED_SHARE: f64 = 0.75;
pub const TITLE_SHARE: f64 = 0.5;

impl SourceWeights {
    pub fn remap(&self) -> Weights {
        let kept = 1.0 - self.quantification;
        Weights {
            f: (self.formatting + self.section_completeness) / kept,
            r: self.keyword_match * REQUIRED_SHARE / kept,
            p: self.keyword_match * (1.0 - REQUIRED_SHARE) / kept,
            t: self.experience_relevance * TITLE_SHARE / kept,
            e: self.experience_relevance * (1.0 - TITLE_SHARE) / kept,
            d: self.education_match / kept,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Profile {
    pub vendor: Vendor,
    pub name: &'static str,
    /// The key used in `floors` overrides.
    pub key: &'static str,
    pub source: SourceWeights,
    /// Parsing strictness σ (ats-screener's `parsingStrictness`).
    pub sigma: f64,
    /// Default floor (ats-screener's thresholds.md, not its profiles' `passingScore`).
    pub floor: u8,
    /// [`NO_AUTO_SCORING`] for vendors whose own documentation, or the research ATSGate cites,
    /// says they do not score or reject applicants automatically.
    pub auto_scoring_note: Option<&'static str>,
    /// The CITATIONS.md id of the vendor documentation this profile links.
    pub citation: &'static str,
}

const fn w(f: f64, k: f64, s: f64, x: f64, e: f64, q: f64) -> SourceWeights {
    SourceWeights {
        formatting: f,
        keyword_match: k,
        section_completeness: s,
        experience_relevance: x,
        education_match: e,
        quantification: q,
    }
}

/// The six profiles, strictest default floor first.
pub const PROFILES: [Profile; 6] = [
    Profile {
        vendor: Vendor::Taleo,
        name: "Taleo",
        key: "taleo",
        source: w(0.2, 0.35, 0.15, 0.15, 0.1, 0.05),
        sigma: 0.85,
        floor: 75,
        auto_scoring_note: None,
        citation: "taleo-attach",
    },
    Profile {
        vendor: Vendor::Workday,
        name: "Workday",
        key: "workday",
        source: w(0.25, 0.3, 0.15, 0.15, 0.1, 0.05),
        sigma: 0.9,
        floor: 70,
        auto_scoring_note: None,
        citation: "wd-parse",
    },
    Profile {
        vendor: Vendor::SuccessFactors,
        name: "SuccessFactors",
        key: "successfactors",
        source: w(0.25, 0.25, 0.2, 0.15, 0.1, 0.05),
        sigma: 0.85,
        floor: 65,
        auto_scoring_note: None,
        citation: "sap-kba",
    },
    Profile {
        vendor: Vendor::Icims,
        name: "iCIMS",
        key: "icims",
        source: w(0.15, 0.3, 0.15, 0.2, 0.1, 0.1),
        sigma: 0.6,
        floor: 60,
        auto_scoring_note: None,
        citation: "icims-blog",
    },
    Profile {
        vendor: Vendor::Greenhouse,
        name: "Greenhouse",
        key: "greenhouse",
        source: w(0.1, 0.25, 0.1, 0.25, 0.1, 0.2),
        sigma: 0.4,
        floor: 50,
        auto_scoring_note: Some(NO_AUTO_SCORING),
        citation: "gh-tm-faq",
    },
    Profile {
        vendor: Vendor::Lever,
        name: "Lever",
        key: "lever",
        source: w(0.08, 0.22, 0.1, 0.3, 0.1, 0.2),
        sigma: 0.35,
        floor: 50,
        auto_scoring_note: Some(NO_AUTO_SCORING),
        citation: "ats-screener-research",
    },
];

pub fn profile(vendor: Vendor) -> &'static Profile {
    PROFILES
        .iter()
        .find(|p| p.vendor == vendor)
        .expect("PROFILES lists every Vendor")
}

pub const SIMULATED_LABEL: &str = "simulated parse and keyword risk";
pub const NO_AUTO_SCORING: &str = "no documented auto-scoring";
