//! The JSON Resume sections ATSGate accepts (schema v1.0.0 field names), with unknown fields
//! rejected. Each dated section may carry an `x-atsgate` item extension.

use crate::date::IsoDate;
use crate::item::{BasicsExtension, ItemExtension, ProfileExtension};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Location {
    pub address: Option<String>,
    #[serde(rename = "postalCode")]
    pub postal_code: Option<String>,
    pub city: Option<String>,
    #[serde(rename = "countryCode")]
    pub country_code: Option<String>,
    pub region: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub network: String,
    pub username: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "x-atsgate")]
    pub extension: Option<ProfileExtension>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Basics {
    #[schemars(length(min = 1))]
    pub name: String,
    pub label: Option<String>,
    pub image: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub url: Option<String>,
    pub summary: Option<String>,
    pub location: Option<Location>,
    pub profiles: Option<Vec<Profile>>,
    #[serde(rename = "x-atsgate")]
    pub extension: Option<BasicsExtension>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Work {
    #[schemars(length(min = 1))]
    pub name: String,
    #[schemars(length(min = 1))]
    pub position: String,
    /// Absent when the dates are not known; such a role cannot be placed on a dated resume.
    #[serde(rename = "startDate")]
    pub start_date: Option<IsoDate>,
    /// Absent with a start date means the role is current.
    #[serde(rename = "endDate")]
    pub end_date: Option<IsoDate>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub summary: Option<String>,
    pub highlights: Option<Vec<String>>,
    #[serde(rename = "x-atsgate")]
    pub extension: Option<ItemExtension>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Volunteer {
    #[schemars(length(min = 1))]
    pub organization: String,
    pub position: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "startDate")]
    pub start_date: Option<IsoDate>,
    #[serde(rename = "endDate")]
    pub end_date: Option<IsoDate>,
    pub summary: Option<String>,
    pub highlights: Option<Vec<String>>,
    #[serde(rename = "x-atsgate")]
    pub extension: Option<ItemExtension>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Education {
    #[schemars(length(min = 1))]
    pub institution: String,
    pub url: Option<String>,
    pub area: Option<String>,
    #[serde(rename = "studyType")]
    pub study_type: Option<String>,
    #[serde(rename = "startDate")]
    pub start_date: Option<IsoDate>,
    #[serde(rename = "endDate")]
    pub end_date: Option<IsoDate>,
    pub score: Option<String>,
    pub courses: Option<Vec<String>>,
    #[serde(rename = "x-atsgate")]
    pub extension: Option<ItemExtension>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Project {
    #[schemars(length(min = 1))]
    pub name: String,
    pub description: Option<String>,
    pub highlights: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
    #[serde(rename = "startDate")]
    pub start_date: Option<IsoDate>,
    #[serde(rename = "endDate")]
    pub end_date: Option<IsoDate>,
    pub url: Option<String>,
    pub roles: Option<Vec<String>>,
    pub entity: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    #[serde(rename = "x-atsgate")]
    pub extension: Option<ItemExtension>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Award {
    pub title: String,
    pub date: Option<IsoDate>,
    pub awarder: Option<String>,
    pub summary: Option<String>,
    #[serde(rename = "x-atsgate")]
    pub extension: Option<ItemExtension>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Certificate {
    pub name: String,
    pub date: Option<IsoDate>,
    pub url: Option<String>,
    pub issuer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Publication {
    pub name: String,
    pub publisher: Option<String>,
    #[serde(rename = "releaseDate")]
    pub release_date: Option<IsoDate>,
    pub url: Option<String>,
    pub summary: Option<String>,
    #[serde(rename = "x-atsgate")]
    pub extension: Option<ItemExtension>,
}

/// A named group of keywords (`skills`, `interests`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Keywords {
    pub name: String,
    pub level: Option<String>,
    pub keywords: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Language {
    pub language: String,
    pub fluency: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    pub name: String,
    pub reference: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Meta {
    pub canonical: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "lastModified")]
    pub last_modified: Option<String>,
}
