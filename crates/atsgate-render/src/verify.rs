//! The round-trip check: a rendered file is extracted and parsed by the same pure crates that
//! score uploaded resumes, and must read back as the document it was rendered from. Every
//! section in order, every entry's title, organization, dates and bullets, the skills and the
//! summary, with no integrity flag, no layout-risk signal, no header or footer text and no
//! heading a parser would not recognize. A render that fails this is never written.

use crate::blocks::PART_SEPARATOR;
use crate::model::{Body, Entry, ResumeDocument};
use atsgate_extract::{Document, Format, Thresholds};
use atsgate_resume::{Resume, Section};

fn compare<T: PartialEq + std::fmt::Debug>(
    problems: &mut Vec<String>,
    what: &str,
    want: T,
    got: T,
) {
    if want != got {
        problems.push(format!("{what}: rendered {want:?}, read back {got:?}"));
    }
}

fn entries(problems: &mut Vec<String>, kind: &str, want: &[Entry], got: &Section) {
    compare(
        problems,
        &format!("{kind} entry count"),
        want.len(),
        got.entries.len(),
    );
    for (w, g) in want.iter().zip(&got.entries) {
        let at = format!("{kind} {}", w.id);
        compare(
            problems,
            &format!("{at} title"),
            Some(w.title.as_str()),
            Some(g.title.as_str()),
        );
        compare(
            problems,
            &format!("{at} organization"),
            w.organization.as_deref(),
            g.organization.as_deref(),
        );
        compare(
            problems,
            &format!("{at} dates"),
            w.dates.as_ref().map(|d| d.parsed()),
            g.dates,
        );
        compare(problems, &format!("{at} bullets"), &w.bullets, &g.bullets);
    }
    compare(
        problems,
        &format!("{kind} loose text"),
        "",
        got.text.as_str(),
    );
}

fn body(problems: &mut Vec<String>, want: &Body, got: &Section) {
    let kind = format!("{:?}", got.kind);
    match want {
        Body::Text(text) => compare(problems, &kind, text.as_str(), got.text.as_str()),
        Body::Entries(list) if got.kind.has_entries() => entries(problems, &kind, list, got),
        Body::Entries(list) => {
            compare(
                problems,
                &format!("{kind} is not empty"),
                list.is_empty(),
                got.items.is_empty(),
            );
        }
        Body::Groups(groups) => {
            let items: Vec<&str> = groups
                .iter()
                .flat_map(|g| g.items.iter().map(String::as_str))
                .collect();
            let read: Vec<&str> = got.items.iter().map(String::as_str).collect();
            compare(problems, &kind, items, read);
        }
    }
}

/// Every way `resume`, read from `doc`, differs from `want`. Empty means the round trip held.
pub fn differences(want: &ResumeDocument, doc: &Document, resume: &Resume) -> Vec<String> {
    let mut problems: Vec<String> = doc
        .signals
        .iter()
        .map(|s| format!("signal {}", s.kind.code()))
        .collect();
    problems.extend(
        resume
            .header_footer
            .iter()
            .map(|l| format!("header/footer text {l:?}")),
    );
    problems.extend(
        resume
            .unrecognized_headings
            .iter()
            .map(|l| format!("unrecognized heading on line {}", l.index)),
    );
    let preamble = [
        want.name.clone(),
        want.title.clone(),
        want.contact.join(PART_SEPARATOR),
    ];
    compare(
        &mut problems,
        "preamble",
        preamble.as_slice(),
        resume.preamble.as_slice(),
    );
    let want_kinds: Vec<_> = want.sections.iter().map(|s| s.kind).collect();
    let got_kinds: Vec<_> = resume.sections.iter().map(|s| s.kind).collect();
    compare(&mut problems, "section order", want_kinds, got_kinds);
    for (w, g) in want.sections.iter().zip(&resume.sections) {
        if w.kind == g.kind {
            body(&mut problems, &w.body, g);
        }
    }
    problems
}

/// Extracts `bytes` as `format` with the standard thresholds and checks it against `want`.
pub fn round_trip(
    want: &ResumeDocument,
    bytes: &[u8],
    format: Format,
) -> Result<(), crate::RenderError> {
    let doc = atsgate_extract::extract(bytes, format, &Thresholds::STANDARD)
        .map_err(|e| crate::RenderError::Extract(e.to_string()))?;
    let problems = differences(want, &doc, &atsgate_resume::parse(&doc));
    if problems.is_empty() {
        Ok(())
    } else {
        Err(crate::RenderError::RoundTrip {
            format: format!("{format:?}"),
            problems,
        })
    }
}
