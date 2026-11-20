//! Infers an O*NET-SOC occupation from skills when the title alone does not name one: the
//! occupation whose O*NET Technology Skills cover the most required skills (then preferred).

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
            (r >= MIN_SHARED_SKILLS).then(|| ((r, p), code.to_owned(), c.canonical().to_owned()))
        })
        .max_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)))
        .map(|(_, code, title)| (code, title))
}
