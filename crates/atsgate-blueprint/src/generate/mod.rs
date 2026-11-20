//! The deterministic job description -> blueprint generator.
//!
//! Input is plain text (for a PDF or HTML posting, `atsgate_extract::Document::reading_text`).
//! Skills come from the taxonomy gazetteer; whether each is required or preferred comes from the
//! section it appears in:
//!
//! | Section | Skill goes to |
//! |---|---|
//! | Required cue ("Requirements", "Qualifications", "Stack:") | `required_skills` |
//! | Preferred cue ("Nice to have", "Preferred", "... is a plus") | `preferred_skills` |
//! | Responsibilities | `preferred_skills` if a required cue exists, else `required_skills` |
//! | Body (before any cue) | `domain_keywords` if a required cue exists (never generic office or CAD software), else `required_skills` |
//! | Benefits, compensation, about us | nothing |
//!
//! The output is always `provenance {source: generated, reviewed: false}`. A posting whose title
//! states no seniority gets `seniority: unspecified`, never a guessed level.

mod occupation;
mod requirements;
mod resolve;
mod sections;
mod skills;
mod title;

pub use occupation::{MIN_SHARED_SKILLS, breadth_weighted};
pub use requirements::DEGREES;
pub use resolve::{MAX_QUALIFIER_WORDS, resolve_occupation, strip_qualifiers};
pub use sections::{HEADING_CUES, INLINE_PREFERRED, Section};
pub use skills::{GENERIC_SOFTWARE, GENERIC_SOFTWARE_CATEGORIES, is_generic_software};
pub use title::{GENERIC_TITLES, SENIORITY_WORDS, TITLE_HEADS, seniority_of, strip_seniority};

use crate::{
    Blueprint, BlueprintError, EducationLevel, EducationRequirement, ExperienceRequirement,
    Knockout, KnockoutType, LevelFramework, Provenance, ProvenanceSource, Seniority, UnknownPolicy,
};
use atsgate_taxonomy::{Gazetteer, Scope, Taxonomy};

/// Identifies this generator in `provenance.generator_version`.
pub const GENERATOR_VERSION: &str = concat!("atsgate-blueprint/", env!("CARGO_PKG_VERSION"));

/// Everything the generator cannot read from the text; no field has a default.
#[derive(Debug, Clone, PartialEq)]
pub struct GenerateOptions {
    pub id: String,
    pub version: String,
    /// `Some` overrides the occupation; `None` infers it from the title, then from the skills,
    /// and fails when neither names one.
    pub occupational_category: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum GenerateError {
    #[error("no job title found in the first lines of the description")]
    NoTitle,
    #[error(
        "the title does not name an O*NET occupation and fewer than {MIN_SHARED_SKILLS} required \
         skills match one; pass occupational_category explicitly"
    )]
    UnknownOccupation,
    #[error("generate needs a gazetteer built for postings (Scope::Postings)")]
    ResumeGazetteer,
    #[error("the generated blueprint is invalid: {0}")]
    Invalid(#[from] BlueprintError),
}

/// Generates a candidate blueprint from a job description.
pub fn generate(
    text: &str,
    options: &GenerateOptions,
    gazetteer: &Gazetteer<'_>,
) -> Result<Blueprint, GenerateError> {
    if gazetteer.scope() != Scope::Postings {
        return Err(GenerateError::ResumeGazetteer);
    }
    let taxonomy: &Taxonomy = gazetteer.taxonomy();
    let lines = sections::label(text);
    let leading: Vec<&str> = lines.iter().map(|l| l.text).collect();
    let found = skills::collect(&lines, gazetteer);
    let inferred = occupation::infer(taxonomy, &found.required_ids, &found.preferred_ids);
    let context = options
        .occupational_category
        .clone()
        .or_else(|| inferred.as_ref().map(|(code, _)| code.clone()));
    let guess =
        title::guess(&leading, taxonomy, context.as_deref()).ok_or(GenerateError::NoTitle)?;
    let occupation = match &options.occupational_category {
        Some(code) => code.clone(),
        None => guess
            .onet
            .clone()
            .or(inferred)
            .map(|(code, _)| code)
            .ok_or(GenerateError::UnknownOccupation)?,
    };
    let requirement_lines: Vec<&str> = lines
        .iter()
        .filter(|l| matches!(l.section, Section::Required | Section::Body))
        .map(|l| l.text)
        .collect();
    let min_years = requirement_lines
        .iter()
        .filter_map(|l| requirements::years(l))
        .reduce(f64::max);
    let degree = requirement_lines
        .iter()
        .filter_map(|l| requirements::degree(l).map(|d| (d, *l)))
        .min_by_key(|((level, _), _)| *level);
    let mut knockouts = Vec::new();
    if let Some(y) = min_years
        && lines
            .iter()
            .any(|l| l.section == Section::Required && requirements::years(l.text).is_some())
    {
        knockouts.push(Knockout {
            kind: KnockoutType::MinYears,
            value: format!("{y}"),
            unknown_policy: UnknownPolicy::Review,
        });
    }
    let all_text: Vec<&str> = lines
        .iter()
        .filter(|l| l.section != Section::Ignored)
        .map(|l| l.text)
        .collect();
    let mut certifications: Vec<String> = all_text
        .iter()
        .flat_map(|l| requirements::certifications(l))
        .collect();
    certifications.dedup();
    let mut languages: Vec<String> = all_text
        .iter()
        .flat_map(|l| requirements::languages(l))
        .collect();
    languages.sort();
    languages.dedup();
    let mut relevant_titles = vec![guess.title.clone()];
    relevant_titles.extend(guess.onet.as_ref().map(|(_, t)| t.clone()));
    relevant_titles.dedup();
    let blueprint = Blueprint {
        schema: None,
        id: options.id.clone(),
        version: options.version.clone(),
        title: guess.title.clone(),
        title_aliases: Vec::new(),
        occupational_category: occupation,
        esco_occupation: guess.esco.clone(),
        seniority: guess.seniority.unwrap_or(Seniority::Unspecified),
        required_skills: found.required,
        preferred_skills: found.preferred,
        knockouts,
        education: EducationRequirement {
            min_level: degree.as_ref().map(|((level, _), _)| EducationLevel {
                framework: LevelFramework::Isced,
                level: *level,
            }),
            fields: degree
                .as_ref()
                .map(|((_, f), _)| f.clone())
                .unwrap_or_default(),
            experience_in_place_of_education: degree
                .as_ref()
                .is_some_and(|(_, line)| requirements::equivalent_experience(line)),
        },
        experience: ExperienceRequirement {
            min_years: min_years.unwrap_or(0.0),
            relevant_titles,
        },
        certifications,
        domain_keywords: found.domain_keywords,
        languages,
        provenance: Provenance {
            source: ProvenanceSource::Generated,
            generator_version: Some(GENERATOR_VERSION.to_owned()),
            taxonomy_versions: taxonomy.versions().clone(),
            reviewed: false,
        },
    };
    blueprint.validate()?;
    Ok(blueprint)
}
