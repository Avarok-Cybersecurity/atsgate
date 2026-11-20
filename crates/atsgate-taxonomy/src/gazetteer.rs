//! A deterministic gazetteer: finds skill names in free text by longest match over folded tokens.

use crate::data::ConceptKind;
use crate::fold::{fold, tokens};
use crate::model::{ConceptId, Namespace, Taxonomy};
use std::collections::HashMap;

/// The longest phrase, in tokens, the gazetteer looks for.
pub const MAX_PHRASE_TOKENS: usize = 6;

/// The longest name, in characters, that [`is_acronym`] treats as an acronym.
pub const ACRONYM_MAX_CHARS: usize = 6;

/// Words that never make a phrase on their own.
const STOPWORDS: [&str; 24] = [
    "a", "an", "and", "as", "at", "be", "by", "for", "from", "in", "is", "it", "of", "on", "or",
    "the", "to", "with", "we", "you", "our", "your", "use", "using",
];

/// A skill found in text: the concept and the byte range of the source text that named it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mention {
    pub concept: ConceptId,
    pub start: usize,
    pub end: usize,
}

/// How a phrase must be spelled in the text to count.
#[derive(Debug, Clone)]
struct CaseRule {
    spellings: Vec<String>,
    /// Ordinary words ("Go", "Rust") also may not open a sentence.
    not_sentence_initial: bool,
}

#[derive(Debug, Clone)]
struct Phrase {
    concept: ConceptId,
    case: Option<CaseRule>,
}

#[derive(Debug)]
pub struct Gazetteer<'t> {
    taxonomy: &'t Taxonomy,
    phrases: HashMap<String, Phrase>,
}

fn eligible_key(key: &str) -> bool {
    let words: Vec<&str> = key.split(' ').collect();
    let has_content = words.iter().any(|w| !STOPWORDS.contains(w));
    has_content && words.len() <= MAX_PHRASE_TOKENS && key.chars().count() >= 2
}

/// "RSS", "DID", "AWS": short all-capital names, which as lowercase text are ordinary words.
pub fn is_acronym(name: &str) -> bool {
    let letters = name.chars().filter(|c| c.is_alphabetic()).count();
    letters >= 2
        && name.chars().count() <= ACRONYM_MAX_CHARS
        && !name.chars().any(char::is_lowercase)
}

/// True when the text before `start` ends a sentence (or there is none).
fn sentence_initial(text: &str, start: usize) -> bool {
    let before = text[..start].trim_end_matches([' ', '\t', '"', '\u{201C}', '(']);
    before.is_empty() || before.ends_with(['.', '!', '?', '\n']) || text[..start].ends_with('\n')
}

fn case_rule(spellings: Vec<String>, curated: Option<&String>) -> Option<CaseRule> {
    match curated {
        Some(word) => Some(CaseRule {
            spellings: vec![word.clone()],
            not_sentence_initial: true,
        }),
        None => spellings.iter().all(|s| is_acronym(s)).then_some(CaseRule {
            spellings,
            not_sentence_initial: false,
        }),
    }
}

impl<'t> Gazetteer<'t> {
    /// Indexes every concrete skill (not skill groups) by its canonical name and aliases. A key
    /// shared by several concepts goes to the one [`Taxonomy::resolve_skill`] would choose.
    /// Acronym names and the curated `case_sensitive` phrases must match their spelling.
    pub fn skills(taxonomy: &'t Taxonomy) -> Self {
        let curated_case: HashMap<String, &String> = taxonomy
            .case_sensitive
            .iter()
            .map(|p| (fold(p), p))
            .collect();
        let mut phrases = HashMap::new();
        for (ns, key) in taxonomy.index.keys() {
            if *ns != Namespace::Skill || taxonomy.stoplist.contains(key) {
                continue;
            }
            if !(eligible_key(key) || curated_case.contains_key(key)) {
                continue;
            }
            let Some(e) = taxonomy
                .entries(Namespace::Skill, key)
                .into_iter()
                .find(|e| taxonomy.concept(e.concept).kind() == ConceptKind::Skill)
            else {
                continue;
            };
            let c = taxonomy.concept(e.concept);
            let spellings: Vec<String> = std::iter::once(c.canonical())
                .chain(c.aliases())
                .filter(|n| fold(n) == *key)
                .map(str::to_owned)
                .collect();
            let phrase = Phrase {
                concept: e.concept,
                case: case_rule(spellings, curated_case.get(key).copied()),
            };
            phrases.insert(key.clone(), phrase);
        }
        Gazetteer { taxonomy, phrases }
    }

    pub fn taxonomy(&self) -> &'t Taxonomy {
        self.taxonomy
    }

    /// Every skill mention in `text`, left to right, longest phrase first, never overlapping.
    pub fn find(&self, text: &str) -> Vec<Mention> {
        let toks = tokens(text);
        let mut out = Vec::new();
        let mut i = 0;
        while i < toks.len() {
            let longest = MAX_PHRASE_TOKENS.min(toks.len() - i);
            let hit = (1..=longest).rev().find_map(|n| {
                let span = &toks[i..i + n];
                let key = span
                    .iter()
                    .map(|t| t.folded.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
                let phrase = self.phrases.get(&key)?;
                let (start, end) = (span[0].start, span[n - 1].end);
                if let Some(rule) = &phrase.case
                    && (!rule.spellings.iter().any(|s| &text[start..end] == s)
                        || (rule.not_sentence_initial && sentence_initial(text, start)))
                {
                    return None;
                }
                Some((
                    n,
                    Mention {
                        concept: phrase.concept,
                        start,
                        end,
                    },
                ))
            });
            match hit {
                Some((n, m)) => {
                    out.push(m);
                    i += n;
                }
                None => i += 1,
            }
        }
        out
    }
}
