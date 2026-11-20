//! Groups the lines of a dated section (experience, education, projects) into entries.

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
    bullets: Vec<String>,
    evidence: Vec<LineRef>,
}

const ORG_SEPARATORS: [&str; 5] = [" | ", " - ", ", ", " at ", " @ "];

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
                || (has_text && self.dates.is_some() && !self.header.is_empty()))
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
        if let Some(bullet) = bullet_text(text) {
            if draft.header.is_empty() && draft.dates.is_none() {
                notes.push(bullet.to_owned());
            } else {
                draft.bullets.push(bullet.to_owned());
                draft.evidence.push(at.clone());
            }
            continue;
        }
        let (range, rest) = match dates::find_range(text).or_else(|| dates::find_single(text)) {
            Some((range, rest)) => (
                Some(range),
                rest.trim_matches([' ', ',', '|', '-']).to_owned(),
            ),
            None => (None, text.clone()),
        };
        if draft.starts_new(!rest.is_empty(), range.is_some()) {
            entries.extend(std::mem::take(&mut draft).finish());
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
