//! The gate: fails on any vendor below its floor, any failed knockout, and any integrity flag
//! when `fail_on_integrity_flags` is set.

use crate::floors::Floors;
use crate::knockouts::Outcome;
use crate::report::Report;
use crate::vendor::Vendor;
use schemars::JsonSchema;
use serde::Serialize;

/// A score meets its floor when it is at least the floor: 70.0 passes a floor of 70.
pub fn meets_floor(score: f64, floor: u8) -> bool {
    score >= f64::from(floor)
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct VendorVerdict {
    pub vendor: Vendor,
    pub name: String,
    pub score: f64,
    pub floor: u8,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct Verdict {
    pub passed: bool,
    pub min_vendor_score: f64,
    pub vendors: Vec<VendorVerdict>,
    pub fail_on_integrity_flags: bool,
    /// Why the gate failed, one line per cause; empty when it passed.
    pub reasons: Vec<String>,
    /// Knockouts left for a person to decide (`unknown_policy: review`); they do not fail it.
    pub review: Vec<String>,
}

pub fn gate(report: &Report, floors: &Floors, fail_on_integrity_flags: bool) -> Verdict {
    let mut reasons = Vec::new();
    let vendors: Vec<VendorVerdict> = report
        .vendors
        .iter()
        .map(|v| {
            let floor = floors.get(v.vendor);
            let passed = meets_floor(v.score, floor);
            if !passed {
                reasons.push(format!(
                    "{} {:.1} is below its floor {floor}",
                    v.name, v.score
                ));
            }
            VendorVerdict {
                vendor: v.vendor,
                name: v.name.clone(),
                score: v.score,
                floor,
                passed,
            }
        })
        .collect();
    let describe = |k: &crate::knockouts::KnockoutResult| format!("{:?} {}", k.kind, k.value);
    for k in report
        .knockouts
        .iter()
        .filter(|k| k.outcome == Outcome::Fail)
    {
        reasons.push(format!("knockout failed: {} ({:?})", describe(k), k.status));
    }
    if fail_on_integrity_flags {
        for f in &report.integrity_flags {
            reasons.push(format!("integrity flag: {}", f.rule));
        }
    }
    let review = report
        .knockouts
        .iter()
        .filter(|k| k.outcome == Outcome::Review)
        .map(describe)
        .collect();
    let min_vendor_score = vendors
        .iter()
        .map(|v| v.score)
        .fold(f64::INFINITY, f64::min);
    Verdict {
        passed: reasons.is_empty(),
        min_vendor_score,
        vendors,
        fail_on_integrity_flags,
        reasons,
        review,
    }
}
