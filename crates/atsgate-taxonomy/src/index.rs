//! The narrower links and the folded-label index, derived from the loaded concepts.

use crate::data::SourceName;
use crate::fold::fold;
use crate::model::{ConceptId, IndexEntry, Namespace, Taxonomy};

impl Taxonomy {
    pub(crate) fn link_narrower(&mut self) {
        let links: Vec<(ConceptId, ConceptId)> = self
            .concepts
            .iter()
            .enumerate()
            .flat_map(|(i, c)| c.broader.iter().map(move |b| (*b, ConceptId(i as u32))))
            .collect();
        for (broader, narrower) in links {
            self.concepts[broader.0 as usize].narrower.push(narrower);
        }
    }

    pub(crate) fn build_index(&mut self) {
        for (i, c) in self.concepts.iter().enumerate() {
            let ns = Namespace::of(c.kind);
            let concept = ConceptId(i as u32);
            let mut add = |text: &str, exact: bool, curated: bool| {
                let key = fold(text);
                if key.is_empty() {
                    return;
                }
                let entries = self.index.entry((ns, key)).or_default();
                match entries.iter_mut().find(|e| e.concept == concept) {
                    Some(e) => {
                        e.exact |= exact;
                        e.curated |= curated;
                    }
                    None => entries.push(IndexEntry {
                        concept,
                        exact,
                        curated,
                    }),
                }
            };
            let curated_concept = c.source == SourceName::Curated || c.label.is_some();
            add(&c.canonical, true, curated_concept);
            for a in &c.curated_aliases {
                add(a, false, true);
            }
            for a in c.label.iter().chain(&c.aliases) {
                add(a, false, false);
            }
        }
    }
}
