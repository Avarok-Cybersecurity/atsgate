//! A job title to its O*NET occupation, as both the generator and the title score read it.
//!
//! The title is tried as written, without seniority words ("Senior"), and without technology
//! qualifiers as well ("Rust Software Engineer" is a Software Engineer who uses Rust). When the
//! title is read against an occupation (the blueprint's, or the one the posting's skills point
//! to), ambiguous titles take that context: the context's own occupation wins when the title
//! names it among others ("Systems Engineer" lists Software Developers beside Logistics
//! Engineers), then the curated `contextual_titles` apply ("Engineer" is a software title in
//! 15-12xx). A qualifier is context too: the occupation O*NET links to "Rust". Only then does
//! the title's first O*NET reading stand, and never for a title as bare as "Engineer".

use super::title::{GENERIC_TITLES, TITLE_HEADS, strip_seniority};
use atsgate_taxonomy::{
    CURATED_PREFIX, ConceptId, ConceptKind, ONET_SOC_PREFIX, Resolution, SourceName, Taxonomy, fold,
};

/// The longest technology qualifier, in words ("Google Cloud", "Amazon Web Services").
pub const MAX_QUALIFIER_WORDS: usize = 3;

/// Whether `words` name a technology (an O*NET technology example or a curated tool): the
/// qualifiers a title can drop. One-letter words never qualify ("R" in "R&D").
fn qualifier(taxonomy: &Taxonomy, words: &[&str]) -> Option<ConceptId> {
    if words.len() == 1 && (words[0].chars().count() < 2 || TITLE_HEADS.contains(&words[0])) {
        return None;
    }
    let r = taxonomy.resolve_skill(&words.join(" "))?;
    let technology =
        r.taxonomy_ref.starts_with("onet-tech:") || r.taxonomy_ref.starts_with(CURATED_PREFIX);
    technology.then_some(r.concept)
}

/// `folded` without technology qualifiers, and the concepts they named.
pub fn strip_qualifiers(folded: &str, taxonomy: &Taxonomy) -> (String, Vec<ConceptId>) {
    let words: Vec<&str> = folded.split_whitespace().collect();
    let (mut kept, mut found) = (Vec::new(), Vec::new());
    let mut i = 0;
    while i < words.len() {
        let longest = MAX_QUALIFIER_WORDS.min(words.len() - i);
        match (1..=longest)
            .rev()
            .find_map(|n| qualifier(taxonomy, &words[i..i + n]).map(|id| (n, id)))
        {
            Some((n, id)) => {
                found.push(id);
                i += n;
            }
            None => {
                kept.push(words[i]);
                i += 1;
            }
        }
    }
    (kept.join(" "), found)
}

/// The O*NET-SOC code of the first occupation whose technology list names one of `skills`.
fn qualifier_context(taxonomy: &Taxonomy, skills: &[ConceptId]) -> Option<String> {
    taxonomy
        .concepts()
        .filter(|c| c.kind() == ConceptKind::Occupation && c.source() == SourceName::Onet)
        .find(|c| c.skills().iter().any(|s| skills.contains(s)))
        .and_then(|c| {
            c.taxonomy_ref()
                .strip_prefix(ONET_SOC_PREFIX)
                .map(str::to_owned)
        })
}

/// The O*NET occupation `title` names, read in `context` (an O*NET-SOC code) when given.
pub fn resolve_occupation(
    title: &str,
    taxonomy: &Taxonomy,
    context: Option<&str>,
) -> Option<Resolution> {
    let folded = fold(title);
    let core = strip_seniority(&folded);
    let (bare, qualifiers) = strip_qualifiers(&core, taxonomy);
    let mut variants: Vec<String> = Vec::new();
    for v in [folded, core, bare] {
        if !v.is_empty() && !variants.contains(&v) {
            variants.push(v);
        }
    }
    let context = context
        .map(|c| c.strip_prefix(ONET_SOC_PREFIX).unwrap_or(c).to_owned())
        .or_else(|| qualifier_context(taxonomy, &qualifiers));
    if let Some(ctx) = context.as_deref() {
        let own = format!("{ONET_SOC_PREFIX}{ctx}");
        let named = variants.iter().find_map(|v| {
            taxonomy
                .title_candidates(v, SourceName::Onet)
                .into_iter()
                .find(|r| r.taxonomy_ref == own)
        });
        if let Some(r) = named.or_else(|| {
            variants
                .iter()
                .find_map(|v| taxonomy.contextual_title(v, ctx))
        }) {
            return Some(r);
        }
    }
    variants
        .iter()
        .filter(|v| !GENERIC_TITLES.contains(&v.as_str()))
        .find_map(|v| taxonomy.resolve_title_from(v, SourceName::Onet))
}
