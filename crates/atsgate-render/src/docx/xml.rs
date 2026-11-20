//! WordprocessingML for the resume: one paragraph per block, direct formatting only, no tables,
//! text boxes, headers, footers or fields, so every parser reads the body in order.

use crate::blocks::{BULLET, Block};
use quick_xml::escape::escape;

const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

/// Font sizes in half-points, matching the PDF template.
const NAME_SIZE: u32 = 36;
const TITLE_SIZE: u32 = 23;
const HEADING_SIZE: u32 = 25;
const BODY_SIZE: u32 = 21;

/// US Letter with 0.75 in side margins and 0.85 in top and bottom margins, in twips.
const SECTION: &str = r#"<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1224" w:right="1080" w:bottom="1224" w:left="1080" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr>"#;

fn run(text: &str, bold: bool, size: u32) -> String {
    let b = if bold { "<w:b/>" } else { "" };
    format!(
        r#"<w:r><w:rPr>{b}<w:sz w:val="{size}"/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r>"#,
        escape(text)
    )
}

/// A paragraph: spacing before it in twips, an optional hanging indent, then its runs.
fn paragraph(before: u32, hanging: Option<(u32, u32)>, runs: &[String]) -> String {
    let indent = hanging.map_or_else(String::new, |(left, hang)| {
        format!(r#"<w:ind w:left="{left}" w:hanging="{hang}"/>"#)
    });
    format!(
        r#"<w:p><w:pPr><w:spacing w:before="{before}" w:after="0"/>{indent}</w:pPr>{}</w:p>"#,
        runs.concat()
    )
}

fn block(b: &Block) -> String {
    match b {
        Block::Name { text } => paragraph(0, None, &[run(text, true, NAME_SIZE)]),
        Block::Title { text } => paragraph(40, None, &[run(text, false, TITLE_SIZE)]),
        Block::Contact { text } => paragraph(40, None, &[run(text, false, BODY_SIZE)]),
        Block::Heading { text } => paragraph(240, None, &[run(text, true, HEADING_SIZE)]),
        Block::Paragraph { text } => paragraph(80, None, &[run(text, false, BODY_SIZE)]),
        Block::Entry { title, rest } => paragraph(
            140,
            None,
            &[run(title, true, BODY_SIZE), run(rest, false, BODY_SIZE)],
        ),
        Block::Bullet { text } => paragraph(
            60,
            Some((360, 200)),
            &[run(&format!("{BULLET}{text}"), false, BODY_SIZE)],
        ),
        Block::Labeled { label, text } => paragraph(
            80,
            Some((240, 240)),
            &[
                run(&format!("{label}:"), true, BODY_SIZE),
                run(&format!(" {text}"), false, BODY_SIZE),
            ],
        ),
    }
}

pub fn document(blocks: &[Block]) -> String {
    let body: String = blocks.iter().map(block).collect();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="{W_NS}"><w:body>{body}{SECTION}</w:body></w:document>"#
    )
}

pub const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;

pub const ROOT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
