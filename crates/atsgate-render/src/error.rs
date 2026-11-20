//! Everything that can stop a render.

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("PDF typesetting failed: {0}")]
    Pdf(String),
    #[error("the resume is {pages} pages long; the tailoring file allows {max_pages}")]
    TooManyPages { pages: usize, max_pages: u32 },
    #[error("DOCX packaging failed: {0}")]
    Docx(String),
    #[error(
        "refusing to render private contact details (phone, street) in CI; pass \
         --allow-private-contact if this artifact is meant to carry them"
    )]
    PrivateContactInCi,
    #[error(
        "{0} is set in the career file; private contact details belong in a contact file \
         (--contact <file>), not in the career SSOT"
    )]
    PrivateFieldInCareer(&'static str),
    #[error("{0} is not an http(s) URL; refusing to link it")]
    UnsafeUrl(String),
    #[error("marker <!-- atsgate:{0} --> is missing, repeated or unclosed")]
    Marker(String),
    #[error("the HTML has a marker for {0:?}, which is not a partial")]
    UnknownMarker(String),
    #[error("JSON serialization failed: {0}")]
    Json(String),
    #[error(transparent)]
    Tailoring(#[from] atsgate_career::TailoringError),
    #[error("could not read the rendered file back: {0}")]
    Extract(String),
    #[error("the rendered {format} does not read back as rendered:\n  {}", problems.join("\n  "))]
    RoundTrip {
        format: String,
        problems: Vec<String>,
    },
}
