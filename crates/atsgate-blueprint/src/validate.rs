//! Checks the schema cannot express.

use crate::types::{
    Blueprint, ESCO_OCCUPATION_PREFIX, KnockoutType, ONET_SOC_PATTERN, ProvenanceSource, Skill,
};

#[derive(Debug, thiserror::Error)]
pub enum BlueprintError {
    #[error("blueprint does not match the schema: {0}")]
    Parse(serde_json::Error),
    #[error("occupationalCategory {0:?} is not an O*NET-SOC code ({ONET_SOC_PATTERN})")]
    OccupationalCategory(String),
    #[error("esco_occupation {0:?} is not an ESCO occupation URI")]
    EscoOccupation(String),
    #[error("required_skills is empty: a blueprint must require at least one skill")]
    NoRequiredSkills,
    #[error("skill {0:?}: weight must be finite and greater than zero")]
    Weight(String),
    #[error("skill {0:?} is listed more than once")]
    DuplicateSkill(String),
    #[error("knockout min_years value {0:?} is not a non-negative number")]
    MinYears(String),
    #[error("experience.min_years must be finite and not negative")]
    ExperienceYears,
    #[error("education level {0} is out of range for its framework")]
    EducationLevel(u8),
    #[error("provenance.generator_version must be set exactly when provenance.source is generated")]
    GeneratorVersion,
}

fn onet_soc(code: &str) -> bool {
    let b = code.as_bytes();
    b.len() == 10
        && b[2] == b'-'
        && b[7] == b'.'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 2 || i == 7 || c.is_ascii_digit())
}

fn check_skills<'a>(skills: impl Iterator<Item = &'a Skill>) -> Result<(), BlueprintError> {
    let mut seen = std::collections::BTreeSet::new();
    for s in skills {
        if !(s.weight.is_finite() && s.weight > 0.0) {
            return Err(BlueprintError::Weight(s.canonical.clone()));
        }
        if !seen.insert(s.canonical.to_lowercase()) {
            return Err(BlueprintError::DuplicateSkill(s.canonical.clone()));
        }
    }
    Ok(())
}

impl Blueprint {
    pub fn validate(&self) -> Result<(), BlueprintError> {
        if !onet_soc(&self.occupational_category) {
            return Err(BlueprintError::OccupationalCategory(
                self.occupational_category.clone(),
            ));
        }
        if let Some(uri) = self
            .esco_occupation
            .as_ref()
            .filter(|u| !u.starts_with(ESCO_OCCUPATION_PREFIX))
        {
            return Err(BlueprintError::EscoOccupation(uri.clone()));
        }
        if self.required_skills.is_empty() {
            return Err(BlueprintError::NoRequiredSkills);
        }
        check_skills(self.required_skills.iter().chain(&self.preferred_skills))?;
        for k in self
            .knockouts
            .iter()
            .filter(|k| k.kind == KnockoutType::MinYears)
        {
            if !k
                .value
                .parse::<f64>()
                .is_ok_and(|v| v.is_finite() && v >= 0.0)
            {
                return Err(BlueprintError::MinYears(k.value.clone()));
            }
        }
        if !(self.experience.min_years.is_finite() && self.experience.min_years >= 0.0) {
            return Err(BlueprintError::ExperienceYears);
        }
        if let Some(level) = self.education.min_level {
            let max_ok = level.level <= 8;
            let eqf_ok = level.framework != crate::types::LevelFramework::Eqf || level.level >= 1;
            if !(max_ok && eqf_ok) {
                return Err(BlueprintError::EducationLevel(level.level));
            }
        }
        let generated = self.provenance.source == ProvenanceSource::Generated;
        if generated != self.provenance.generator_version.is_some() {
            return Err(BlueprintError::GeneratorVersion);
        }
        Ok(())
    }
}
