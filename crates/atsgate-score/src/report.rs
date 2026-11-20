//! The report: every dimension with its evidence, the skills and gaps, knockouts, parse
//! penalties, integrity flags, the advisory Q, and one simulated result per vendor.

use crate::evidence::Evidence;
use crate::knockouts::KnockoutResult;
use crate::parse::{IntegrityFlag, Penalty, Quantification};
use crate::skills::{Requirement, SkillResult};
use crate::vendor::{Vendor, Weights};
use atsgate_blueprint::ProvenanceSource;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct Dimension {
    /// 0-1; `null` when the blueprint asks nothing of this dimension (it then carries no weight).
    pub score: Option<f64>,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct Dimensions {
    /// R: required skills.
    pub required_skills: Dimension,
    /// P: preferred skills.
    pub preferred_skills: Dimension,
    /// T: title and seniority.
    pub title: Dimension,
    /// E: years of relevant experience (gaps never penalized).
    pub experience: Dimension,
    /// D: education level and field (institutions never weighted).
    pub education: Dimension,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct Gap {
    pub canonical: String,
    pub requirement: Requirement,
    pub weight: f64,
    /// Spellings the scorer would have matched: the canonical name first, then the blueprint's
    /// aliases, then the taxonomy's, deduplicated ignoring case and capped at
    /// `gaps::WOULD_MATCH_CAP`.
    pub would_match: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct BlueprintInfo {
    pub id: String,
    pub version: String,
    pub title: String,
    pub source: ProvenanceSource,
    pub reviewed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct Citation {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct VendorResult {
    pub vendor: Vendor,
    pub name: String,
    /// 0-100, rounded to one decimal; a simulation, not the vendor's score.
    pub score: f64,
    /// Always "simulated parse and keyword risk".
    pub label: String,
    /// "no documented auto-scoring" for Greenhouse and Lever.
    pub note: Option<String>,
    /// F for this vendor's σ, 0-100.
    pub parse_score: f64,
    pub sigma: f64,
    pub weights: Weights,
    pub citation: Citation,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct Report {
    /// Always "simulated parse and keyword risk".
    pub label: String,
    /// The month "Present" is counted through (`YYYY-MM`).
    pub as_of: String,
    pub blueprint: BlueprintInfo,
    pub dimensions: Dimensions,
    pub relevant_years: f64,
    pub skills: Vec<SkillResult>,
    /// Blueprint skills the resume never names, heaviest first.
    pub gaps: Vec<Gap>,
    pub knockouts: Vec<KnockoutResult>,
    /// Points at σ = 1 that each vendor charges against F.
    pub parse_penalties: Vec<Penalty>,
    /// Hidden or metadata-only text: reported, never scored up; fails the gate by default.
    pub integrity_flags: Vec<IntegrityFlag>,
    /// Advisory only, never weighted.
    pub quantification: Quantification,
    pub vendors: Vec<VendorResult>,
}
