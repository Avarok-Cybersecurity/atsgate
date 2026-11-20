//! Canonical skills and job titles with alias tables, built from the O*NET 31.0 Database
//! (USDOL/ETA, CC BY 4.0) and ESCO v1.2.1 (European Commission, Decision 2011/833/EU), with a
//! curated alias table on top. No Lightcast data.
//!
//! The crate is pure: the compiled data is embedded at build time (`data/*.jsonl.zst`, written by
//! `cargo run -p xtask -- taxonomy --write`), and every function is a computation over it.
//! Attribution for the data is in DATA-LICENSES.md and in each data file's header.

mod curated;
pub mod data;
mod fold;
mod gazetteer;
mod lookup;
mod model;

pub use curated::{CURATED_PREFIX, Curated, CuratedEntry, DeniedAlias};
pub use data::{ConceptKind, SourceName};
pub use fold::{Token, fold, tokens};
pub use gazetteer::{Gazetteer, MAX_PHRASE_TOKENS, Mention};
pub use lookup::{ConceptView, MatchKind, Resolution};
pub use model::{ConceptId, Namespace, Taxonomy, TaxonomyError};

/// The compiled O*NET 31.0 data (CC BY 4.0).
pub const ONET_DATA: &[u8] = include_bytes!("../data/onet-31.0.jsonl.zst");
/// The compiled ESCO v1.2.1 data (Commission reuse policy, CC BY 4.0).
pub const ESCO_DATA: &[u8] = include_bytes!("../data/esco-v1.2.1.jsonl.zst");
/// The curated alias table (part of this crate's code, MIT OR Apache-2.0).
pub const CURATED_YAML: &str = include_str!("../data/curated.yaml");

/// Parses a curated alias table written in the `data/curated.yaml` format.
pub fn parse_curated(text: &str) -> Result<Curated, TaxonomyError> {
    curated::parse(text).map_err(|e| TaxonomyError::Curated(e.to_string()))
}

impl Taxonomy {
    /// The built-in taxonomy, decoded once per process.
    pub fn builtin() -> Result<&'static Taxonomy, &'static TaxonomyError> {
        static BUILTIN: std::sync::OnceLock<Result<Taxonomy, TaxonomyError>> =
            std::sync::OnceLock::new();
        BUILTIN
            .get_or_init(|| {
                let curated = parse_curated(CURATED_YAML)?;
                Taxonomy::from_sources(&[ONET_DATA, ESCO_DATA], &curated)
            })
            .as_ref()
    }
}
