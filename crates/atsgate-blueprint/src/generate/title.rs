//! The job title and seniority, read from the first lines of a job description.

use crate::Seniority;
use atsgate_taxonomy::{SourceName, Taxonomy, fold};

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
const GENERIC_TITLES: [&str; 6] = [
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
        .filter(|(_, s)| *s <= Seniority::Lead)
    {
        out = out.replace(&format!(" {w} "), " ");
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn segments(line: &str) -> impl Iterator<Item = &str> {
    line.split(['|', '\u{2014}', '\u{2013}', '(', ')', ','])
        .flat_map(|s| s.split(" - "))
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

/// The first segment of the leading lines that names a title.
pub fn guess(lines: &[&str], taxonomy: &Taxonomy) -> Option<TitleGuess> {
    let segment = lines
        .iter()
        .take(TITLE_LINES)
        .flat_map(|l| segments(l))
        .find(|s| {
            let words: Vec<String> = s.split_whitespace().map(|w| fold(singular(w))).collect();
            words.len() <= 6 && words.iter().any(|w| TITLE_HEADS.contains(&w.as_str()))
        })?;
    let title = segment
        .split_whitespace()
        .map(singular)
        .collect::<Vec<_>>()
        .join(" ");
    let folded = fold(&title);
    let core = strip_seniority(&folded);
    let specific = !core.is_empty() && !GENERIC_TITLES.contains(&core.as_str());
    let resolve = |source| {
        specific
            .then(|| {
                taxonomy
                    .resolve_title_from(&folded, source)
                    .or_else(|| taxonomy.resolve_title_from(&core, source))
            })
            .flatten()
    };
    Some(TitleGuess {
        seniority: seniority_of(&folded),
        onet: resolve(SourceName::Onet).and_then(|r| {
            let code = r.taxonomy_ref.strip_prefix("onet-soc:")?.to_owned();
            Some((code, r.canonical))
        }),
        esco: resolve(SourceName::Esco).map(|r| r.taxonomy_ref),
        title,
    })
}
