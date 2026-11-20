//! Education D: degree level and field against the blueprint.
//!
//! Only each education entry's title ("B.S. Computer Science") is read. The institution and the
//! dates are never read, so neither school prestige nor graduation year can move a score.
//!
//! - Level: 1.0 at or above `min_level`; a lower degree earns level / min_level; none earns 0.
//! - Field: 1.0 when a title names one of `fields`, else 0.
//! - D is the mean of the parts the blueprint asks for, and `null` when it asks for neither.
//! - `experience_in_place_of_education`: relevant years at or above the blueprint minimum (and
//!   above zero) satisfy D in full.
//!
//! Levels are ISCED 2011. EQF levels 5-8 equal ISCED 5-8; EQF 1-4 map to ISCED 1, 2, 3, 3.

use crate::evidence::Evidence;
use atsgate_blueprint::generate::DEGREES;
use atsgate_blueprint::{Blueprint, EducationLevel, LevelFramework};
use atsgate_extract::Document;
use atsgate_resume::{Resume, SectionKind};
use atsgate_taxonomy::{fold, tokens};

pub fn isced(level: EducationLevel) -> u8 {
    match (level.framework, level.level) {
        (LevelFramework::Isced, l) => l,
        (LevelFramework::Eqf, 4) => 3,
        (LevelFramework::Eqf, l) => l,
    }
}

/// The ISCED level a degree title names. A context word ("BS", "Master") counts when it opens
/// the title, as resumes write it ("B.S. Physics", "Master of Science").
pub fn degree_level(title: &str) -> Option<u8> {
    let words: Vec<String> = tokens(title).into_iter().map(|t| t.folded).collect();
    words
        .iter()
        .enumerate()
        .filter_map(|(i, w)| {
            let (_, level, needs_context) = DEGREES.iter().find(|(d, _, _)| d == w)?;
            let in_context = i == 0 || words.get(i + 1).is_some_and(|n| n == "in" || n == "of");
            (!needs_context || in_context).then_some(*level)
        })
        .max()
}

#[derive(Debug, Clone, PartialEq)]
pub struct EducationScore {
    pub score: Option<f64>,
    /// The highest degree level the resume names, for the degree knockout.
    pub highest_level: Option<u8>,
    pub evidence: Vec<Evidence>,
}

fn contains_phrase(haystack: &str, phrase: &str) -> bool {
    format!(" {} ", fold(haystack)).contains(&format!(" {} ", fold(phrase)))
}

pub fn score(
    doc: &Document,
    resume: &Resume,
    blueprint: &Blueprint,
    experience_met: bool,
) -> EducationScore {
    let req = &blueprint.education;
    let entries = resume
        .sections
        .iter()
        .filter(|s| s.kind == SectionKind::Education)
        .flat_map(|s| &s.entries);
    let mut evidence = Vec::new();
    let mut highest: Option<u8> = None;
    let mut field_met = false;
    for e in entries {
        let level = degree_level(&e.title);
        let field = req.fields.iter().find(|f| contains_phrase(&e.title, f));
        if level.is_none() && field.is_none() {
            continue;
        }
        highest = highest.max(level);
        field_met |= field.is_some();
        let text = format!("{} (level {level:?}, field {field:?})", e.title);
        evidence.push(match e.evidence.first() {
            Some(at) => Evidence::at_line("education.entry", doc, at, text),
            None => Evidence::note("education.entry", text),
        });
    }
    let level_part = req.min_level.map(|min| match highest {
        Some(l) if l >= isced(min) => 1.0,
        Some(l) => f64::from(l) / f64::from(isced(min)),
        None => 0.0,
    });
    let field_part = (!req.fields.is_empty()).then_some(if field_met { 1.0 } else { 0.0 });
    let parts: Vec<f64> = level_part.into_iter().chain(field_part).collect();
    let mut score = (!parts.is_empty()).then(|| parts.iter().sum::<f64>() / parts.len() as f64);
    if req.experience_in_place_of_education && experience_met && score.is_some_and(|s| s < 1.0) {
        evidence.push(Evidence::note(
            "education.experience_in_place",
            "relevant experience meets the minimum, which the blueprint accepts in place of education",
        ));
        score = Some(1.0);
    }
    EducationScore {
        score,
        highest_level: highest,
        evidence,
    }
}
