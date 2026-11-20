//! Resolves a tailoring file against a career: every id, index and variant must exist.

use super::resolved::{Picked, Tailored, TailoredEntry, TailoringError};
use super::{EntrySelection, Phrasing, TAILORING_VERSION, Tailoring};
use crate::Career;
use crate::extension::{Id, Stat};
use crate::item::ItemExtension;
use std::collections::BTreeSet;

fn unique<'v, V: ToString + Ord + 'v>(
    section: &'static str,
    values: impl IntoIterator<Item = &'v V>,
) -> Result<(), TailoringError> {
    let mut seen = BTreeSet::new();
    for v in values {
        if !seen.insert(v) {
            return Err(TailoringError::Duplicate {
                section,
                value: v.to_string(),
            });
        }
    }
    Ok(())
}

/// Whole entries by id, in the order the tailoring file lists them.
fn picked<'a, T>(
    section: &'static str,
    items: &'a Option<Vec<T>>,
    ext: fn(&T) -> Option<&ItemExtension>,
    ids: &[Id],
) -> Result<Vec<Picked<'a, T>>, TailoringError> {
    unique(section, ids)?;
    ids.iter()
        .map(|id| find(section, items, ext, id).map(|(entry, x)| Picked { entry, id: &x.id }))
        .collect()
}

fn find<'a, T>(
    section: &'static str,
    items: &'a Option<Vec<T>>,
    ext: fn(&T) -> Option<&ItemExtension>,
    id: &Id,
) -> Result<(&'a T, &'a ItemExtension), TailoringError> {
    items
        .iter()
        .flatten()
        .find_map(|item| ext(item).filter(|x| &x.id == id).map(|x| (item, x)))
        .ok_or_else(|| TailoringError::UnknownId {
            section,
            id: id.as_str().to_owned(),
        })
}

fn pick<'a, V: ?Sized>(
    id: &Id,
    field: &'static str,
    pool: &[&'a V],
    indexes: &[usize],
) -> Result<Vec<&'a V>, TailoringError> {
    unique(field, indexes)?;
    indexes
        .iter()
        .map(|&index| {
            pool.get(index)
                .copied()
                .ok_or_else(|| TailoringError::IndexOutOfRange {
                    id: id.as_str().to_owned(),
                    field,
                    index,
                    len: pool.len(),
                })
        })
        .collect()
}

fn entries<'a, T>(
    section: &'static str,
    items: &'a Option<Vec<T>>,
    ext: fn(&T) -> Option<&ItemExtension>,
    highlights: fn(&T) -> &Option<Vec<String>>,
    selection: &'a [EntrySelection],
) -> Result<Vec<TailoredEntry<'a, T>>, TailoringError> {
    unique(section, selection.iter().map(|s| &s.id))?;
    selection
        .iter()
        .map(|s| {
            let (entry, x) = find(section, items, ext, &s.id)?;
            let pool: Vec<&str> = highlights(entry)
                .iter()
                .flatten()
                .map(String::as_str)
                .collect();
            let stats: Vec<&Stat> = x.stats.iter().flatten().collect();
            Ok(TailoredEntry {
                entry,
                id: &x.id,
                highlights: pick(&s.id, "highlights", &pool, &s.highlights)?,
                stats: pick(&s.id, "stats", &stats, &s.stats)?,
            })
        })
        .collect()
}

fn phrasing<'a>(
    field: &'static str,
    choice: &Phrasing,
    basics: Option<&'a String>,
    variants: &'a std::collections::BTreeMap<Id, String>,
) -> Result<&'a str, TailoringError> {
    match choice {
        Phrasing::Basics => basics
            .map(String::as_str)
            .ok_or(TailoringError::MissingBasics { field }),
        Phrasing::Variant(id) => {
            variants
                .get(id)
                .map(String::as_str)
                .ok_or_else(|| TailoringError::UnknownVariant {
                    field,
                    id: id.as_str().to_owned(),
                })
        }
    }
}

fn blueprint_ok(reference: &str) -> bool {
    match reference.split_once(':') {
        Some(("builtin", id)) => Id::try_from(id.to_owned()).is_ok(),
        Some(("https", rest)) => rest.starts_with("//") && rest.len() > 2,
        Some(_) => false,
        None => {
            !reference.starts_with('/') && !reference.split('/').any(|p| p == ".." || p.is_empty())
        }
    }
}

impl Tailoring {
    /// Resolves this file against `career`, failing on anything that does not already exist.
    pub fn check<'a>(&'a self, career: &'a Career) -> Result<Tailored<'a>, TailoringError> {
        if self.version != TAILORING_VERSION {
            return Err(TailoringError::Version {
                found: self.version,
                expected: TAILORING_VERSION,
            });
        }
        if !blueprint_ok(&self.blueprint) {
            return Err(TailoringError::Blueprint(self.blueprint.clone()));
        }
        if self.max_pages == 0 {
            return Err(TailoringError::MaxPages);
        }
        let variants = &career.extension.variants;
        let (basics, c) = (&career.basics, career);
        let title = phrasing(
            "label",
            &self.title,
            basics.label.as_ref(),
            &variants.titles,
        )?;
        let summary = phrasing(
            "summary",
            &self.summary,
            basics.summary.as_ref(),
            &variants.summaries,
        )?;
        unique("skills", &self.skills)?;
        let skills = self.skills.iter().map(|name| {
            c.skills
                .iter()
                .flatten()
                .find(|k| &k.name == name)
                .ok_or_else(|| TailoringError::UnknownSkillGroup(name.clone()))
        });
        Ok(Tailored {
            career,
            blueprint: &self.blueprint,
            title,
            summary,
            max_pages: self.max_pages,
            work: entries(
                "work",
                &c.work,
                |w| w.extension.as_ref(),
                |w| &w.highlights,
                &self.work,
            )?,
            projects: entries(
                "projects",
                &c.projects,
                |p| p.extension.as_ref(),
                |p| &p.highlights,
                &self.projects,
            )?,
            skills: skills.collect::<Result<_, _>>()?,
            education: picked(
                "education",
                &c.education,
                |e| e.extension.as_ref(),
                &self.education,
            )?,
            publications: picked(
                "publications",
                &c.publications,
                |p| p.extension.as_ref(),
                &self.publications,
            )?,
            awards: picked("awards", &c.awards, |a| a.extension.as_ref(), &self.awards)?,
        })
    }
}
