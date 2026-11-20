//! The job title and seniority, read from the first lines of a job description.

use super::resolve::resolve_occupation;
use crate::Seniority;
use atsgate_taxonomy::{ONET_SOC_PREFIX, SourceName, Taxonomy, fold};

/// Seniority words (folded) in the order they are checked; the first hit wins.
pub const SENIORITY_WORDS: [(&str, Seniority); 20] = [
    ("intern", Seniority::Intern),
    ("internship", Seniority::Intern),
    ("junior", Seniority::Junior),
    ("jr", Seniority::Junior),
    ("entry level", Seniority::Junior),
    ("mid level", Seniority::Mid),
    ("intermediate", Seniority::Mid),
    ("senior", Seniority::Senior),
    ("sr", Seniority::Senior),
    ("staff", Seniority::Staff),
    ("principal", Seniority::Principal),
    ("lead", Seniority::Lead),
    ("head of", Seniority::Manager),
    ("manager", Seniority::Manager),
    ("director", Seniority::Director),
    ("vp", Seniority::Executive),
    ("vice president", Seniority::Executive),
    ("chief", Seniority::Executive),
    ("cto", Seniority::Executive),
    ("ceo", Seniority::Executive),
];

/// Words that make a segment look like a job title.
pub const TITLE_HEADS: [&str; 22] = [
    "engineer",
    "developer",
    "programmer",
    "designer",
    "scientist",
    "analyst",
    "manager",
    "accountant",
    "paralegal",
    "attorney",
    "lawyer",
    "counsel",
    "marketer",
    "representative",
    "specialist",
    "architect",
    "consultant",
    "director",
    "administrator",
    "researcher",
    "executive",
    "associate",
];

/// A title with nothing left but one of these says nothing about the occupation.
pub const GENERIC_TITLES: [&str; 6] = [
    "engineer",
    "developer",
    "manager",
    "specialist",
    "associate",
    "consultant",
];

/// How many leading non-empty lines may hold the title.
pub const TITLE_LINES: usize = 3;

#[derive(Debug, Clone, PartialEq)]
pub struct TitleGuess {
    /// The title as written, singular ("Senior Engineer").
    pub title: String,
    pub seniority: Option<Seniority>,
    /// The O*NET occupation the title resolves to, when it resolves.
    pub onet: Option<(String, String)>,
    /// The ESCO occupation URI the title resolves to, when it resolves.
    pub esco: Option<String>,
}

fn singular(word: &str) -> &str {
    word.strip_suffix('s')
        .filter(|w| TITLE_HEADS.contains(&fold(w).as_str()))
        .unwrap_or(word)
}

/// The first seniority word in a folded title.
pub fn seniority_of(folded: &str) -> Option<Seniority> {
    let padded = format!(" {folded} ");
    SENIORITY_WORDS
        .iter()
        .find(|(w, _)| padded.contains(&format!(" {w} ")))
        .map(|(_, s)| *s)
}

/// `folded` without the seniority words up to Lead ("senior software engineer" -> "software engineer").
pub fn strip_seniority(folded: &str) -> String {
    let mut out = format!(" {folded} ");
    for (w, _) in SENIORITY_WORDS
        .iter()
        .filter(|(_, s)| *s <= Seniority::Lead && *s != Seniority::Unspecified)
    {
        out = out.replace(&format!(" {w} "), " ");
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Characters that end a title inside a line: "Title | Company", "Title (Team)", "Title [stack]".
const SEGMENT_BREAKS: [char; 8] = ['|', '\u{2014}', '\u{2013}', '(', ')', '[', ']', ','];

/// Lowercase words a title may contain ("Head of Engineering", "Research & Development").
const TITLE_CONNECTORS: [&str; 7] = ["of", "and", "&", "for", "to", "the", "in"];

fn segments(line: &str) -> impl Iterator<Item = &str> {
    line.split(SEGMENT_BREAKS)
        .flat_map(|s| s.split(" - "))
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

/// Title-cased (every word but a connector capitalized), at most six words, with a title head.
/// "Looking for engineers" is prose, not a title.
fn names_title(segment: &str) -> bool {
    let raw: Vec<&str> = segment.split_whitespace().collect();
    let cased = raw.iter().all(|w| {
        TITLE_CONNECTORS.contains(w) || !w.chars().next().is_some_and(char::is_lowercase)
    });
    let words: Vec<String> = raw.iter().map(|w| fold(singular(w))).collect();
    cased && words.len() <= 6 && words.iter().any(|w| TITLE_HEADS.contains(&w.as_str()))
}

/// The title a line opens with when it is an item of a role list: "Senior Engineer (Platform)
/// — Build ...", "- Backend Engineer [rust, python] - ...".
fn role_item(line: &str) -> Option<&str> {
    let item = line
        .trim_start_matches(['-', '*', '\u{2022}', ' '])
        .split(SEGMENT_BREAKS)
        .next()?
        .split(" - ")
        .next()?
        .trim();
    names_title(item).then_some(item)
}

/// The first segment of the leading lines that names a title, else the first role-list item of
/// a posting that lists several roles (Hacker News "Who's Hiring"). Its O*NET occupation is
/// read in `context` (an O*NET-SOC code, see [`resolve_occupation`]).
pub fn guess(lines: &[&str], taxonomy: &Taxonomy, context: Option<&str>) -> Option<TitleGuess> {
    let segment = lines
        .iter()
        .take(TITLE_LINES)
        .flat_map(|l| segments(l))
        .find(|s| names_title(s))
        .or_else(|| lines.iter().find_map(|l| role_item(l)))?;
    let title = segment
        .split_whitespace()
        .map(singular)
        .collect::<Vec<_>>()
        .join(" ");
    let folded = fold(&title);
    let core = strip_seniority(&folded);
    let specific = !core.is_empty() && !GENERIC_TITLES.contains(&core.as_str());
    let esco = specific
        .then(|| {
            taxonomy
                .resolve_title_from(&folded, SourceName::Esco)
                .or_else(|| taxonomy.resolve_title_from(&core, SourceName::Esco))
        })
        .flatten();
    Some(TitleGuess {
        seniority: seniority_of(&folded),
        onet: resolve_occupation(&title, taxonomy, context).and_then(|r| {
            let code = r.taxonomy_ref.strip_prefix(ONET_SOC_PREFIX)?.to_owned();
            Some((code, r.canonical))
        }),
        esco: esco.map(|r| r.taxonomy_ref),
        title,
    })
}
