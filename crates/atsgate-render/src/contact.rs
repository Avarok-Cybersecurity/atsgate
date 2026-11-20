//! Contact profiles. `Public` prints what a public career file holds: city and region, email,
//! website and identity profiles. `Private` adds the phone and street from a private contact
//! file. Neither ever reads a phone or street from the career file itself.

use crate::error::RenderError;
use atsgate_career::{Basics, PrivateContact, ProfileKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContactProfile {
    Public,
    Private(PrivateContact),
}

impl ContactProfile {
    /// Whether this profile would print a phone number or a street address.
    pub fn has_private_fields(&self) -> bool {
        matches!(self, Self::Private(p) if p.phone.is_some() || p.street.is_some())
    }
}

/// Refuses private contact details in CI unless the caller explicitly allowed them. The caller
/// says whether it runs in CI; this crate never reads the environment.
pub fn guard_private_contact(
    profile: &ContactProfile,
    in_ci: bool,
    allow_private: bool,
) -> Result<(), RenderError> {
    if in_ci && profile.has_private_fields() && !allow_private {
        return Err(RenderError::PrivateContactInCi);
    }
    Ok(())
}

/// `https://example.com/` as `example.com`.
pub fn host(url: &str) -> &str {
    let bare = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    bare.strip_suffix('/').unwrap_or(bare)
}

/// The contact items in print order: street, city and region, phone, email, website, profiles.
pub fn contact_items(
    basics: &Basics,
    profile: &ContactProfile,
) -> Result<Vec<String>, RenderError> {
    let location = basics.location.as_ref();
    let in_career = [
        ("basics.phone", basics.phone.is_some()),
        (
            "basics.location.address",
            location.is_some_and(|l| l.address.is_some()),
        ),
        (
            "basics.location.postalCode",
            location.is_some_and(|l| l.postal_code.is_some()),
        ),
    ];
    if let Some((field, _)) = in_career.iter().find(|(_, set)| *set) {
        return Err(RenderError::PrivateFieldInCareer(field));
    }
    let private = match profile {
        ContactProfile::Public => None,
        ContactProfile::Private(p) => Some(p),
    };
    let mut items = Vec::new();
    items.extend(private.and_then(|p| p.street.clone()));
    if let Some(l) = location {
        let place: Vec<&str> = [&l.city, &l.region, &l.country_code]
            .into_iter()
            .filter_map(|p| p.as_deref())
            .collect();
        if !place.is_empty() {
            items.push(place.join(", "));
        }
    }
    items.extend(private.and_then(|p| p.phone.clone()));
    items.extend(basics.email.clone());
    items.extend(basics.url.as_deref().map(|u| host(u).to_owned()));
    for p in basics.profiles.iter().flatten() {
        let product = p
            .extension
            .as_ref()
            .is_some_and(|x| x.kind == ProfileKind::Product);
        if product {
            continue;
        }
        let shown = p
            .username
            .clone()
            .or_else(|| p.url.as_deref().map(|u| host(u).to_owned()));
        items.extend(shown);
    }
    Ok(items)
}
