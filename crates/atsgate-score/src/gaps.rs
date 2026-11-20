//! Skills the resume never names, and the spellings that would have matched each.

use crate::report::Gap;
use crate::skills::{Requirement, SkillResult};
use atsgate_blueprint::{Blueprint, Skill};
use atsgate_taxonomy::{Gazetteer, fold};

pub fn gaps(results: &[SkillResult], blueprint: &Blueprint, gz: &Gazetteer<'_>) -> Vec<Gap> {
    let all = blueprint
        .required_skills
        .iter()
        .chain(&blueprint.preferred_skills);
    let mut out: Vec<Gap> = results
        .iter()
        .zip(all)
        .filter(|(r, _)| r.matched.is_none())
        .map(|(r, s)| Gap {
            canonical: r.canonical.clone(),
            requirement: r.requirement,
            weight: r.weight,
            would_match: would_match(s, gz),
        })
        .collect();
    out.sort_by(|a, b| {
        let req = |g: &Gap| g.requirement == Requirement::Preferred;
        req(a).cmp(&req(b)).then(b.weight.total_cmp(&a.weight))
    });
    out
}

/// The most spellings a gap lists: every curated alias list fits, and a reader can still scan it.
pub const WOULD_MATCH_CAP: usize = 10;

/// The spellings that would have matched a skill, folded duplicates removed, at most
/// [`WOULD_MATCH_CAP`]: the canonical name, the blueprint's aliases, then every alias the
/// taxonomy indexes for the concept (curated, the source label if renamed, and the source's own
/// alternative labels), since [`atsgate_taxonomy::Taxonomy::relate`] accepts each of them.
pub fn would_match(skill: &Skill, gazetteer: &Gazetteer<'_>) -> Vec<String> {
    let taxonomy = gazetteer.taxonomy();
    let mut out = vec![skill.canonical.clone()];
    out.extend(skill.aliases.iter().cloned());
    if let Some(c) = skill
        .taxonomy_ref
        .as_deref()
        .and_then(|r| taxonomy.by_ref(r))
    {
        out.extend(c.aliases().map(str::to_owned));
    }
    let mut seen = Vec::new();
    out.retain(|n| {
        let key = fold(n);
        let fresh = !seen.contains(&key);
        seen.push(key);
        fresh
    });
    out.truncate(WOULD_MATCH_CAP);
    out
}
