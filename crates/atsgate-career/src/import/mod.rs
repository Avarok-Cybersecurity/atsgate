//! Pure converters from other career formats into a [`Career`]. Text in, a validated career
//! out; nothing is guessed, and anything the target cannot represent is an error.

mod ids;
mod json_resume;
mod site;
mod site_entries;
mod site_format;
mod site_patent;

use crate::{Career, CareerError};

/// The format of the file being imported. Callers name it; nothing is sniffed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportFormat {
    /// A plain JSON Resume v1.0.0 document.
    JsonResume,
    /// The thomaspbraun.com `content/career.json` format (tbraun96 repository).
    Tbraun96,
}

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("the input is not valid {format:?}: {source}")]
    Parse {
        format: ImportFormat,
        source: serde_json::Error,
    },
    #[error("the input already has an x-atsgate block; it is a career file, not JSON Resume")]
    AlreadyCareer,
    #[error("{item}: {reason}")]
    Unrepresentable { item: String, reason: String },
    #[error(transparent)]
    Career(#[from] CareerError),
}

impl ImportError {
    fn unrepresentable(item: &str, reason: impl Into<String>) -> Self {
        Self::Unrepresentable {
            item: item.to_owned(),
            reason: reason.into(),
        }
    }
}

/// Converts `text`, written in `format`, into a validated career.
pub fn import(text: &str, format: ImportFormat) -> Result<Career, ImportError> {
    let parse = |source| ImportError::Parse { format, source };
    let career = match format {
        ImportFormat::JsonResume => {
            json_resume::convert(serde_json::from_str(text).map_err(parse)?)?
        }
        ImportFormat::Tbraun96 => site::convert(serde_json::from_str(text).map_err(parse)?)?,
    };
    career.validate()?;
    Ok(career)
}
