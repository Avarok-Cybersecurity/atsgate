//! The golden import: tbraun96's real `content/career.json` converts into a career file that
//! both the Rust types and the generated JSON Schema accept, with no information lost.

use atsgate_career::{Career, ImportError, ImportFormat, import};
use serde_json::Value;
use std::collections::BTreeMap;

const SOURCE: &str = include_str!("../../../fixtures/import/tbraun96-career.json");
const GOLDEN: &str = include_str!("../../../fixtures/import/tbraun96.career.json");
const SCHEMA: &str = include_str!("../../../schemas/career.schema.json");

/// Source values that become structure rather than text in JSON Resume, each asserted below:
/// the `present`/`unknown` date markers, the "City, Region, Country" line, and the pointer to
/// the source format's own schema.
const TRANSFORMED: [&str; 4] = [
    "present",
    "unknown",
    "Charlotte, North Carolina, United States",
    "./career.schema.json",
];

fn leaves(v: &Value, out: &mut BTreeMap<String, usize>) {
    match v {
        Value::Object(map) => map.values().for_each(|x| leaves(x, out)),
        Value::Array(items) => items.iter().for_each(|x| leaves(x, out)),
        Value::Null => {}
        Value::String(s) => *out.entry(s.clone()).or_default() += 1,
        other => *out.entry(other.to_string()).or_default() += 1,
    }
}

fn imported() -> Career {
    import(SOURCE, ImportFormat::Tbraun96).expect("the real file imports")
}

#[test]
fn the_import_matches_the_committed_golden_file() {
    assert_eq!(imported().to_json().expect("serializes"), GOLDEN);
}

#[test]
fn the_golden_file_passes_the_schema_and_the_parser() {
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema is JSON");
    let validator = jsonschema::validator_for(&schema).expect("schema compiles");
    let golden: Value = serde_json::from_str(GOLDEN).expect("golden is JSON");
    let errors: Vec<String> = validator
        .iter_errors(&golden)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
    Career::from_json(GOLDEN).expect("the golden file validates");
}

#[test]
fn every_source_value_survives_the_import() {
    let (mut source, mut output) = (BTreeMap::new(), BTreeMap::new());
    leaves(
        &serde_json::from_str(SOURCE).expect("source is JSON"),
        &mut source,
    );
    leaves(
        &serde_json::from_str(GOLDEN).expect("golden is JSON"),
        &mut output,
    );
    let lost: Vec<(&String, usize)> = source
        .iter()
        .filter(|(value, _)| !TRANSFORMED.contains(&value.as_str()))
        .filter_map(|(value, n)| {
            let kept = output.get(value).copied().unwrap_or(0);
            (kept < *n).then_some((value, n - kept))
        })
        .collect();
    assert!(lost.is_empty(), "values lost in the import: {lost:#?}");
}

#[test]
fn the_transformed_values_are_kept_as_structure() {
    let c = imported();
    let location = c.basics.location.as_ref().expect("location");
    assert_eq!(
        (location.city.as_deref(), location.region.as_deref()),
        (Some("Charlotte"), Some("North Carolina"))
    );
    assert_eq!(location.country_code.as_deref(), Some("US"));
    let work = c.work.as_ref().expect("work");
    let id = |w: &atsgate_career::Work| w.extension.as_ref().expect("ext").id.as_str().to_owned();
    let atlas = work.iter().find(|w| id(w) == "atlas").expect("atlas");
    assert_eq!(
        atlas.start_date.as_ref().map(|d| d.as_str()),
        Some("2026-03")
    );
    assert!(atlas.end_date.is_none(), "present is an open end");
    let ansys = work.iter().find(|w| id(w) == "ansys").expect("ansys");
    assert!(
        ansys.start_date.is_none() && ansys.end_date.is_none(),
        "unknown is undated"
    );
    let counts = (work.len(), c.projects.as_ref().map(Vec::len));
    assert_eq!(counts, (9, Some(10)));
    let award = &c.awards.as_ref().expect("awards")[0];
    let patent = &c.publications.as_ref().expect("publications")[0];
    assert_eq!(
        award.extension.as_ref().and_then(|x| x.subject.as_ref()),
        patent.extension.as_ref().map(|x| &x.id),
        "the recognition points at the patent"
    );
}

#[test]
fn a_field_the_importer_does_not_know_fails_rather_than_vanishing() {
    let mut doc: Value = serde_json::from_str(SOURCE).expect("source is JSON");
    doc["experience"][0]["salary"] = Value::from("redacted");
    let err = import(&doc.to_string(), ImportFormat::Tbraun96).expect_err("unknown field");
    assert!(matches!(err, ImportError::Parse { .. }), "{err}");
    let mut doc: Value = serde_json::from_str(SOURCE).expect("source is JSON");
    doc["experience"][0]["end"] = Value::from("unknown");
    let err = import(&doc.to_string(), ImportFormat::Tbraun96).expect_err("half-known dates");
    assert!(matches!(err, ImportError::Unrepresentable { .. }), "{err}");
}
