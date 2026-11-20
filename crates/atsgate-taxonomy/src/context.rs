//! Titles that name an occupation only in context ("Engineer", "Systems Engineer"): curated in
//! `contextual_titles`, each read as one occupation when the surrounding occupation's O*NET-SOC
//! code starts with the entry's group.

use crate::curated::ContextualTitle;
use crate::data::ConceptKind;
use crate::fold::fold;
use crate::lookup::{MatchKind, Resolution};
use crate::model::{Taxonomy, TaxonomyError};

/// The prefix of O*NET-SOC occupation references.
pub const ONET_SOC_PREFIX: &str = "onet-soc:";

impl Taxonomy {
    pub(crate) fn link_contextual(
        &mut self,
        entries: &[ContextualTitle],
    ) -> Result<(), TaxonomyError> {
        for e in entries {
            let id = self.require(&e.reference)?;
            if self.concept(id).kind() != ConceptKind::Occupation {
                return Err(TaxonomyError::Curated(format!(
                    "contextual title {:?} names {}, which is not an occupation",
                    e.title, e.reference
                )));
            }
            self.contextual.push((fold(&e.title), e.group.clone(), id));
        }
        Ok(())
    }

    /// The occupation `title` names when read inside `context`, an O*NET-SOC code such as
    /// `15-1252.00`: the curated entry for the folded title whose group `context` starts with.
    pub fn contextual_title(&self, title: &str, context: &str) -> Option<Resolution> {
        let key = fold(title);
        let context = context.strip_prefix(ONET_SOC_PREFIX).unwrap_or(context);
        let &(_, _, id) = self
            .contextual
            .iter()
            .find(|(t, group, _)| *t == key && context.starts_with(group.as_str()))?;
        let c = self.concept(id);
        Some(Resolution {
            canonical: c.canonical().to_owned(),
            taxonomy_ref: c.taxonomy_ref().to_owned(),
            match_kind: MatchKind::Alias,
            concept: id,
        })
    }
}
