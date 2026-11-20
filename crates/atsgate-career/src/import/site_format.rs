//! The thomaspbraun.com career format (`content/career.json` in the tbraun96 repository), as it
//! is written. Unknown fields are errors, so a field added there fails the import instead of
//! being dropped; the mapping destructures every struct, so a field added here fails to compile
//! until it is mapped.

use crate::item::{Engagement, ProfileKind};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteCareer {
    /// A pointer to the source format's own schema; it carries no career information, so the
    /// import accepts it and does not carry it over.
    #[serde(rename = "$schema")]
    #[expect(dead_code, reason = "accepted so the source parses; not career data")]
    pub schema: Option<String>,
    pub person: Person,
    pub summary: String,
    pub experience: Vec<Role>,
    pub patent: Patent,
    pub education: Vec<School>,
    pub projects: Vec<SiteProject>,
    pub skills: Skills,
    pub sources: BTreeMap<String, SiteSource>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Person {
    pub name: String,
    pub full_name: String,
    pub short_name: String,
    pub headline: String,
    pub email: String,
    pub website: String,
    pub profiles: Vec<SiteProfile>,
    /// "City, Region, Country".
    pub location: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteProfile {
    pub label: String,
    pub url: String,
    pub icon: String,
    pub kind: ProfileKind,
    pub handle: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Role {
    pub id: String,
    pub title: String,
    pub org: String,
    pub org_url: Option<String>,
    pub unit: Option<String>,
    pub location: Option<String>,
    pub engagement: Option<Engagement>,
    /// `YYYY-MM` or `unknown`.
    pub start: String,
    /// `YYYY-MM`, `present` or `unknown`.
    pub end: String,
    pub summary: String,
    pub bullets: Vec<String>,
    pub logo: Option<String>,
    pub sources: Vec<String>,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Patent {
    pub title: String,
    pub number: String,
    pub url: String,
    pub assignee: String,
    pub granted: String,
    pub text: String,
    pub bullets: Vec<String>,
    pub recognition: Recognition,
    pub image: PatentImage,
    pub sources: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recognition {
    pub title: String,
    pub by: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatentImage {
    pub alt: String,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct School {
    pub degree: String,
    pub field: String,
    pub school: String,
    pub place: String,
    pub year: Option<u16>,
    pub logo: String,
    pub text: String,
    pub on_resume: bool,
    pub sources: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteProject {
    pub id: String,
    pub name: String,
    pub category: String,
    pub kind: String,
    pub role: Option<String>,
    pub url: Option<String>,
    pub repo: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub description: String,
    pub technologies: Vec<String>,
    pub highlights: Vec<String>,
    pub stats: Vec<SiteStat>,
    pub icon: String,
    pub sources: Vec<String>,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteStat {
    pub label: String,
    pub value: String,
    pub source: String,
    pub retrieved: String,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Skills {
    pub groups: Vec<SkillGroup>,
    pub areas: Vec<Area>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillGroup {
    pub name: String,
    pub items: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Area {
    pub name: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteSource {
    pub title: String,
    pub url: Option<String>,
    pub path: Option<String>,
    pub retrieved: String,
    pub note: Option<String>,
}
