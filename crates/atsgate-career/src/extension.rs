//! The `x-atsgate` extension: sources, statistics, product families and phrasing variants.

use crate::date::IsoDate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const ID_PATTERN: &str = r"^[a-z0-9][a-z0-9-]*$";

/// A stable identifier: lowercase letters, digits and hyphens.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(try_from = "String", into = "String")]
#[schemars(extend("pattern" = ID_PATTERN))]
pub struct Id(String);

impl TryFrom<String> for Id {
    type Error = String;

    fn try_from(v: String) -> Result<Self, Self::Error> {
        let mut chars = v.chars();
        let first_ok = chars
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
        let rest_ok = chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if first_ok && rest_ok {
            Ok(Self(v))
        } else {
            Err(format!("{v:?} is not an id (a-z, 0-9, hyphen)"))
        }
    }
}

impl From<Id> for String {
    fn from(id: Id) -> String {
        id.0
    }
}

impl Id {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Where a fact came from. At most one of `url` or `path` locates it; a source with neither is
/// an unlocated statement (for example a figure the owner stated) and reads as unverified.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Source {
    #[schemars(length(min = 1))]
    pub title: String,
    pub url: Option<String>,
    pub path: Option<String>,
    pub retrieved: IsoDate,
    pub note: Option<String>,
}

/// A statistic and the evidence for it. A statistic without a source is invalid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Stat {
    #[schemars(length(min = 1))]
    pub label: String,
    #[schemars(length(min = 1))]
    pub value: String,
    /// What the figure was read from: a source id or a description of the measurement.
    #[schemars(length(min = 1))]
    pub source: String,
    pub retrieved: IsoDate,
    pub note: Option<String>,
}

/// Related products shown together, by project id.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Family {
    pub id: Id,
    #[schemars(length(min = 1))]
    pub name: String,
    pub members: Vec<Id>,
}

/// Alternative phrasings a tailoring file may choose between. It may never add new ones.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Variants {
    pub summaries: BTreeMap<Id, String>,
    pub titles: BTreeMap<Id, String>,
}

/// The top-level extension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Extension {
    /// The extension format version this file follows.
    pub version: u32,
    pub sources: BTreeMap<Id, Source>,
    pub families: Vec<Family>,
    pub variants: Variants,
    /// Areas of expertise, each a name and a sentence; for a site, not for the resume.
    pub focus_areas: Option<Vec<FocusArea>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FocusArea {
    #[schemars(length(min = 1))]
    pub name: String,
    #[schemars(length(min = 1))]
    pub summary: String,
}
