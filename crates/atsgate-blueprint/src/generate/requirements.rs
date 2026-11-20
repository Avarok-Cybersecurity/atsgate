//! Years of experience, degrees, certifications and languages, read line by line.

use atsgate_taxonomy::{fold, tokens};

/// Folded degree words and the ISCED 2011 level they name. `needs_context` words ("BS", "MS")
/// count only on a line that also says "degree" or is followed by "in".
pub const DEGREES: [(&str, u8, bool); 22] = [
    ("phd", 8, false),
    ("ph.d", 8, false),
    ("doctorate", 8, false),
    ("doctoral", 8, false),
    ("masters", 7, false),
    ("master", 7, true),
    ("mba", 7, false),
    ("msc", 7, false),
    ("ms", 7, true),
    ("m.s", 7, true),
    ("jd", 7, true),
    ("j.d", 7, false),
    ("juris", 7, false),
    ("bachelors", 6, false),
    ("bachelor", 6, true),
    ("bsc", 6, false),
    ("bs", 6, true),
    ("b.s", 6, true),
    ("ba", 6, true),
    ("b.a", 6, true),
    ("associates", 5, true),
    ("associate", 5, true),
];

const EQUIVALENT_CUES: [&str; 4] = [
    "or equivalent",
    "equivalent experience",
    "equivalent practical experience",
    "or related experience",
];

/// Certifications recognized by name, as written (matched on folded tokens).
pub const CERTIFICATIONS: [&str; 16] = [
    "CPA",
    "CFA",
    "CMA",
    "CIA",
    "PMP",
    "CISSP",
    "CISM",
    "CKA",
    "CKAD",
    "SHRM-CP",
    "SHRM-SCP",
    "Series 7",
    "Series 63",
    "Certified Paralegal",
    "AWS Certified Solutions Architect",
    "Google Analytics Certification",
];

/// Language names recognized when the line also asks for fluency.
pub const LANGUAGES: [&str; 14] = [
    "English",
    "Spanish",
    "French",
    "German",
    "Mandarin",
    "Chinese",
    "Japanese",
    "Portuguese",
    "Italian",
    "Korean",
    "Arabic",
    "Hindi",
    "Russian",
    "Dutch",
];

const FLUENCY_CUES: [&str; 7] = [
    "fluent",
    "fluency",
    "proficient",
    "proficiency",
    "bilingual",
    "native",
    "speak",
];

fn number(word: &str) -> Option<f64> {
    word.trim_end_matches('+').parse::<f64>().ok()
}

/// The smallest stated minimum on a line: "5+ years", "3-5 years", "at least 2 yrs".
pub fn years(line: &str) -> Option<f64> {
    let words: Vec<String> = tokens(line).into_iter().map(|t| t.folded).collect();
    let folded = words.join(" ");
    if !(folded.contains("experience") || folded.contains("years of")) {
        return None;
    }
    words
        .iter()
        .enumerate()
        .filter_map(|(i, w)| {
            let n = number(w)?;
            let unit_at = words[i + 1..]
                .iter()
                .take(3)
                .position(|u| matches!(u.as_str(), "years" | "year" | "yrs" | "yr"))?;
            let between = &words[i + 1..i + 1 + unit_at];
            between
                .iter()
                .all(|b| b == "plus" || b == "to" || b == "+" || number(b).is_some())
                .then_some(n)
        })
        .filter(|n| (0.0..=40.0).contains(n))
        .reduce(f64::min)
}

/// The lowest degree level a line accepts, and the fields named after "in".
pub fn degree(line: &str) -> Option<(u8, Vec<String>)> {
    let words: Vec<String> = tokens(line).into_iter().map(|t| t.folded).collect();
    let has_degree_word = words.iter().any(|w| w == "degree" || w == "degrees");
    let level = words
        .iter()
        .enumerate()
        .filter_map(|(i, w)| {
            let (_, level, needs_context) = DEGREES.iter().find(|(d, _, _)| d == w)?;
            let followed_by_in = words.get(i + 1).is_some_and(|n| n == "in");
            (!needs_context || has_degree_word || followed_by_in).then_some(*level)
        })
        .min()?;
    Some((level, fields(line)))
}

fn fields(line: &str) -> Vec<String> {
    let Some((_, after)) = line.split_once(" in ") else {
        return Vec::new();
    };
    let end = after.find(['.', ';', '(', ':']).unwrap_or(after.len());
    let mut list = after[..end].to_owned();
    for cut in [" or equivalent", " or related", ", or equivalent"] {
        if let Some(i) = list.find(cut) {
            list.truncate(i);
        }
    }
    list.split([','])
        .flat_map(|s| s.split(" or "))
        .flat_map(|s| s.split(" and "))
        .map(|s| s.trim().trim_start_matches("a ").trim())
        .filter(|s| !s.is_empty() && s.split_whitespace().count() <= 5)
        .take(5)
        .map(str::to_owned)
        .collect()
}

pub fn equivalent_experience(line: &str) -> bool {
    let folded = fold(line);
    EQUIVALENT_CUES.iter().any(|c| folded.contains(c))
}

fn contains_phrase(folded_line: &str, phrase: &str) -> bool {
    format!(" {folded_line} ").contains(&format!(" {} ", fold(phrase)))
}

pub fn certifications(line: &str) -> Vec<String> {
    let folded = fold(line);
    CERTIFICATIONS
        .iter()
        .filter(|c| contains_phrase(&folded, c) && line.contains(*c))
        .map(|c| (*c).to_owned())
        .collect()
}

pub fn languages(line: &str) -> Vec<String> {
    let folded = fold(line);
    if !FLUENCY_CUES.iter().any(|c| contains_phrase(&folded, c)) {
        return Vec::new();
    }
    LANGUAGES
        .iter()
        .filter(|l| contains_phrase(&folded, l))
        .map(|l| (*l).to_owned())
        .collect()
}
