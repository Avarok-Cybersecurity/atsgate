//! Layouts that only position tells apart: a running header and footer, a section heading at
//! the top of page 2, and the entry layout a browser prints from an HTML resume.

use super::content::{self as c, Entry};
use super::pdf_layouts::{BLACK, BODY, Col, body_first, body_rest, identity, spec};
use super::pdf_writer::{Font, Op, PAGE_H, PdfSpec};

/// Baseline of the first line on a page printed with half-inch margins, as Chromium prints:
/// inside the header band, so only repetition tells a running header from page content.
const TOP_LINE_Y: f32 = PAGE_H - 42.0;

fn footer(text: String) -> Op {
    Op::Text {
        x: 290.0,
        y: 24.0,
        size: 9.0,
        font: Font::Regular,
        rgb: BLACK,
        text,
    }
}

/// Name and contact details in the header band and "Page N of 2" in the footer band of both
/// pages: a running header and footer, repeated at the same position.
pub fn header_footer() -> PdfSpec {
    let page = |n: usize, write: fn(&mut Col)| {
        let mut head = Col::new(60.0, PAGE_H - 24.0);
        head.line(14.0, Font::Bold, c::NAME);
        head.line(9.0, Font::Regular, &c::CONTACT.join(" | "));
        let mut col = Col::new(60.0, 700.0);
        write(&mut col);
        let mut ops = head.ops;
        ops.extend(col.ops);
        ops.push(footer(format!("Page {n} of 2")));
        ops
    };
    spec(vec![page(1, body_first), page(2, body_rest)])
}

/// A section heading as the first line of page 2, inside the header band, appearing once.
pub fn page_top_heading() -> PdfSpec {
    let mut first = Col::new(60.0, 720.0);
    identity(&mut first, true);
    body_first(&mut first);
    // heading() moves down 6 pt before writing.
    let mut second = Col::new(60.0, TOP_LINE_Y + 6.0);
    body_rest(&mut second);
    spec(vec![first.ops, second.ops])
}

/// How far a browser indents list items (its markers are drawn as shapes, not text).
const LIST_INDENT: f32 = 14.0;
/// The right edge the dates are aligned to.
const RIGHT_EDGE: f32 = 552.0;
/// CSS `line-height: 1.2`: tight enough that a wrapped-line test alone would join the
/// organization line to the dated title line above it.
const BROWSER_LINE_HEIGHT: f32 = BODY * 1.2;

/// An entry as Chromium prints a typical HTML resume: the title alone with the dates flush
/// right on its line, the organization on the next line, and bullets indented with no marker
/// glyph.
fn browser_entry(col: &mut Col, e: &Entry) {
    let (title, org) = e
        .title
        .split_once(", ")
        .expect("fixture titles name an organization");
    col.at(0.0, BODY, Font::Regular, title);
    // Helvetica digits and capitals average about 0.55 em.
    let width = e.dates.chars().count() as f32 * BODY * 0.55;
    col.at(RIGHT_EDGE - width - col.x, BODY, Font::Regular, e.dates);
    col.y -= BROWSER_LINE_HEIGHT;
    col.at(0.0, BODY, Font::Regular, org);
    col.y -= BROWSER_LINE_HEIGHT;
    for b in e.bullets {
        col.at(LIST_INDENT, BODY, Font::Regular, b);
        col.y -= BROWSER_LINE_HEIGHT;
    }
}

pub fn browser_print() -> PdfSpec {
    let mut col = Col::new(60.0, 720.0);
    identity(&mut col, true);
    col.heading(c::SUMMARY_HEADING);
    col.lines(&c::SUMMARY);
    col.heading(c::EXPERIENCE_HEADING);
    c::ROLES.iter().for_each(|e| browser_entry(&mut col, e));
    col.heading(c::EDUCATION_HEADING);
    c::EDUCATION.iter().for_each(|e| browser_entry(&mut col, e));
    col.heading(c::SKILLS_HEADING);
    col.lines(&c::SKILLS);
    spec(vec![col.ops])
}
