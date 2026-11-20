//! Required (R) and preferred (P) skills: Σ wᵢ·mᵢ·cᵢ / Σ wᵢ, each skill counted once.
//!
//! - m: 1.0 canonical name, 0.85 taxonomy alias, 0.5 broader or narrower concept, 0 otherwise.
//! - c: 1.0 when the skill appears where the blueprint's `evidence_scope` asks for it. For
//!   `experience_or_projects`, a skill found only in the skills list (or summary) gets 0.8.
//! - Each skill keeps its single best occurrence, so repeating a keyword gains nothing.

use crate::evidence::{Evidence, found};
use crate::units::{Context, EntryId, Unit};
use atsgate_blueprint::{EvidenceScope, Skill};
use atsgate_extract::Document;
use atsgate_taxonomy::{ConceptId, Gazetteer, MatchKind, fold, tokens};
use schemars::JsonSchema;
use serde::Serialize;

pub const M_CANONICAL: f64 = 1.0;
pub const M_ALIAS: f64 = 0.85;
pub const M_RELATED: f64 = 0.5;
pub const C_IN_SCOPE: f64 = 1.0;
pub const C_LIST_ONLY: f64 = 0.8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Requirement {
    Required,
    Preferred,
}

impl Requirement {
    fn rule(self) -> &'static str {
        match self {
            Self::Required => "skill.required",
            Self::Preferred => "skill.preferred",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SkillMatch {
    Canonical,
    Alias,
    Broader,
    Narrower,
}

impl SkillMatch {
    pub fn m(self) -> f64 {
        match self {
            Self::Canonical => M_CANONICAL,
            Self::Alias => M_ALIAS,
            Self::Broader | Self::Narrower => M_RELATED,
        }
    }
    fn of(kind: MatchKind) -> Self {
        match kind {
            MatchKind::Exact => Self::Canonical,
            MatchKind::Alias => Self::Alias,
            MatchKind::Broader => Self::Broader,
            MatchKind::Narrower => Self::Narrower,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct SkillResult {
    pub canonical: String,
    pub requirement: Requirement,
    pub weight: f64,
    /// `null` when the resume never names the skill (then it is also listed under `gaps`).
    pub matched: Option<SkillMatch>,
    pub context: Option<Context>,
    pub m: f64,
    pub c: f64,
    /// m·c: the share of the weight this skill earns.
    pub credit: f64,
    pub evidence: Option<Evidence>,
    #[serde(skip)]
    pub entry: Option<EntryId>,
}

/// One occurrence: (m·c, match, context, surface text, unit index).
type Hit = (f64, SkillMatch, Context, String, usize);

fn c_factor(scope: EvidenceScope, context: Context) -> f64 {
    match scope {
        EvidenceScope::Anywhere => C_IN_SCOPE,
        EvidenceScope::ExperienceOrProjects if context.is_experience_or_project() => C_IN_SCOPE,
        EvidenceScope::ExperienceOrProjects => C_LIST_ONLY,
    }
}

/// Matches of a skill with no taxonomy concept: its canonical name or listed aliases as a
/// folded phrase. One-word names must also match their spelling ("Go" is not "go").
fn phrase_matches(skill: &Skill, text: &str) -> Vec<(SkillMatch, String)> {
    let toks = tokens(text);
    let names = std::iter::once((SkillMatch::Canonical, &skill.canonical))
        .chain(skill.aliases.iter().map(|a| (SkillMatch::Alias, a)));
    let mut out = Vec::new();
    for (kind, name) in names {
        let want: Vec<String> = tokens(name).into_iter().map(|t| t.folded).collect();
        if want.is_empty() || want.len() > toks.len() {
            continue;
        }
        for w in toks.windows(want.len()) {
            let surface = &text[w[0].start..w[want.len() - 1].end];
            let same = w.iter().zip(&want).all(|(t, f)| &t.folded == f);
            if same && (want.len() > 1 || surface == name) {
                out.push((kind, surface.to_owned()));
            }
        }
    }
    out
}

fn concept_matches(
    gazetteer: &Gazetteer<'_>,
    target: ConceptId,
    unit: &Unit,
) -> Vec<(SkillMatch, String)> {
    let taxonomy = gazetteer.taxonomy();
    let mut surfaces: Vec<String> = gazetteer
        .find(&unit.text)
        .into_iter()
        .map(|m| unit.text[m.start..m.end].to_owned())
        .collect();
    if unit.context == Context::SkillsList {
        surfaces.insert(0, unit.text.clone());
    }
    surfaces
        .into_iter()
        .filter_map(|s| {
            let r = taxonomy.relate(&s, target)?;
            Some((SkillMatch::of(r.match_kind), s))
        })
        .collect()
}

fn best_hit(
    skill: &Skill,
    target: Option<ConceptId>,
    gz: &Gazetteer<'_>,
    units: &[Unit],
) -> Option<Hit> {
    let mut best: Option<Hit> = None;
    for (i, unit) in units.iter().enumerate() {
        let matches = match target {
            Some(id) => concept_matches(gz, id, unit),
            None => phrase_matches(skill, &unit.text),
        };
        for (kind, surface) in matches {
            let credit = kind.m() * c_factor(skill.evidence_scope, unit.context);
            if best.as_ref().is_none_or(|b| credit > b.0) {
                best = Some((credit, kind, unit.context, surface, i));
            }
        }
    }
    best
}

/// Scores every skill of one list against the resume units.
pub fn score_skills(
    skills: &[Skill],
    requirement: Requirement,
    gazetteer: &Gazetteer<'_>,
    doc: &Document,
    units: &[Unit],
) -> Vec<SkillResult> {
    let taxonomy = gazetteer.taxonomy();
    skills
        .iter()
        .map(|skill| {
            let target = skill
                .taxonomy_ref
                .as_deref()
                .and_then(|r| taxonomy.by_ref(r))
                .map(|c| c.id());
            let hit = best_hit(skill, target, gazetteer, units);
            let (m, c) = hit.as_ref().map_or((0.0, 0.0), |h| {
                (h.1.m(), c_factor(skill.evidence_scope, h.2))
            });
            SkillResult {
                canonical: skill.canonical.clone(),
                requirement,
                weight: skill.weight,
                matched: hit.as_ref().map(|h| h.1),
                context: hit.as_ref().map(|h| h.2),
                m,
                c,
                credit: m * c,
                evidence: hit.as_ref().map(|h| {
                    let u = &units[h.4];
                    found(requirement.rule(), doc, &u.lines, &h.3, &h.3)
                }),
                entry: hit.as_ref().and_then(|h| units[h.4].entry),
            }
        })
        .collect()
}

/// Σ w·m·c / Σ w over `results`; `None` when the blueprint lists no such skills.
pub fn dimension(results: &[SkillResult]) -> Option<f64> {
    let total: f64 = results.iter().map(|r| r.weight).sum();
    (total > 0.0).then(|| results.iter().map(|r| r.weight * r.credit).sum::<f64>() / total)
}

/// The spellings that would have matched a skill: canonical name, blueprint aliases, and the
/// taxonomy concept's curated aliases.
pub fn would_match(skill: &Skill, gazetteer: &Gazetteer<'_>) -> Vec<String> {
    let taxonomy = gazetteer.taxonomy();
    let mut out = vec![skill.canonical.clone()];
    out.extend(skill.aliases.iter().cloned());
    if let Some(c) = skill
        .taxonomy_ref
        .as_deref()
        .and_then(|r| taxonomy.by_ref(r))
    {
        out.extend(c.curated_aliases().iter().cloned());
    }
    let mut seen = Vec::new();
    out.retain(|n| {
        let key = fold(n);
        let fresh = !seen.contains(&key);
        seen.push(key);
        fresh
    });
    out
}
