//! Title and seniority T = best title match over the resume's roles × (1 − |Δlevel| / L).
//!
//! A role title matches a blueprint title (its `title`, `title_aliases` and
//! `experience.relevant_titles`), with seniority words removed from both:
//! - 1.0 the same words;
//! - 0.85 the role title resolves to the blueprint's O*NET occupation (a taxonomy alias), with
//!   seniority words and technology qualifiers removed and ambiguous titles read in the
//!   blueprint's context ("Senior Rust Systems Engineer" is a Software Developer);
//! - 0.5 it resolves to an occupation in the same SOC broad group (`15-125x`: the parent of the
//!   detailed occupation, as "Software and Web Developers, Programmers, and Testers" is of
//!   Software Developers);
//! - 0 otherwise.
//!
//! L is the span of the seniority ladder (intern to executive). A role whose title states no
//! seniority is not penalized for it, and neither is any role when the blueprint's seniority is
//! `unspecified`; the evidence says which.
//!
//! Under-seniority costs the full |Δlevel| / L. Over-seniority costs [`OVER_SENIORITY_FACTOR`]
//! (one half) of it: a role above the posting's level has shown every responsibility that
//! level implies, so the remaining question (will they take the level, at its pay) is a
//! recruiter's judgement rather than a parse or keyword risk, while a role below it has not
//! shown the level at all. It still costs something, because screeners do filter on level.

use crate::evidence::Evidence;
use crate::units::EntryId;
use atsgate_blueprint::Blueprint;
use atsgate_blueprint::Seniority;
use atsgate_blueprint::generate::{resolve_occupation, seniority_of, strip_seniority};
use atsgate_extract::Document;
use atsgate_resume::{Resume, SectionKind};
use atsgate_taxonomy::{ONET_SOC_PREFIX, Taxonomy, fold};

pub const T_SAME: f64 = 1.0;
pub const T_OCCUPATION: f64 = 0.85;
pub const T_BROAD_GROUP: f64 = 0.5;

/// The rung of a seniority on the ladder, intern (0) to executive (9); `None` for unspecified.
fn rung(s: Seniority) -> Option<usize> {
    Some(match s {
        Seniority::Intern => 0,
        Seniority::Junior => 1,
        Seniority::Mid => 2,
        Seniority::Senior => 3,
        Seniority::Staff => 4,
        Seniority::Principal => 5,
        Seniority::Lead => 6,
        Seniority::Manager => 7,
        Seniority::Director => 8,
        Seniority::Executive => 9,
        Seniority::Unspecified => return None,
    })
}

/// L: the largest possible |Δlevel| (executive minus intern).
pub const LADDER_SPAN: f64 = 9.0;

/// The share of the |Δlevel| / L penalty charged when the role is above the wanted level.
pub const OVER_SENIORITY_FACTOR: f64 = 0.5;

/// The seniority factor of a role at `have` for a blueprint wanting `want`.
pub fn seniority_factor(want: usize, have: usize) -> f64 {
    let steps = want.abs_diff(have) as f64 / LADDER_SPAN;
    if have > want {
        1.0 - OVER_SENIORITY_FACTOR * steps
    } else {
        1.0 - steps
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleScore {
    pub score: f64,
    pub evidence: Vec<Evidence>,
    /// Roles whose title names the blueprint's occupation (a match of at least 0.85); they count
    /// as relevant experience. A broad-group match moves T but not the years.
    pub matched_entries: Vec<EntryId>,
}

/// The O*NET-SOC code of `title`, read in the context of the blueprint's occupation (see
/// [`resolve_occupation`]).
fn onet_code(taxonomy: &Taxonomy, title: &str, blueprint: &Blueprint) -> Option<String> {
    let r = resolve_occupation(title, taxonomy, Some(&blueprint.occupational_category))?;
    r.taxonomy_ref
        .strip_prefix(ONET_SOC_PREFIX)
        .map(str::to_owned)
}

/// How well one role title matches the blueprint, before seniority.
fn title_match(role: &str, blueprint: &Blueprint, taxonomy: &Taxonomy) -> (f64, &'static str) {
    let core = strip_seniority(&fold(role));
    if core.is_empty() {
        return (0.0, "title.none");
    }
    let wanted = std::iter::once(&blueprint.title)
        .chain(&blueprint.title_aliases)
        .chain(&blueprint.experience.relevant_titles);
    if wanted.clone().any(|t| strip_seniority(&fold(t)) == core) {
        return (T_SAME, "title.same");
    }
    let code = onet_code(taxonomy, role, blueprint);
    match code {
        Some(c) if c == blueprint.occupational_category => (T_OCCUPATION, "title.occupation"),
        Some(c) if c.get(..6) == blueprint.occupational_category.get(..6) => {
            (T_BROAD_GROUP, "title.broad_group")
        }
        _ => (0.0, "title.none"),
    }
}

pub fn score(
    doc: &Document,
    resume: &Resume,
    blueprint: &Blueprint,
    taxonomy: &Taxonomy,
) -> TitleScore {
    let want = rung(blueprint.seniority);
    let mut best: Option<(f64, Evidence)> = None;
    let mut matched_entries = Vec::new();
    for (si, section) in resume.sections.iter().enumerate() {
        if section.kind != SectionKind::Experience {
            continue;
        }
        for (ei, entry) in section.entries.iter().enumerate() {
            let (m, rule) = title_match(&entry.title, blueprint, taxonomy);
            if m == 0.0 {
                continue;
            }
            if m >= T_OCCUPATION {
                matched_entries.push((si, ei));
            }
            let level = seniority_of(&fold(&entry.title));
            let (factor, note) = match (want, level.and_then(rung)) {
                (None, _) => (
                    1.0,
                    format!(
                        "{} (the blueprint states no seniority; not penalized)",
                        entry.title
                    ),
                ),
                (Some(w), Some(have)) => (
                    seniority_factor(w, have),
                    format!(
                        "{} (seniority {:?}, wanted {:?})",
                        entry.title,
                        level.expect("rung came from a level"),
                        blueprint.seniority
                    ),
                ),
                (Some(_), None) => (
                    1.0,
                    format!("{} (no seniority stated; not penalized)", entry.title),
                ),
            };
            let evidence = match entry.evidence.first() {
                Some(at) => Evidence::at_line(rule, doc, at, note),
                None => Evidence::note(rule, note),
            };
            if best.as_ref().is_none_or(|(s, _)| m * factor > *s) {
                best = Some((m * factor, evidence));
            }
        }
    }
    match best {
        Some((score, evidence)) => TitleScore {
            score,
            evidence: vec![evidence],
            matched_entries,
        },
        None => TitleScore {
            score: 0.0,
            evidence: vec![Evidence::note(
                "title.none",
                "no role title matches the blueprint",
            )],
            matched_entries,
        },
    }
}
