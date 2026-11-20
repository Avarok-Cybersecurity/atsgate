//! Whether text found only in document metadata is an integrity flag.
//!
//! Every PDF and DOCX carries a Title or Subject that may not be printed on the page ("resume",
//! a file name, the author's name), and an ATS does not score it. Metadata matters when it holds
//! content a reader is not shown: a blueprint skill or keyword that the page lacks, or about a
//! paragraph of words. Only then is the extractor's metadata-only signal raised as a flag.

use crate::skills::unit_matches;
use crate::units::{Context, Unit};
use atsgate_blueprint::Blueprint;
use atsgate_extract::{Document, Signal, SignalKind};
use atsgate_taxonomy::{Gazetteer, fold};

/// Metadata-only words from this many on are flagged whatever they say: a title or author line
/// runs to a handful of words, so a dozen words the page never shows is hidden prose.
pub const METADATA_LONG_TERMS: usize = 12;

/// Separators of a keywords-style field ("Rust, Go; Kubernetes").
const FIELD_SEPARATORS: [char; 3] = [',', ';', '|'];

fn terms(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Whether the metadata-only `signal` of `doc` carries content (see the module docs).
pub fn carries_content(
    doc: &Document,
    signal: &Signal,
    blueprint: &Blueprint,
    gz: &Gazetteer<'_>,
) -> bool {
    let SignalKind::MetadataOnlyText { field } = signal.kind else {
        return false;
    };
    let absent: Vec<String> = signal
        .evidence
        .iter()
        .flat_map(|e| terms(&e.text))
        .collect();
    if absent.len() >= METADATA_LONG_TERMS {
        return true;
    }
    let value = doc
        .metadata
        .iter()
        .filter(|m| m.name == field)
        .map(|m| m.value.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let hides = |surface: &str| terms(surface).iter().any(|t| absent.contains(t));
    let pieces: Vec<Unit> = value
        .split(FIELD_SEPARATORS)
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|p| Unit {
            text: p.to_owned(),
            context: Context::SkillsList,
            lines: Vec::new(),
            entry: None,
        })
        .collect();
    let skill_hidden = blueprint
        .required_skills
        .iter()
        .chain(&blueprint.preferred_skills)
        .any(|s| {
            pieces.iter().any(|u| {
                unit_matches(s, gz, u)
                    .iter()
                    .any(|(_, surface)| hides(surface))
            })
        });
    let folded = format!(" {} ", fold(&value));
    let keyword_hidden = blueprint
        .domain_keywords
        .iter()
        .any(|k| folded.contains(&format!(" {} ", fold(k))) && hides(k));
    skill_hidden || keyword_hidden
}
