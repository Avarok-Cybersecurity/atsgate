//! The curated alias tables (`data/curated.yaml`): hand-written canonical names and aliases laid
//! over the imported data, plus the gazetteer's ambiguity rules.

use serde::Deserialize;

/// One curated concept. When `reference` names an imported concept, the entry renames it to
/// `canonical` and adds `aliases`; when it is an `atsgate:` reference, the entry is a new concept
/// that the imported taxonomies lack, linked into them through `broader`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CuratedEntry {
    pub canonical: String,
    #[serde(rename = "ref")]
    pub reference: String,
    pub aliases: Vec<String>,
    pub broader: Vec<String>,
}

/// An imported alias that is wrong for its concept and is dropped (e.g. "Mocha" for JavaScript).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeniedAlias {
    #[serde(rename = "ref")]
    pub reference: String,
    pub alias: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Curated {
    pub version: String,
    pub skills: Vec<CuratedEntry>,
    pub titles: Vec<CuratedEntry>,
    pub denied_aliases: Vec<DeniedAlias>,
    /// Phrases that are also ordinary words ("Go", "Rust", "Excel"). In free text they match only
    /// with this exact capitalization and never as the first word of a sentence.
    pub case_sensitive: Vec<String>,
    /// Folded phrases the gazetteer never reports (too generic to be a requirement on their own).
    pub gazetteer_stoplist: Vec<String>,
}

/// The reference prefix of concepts that exist only in the curated table.
pub const CURATED_PREFIX: &str = "atsgate:";

pub fn parse(text: &str) -> Result<Curated, serde_saphyr::Error> {
    serde_saphyr::from_str(text)
}
