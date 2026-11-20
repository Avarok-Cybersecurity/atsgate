//! Knockouts: one gate per item, met, unmet or unknown. They earn no points.
//!
//! - `min_years`: relevant years against the value; unknown when no role carries dates.
//! - `degree`: the highest degree level the resume names against the level the value names;
//!   unknown when the resume names no degree.
//! - `certification`, `clearance`, `authorization`, `language`: met when the value appears as a
//!   phrase in the resume text the scorer reads; otherwise unknown, because a resume that does
//!   not mention something has not shown that the candidate lacks it.
//!
//! An unknown knockout resolves through its `unknown_policy`: pass, fail, or review (reported
//! for a person to decide; it does not fail the gate).

use crate::education::degree_level;
use crate::evidence::{Evidence, found};
use crate::units::Unit;
use atsgate_blueprint::{Knockout, KnockoutType, UnknownPolicy};
use atsgate_extract::Document;
use atsgate_taxonomy::fold;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Met,
    Unmet,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Pass,
    Fail,
    Review,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct KnockoutResult {
    #[serde(rename = "type")]
    pub kind: KnockoutType,
    pub value: String,
    pub status: Status,
    pub unknown_policy: UnknownPolicy,
    pub outcome: Outcome,
    pub evidence: Vec<Evidence>,
}

/// What the knockouts read from the rest of the score.
#[derive(Debug, Clone, Copy)]
pub struct Facts {
    pub relevant_years: f64,
    pub dated_roles: usize,
    pub highest_degree: Option<u8>,
}

fn outcome(status: Status, policy: UnknownPolicy) -> Outcome {
    match (status, policy) {
        (Status::Met, _) | (Status::Unknown, UnknownPolicy::Pass) => Outcome::Pass,
        (Status::Unmet, _) | (Status::Unknown, UnknownPolicy::Fail) => Outcome::Fail,
        (Status::Unknown, UnknownPolicy::Review) => Outcome::Review,
    }
}

fn phrase(doc: &Document, units: &[Unit], value: &str) -> (Status, Vec<Evidence>) {
    let key = format!(" {} ", fold(value));
    match units
        .iter()
        .find(|u| format!(" {} ", fold(&u.text)).contains(&key))
    {
        Some(u) => (
            Status::Met,
            vec![found("knockout.phrase", doc, &u.lines, value, &u.text)],
        ),
        None => (
            Status::Unknown,
            vec![Evidence::note(
                "knockout.phrase",
                format!("{value:?} is not mentioned"),
            )],
        ),
    }
}

fn evaluate(k: &Knockout, facts: Facts, doc: &Document, units: &[Unit]) -> (Status, Vec<Evidence>) {
    match k.kind {
        KnockoutType::MinYears => {
            let Ok(want) = k.value.trim().parse::<f64>() else {
                return (
                    Status::Unknown,
                    vec![Evidence::note(
                        "knockout.min_years",
                        "value is not a number",
                    )],
                );
            };
            let status = match facts.dated_roles {
                0 => Status::Unknown,
                _ if facts.relevant_years >= want => Status::Met,
                _ => Status::Unmet,
            };
            let note = format!("{:.2} relevant years against {want}", facts.relevant_years);
            (status, vec![Evidence::note("knockout.min_years", note)])
        }
        KnockoutType::Degree => {
            let want = degree_level(&k.value);
            let status = match (facts.highest_degree, want) {
                (_, None) | (None, _) => Status::Unknown,
                (Some(have), Some(want)) if have >= want => Status::Met,
                _ => Status::Unmet,
            };
            let note = format!("highest level {:?} against {want:?}", facts.highest_degree);
            (status, vec![Evidence::note("knockout.degree", note)])
        }
        KnockoutType::Certification
        | KnockoutType::Clearance
        | KnockoutType::Authorization
        | KnockoutType::Language => phrase(doc, units, &k.value),
    }
}

pub fn evaluate_all(
    knockouts: &[Knockout],
    facts: Facts,
    doc: &Document,
    units: &[Unit],
) -> Vec<KnockoutResult> {
    knockouts
        .iter()
        .map(|k| {
            let (status, evidence) = evaluate(k, facts, doc, units);
            KnockoutResult {
                kind: k.kind,
                value: k.value.clone(),
                status,
                unknown_policy: k.unknown_policy,
                outcome: outcome(status, k.unknown_policy),
                evidence,
            }
        })
        .collect()
}
