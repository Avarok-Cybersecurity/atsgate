//! Tailoring files (`tailoring/<target>.yaml`): which SSOT entries, highlights and phrasings one
//! resume uses, and in what order. A tailoring file can only select, order and choose between
//! things the career file already holds. It has no field that could carry a new fact: unknown
//! fields are parse errors, and every id, index and variant must resolve against the career.

mod check;
mod resolved;

pub use resolved::{Picked, Tailored, TailoredEntry, TailoringError};

use crate::extension::Id;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The version of the tailoring format this crate reads.
pub const TAILORING_VERSION: u32 = 1;

/// Which existing phrasing a resume line uses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Phrasing {
    /// The career file's own `basics.label` (for the title) or `basics.summary`.
    Basics,
    /// An id from `x-atsgate.variants.titles` or `x-atsgate.variants.summaries`.
    Variant(Id),
}

/// One work or project entry, and which of its highlights and statistics appear, in order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EntrySelection {
    pub id: Id,
    /// Zero-based indexes into the entry's `highlights`.
    pub highlights: Vec<usize>,
    /// Zero-based indexes into the entry's `x-atsgate.stats`.
    pub stats: Vec<usize>,
}

/// A tailoring file. Each list is the selection in the order it appears; an entry that is not
/// listed is left out. Every key is required, so nothing is implied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(
    title = "ATSGate tailoring file",
    description = "Selects, orders and chooses phrasings from a career file; never adds facts."
)]
pub struct Tailoring {
    pub version: u32,
    /// The job blueprint the selection targets: `builtin:<id>`, an `https://` URL, or a path
    /// relative to the repository root.
    #[schemars(length(min = 1))]
    pub blueprint: String,
    pub title: Phrasing,
    pub summary: Phrasing,
    /// The longest the rendered PDF may be.
    #[schemars(range(min = 1))]
    pub max_pages: u32,
    pub work: Vec<EntrySelection>,
    pub projects: Vec<EntrySelection>,
    /// Skill group names (`skills[].name`).
    pub skills: Vec<String>,
    /// Education entry ids.
    pub education: Vec<Id>,
    /// Publication entry ids.
    pub publications: Vec<Id>,
    /// Award entry ids.
    pub awards: Vec<Id>,
}

impl Tailoring {
    /// Parses a tailoring file written in YAML. Resolve it with [`Tailoring::check`].
    pub fn from_yaml(text: &str) -> Result<Self, TailoringError> {
        serde_saphyr::from_str(text).map_err(|e| TailoringError::Parse(e.to_string()))
    }
}
