//! A minimal, deterministic PDF writer for fixtures, built on lopdf.

use super::png::{SCAN_H, SCAN_W, scan_pixels};
use lopdf::content::{Content, Operation};
use lopdf::{Dictionary, Document, Object, Stream, dictionary};

pub const PAGE_W: f32 = 612.0;
pub const PAGE_H: f32 = 792.0;

/// Which font resource a text run uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Font {
    /// Helvetica, WinAnsiEncoding: a standard font that needs no ToUnicode.
    Regular,
    /// Helvetica-Bold, WinAnsiEncoding.
    Bold,
    /// Helvetica with a ToUnicode CMap that maps two spare codes to U+200B and U+FB01.
    Mapped,
    /// A Type0 Identity-H font with no ToUnicode CMap: its text cannot be mapped to Unicode.
    Unmapped,
}

#[derive(Debug, Clone)]
pub enum Op {
    Text {
        x: f32,
        y: f32,
        size: f32,
        font: Font,
        rgb: [f32; 3],
        text: String,
    },
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    Image {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
}

#[derive(Debug, Clone)]
pub struct PdfSpec {
    pub title: String,
    pub keywords: Option<String>,
    pub pages: Vec<Vec<Op>>,
}

const ZERO_WIDTH_CODE: u8 = 0x81;
const FI_LIGATURE_CODE: u8 = 0x82;

fn encode(font: Font, text: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    for c in text.chars() {
        let code = match (font, c) {
            (Font::Unmapped, c) if c.is_ascii() => {
                out.push(0);
                c as u8
            }
            (_, c) if (' '..='~').contains(&c) => c as u8,
            (_, '\u{2022}') => 0x95,
            (_, '\u{2013}') => 0x96,
            (Font::Mapped, '\u{200B}') => ZERO_WIDTH_CODE,
            (Font::Mapped, '\u{FB01}') => FI_LIGATURE_CODE,
            _ => return Err(format!("fixture font {font:?} cannot encode {c:?}")),
        };
        out.push(code);
    }
    Ok(out)
}

fn to_unicode_cmap() -> Vec<u8> {
    let body = format!(
        "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n/CMapName /AtsgateFixture def\n\
         /CMapType 2 def\n1 begincodespacerange\n<00> <FF>\nendcodespacerange\n\
         4 beginbfchar\n<95> <2022>\n<96> <2013>\n<{ZERO_WIDTH_CODE:02X}> <200B>\n<{FI_LIGATURE_CODE:02X}> <FB01>\nendbfchar\n\
         1 beginbfrange\n<20> <7E> <0020>\nendbfrange\nendcmap\n\
         CMapName currentdict /CMap defineresource pop\nend\nend\n"
    );
    body.into_bytes()
}

fn fonts(doc: &mut Document) -> Dictionary {
    let simple = |base: &str| {
        dictionary! { "Type" => "Font", "Subtype" => "Type1", "BaseFont" => base.to_owned(), "Encoding" => "WinAnsiEncoding" }
    };
    let cmap = doc.add_object(Stream::new(Dictionary::new(), to_unicode_cmap()));
    let mut mapped = simple("Helvetica");
    mapped.set("ToUnicode", cmap);
    let cid = doc.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "CIDFontType2", "BaseFont" => "ArialMT",
        "CIDSystemInfo" => dictionary! { "Registry" => Object::string_literal("Adobe"), "Ordering" => Object::string_literal("Identity"), "Supplement" => 0 },
        "DW" => 556,
    });
    let unmapped = dictionary! {
        "Type" => "Font", "Subtype" => "Type0", "BaseFont" => "ArialMT",
        "Encoding" => "Identity-H", "DescendantFonts" => vec![cid.into()],
    };
    dictionary! {
        "F1" => doc.add_object(simple("Helvetica")),
        "F2" => doc.add_object(simple("Helvetica-Bold")),
        "F3" => doc.add_object(mapped),
        "F4" => doc.add_object(unmapped),
    }
}

/// The scan raster shared with the DOCX fixture, as an image XObject.
fn scan_image() -> Stream {
    let dict = dictionary! {
        "Type" => "XObject", "Subtype" => "Image", "Width" => i64::from(SCAN_W), "Height" => i64::from(SCAN_H),
        "ColorSpace" => "DeviceGray", "BitsPerComponent" => 8,
    };
    Stream::new(dict, scan_pixels())
}

fn ops(page: &[Op]) -> Result<Vec<Operation>, String> {
    let r = |v: f32| Object::Real(v);
    let mut out = Vec::new();
    for op in page {
        match op {
            Op::Text {
                x,
                y,
                size,
                font,
                rgb,
                text,
            } => {
                let name = match font {
                    Font::Regular => "F1",
                    Font::Bold => "F2",
                    Font::Mapped => "F3",
                    Font::Unmapped => "F4",
                };
                out.push(Operation::new("rg", rgb.iter().map(|c| r(*c)).collect()));
                out.push(Operation::new("BT", vec![]));
                out.push(Operation::new("Tf", vec![name.into(), r(*size)]));
                out.push(Operation::new(
                    "Tm",
                    vec![r(1.0), r(0.0), r(0.0), r(1.0), r(*x), r(*y)],
                ));
                let bytes = encode(*font, text)?;
                out.push(Operation::new(
                    "Tj",
                    vec![Object::String(bytes, lopdf::StringFormat::Literal)],
                ));
                out.push(Operation::new("ET", vec![]));
            }
            Op::Rect { x, y, w, h } => {
                out.push(Operation::new("re", vec![r(*x), r(*y), r(*w), r(*h)]));
                out.push(Operation::new("S", vec![]));
            }
            Op::Image { x, y, w, h } => {
                out.push(Operation::new("q", vec![]));
                out.push(Operation::new(
                    "cm",
                    vec![r(*w), r(0.0), r(0.0), r(*h), r(*x), r(*y)],
                ));
                out.push(Operation::new("Do", vec!["Im1".into()]));
                out.push(Operation::new("Q", vec![]));
            }
        }
    }
    Ok(out)
}

pub fn build(spec: &PdfSpec) -> Result<Vec<u8>, String> {
    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();
    let font_dict = fonts(&mut doc);
    let mut resources = dictionary! { "Font" => font_dict };
    if spec
        .pages
        .iter()
        .flatten()
        .any(|op| matches!(op, Op::Image { .. }))
    {
        resources.set(
            "XObject",
            dictionary! { "Im1" => doc.add_object(scan_image()) },
        );
    }
    let resources = doc.add_object(resources);
    let mut kids = Vec::new();
    for page in &spec.pages {
        let content = Content {
            operations: ops(page)?,
        }
        .encode()
        .map_err(|e| e.to_string())?;
        let content_id = doc.add_object(Stream::new(Dictionary::new(), content));
        kids.push(Object::from(doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id, "Contents" => content_id, "Resources" => resources,
            "MediaBox" => vec![0.into(), 0.into(), Object::Real(PAGE_W), Object::Real(PAGE_H)],
        })));
    }
    let count = kids.len() as i64;
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! { "Type" => "Pages", "Kids" => kids, "Count" => count }),
    );
    let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    let mut info = dictionary! { "Title" => Object::string_literal(spec.title.clone()), "Producer" => Object::string_literal("atsgate xtask fixtures") };
    if let Some(k) = &spec.keywords {
        info.set("Keywords", Object::string_literal(k.clone()));
    }
    let info_id = doc.add_object(info);
    doc.trailer.set("Root", catalog);
    doc.trailer.set("Info", info_id);
    let mut bytes = Vec::new();
    doc.save_to(&mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes)
}
