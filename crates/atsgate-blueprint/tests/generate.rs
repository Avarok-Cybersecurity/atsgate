//! The JD -> blueprint generator, end to end over the real taxonomy (no mocks).

use atsgate_blueprint::generate::{GenerateError, GenerateOptions, MIN_SHARED_SKILLS, generate};
use atsgate_blueprint::{Blueprint, KnockoutType, ProvenanceSource, Seniority};
use atsgate_extract::{Format, Thresholds, extract};
use atsgate_taxonomy::{Gazetteer, Taxonomy};

fn options(category: Option<&str>) -> GenerateOptions {
    GenerateOptions {
        id: "test".to_owned(),
        version: "0.1.0".to_owned(),
        // Test choice: a value no posting below states, so a leak of the fallback is visible.
        fallback_seniority: Seniority::Mid,
        occupational_category: category.map(str::to_owned),
    }
}

fn run(text: &str, category: Option<&str>) -> Result<Blueprint, GenerateError> {
    let gazetteer = Gazetteer::skills(Taxonomy::builtin().expect("taxonomy"));
    generate(text, &options(category), &gazetteer)
}

fn names(skills: &[atsgate_blueprint::Skill]) -> Vec<&str> {
    skills.iter().map(|s| s.canonical.as_str()).collect()
}

const SPLIT: &str = "Senior Software Engineer\n\
    We build payment infrastructure.\n\
    Requirements\n\
    - 5+ years of experience with Python and PostgreSQL\n\
    - BS degree in Computer Science or equivalent experience\n\
    Nice to have\n\
    - Kubernetes and Kafka\n\
    Benefits\n\
    - Free Salesforce training\n";

#[test]
fn section_cues_split_required_from_preferred() {
    let b = run(SPLIT, None).expect("generates");
    assert_eq!(names(&b.required_skills), ["Python", "PostgreSQL"]);
    assert_eq!(names(&b.preferred_skills), ["Kubernetes", "Apache Kafka"]);
    assert!(!format!("{b:?}").contains("Salesforce"), "benefits leaked");
}

#[test]
fn inline_cue_marks_a_line_preferred() {
    let text =
        "Data Scientist\nRequirements\nSQL and statistics.\nExperience with Tableau is a plus.\n";
    let b = run(text, None).expect("generates");
    assert!(names(&b.preferred_skills).contains(&"Tableau"));
    assert!(!names(&b.required_skills).contains(&"Tableau"));
}

#[test]
fn years_degree_and_seniority_are_read() {
    let b = run(SPLIT, None).expect("generates");
    assert_eq!(b.seniority, Seniority::Senior);
    assert_eq!(b.experience.min_years, 5.0);
    assert!(
        b.knockouts
            .iter()
            .any(|k| k.kind == KnockoutType::MinYears && k.value == "5")
    );
    let level = b.education.min_level.expect("degree read");
    assert_eq!(level.level, 6);
    assert!(b.education.experience_in_place_of_education);
    assert_eq!(b.occupational_category, "15-1252.00");
}

#[test]
fn output_is_unreviewed_generated_provenance_and_deterministic() {
    let a = run(SPLIT, None).expect("generates");
    assert_eq!(a.provenance.source, ProvenanceSource::Generated);
    assert!(!a.provenance.reviewed);
    assert!(a.provenance.generator_version.is_some());
    assert_eq!(a, run(SPLIT, None).expect("generates"));
}

#[test]
fn without_cues_every_skill_is_required() {
    let text = "Frontend Engineer\nYou will build our app with React and TypeScript.\n";
    let b = run(text, None).expect("generates");
    assert_eq!(names(&b.required_skills), ["React", "TypeScript"]);
    assert!(b.preferred_skills.is_empty());
}

#[test]
fn certifications_are_collected() {
    let text = "Staff Accountant\nRequirements\nCPA required. QuickBooks and Excel.\n";
    let b = run(text, None).expect("generates");
    assert_eq!(b.certifications, ["CPA"]);
    assert_eq!(b.occupational_category, "13-2011.00");
}

#[test]
fn occupation_is_inferred_from_skills_when_the_title_is_vague() {
    // "Senior Engineer" names no occupation; the skills choose one whose O*NET Technology
    // Skills include at least MIN_SHARED_SKILLS of them.
    let text = "Senior Engineer\nStack: Rust, TypeScript, React, PostgreSQL.\n";
    let b = run(text, None).expect("generates");
    let taxonomy = Taxonomy::builtin().expect("taxonomy");
    let occupation = taxonomy
        .by_ref(&format!("onet-soc:{}", b.occupational_category))
        .expect("an O*NET occupation");
    let listed: Vec<&str> = occupation
        .skills()
        .iter()
        .map(|&id| taxonomy.concept(id).canonical())
        .collect();
    let shared = names(&b.required_skills)
        .iter()
        .filter(|s| listed.contains(s))
        .count();
    assert!(
        shared >= MIN_SHARED_SKILLS,
        "{} shares {shared}",
        b.occupational_category
    );
}

#[test]
fn an_explicit_occupation_overrides_inference() {
    let b = run(SPLIT, Some("15-1253.00")).expect("generates");
    assert_eq!(b.occupational_category, "15-1253.00");
}

#[test]
fn unknown_occupation_and_missing_title_are_errors() {
    assert!(matches!(
        run("Dragon Wrangler Specialist\nMagic.\n", None),
        Err(GenerateError::UnknownOccupation)
    ));
    assert!(matches!(run("", None), Err(GenerateError::NoTitle)));
}

#[test]
fn an_html_posting_generates_through_the_extractor() {
    let html = "<html><body><h1>Senior Backend Engineer</h1><h2>Requirements</h2>\
        <ul><li>Go and PostgreSQL</li></ul><h2>Nice to have</h2><ul><li>Redis</li></ul></body></html>";
    let doc = extract(html.as_bytes(), Format::Html, &Thresholds::STANDARD).expect("extracts");
    let b = run(&doc.reading_text(), None).expect("generates");
    assert_eq!(names(&b.required_skills), ["Go", "PostgreSQL"]);
    assert_eq!(names(&b.preferred_skills), ["Redis"]);
}

#[test]
fn artcraft_required_stack_regression() {
    let text = include_str!("../../../tests/fixtures/jd/artcraft-hn-2026-11-02.txt");
    let body = text.split_once("\n---\n").expect("fixture header").1;
    let b = run(body, None).expect("generates");
    assert_eq!(
        names(&b.required_skills),
        [
            "Rust",
            "TypeScript",
            "React",
            "egui",
            "Tauri",
            "MySQL",
            "PostgreSQL",
            "Redis",
            "Elasticsearch",
            "Kubernetes"
        ]
    );
    assert_eq!(b.seniority, Seniority::Senior);
    assert_eq!(b.occupational_category, "15-1252.00");
}
