//! Deterministic scoring of a resume against a blueprint; every point carries evidence.
//!
//! Pure: an extracted [`Document`], a [`Blueprint`] and the taxonomy [`Gazetteer`] in, a
//! [`Report`] out. The document is parsed into a resume after hidden text is removed
//! ([`visible`]), so hidden keywords earn nothing. [`gate`] applies vendor floors, knockouts and integrity
//! flags to a report. Nothing here reads a file, the network or the clock.
//!
//! Never read as inputs: the name and contact lines (the preamble and page headers), photos,
//! institutions, graduation years, and the time between roles.

pub mod citations;
pub mod education;
pub mod evidence;
pub mod experience;
pub mod floors;
pub mod gate;
pub mod knockouts;
pub mod parse;
pub mod report;
pub mod skills;
pub mod title;
pub mod units;
pub mod vendor;
pub mod visible;

pub use evidence::Evidence;
pub use experience::YearMonth;
pub use floors::{Floors, FloorsError};
pub use gate::{Verdict, gate, meets_floor};
pub use report::Report;
pub use vendor::{PROFILES, SIMULATED_LABEL, Vendor};

use atsgate_blueprint::Blueprint;
use atsgate_extract::Document;
use atsgate_taxonomy::Gazetteer;
use report::{BlueprintInfo, Citation, Dimension, Dimensions, Gap, VendorResult};
use skills::{Requirement, SkillResult};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy)]
pub struct Input<'a> {
    pub document: &'a Document,
    pub blueprint: &'a Blueprint,
    pub gazetteer: &'a Gazetteer<'a>,
    /// The month "Present" counts through.
    pub as_of: YearMonth,
}

fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

fn gaps(results: &[SkillResult], blueprint: &Blueprint, gz: &Gazetteer<'_>) -> Vec<Gap> {
    let all = blueprint
        .required_skills
        .iter()
        .chain(&blueprint.preferred_skills);
    let mut out: Vec<Gap> = results
        .iter()
        .zip(all)
        .filter(|(r, _)| r.matched.is_none())
        .map(|(r, s)| Gap {
            canonical: r.canonical.clone(),
            requirement: r.requirement,
            weight: r.weight,
            would_match: skills::would_match(s, gz),
        })
        .collect();
    out.sort_by(|a, b| {
        let req = |g: &Gap| g.requirement == Requirement::Preferred;
        req(a).cmp(&req(b)).then(b.weight.total_cmp(&a.weight))
    });
    out
}

fn vendor_results(dims: &Dimensions, penalties: &[parse::Penalty]) -> Vec<VendorResult> {
    PROFILES
        .iter()
        .map(|p| {
            let w = p.source.remap();
            let f = parse::parse_score(penalties, p.sigma);
            let parts = [
                (w.f, Some(f / 100.0)),
                (w.r, dims.required_skills.score),
                (w.p, dims.preferred_skills.score),
                (w.t, dims.title.score),
                (w.e, dims.experience.score),
                (w.d, dims.education.score),
            ];
            let weight: f64 = parts
                .iter()
                .filter(|(_, x)| x.is_some())
                .map(|(w, _)| w)
                .sum();
            let earned: f64 = parts.iter().filter_map(|(w, x)| x.map(|x| w * x)).sum();
            VendorResult {
                vendor: p.vendor,
                name: p.name.to_owned(),
                score: round1(100.0 * earned / weight),
                label: SIMULATED_LABEL.to_owned(),
                note: p.auto_scoring_note.map(str::to_owned),
                parse_score: round1(f),
                sigma: p.sigma,
                weights: w,
                citation: Citation {
                    id: p.citation.to_owned(),
                    url: citations::url(p.citation)
                        .expect("every vendor citation is in CITATIONS.md (a test checks it)")
                        .to_owned(),
                },
            }
        })
        .collect()
}

/// Scores a resume against a blueprint.
pub fn score(input: &Input<'_>) -> Report {
    let Input {
        document: original,
        blueprint,
        gazetteer: gz,
        as_of,
    } = *input;
    let seen = visible::visible(original);
    let doc = &seen;
    let resume = &atsgate_resume::parse(doc);
    let units = units::units(resume);
    let required = skills::score_skills(
        &blueprint.required_skills,
        Requirement::Required,
        gz,
        doc,
        &units,
    );
    let preferred = skills::score_skills(
        &blueprint.preferred_skills,
        Requirement::Preferred,
        gz,
        doc,
        &units,
    );
    let title = title::score(doc, resume, blueprint, gz.taxonomy());
    let relevant: BTreeSet<units::EntryId> = required
        .iter()
        .chain(&preferred)
        .filter_map(|r| r.entry)
        .chain(title.matched_entries.iter().copied())
        .collect();
    let min_years = blueprint.experience.min_years;
    let exp = experience::score(doc, resume, &relevant, min_years, as_of);
    let experience_met = exp.relevant_years > 0.0 && exp.relevant_years >= min_years;
    let edu = education::score(doc, resume, blueprint, experience_met);
    let facts = knockouts::Facts {
        relevant_years: exp.relevant_years,
        dated_roles: exp.dated_roles,
        highest_degree: edu.highest_level,
    };
    let skill_evidence =
        |rs: &[SkillResult]| rs.iter().filter_map(|r| r.evidence.clone()).collect();
    let dimensions = Dimensions {
        required_skills: Dimension {
            score: skills::dimension(&required),
            evidence: skill_evidence(&required),
        },
        preferred_skills: Dimension {
            score: skills::dimension(&preferred),
            evidence: skill_evidence(&preferred),
        },
        title: Dimension {
            score: Some(title.score),
            evidence: title.evidence,
        },
        experience: Dimension {
            score: Some(exp.score),
            evidence: exp.evidence,
        },
        education: Dimension {
            score: edu.score,
            evidence: edu.evidence,
        },
    };
    let parse_penalties = parse::penalties(doc, resume);
    let all_skills: Vec<SkillResult> = required.into_iter().chain(preferred).collect();
    Report {
        label: SIMULATED_LABEL.to_owned(),
        as_of: as_of.to_string(),
        blueprint: BlueprintInfo {
            id: blueprint.id.clone(),
            version: blueprint.version.clone(),
            title: blueprint.title.clone(),
            source: blueprint.provenance.source,
            reviewed: blueprint.provenance.reviewed,
        },
        vendors: vendor_results(&dimensions, &parse_penalties),
        dimensions,
        relevant_years: exp.relevant_years,
        gaps: gaps(&all_skills, blueprint, gz),
        skills: all_skills,
        knockouts: knockouts::evaluate_all(&blueprint.knockouts, facts, doc, &units),
        parse_penalties,
        integrity_flags: parse::integrity_flags(original),
        quantification: parse::quantification(resume),
    }
}
