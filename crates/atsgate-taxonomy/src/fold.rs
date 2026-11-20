//! Case, punctuation and Unicode folding. Labels, aliases and free text all pass through the same
//! tokenizer, so a phrase matches exactly when its folded tokens are equal.

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// A folded token and the byte range of the original text it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub folded: String,
    pub start: usize,
    pub end: usize,
}

/// Characters that join the alphanumerics around them into one token (`node.js`, `r&d`).
const JOINERS: [char; 3] = ['.', '&', '\''];
/// Characters that belong to a token wherever they appear (`c++`, `c#`).
const SUFFIXES: [char; 2] = ['+', '#'];

fn fold_char(c: char, out: &mut String) {
    for d in c.nfkd().filter(|d| !is_combining_mark(*d)) {
        out.extend(d.to_lowercase());
    }
}

/// Splits `text` into folded tokens with their source byte ranges.
pub fn tokens(text: &str) -> Vec<Token> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut out = Vec::new();
    let mut current: Option<Token> = None;
    for (i, &(at, c)) in chars.iter().enumerate() {
        let next_alnum = chars.get(i + 1).is_some_and(|(_, n)| n.is_alphanumeric());
        let in_token = current.is_some();
        let keep = c.is_alphanumeric()
            || SUFFIXES.contains(&c)
            || (c == '.' && next_alnum)
            || (JOINERS.contains(&c) && c != '.' && in_token && next_alnum);
        if !keep {
            out.extend(current.take());
            continue;
        }
        if c == '\'' {
            // An apostrophe joins nothing in the folded form ("Bachelor's" -> "bachelors").
            if let Some(t) = current.as_mut() {
                t.end = at + c.len_utf8();
            }
            continue;
        }
        let token = current.get_or_insert_with(|| Token {
            folded: String::new(),
            start: at,
            end: at,
        });
        fold_char(c, &mut token.folded);
        token.end = at + c.len_utf8();
    }
    out.extend(current);
    out
}

/// The folded form of a whole phrase: its tokens joined by single spaces.
pub fn fold(text: &str) -> String {
    tokens(text)
        .into_iter()
        .map(|t| t.folded)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_case_punctuation_and_diacritics() {
        assert_eq!(fold("  Front-End / UX "), "front end ux");
        assert_eq!(fold("Résumé Écrit"), "resume ecrit");
        assert_eq!(fold("ﬁnance"), "finance");
        assert_eq!(fold("Kubernetes (K8s)"), "kubernetes k8s");
    }

    #[test]
    fn keeps_symbols_that_name_languages() {
        assert_eq!(fold("C++ and C#"), "c++ and c#");
        assert_eq!(fold("Node.js, .NET and R&D."), "node.js .net and r&d");
        assert_eq!(fold("Bachelor's degree"), "bachelors degree");
    }

    #[test]
    fn tokens_point_back_at_the_source() {
        let text = "Use Rust.";
        let t = tokens(text);
        assert_eq!(t.len(), 2);
        assert_eq!(&text[t[1].start..t[1].end], "Rust");
    }
}
