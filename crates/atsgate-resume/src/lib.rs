//! Pure segmentation: an extracted [`Document`] in, a [`Resume`] with evidence out.
//!
//! Basic by design: standard headings, common date formats, bullets and dated entries. Header
//! and footer lines are kept apart, because many parsers never read them.

mod dates;
mod entries;
mod headings;
mod normalize;

pub use dates::{DateEnd, DateRange, PartialDate};
pub use entries::Entry;
pub use headings::SectionKind;
pub use normalize::{LineRef, normalize};

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

fn items(lines: &[(String, LineRef)]) -> Vec<String> {
    lines
        .iter()
        .map(|(text, _)| normalize::bullet_text(text).unwrap_or(text))
        .map(|text| text.split_once(':').map_or(text, |(_, rest)| rest))
        .flat_map(|text| text.split([',', ';', '|', '\u{2022}', '\u{00B7}']))
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
    for (index, line) in doc.lines.iter().enumerate() {
        let text = normalize(&line.text);
        if text.is_empty() {
            continue;
        }
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
