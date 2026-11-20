//! The resume PDF, typeset by Typst embedded as a library.

mod world;

use crate::blocks::{BULLET, Block};
use crate::error::RenderError;
use typst::foundations::Smart;
use typst_layout::PagedDocument;
use typst_pdf::PdfOptions;
use world::ResumeWorld;

const TEMPLATE: &str = include_str!("resume.typ");

/// What the template reads from `/resume.json`.
#[derive(serde::Serialize)]
struct Data<'a> {
    name: &'a str,
    bullet: &'a str,
    blocks: &'a [Block],
}

fn messages<T: std::fmt::Debug>(diagnostics: &[T]) -> String {
    diagnostics
        .iter()
        .map(|d| format!("{d:?}"))
        .collect::<Vec<_>>()
        .join("; ")
}

/// Typesets `blocks` and fails when the result is longer than `max_pages`.
pub fn render(blocks: &[Block], name: &str, max_pages: u32) -> Result<Vec<u8>, RenderError> {
    let data = serde_json::to_vec(&Data {
        name,
        bullet: BULLET,
        blocks,
    })
    .map_err(|e| RenderError::Pdf(e.to_string()))?;
    let world = ResumeWorld::new(TEMPLATE, data);
    let compiled = typst::compile::<PagedDocument>(&world);
    if !compiled.warnings.is_empty() {
        return Err(RenderError::Pdf(messages(&compiled.warnings)));
    }
    let document = compiled
        .output
        .map_err(|errors| RenderError::Pdf(messages(&errors)))?;
    let pages = document.pages().len();
    if pages > max_pages as usize {
        return Err(RenderError::TooManyPages { pages, max_pages });
    }
    let options = PdfOptions {
        ident: Smart::Custom("atsgate-resume".to_owned()),
        creator: Smart::Custom(Some("ATSGate".to_owned())),
        timestamp: None,
        ..PdfOptions::default()
    };
    typst_pdf::pdf(&document, &options).map_err(|errors| RenderError::Pdf(messages(&errors)))
}
