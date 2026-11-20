//! Splits a job description into lines and labels each with the section it belongs to, from
//! heading cues ("Requirements", "Nice to have") and inline cues ("Stack:", "... is a plus").

use atsgate_taxonomy::fold;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// Text before any recognized heading.
    Body,
    Required,
    Preferred,
    Responsibilities,
    /// Benefits, compensation, company boilerplate: never a requirement.
    Ignored,
}

/// Heading cues in priority order (folded). A heading matches a cue when it starts with it.
pub const HEADING_CUES: [(&str, Section); 47] = [
    ("nice to have", Section::Preferred),
    ("nice to haves", Section::Preferred),
    ("preferred", Section::Preferred),
    ("bonus", Section::Preferred),
    ("pluses", Section::Preferred),
    ("good to have", Section::Preferred),
    ("desired", Section::Preferred),
    ("desirable", Section::Preferred),
    ("extra credit", Section::Preferred),
    ("requirements", Section::Required),
    ("required", Section::Required),
    ("qualifications", Section::Required),
    ("minimum qualifications", Section::Required),
    ("basic qualifications", Section::Required),
    ("must have", Section::Required),
    ("must haves", Section::Required),
    ("what you need", Section::Required),
    ("what youll need", Section::Required),
    ("what you will need", Section::Required),
    ("what were looking for", Section::Required),
    ("what we are looking for", Section::Required),
    ("who you are", Section::Required),
    ("about you", Section::Required),
    ("you have", Section::Required),
    ("skills", Section::Required),
    ("tech stack", Section::Required),
    ("our stack", Section::Required),
    ("stack", Section::Required),
    ("technologies", Section::Required),
    ("responsibilities", Section::Responsibilities),
    ("key responsibilities", Section::Responsibilities),
    ("what youll do", Section::Responsibilities),
    ("what you will do", Section::Responsibilities),
    ("the role", Section::Responsibilities),
    ("your role", Section::Responsibilities),
    ("duties", Section::Responsibilities),
    ("in this role", Section::Responsibilities),
    ("benefits", Section::Ignored),
    ("perks", Section::Ignored),
    ("compensation", Section::Ignored),
    ("salary", Section::Ignored),
    ("about us", Section::Ignored),
    ("who we are", Section::Ignored),
    ("how to apply", Section::Ignored),
    ("what we offer", Section::Ignored),
    ("why join us", Section::Ignored),
    ("equal opportunity", Section::Ignored),
];

/// Heading cues whose inline content is a list of technologies ("Stack: [python, rust]"): each
/// item names a skill whatever its case.
pub const LIST_CUES: [&str; 4] = ["tech stack", "our stack", "stack", "technologies"];

/// Phrases inside a line that mark that line alone as preferred.
pub const INLINE_PREFERRED: [&str; 8] = [
    "nice to have",
    "is a plus",
    "a plus",
    "a bonus",
    "bonus points",
    "preferred",
    "preferably",
    "would be great",
];

/// A heading may carry this many words beyond its cue ("Preferred qualifications (bonus)").
const HEADING_SLACK_WORDS: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line<'a> {
    pub text: &'a str,
    /// Byte offset of `text` in the input.
    pub offset: usize,
    pub section: Section,
    /// The text after a [`LIST_CUES`] colon: comma-separated items, each one skill.
    pub list: Option<&'a str>,
}

fn strip_marker(line: &str) -> &str {
    line.trim()
        .trim_start_matches(['#', '*', '-', '\u{2022}', '\u{00B7}', '>', ' '])
        .trim()
}

/// The cue a line opens with, its section, and the text after the cue's colon (if any).
fn heading(line: &str) -> Option<(&'static str, Section, &str)> {
    let stripped = strip_marker(line);
    let (head, rest) = match stripped.split_once(':') {
        Some((h, r)) => (h, r.trim()),
        None => (stripped, ""),
    };
    let folded = fold(head);
    let words = folded.split(' ').count();
    HEADING_CUES.iter().find_map(|(cue, section)| {
        let cue_words = cue.split(' ').count();
        let matches = folded == *cue
            || (folded.starts_with(&format!("{cue} ")) && words <= cue_words + HEADING_SLACK_WORDS);
        let bare_heading = rest.is_empty() && stripped.len() == head.len();
        let colon_heading = stripped.len() > head.len();
        (matches && (bare_heading || colon_heading)).then_some((*cue, *section, rest))
    })
}

fn inline_preferred(text: &str) -> bool {
    let folded = format!(" {} ", fold(text));
    INLINE_PREFERRED
        .iter()
        .any(|cue| folded.contains(&format!(" {cue} ")))
}

/// Byte ranges of the sentences of a line: split after ". ", "! " or "? ".
fn sentences(line: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start = 0;
    let bytes = line.as_bytes();
    for i in 0..bytes.len().saturating_sub(1) {
        if matches!(bytes[i], b'.' | b'!' | b'?') && bytes[i + 1] == b' ' {
            out.push((start, i + 1));
            start = i + 2;
        }
    }
    out.push((start, line.len()));
    out.into_iter()
        .filter(|(s, e)| line[*s..*e].trim() != "")
        .collect()
}

/// Labels every sentence of every non-empty line. A bare heading line sets the section for the
/// lines after it; a cue with content after its colon ("Stack: Rust, Go"), at the start of a line
/// or of a sentence, labels only that sentence.
pub fn label(text: &str) -> Vec<Line<'_>> {
    let mut out = Vec::new();
    let mut current = Section::Body;
    let mut offset = 0;
    for raw in text.split_inclusive('\n') {
        let line_offset = offset;
        offset += raw.len();
        let line = raw.trim_end_matches(['\n', '\r']);
        if line.trim().is_empty() {
            continue;
        }
        if let Some((_, section, "")) = heading(line.trim()) {
            current = section;
        }
        for (s, e) in sentences(line) {
            let sentence = line[s..e].trim();
            let lead = line[s..e].len() - line[s..e].trim_start().len();
            let (section, list) = match heading(sentence) {
                Some((cue, section, rest)) if !rest.is_empty() => {
                    (section, LIST_CUES.contains(&cue).then_some(rest))
                }
                _ => (current, None),
            };
            let section = if section != Section::Ignored && inline_preferred(sentence) {
                Section::Preferred
            } else {
                section
            };
            out.push(Line {
                text: sentence,
                offset: line_offset + s + lead,
                section,
                list,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_list_cue_is_a_heading_cue() {
        for cue in LIST_CUES {
            assert!(HEADING_CUES.iter().any(|(c, _)| *c == cue), "{cue}");
        }
    }
}
