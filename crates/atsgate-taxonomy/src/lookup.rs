//! Lookup: free text to a canonical concept, and how one concept relates to another.

use crate::data::{ConceptKind, SourceName};
use crate::fold::fold;
use crate::model::{Concept, ConceptId, IndexEntry, Namespace, Taxonomy};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchKind {
    /// The text is the concept's canonical name (after folding).
    Exact,
    /// The text is one of the concept's aliases.
    Alias,
    /// The text names a concept broader than the target (e.g. "Programming" for "Rust").
    Broader,
    /// The text names a concept narrower than the target.
    Narrower,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Resolution {
    pub canonical: String,
    pub taxonomy_ref: String,
    pub match_kind: MatchKind,
    #[serde(skip)]
    pub concept: ConceptId,
}

/// A read-only view of one concept.
#[derive(Debug, Clone, Copy)]
pub struct ConceptView<'t> {
    taxonomy: &'t Taxonomy,
    id: ConceptId,
    concept: &'t Concept,
}

impl<'t> ConceptView<'t> {
    pub fn id(&self) -> ConceptId {
        self.id
    }
    pub fn kind(&self) -> ConceptKind {
        self.concept.kind
    }
    pub fn source(&self) -> SourceName {
        self.concept.source
    }
    pub fn canonical(&self) -> &'t str {
        &self.concept.canonical
    }
    pub fn taxonomy_ref(&self) -> &'t str {
        &self.concept.reference
    }
    /// Every alias: curated ones first, then the source label (if renamed) and source aliases.
    pub fn aliases(&self) -> impl Iterator<Item = &'t str> {
        let c = self.concept;
        c.curated_aliases
            .iter()
            .chain(&c.label)
            .chain(&c.aliases)
            .map(String::as_str)
    }
    pub fn curated_aliases(&self) -> &'t [String] {
        &self.concept.curated_aliases
    }
    pub fn broader(&self) -> impl Iterator<Item = ConceptView<'t>> {
        let t = self.taxonomy;
        self.concept.broader.iter().map(move |&id| t.concept(id))
    }
    pub fn narrower(&self) -> impl Iterator<Item = ConceptView<'t>> {
        let t = self.taxonomy;
        self.concept.narrower.iter().map(move |&id| t.concept(id))
    }
    /// For occupations: the skills the source links to the occupation.
    pub fn skills(&self) -> &'t [ConceptId] {
        &self.concept.skills
    }
}

fn source_rank(s: SourceName) -> u8 {
    match s {
        SourceName::Curated => 0,
        SourceName::Onet => 1,
        SourceName::Esco => 2,
    }
}

impl Taxonomy {
    pub fn concept(&self, id: ConceptId) -> ConceptView<'_> {
        ConceptView {
            taxonomy: self,
            id,
            concept: &self.concepts[id.0 as usize],
        }
    }

    pub fn concepts(&self) -> impl Iterator<Item = ConceptView<'_>> {
        (0..self.concepts.len() as u32).map(|i| self.concept(ConceptId(i)))
    }

    pub fn by_ref(&self, taxonomy_ref: &str) -> Option<ConceptView<'_>> {
        self.by_ref.get(taxonomy_ref).map(|&id| self.concept(id))
    }

    /// Source name to version, e.g. `{"esco": "v1.2.1", "onet": "31.0", ...}`.
    pub fn versions(&self) -> &BTreeMap<String, String> {
        &self.versions
    }

    /// The attribution lines of the loaded data files.
    pub fn credits(&self) -> &[String] {
        &self.credits
    }

    /// The deterministic preference order among concepts sharing a folded key.
    fn rank(&self, e: &IndexEntry) -> (bool, bool, u8, bool, &str) {
        let c = &self.concepts[e.concept.0 as usize];
        (
            !e.curated,
            !e.exact,
            source_rank(c.source),
            c.kind != ConceptKind::Skill && c.kind != ConceptKind::Occupation,
            c.reference.as_str(),
        )
    }

    pub(crate) fn entries(&self, ns: Namespace, key: &str) -> Vec<IndexEntry> {
        let mut entries = self
            .index
            .get(&(ns, key.to_owned()))
            .cloned()
            .unwrap_or_default();
        entries.sort_by(|a, b| self.rank(a).cmp(&self.rank(b)));
        entries
    }

    fn best(&self, ns: Namespace, text: &str) -> Option<Resolution> {
        self.best_where(ns, text, |_| true)
    }

    fn best_where(
        &self,
        ns: Namespace,
        text: &str,
        keep: impl Fn(&ConceptView<'_>) -> bool,
    ) -> Option<Resolution> {
        let e = self
            .entries(ns, &fold(text))
            .into_iter()
            .find(|e| keep(&self.concept(e.concept)))?;
        let c = self.concept(e.concept);
        Some(Resolution {
            canonical: c.canonical().to_owned(),
            taxonomy_ref: c.taxonomy_ref().to_owned(),
            match_kind: if e.exact {
                MatchKind::Exact
            } else {
                MatchKind::Alias
            },
            concept: e.concept,
        })
    }

    /// Resolves a skill name ("k8s", "Postgres") to its canonical concept.
    pub fn resolve_skill(&self, text: &str) -> Option<Resolution> {
        self.best(Namespace::Skill, text)
    }

    /// Resolves a job title ("Backend Developer") to its canonical occupation.
    pub fn resolve_title(&self, text: &str) -> Option<Resolution> {
        self.best(Namespace::Title, text)
    }

    /// Resolves a job title to an occupation of one source (an O*NET-SOC or ESCO occupation).
    pub fn resolve_title_from(&self, text: &str, source: SourceName) -> Option<Resolution> {
        self.best_where(Namespace::Title, text, |c| c.source() == source)
    }

    /// How `text` relates to the concept `target`: the same concept (exact or alias), or one
    /// link broader or narrower. Concepts from different sources with the same folded canonical
    /// name count as the same concept.
    pub fn relate(&self, text: &str, target: ConceptId) -> Option<Resolution> {
        let t = self.concept(target);
        let ns = Namespace::of(t.kind());
        let target_key = fold(t.canonical());
        let mut found: Option<(MatchKind, ConceptId)> = None;
        for e in self.entries(ns, &fold(text)) {
            let c = self.concept(e.concept);
            let kind = if e.concept == target || fold(c.canonical()) == target_key {
                if e.exact {
                    MatchKind::Exact
                } else {
                    MatchKind::Alias
                }
            } else if t.broader().any(|b| b.id() == e.concept) {
                MatchKind::Broader
            } else if t.narrower().any(|n| n.id() == e.concept) {
                MatchKind::Narrower
            } else {
                continue;
            };
            if found.is_none_or(|(k, _)| kind < k) {
                found = Some((kind, e.concept));
            }
        }
        found.map(|(match_kind, _)| Resolution {
            canonical: t.canonical().to_owned(),
            taxonomy_ref: t.taxonomy_ref().to_owned(),
            match_kind,
            concept: target,
        })
    }
}
