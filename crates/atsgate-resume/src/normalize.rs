//! Text normalization for parsing. Evidence keeps the raw text; matching uses this form.

use atsgate_extract::Line;

const LIGATURES: [(char, &str); 7] = [
    ('\u{FB00}', "ff"),
    ('\u{FB01}', "fi"),
    ('\u{FB02}', "fl"),
    ('\u{FB03}', "ffi"),
    ('\u{FB04}', "ffl"),
    ('\u{FB05}', "st"),
    ('\u{FB06}', "st"),
];
const INVISIBLE: [char; 6] = [
    '\u{200B}', '\u{200C}', '\u{200D}', '\u{2060}', '\u{FEFF}', '\u{00AD}',
];
const DASHES: [char; 5] = ['\u{2010}', '\u{2012}', '\u{2013}', '\u{2014}', '\u{2212}'];

/// Expands ligatures, drops invisible characters, unifies dashes and collapses whitespace.
pub fn normalize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if let Some((_, expanded)) = LIGATURES.iter().find(|(l, _)| *l == c) {
            out.push_str(expanded);
        } else if DASHES.contains(&c) {
            out.push('-');
        } else if !INVISIBLE.contains(&c) {
            out.push(if c.is_whitespace() { ' ' } else { c });
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

const BULLETS: [char; 7] = [
    '\u{2022}', '\u{25AA}', '\u{25E6}', '\u{2023}', '\u{25CF}', '*', '-',
];

/// The text after a leading bullet marker, when the line is a bullet.
pub fn bullet_text(normalized: &str) -> Option<&str> {
    let mut chars = normalized.chars();
    let first = chars.next()?;
    let rest = chars.as_str();
    (BULLETS.contains(&first) && rest.starts_with(' ')).then(|| rest.trim_start())
}

/// A reference from parsed output back to the document line it came from.
#[derive(Debug, Clone, PartialEq)]
pub struct LineRef {
    /// Index into `Document::lines`.
    pub index: usize,
    pub page: u32,
    pub bbox: Option<atsgate_extract::BBox>,
}

impl LineRef {
    pub fn of(index: usize, line: &Line) -> Self {
        Self {
            index,
            page: line.page,
            bbox: line.bbox,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_ligatures_invisibles_dashes_and_space() {
        assert_eq!(
            normalize("\u{FB01}nance  Ru\u{200B}st\t2012\u{2013}2016"),
            "finance Rust 2012-2016"
        );
    }

    #[test]
    fn bullets_need_a_marker_and_a_space() {
        assert_eq!(bullet_text("\u{2022} Led a team"), Some("Led a team"));
        assert_eq!(bullet_text("- Led"), Some("Led"));
        assert_eq!(bullet_text("-5% cost"), None);
    }
}
