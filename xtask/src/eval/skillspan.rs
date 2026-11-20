//! SkillSpan (Zhang et al., NAACL 2022; MIT): span-level precision and recall of the gazetteer
//! against the union of the annotated skill and knowledge spans.

use atsgate_taxonomy::Gazetteer;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Sentence {
    pub tokens: Vec<String>,
    pub tags_skill: Vec<String>,
    pub tags_knowledge: Vec<String>,
    pub source: String,
}

pub fn parse(jsonl: &str) -> Result<Vec<Sentence>, String> {
    jsonl
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).map_err(|e| format!("SkillSpan: {e}")))
        .collect()
}

/// Token ranges `[start, end)` of B/I runs.
fn spans(tags: &[String]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut open: Option<usize> = None;
    for (i, t) in tags.iter().enumerate() {
        match t.as_str() {
            "B" => {
                if let Some(s) = open.take() {
                    out.push((s, i));
                }
                open = Some(i);
            }
            "I" if open.is_some() => {}
            _ => {
                if let Some(s) = open.take() {
                    out.push((s, i));
                }
            }
        }
    }
    out.extend(open.map(|s| (s, tags.len())));
    out
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Counts {
    pub gold: usize,
    pub predicted: usize,
    pub exact: usize,
    /// Predicted spans that overlap any gold span (precision, loose).
    pub predicted_overlapping: usize,
    /// Gold spans that some prediction overlaps (recall, loose).
    pub gold_overlapped: usize,
}

impl Counts {
    pub fn add(&mut self, o: Counts) {
        self.gold += o.gold;
        self.predicted += o.predicted;
        self.exact += o.exact;
        self.predicted_overlapping += o.predicted_overlapping;
        self.gold_overlapped += o.gold_overlapped;
    }
}

fn overlaps(a: (usize, usize), b: (usize, usize)) -> bool {
    a.0 < b.1 && b.0 < a.1
}

/// Runs the gazetteer on one sentence (tokens joined by spaces) and counts span agreement.
pub fn score(s: &Sentence, gazetteer: &Gazetteer<'_>) -> Counts {
    let mut gold = spans(&s.tags_skill);
    gold.extend(spans(&s.tags_knowledge));
    let mut starts = Vec::new();
    let mut text = String::new();
    for t in &s.tokens {
        if !text.is_empty() {
            text.push(' ');
        }
        starts.push(text.len());
        text.push_str(t);
    }
    let token_at = |byte: usize| starts.iter().rposition(|&s| s <= byte).unwrap_or(0);
    let predicted: Vec<(usize, usize)> = gazetteer
        .find(&text)
        .into_iter()
        .map(|m| (token_at(m.start), token_at(m.end.saturating_sub(1)) + 1))
        .collect();
    Counts {
        gold: gold.len(),
        predicted: predicted.len(),
        exact: predicted.iter().filter(|p| gold.contains(p)).count(),
        predicted_overlapping: predicted
            .iter()
            .filter(|p| gold.iter().any(|g| overlaps(**p, *g)))
            .count(),
        gold_overlapped: gold
            .iter()
            .filter(|g| predicted.iter().any(|p| overlaps(*p, **g)))
            .count(),
    }
}

/// Counts per subset ("house", "tech") and overall, in that order.
pub fn evaluate(sentences: &[Sentence], gazetteer: &Gazetteer<'_>) -> Vec<(String, Counts)> {
    let mut subsets: Vec<(String, Counts)> = vec![
        ("house".into(), Counts::default()),
        ("tech".into(), Counts::default()),
        ("all".into(), Counts::default()),
    ];
    for s in sentences {
        let c = score(s, gazetteer);
        if let Some(sub) = subsets.iter_mut().find(|(n, _)| *n == s.source) {
            sub.1.add(c);
        }
        subsets[2].1.add(c);
    }
    subsets
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags(s: &str) -> Vec<String> {
        s.split(' ').map(str::to_owned).collect()
    }

    #[test]
    fn spans_follow_bio_runs() {
        assert_eq!(spans(&tags("O B I O B B")), vec![(1, 3), (4, 5), (5, 6)]);
        assert_eq!(spans(&tags("I O")), Vec::<(usize, usize)>::new());
    }
}
