//! Static-site output: HTML partials for marker injection, and schema.org JSON-LD.

mod html;
mod inject;
mod jsonld;

pub use inject::inject;
pub use jsonld::person as jsonld;

use crate::error::RenderError;
use crate::model::{Body, ResumeDocument};
use atsgate_resume::SectionKind;
use std::collections::BTreeMap;

/// The partials a site can inject, by marker name, and the section each one shows. A section the
/// tailoring left empty still yields its (empty) partial, so a page's markers always resolve.
pub const PARTIALS: [(&str, SectionKind); 4] = [
    ("experience", SectionKind::Experience),
    ("projects", SectionKind::Projects),
    ("skills", SectionKind::Skills),
    ("education", SectionKind::Education),
];

pub fn partials(doc: &ResumeDocument) -> Result<BTreeMap<String, String>, RenderError> {
    let body = |kind: SectionKind| {
        doc.sections
            .iter()
            .find(|s| s.kind == kind)
            .map(|s| &s.body)
    };
    let mut out = BTreeMap::new();
    for (name, kind) in PARTIALS {
        let fragment = match body(kind) {
            Some(Body::Groups(groups)) => html::skills(groups),
            Some(Body::Entries(list)) => html::entries(name, list, kind == SectionKind::Projects)?,
            Some(Body::Text(_)) | None if kind == SectionKind::Skills => html::skills(&[]),
            Some(Body::Text(_)) | None => html::entries(name, &[], false)?,
        };
        out.insert(name.to_owned(), fragment);
    }
    Ok(out)
}
