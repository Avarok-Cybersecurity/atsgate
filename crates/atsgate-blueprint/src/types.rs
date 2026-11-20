//! The job blueprint: what a role requires, aligned with schema.org JobPosting. Every key is
//! required; a value that does not apply is written as `null` or `[]`, never left out.

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

/// Schema for a required key whose value may be `null` (the counterpart of [`nullable`]).
#[derive(Debug)]
pub struct Nullable<T>(std::marker::PhantomData<T>);

impl<T: JsonSchema> JsonSchema for Nullable<T> {
    fn inline_schema() -> bool {
        true
    }
    fn schema_name() -> std::borrow::Cow<'static, str> {
        format!("Nullable_{}", T::schema_name()).into()
    }
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        <Option<T>>::json_schema(generator)
    }
}

/// Makes an `Option` field required-but-nullable: serde would otherwise accept a missing key.
fn nullable<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(d)
}

pub const ONET_SOC_PATTERN: &str = r"^[0-9]{2}-[0-9]{4}\.[0-9]{2}$";
pub const ESCO_OCCUPATION_PREFIX: &str = "http://data.europa.eu/esco/occupation/";

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Seniority {
    Intern,
    Junior,
    Mid,
    Senior,
    Staff,
    Principal,
    Lead,
    Manager,
    Director,
    Executive,
}

/// Where a skill must appear to earn full credit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceScope {
    Anywhere,
    ExperienceOrProjects,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Skill {
    #[schemars(length(min = 1))]
    pub canonical: String,
    /// An ESCO skill URI or O*NET element id; `null` when the skill is not in a taxonomy.
    #[serde(deserialize_with = "nullable")]
    #[schemars(with = "Nullable<String>")]
    pub taxonomy_ref: Option<String>,
    pub aliases: Vec<String>,
    /// Relative weight; must be finite and greater than zero.
    pub weight: f64,
    pub evidence_scope: EvidenceScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum KnockoutType {
    Degree,
    Certification,
    MinYears,
    Clearance,
    Authorization,
    Language,
}

/// What a knockout resolves to when the resume neither meets nor misses it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UnknownPolicy {
    Pass,
    Fail,
    Review,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Knockout {
    #[serde(rename = "type")]
    pub kind: KnockoutType,
    #[schemars(length(min = 1))]
    pub value: String,
    pub unknown_policy: UnknownPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LevelFramework {
    Isced,
    Eqf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EducationLevel {
    pub framework: LevelFramework,
    /// ISCED 2011: 0-8; EQF: 1-8.
    #[schemars(range(min = 0, max = 8))]
    pub level: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EducationRequirement {
    #[serde(deserialize_with = "nullable")]
    #[schemars(with = "Nullable<EducationLevel>")]
    pub min_level: Option<EducationLevel>,
    pub fields: Vec<String>,
    /// schema.org `experienceInPlaceOfEducation`.
    pub experience_in_place_of_education: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExperienceRequirement {
    pub min_years: f64,
    pub relevant_titles: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceSource {
    Builtin,
    File,
    Url,
    Generated,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub source: ProvenanceSource,
    /// Set exactly when `source` is `generated`.
    #[serde(deserialize_with = "nullable")]
    #[schemars(with = "Nullable<String>")]
    pub generator_version: Option<String>,
    /// Taxonomy name to version, e.g. `{"onet": "31.0", "esco": "1.2.1"}`.
    pub taxonomy_versions: BTreeMap<String, String>,
    pub reviewed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(
    title = "ATSGate job blueprint",
    description = "What a role requires, aligned with schema.org JobPosting."
)]
pub struct Blueprint {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    #[schemars(regex(pattern = r"^[a-z0-9][a-z0-9-]*$"))]
    pub id: String,
    #[schemars(length(min = 1))]
    pub version: String,
    #[schemars(length(min = 1))]
    pub title: String,
    pub title_aliases: Vec<String>,
    /// schema.org `occupationalCategory`, as an O*NET-SOC code.
    #[serde(rename = "occupationalCategory")]
    #[schemars(regex(pattern = ONET_SOC_PATTERN))]
    pub occupational_category: String,
    #[serde(deserialize_with = "nullable")]
    #[schemars(with = "Nullable<String>")]
    pub esco_occupation: Option<String>,
    pub seniority: Seniority,
    pub required_skills: Vec<Skill>,
    pub preferred_skills: Vec<Skill>,
    pub knockouts: Vec<Knockout>,
    pub education: EducationRequirement,
    pub experience: ExperienceRequirement,
    pub certifications: Vec<String>,
    pub domain_keywords: Vec<String>,
    pub languages: Vec<String>,
    pub provenance: Provenance,
}
