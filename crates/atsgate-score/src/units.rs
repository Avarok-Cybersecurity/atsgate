//! The resume text the scorer reads, split into units that remember their context and lines.
//!
//! Never read: the preamble (name, photo caption, address), header and footer lines, and the
//! education section's institutions and dates. Education titles are read only by [`crate::education`].

use atsgate_resume::{LineRef, Resume, SectionKind};
use schemars::JsonSchema;
use serde::Serialize;

/// Where a piece of text sits, which decides the context factor `c` of a skill found in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Context {
    /// A role's title, bullets or notes.
    Experience,
    /// A project's title, bullets or notes.
    Project,
    /// One item of the skills section.
    SkillsList,
    /// Summary, certifications, languages, awards, publications.
    Other,
}

impl Context {
    pub fn is_experience_or_project(self) -> bool {
        matches!(self, Self::Experience | Self::Project)
    }
}

/// An experience or project entry: (section index, entry index) in [`Resume::sections`].
pub type EntryId = (usize, usize);

#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    pub text: String,
    pub context: Context,
    /// The lines the text may come from; evidence points at the one that contains the match.
    pub lines: Vec<LineRef>,
    pub entry: Option<EntryId>,
}

fn context_of(kind: SectionKind) -> Option<Context> {
    match kind {
        SectionKind::Experience | SectionKind::Volunteer => Some(Context::Experience),
        SectionKind::Projects => Some(Context::Project),
        SectionKind::Skills => Some(Context::SkillsList),
        SectionKind::Summary
        | SectionKind::Certifications
        | SectionKind::Languages
        | SectionKind::Awards
        | SectionKind::Publications => Some(Context::Other),
        SectionKind::Education => None,
    }
}

/// Every unit of `resume`, in document order.
pub fn units(resume: &Resume) -> Vec<Unit> {
    let mut out = Vec::new();
    for (si, section) in resume.sections.iter().enumerate() {
        let Some(context) = context_of(section.kind) else {
            continue;
        };
        let unit = |text: &str, lines: &[LineRef], entry| Unit {
            text: text.to_owned(),
            context,
            lines: lines.to_vec(),
            entry,
        };
        for (ei, e) in section.entries.iter().enumerate() {
            let id = Some((si, ei));
            out.push(unit(&e.title, &e.evidence, id));
            out.extend(e.bullets.iter().map(|b| unit(b, &e.evidence, id)));
        }
        if !section.text.is_empty() {
            out.push(unit(&section.text, &section.evidence, None));
        }
        out.extend(
            section
                .items
                .iter()
                .map(|item| unit(item, &section.evidence, None)),
        );
    }
    out
}

/// Bullets of experience and project entries: what the quantification heuristic reads.
pub fn bullets(resume: &Resume) -> Vec<(&str, &[LineRef])> {
    resume
        .sections
        .iter()
        .filter(|s| matches!(s.kind, SectionKind::Experience | SectionKind::Projects))
        .flat_map(|s| &s.entries)
        .flat_map(|e| {
            e.bullets
                .iter()
                .map(|b| (b.as_str(), e.evidence.as_slice()))
        })
        .collect()
}
