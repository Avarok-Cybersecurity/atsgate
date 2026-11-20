//! Regression thresholds over the committed samples, so a change to the taxonomy or gazetteer
//! that loses quality fails offline. Each floor sits a little under the value measured on the
//! sample when it was committed (shown beside it); docs/evaluation.md has the full-set numbers.

use super::{SAMPLE_DIR, skillspan, talentclef, techwolf};
use crate::root;
use atsgate_taxonomy::{Gazetteer, Scope, Taxonomy};

fn sample(name: &str) -> String {
    let path = root().join(SAMPLE_DIR).join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn ratio(n: usize, d: usize) -> f64 {
    assert!(d > 0, "empty sample");
    n as f64 / d as f64
}

fn taxonomy() -> &'static Taxonomy {
    Taxonomy::builtin().expect("taxonomy")
}

#[test]
fn skillspan_sample_meets_its_floor() {
    let sentences = skillspan::parse(&sample("skillspan-test-sample.jsonl")).expect("parses");
    let all = skillspan::evaluate(&sentences, &Gazetteer::new(taxonomy(), Scope::Postings))[2].1;
    let precision = ratio(all.predicted_overlapping, all.predicted);
    let recall = ratio(all.gold_overlapped, all.gold);
    println!("skillspan sample: overlap precision {precision:.3}, recall {recall:.3}");
    assert!(
        precision >= SKILLSPAN_OVERLAP_PRECISION_FLOOR,
        "{precision}"
    );
    assert!(recall >= SKILLSPAN_OVERLAP_RECALL_FLOOR, "{recall}");
}

#[test]
fn techwolf_sample_meets_its_floor() {
    let rows = techwolf::parse(sample("techwolf-sample.csv").as_bytes()).expect("parses");
    let c = techwolf::evaluate(&rows, &Gazetteer::new(taxonomy(), Scope::Postings));
    let recall = ratio(c.found_related, c.resolvable);
    let precision = ratio(c.predicted_target, c.predicted);
    println!("techwolf sample: related recall {recall:.3}, precision {precision:.3}");
    assert!(recall >= TECHWOLF_RELATED_RECALL_FLOOR, "{recall}");
    assert!(precision >= TECHWOLF_PRECISION_FLOOR, "{precision}");
}

#[test]
fn talentclef_validation_beats_the_string_baseline() {
    let dir = "talentclef-taskA-en-validation";
    let read = |f: &str| sample(&format!("{dir}/{f}"));
    let split = talentclef::parse(
        &read("queries"),
        &read("corpus_elements"),
        &read("qrels.tsv"),
    );
    let s = talentclef::evaluate(&split, taxonomy());
    println!(
        "talentclef: MAP {:.3}, baseline {:.3}",
        s.map, s.string_baseline_map
    );
    assert!(s.map >= TALENTCLEF_MAP_FLOOR, "{}", s.map);
    assert!(s.map > s.string_baseline_map);
}

// The recall floors were lowered (from 0.23 and 0.30) and the precision floors raised (from 0.45
// and 0.40) when the postings gazetteer stopped reporting ESCO alternative labels, one-word labels
// and transversal skills: the generator is scored against these sets, and what they lost was
// mostly the noise ("email", "curiosity", "radar") that real postings turned into requirements.
// Both SkillSpan subsets and TechWolf's ESCO targets annotate soft skills, so their recall
// counts that trade as a loss; docs/evaluation.md records the before and after.
const SKILLSPAN_OVERLAP_PRECISION_FLOOR: f64 = 0.59; // measured 0.619
const SKILLSPAN_OVERLAP_RECALL_FLOOR: f64 = 0.12; // measured 0.139
const TECHWOLF_RELATED_RECALL_FLOOR: f64 = 0.17; // measured 0.190
const TECHWOLF_PRECISION_FLOOR: f64 = 0.65; // measured 0.688
const TALENTCLEF_MAP_FLOOR: f64 = 0.035; // measured 0.040
