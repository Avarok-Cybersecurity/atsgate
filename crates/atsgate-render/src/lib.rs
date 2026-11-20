//! Renders a career SSOT into an ATS-safe resume and static-site partials. Pure: bytes in,
//! bytes out. The CLI reads the inputs, decides whether it runs in CI and writes the files.

mod blocks;
mod build;
mod contact;
mod dates;
mod docx;
mod error;
mod model;
mod pdf;
mod site;
mod verify;

pub use blocks::{BULLET, Block, PART_SEPARATOR, blocks};
pub use build::{Artifacts, DOCX_FILE, JSONLD_FILE, MANIFEST_FILE, PDF_FILE, build, sha256_hex};
pub use contact::{ContactProfile, guard_private_contact};
pub use dates::Dates;
pub use docx::render as render_docx;
pub use error::RenderError;
pub use model::{Body, Entry, ResumeDocument, Section, SkillGroup, document, heading};
pub use pdf::render as render_pdf;
pub use site::{PARTIALS, inject, jsonld, partials};
pub use verify::{differences, round_trip};
