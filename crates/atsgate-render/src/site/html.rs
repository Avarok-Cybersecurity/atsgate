//! HTML fragments for a static site: experience, projects, skills and education. Text is
//! escaped; links must be http(s), so a career file cannot smuggle `javascript:` into a page.

use crate::dates::{Dates, PRESENT, RANGE_SEPARATOR, month_year};
use crate::error::RenderError;
use crate::model::{Entry, SkillGroup};

pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// A link target that is safe to put in `href`.
pub fn safe_url(url: &str) -> Result<&str, RenderError> {
    if url.starts_with("https://") || url.starts_with("http://") {
        Ok(url)
    } else {
        Err(RenderError::UnsafeUrl(url.to_owned()))
    }
}

fn time(iso: &str, text: &str) -> String {
    format!(
        r#"<time datetime="{}">{}</time>"#,
        escape(iso),
        escape(text)
    )
}

fn dates(d: &Dates) -> String {
    match d {
        Dates::Range { start, end } => {
            let end = end
                .as_ref()
                .map_or_else(|| PRESENT.to_owned(), |e| time(e.as_str(), &month_year(e)));
            format!(
                "{}{RANGE_SEPARATOR}{end}",
                time(start.as_str(), &month_year(start))
            )
        }
        Dates::Point(p) => time(p.as_str(), &month_year(p)),
    }
}

fn linked(text: &str, url: Option<&str>) -> Result<String, RenderError> {
    Ok(match url {
        Some(u) => format!(r#"<a href="{}">{}</a>"#, escape(safe_url(u)?), escape(text)),
        None => escape(text),
    })
}

/// One `<li>` per entry. `link_title` puts the entry's URL on the title (projects) instead of
/// on the organization (roles, schools).
fn entry(kind: &str, e: &Entry, link_title: bool) -> Result<String, RenderError> {
    let url = e.url.as_deref();
    let title = linked(&e.title, url.filter(|_| link_title))?;
    let mut out = format!(
        r#"<li class="atsgate-entry" id="atsgate-{kind}-{}"><h3 class="atsgate-title">{title}</h3>"#,
        escape(&e.id)
    );
    if let Some(org) = &e.organization {
        let org = linked(org, url.filter(|_| !link_title))?;
        out.push_str(&format!(r#"<p class="atsgate-org">{org}</p>"#));
    }
    if let Some(d) = &e.dates {
        out.push_str(&format!(r#"<p class="atsgate-dates">{}</p>"#, dates(d)));
    }
    if !e.bullets.is_empty() {
        out.push_str("<ul>");
        for b in &e.bullets {
            out.push_str(&format!("<li>{}</li>", escape(b)));
        }
        out.push_str("</ul>");
    }
    out.push_str("</li>");
    Ok(out)
}

pub fn entries(kind: &str, list: &[Entry], link_title: bool) -> Result<String, RenderError> {
    let items: Vec<String> = list
        .iter()
        .map(|e| entry(kind, e, link_title))
        .collect::<Result<_, _>>()?;
    Ok(format!(
        "<ol class=\"atsgate-{kind}\">\n{}\n</ol>\n",
        items.join("\n")
    ))
}

pub fn skills(groups: &[SkillGroup]) -> String {
    let rows: Vec<String> = groups
        .iter()
        .map(|g| {
            format!(
                "<dt>{}</dt><dd>{}</dd>",
                escape(&g.label),
                escape(&g.items.join(", "))
            )
        })
        .collect();
    format!(
        "<dl class=\"atsgate-skills\">\n{}\n</dl>\n",
        rows.join("\n")
    )
}
