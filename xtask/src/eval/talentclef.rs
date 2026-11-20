//! TalentCLEF 2025 Task A (CC BY 4.0), English validation split: job-title normalization as
//! retrieval. A query and a corpus title are "the same job" when both resolve to the same
//! occupation; corpus titles that do are ranked first (in corpus order), the rest after.
//! Mean average precision (MAP) over queries, against the official qrels.

use atsgate_taxonomy::{ConceptId, SourceName, Taxonomy, fold};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Split {
    pub queries: Vec<(String, String)>,
    pub corpus: Vec<(String, String)>,
    /// query id -> relevant corpus ids
    pub qrels: BTreeMap<String, BTreeSet<String>>,
}

fn tsv_pairs(text: &str) -> Vec<(String, String)> {
    text.lines()
        .skip(1)
        .filter_map(|l| l.split_once('\t'))
        .map(|(a, b)| (a.trim().to_owned(), b.trim().to_owned()))
        .collect()
}

pub fn parse(queries: &str, corpus: &str, qrels: &str) -> Split {
    let mut rel: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for l in qrels.lines() {
        let f: Vec<&str> = l.split('\t').collect();
        if f.len() == 4 && f[3].trim() != "0" {
            rel.entry(f[0].to_owned())
                .or_default()
                .insert(f[2].to_owned());
        }
    }
    Split {
        queries: tsv_pairs(queries),
        corpus: tsv_pairs(corpus),
        qrels: rel,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scores {
    pub queries: usize,
    pub resolved: usize,
    pub map: f64,
    /// The same ranking with exact folded-string equality instead of the taxonomy.
    pub string_baseline_map: f64,
    /// Resolving against ESCO alone. The qrels group ESCO occupation labels and the taxonomy
    /// embeds those labels, so this measures lookup of known labels, not generalization.
    pub esco_only_map: f64,
}

fn average_precision(ranked: &[&str], relevant: &BTreeSet<String>) -> f64 {
    if relevant.is_empty() {
        return 0.0;
    }
    let mut hits = 0.0;
    let mut sum = 0.0;
    for (i, id) in ranked.iter().enumerate() {
        if relevant.contains(*id) {
            hits += 1.0;
            sum += hits / (i as f64 + 1.0);
        }
    }
    sum / relevant.len() as f64
}

fn map_with<K: PartialEq>(split: &Split, key: impl Fn(&str) -> Option<K>) -> (usize, f64) {
    let corpus_keys: Vec<Option<K>> = split.corpus.iter().map(|(_, t)| key(t)).collect();
    let mut resolved = 0;
    let mut total = 0.0;
    let mut n = 0;
    for (qid, title) in &split.queries {
        let Some(relevant) = split.qrels.get(qid) else {
            continue;
        };
        n += 1;
        let q = key(title);
        resolved += usize::from(q.is_some());
        let (mut first, mut rest) = (Vec::new(), Vec::new());
        for ((cid, _), k) in split.corpus.iter().zip(&corpus_keys) {
            let same = q.is_some() && k.as_ref() == q.as_ref();
            if same { &mut first } else { &mut rest }.push(cid.as_str());
        }
        first.extend(rest);
        total += average_precision(&first, relevant);
    }
    (resolved, if n == 0 { 0.0 } else { total / n as f64 })
}

pub fn evaluate(split: &Split, taxonomy: &Taxonomy) -> Scores {
    let concept = |t: &str| -> Option<ConceptId> { taxonomy.resolve_title(t).map(|r| r.concept) };
    let (resolved, map) = map_with(split, concept);
    let (_, string_baseline_map) = map_with(split, |t| Some(fold(t)));
    let esco = |t: &str| -> Option<ConceptId> {
        taxonomy
            .resolve_title_from(t, SourceName::Esco)
            .map(|r| r.concept)
    };
    let (_, esco_only_map) = map_with(split, esco);
    Scores {
        queries: split
            .queries
            .iter()
            .filter(|(q, _)| split.qrels.contains_key(q))
            .count(),
        resolved,
        map,
        string_baseline_map,
        esco_only_map,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn average_precision_rewards_early_hits() {
        let rel: BTreeSet<String> = ["a".to_owned(), "c".to_owned()].into();
        assert_eq!(average_precision(&["a", "c", "b"], &rel), 1.0);
        assert!((average_precision(&["b", "a", "c"], &rel) - (0.5 + 2.0 / 3.0) / 2.0).abs() < 1e-9);
    }
}
