//! The built-in blueprints parse, pass the JSON Schema, and reference the taxonomy exactly.

use atsgate_blueprint::{BUILTINS, Blueprint, BlueprintError, ProvenanceSource, builtin};
use atsgate_taxonomy::{ConceptKind, Taxonomy};
use serde_json::Value;

const SCHEMA: &str = include_str!("../../../schemas/blueprint.schema.json");

fn all() -> Vec<Blueprint> {
    BUILTINS
        .iter()
        .map(|(id, _)| builtin(id).unwrap_or_else(|e| panic!("builtin {id}: {e}")))
        .collect()
}

#[test]
fn ten_builtins_parse_and_carry_their_own_id() {
    assert_eq!(BUILTINS.len(), 10);
    for ((id, _), blueprint) in BUILTINS.iter().zip(all()) {
        assert_eq!(blueprint.id, *id);
    }
}

#[test]
fn builtins_are_reviewed_builtin_provenance_with_taxonomy_versions() {
    let versions = Taxonomy::builtin().expect("taxonomy").versions();
    for b in all() {
        let p = &b.provenance;
        assert_eq!(p.source, ProvenanceSource::Builtin, "{}", b.id);
        assert!(p.reviewed && p.generator_version.is_none(), "{}", b.id);
        assert_eq!(&p.taxonomy_versions, versions, "{}", b.id);
    }
}

#[test]
fn every_builtin_skill_resolves_to_its_taxonomy_concept() {
    let taxonomy = Taxonomy::builtin().expect("taxonomy");
    let mut problems = Vec::new();
    for b in all() {
        if b.required_skills.is_empty() {
            problems.push(format!("{}: no required skills", b.id));
        }
        for skill in b.required_skills.iter().chain(&b.preferred_skills) {
            let name = &skill.canonical;
            let Some(reference) = skill.taxonomy_ref.as_deref() else {
                problems.push(format!("{}: {name} has no taxonomy_ref", b.id));
                continue;
            };
            match taxonomy.by_ref(reference) {
                None => problems.push(format!("{}: {name}: unknown ref {reference}", b.id)),
                Some(c) if c.canonical() != name || c.kind() == ConceptKind::Occupation => problems
                    .push(format!(
                        "{}: {name}: {reference} is {:?} {}",
                        b.id,
                        c.kind(),
                        c.canonical()
                    )),
                Some(_) => {}
            }
            match taxonomy.resolve_skill(name) {
                Some(r) if r.taxonomy_ref == reference => {}
                other => problems.push(format!("{}: {name} resolves to {other:?}", b.id)),
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn every_builtin_occupation_is_in_the_taxonomy() {
    let taxonomy = Taxonomy::builtin().expect("taxonomy");
    for b in all() {
        let soc = format!("onet-soc:{}", b.occupational_category);
        let onet = taxonomy
            .by_ref(&soc)
            .unwrap_or_else(|| panic!("{}: {soc}", b.id));
        assert_eq!(onet.kind(), ConceptKind::Occupation);
        let esco = b
            .esco_occupation
            .as_deref()
            .expect("every builtin has an ESCO code");
        let esco = taxonomy
            .by_ref(esco)
            .unwrap_or_else(|| panic!("{}: {esco}", b.id));
        assert_eq!(esco.kind(), ConceptKind::Occupation);
    }
}

#[test]
fn builtins_pass_the_json_schema() {
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema is JSON");
    let validator = jsonschema::validator_for(&schema).expect("schema compiles");
    for b in all() {
        let doc = serde_json::to_value(&b).expect("serializes");
        assert!(validator.is_valid(&doc), "{} fails the schema", b.id);
    }
}

#[test]
fn an_unknown_builtin_is_an_error() {
    assert!(matches!(
        builtin("astronaut"),
        Err(BlueprintError::UnknownBuiltin(id)) if id == "astronaut"
    ));
}
