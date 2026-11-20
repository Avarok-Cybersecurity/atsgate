//! A blueprint is rejected for a missing key, an unknown field or an invalid value, by both the
//! Rust types and the generated JSON Schema.

use atsgate_blueprint::{Blueprint, BlueprintError};
use serde_json::{Value, json};

const SCHEMA: &str = include_str!("../../../schemas/blueprint.schema.json");

fn valid() -> Value {
    json!({
        "id": "backend-engineer",
        "version": "1.0.0",
        "title": "Backend Engineer",
        "title_aliases": ["Server Engineer"],
        "occupationalCategory": "15-1252.00",
        "esco_occupation": null,
        "seniority": "senior",
        "required_skills": [{ "canonical": "Rust", "taxonomy_ref": null, "aliases": ["rust-lang"], "weight": 2.0, "evidence_scope": "experience_or_projects" }],
        "preferred_skills": [],
        "knockouts": [{ "type": "min_years", "value": "3", "unknown_policy": "review" }],
        "education": { "min_level": { "framework": "isced", "level": 6 }, "fields": ["Computer Science"], "experience_in_place_of_education": true },
        "experience": { "min_years": 3.0, "relevant_titles": ["Software Engineer"] },
        "certifications": [],
        "domain_keywords": ["data pipelines"],
        "languages": ["English"],
        "provenance": { "source": "file", "generator_version": null, "taxonomy_versions": {}, "reviewed": true }
    })
}

fn schema_accepts(doc: &Value) -> bool {
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema is JSON");
    jsonschema::validator_for(&schema)
        .expect("schema compiles")
        .is_valid(doc)
}

fn rust(doc: &Value) -> Result<Blueprint, BlueprintError> {
    Blueprint::from_json(&doc.to_string())
}

fn edit(f: impl FnOnce(&mut Value)) -> Value {
    let mut doc = valid();
    f(&mut doc);
    doc
}

fn remove(doc: &mut Value, key: &str) {
    doc.as_object_mut().expect("object").remove(key);
}

#[test]
fn the_valid_blueprint_passes_both() {
    assert!(schema_accepts(&valid()));
    rust(&valid()).expect("valid blueprint");
}

#[test]
fn every_key_is_required_even_nullable_ones() {
    for key in ["title", "esco_occupation", "preferred_skills", "provenance"] {
        let doc = edit(|d| remove(d, key));
        assert!(
            !schema_accepts(&doc),
            "schema accepted a blueprint without {key}"
        );
        assert!(
            matches!(rust(&doc), Err(BlueprintError::Parse(_))),
            "rust accepted a blueprint without {key}"
        );
    }
    let no_ref = edit(|d| remove(&mut d["required_skills"][0], "taxonomy_ref"));
    assert!(!schema_accepts(&no_ref) && rust(&no_ref).is_err());
}

#[test]
fn an_unknown_field_fails_both() {
    for doc in [
        edit(|d| d["salary"] = json!(100)),
        edit(|d| d["required_skills"][0]["level"] = json!("expert")),
    ] {
        assert!(!schema_accepts(&doc), "schema accepted {doc}");
        assert!(matches!(rust(&doc), Err(BlueprintError::Parse(_))));
    }
}

#[test]
fn seniority_may_be_explicitly_unspecified() {
    let doc = edit(|d| d["seniority"] = json!("unspecified"));
    assert!(schema_accepts(&doc), "schema rejected {doc}");
    let b = rust(&doc).expect("parses");
    assert_eq!(b.seniority, atsgate_blueprint::Seniority::Unspecified);
    let missing = edit(|d| {
        d.as_object_mut().expect("object").remove("seniority");
    });
    assert!(
        !schema_accepts(&missing) && rust(&missing).is_err(),
        "still required"
    );
}

#[test]
fn invalid_values_fail() {
    let both = [
        edit(|d| d["occupationalCategory"] = json!("15-1252")),
        edit(|d| d["seniority"] = json!("rockstar")),
        edit(|d| d["knockouts"][0]["unknown_policy"] = json!("ignore")),
    ];
    for doc in &both {
        assert!(!schema_accepts(doc), "schema accepted {doc}");
        assert!(rust(doc).is_err());
    }
    let rust_only = [
        (
            edit(|d| d["required_skills"] = json!([])),
            "NoRequiredSkills",
        ),
        (
            edit(|d| d["required_skills"][0]["weight"] = json!(0.0)),
            "Weight",
        ),
        (
            edit(|d| d["knockouts"][0]["value"] = json!("three")),
            "MinYears",
        ),
        (
            edit(|d| d["provenance"]["source"] = json!("generated")),
            "GeneratorVersion",
        ),
        (
            edit(|d| d["esco_occupation"] = json!("https://example.com/job")),
            "EscoOccupation",
        ),
    ];
    for (doc, want) in rust_only {
        let err = rust(&doc).expect_err(want);
        assert!(format!("{err:?}").starts_with(want), "{want}: got {err:?}");
    }
}
