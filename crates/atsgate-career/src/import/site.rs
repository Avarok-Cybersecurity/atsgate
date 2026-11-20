//! The thomaspbraun.com format to a career file. Every source struct is destructured in full, so
//! the compiler rejects a field that is read but never mapped.

use super::ImportError;
use super::ids::Ids;
use super::site_entries::{date, education, project, work};
use super::site_format::*;
use super::site_patent::patent;
use crate::extension::{Extension, FocusArea, Id, Source, Variants};
use crate::item::{BasicsExtension, ProfileExtension};
use crate::resume::{Basics, Keywords, Location, Profile};
use crate::{Career, EXTENSION_VERSION};
use std::collections::BTreeMap;

/// Country names the source writes, and the ISO 3166-1 code JSON Resume stores.
const COUNTRIES: [(&str, &str); 1] = [("United States", "US")];

fn location(text: &str) -> Result<Location, ImportError> {
    let parts: Vec<&str> = text.split(", ").collect();
    let [city, region, country] = parts.as_slice() else {
        return Err(ImportError::unrepresentable(
            "person.location",
            "expected \"City, Region, Country\"",
        ));
    };
    let code = COUNTRIES
        .iter()
        .find(|(name, _)| name == country)
        .map(|(_, code)| *code);
    let code = code.ok_or_else(|| {
        ImportError::unrepresentable(
            "person.location",
            format!("no country code for {country:?}"),
        )
    })?;
    Ok(Location {
        address: None,
        postal_code: None,
        city: Some((*city).to_owned()),
        region: Some((*region).to_owned()),
        country_code: Some(code.to_owned()),
    })
}

/// The variant ids, checked like every other id in the career file.
fn titles(variants: BTreeMap<String, String>) -> Result<BTreeMap<Id, String>, ImportError> {
    variants
        .into_iter()
        .map(|(key, title)| {
            let field = format!("person.titleVariants.{key}");
            let id = Id::try_from(key).map_err(|e| ImportError::unrepresentable(&field, e))?;
            Ok((id, title))
        })
        .collect()
}

fn basics(person: Person, summary: String) -> Result<(Basics, BTreeMap<Id, String>), ImportError> {
    let Person {
        name,
        full_name,
        headline,
        title_variants,
        email,
        website,
        profiles,
        location: place,
    } = person;
    let profiles = profiles.into_iter().map(
        |SiteProfile {
             label,
             url,
             icon,
             kind,
             handle,
         }| Profile {
            network: label,
            username: handle,
            url: Some(url),
            extension: Some(ProfileExtension {
                kind,
                icon: Some(icon),
            }),
        },
    );
    let basics = Basics {
        name,
        label: Some(headline),
        image: None,
        email: Some(email),
        phone: None,
        url: Some(website),
        summary: Some(summary),
        location: Some(location(&place)?),
        profiles: Some(profiles.collect()),
        extension: Some(BasicsExtension {
            full_name: Some(full_name),
        }),
    };
    Ok((basics, titles(title_variants)?))
}

fn sources(map: BTreeMap<String, SiteSource>) -> Result<BTreeMap<Id, Source>, ImportError> {
    let one = |key: String, source: SiteSource| {
        let SiteSource {
            title,
            url,
            path,
            retrieved,
            note,
        } = source;
        let id = Id::try_from(key.clone()).map_err(|e| ImportError::unrepresentable(&key, e))?;
        let retrieved = date(&key, retrieved)?;
        Ok((
            id,
            Source {
                title,
                url,
                path,
                retrieved,
                note,
            },
        ))
    };
    map.into_iter().map(|(k, s)| one(k, s)).collect()
}

pub fn convert(site: SiteCareer) -> Result<Career, ImportError> {
    let SiteCareer {
        schema: _,
        person,
        summary,
        experience,
        patent: pat,
        education: schools,
        projects,
        skills,
        sources: srcs,
    } = site;
    let (basics, titles) = basics(person, summary)?;
    let mut ids = Ids::default();
    let work = experience
        .into_iter()
        .map(|r| work(r, &mut ids))
        .collect::<Result<_, _>>()?;
    let projects = projects
        .into_iter()
        .map(|p| project(p, &mut ids))
        .collect::<Result<_, _>>()?;
    let education = schools
        .into_iter()
        .map(|s| education(s, &mut ids))
        .collect::<Result<_, _>>()?;
    let (publication, award) = patent(pat, &mut ids)?;
    let Skills { groups, areas } = skills;
    let skills = groups
        .into_iter()
        .map(|SkillGroup { name, items }| Keywords {
            name,
            level: None,
            keywords: Some(items),
        });
    let focus = areas.into_iter().map(|Area { name, text }| FocusArea {
        name,
        summary: text,
    });
    Ok(Career {
        schema: None,
        basics,
        work: Some(work),
        volunteer: None,
        education: Some(education),
        awards: Some(vec![award]),
        certificates: None,
        publications: Some(vec![publication]),
        skills: Some(skills.collect()),
        languages: None,
        interests: None,
        references: None,
        projects: Some(projects),
        meta: None,
        extension: Extension {
            version: EXTENSION_VERSION,
            sources: sources(srcs)?,
            families: Vec::new(),
            variants: Variants {
                summaries: BTreeMap::new(),
                titles,
            },
            focus_areas: Some(focus.collect()),
        },
    })
}
