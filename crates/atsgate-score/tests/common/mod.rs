//! Shared helpers: real extraction, parsing, taxonomy and scoring; nothing is faked.

#![allow(dead_code)]

use atsgate_blueprint::{Blueprint, builtin};
use atsgate_extract::{Format, Thresholds, extract};
use atsgate_score::{Input, Report, YearMonth, score};
use atsgate_taxonomy::{Gazetteer, Taxonomy};

/// Test choice: a fixed month, so "Present" roles count the same on every run.
pub const AS_OF: YearMonth = YearMonth {
    year: 2026,
    month: 11,
};

pub fn gazetteer() -> &'static Gazetteer<'static> {
    static GZ: std::sync::OnceLock<Gazetteer<'static>> = std::sync::OnceLock::new();
    GZ.get_or_init(|| Gazetteer::skills(Taxonomy::builtin().expect("taxonomy")))
}

pub fn blueprint(id: &str) -> Blueprint {
    builtin(id).unwrap_or_else(|e| panic!("builtin {id}: {e}"))
}

pub fn score_bytes(format: Format, bytes: &[u8], blueprint: &Blueprint) -> Report {
    let document = extract(bytes, format, &Thresholds::STANDARD).expect("extracts");
    score(&Input {
        document: &document,
        blueprint,
        gazetteer: gazetteer(),
        as_of: AS_OF,
    })
}

pub fn score_text(text: &str, blueprint: &Blueprint) -> Report {
    score_bytes(Format::Text, text.as_bytes(), blueprint)
}

/// Every number in a report, with no evidence text or positions: what fairness compares.
pub fn numbers(r: &Report) -> Vec<(String, Option<u64>)> {
    let bits = |x: Option<f64>| x.map(f64::to_bits);
    let d = &r.dimensions;
    let mut out = vec![
        ("R".to_owned(), bits(d.required_skills.score)),
        ("P".to_owned(), bits(d.preferred_skills.score)),
        ("T".to_owned(), bits(d.title.score)),
        ("E".to_owned(), bits(d.experience.score)),
        ("D".to_owned(), bits(d.education.score)),
        ("years".to_owned(), bits(Some(r.relevant_years))),
        ("Q".to_owned(), bits(r.quantification.score)),
    ];
    for s in &r.skills {
        out.push((format!("skill {}", s.canonical), bits(Some(s.credit))));
    }
    for k in &r.knockouts {
        out.push((format!("knockout {} {:?}", k.value, k.outcome), None));
    }
    for v in &r.vendors {
        out.push((format!("vendor {}", v.name), bits(Some(v.score))));
    }
    out
}
