//! The job blueprint: schema, validation, built-ins and the job-description generator.
//!
//! - The types, their validation, and the JSON Schema in `schemas/blueprint.schema.json`,
//!   generated from these types.
//! - Ten built-in blueprints (`blueprints/*.yaml`), embedded at build time.
//! - [`generate`]: a deterministic job description -> blueprint generator over the
//!   `atsgate-taxonomy` gazetteer.

mod builtin;
pub mod generate;
mod types;
mod validate;

pub use builtin::{BUILTINS, builtin};
pub use types::*;
pub use validate::BlueprintError;

impl Blueprint {
    /// Parses and validates a blueprint written as JSON.
    pub fn from_json(text: &str) -> Result<Self, BlueprintError> {
        let blueprint: Blueprint = serde_json::from_str(text).map_err(BlueprintError::Parse)?;
        blueprint.validate()?;
        Ok(blueprint)
    }

    /// Parses and validates a blueprint written as YAML.
    pub fn from_yaml(text: &str) -> Result<Self, BlueprintError> {
        let blueprint: Blueprint =
            serde_saphyr::from_str(text).map_err(|e| BlueprintError::Yaml(e.to_string()))?;
        blueprint.validate()?;
        Ok(blueprint)
    }

    /// Renders the blueprint as YAML (the format of `blueprints/*.yaml`).
    pub fn to_yaml(&self) -> Result<String, BlueprintError> {
        serde_saphyr::to_string(self).map_err(|e| BlueprintError::Yaml(e.to_string()))
    }
}
