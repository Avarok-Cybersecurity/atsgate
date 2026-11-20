//! Groups the lines of a dated section (experience, education, projects) into entries.
//!
//! Two layouts need geometry rather than markers, and both are what a browser prints from an
//! HTML resume:
//! - the title alone with its dates on one line and the organization on the next: a line after
//!   a dated title that names no organization is the organization, not a new entry;
//! - list items whose marker is drawn as a shape: a line indented at least
//!   [`HANGING_INDENT_MIN_PT`] right of its entry's first line is a bullet.

use crate::dates::{self, DateRange};
use crate::normalize::{LineRef, bullet_text};

/// One role, degree or project, with the lines it was read from.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub title: String,
    pub organization: Option<String>,
    pub dates: Option<DateRange>,
    pub bullets: Vec<String>,
    pub evidence: Vec<LineRef>,
}

#[derive(Debug, Default)]
struct Draft {
    header: Vec<String>,
    dates: Option<DateRange>,
    /// The dates were on the same line as the first header line.
    dates_inline: bool,
    bullets: Vec<String>,
    evidence: Vec<LineRef>,
}

const ORG_SEPARATORS: [&str; 5] = [" | ", " - ", ", ", " at ", " @ "];

/// The least indentation, in points, that makes an unmarked line a bullet: wider than glyph
/// side bearings and a word space, narrower than any list indent (browsers use 14-40 pt, word
/// processors 18 pt or more).
pub const HANGING_INDENT_MIN_PT: f32 = 6.0;

fn split_title(header: &str) -> (String, Option<String>) {
    for sep in ORG_SEPARATORS {
        if let Some((title, org)) = header.split_once(sep) {
            return (title.trim().to_owned(), Some(org.trim().to_owned()));
        }
    }
    (header.trim().to_owned(), None)
}

impl Draft {
    fn starts_new(&self, has_text: bool, has_dates: bool) -> bool {
        let empty = self.header.is_empty() && self.dates.is_none();
        !empty
            && (!self.bullets.is_empty()
                || (has_dates && self.dates.is_some())
                || (has_text
                    && self.dates.is_some()
                    && !self.header.is_empty()
                    && !(!has_dates && self.awaits_organization())))
    }

    /// A dated title line with no organization of its own: the next line may name it.
    fn awaits_organization(&self) -> bool {
        self.dates_inline
            && self.header.len() == 1
            && self.bullets.is_empty()
            && split_title(&self.header[0]).1.is_none()
    }

    /// The left edge of the entry's first line, when the format has geometry.
    fn left(&self) -> Option<f32> {
        self.evidence.first().and_then(|at| at.bbox).map(|b| b.x0)
    }

    fn indented(&self, at: &LineRef) -> bool {
        let started = !self.header.is_empty() || self.dates.is_some();
        match (self.left(), at.bbox) {
            (Some(left), Some(b)) => started && b.x0 >= left + HANGING_INDENT_MIN_PT,
            _ => false,
        }
    }

    fn finish(self) -> Option<Entry> {
        let mut header = self.header.into_iter();
        let (title, mut organization) = split_title(&header.next()?);
        if organization.is_none() {
            organization = header.next();
        }
        Some(Entry {
            title,
            organization,
            dates: self.dates,
            bullets: self.bullets,
            evidence: self.evidence,
        })
    }
}

/// Builds entries from normalized lines. Lines before the first header become loose `notes`.
pub fn group(lines: &[(String, LineRef)]) -> (Vec<Entry>, Vec<String>) {
    let (mut entries, mut notes) = (Vec::new(), Vec::new());
    let mut draft = Draft::default();
    for (text, at) in lines {
        let found = dates::find_range(text).or_else(|| dates::find_single(text));
        let marked = bullet_text(text)
            .or_else(|| (found.is_none() && draft.indented(at)).then_some(text.as_str()));
        if let Some(bullet) = marked {
            if draft.header.is_empty() && draft.dates.is_none() {
                notes.push(bullet.to_owned());
            } else {
                draft.bullets.push(bullet.to_owned());
                draft.evidence.push(at.clone());
            }
            continue;
        }
        let (range, rest) = match found {
            Some((range, rest)) => (
                Some(range),
                rest.trim_matches([' ', ',', '|', '-']).to_owned(),
            ),
            None => (None, text.clone()),
        };
        if draft.starts_new(!rest.is_empty(), range.is_some()) {
            entries.extend(std::mem::take(&mut draft).finish());
        }
        if range.is_some() && draft.dates.is_none() {
            draft.dates_inline = draft.header.is_empty() && !rest.is_empty();
        }
        if !rest.is_empty() {
            draft.header.push(rest);
        }
        if range.is_some() {
            draft.dates = range;
        }
        draft.evidence.push(at.clone());
    }
    entries.extend(draft.finish());
    (entries, notes)
}
