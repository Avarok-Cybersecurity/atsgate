//! `--blueprint` specs: `builtin:<id>`, a file, an `https://` URL, or `generate:<path-or-url>`.
//! Fetched and read bytes come through the [`Io`] seam; parsing and validation are the
//! blueprint crate's.

use crate::io::Io;
use atsgate_blueprint::generate::{GenerateOptions, generate};
use atsgate_blueprint::{Blueprint, builtin};
use atsgate_extract::{Format, Thresholds, extract};
use atsgate_taxonomy::{Gazetteer, Scope, Taxonomy};
use std::path::Path;

/// The largest blueprint accepted from a URL: a blueprint is a few kilobytes.
pub const MAX_BLUEPRINT_BYTES: u64 = 1024 * 1024;
/// The largest job description accepted for `generate:`: a PDF posting with images.
pub const MAX_POSTING_BYTES: u64 = 16 * 1024 * 1024;
/// `version` of a blueprint generated on the fly: a first draft, as `xtask examples` writes.
pub const GENERATED_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Spec<'a> {
    Builtin(&'a str),
    Url(&'a str),
    Generate(&'a str),
    File(&'a Path),
}

pub fn parse(spec: &str) -> Result<Spec<'_>, String> {
    if let Some(id) = spec.strip_prefix("builtin:") {
        return Ok(Spec::Builtin(id));
    }
    if let Some(source) = spec.strip_prefix("generate:") {
        return (!source.is_empty())
            .then_some(Spec::Generate(source))
            .ok_or_else(|| "generate: needs a path or an https:// URL".to_owned());
    }
    if spec.starts_with("http://") {
        return Err(format!("{spec}: only https:// URLs are fetched"));
    }
    if spec.starts_with("https://") {
        return Ok(Spec::Url(spec));
    }
    Ok(Spec::File(Path::new(spec)))
}

#[derive(Debug, Clone, PartialEq)]
pub struct Resolved {
    pub blueprint: Blueprint,
    /// For `generate:`, the generated blueprint as YAML, to be written out for review.
    pub generated_yaml: Option<String>,
}

/// A blueprint document: JSON when it opens with `{`, YAML otherwise (both are validated).
fn parse_blueprint(bytes: &[u8], origin: &str) -> Result<Blueprint, String> {
    let text = std::str::from_utf8(bytes).map_err(|e| format!("{origin}: {e}"))?;
    let parsed = if text.trim_start().starts_with('{') {
        Blueprint::from_json(text)
    } else {
        Blueprint::from_yaml(text)
    };
    parsed.map_err(|e| format!("{origin}: {e}"))
}

/// The format of a job description, from its leading bytes.
pub fn sniff(bytes: &[u8]) -> Format {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(512)]).to_ascii_lowercase();
    if bytes.starts_with(b"%PDF-") {
        Format::Pdf
    } else if bytes.starts_with(b"PK\x03\x04") {
        Format::Docx
    } else if head.trim_start().starts_with('<') || head.contains("<html") {
        Format::Html
    } else {
        Format::Text
    }
}

/// A blueprint id from a source name: "jobs/ArtCraft HN.txt" -> "artcraft-hn".
pub fn id_from(source: &str) -> Result<String, String> {
    let stem = source.rsplit(['/', '\\']).next().unwrap_or(source);
    let stem = stem.split(['?', '#']).next().unwrap_or(stem);
    let stem = stem.rsplit_once('.').map_or(stem, |(s, _)| s);
    let mut id = String::new();
    for c in stem.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            id.push(c);
        } else if !id.ends_with('-') && !id.is_empty() {
            id.push('-');
        }
    }
    let id = id.trim_end_matches('-').to_owned();
    (!id.is_empty())
        .then_some(id)
        .ok_or_else(|| format!("generate:{source}: the name gives no blueprint id"))
}

pub fn resolve(spec: &str, io: &dyn Io, taxonomy: &Taxonomy) -> Result<Resolved, String> {
    let plain = |blueprint| {
        Ok(Resolved {
            blueprint,
            generated_yaml: None,
        })
    };
    match parse(spec)? {
        Spec::Builtin(id) => plain(builtin(id).map_err(|e| e.to_string())?),
        Spec::Url(url) => plain(parse_blueprint(&io.fetch(url, MAX_BLUEPRINT_BYTES)?, url)?),
        Spec::File(path) => plain(parse_blueprint(
            &io.read(path)?,
            &path.display().to_string(),
        )?),
        Spec::Generate(source) => {
            let bytes = match parse(source)? {
                Spec::Url(url) => io.fetch(url, MAX_POSTING_BYTES)?,
                Spec::File(path) => io.read(path)?,
                _ => {
                    return Err(format!(
                        "generate:{source}: expected a path or an https:// URL"
                    ));
                }
            };
            let doc = extract(&bytes, sniff(&bytes), &Thresholds::STANDARD)
                .map_err(|e| format!("{source}: {e}"))?;
            let options = GenerateOptions {
                id: id_from(source)?,
                version: GENERATED_VERSION.to_owned(),
                occupational_category: None,
            };
            let postings = Gazetteer::new(taxonomy, Scope::Postings);
            let blueprint = generate(&doc.reading_text(), &options, &postings)
                .map_err(|e| format!("{source}: {e}"))?;
            let yaml = blueprint.to_yaml().map_err(|e| e.to_string())?;
            Ok(Resolved {
                blueprint,
                generated_yaml: Some(yaml),
            })
        }
    }
}

#[cfg(test)]
#[path = "spec_tests.rs"]
mod tests;
