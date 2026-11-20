//! Infers an O*NET-SOC occupation from skills when the title alone does not name one.
//!
//! Each candidate scores `shared × ln(list_len)`: the required skills its O*NET Technology Skills
//! list shares with the posting, weighted by the log of that list's length (ties: preferred
//! skills shared, then the lower code). Raw overlap picked narrow specializations whose short
//! list happens to name a stack (Blockchain Engineers, 50 entries, for Rust + TypeScript + React +
//! PostgreSQL), and dividing by list length (`shared / sqrt(len)`, Jaccard) makes that worse,
//! because the narrow lists are the short ones. A long list is O*NET's record of an occupation
//! observed across many postings, so its length works as a prior on how common the occupation
//! is; the log keeps that prior from outweighing more than one shared skill.

use atsgate_taxonomy::{ConceptId, ConceptKind, SourceName, Taxonomy, fold};
use std::collections::BTreeSet;

/// Fewer required skills than this in common is not evidence enough to name an occupation.
pub const MIN_SHARED_SKILLS: usize = 2;

/// The O*NET concept that a skill concept stands for (itself, or the O*NET concept with the same
/// folded canonical name).
fn onet_equivalent(taxonomy: &Taxonomy, id: ConceptId) -> Option<ConceptId> {
    let c = taxonomy.concept(id);
    if c.source() == SourceName::Onet {
        return Some(id);
    }
    let key = fold(c.canonical());
    taxonomy
        .resolve_skill(c.canonical())
        .filter(|r| taxonomy.concept(r.concept).source() == SourceName::Onet)
        .map(|r| r.concept)
        .or_else(|| {
            c.aliases()
                .filter_map(|a| taxonomy.resolve_skill(a))
                .find(|r| {
                    let o = taxonomy.concept(r.concept);
                    o.source() == SourceName::Onet && fold(o.canonical()) == key
                })
                .map(|r| r.concept)
        })
}

/// `shared` skills in common, weighted by the log of the occupation's technology-list length.
pub fn breadth_weighted(shared: usize, list_len: usize) -> f64 {
    shared as f64 * (list_len as f64).ln()
}

/// (O*NET-SOC code, occupation title) with the best skill overlap, or `None` below the minimum.
pub fn infer(
    taxonomy: &Taxonomy,
    required: &[ConceptId],
    preferred: &[ConceptId],
) -> Option<(String, String)> {
    let set = |ids: &[ConceptId]| -> BTreeSet<ConceptId> {
        ids.iter()
            .filter_map(|&id| onet_equivalent(taxonomy, id))
            .collect()
    };
    let (req, pref) = (set(required), set(preferred));
    taxonomy
        .concepts()
        .filter(|c| c.kind() == ConceptKind::Occupation && c.source() == SourceName::Onet)
        .filter_map(|c| {
            let skills: BTreeSet<ConceptId> = c.skills().iter().copied().collect();
            let r = req.intersection(&skills).count();
            let p = pref.intersection(&skills).count();
            let code = c.taxonomy_ref().strip_prefix("onet-soc:")?;
            let score = breadth_weighted(r, skills.len());
            (r >= MIN_SHARED_SKILLS).then(|| (score, p, code.to_owned(), c.canonical().to_owned()))
        })
        .max_by(|a, b| {
            a.0.total_cmp(&b.0)
                .then_with(|| a.1.cmp(&b.1))
                .then_with(|| b.2.cmp(&a.2))
        })
        .map(|(_, _, code, title)| (code, title))
}
