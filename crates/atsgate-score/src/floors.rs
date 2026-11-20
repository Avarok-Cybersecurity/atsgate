//! Vendor floors: the owner's defaults (taken from ats-screener's thresholds.md) and overrides
//! written as `taleo=80, lever=0`.

use crate::vendor::{PROFILES, Vendor};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FloorsError {
    #[error(
        "unknown vendor {0:?} in floors; vendors: workday, taleo, icims, greenhouse, lever, successfactors"
    )]
    UnknownVendor(String),
    #[error("floor {0:?} is not `vendor=N`")]
    Malformed(String),
    #[error("floor for {vendor} must be a whole number from 0 to 100, not {value:?}")]
    BadValue { vendor: String, value: String },
    #[error("{0} is given twice in floors")]
    Repeated(String),
}

/// A floor (0-100) per vendor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Floors(BTreeMap<Vendor, u8>);

impl Floors {
    /// The default floors: Taleo 75, Workday 70, SuccessFactors 65, iCIMS 60, Greenhouse 50,
    /// Lever 50.
    pub fn defaults() -> Self {
        Self(PROFILES.iter().map(|p| (p.vendor, p.floor)).collect())
    }

    /// The defaults with `overrides` applied. An empty string overrides nothing.
    pub fn parse(overrides: &str) -> Result<Self, FloorsError> {
        let mut floors = Self::defaults();
        let mut seen = Vec::new();
        for item in overrides
            .split([',', '\n'])
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let (name, value) = item
                .split_once('=')
                .ok_or_else(|| FloorsError::Malformed(item.to_owned()))?;
            let name = name.trim().to_ascii_lowercase();
            let profile = PROFILES
                .iter()
                .find(|p| p.key == name)
                .ok_or_else(|| FloorsError::UnknownVendor(name.clone()))?;
            let floor: u8 = value
                .trim()
                .parse()
                .ok()
                .filter(|v| *v <= 100)
                .ok_or_else(|| FloorsError::BadValue {
                    vendor: name.clone(),
                    value: value.trim().to_owned(),
                })?;
            if seen.contains(&profile.vendor) {
                return Err(FloorsError::Repeated(name));
            }
            seen.push(profile.vendor);
            floors.0.insert(profile.vendor, floor);
        }
        Ok(floors)
    }

    /// Every vendor has a floor: both constructors start from the defaults.
    pub fn get(&self, vendor: Vendor) -> u8 {
        self.0[&vendor]
    }
}
