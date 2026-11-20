//! Generates every committed fixture under `fixtures/` from one synthetic resume.
//!
//! Pure: returns (relative path, bytes) pairs; the caller writes or checks them.

mod content;
mod docx_layouts;
mod docx_package;
mod docx_xml;
mod pdf_layouts;
mod pdf_pages;
mod pdf_variants;
mod pdf_writer;
mod png;

use docx_layouts as d;
use pdf_layouts as p;
use pdf_pages as pages;
use pdf_variants as v;

type Gen<T> = (&'static str, fn() -> T);

const PDFS: [Gen<pdf_writer::PdfSpec>; 15] = [
    ("one-column", p::one_column),
    ("two-column", p::two_column),
    ("table", p::table),
    ("text-box", p::text_box),
    ("header-footer", pages::header_footer),
    ("page-top-heading", pages::page_top_heading),
    ("browser-print", pages::browser_print),
    ("image-only", p::image_only),
    ("hidden-white", v::hidden_white),
    ("hidden-tiny", v::hidden_tiny),
    ("off-page", v::off_page),
    ("metadata-only", v::metadata_only),
    ("zero-width", v::zero_width),
    ("ligature", v::ligature),
    ("missing-tounicode", v::missing_to_unicode),
];

const DOCXS: [Gen<docx_package::DocxSpec>; 11] = [
    ("one-column", d::one_column),
    ("two-column", d::two_column),
    ("table", d::table),
    ("text-box", d::text_box),
    ("header-footer", d::header_footer),
    ("image-only", d::image_only),
    ("hidden-white", d::hidden_white),
    ("hidden-vanish", d::hidden_vanish),
    ("hidden-tiny", d::hidden_tiny),
    ("zero-width", d::zero_width),
    ("metadata-only", d::metadata_only),
];

pub fn all() -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut out = Vec::new();
    for (name, make) in PDFS {
        out.push((
            format!("fixtures/pdf/{name}.pdf"),
            pdf_writer::build(&make())?,
        ));
    }
    for (name, make) in DOCXS {
        out.push((
            format!("fixtures/docx/{name}.docx"),
            docx_package::build(&make())?,
        ));
    }
    out.push((
        "fixtures/import/tbraun96.career.json".to_owned(),
        imported_tbraun96()?.into_bytes(),
    ));
    Ok(out)
}

/// The golden import: the committed copy of tbraun96's `content/career.json`, converted.
fn imported_tbraun96() -> Result<String, String> {
    const SOURCE: &str = include_str!("../../../fixtures/import/tbraun96-career.json");
    let career = atsgate_career::import(SOURCE, atsgate_career::ImportFormat::Tbraun96)
        .map_err(|e| format!("importing the tbraun96 fixture: {e}"))?;
    career.to_json().map_err(|e| e.to_string())
}
