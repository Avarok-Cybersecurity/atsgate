//! Skill mentions per section, turned into blueprint skills.

use super::requirements::{self, LANGUAGES};
use super::sections::{Line, Section};
use crate::{EvidenceScope, Skill};
use atsgate_taxonomy::{ConceptId, ConceptKind, ConceptView, Gazetteer, fold};
use std::collections::BTreeMap;

/// Every generated skill carries the same weight; a reviewer adjusts it.
pub const GENERATED_WEIGHT: f64 = 1.0;

/// O*NET software categories so common to office work that a mention outside the requirements
/// says nothing about the role: office suites, word processing, presentations, spreadsheets,
/// e-mail, calendars, and CAD/CAM.
pub const GENERIC_SOFTWARE_CATEGORIES: [&str; 9] = [
    "onet:2.E.1.k",
    "onet:2.E.4.d",
    "onet:2.E.4.f",
    "onet:2.E.4.i",
    "onet:2.E.16.a",
    "onet:2.E.4.h",
    "onet:2.E.9.c",
    "onet:2.E.9.k",
    "onet:2.E.9.m",
];

/// Generic tools filed under a category too broad to exclude whole (RSS sits beside React and
/// Next.js under "Web platform development software").
pub const GENERIC_SOFTWARE: [&str; 1] = ["onet-tech:RSS"];

/// Generic software is never a domain keyword; named in the requirements it is still a skill.
pub fn is_generic_software(c: &ConceptView<'_>) -> bool {
    GENERIC_SOFTWARE.contains(&c.taxonomy_ref())
        || c.broader()
            .any(|b| GENERIC_SOFTWARE_CATEGORIES.contains(&b.taxonomy_ref()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Bucket {
    Required,
    Preferred,
    Domain,
}

#[derive(Debug, Default)]
pub struct Found {
    pub required: Vec<Skill>,
    pub preferred: Vec<Skill>,
    pub domain_keywords: Vec<String>,
    pub required_ids: Vec<ConceptId>,
    pub preferred_ids: Vec<ConceptId>,
}

fn bucket(section: Section, has_required_cue: bool) -> Option<Bucket> {
    match (section, has_required_cue) {
        (Section::Required, _) => Some(Bucket::Required),
        (Section::Preferred, _) => Some(Bucket::Preferred),
        (Section::Responsibilities, true) => Some(Bucket::Preferred),
        (Section::Body, true) => Some(Bucket::Domain),
        (Section::Responsibilities | Section::Body, false) => Some(Bucket::Required),
        (Section::Ignored, _) => None,
    }
}

/// The aliases a blueprint lists for a skill: the curated ones (short, reviewed) only.
pub fn blueprint_aliases(c: &ConceptView<'_>) -> Vec<String> {
    c.curated_aliases().to_vec()
}

fn skill(c: &ConceptView<'_>) -> Skill {
    Skill {
        canonical: c.canonical().to_owned(),
        taxonomy_ref: Some(c.taxonomy_ref().to_owned()),
        aliases: blueprint_aliases(c),
        weight: GENERATED_WEIGHT,
        evidence_scope: EvidenceScope::Anywhere,
    }
}

/// Characters that separate the items of a stack list.
const LIST_SEPARATORS: [char; 6] = [',', ';', '/', '[', ']', '|'];

/// The items of a stack list with their byte offsets in `list`.
fn list_items(list: &str) -> impl Iterator<Item = (usize, &str)> {
    list.split(LIST_SEPARATORS).filter_map(move |raw| {
        let item = raw.trim_matches(|c: char| c.is_whitespace() || c == '.' || c == '\u{2026}');
        let start = item.as_ptr() as usize - list.as_ptr() as usize;
        (!item.is_empty()).then_some((start, item))
    })
}

pub fn collect(lines: &[Line<'_>], gazetteer: &Gazetteer<'_>) -> Found {
    let taxonomy = gazetteer.taxonomy();
    let has_required_cue = lines.iter().any(|l| l.section == Section::Required);
    let languages: Vec<String> = LANGUAGES.iter().map(|l| fold(l)).collect();
    // folded canonical -> (best bucket, first position, concept)
    let mut best: BTreeMap<String, (Bucket, usize, ConceptId)> = BTreeMap::new();
    for line in lines {
        let Some(b) = bucket(line.section, has_required_cue) else {
            continue;
        };
        // A degree's field of study ("BS in Computer Science") is education, not a skill.
        let degree_fields: Vec<String> = requirements::degree(line.text)
            .map(|(_, fields)| fields.iter().map(|f| fold(f)).collect())
            .unwrap_or_default();
        let listed = line.list.into_iter().flat_map(|list| {
            let at = list.as_ptr() as usize - line.text.as_ptr() as usize;
            list_items(list).filter_map(move |(start, item)| {
                let r = taxonomy.resolve_skill(item)?;
                Some((at + start, r.concept))
            })
        });
        let mentions = gazetteer.find(line.text).into_iter().map(|m| (m.start, m.concept));
        for (start, concept) in mentions.chain(listed) {
            let c = taxonomy.concept(concept);
            if c.kind() != ConceptKind::Skill {
                continue;
            }
            let key = fold(c.canonical());
            if languages.contains(&key) || degree_fields.contains(&key) {
                continue;
            }
            let pos = line.offset + start;
            let entry = best.entry(key).or_insert((b, pos, concept));
            if b < entry.0 {
                *entry = (b, entry.1, concept);
            }
        }
    }
    let mut ordered: Vec<(Bucket, usize, ConceptId)> = best.into_values().collect();
    ordered.sort_by_key(|(b, pos, _)| (*b, *pos));
    let mut found = Found::default();
    for (b, _, id) in ordered {
        let c = taxonomy.concept(id);
        match b {
            Bucket::Required => {
                found.required.push(skill(&c));
                found.required_ids.push(id);
            }
            Bucket::Preferred => {
                found.preferred.push(skill(&c));
                found.preferred_ids.push(id);
            }
            Bucket::Domain if is_generic_software(&c) => {}
            Bucket::Domain => found.domain_keywords.push(c.canonical().to_owned()),
        }
    }
    found
}
