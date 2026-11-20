//! `site --write | --check`: the data the ats.express pages are rendered from (`site/data/`),
//! taken from the code and the repository's sources of truth, so the site cannot drift from them.
//!
//! - `scoring.json`: the vendor profiles, the match constants and the parse penalties, the
//!   penalties obtained by running `atsgate_score::parse::penalties` on a document that raises
//!   every signal, so the table is what the scorer charges.
//! - `blueprints/<id>.json`: each built-in blueprint as the crate parses it.
//! - `blueprint-schema.json`: the JSON Schema generated from the `Blueprint` type.
//! - `action.json`: the inputs and outputs of `action.yml`, in file order.
//! - `citations.json`: CITATIONS.md, parsed and validated.
//! - `evaluation.json`: docs/evaluation.md.

use crate::citations;
use atsgate_blueprint::{BUILTINS, builtin};
use atsgate_extract::{Document, Format, HiddenReason, Page, Signal, SignalKind};
use atsgate_score::{parse, skills, title, vendor};
use serde::Deserialize;
use serde::de::{MapAccess, Visitor};
use serde_json::{Value, json};
use std::fmt;

fn pretty(value: &Value) -> Result<Vec<u8>, String> {
    let mut text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    text.push('\n');
    Ok(text.into_bytes())
}

fn read(rel: &str) -> Result<String, String> {
    std::fs::read_to_string(crate::root().join(rel)).map_err(|e| format!("{rel}: {e}"))
}

/// Every signal that carries a base penalty, raised once on a three-page document.
fn penalized_document() -> Document {
    let kinds = [
        SignalKind::ImageOnlyPage,
        SignalKind::MultiColumn { columns: 2 },
        SignalKind::ReadingOrderDisagreement { displaced_spans: 1 },
        SignalKind::MissingToUnicode {
            font: String::new(),
        },
        SignalKind::Table,
        SignalKind::TextBox,
        SignalKind::HeaderFooterContent,
        SignalKind::UnmappedGlyphs,
        SignalKind::LigatureGlyphs,
    ];
    let page = |number| Page {
        number,
        size: None,
        image_count: 0,
        span_count: 0,
    };
    Document {
        format: Format::Text,
        pages: (1..=u32::try_from(parse::PAGE_LIMIT).unwrap_or(u32::MAX) + 1)
            .map(page)
            .collect(),
        spans: Vec::new(),
        lines: Vec::new(),
        metadata: Vec::new(),
        signals: kinds
            .into_iter()
            .map(|kind| Signal {
                kind,
                page: None,
                evidence: Vec::new(),
            })
            .collect(),
    }
}

fn integrity_rules() -> Vec<&'static str> {
    [
        HiddenReason::White,
        HiddenReason::Tiny,
        HiddenReason::ZeroWidth,
        HiddenReason::OffPage,
        HiddenReason::Vanished,
    ]
    .into_iter()
    .map(|r| SignalKind::HiddenText(r).code())
    .chain([SignalKind::MetadataOnlyText { field: "" }.code()])
    .collect()
}

pub fn scoring() -> Value {
    let doc = penalized_document();
    let penalties: Vec<Value> = parse::penalties(&doc, &atsgate_resume::Resume::default())
        .into_iter()
        .map(|p| json!({ "rule": p.rule, "points": p.points }))
        .collect();
    let vendors: Vec<Value> = vendor::PROFILES
        .iter()
        .map(|p| {
            json!({
                "name": p.name,
                "key": p.key,
                "sigma": p.sigma,
                "floor": p.floor,
                "note": p.auto_scoring_note,
                "citation": p.citation,
                "weights": p.source.remap(),
                "source": {
                    "formatting": p.source.formatting,
                    "keyword_match": p.source.keyword_match,
                    "section_completeness": p.source.section_completeness,
                    "experience_relevance": p.source.experience_relevance,
                    "education_match": p.source.education_match,
                    "quantification": p.source.quantification,
                },
            })
        })
        .collect();
    json!({
        "label": vendor::SIMULATED_LABEL,
        "no_auto_scoring": vendor::NO_AUTO_SCORING,
        "required_share": vendor::REQUIRED_SHARE,
        "title_share": vendor::TITLE_SHARE,
        "match": {
            "canonical": skills::M_CANONICAL,
            "alias": skills::M_ALIAS,
            "related": skills::M_RELATED,
            "in_scope": skills::C_IN_SCOPE,
            "list_only": skills::C_LIST_ONLY,
        },
        "title": {
            "same": title::T_SAME,
            "occupation": title::T_OCCUPATION,
            "broad_group": title::T_BROAD_GROUP,
            "ladder_span": title::LADDER_SPAN,
            "over_seniority_factor": title::OVER_SENIORITY_FACTOR,
        },
        "page_limit": parse::PAGE_LIMIT,
        "per_item": parse::PER_ITEM,
        "per_item_cap": parse::PER_ITEM_CAP,
        "penalties": penalties,
        "integrity_rules": integrity_rules(),
        "quantification_label": parse::QUANTIFICATION_LABEL,
        "vendors": vendors,
    })
}

#[derive(Debug, Deserialize)]
struct ActionField {
    description: String,
    #[serde(default)]
    required: Option<bool>,
    #[serde(default)]
    default: Option<String>,
}

/// A YAML mapping read in file order (serde maps would sort or hash it).
#[derive(Debug)]
struct Ordered(Vec<(String, ActionField)>);

impl<'de> Deserialize<'de> for Ordered {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Ordered;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a mapping of action fields")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Ordered, A::Error> {
                let mut out = Vec::new();
                while let Some(entry) = map.next_entry()? {
                    out.push(entry);
                }
                Ok(Ordered(out))
            }
        }
        d.deserialize_map(V)
    }
}

#[derive(Debug, Deserialize)]
struct Action {
    description: String,
    inputs: Ordered,
    outputs: Ordered,
}

pub fn action(yaml: &str) -> Result<Value, String> {
    let action: Action = serde_saphyr::from_str(yaml).map_err(|e| format!("action.yml: {e}"))?;
    let fields = |o: &Ordered| -> Vec<Value> {
        o.0.iter()
            .map(|(name, f)| {
                json!({
                    "name": name,
                    "description": f.description,
                    "required": f.required,
                    "default": f.default,
                })
            })
            .collect()
    };
    Ok(json!({
        "description": action.description,
        "inputs": fields(&action.inputs),
        "outputs": fields(&action.outputs),
    }))
}

pub fn all() -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut files = vec![
        ("site/data/scoring.json".to_owned(), pretty(&scoring())?),
        (
            "site/data/blueprint-schema.json".to_owned(),
            pretty(
                &serde_json::to_value(schemars::schema_for!(atsgate_blueprint::Blueprint))
                    .map_err(|e| e.to_string())?,
            )?,
        ),
        (
            "site/data/action.json".to_owned(),
            pretty(&action(&read("action.yml")?)?)?,
        ),
    ];
    let cites = citations::parse(&read("CITATIONS.md")?)?;
    let cites: Vec<Value> = cites
        .iter()
        .map(|c| {
            json!({
                "id": c.id, "title": c.title, "authors": c.authors, "date": c.date,
                "url": c.url, "accessed": c.accessed, "grade": c.grade, "supports": c.supports,
            })
        })
        .collect();
    files.push((
        "site/data/citations.json".to_owned(),
        pretty(&json!(cites))?,
    ));
    files.push((
        "site/data/evaluation.json".to_owned(),
        pretty(&json!({ "markdown": read("docs/evaluation.md")? }))?,
    ));
    for (id, _) in BUILTINS {
        let blueprint = builtin(id).map_err(|e| format!("builtin {id}: {e}"))?;
        let value = serde_json::to_value(&blueprint).map_err(|e| e.to_string())?;
        files.push((format!("site/data/blueprints/{id}.json"), pretty(&value)?));
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_penalized_signal_and_the_page_count_reach_the_table() {
        let scoring = scoring();
        let rules: Vec<&str> = scoring["penalties"]
            .as_array()
            .expect("penalties")
            .iter()
            .map(|p| p["rule"].as_str().expect("rule"))
            .collect();
        for rule in [
            "layout.image_only_page",
            "layout.multi_column",
            "glyphs.ligature",
            "parse.page_count",
        ] {
            assert!(rules.contains(&rule), "{rule} missing from {rules:?}");
        }
        assert_eq!(
            rules.len(),
            10,
            "one row per penalized signal plus the page count"
        );
    }

    #[test]
    fn vendor_rows_follow_the_profiles() {
        let scoring = scoring();
        let vendors = scoring["vendors"].as_array().expect("vendors");
        assert_eq!(vendors.len(), vendor::PROFILES.len());
        for (row, p) in vendors.iter().zip(vendor::PROFILES.iter()) {
            assert_eq!(row["sigma"], json!(p.sigma));
            assert_eq!(row["floor"], json!(p.floor));
            let w = &row["weights"];
            let sum: f64 = ["f", "r", "p", "t", "e", "d"]
                .iter()
                .map(|k| w[k].as_f64().expect("weight"))
                .sum();
            assert!((sum - 1.0).abs() < 1e-9, "{} weights sum to {sum}", p.name);
        }
    }

    #[test]
    fn action_fields_keep_file_order() {
        let yaml = "description: d\ninputs:\n  zeta: { description: z, required: true }\n  alpha: { description: a, required: false, default: \"x\" }\noutputs:\n  out: { description: o, value: v }\n";
        let got = action(yaml).expect("parses");
        assert_eq!(got["inputs"][0]["name"], "zeta");
        assert_eq!(got["inputs"][1]["default"], "x");
        assert_eq!(got["outputs"][0]["name"], "out");
    }
}
