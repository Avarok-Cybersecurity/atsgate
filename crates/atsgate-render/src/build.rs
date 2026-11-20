//! One render, end to end: a career, a tailoring and a contact profile in; every output file
//! out, each PDF and DOCX already proven to read back as rendered, plus a manifest of hashes.
//! Pure: the caller writes the files.

use crate::blocks::blocks;
use crate::contact::ContactProfile;
use crate::error::RenderError;
use crate::{docx, model, pdf, site, verify};
use atsgate_career::{Career, Tailoring};
use atsgate_extract::Format;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write;

pub const PDF_FILE: &str = "resume.pdf";
pub const DOCX_FILE: &str = "resume.docx";
pub const JSONLD_FILE: &str = "partials/jsonld.json";
pub const MANIFEST_FILE: &str = "manifest.json";

/// The rendered files by path relative to the output directory, `manifest.json` included.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifacts {
    pub files: BTreeMap<String, Vec<u8>>,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut out, b| {
            let _ = write!(out, "{b:02x}");
            out
        })
}

/// `{"files": {"resume.pdf": "<sha256>", ...}}`, in path order.
fn manifest(files: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>, RenderError> {
    let hashes: BTreeMap<&str, String> = files
        .iter()
        .map(|(p, b)| (p.as_str(), sha256_hex(b)))
        .collect();
    let mut text = serde_json::to_string_pretty(&serde_json::json!({ "files": hashes }))
        .map_err(|e| RenderError::Json(e.to_string()))?;
    text.push('\n');
    Ok(text.into_bytes())
}

pub fn build(
    career: &Career,
    tailoring: &Tailoring,
    profile: &ContactProfile,
) -> Result<Artifacts, RenderError> {
    let tailored = tailoring.check(career)?;
    let doc = model::document(&tailored, profile)?;
    let layout = blocks(&doc);
    let pdf = pdf::render(&layout, &doc.name, doc.max_pages)?;
    verify::round_trip(&doc, &pdf, Format::Pdf)?;
    let docx = docx::render(&layout)?;
    verify::round_trip(&doc, &docx, Format::Docx)?;
    let mut files = BTreeMap::from([(PDF_FILE.to_owned(), pdf), (DOCX_FILE.to_owned(), docx)]);
    for (name, html) in site::partials(&doc)? {
        files.insert(format!("partials/{name}.html"), html.into_bytes());
    }
    let jsonld = site::jsonld(&doc, &career.basics, profile)?;
    files.insert(JSONLD_FILE.to_owned(), jsonld.into_bytes());
    let manifest = manifest(&files)?;
    files.insert(MANIFEST_FILE.to_owned(), manifest);
    Ok(Artifacts { files })
}
