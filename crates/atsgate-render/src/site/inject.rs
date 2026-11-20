//! Fills `<!-- atsgate:NAME -->` ... `<!-- /atsgate:NAME -->` marker pairs in an HTML page with
//! the partial of that name. Every partial needs exactly one well-formed pair, and every marker
//! in the page must name a partial: a missing or stray marker is an error, never skipped.

use crate::error::RenderError;
use std::collections::BTreeMap;

const OPEN_PREFIX: &str = "<!-- atsgate:";
const MARKER_END: &str = " -->";

fn markers(name: &str) -> (String, String) {
    (
        format!("{OPEN_PREFIX}{name}{MARKER_END}"),
        format!("<!-- /atsgate:{name}{MARKER_END}"),
    )
}

/// The names of every opening marker in `html`.
fn marker_names(html: &str) -> Vec<&str> {
    html.match_indices(OPEN_PREFIX)
        .filter_map(|(at, _)| {
            let rest = &html[at + OPEN_PREFIX.len()..];
            rest.find(MARKER_END).map(|end| &rest[..end])
        })
        .collect()
}

pub fn inject(html: &str, partials: &BTreeMap<String, String>) -> Result<String, RenderError> {
    if let Some(stray) = marker_names(html)
        .into_iter()
        .find(|n| !partials.contains_key(*n))
    {
        return Err(RenderError::UnknownMarker(stray.to_owned()));
    }
    let mut out = html.to_owned();
    for (name, fragment) in partials {
        let (open, close) = markers(name);
        let marker_error = || RenderError::Marker(name.clone());
        if out.matches(&open).count() != 1 || out.matches(&close).count() != 1 {
            return Err(marker_error());
        }
        let start = out.find(&open).ok_or_else(marker_error)? + open.len();
        let end = out.find(&close).ok_or_else(marker_error)?;
        if end < start {
            return Err(marker_error());
        }
        out.replace_range(start..end, &format!("\n{fragment}"));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts() -> BTreeMap<String, String> {
        BTreeMap::from([("skills".to_owned(), "<dl></dl>\n".to_owned())])
    }

    #[test]
    fn replaces_between_markers_and_is_idempotent() {
        let page = "<main><!-- atsgate:skills -->old<!-- /atsgate:skills --></main>";
        let once = inject(page, &parts()).expect("injects");
        assert_eq!(
            once,
            "<main><!-- atsgate:skills -->\n<dl></dl>\n<!-- /atsgate:skills --></main>"
        );
        assert_eq!(inject(&once, &parts()).expect("again"), once);
    }

    #[test]
    fn missing_repeated_reversed_or_stray_markers_fail() {
        let pages = [
            "<main></main>",
            "<!-- atsgate:skills --><!-- /atsgate:skills --><!-- atsgate:skills -->",
            "<!-- /atsgate:skills --><!-- atsgate:skills -->",
            "<!-- atsgate:skills -->x",
        ];
        for page in pages {
            assert!(
                matches!(inject(page, &parts()), Err(RenderError::Marker(_))),
                "{page}"
            );
        }
        let stray = "<!-- atsgate:skills --><!-- /atsgate:skills --><!-- atsgate:awards -->";
        assert!(matches!(
            inject(stray, &parts()),
            Err(RenderError::UnknownMarker(_))
        ));
    }
}
