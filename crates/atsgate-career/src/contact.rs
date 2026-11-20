//! Private contact details. They never live in the career SSOT, which is meant to be public: a
//! resume that should carry a phone number or a street address reads them from a separate
//! contact file, kept out of the repository.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(
    title = "ATSGate private contact file",
    description = "Phone and street address for a resume built locally; never commit it."
)]
pub struct PrivateContact {
    #[schemars(length(min = 1))]
    pub phone: Option<String>,
    /// The street line, as it should be printed.
    #[schemars(length(min = 1))]
    pub street: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ContactError {
    #[error("contact file does not match the schema: {0}")]
    Parse(serde_json::Error),
    #[error("contact file sets neither phone nor street; use the public profile instead")]
    Empty,
    #[error("contact file {0} is blank")]
    Blank(&'static str),
}

impl PrivateContact {
    pub fn from_json(text: &str) -> Result<Self, ContactError> {
        let contact: Self = serde_json::from_str(text).map_err(ContactError::Parse)?;
        for (name, value) in [("phone", &contact.phone), ("street", &contact.street)] {
            if value.as_deref().is_some_and(|v| v.trim().is_empty()) {
                return Err(ContactError::Blank(name));
            }
        }
        if contact.phone.is_none() && contact.street.is_none() {
            return Err(ContactError::Empty);
        }
        Ok(contact)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_contact_file_must_set_something_real() {
        let ok = PrivateContact::from_json(r#"{"phone": "+1 555 0100"}"#).expect("phone only");
        assert_eq!(ok.street, None);
        for bad in [r#"{}"#, r#"{"phone": " "}"#, r#"{"fax": "1"}"#, "["] {
            assert!(PrivateContact::from_json(bad).is_err(), "{bad}");
        }
    }
}
