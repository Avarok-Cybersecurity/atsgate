//! The job blueprint: schema, validation, built-ins and the job-description generator.
//!
//! Phase 2b ships the schema half: the types, their validation, and the JSON Schema in
//! `schemas/blueprint.schema.json`, generated from these types. Built-ins and the generator
//! arrive in phase 3.

mod types;
mod validate;

pub use types::*;
pub use validate::BlueprintError;

impl Blueprint {
    /// Parses and validates a blueprint written as JSON.
    pub fn from_json(text: &str) -> Result<Self, BlueprintError> {
        let blueprint: Blueprint = serde_json::from_str(text).map_err(BlueprintError::Parse)?;
        blueprint.validate()?;
        Ok(blueprint)
    }
}
