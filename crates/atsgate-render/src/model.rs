//! The document model: a resolved tailoring and a contact profile in, the resume's sections and
//! entries out. Every renderer (PDF, DOCX, site partials, JSON-LD) and the round-trip check read
//! this one model, so they cannot disagree about what the resume says.

use crate::blocks::PART_SEPARATOR;
use crate::contact::{ContactProfile, contact_items, host};
use crate::dates::Dates;
use crate::error::RenderError;
use atsgate_career::{Stat, Tailored};
use atsgate_resume::SectionKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The career id of the entry this was drawn from.
    pub id: String,
    pub title: String,
    /// Everything after the title on the entry's first line, before the dates.
    pub organization: Option<String>,
    /// The employer or institution alone, for structured data (JSON-LD).
    pub org_name: Option<String>,
    pub dates: Option<Dates>,
    pub url: Option<String>,
    pub bullets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillGroup {
    pub label: String,
    pub items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Body {
    Text(String),
    Entries(Vec<Entry>),
    Groups(Vec<SkillGroup>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub kind: SectionKind,
    pub body: Body,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeDocument {
    pub name: String,
    pub title: String,
    pub contact: Vec<String>,
    pub sections: Vec<Section>,
    pub max_pages: u32,
}

/// The heading each section is printed under: the plain names every parser knows.
pub fn heading(kind: SectionKind) -> &'static str {
    match kind {
        SectionKind::Summary => "Summary",
        SectionKind::Experience => "Experience",
        SectionKind::Projects => "Projects",
        SectionKind::Skills => "Skills",
        SectionKind::Education => "Education",
        SectionKind::Publications => "Publications",
        SectionKind::Awards => "Awards",
        SectionKind::Certifications => "Certifications",
        SectionKind::Languages => "Languages",
        SectionKind::Volunteer => "Volunteer",
    }
}

/// Selected statistics as one bullet: "737 GitHub stars; 107 forks."
fn stats_bullet(stats: &[&Stat]) -> Option<String> {
    let parts: Vec<String> = stats
        .iter()
        .map(|s| format!("{} {}", s.value, s.label))
        .collect();
    (!parts.is_empty()).then(|| format!("{}.", parts.join("; ")))
}

fn joined(parts: &[Option<&str>], separator: &str) -> Option<String> {
    let present: Vec<&str> = parts
        .iter()
        .flatten()
        .copied()
        .filter(|p| !p.is_empty())
        .collect();
    (!present.is_empty()).then(|| present.join(separator))
}

fn bullets(highlights: &[&str], stats: &[&Stat]) -> Vec<String> {
    let mut out: Vec<String> = highlights.iter().map(|h| (*h).to_owned()).collect();
    out.extend(stats_bullet(stats));
    out
}

fn experience(t: &Tailored) -> Vec<Entry> {
    t.work
        .iter()
        .map(|s| {
            let w = s.entry;
            let unit = w.extension.as_ref().and_then(|x| x.unit.as_deref());
            let org = [Some(w.name.as_str()), unit, w.location.as_deref()];
            Entry {
                id: s.id.to_string(),
                title: w.position.clone(),
                organization: joined(&org, ", "),
                org_name: Some(w.name.clone()),
                dates: Dates::of(w.start_date.as_ref(), w.end_date.as_ref(), true),
                url: w.url.clone(),
                bullets: bullets(&s.highlights, &s.stats),
            }
        })
        .collect()
}

fn projects(t: &Tailored) -> Vec<Entry> {
    t.projects
        .iter()
        .map(|s| {
            let p = s.entry;
            let repo = p.extension.as_ref().and_then(|x| x.repo.as_deref());
            let link = p.url.as_deref().or(repo);
            let roles = p.roles.as_ref().map(|r| r.join(", "));
            Entry {
                id: s.id.to_string(),
                title: p.name.clone(),
                organization: joined(&[roles.as_deref(), link.map(host)], PART_SEPARATOR),
                org_name: None,
                dates: Dates::of(p.start_date.as_ref(), p.end_date.as_ref(), true),
                url: link.map(str::to_owned),
                bullets: bullets(&s.highlights, &s.stats),
            }
        })
        .collect()
}

fn education(t: &Tailored) -> Vec<Entry> {
    t.education
        .iter()
        .map(|picked| {
            let e = picked.entry;
            let degree = joined(&[e.study_type.as_deref(), e.area.as_deref()], ", ");
            Entry {
                id: picked.id.to_string(),
                title: degree.clone().unwrap_or_else(|| e.institution.clone()),
                organization: degree.map(|_| e.institution.clone()),
                org_name: Some(e.institution.clone()),
                dates: Dates::of(e.start_date.as_ref(), e.end_date.as_ref(), true),
                url: e.url.clone(),
                bullets: Vec::new(),
            }
        })
        .collect()
}

fn publications(t: &Tailored) -> Vec<Entry> {
    t.publications
        .iter()
        .map(|picked| {
            let p = picked.entry;
            let number = p.extension.as_ref().and_then(|x| x.number.as_deref());
            Entry {
                id: picked.id.to_string(),
                title: p.name.clone(),
                organization: joined(&[number, p.publisher.as_deref()], ", "),
                org_name: p.publisher.clone(),
                dates: Dates::of(None, p.release_date.as_ref(), false),
                url: p.url.clone(),
                bullets: p.summary.iter().cloned().collect(),
            }
        })
        .collect()
}

fn awards(t: &Tailored) -> Vec<Entry> {
    t.awards
        .iter()
        .map(|picked| {
            let a = picked.entry;
            Entry {
                id: picked.id.to_string(),
                title: a.title.clone(),
                organization: a.awarder.clone(),
                org_name: a.awarder.clone(),
                dates: Dates::of(None, a.date.as_ref(), false),
                url: None,
                bullets: a.summary.iter().cloned().collect(),
            }
        })
        .collect()
}

/// Builds the document a tailoring file selects, with the contact details `profile` allows.
pub fn document(t: &Tailored, profile: &ContactProfile) -> Result<ResumeDocument, RenderError> {
    let basics = &t.career.basics;
    let skills = t.skills.iter().map(|k| SkillGroup {
        label: k.name.clone(),
        items: k.keywords.iter().flatten().cloned().collect(),
    });
    let mut sections = vec![Section {
        kind: SectionKind::Summary,
        body: Body::Text(t.summary.to_owned()),
    }];
    let lists = [
        (SectionKind::Experience, Body::Entries(experience(t))),
        (SectionKind::Projects, Body::Entries(projects(t))),
        (SectionKind::Skills, Body::Groups(skills.collect())),
        (SectionKind::Education, Body::Entries(education(t))),
        (SectionKind::Publications, Body::Entries(publications(t))),
        (SectionKind::Awards, Body::Entries(awards(t))),
    ];
    for (kind, body) in lists {
        let empty = matches!(&body, Body::Entries(e) if e.is_empty())
            || matches!(&body, Body::Groups(g) if g.is_empty());
        if !empty {
            sections.push(Section { kind, body });
        }
    }
    Ok(ResumeDocument {
        name: basics.name.clone(),
        title: t.title.to_owned(),
        contact: contact_items(basics, profile)?,
        sections,
        max_pages: t.max_pages,
    })
}
