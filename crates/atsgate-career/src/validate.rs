//! Checks the schema cannot express: references resolve, ids are unique, sources locate.

use crate::extension::{Id, ItemExtension};
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
    #[error("source {0:?} must set exactly one of url or path")]
    SourceLocator(String),
    #[error("item id {0:?} is used more than once")]
    DuplicateId(String),
    #[error("family {family:?} lists {member:?}, which is not a project id")]
    UnknownFamilyMember { family: String, member: String },
}

impl Career {
    fn item_extensions(&self) -> Vec<&ItemExtension> {
        let work = self
            .work
            .iter()
            .flatten()
            .filter_map(|w| w.extension.as_ref());
        let volunteer = self
            .volunteer
            .iter()
            .flatten()
            .filter_map(|v| v.extension.as_ref());
        let education = self
            .education
            .iter()
            .flatten()
            .filter_map(|e| e.extension.as_ref());
        let projects = self
            .projects
            .iter()
            .flatten()
            .filter_map(|p| p.extension.as_ref());
        work.chain(volunteer)
            .chain(education)
            .chain(projects)
            .collect()
    }

    pub fn validate(&self) -> Result<(), CareerError> {
        let ext = &self.extension;
        if ext.version != EXTENSION_VERSION {
            return Err(CareerError::Version {
                found: ext.version,
                expected: EXTENSION_VERSION,
            });
        }
        for (id, source) in &ext.sources {
            if source.url.is_some() == source.path.is_some() {
                return Err(CareerError::SourceLocator(id.as_str().to_owned()));
            }
        }
        let mut ids = BTreeSet::new();
        for item in self.item_extensions() {
            let name = item.id.as_str().to_owned();
            if !ids.insert(item.id.clone()) {
                return Err(CareerError::DuplicateId(name));
            }
            if let Some(missing) = item.sources.iter().find(|s| !ext.sources.contains_key(*s)) {
                return Err(CareerError::UnknownSource {
                    item: name,
                    source_id: missing.as_str().to_owned(),
                });
            }
            for stat in item.stats.iter().flatten() {
                if stat.source.trim().is_empty() {
                    return Err(CareerError::UnsourcedStat {
                        item: name,
                        label: stat.label.clone(),
                    });
                }
            }
        }
        let projects: BTreeSet<&Id> = self
            .projects
            .iter()
            .flatten()
            .filter_map(|p| p.extension.as_ref())
            .map(|e| &e.id)
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
