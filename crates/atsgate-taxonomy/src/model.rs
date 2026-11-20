//! The loaded taxonomy: concepts from every source, curated overrides applied, and a folded-label
//! index for lookup.

use crate::curated::{CURATED_PREFIX, Curated, CuratedEntry};
use crate::data::{self, ConceptKind, DataError, ESCO_URI_PREFIX, SourceName};
use crate::fold::fold;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConceptId(pub u32);

/// Skills and job titles are looked up separately: "Marketing" the skill is not a title.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Namespace {
    Skill,
    Title,
}

impl Namespace {
    pub fn of(kind: ConceptKind) -> Self {
        match kind {
            ConceptKind::Skill | ConceptKind::SkillGroup => Namespace::Skill,
            ConceptKind::Occupation => Namespace::Title,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Concept {
    pub kind: ConceptKind,
    pub source: SourceName,
    pub reference: String,
    pub canonical: String,
    /// The source label when the curated table renamed the concept.
    pub label: Option<String>,
    pub aliases: Vec<String>,
    pub curated_aliases: Vec<String>,
    pub broader: Vec<ConceptId>,
    pub narrower: Vec<ConceptId>,
    pub skills: Vec<ConceptId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IndexEntry {
    pub concept: ConceptId,
    /// The key is the concept's canonical name (an exact match) rather than an alias.
    pub exact: bool,
    pub curated: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum TaxonomyError {
    #[error(transparent)]
    Data(#[from] DataError),
    #[error("curated table: {0}")]
    Curated(String),
}

/// Canonical skills and occupations with their aliases and broader/narrower links.
#[derive(Debug)]
pub struct Taxonomy {
    pub(crate) concepts: Vec<Concept>,
    pub(crate) by_ref: HashMap<String, ConceptId>,
    pub(crate) index: HashMap<(Namespace, String), Vec<IndexEntry>>,
    pub(crate) versions: BTreeMap<String, String>,
    pub(crate) credits: Vec<String>,
    pub(crate) case_sensitive: Vec<String>,
    pub(crate) stoplist: HashSet<String>,
}

fn source_key(s: SourceName) -> &'static str {
    match s {
        SourceName::Curated => "atsgate-curated",
        SourceName::Onet => "onet",
        SourceName::Esco => "esco",
    }
}

impl Taxonomy {
    /// Builds a taxonomy from compiled data files (`.jsonl.zst`) and the curated table.
    pub fn from_sources(datasets: &[&[u8]], curated: &Curated) -> Result<Self, TaxonomyError> {
        let mut t = Taxonomy {
            concepts: Vec::new(),
            by_ref: HashMap::new(),
            index: HashMap::new(),
            versions: BTreeMap::new(),
            credits: Vec::new(),
            case_sensitive: curated.case_sensitive.clone(),
            stoplist: curated.gazetteer_stoplist.iter().map(|s| fold(s)).collect(),
        };
        for bytes in datasets {
            let (header, records) = data::decode(bytes)?;
            t.versions
                .insert(source_key(header.source).to_owned(), header.version);
            t.credits.push(header.credit);
            let offset = t.concepts.len() as u32;
            let prefix = if header.source == SourceName::Esco {
                ESCO_URI_PREFIX
            } else {
                ""
            };
            for r in records {
                let id = |i: &u32| ConceptId(offset + i);
                t.push(Concept {
                    kind: r.kind,
                    source: header.source,
                    reference: format!("{prefix}{}", r.reference),
                    canonical: r.label,
                    label: None,
                    aliases: r.aliases,
                    curated_aliases: Vec::new(),
                    broader: r.broader.iter().map(id).collect(),
                    narrower: Vec::new(),
                    skills: r.skills.iter().map(id).collect(),
                })?;
            }
        }
        t.versions.insert(
            source_key(SourceName::Curated).to_owned(),
            curated.version.clone(),
        );
        for (entries, kind) in [
            (&curated.skills, ConceptKind::Skill),
            (&curated.titles, ConceptKind::Occupation),
        ] {
            for e in entries {
                t.apply_curated(e, kind)?;
            }
        }
        for d in &curated.denied_aliases {
            let id = t.require(&d.reference)?;
            let aliases = &mut t.concepts[id.0 as usize].aliases;
            let before = aliases.len();
            aliases.retain(|a| a != &d.alias);
            if aliases.len() == before {
                return Err(TaxonomyError::Curated(format!(
                    "denied alias {:?} is not an alias of {}",
                    d.alias, d.reference
                )));
            }
        }
        t.link_narrower();
        t.build_index();
        Ok(t)
    }

    fn push(&mut self, c: Concept) -> Result<(), TaxonomyError> {
        let id = ConceptId(self.concepts.len() as u32);
        if self.by_ref.insert(c.reference.clone(), id).is_some() {
            return Err(TaxonomyError::Curated(format!(
                "reference {} is defined twice",
                c.reference
            )));
        }
        self.concepts.push(c);
        Ok(())
    }

    fn require(&self, reference: &str) -> Result<ConceptId, TaxonomyError> {
        self.by_ref.get(reference).copied().ok_or_else(|| {
            TaxonomyError::Curated(format!("{reference} is not in the taxonomy data"))
        })
    }

    fn apply_curated(&mut self, e: &CuratedEntry, kind: ConceptKind) -> Result<(), TaxonomyError> {
        let broader = e
            .broader
            .iter()
            .map(|r| self.require(r))
            .collect::<Result<Vec<_>, _>>()?;
        if e.reference.starts_with(CURATED_PREFIX) {
            return self.push(Concept {
                kind,
                source: SourceName::Curated,
                reference: e.reference.clone(),
                canonical: e.canonical.clone(),
                label: None,
                aliases: Vec::new(),
                curated_aliases: e.aliases.clone(),
                broader,
                narrower: Vec::new(),
                skills: Vec::new(),
            });
        }
        let id = self.require(&e.reference)?;
        let c = &mut self.concepts[id.0 as usize];
        if Namespace::of(c.kind) != Namespace::of(kind) {
            return Err(TaxonomyError::Curated(format!(
                "{} is listed under the wrong section",
                e.reference
            )));
        }
        if c.canonical != e.canonical {
            c.label = Some(std::mem::replace(&mut c.canonical, e.canonical.clone()));
        }
        c.curated_aliases.extend(e.aliases.iter().cloned());
        c.broader.extend(broader);
        Ok(())
    }

    fn link_narrower(&mut self) {
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

    fn build_index(&mut self) {
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
