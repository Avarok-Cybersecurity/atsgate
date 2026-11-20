//! schema.org structured data for the person the resume describes: `Person` with `jobTitle`,
//! `hasOccupation` (one `Occupation` per role), `worksFor` (current employers) and `alumniOf`.
//! Only what the contact profile allows: the public profile carries no phone or street.

use crate::contact::ContactProfile;
use crate::error::RenderError;
use crate::model::{Body, Entry, ResumeDocument};
use crate::site::html::safe_url;
use atsgate_career::{Basics, ProfileKind};
use atsgate_resume::SectionKind;
use serde_json::{Map, Value, json};

fn entries(doc: &ResumeDocument, kind: SectionKind) -> &[Entry] {
    doc.sections
        .iter()
        .find(|s| s.kind == kind)
        .and_then(|s| match &s.body {
            Body::Entries(e) => Some(e.as_slice()),
            _ => None,
        })
        .unwrap_or(&[])
}

fn address(basics: &Basics, profile: &ContactProfile) -> Option<Value> {
    let mut a = Map::new();
    a.insert("@type".into(), json!("PostalAddress"));
    if let ContactProfile::Private(p) = profile {
        p.street
            .iter()
            .for_each(|s| drop(a.insert("streetAddress".into(), json!(s))));
    }
    let l = basics.location.as_ref()?;
    let fields = [
        ("addressLocality", &l.city),
        ("addressRegion", &l.region),
        ("addressCountry", &l.country_code),
    ];
    for (key, value) in fields {
        value
            .iter()
            .for_each(|v| drop(a.insert(key.into(), json!(v))));
    }
    (a.len() > 1).then_some(Value::Object(a))
}

pub fn person(
    doc: &ResumeDocument,
    basics: &Basics,
    profile: &ContactProfile,
) -> Result<String, RenderError> {
    let mut p = Map::new();
    p.insert("@context".into(), json!("https://schema.org"));
    p.insert("@type".into(), json!("Person"));
    p.insert("name".into(), json!(doc.name));
    p.insert("jobTitle".into(), json!(doc.title));
    if let Some(url) = &basics.url {
        p.insert("url".into(), json!(safe_url(url)?));
    }
    if let Some(email) = &basics.email {
        p.insert("email".into(), json!(format!("mailto:{email}")));
    }
    if let ContactProfile::Private(c) = profile {
        c.phone
            .iter()
            .for_each(|t| drop(p.insert("telephone".into(), json!(t))));
    }
    if let Some(a) = address(basics, profile) {
        p.insert("address".into(), a);
    }
    let identity = basics.profiles.iter().flatten().filter(|x| {
        x.extension
            .as_ref()
            .is_none_or(|e| e.kind == ProfileKind::Identity)
    });
    let same_as: Vec<&str> = identity
        .filter_map(|x| x.url.as_deref())
        .map(safe_url)
        .collect::<Result<_, _>>()?;
    if !same_as.is_empty() {
        p.insert("sameAs".into(), json!(same_as));
    }
    let roles = entries(doc, SectionKind::Experience);
    let occupations: Vec<Value> = roles
        .iter()
        .map(|r| json!({ "@type": "Occupation", "name": r.title }))
        .collect();
    let current = roles
        .iter()
        .filter(|r| r.dates.as_ref().is_some_and(|d| d.iso().1.is_none()));
    let works_for: Vec<Value> = current
        .filter_map(|r| r.org_name.as_ref())
        .map(|name| json!({ "@type": "Organization", "name": name }))
        .collect();
    let schools: Vec<Value> = entries(doc, SectionKind::Education)
        .iter()
        .filter_map(|e| e.org_name.as_ref())
        .map(|name| json!({ "@type": "EducationalOrganization", "name": name }))
        .collect();
    for (key, list) in [
        ("hasOccupation", occupations),
        ("worksFor", works_for),
        ("alumniOf", schools),
    ] {
        if !list.is_empty() {
            p.insert(key.into(), Value::Array(list));
        }
    }
    let mut text = serde_json::to_string_pretty(&Value::Object(p))
        .map_err(|e| RenderError::Json(e.to_string()))?;
    text.push('\n');
    Ok(text)
}
