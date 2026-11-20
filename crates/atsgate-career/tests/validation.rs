//! A career file is rejected for a missing required field, an unknown field or an unsourced
//! statistic, by both the Rust types and the generated JSON Schema.

use atsgate_career::{Career, CareerError};
use serde_json::{Value, json};

const SCHEMA: &str = include_str!("../../../schemas/career.schema.json");

fn valid() -> Value {
    json!({
        "basics": { "name": "Jordan Example", "email": "jordan@example.com" },
        "work": [{
            "name": "Northwind Analytics",
            "position": "Senior Software Engineer",
            "startDate": "2020-01",
            "highlights": ["Moved batch pipelines to Rust, cutting cost 30%."],
            "x-atsgate": {
                "id": "northwind",
                "sources": ["payroll-letter"],
                "stats": [{ "label": "cost reduction", "value": "30%", "source": "payroll-letter", "retrieved": "2026-11-20" }]
            }
        }],
        "projects": [{ "name": "Pipeline", "x-atsgate": { "id": "pipeline", "sources": [] } }],
        "x-atsgate": {
            "version": 1,
            "sources": { "payroll-letter": { "title": "Employer letter", "path": "evidence/letter.pdf", "retrieved": "2026-11-20" } },
            "families": [{ "id": "data", "name": "Data products", "members": ["pipeline"] }],
            "variants": { "summaries": { "backend": "Backend engineer." }, "titles": {} }
        }
    })
}

fn schema_accepts(doc: &Value) -> bool {
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema is JSON");
    jsonschema::validator_for(&schema)
        .expect("schema compiles")
        .is_valid(doc)
}

fn rust_accepts(doc: &Value) -> Result<Career, CareerError> {
    Career::from_json(&doc.to_string())
}

fn edit(f: impl FnOnce(&mut Value)) -> Value {
    let mut doc = valid();
    f(&mut doc);
    doc
}

#[test]
fn the_valid_file_passes_both() {
    assert!(schema_accepts(&valid()));
    rust_accepts(&valid()).expect("valid career");
}

#[test]
fn a_missing_required_field_fails_both() {
    for doc in [
        edit(|d| drop(d["basics"].as_object_mut().map(|o| o.remove("name")))),
        edit(|d| drop(d["work"][0].as_object_mut().map(|o| o.remove("position")))),
        edit(|d| drop(d.as_object_mut().map(|o| o.remove("x-atsgate")))),
    ] {
        assert!(!schema_accepts(&doc), "schema accepted {doc}");
        assert!(matches!(rust_accepts(&doc), Err(CareerError::Parse(_))));
    }
}

#[test]
fn an_unknown_field_fails_both() {
    for doc in [
        edit(|d| d["basics"]["nickname"] = json!("JX")),
        edit(|d| d["x-atsgate"]["extra"] = json!(true)),
        edit(|d| d["work"][0]["x-atsgate"]["score"] = json!(9)),
    ] {
        assert!(!schema_accepts(&doc), "schema accepted {doc}");
        assert!(matches!(rust_accepts(&doc), Err(CareerError::Parse(_))));
    }
}

#[test]
fn an_unsourced_statistic_fails() {
    let missing = edit(|d| {
        drop(
            d["work"][0]["x-atsgate"]["stats"][0]
                .as_object_mut()
                .map(|o| o.remove("source")),
        )
    });
    let empty = edit(|d| d["work"][0]["x-atsgate"]["stats"][0]["source"] = json!(""));
    for doc in [&missing, &empty] {
        assert!(!schema_accepts(doc), "schema accepted {doc}");
        assert!(rust_accepts(doc).is_err());
    }
    let blank = edit(|d| d["work"][0]["x-atsgate"]["stats"][0]["source"] = json!("   "));
    assert!(
        matches!(rust_accepts(&blank), Err(CareerError::UnsourcedStat { .. })),
        "blank is caught by validation"
    );
}

#[test]
fn references_must_resolve() {
    let unknown_source = edit(|d| d["work"][0]["x-atsgate"]["sources"] = json!(["nowhere"]));
    assert!(matches!(
        rust_accepts(&unknown_source),
        Err(CareerError::UnknownSource { .. })
    ));
    let unknown_member = edit(|d| d["x-atsgate"]["families"][0]["members"] = json!(["ghost"]));
    assert!(matches!(
        rust_accepts(&unknown_member),
        Err(CareerError::UnknownFamilyMember { .. })
    ));
    let duplicate = edit(|d| d["projects"][0]["x-atsgate"]["id"] = json!("northwind"));
    assert!(matches!(
        rust_accepts(&duplicate),
        Err(CareerError::DuplicateId(_))
    ));
    let two_locators =
        edit(|d| d["x-atsgate"]["sources"]["payroll-letter"]["url"] = json!("https://example.com"));
    assert!(matches!(
        rust_accepts(&two_locators),
        Err(CareerError::SourceLocator(_))
    ));
}

#[test]
fn malformed_dates_and_ids_fail_both() {
    for doc in [
        edit(|d| d["work"][0]["startDate"] = json!("Jan 2020")),
        edit(|d| d["work"][0]["x-atsgate"]["id"] = json!("North Wind")),
    ] {
        assert!(!schema_accepts(&doc), "schema accepted {doc}");
        assert!(rust_accepts(&doc).is_err());
    }
}

#[test]
fn kind_specific_extension_fields_stay_on_their_kind() {
    let on_work = edit(|d| d["work"][0]["x-atsgate"]["engagement"] = json!("founder"));
    rust_accepts(&on_work).expect("engagement belongs on work");
    let on_project = edit(|d| d["projects"][0]["x-atsgate"]["engagement"] = json!("founder"));
    assert!(matches!(
        rust_accepts(&on_project),
        Err(CareerError::MisplacedField {
            field: "engagement",
            ..
        })
    ));
    let repo_on_work = edit(|d| d["work"][0]["x-atsgate"]["repo"] = json!("https://x.test"));
    assert!(matches!(
        rust_accepts(&repo_on_work),
        Err(CareerError::MisplacedField { field: "repo", .. })
    ));
}

#[test]
fn an_award_subject_must_be_an_item_id() {
    let award = |subject: &str| {
        edit(|d| {
            d["awards"] = json!([{ "title": "Prize", "x-atsgate": {
                "id": "prize", "sources": [], "subject": subject } }])
        })
    };
    rust_accepts(&award("pipeline")).expect("subject resolves");
    assert!(matches!(
        rust_accepts(&award("ghost")),
        Err(CareerError::UnknownSubject { .. })
    ));
}

#[test]
fn dates_are_ordered_and_an_end_needs_a_start() {
    let undated = edit(|d| drop(d["work"][0].as_object_mut().map(|o| o.remove("startDate"))));
    rust_accepts(&undated).expect("a role with unknown dates is allowed");
    let end_only = edit(|d| {
        drop(d["work"][0].as_object_mut().map(|o| o.remove("startDate")));
        d["work"][0]["endDate"] = json!("2021-01");
    });
    assert!(matches!(
        rust_accepts(&end_only),
        Err(CareerError::EndWithoutStart { .. })
    ));
    let reversed = edit(|d| d["work"][0]["endDate"] = json!("2019-12"));
    assert!(matches!(
        rust_accepts(&reversed),
        Err(CareerError::EndBeforeStart { .. })
    ));
    let same_year = edit(|d| d["work"][0]["endDate"] = json!("2020"));
    rust_accepts(&same_year).expect("2020 does not end before 2020-01");
}

#[test]
fn a_source_with_no_locator_is_an_unlocated_statement() {
    let unlocated = edit(|d| {
        let source = &mut d["x-atsgate"]["sources"]["payroll-letter"];
        drop(source.as_object_mut().map(|o| o.remove("path")))
    });
    assert!(schema_accepts(&unlocated));
    rust_accepts(&unlocated).expect("an owner statement has no url or path");
}
