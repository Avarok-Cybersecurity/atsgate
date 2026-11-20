//! ESCO v1.2.1 API search pages to taxonomy records (pure).
//!
//! The bulk download is delivered only by e-mail link, so the import pages through the official
//! ESCO web API pinned to `selectedVersion=v1.2.1`. The API's bytes vary between identical
//! requests (link order), so the pin is a sha256 over the canonical projection of the English
//! records this import keeps, which is stable.

use super::derive::strip_parenthetical;
use atsgate_taxonomy::ConceptKind;
use atsgate_taxonomy::data::{ESCO_URI_PREFIX, Record};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const VERSION: &str = "v1.2.1";
pub const PAGE_SIZE: usize = 100;

pub fn page_url(kind: &str, page: usize) -> String {
    format!(
        "https://ec.europa.eu/esco/api/search?language=en&type={kind}&limit={PAGE_SIZE}\
         &offset={page}&selectedVersion={VERSION}&full=true"
    )
}

/// One English record as kept from the API: the projection the pin covers.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct Projected {
    pub uri: String,
    pub label: String,
    pub aliases: BTreeSet<String>,
    pub broader: BTreeSet<String>,
}

pub fn total(page: &[u8]) -> Result<usize, String> {
    let v: Value = serde_json::from_slice(page).map_err(|e| format!("ESCO page: {e}"))?;
    v["total"]
        .as_u64()
        .map(|t| t as usize)
        .ok_or("ESCO page has no total".into())
}

fn strings(v: &Value) -> impl Iterator<Item = String> + '_ {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(|s| s.as_str().map(str::to_owned))
}

/// Projects the released English records of one page; `broader_link` names the `_links` key.
pub fn project(page: &[u8], broader_link: &str) -> Result<Vec<Projected>, String> {
    let v: Value = serde_json::from_slice(page).map_err(|e| format!("ESCO page: {e}"))?;
    let results = v["_embedded"]["results"]
        .as_array()
        .ok_or("ESCO page has no results")?;
    let mut out = Vec::new();
    for r in results {
        if r["status"].as_str() != Some("released") {
            continue;
        }
        let uri = r["uri"].as_str().ok_or("ESCO record without uri")?;
        let Some(label) = r["preferredLabel"]["en"].as_str() else {
            continue;
        };
        let broader = r["_links"][broader_link]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|l| l["uri"].as_str().map(str::to_owned))
            .collect();
        out.push(Projected {
            uri: uri.to_owned(),
            label: label.to_owned(),
            aliases: strings(&r["alternativeLabel"]["en"]).collect(),
            broader,
        });
    }
    Ok(out)
}

pub fn digest(projected: &[Projected]) -> Result<String, String> {
    let mut sorted = projected.to_vec();
    sorted.sort();
    let bytes = serde_json::to_vec(&sorted).map_err(|e| e.to_string())?;
    Ok(crate::fetch::sha256_hex(&bytes))
}

fn compact(uri: &str) -> Result<String, String> {
    uri.strip_prefix(ESCO_URI_PREFIX)
        .map(str::to_owned)
        .ok_or(format!("{uri} is not an ESCO URI"))
}

/// Skills first, then occupations, each sorted by URI; links stay within the kind.
pub fn records(skills: &[Projected], occupations: &[Projected]) -> Result<Vec<Record>, String> {
    let mut out = Vec::new();
    for (items, kind) in [
        (skills, ConceptKind::Skill),
        (occupations, ConceptKind::Occupation),
    ] {
        let mut sorted = items.to_vec();
        sorted.sort();
        sorted.dedup_by(|a, b| a.uri == b.uri);
        let base = out.len() as u32;
        let index: BTreeMap<&str, u32> = sorted
            .iter()
            .enumerate()
            .map(|(i, p)| (p.uri.as_str(), base + i as u32))
            .collect();
        for p in &sorted {
            let label = atsgate_taxonomy::fold(&p.label);
            let aliases: BTreeSet<String> = p
                .aliases
                .iter()
                .cloned()
                .chain(strip_parenthetical(&p.label))
                .filter(|a| atsgate_taxonomy::fold(a) != label)
                .collect();
            out.push(Record {
                kind,
                reference: compact(&p.uri)?,
                label: p.label.clone(),
                aliases: aliases.into_iter().collect(),
                broader: p
                    .broader
                    .iter()
                    .filter_map(|b| index.get(b.as_str()).copied())
                    .collect(),
                skills: Vec::new(),
            });
        }
    }
    Ok(out)
}
