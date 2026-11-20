//! A tailoring file can only select, order and choose between what the career already holds:
//! an unknown id, index or variant fails to resolve, and a field that could carry a new fact
//! fails to parse, in both the Rust types and the generated JSON Schema.

use atsgate_career::{Career, Tailoring, TailoringError};
use serde_json::Value;

const CAREER: &str = include_str!("../../../fixtures/career/synthetic.career.json");
const TAILORING: &str = include_str!("../../../fixtures/tailoring/synthetic.yaml");
const TBRAUN96: &str = include_str!("../../../fixtures/import/tbraun96.career.json");
const TBRAUN96_GENERAL: &str = include_str!("../../../fixtures/tailoring/tbraun96-general.yaml");
const SCHEMA: &str = include_str!("../../../schemas/tailoring.schema.json");

fn career() -> Career {
    Career::from_json(CAREER).expect("synthetic career")
}

fn resolve(yaml: &str) -> Result<(), TailoringError> {
    let career = career();
    Tailoring::from_yaml(yaml)?.check(&career).map(drop)
}

fn schema_accepts(yaml: &str) -> bool {
    let doc: Value = serde_saphyr::from_str(yaml).expect("YAML");
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema is JSON");
    jsonschema::validator_for(&schema)
        .expect("compiles")
        .is_valid(&doc)
}

/// The synthetic tailoring with one line replaced.
fn with(old: &str, new: &str) -> String {
    assert!(TAILORING.contains(old), "fixture lacks {old:?}");
    TAILORING.replacen(old, new, 1)
}

#[test]
fn the_fixture_resolves_in_tailoring_order() {
    let career = career();
    let tailoring = Tailoring::from_yaml(TAILORING).expect("parses");
    let t = tailoring.check(&career).expect("resolves");
    assert_eq!(t.title, "Senior Data Engineer");
    assert!(t.summary.starts_with("Backend engineer with eight years"));
    let contoso = &t.work[1];
    assert_eq!(contoso.id.as_str(), "contoso");
    assert_eq!(
        contoso.highlights,
        [
            "Mentored four junior engineers.",
            "Built REST APIs in Go for internal billing."
        ]
    );
    let skills: Vec<&str> = t.skills.iter().map(|k| k.name.as_str()).collect();
    assert_eq!(skills, ["Data", "Languages"]);
    assert!(schema_accepts(TAILORING));
}

#[test]
fn the_tbraun96_general_tailoring_resolves() {
    let career = Career::from_json(TBRAUN96).expect("golden career");
    let tailoring = Tailoring::from_yaml(TBRAUN96_GENERAL).expect("parses");
    let t = tailoring.check(&career).expect("resolves");
    assert_eq!((t.work.len(), t.projects.len(), t.max_pages), (8, 7, 2));
    assert!(schema_accepts(TBRAUN96_GENERAL));
}

#[test]
fn an_unknown_id_index_or_variant_is_rejected() {
    let cases = [
        with("id: northwind,", "id: initech,"),
        with("id: ledgerline,", "id: skunkworks,"),
        with("education: [state-university]", "education: [night-school]"),
        with("awards: [excellence]", "awards: [nobel]"),
        with("title: { variant: data }", "title: { variant: rockstar }"),
        with(
            "highlights: [0, 1], stats: [0]",
            "highlights: [0, 7], stats: [0]",
        ),
        with("stats: [0, 1]", "stats: [0, 2]"),
        with("skills: [Data, Languages]", "skills: [Data, Blockchain]"),
        with("skills: [Data, Languages]", "skills: [Data, Data]"),
    ];
    for yaml in &cases {
        let err = resolve(yaml).expect_err(yaml);
        assert!(!matches!(err, TailoringError::Parse(_)), "{err}");
    }
}

#[test]
fn a_field_that_could_carry_a_new_fact_is_rejected() {
    let cases = [
        with(
            "summary: basics",
            "summary: basics\nheadline: World's best engineer",
        ),
        with(
            "title: { variant: data }",
            "title: Chief Everything Officer",
        ),
        with(
            "title: { variant: data }",
            "title: { text: Chief Everything Officer }",
        ),
        with(
            "highlights: [0, 1], stats: [0]",
            "highlights: [Invented the internet], stats: [0]",
        ),
        with("stats: [0] }", "stats: [0], position: CTO }"),
    ];
    for yaml in &cases {
        assert!(
            matches!(resolve(yaml), Err(TailoringError::Parse(_))),
            "parsed: {yaml}"
        );
        assert!(!schema_accepts(yaml), "schema accepted: {yaml}");
    }
}

#[test]
fn every_key_is_required() {
    let yaml = with("max_pages: 1\n", "");
    assert!(matches!(resolve(&yaml), Err(TailoringError::Parse(_))));
    assert!(!schema_accepts(&yaml));
    let zero = with("max_pages: 1", "max_pages: 0");
    assert!(matches!(resolve(&zero), Err(TailoringError::MaxPages)));
    let absolute = with("builtin:data-engineering", "/etc/blueprint.yaml");
    assert!(matches!(
        resolve(&absolute),
        Err(TailoringError::Blueprint(_))
    ));
}
