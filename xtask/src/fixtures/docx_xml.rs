//! WordprocessingML for fixture documents: paragraphs, tables, text boxes and images.

/// One run of text and its formatting.
#[derive(Debug, Clone, Default)]
pub struct Run {
    pub text: String,
    pub bold: bool,
    /// Font size in half-points (`w:sz`), when set.
    pub half_points: Option<u32>,
    /// Hex RGB (`w:color`), when set.
    pub color: Option<&'static str>,
    pub vanish: bool,
    /// Emit a tab before the text.
    pub tab_before: bool,
    /// Emit a column break before the text.
    pub column_break_before: bool,
}

impl Run {
    pub fn plain(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            ..Self::default()
        }
    }
    pub fn bold(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            bold: true,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone)]
pub enum Block {
    Para(Vec<Run>),
    /// Rows of cells; each cell holds blocks.
    Table(Vec<Vec<Vec<Block>>>),
    /// A floating text box anchored in its own paragraph.
    TextBox(Vec<Block>),
    /// The embedded image `rIdImg`, inline in its own paragraph.
    Image,
}

pub fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn run(r: &Run) -> String {
    let mut props = String::new();
    if r.bold {
        props.push_str("<w:b/>");
    }
    if let Some(c) = r.color {
        props.push_str(&format!("<w:color w:val=\"{c}\"/>"));
    }
    if let Some(sz) = r.half_points {
        props.push_str(&format!("<w:sz w:val=\"{sz}\"/>"));
    }
    if r.vanish {
        props.push_str("<w:vanish/>");
    }
    let rpr = if props.is_empty() {
        String::new()
    } else {
        format!("<w:rPr>{props}</w:rPr>")
    };
    let brk = if r.column_break_before {
        "<w:br w:type=\"column\"/>"
    } else {
        ""
    };
    let tab = if r.tab_before { "<w:tab/>" } else { "" };
    format!(
        "<w:r>{rpr}{brk}{tab}<w:t xml:space=\"preserve\">{}</w:t></w:r>",
        escape(&r.text)
    )
}

const TEXT_BOX_OPEN: &str = concat!(
    "<w:r><w:drawing><wp:anchor distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\" simplePos=\"0\" ",
    "relativeHeight=\"1\" behindDoc=\"0\" locked=\"0\" layoutInCell=\"1\" allowOverlap=\"1\">",
    "<wp:simplePos x=\"0\" y=\"0\"/>",
    "<wp:positionH relativeFrom=\"column\"><wp:posOffset>0</wp:posOffset></wp:positionH>",
    "<wp:positionV relativeFrom=\"paragraph\"><wp:posOffset>0</wp:posOffset></wp:positionV>",
    "<wp:extent cx=\"5000000\" cy=\"800000\"/><wp:wrapTopAndBottom/>",
    "<wp:docPr id=\"1\" name=\"Text Box 1\"/><wp:cNvGraphicFramePr/>",
    "<a:graphic><a:graphicData uri=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">",
    "<wps:wsp><wps:cNvSpPr txBox=\"1\"/><wps:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"5000000\" cy=\"800000\"/></a:xfrm>",
    "<a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom><a:ln><a:solidFill><a:srgbClr val=\"000000\"/></a:solidFill></a:ln></wps:spPr>",
    "<wps:txbx><w:txbxContent>",
);
const TEXT_BOX_CLOSE: &str = "</w:txbxContent></wps:txbx><wps:bodyPr/></wps:wsp></a:graphicData></a:graphic></wp:anchor></w:drawing></w:r>";

const IMAGE: &str = concat!(
    "<w:r><w:drawing><wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\">",
    "<wp:extent cx=\"5486400\" cy=\"7132320\"/><wp:docPr id=\"2\" name=\"Scan\"/>",
    "<a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">",
    "<pic:pic><pic:nvPicPr><pic:cNvPr id=\"0\" name=\"scan.png\"/><pic:cNvPicPr/></pic:nvPicPr>",
    "<pic:blipFill><a:blip r:embed=\"rIdImg\"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill>",
    "<pic:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"5486400\" cy=\"7132320\"/></a:xfrm>",
    "<a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></pic:spPr></pic:pic>",
    "</a:graphicData></a:graphic></wp:inline></w:drawing></w:r>",
);

const BORDERS: &str = concat!(
    "<w:tblBorders><w:top w:val=\"single\" w:sz=\"4\"/><w:left w:val=\"single\" w:sz=\"4\"/>",
    "<w:bottom w:val=\"single\" w:sz=\"4\"/><w:right w:val=\"single\" w:sz=\"4\"/>",
    "<w:insideH w:val=\"single\" w:sz=\"4\"/><w:insideV w:val=\"single\" w:sz=\"4\"/></w:tblBorders>",
);

pub fn blocks(blocks: &[Block]) -> String {
    let mut out = String::new();
    for b in blocks {
        match b {
            Block::Para(runs) => {
                out.push_str("<w:p>");
                runs.iter().for_each(|r| out.push_str(&run(r)));
                out.push_str("</w:p>");
            }
            Block::Table(rows) => {
                out.push_str(&format!(
                    "<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/>{BORDERS}</w:tblPr>"
                ));
                out.push_str(
                    "<w:tblGrid><w:gridCol w:w=\"1800\"/><w:gridCol w:w=\"7200\"/></w:tblGrid>",
                );
                for row in rows {
                    out.push_str("<w:tr>");
                    for cell in row {
                        out.push_str(&format!(
                            "<w:tc><w:tcPr><w:tcW w:w=\"0\" w:type=\"auto\"/></w:tcPr>{}</w:tc>",
                            self::blocks(cell)
                        ));
                    }
                    out.push_str("</w:tr>");
                }
                out.push_str("</w:tbl>");
            }
            Block::TextBox(inner) => {
                out.push_str(&format!(
                    "<w:p>{TEXT_BOX_OPEN}{}{TEXT_BOX_CLOSE}</w:p>",
                    self::blocks(inner)
                ));
            }
            Block::Image => out.push_str(&format!("<w:p>{IMAGE}</w:p>")),
        }
    }
    out
}

pub const NAMESPACES: &str = concat!(
    "xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" ",
    "xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" ",
    "xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" ",
    "xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" ",
    "xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\" ",
    "xmlns:wps=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\"",
);
