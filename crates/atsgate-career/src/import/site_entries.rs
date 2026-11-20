//! The thomaspbraun.com entries (roles, projects, schools) to JSON Resume entries. Each source
//! struct is destructured in full, so a field that is never mapped fails to compile.

use super::ImportError;
use super::ids::Ids;
use super::site_format::{Role, School, SiteProject, SiteStat};
use crate::IsoDate;
use crate::extension::{Id, Stat};
use crate::item::{Display, Image, ItemExtension};
use crate::resume::{Education, Project, Work};

pub(super) fn date(item: &str, value: String) -> Result<IsoDate, ImportError> {
    IsoDate::try_from(value).map_err(|e| ImportError::unrepresentable(item, e))
}

/// Source dates: `unknown` and `present` are markers, not dates. JSON Resume writes a current
/// entry as a start with no end, and an undated entry as neither.
fn dates(
    item: &str,
    start: Option<String>,
    end: Option<String>,
) -> Result<(Option<IsoDate>, Option<IsoDate>), ImportError> {
    let marker = |v: &Option<String>, m: &str| v.as_deref() == Some(m);
    match (start, end) {
        (None, None) => Ok((None, None)),
        (s, e) if marker(&s, "unknown") && marker(&e, "unknown") => Ok((None, None)),
        (Some(s), e) if marker(&e, "present") => Ok((Some(date(item, s)?), None)),
        (Some(s), Some(e)) if s != "unknown" && e != "unknown" => {
            Ok((Some(date(item, s)?), Some(date(item, e)?)))
        }
        (s, e) => Err(ImportError::unrepresentable(
            item,
            format!("dates {s:?} to {e:?} have no JSON Resume form"),
        )),
    }
}

pub(super) fn source_ids(ids: Vec<String>, item: &str) -> Result<Vec<Id>, ImportError> {
    ids.into_iter()
        .map(|s| Id::try_from(s).map_err(|e| ImportError::unrepresentable(item, e)))
        .collect()
}

pub(super) fn display(
    logo: Option<String>,
    icon: Option<String>,
    image: Option<Image>,
) -> Option<Display> {
    (logo.is_some() || icon.is_some() || image.is_some()).then_some(Display { logo, icon, image })
}

/// An extension with only the id and sources set; callers fill in what their kind carries.
pub(super) fn ext(id: Id, sources: Vec<Id>) -> ItemExtension {
    ItemExtension {
        id,
        sources,
        stats: None,
        note: None,
        display: None,
        on_resume: None,
        engagement: None,
        unit: None,
        category: None,
        repo: None,
        location: None,
        summary: None,
        number: None,
        highlights: None,
        subject: None,
    }
}

pub(super) fn work(role: Role, ids: &mut Ids) -> Result<Work, ImportError> {
    let Role {
        id,
        title,
        org,
        org_url,
        unit,
        location,
        engagement,
        start,
        end,
        summary,
        bullets,
        logo,
        sources,
        note,
    } = role;
    let (start_date, end_date) = dates(&id, Some(start), Some(end))?;
    let sources = source_ids(sources, &id)?;
    let x = ItemExtension {
        note,
        display: display(logo, None, None),
        engagement,
        unit,
        ..ext(ids.given(&id, id.clone())?, sources)
    };
    Ok(Work {
        name: org,
        position: title,
        start_date,
        end_date,
        location,
        description: None,
        url: org_url,
        summary: Some(summary),
        highlights: Some(bullets),
        extension: Some(x),
    })
}

fn stat(s: SiteStat, item: &str) -> Result<Stat, ImportError> {
    let SiteStat {
        label,
        value,
        source,
        retrieved,
        note,
    } = s;
    Ok(Stat {
        label,
        value,
        source,
        retrieved: date(item, retrieved)?,
        note,
    })
}

pub(super) fn project(p: SiteProject, ids: &mut Ids) -> Result<Project, ImportError> {
    let SiteProject {
        id,
        name,
        category,
        kind,
        role,
        url,
        repo,
        start,
        end,
        description,
        technologies,
        highlights,
        stats,
        icon,
        sources,
        note,
    } = p;
    let (start_date, end_date) = dates(&id, start, end)?;
    let stats = stats
        .into_iter()
        .map(|s| stat(s, &id))
        .collect::<Result<Vec<_>, _>>()?;
    let sources = source_ids(sources, &id)?;
    let base = ext(ids.given(&id, id.clone())?, sources);
    let x = ItemExtension {
        stats: Some(stats),
        note,
        display: display(None, Some(icon), None),
        category: Some(category),
        repo,
        ..base
    };
    Ok(Project {
        name,
        description: Some(description),
        highlights: Some(highlights),
        keywords: Some(technologies),
        start_date,
        end_date,
        url,
        roles: role.map(|r| vec![r]),
        entity: None,
        kind: Some(kind),
        extension: Some(x),
    })
}

pub(super) fn education(s: School, ids: &mut Ids) -> Result<Education, ImportError> {
    let School {
        degree,
        field,
        school,
        place,
        year,
        logo,
        text,
        on_resume,
        sources,
    } = s;
    let end_date = year.map(|y| date(&school, y.to_string())).transpose()?;
    let sources = source_ids(sources, &school)?;
    let base = ext(ids.derive(&school, "education")?, sources);
    let x = ItemExtension {
        display: display(Some(logo), None, None),
        on_resume: Some(on_resume),
        location: Some(place),
        summary: Some(text),
        ..base
    };
    Ok(Education {
        institution: school,
        url: None,
        area: Some(field),
        study_type: Some(degree),
        start_date: None,
        end_date,
        score: None,
        courses: None,
        extension: Some(x),
    })
}
