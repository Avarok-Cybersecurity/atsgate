//! Competence: a resume scored against a blueprint for a different occupation falls below every
//! vendor floor, while it clears more of them against its own occupation.

mod common;

use atsgate_extract::Format;
use atsgate_score::{Floors, gate};

const RESUME: &[u8] = include_bytes!("../../../fixtures/pdf/one-column.pdf");

#[test]
fn a_backend_resume_is_below_every_floor_for_unrelated_occupations() {
    for id in [
        "accountant",
        "paralegal",
        "marketing-manager",
        "sales-representative",
        "product-designer",
    ] {
        let r = common::score_bytes(Format::Pdf, RESUME, &common::blueprint(id));
        let verdict = gate(&r, &Floors::defaults(), true);
        assert!(!verdict.passed, "{id}");
        for v in &verdict.vendors {
            assert!(
                !v.passed,
                "{id}: {} scored {} against floor {}",
                v.name, v.score, v.floor
            );
        }
    }
}

#[test]
fn the_matching_occupation_scores_higher_than_any_unrelated_one() {
    let own = common::score_bytes(
        Format::Pdf,
        RESUME,
        &common::blueprint("software-engineer-backend"),
    );
    for id in [
        "accountant",
        "paralegal",
        "marketing-manager",
        "sales-representative",
        "product-designer",
    ] {
        let other = common::score_bytes(Format::Pdf, RESUME, &common::blueprint(id));
        for (a, b) in own.vendors.iter().zip(&other.vendors) {
            assert!(
                a.score > b.score,
                "{id}: {} {} vs {}",
                a.name,
                a.score,
                b.score
            );
        }
    }
}

#[test]
fn missing_skills_are_gaps_with_the_spellings_that_would_match() {
    let r = common::score_bytes(
        Format::Pdf,
        RESUME,
        &common::blueprint("software-engineer-backend"),
    );
    let kafka = r
        .gaps
        .iter()
        .find(|g| g.canonical == "Apache Kafka")
        .expect("Kafka is a gap");
    assert_eq!(kafka.would_match, ["Apache Kafka", "Kafka"]);
    assert!(r.gaps.iter().all(|g| {
        r.skills
            .iter()
            .any(|s| s.canonical == g.canonical && s.matched.is_none())
    }));
    let required_first = r
        .gaps
        .iter()
        .position(|g| g.requirement == atsgate_score::skills::Requirement::Preferred);
    assert!(required_first.is_none_or(|i| {
        r.gaps[..i]
            .iter()
            .all(|g| g.requirement == atsgate_score::skills::Requirement::Required)
    }));
}

#[test]
fn every_point_carries_evidence() {
    let r = common::score_bytes(
        Format::Pdf,
        RESUME,
        &common::blueprint("software-engineer-backend"),
    );
    for s in r.skills.iter().filter(|s| s.matched.is_some()) {
        let e = s.evidence.as_ref().expect("a matched skill has evidence");
        assert!(
            e.page == Some(1) && e.bbox.is_some() && !e.spans.is_empty(),
            "{}",
            s.canonical
        );
    }
    for d in [
        &r.dimensions.title,
        &r.dimensions.experience,
        &r.dimensions.education,
    ] {
        assert!(
            d.evidence
                .iter()
                .any(|e| e.page.is_some() && e.bbox.is_some()),
            "{d:?}"
        );
    }
}

#[test]
fn repeating_a_skill_gains_nothing() {
    let blueprint = common::blueprint("software-engineer-backend");
    let once = "EXPERIENCE\nSoftware Engineer, Acme Jan 2020 - Present\n- Built services in Go.\n";
    let many = "EXPERIENCE\nSoftware Engineer, Acme Jan 2020 - Present\n- Built services in Go.\n\
                - Wrote more Go. Then Go, Go and Go again.\n\nSKILLS\nGo, Go, Golang\n";
    let a = common::score_text(once, &blueprint);
    let b = common::score_text(many, &blueprint);
    assert_eq!(
        a.dimensions.preferred_skills.score,
        b.dimensions.preferred_skills.score
    );
}

#[test]
fn a_skills_list_mention_earns_less_than_experience_where_the_blueprint_asks_for_it() {
    // Python is `experience_or_projects` in the ML blueprint.
    let blueprint = common::blueprint("ml-inference-engineer");
    let listed = common::score_text("SKILLS\nPython\n", &blueprint);
    let used = common::score_text(
        "EXPERIENCE\nEngineer, Acme Jan 2020 - Present\n- Served models in Python.\n",
        &blueprint,
    );
    let credit = |r: &atsgate_score::Report| {
        r.skills
            .iter()
            .find(|s| s.canonical == "Python")
            .expect("python")
            .credit
    };
    assert_eq!(credit(&listed), 0.8);
    assert_eq!(credit(&used), 1.0);
}
