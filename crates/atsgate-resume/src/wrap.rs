//! Rejoins lines that a PDF wrapped. A PDF stores each visual line on its own, so a bullet or a
//! paragraph that runs over several lines would otherwise read as several entries. A line
//! continues the one above when all of these hold:
//!
//! - it sits on the same page and in the same region, with geometry (flow formats never wrap);
//! - it is not a bullet, not a standard heading, and does not follow a heading;
//! - it advances by about one line height, not by a paragraph gap;
//! - it starts no further left than the paragraph's first line (hanging indents start further
//!   right, a new entry starts further left);
//! - the line above runs so close to the right edge that this line's first word could not have
//!   fitted after it.

use crate::headings;
use crate::normalize::{bullet_text, normalize};
use atsgate_extract::{Document, Line};

/// The largest baseline advance, in font sizes, that still counts as a wrapped line: single
/// spacing advances 1.1-1.2 font sizes, while paragraph spacing in word processors and Typst
/// adds at least another 0.25.
pub const WRAP_MAX_ADVANCE: f32 = 1.3;
/// Slack, in font sizes, when asking whether the next word would have fitted on the line above:
/// about one word space plus side bearings.
pub const FIT_SLACK: f32 = 0.5;
/// Glyph side bearings move a line's left edge by a fraction of a point.
pub const LEFT_EDGE_TOLERANCE_PT: f32 = 1.0;

/// One logical line: the index of its first physical line, and its normalized text.
pub type Logical = (usize, String);

fn font_size(doc: &Document, line: &Line) -> Option<f32> {
    line.spans.first().and_then(|id| doc.span(*id).font_size)
}

fn right_edge(doc: &Document, page: u32) -> f32 {
    doc.lines
        .iter()
        .filter(|l| l.page == page)
        .filter_map(|l| l.bbox.map(|b| b.x1))
        .fold(f32::MIN, f32::max)
}

/// The width of `text`'s first word, estimated from the line's width per character.
fn first_word_width(line: &Line, text: &str) -> Option<f32> {
    let chars = text.chars().count();
    let word = text.split_whitespace().next()?.chars().count();
    let width = line.bbox?.width();
    (chars > 0).then(|| width * word as f32 / chars as f32)
}

fn continues(doc: &Document, first: &Line, prev: &Line, line: &Line, text: &str) -> bool {
    let (Some(p), Some(b), Some(f), Some(size)) =
        (prev.bbox, line.bbox, first.bbox, font_size(doc, line))
    else {
        return false;
    };
    if prev.page != line.page || prev.region != line.region {
        return false;
    }
    if bullet_text(text).is_some() || headings::standard(text).is_some() {
        return false;
    }
    let advance = p.y0 - b.y0;
    let wrapped_spacing = advance > 0.0 && advance <= WRAP_MAX_ADVANCE * size;
    let aligned = b.x0 >= f.x0 - LEFT_EDGE_TOLERANCE_PT;
    let full = first_word_width(line, text)
        .is_some_and(|w| p.x1 + w >= right_edge(doc, line.page) - FIT_SLACK * size);
    wrapped_spacing && aligned && full
}

/// The document's lines in reading order, with wrapped lines joined to the line they continue.
pub fn logical_lines(doc: &Document) -> Vec<Logical> {
    let mut out: Vec<Logical> = Vec::new();
    let mut prev: Option<usize> = None;
    for (index, line) in doc.lines.iter().enumerate() {
        let text = normalize(&line.text);
        if text.is_empty() {
            continue;
        }
        let joined = match (out.last_mut(), prev) {
            (Some((first, so_far)), Some(p))
                if headings::standard(so_far).is_none()
                    && continues(doc, &doc.lines[*first], &doc.lines[p], line, &text) =>
            {
                so_far.push(' ');
                so_far.push_str(&text);
                true
            }
            _ => false,
        };
        if !joined {
            out.push((index, text));
        }
        prev = Some(index);
    }
    out
}
