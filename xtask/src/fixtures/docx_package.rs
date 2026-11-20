//! Packages fixture WordprocessingML into a deterministic .docx (OPC zip) file.

use super::docx_xml::{Block, NAMESPACES, blocks, escape};
use super::png;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct DocxSpec {
    pub title: String,
    pub keywords: Option<String>,
    pub body: Vec<Block>,
    pub header: Option<Vec<Block>>,
    pub footer: Option<Vec<Block>>,
    /// Number of newspaper columns in the section (`w:cols`).
    pub columns: u32,
}

fn uses_image(blocks: &[Block]) -> bool {
    blocks.iter().any(|b| matches!(b, Block::Image))
}

fn document(spec: &DocxSpec) -> String {
    let mut sect = String::from("<w:sectPr>");
    if spec.header.is_some() {
        sect.push_str("<w:headerReference w:type=\"default\" r:id=\"rIdHeader\"/>");
    }
    if spec.footer.is_some() {
        sect.push_str("<w:footerReference w:type=\"default\" r:id=\"rIdFooter\"/>");
    }
    sect.push_str("<w:pgSz w:w=\"12240\" w:h=\"15840\"/><w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" w:header=\"720\" w:footer=\"720\" w:gutter=\"0\"/>");
    sect.push_str(&format!(
        "<w:cols w:num=\"{}\" w:space=\"720\"/></w:sectPr>",
        spec.columns
    ));
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:document {NAMESPACES}><w:body>{}{sect}</w:body></w:document>",
        blocks(&spec.body)
    )
}

fn part(root: &str, inner: &[Block]) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:{root} {NAMESPACES}>{}</w:{root}>",
        blocks(inner)
    )
}

fn content_types(spec: &DocxSpec) -> String {
    let mut s = String::from(concat!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n",
        "<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">",
        "<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>",
        "<Default Extension=\"xml\" ContentType=\"application/xml\"/>",
        "<Default Extension=\"png\" ContentType=\"image/png\"/>",
        "<Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>",
        "<Override PartName=\"/docProps/core.xml\" ContentType=\"application/vnd.openxmlformats-package.core-properties+xml\"/>",
    ));
    if spec.header.is_some() {
        s.push_str("<Override PartName=\"/word/header1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml\"/>");
    }
    if spec.footer.is_some() {
        s.push_str("<Override PartName=\"/word/footer1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml\"/>");
    }
    s.push_str("</Types>");
    s
}

const ROOT_RELS: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n",
    "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
    "<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/>",
    "<Relationship Id=\"rId2\" Type=\"http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties\" Target=\"docProps/core.xml\"/>",
    "</Relationships>",
);

fn document_rels(spec: &DocxSpec) -> String {
    let base = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    let mut s = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
    );
    if spec.header.is_some() {
        s.push_str(&format!(
            "<Relationship Id=\"rIdHeader\" Type=\"{base}/header\" Target=\"header1.xml\"/>"
        ));
    }
    if spec.footer.is_some() {
        s.push_str(&format!(
            "<Relationship Id=\"rIdFooter\" Type=\"{base}/footer\" Target=\"footer1.xml\"/>"
        ));
    }
    if uses_image(&spec.body) {
        s.push_str(&format!(
            "<Relationship Id=\"rIdImg\" Type=\"{base}/image\" Target=\"media/scan.png\"/>"
        ));
    }
    s.push_str("</Relationships>");
    s
}

fn core(spec: &DocxSpec) -> String {
    let keywords = spec
        .keywords
        .as_deref()
        .map(|k| format!("<cp:keywords>{}</cp:keywords>", escape(k)));
    format!(
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n",
            "<cp:coreProperties xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\" ",
            "xmlns:dc=\"http://purl.org/dc/elements/1.1/\"><dc:title>{}</dc:title>{}</cp:coreProperties>",
        ),
        escape(&spec.title),
        keywords.unwrap_or_default()
    )
}

pub fn build(spec: &DocxSpec) -> Result<Vec<u8>, String> {
    let mut parts: Vec<(&str, Vec<u8>)> = vec![
        ("[Content_Types].xml", content_types(spec).into_bytes()),
        ("_rels/.rels", ROOT_RELS.as_bytes().to_vec()),
        ("docProps/core.xml", core(spec).into_bytes()),
        ("word/document.xml", document(spec).into_bytes()),
        (
            "word/_rels/document.xml.rels",
            document_rels(spec).into_bytes(),
        ),
    ];
    if let Some(h) = &spec.header {
        parts.push(("word/header1.xml", part("hdr", h).into_bytes()));
    }
    if let Some(f) = &spec.footer {
        parts.push(("word/footer1.xml", part("ftr", f).into_bytes()));
    }
    if uses_image(&spec.body) {
        parts.push(("word/media/scan.png", png::scan()));
    }
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .last_modified_time(zip::DateTime::default());
    for (name, bytes) in parts {
        zip.start_file(name, options).map_err(|e| e.to_string())?;
        zip.write_all(&bytes).map_err(|e| e.to_string())?;
    }
    Ok(zip.finish().map_err(|e| e.to_string())?.into_inner())
}
