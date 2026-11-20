//! Extraction errors. Every failure is reported; nothing is silently skipped.

#[derive(Debug, thiserror::Error)]
pub enum ExtractError {
    #[error("PDF could not be parsed: {0}")]
    Pdf(String),
    #[error("DOCX package could not be read: {0}")]
    Docx(String),
    #[error("DOCX package has no {0} part")]
    MissingPart(&'static str),
    #[error("input is not valid UTF-8: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("XML is malformed: {0}")]
    Xml(String),
}
