//! Checks the schema cannot express: references resolve, ids are unique, dates are ordered, and
//! kind-specific extension fields sit on the right kind of entry.

use crate::date::IsoDate;
use crate::extension::Id;
use crate::item::{ItemExtension, ItemKind};
use crate::{Career, EXTENSION_VERSION};
use std::collections::BTreeSet;

#[derive(Debug, thiserror::Error)]
pub enum CareerError {
    #[error("career file does not match the schema: {0}")]
    Parse(serde_json::Error),
    #[error("x-atsgate.version {found} is not supported (expected {expected})")]
    Version { found: u32, expected: u32 },
    #[error("{item}: statistic {label:?} has no source")]
    UnsourcedStat { item: String, label: String },
    #[error("{item}: source {source_id:?} is not defined in x-atsgate.sources")]
    UnknownSource { item: String, source_id: String },
    #[error("source {0:?} sets both url and path; at most one may locate it")]
    SourceLocator(String),
    #[error("item id {0:?} is used more than once")]
    DuplicateId(String),
    #[error("family {family:?} lists {member:?}, which is not a project id")]
    UnknownFamilyMember { family: String, member: String },
    #[error("{item}: x-atsgate.{field} does not apply to a {kind:?} entry")]
    MisplacedField {
        item: String,
        field: &'static str,
        kind: ItemKind,
    },
    #[error("{item}: x-atsgate.subject {subject:?} is not an item id")]
    UnknownSubject { item: String, subject: String },
    #[error("{item}: a work entry has an endDate but no startDate")]
    EndWithoutStart { item: String },
    #[error("{item}: endDate {end} is before startDate {start}")]
    EndBeforeStart {
        item: String,
        start: String,
        end: String,
    },
}

/// One entry of any section, as validation sees it.
struct Item<'a> {
    kind: ItemKind,
    name: &'a str,
    ext: Option<&'a ItemExtension>,
    start: Option<&'a IsoDate>,
    end: Option<&'a IsoDate>,
}

impl Item<'_> {
    fn label(&self) -> String {
        self.ext
            .map_or_else(|| self.name.to_owned(), |e| e.id.as_str().to_owned())
    }
}

/// An end date alone is normal for a degree (the graduation year) or a finished project, but a
/// role that ended with no known start cannot be placed on a timeline.
fn check_dates(item: &Item) -> Result<(), CareerError> {
    match (item.start, item.end) {
        (None, Some(_)) if item.kind == ItemKind::Work => {
            Err(CareerError::EndWithoutStart { item: item.label() })
        }
        (Some(start), Some(end)) => {
            let n = start.as_str().len().min(end.as_str().len());
            if end.as_str()[..n] < start.as_str()[..n] {
                return Err(CareerError::EndBeforeStart {
                    item: item.label(),
                    start: start.as_str().to_owned(),
                    end: end.as_str().to_owned(),
                });
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

impl Career {
    fn items(&self) -> Vec<Item<'_>> {
        use ItemKind::*;
        let mut out = Vec::new();
        for w in self.work.iter().flatten() {
            let (start, end) = (w.start_date.as_ref(), w.end_date.as_ref());
            out.push(Item {
                kind: Work,
                name: &w.name,
                ext: w.extension.as_ref(),
                start,
                end,
            });
        }
        for v in self.volunteer.iter().flatten() {
            let (start, end) = (v.start_date.as_ref(), v.end_date.as_ref());
            let (name, ext) = (v.organization.as_str(), v.extension.as_ref());
            out.push(Item {
                kind: Volunteer,
                name,
                ext,
                start,
                end,
            });
        }
        for e in self.education.iter().flatten() {
            let (start, end) = (e.start_date.as_ref(), e.end_date.as_ref());
            let (name, ext) = (e.institution.as_str(), e.extension.as_ref());
            out.push(Item {
                kind: Education,
                name,
                ext,
                start,
                end,
            });
        }
        for p in self.projects.iter().flatten() {
            let (start, end) = (p.start_date.as_ref(), p.end_date.as_ref());
            out.push(Item {
                kind: Project,
                name: &p.name,
                ext: p.extension.as_ref(),
                start,
                end,
            });
        }
        for p in self.publications.iter().flatten() {
            let ext = p.extension.as_ref();
            out.push(Item {
                kind: Publication,
                name: &p.name,
                ext,
                start: None,
                end: None,
            });
        }
        for a in self.awards.iter().flatten() {
            let ext = a.extension.as_ref();
            out.push(Item {
                kind: Award,
                name: &a.title,
                ext,
                start: None,
                end: None,
            });
        }
        out
    }

    pub fn validate(&self) -> Result<(), CareerError> {
        let ext = &self.extension;
        if ext.version != EXTENSION_VERSION {
            return Err(CareerError::Version {
                found: ext.version,
                expected: EXTENSION_VERSION,
            });
        }
        if let Some((id, _)) = ext
            .sources
            .iter()
            .find(|(_, s)| s.url.is_some() && s.path.is_some())
        {
            return Err(CareerError::SourceLocator(id.as_str().to_owned()));
        }
        let items = self.items();
        let mut ids = BTreeSet::new();
        for item in &items {
            check_dates(item)?;
            let Some(x) = item.ext else { continue };
            let name = item.label();
            if !ids.insert(&x.id) {
                return Err(CareerError::DuplicateId(name));
            }
            if let Some(field) = x.misplaced_field(item.kind) {
                let kind = item.kind;
                return Err(CareerError::MisplacedField {
                    item: name,
                    field,
                    kind,
                });
            }
            if let Some(missing) = x.sources.iter().find(|s| !ext.sources.contains_key(*s)) {
                let source_id = missing.as_str().to_owned();
                return Err(CareerError::UnknownSource {
                    item: name,
                    source_id,
                });
            }
            if let Some(stat) = x
                .stats
                .iter()
                .flatten()
                .find(|s| s.source.trim().is_empty())
            {
                let label = stat.label.clone();
                return Err(CareerError::UnsourcedStat { item: name, label });
            }
        }
        for x in items.iter().filter_map(|i| i.ext) {
            if let Some(subject) = x.subject.as_ref().filter(|s| !ids.contains(s)) {
                let (item, subject) = (x.id.as_str().to_owned(), subject.as_str().to_owned());
                return Err(CareerError::UnknownSubject { item, subject });
            }
        }
        let projects: BTreeSet<&Id> = items
            .iter()
            .filter(|i| i.kind == ItemKind::Project)
            .filter_map(|i| i.ext.map(|e| &e.id))
            .collect();
        for family in &ext.families {
            if let Some(m) = family.members.iter().find(|m| !projects.contains(m)) {
                return Err(CareerError::UnknownFamilyMember {
                    family: family.id.as_str().to_owned(),
                    member: m.as_str().to_owned(),
                });
            }
        }
        Ok(())
    }
}
