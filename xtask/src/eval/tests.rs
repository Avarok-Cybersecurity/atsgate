//! Regression thresholds over the committed samples, so a change to the taxonomy or gazetteer
//! that loses quality fails offline. Each floor sits a little under the value measured on the
//! sample when it was committed (shown beside it); docs/evaluation.md has the full-set numbers.

use super::{SAMPLE_DIR, skillspan, talentclef, techwolf};
use crate::root;
use atsgate_taxonomy::{Gazetteer, Taxonomy};

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
    let all = skillspan::evaluate(&sentences, &Gazetteer::skills(taxonomy()))[2].1;
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
    let c = techwolf::evaluate(&rows, &Gazetteer::skills(taxonomy()));
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

const SKILLSPAN_OVERLAP_PRECISION_FLOOR: f64 = 0.45; // measured 0.475
const SKILLSPAN_OVERLAP_RECALL_FLOOR: f64 = 0.23; // measured 0.251
const TECHWOLF_RELATED_RECALL_FLOOR: f64 = 0.30; // measured 0.318
const TECHWOLF_PRECISION_FLOOR: f64 = 0.40; // measured 0.427
const TALENTCLEF_MAP_FLOOR: f64 = 0.035; // measured 0.040
