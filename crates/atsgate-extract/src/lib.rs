//! Pure extraction: document bytes in, a [`Document`] out. No file system or network access.
//!
//! Every format yields spans in stored order, lines in reading order, and [`Signal`]s: layout
//! risks that break naive parsers, and integrity flags for text a reader cannot see.

mod docx;
mod error;
mod flow;
mod integrity;
mod layout;
mod model;
mod pdf;
mod signals;
mod thresholds;

pub use error::ExtractError;
pub use model::{BBox, Document, Format, Line, MetadataField, Page, Region, Rgb, Span, SpanId};
pub use signals::{Evidence, HiddenReason, Signal, SignalKind};
pub use thresholds::Thresholds;

/// Extracts `bytes`, read as `format`, using the detection `thresholds` the caller chose.
pub fn extract(
    bytes: &[u8],
    format: Format,
    thresholds: &Thresholds,
) -> Result<Document, ExtractError> {
    match format {
        Format::Pdf => pdf::extract(bytes, thresholds),
        Format::Docx => docx::extract(bytes, thresholds),
        Format::Html => flow::html(std::str::from_utf8(bytes)?, thresholds),
        Format::Markdown => flow::markdown(std::str::from_utf8(bytes)?, thresholds),
        Format::Text => flow::text(std::str::from_utf8(bytes)?, thresholds),
    }
}
