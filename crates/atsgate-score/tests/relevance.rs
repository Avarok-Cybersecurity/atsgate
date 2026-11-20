//! Which roles count as relevant experience, and how seniority moves T.

mod common;

use atsgate_blueprint::Seniority;
use atsgate_score::title::{LADDER_SPAN, OVER_SENIORITY_FACTOR};

/// Two Rust roles years apart under titles that name no occupation, and one unrelated role.
const TWO_RUST_ROLES: &str = "EXPERIENCE\n\
    Founder, Acme Jan 2022 - Present\n\
    - Built a networking SDK in Rust.\n\
    Consultant, Initech Jan 2016 - Dec 2019\n\
    - Wrote Rust services for clients.\n\
    Barista, Cafe Jan 2010 - Dec 2011\n\
    - Made coffee.\n";

#[test]
fn every_role_that_evidences_a_blueprint_skill_counts_toward_years() {
    let r = common::score_text(
        TWO_RUST_ROLES,
        &common::blueprint("software-engineer-systems-rust"),
    );
    // Jan 2022 - Nov 2026 (59 months) and Jan 2016 - Dec 2019 (48 months); the barista role
    // names no blueprint skill and stays out.
    assert_eq!(r.relevant_years, 107.0 / 12.0);
    let roles: Vec<&str> = r
        .dimensions
        .experience
        .evidence
        .iter()
        .map(|e| e.text.as_str())
        .collect();
    assert_eq!(roles, ["Founder (59 months)", "Consultant (48 months)"]);
}

fn title_score(role: &str, wanted: Seniority) -> f64 {
    let mut b = common::blueprint("software-engineer-backend");
    b.seniority = wanted;
    let text = format!("EXPERIENCE\n{role}, Acme Jan 2020 - Dec 2023\n- Built services.\n");
    common::score_text(&text, &b)
        .dimensions
        .title
        .score
        .expect("T")
}

#[test]
fn under_seniority_costs_the_full_step_and_over_seniority_half() {
    let none = title_score("Software Engineer", Seniority::Senior);
    let under = title_score("Junior Software Engineer", Seniority::Senior);
    let over = title_score("Principal Software Engineer", Seniority::Senior);
    // Junior is two rungs under Senior; Principal two over.
    assert_eq!(under, none * (1.0 - 2.0 / LADDER_SPAN));
    assert_eq!(
        over,
        none * (1.0 - OVER_SENIORITY_FACTOR * 2.0 / LADDER_SPAN)
    );
    assert!(over > under);
}

#[test]
fn an_unspecified_seniority_applies_no_penalty() {
    for role in [
        "Junior Software Engineer",
        "Software Engineer",
        "Lead Software Engineer",
    ] {
        assert_eq!(
            title_score(role, Seniority::Unspecified),
            title_score("Software Engineer", Seniority::Senior),
            "{role}"
        );
    }
    let mut b = common::blueprint("software-engineer-backend");
    b.seniority = Seniority::Unspecified;
    let r = common::score_text(
        "EXPERIENCE\nSenior Software Engineer, Acme 2020 - 2023\n- Built services.\n",
        &b,
    );
    let note = &r.dimensions.title.evidence[0].text;
    assert!(note.contains("the blueprint states no seniority"), "{note}");
}

#[test]
fn technology_qualified_and_ambiguous_titles_match_the_occupation() {
    let b = common::blueprint("software-engineer-backend");
    for role in [
        "Principal Rust Software Engineer",
        "Senior Rust Systems Engineer",
        "Staff Engineer",
        "Founding Engineer",
    ] {
        let text = format!("EXPERIENCE\n{role}, Acme 2020 - 2023\n- Built services.\n");
        let r = common::score_text(&text, &b);
        assert_eq!(
            r.dimensions.title.evidence[0].rule, "title.occupation",
            "{role}"
        );
    }
}
