//! The `x-atsgate` blocks that sit on individual JSON Resume objects: entries, profiles and basics.

use crate::extension::{Id, Stat};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The JSON Resume sections whose entries may carry an [`ItemExtension`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Work,
    Volunteer,
    Education,
    Project,
    Publication,
    Award,
}

/// How a role was held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Engagement {
    Employee,
    Contract,
    Founder,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Image {
    #[schemars(length(min = 1))]
    pub alt: String,
    pub width: u32,
    pub height: u32,
}

/// Presentation hints for a site. The resume renderers never read them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Display {
    /// A site asset path, such as `/logos/acme.svg`.
    pub logo: Option<String>,
    /// An icon name from the site's own icon set.
    pub icon: Option<String>,
    pub image: Option<Image>,
}

/// The extension on one entry. The kind-specific fields are rejected on other kinds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ItemExtension {
    /// Referenced by tailoring files, families and awards.
    pub id: Id,
    /// Ids of entries in the top-level `x-atsgate.sources`.
    pub sources: Vec<Id>,
    pub stats: Option<Vec<Stat>>,
    /// A note for whoever maintains the file. Never rendered.
    pub note: Option<String>,
    pub display: Option<Display>,
    /// Whether the source format listed the entry on its resume. Tailoring files decide what a
    /// resume shows; this keeps an imported file lossless.
    pub on_resume: Option<bool>,
    /// Work only.
    pub engagement: Option<Engagement>,
    /// Work only: the team, division or product inside the organization.
    pub unit: Option<String>,
    /// Projects only: a short category line.
    pub category: Option<String>,
    /// Projects only: the source repository.
    pub repo: Option<String>,
    /// Education only: where the institution is.
    pub location: Option<String>,
    /// Education only: a description of the studies.
    pub summary: Option<String>,
    /// Publications only: an identifier such as a patent number.
    pub number: Option<String>,
    /// Publications only: highlights, as JSON Resume gives work and projects.
    pub highlights: Option<Vec<String>>,
    /// Awards only: the id of the entry this award recognizes.
    pub subject: Option<Id>,
}

impl ItemExtension {
    /// The first kind-specific field set on an entry of the wrong kind.
    pub fn misplaced_field(&self, kind: ItemKind) -> Option<&'static str> {
        use ItemKind::*;
        let fields: [(&'static str, bool, &[ItemKind]); 9] = [
            ("engagement", self.engagement.is_some(), &[Work]),
            ("unit", self.unit.is_some(), &[Work]),
            ("category", self.category.is_some(), &[Project]),
            ("repo", self.repo.is_some(), &[Project]),
            ("location", self.location.is_some(), &[Education]),
            ("summary", self.summary.is_some(), &[Education]),
            ("number", self.number.is_some(), &[Publication]),
            ("highlights", self.highlights.is_some(), &[Publication]),
            ("subject", self.subject.is_some(), &[Award]),
        ];
        fields
            .into_iter()
            .find(|(_, set, kinds)| *set && !kinds.contains(&kind))
            .map(|(name, _, _)| name)
    }
}

/// Whether a profile is a way to reach the person or a product they made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProfileKind {
    Identity,
    Product,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProfileExtension {
    pub kind: ProfileKind,
    pub icon: Option<String>,
}

/// The longer form of `basics.name`, which is the name a resume prints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BasicsExtension {
    pub full_name: Option<String>,
}
