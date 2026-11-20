//! Date tokens, each with its byte range in the source text, so a matched date can be cut out of
//! the original text without disturbing the rest of it (hyphenated words stay hyphenated).

use crate::dates::numeric_month;

/// Splits words, and splits dashes off everything but an ISO year-month ("2016-06").
pub fn tokens(text: &str) -> (Vec<String>, Vec<(usize, usize)>) {
    let (mut out, mut at) = (Vec::new(), Vec::new());
    let base = text.as_ptr() as usize;
    for word in text.split_whitespace() {
        let start = word.as_ptr() as usize - base;
        if numeric_month(word).is_some() {
            out.push(word.to_owned());
            at.push((start, start + word.len()));
            continue;
        }
        let mut offset = start;
        for (i, part) in word.split('-').enumerate() {
            if i > 0 {
                out.push("-".to_owned());
                at.push((offset, offset + 1));
                offset += 1;
            }
            if !part.is_empty() {
                out.push(part.to_owned());
                at.push((offset, offset + part.len()));
            }
            offset += part.len();
        }
    }
    (out, at)
}

/// `text` without the tokens `from..to`, the two sides joined by one space.
pub fn cut(text: &str, at: &[(usize, usize)], from: usize, to: usize) -> String {
    let before = text[..at[from].0].trim_end();
    let after = text[at[to - 1].1..].trim_start();
    match (before.is_empty(), after.is_empty()) {
        (false, false) => format!("{before} {after}"),
        _ => format!("{before}{after}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cutting_a_date_keeps_hyphenated_words_intact() {
        let text = "Co-Founder | Atlas Mar 2026 - Present";
        let (t, at) = tokens(text);
        let from = t.iter().position(|w| w == "Mar").expect("month");
        assert_eq!(cut(text, &at, from, t.len()), "Co-Founder | Atlas");
        let text = "R&D Engineer, ML-KEM 2019 - 2021 (contract)";
        let (t, at) = tokens(text);
        let from = t.iter().position(|w| w == "2019").expect("year");
        assert_eq!(
            cut(text, &at, from, from + 3),
            "R&D Engineer, ML-KEM (contract)"
        );
    }
}
