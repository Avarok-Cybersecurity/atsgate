//! Pure segmentation: an extracted [`Document`] in, a [`Resume`] with evidence out.
//!
//! Basic by design: standard headings, common date formats, bullets and dated entries. Header
//! and footer lines are kept apart, because many parsers never read them. Lines a PDF wrapped
//! are joined back first (see [`logical_lines`]).

mod dates;
mod entries;
mod headings;
mod normalize;
mod tokens;
mod wrap;

pub use dates::{DateEnd, DateRange, PartialDate};
pub use entries::{Entry, HANGING_INDENT_MIN_PT};
pub use headings::SectionKind;
pub use normalize::{LineRef, normalize};
pub use wrap::{
    FIT_SLACK, LEFT_EDGE_TOLERANCE_PT, WRAP_MAX_ADVANCE, WRAP_MAX_INNER_GAP, logical_lines,
};

use atsgate_extract::{Document, Region};

#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub kind: SectionKind,
    pub heading: LineRef,
    /// Dated entries, for experience, education, projects and volunteering.
    pub entries: Vec<Entry>,
    /// Individual items, for skills, certifications, languages and awards.
    pub items: Vec<String>,
    /// Running text: the summary, or lines in an entry section that belong to no entry.
    pub text: String,
    pub evidence: Vec<LineRef>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Resume {
    pub sections: Vec<Section>,
    /// Lines before the first heading (usually name and contact details).
    pub preamble: Vec<String>,
    /// Lines in page headers or footers.
    pub header_footer: Vec<String>,
    /// Short all-capitals lines that are not standard headings.
    pub unrecognized_headings: Vec<LineRef>,
}

impl Resume {
    pub fn section(&self, kind: SectionKind) -> Option<&Section> {
        self.sections.iter().find(|s| s.kind == kind)
    }
}

const ITEM_SEPARATORS: [char; 5] = [',', ';', '|', '\u{2022}', '\u{00B7}'];

/// `text` split on item separators outside parentheses: "PQC (ML-KEM, ML-DSA), TLS" is two
/// items, not three.
fn split_items(text: &str) -> Vec<&str> {
    let (mut out, mut depth, mut start) = (Vec::new(), 0usize, 0);
    for (at, c) in text.char_indices() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            c if depth == 0 && ITEM_SEPARATORS.contains(&c) => {
                out.push(&text[start..at]);
                start = at + c.len_utf8();
            }
            _ => {}
        }
    }
    out.push(&text[start..]);
    out
}

fn items(lines: &[(String, LineRef)]) -> Vec<String> {
    lines
        .iter()
        .map(|(text, _)| normalize::bullet_text(text).unwrap_or(text))
        .map(|text| text.split_once(':').map_or(text, |(_, rest)| rest))
        .flat_map(split_items)
        .map(|item| item.trim().to_owned())
        .filter(|item| !item.is_empty())
        .collect()
}

fn build(kind: SectionKind, heading: LineRef, lines: Vec<(String, LineRef)>) -> Section {
    let evidence = lines.iter().map(|(_, at)| at.clone()).collect();
    let (entries, notes, list, text) = if kind.has_entries() {
        let (entries, notes) = entries::group(&lines);
        (entries, notes.join(" "), Vec::new(), String::new())
    } else if kind == SectionKind::Summary {
        (
            Vec::new(),
            String::new(),
            Vec::new(),
            lines
                .iter()
                .map(|(t, _)| t.as_str())
                .collect::<Vec<_>>()
                .join(" "),
        )
    } else {
        (Vec::new(), String::new(), items(&lines), String::new())
    };
    let text = if notes.is_empty() { text } else { notes };
    Section {
        kind,
        heading,
        entries,
        items: list,
        text,
        evidence,
    }
}

/// A section being collected: its kind, heading line, and normalized body lines.
type OpenSection = (SectionKind, LineRef, Vec<(String, LineRef)>);

pub fn parse(doc: &Document) -> Resume {
    let mut resume = Resume::default();
    let mut open: Option<OpenSection> = None;
    for (index, text) in logical_lines(doc) {
        let line = &doc.lines[index];
        if matches!(line.region, Region::Header | Region::Footer) {
            resume.header_footer.push(text);
            continue;
        }
        let at = LineRef::of(index, line);
        if let Some(kind) = headings::standard(&text) {
            if let Some((k, h, lines)) = open.take() {
                resume.sections.push(build(k, h, lines));
            }
            open = Some((kind, at, Vec::new()));
            continue;
        }
        if headings::looks_like_heading(&text) {
            resume.unrecognized_headings.push(at.clone());
        }
        match open.as_mut() {
            Some((_, _, lines)) => lines.push((text, at)),
            None => resume.preamble.push(text),
        }
    }
    if let Some((k, h, lines)) = open {
        resume.sections.push(build(k, h, lines));
    }
    resume
}

#[cfg(test)]
mod tests {
    use super::split_items;

    #[test]
    fn separators_inside_parentheses_do_not_split_items() {
        let items = split_items("PQC (ML-KEM, ML-DSA), TLS; QUIC | Rust [async, sync]");
        assert_eq!(
            items,
            [
                "PQC (ML-KEM, ML-DSA)",
                " TLS",
                " QUIC ",
                " Rust [async, sync]"
            ]
        );
        assert_eq!(split_items("a) b, c"), ["a) b", " c"]);
    }
}
