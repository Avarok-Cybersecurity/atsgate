//! Generates every committed fixture under `fixtures/` from one synthetic resume.
//!
//! Pure: returns (relative path, bytes) pairs; the caller writes or checks them.

mod content;
mod docx_layouts;
mod docx_package;
mod docx_xml;
mod pdf_layouts;
mod pdf_variants;
mod pdf_writer;
mod png;

use docx_layouts as d;
use pdf_layouts as p;
use pdf_variants as v;

type Gen<T> = (&'static str, fn() -> T);

const PDFS: [Gen<pdf_writer::PdfSpec>; 13] = [
    ("one-column", p::one_column),
    ("two-column", p::two_column),
    ("table", p::table),
    ("text-box", p::text_box),
    ("header-footer", p::header_footer),
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
    Ok(out)
}
