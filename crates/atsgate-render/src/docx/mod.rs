//! The resume DOCX: a minimal WordprocessingML package written with `zip` and escaped by
//! `quick-xml`. Stored entries with a fixed timestamp, so the same blocks give the same bytes.

mod xml;

use crate::blocks::Block;
use crate::error::RenderError;
use std::io::Write;

pub fn render(blocks: &[Block]) -> Result<Vec<u8>, RenderError> {
    let err = |e: &dyn std::fmt::Display| RenderError::Docx(e.to_string());
    let document = xml::document(blocks);
    let parts: [(&str, &str); 3] = [
        ("[Content_Types].xml", xml::CONTENT_TYPES),
        ("_rels/.rels", xml::ROOT_RELS),
        ("word/document.xml", &document),
    ];
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .last_modified_time(zip::DateTime::default());
    for (name, body) in parts {
        zip.start_file(name, options).map_err(|e| err(&e))?;
        zip.write_all(body.as_bytes()).map_err(|e| err(&e))?;
    }
    Ok(zip.finish().map_err(|e| err(&e))?.into_inner())
}
