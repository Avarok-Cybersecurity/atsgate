//! The career SSOT format: [JSON Resume](https://jsonresume.org/schema) plus the namespaced
//! `x-atsgate` extension. Unknown fields, missing required fields and unsourced statistics are
//! errors. The JSON Schema in `schemas/career.schema.json` is generated from these types.

mod contact;
mod date;
mod extension;
mod import;
mod item;
mod resume;
mod tailoring;
mod validate;

pub use contact::{ContactError, PrivateContact};
pub use date::{ISO_DATE_PATTERN, IsoDate};
pub use extension::{Extension, Family, FocusArea, ID_PATTERN, Id, Source, Stat, Variants};
pub use import::{ImportError, ImportFormat, import};
pub use item::{
    BasicsExtension, Display, Engagement, Image, ItemExtension, ItemKind, ProfileExtension,
    ProfileKind,
};
pub use resume::{
    Award, Basics, Certificate, Education, Keywords, Language, Location, Meta, Profile, Project,
    Publication, Reference, Volunteer, Work,
};
pub use tailoring::{
    EntrySelection, Phrasing, Picked, TAILORING_VERSION, Tailored, TailoredEntry, Tailoring,
    TailoringError,
};
pub use validate::CareerError;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The version of the `x-atsgate` extension this crate reads and writes.
pub const EXTENSION_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(
    title = "ATSGate career file",
    description = "JSON Resume v1.0.0 fields plus the x-atsgate extension."
)]
pub struct Career {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    pub basics: Basics,
    pub work: Option<Vec<Work>>,
    pub volunteer: Option<Vec<Volunteer>>,
    pub education: Option<Vec<Education>>,
    pub awards: Option<Vec<Award>>,
    pub certificates: Option<Vec<Certificate>>,
    pub publications: Option<Vec<Publication>>,
    pub skills: Option<Vec<Keywords>>,
    pub languages: Option<Vec<Language>>,
    pub interests: Option<Vec<Keywords>>,
    pub references: Option<Vec<Reference>>,
    pub projects: Option<Vec<Project>>,
    pub meta: Option<Meta>,
    #[serde(rename = "x-atsgate")]
    pub extension: Extension,
}

impl Career {
    /// Parses and validates a career file.
    pub fn from_json(text: &str) -> Result<Self, CareerError> {
        let career: Career = serde_json::from_str(text).map_err(CareerError::Parse)?;
        career.validate()?;
        Ok(career)
    }

    /// The canonical text of a career file: pretty JSON, absent fields left out rather than
    /// written as `null`, ending in a newline.
    pub fn to_json(&self) -> Result<String, CareerError> {
        fn strip_nulls(v: &mut serde_json::Value) {
            match v {
                serde_json::Value::Object(map) => {
                    map.retain(|_, x| !x.is_null());
                    map.values_mut().for_each(strip_nulls);
                }
                serde_json::Value::Array(items) => items.iter_mut().for_each(strip_nulls),
                _ => {}
            }
        }
        let mut value = serde_json::to_value(self).map_err(CareerError::Parse)?;
        strip_nulls(&mut value);
        let mut text = serde_json::to_string_pretty(&value).map_err(CareerError::Parse)?;
        text.push('\n');
        Ok(text)
    }
}
