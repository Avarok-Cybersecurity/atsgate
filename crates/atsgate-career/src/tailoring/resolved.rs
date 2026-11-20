//! What resolving a tailoring file produces, and every way it can fail.

use crate::Career;
use crate::extension::{Id, Stat};
use crate::resume::{Award, Education, Keywords, Project, Publication, Work};

#[derive(Debug, thiserror::Error)]
pub enum TailoringError {
    #[error("tailoring file does not match the schema: {0}")]
    Parse(String),
    #[error("tailoring version {found} is not supported (expected {expected})")]
    Version { found: u32, expected: u32 },
    #[error("blueprint {0:?} is not builtin:<id>, an https:// URL or a relative path")]
    Blueprint(String),
    #[error("max_pages must be at least 1")]
    MaxPages,
    #[error("{section}: {id:?} is not an id in the career file")]
    UnknownId { section: &'static str, id: String },
    #[error("{field}: variant {id:?} does not exist in x-atsgate.variants")]
    UnknownVariant { field: &'static str, id: String },
    #[error("{field}: the career file has no basics.{field} to use")]
    MissingBasics { field: &'static str },
    #[error("{id}: {field} index {index} is out of range ({len} available)")]
    IndexOutOfRange {
        id: String,
        field: &'static str,
        index: usize,
        len: usize,
    },
    #[error("{section}: {value:?} is selected more than once")]
    Duplicate {
        section: &'static str,
        value: String,
    },
    #[error("skills: {0:?} is not a skill group in the career file")]
    UnknownSkillGroup(String),
}

/// One selected entry with its selected highlights and statistics, in tailoring order.
#[derive(Debug, Clone, PartialEq)]
pub struct TailoredEntry<'a, T> {
    pub entry: &'a T,
    pub id: &'a Id,
    pub highlights: Vec<&'a str>,
    pub stats: Vec<&'a Stat>,
}

/// One selected entry that is shown whole (education, publications, awards).
#[derive(Debug, Clone, PartialEq)]
pub struct Picked<'a, T> {
    pub entry: &'a T,
    pub id: &'a Id,
}

/// A tailoring file resolved against its career. It holds only references into the career, so
/// a renderer given one cannot show a fact the career file does not contain.
#[derive(Debug, Clone, PartialEq)]
pub struct Tailored<'a> {
    pub career: &'a Career,
    pub blueprint: &'a str,
    pub title: &'a str,
    pub summary: &'a str,
    pub max_pages: u32,
    pub work: Vec<TailoredEntry<'a, Work>>,
    pub projects: Vec<TailoredEntry<'a, Project>>,
    pub skills: Vec<&'a Keywords>,
    pub education: Vec<Picked<'a, Education>>,
    pub publications: Vec<Picked<'a, Publication>>,
    pub awards: Vec<Picked<'a, Award>>,
}
