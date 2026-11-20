//! The resume laid out as each fixture variant, as page operations for [`super::pdf_writer`].

use super::content::{self as c, Entry};
use super::pdf_writer::{Font, Op, PAGE_H, PAGE_W, PdfSpec};

pub const BLACK: [f32; 3] = [0.0, 0.0, 0.0];
pub const WHITE: [f32; 3] = [1.0, 1.0, 1.0];
pub const BODY: f32 = 10.0;
const HEAD: f32 = 11.0;

/// A top-down text cursor in one column.
struct Col {
    x: f32,
    y: f32,
    ops: Vec<Op>,
}

impl Col {
    fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            ops: Vec::new(),
        }
    }
    fn at(&mut self, dx: f32, size: f32, font: Font, text: &str) {
        let (x, y) = (self.x + dx, self.y);
        self.ops.push(Op::Text {
            x,
            y,
            size,
            font,
            rgb: BLACK,
            text: text.to_owned(),
        });
    }
    fn line(&mut self, size: f32, font: Font, text: &str) {
        self.at(0.0, size, font, text);
        self.y -= size * 1.45;
    }
    fn gap(&mut self, pts: f32) {
        self.y -= pts;
    }
    fn heading(&mut self, text: &str) {
        self.gap(6.0);
        self.line(HEAD, Font::Bold, text);
    }
    fn lines(&mut self, lines: &[&str]) {
        lines.iter().for_each(|l| self.line(BODY, Font::Regular, l));
    }
    fn bullets(&mut self, bullets: &[&str]) {
        for b in bullets {
            self.at(10.0, BODY, Font::Regular, &format!("{}{b}", c::BULLET));
            self.y -= BODY * 1.45;
        }
    }
    /// An entry with its dates right-aligned on the title's baseline.
    fn entry_inline(&mut self, e: &Entry, dates_x: f32) {
        self.at(0.0, BODY, Font::Regular, e.title);
        self.at(dates_x - self.x, BODY, Font::Regular, e.dates);
        self.y -= BODY * 1.45;
        self.bullets(e.bullets);
    }
    /// An entry with its dates on the line below the title.
    fn entry_stacked(&mut self, e: &Entry) {
        self.lines(&[e.title, e.dates]);
        self.bullets(e.bullets);
    }
}

fn spec(pages: Vec<Vec<Op>>) -> PdfSpec {
    PdfSpec {
        title: c::NAME.to_owned(),
        keywords: None,
        pages,
    }
}

fn identity(col: &mut Col, contact_line: bool) {
    col.line(16.0, Font::Bold, c::NAME);
    if contact_line {
        col.line(BODY, Font::Regular, &c::CONTACT.join(" | "));
    } else {
        col.lines(&c::CONTACT);
    }
}

/// Summary, Experience, Education and Skills in one column below `col`'s cursor.
fn body(col: &mut Col) {
    col.heading(c::SUMMARY_HEADING);
    col.lines(&c::SUMMARY);
    col.heading(c::EXPERIENCE_HEADING);
    c::ROLES.iter().for_each(|e| col.entry_inline(e, 450.0));
    col.heading(c::EDUCATION_HEADING);
    c::EDUCATION.iter().for_each(|e| col.entry_inline(e, 450.0));
    col.heading(c::SKILLS_HEADING);
    col.lines(&c::SKILLS);
}

pub fn one_column() -> PdfSpec {
    let mut col = Col::new(60.0, 720.0);
    identity(&mut col, true);
    body(&mut col);
    spec(vec![col.ops])
}

pub fn two_column() -> PdfSpec {
    let mut left = Col::new(40.0, 720.0);
    identity(&mut left, false);
    left.heading(c::SKILLS_HEADING);
    left.lines(&c::SKILLS);
    left.heading(c::EDUCATION_HEADING);
    c::EDUCATION.iter().for_each(|e| left.entry_stacked(e));
    let mut right = Col::new(290.0, 720.0);
    right.heading(c::SUMMARY_HEADING);
    right.lines(&c::SUMMARY);
    right.heading(c::EXPERIENCE_HEADING);
    c::ROLES.iter().for_each(|e| right.entry_stacked(e));
    let mut ops = left.ops;
    ops.extend(right.ops);
    spec(vec![ops])
}

/// Writes one table cell's content.
type Fill = dyn Fn(&mut Col);

/// One ruled table: a heading cell on the left and the section's content on the right.
pub fn table() -> PdfSpec {
    let mut col = Col::new(60.0, 720.0);
    identity(&mut col, true);
    let mut ops = std::mem::take(&mut col.ops);
    let (left_x, split_x, right_x) = (50.0, 160.0, 562.0);
    let mut top = col.y;
    let rows: [(&str, &Fill); 4] = [
        (c::SUMMARY_HEADING, &|c2| c2.lines(&c::SUMMARY)),
        (c::EXPERIENCE_HEADING, &|c2| {
            c::ROLES.iter().for_each(|e| c2.entry_stacked(e))
        }),
        (c::EDUCATION_HEADING, &|c2| {
            c::EDUCATION.iter().for_each(|e| c2.entry_stacked(e))
        }),
        (c::SKILLS_HEADING, &|c2| c2.lines(&c::SKILLS)),
    ];
    for (heading, fill) in rows {
        let mut head = Col::new(left_x + 6.0, top - 14.0);
        head.line(HEAD, Font::Bold, heading);
        let mut cell = Col::new(split_x + 6.0, top - 14.0);
        fill(&mut cell);
        let bottom = cell.y.min(head.y) + 4.0;
        ops.extend(head.ops);
        ops.extend(cell.ops);
        ops.push(Op::Rect {
            x: left_x,
            y: bottom,
            w: split_x - left_x,
            h: top - bottom,
        });
        ops.push(Op::Rect {
            x: split_x,
            y: bottom,
            w: right_x - split_x,
            h: top - bottom,
        });
        top = bottom;
    }
    spec(vec![ops])
}

/// The summary sits in a framed box near the top but is drawn last, as word processors do.
pub fn text_box() -> PdfSpec {
    let mut col = Col::new(60.0, 720.0);
    identity(&mut col, true);
    let box_top = col.y;
    col.gap(70.0);
    col.heading(c::EXPERIENCE_HEADING);
    c::ROLES.iter().for_each(|e| col.entry_inline(e, 450.0));
    col.heading(c::EDUCATION_HEADING);
    c::EDUCATION.iter().for_each(|e| col.entry_inline(e, 450.0));
    col.heading(c::SKILLS_HEADING);
    col.lines(&c::SKILLS);
    let mut boxed = Col::new(70.0, box_top - 8.0);
    boxed.heading(c::SUMMARY_HEADING);
    boxed.lines(&c::SUMMARY);
    let mut ops = col.ops;
    ops.extend(boxed.ops);
    ops.push(Op::Rect {
        x: 60.0,
        y: box_top - 66.0,
        w: 400.0,
        h: 60.0,
    });
    spec(vec![ops])
}

/// Name and contact details in the page header band, a page number in the footer band.
pub fn header_footer() -> PdfSpec {
    let mut head = Col::new(60.0, PAGE_H - 24.0);
    head.line(14.0, Font::Bold, c::NAME);
    head.line(9.0, Font::Regular, &c::CONTACT.join(" | "));
    let mut col = Col::new(60.0, 700.0);
    body(&mut col);
    let mut ops = head.ops;
    ops.extend(col.ops);
    ops.push(Op::Text {
        x: 290.0,
        y: 24.0,
        size: 9.0,
        font: Font::Regular,
        rgb: BLACK,
        text: "Page 1 of 1".into(),
    });
    spec(vec![ops])
}

pub fn image_only() -> PdfSpec {
    spec(vec![vec![Op::Image {
        x: 36.0,
        y: 36.0,
        w: PAGE_W - 72.0,
        h: PAGE_H - 72.0,
    }]])
}
