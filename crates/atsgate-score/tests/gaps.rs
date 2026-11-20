//! A gap lists the spellings the matcher would have accepted for the missing skill.

mod common;

use atsgate_blueprint::{EvidenceScope, Skill};
use atsgate_score::gaps::WOULD_MATCH_CAP;

fn skill(canonical: &str, taxonomy_ref: &str) -> Skill {
    Skill {
        canonical: canonical.to_owned(),
        taxonomy_ref: Some(taxonomy_ref.to_owned()),
        aliases: Vec::new(),
        weight: 1.0,
        evidence_scope: EvidenceScope::Anywhere,
    }
}

fn gap_for(s: Skill) -> Vec<String> {
    let mut b = common::blueprint("software-engineer-backend");
    let canonical = s.canonical.clone();
    b.required_skills = vec![s];
    b.preferred_skills.clear();
    let r = common::score_text(
        "EXPERIENCE\nBaker, Bakery 2020 - 2021\n- Baked bread.\n",
        &b,
    );
    let gap = r.gaps.iter().find(|g| g.canonical == canonical);
    gap.expect("the skill is a gap").would_match.clone()
}

#[test]
fn would_match_includes_the_taxonomy_aliases_the_matcher_accepts() {
    let got = gap_for(skill(
        "cloud technologies",
        "http://data.europa.eu/esco/skill/bd14968e-e409-45af-b362-3495ed7b10e0",
    ));
    assert_eq!(
        got,
        ["cloud technologies", "cloud computing", "edge computing"]
    );
}

#[test]
fn would_match_is_deduplicated_and_capped() {
    // ESCO statistics carries more aliases than the cap; folded duplicates appear once.
    let got = gap_for(skill(
        "statistics",
        "http://data.europa.eu/esco/skill/7ee4c2ea-b349-4bd2-81a3-ec31475d4833",
    ));
    assert_eq!(got.len(), WOULD_MATCH_CAP);
    assert_eq!(got[0], "statistics");
    let mut folded: Vec<String> = got.iter().map(|s| atsgate_taxonomy::fold(s)).collect();
    folded.sort();
    folded.dedup();
    assert_eq!(folded.len(), got.len());
}
