//! The resume laid out as each DOCX fixture variant.

use super::content::{self as c, Entry};
use super::docx_package::DocxSpec;
use super::docx_xml::{Block, Run};

fn para(text: &str) -> Block {
    Block::Para(vec![Run::plain(text)])
}

fn heading(text: &str) -> Block {
    Block::Para(vec![Run::bold(text)])
}

fn bullets(e: &Entry) -> impl Iterator<Item = Block> + '_ {
    e.bullets.iter().map(|b| para(&format!("{}{b}", c::BULLET)))
}

/// Title and dates in one paragraph, separated by a tab.
fn entry_inline(e: &Entry) -> Vec<Block> {
    let dates = Run {
        tab_before: true,
        ..Run::plain(e.dates)
    };
    let mut out = vec![Block::Para(vec![Run::plain(e.title), dates])];
    out.extend(bullets(e));
    out
}

fn entry_stacked(e: &Entry) -> Vec<Block> {
    let mut out = vec![para(e.title), para(e.dates)];
    out.extend(bullets(e));
    out
}

fn identity() -> Vec<Block> {
    vec![heading(c::NAME), para(&c::CONTACT.join(" | "))]
}

fn summary() -> Vec<Block> {
    let mut out = vec![heading(c::SUMMARY_HEADING)];
    out.extend(c::SUMMARY.iter().map(|l| para(l)));
    out
}

fn rest() -> Vec<Block> {
    let mut out = vec![heading(c::EXPERIENCE_HEADING)];
    c::ROLES.iter().for_each(|e| out.extend(entry_inline(e)));
    out.push(heading(c::EDUCATION_HEADING));
    c::EDUCATION
        .iter()
        .for_each(|e| out.extend(entry_inline(e)));
    out.push(heading(c::SKILLS_HEADING));
    out.extend(c::SKILLS.iter().map(|l| para(l)));
    out
}

fn spec(body: Vec<Block>) -> DocxSpec {
    DocxSpec {
        title: c::NAME.to_owned(),
        keywords: None,
        body,
        header: None,
        footer: None,
        columns: 1,
    }
}

pub fn one_column() -> DocxSpec {
    let mut body = identity();
    body.extend(summary());
    body.extend(rest());
    spec(body)
}

/// Section columns: identity, skills and education, then a column break into summary and experience.
pub fn two_column() -> DocxSpec {
    let mut body = identity();
    body.push(heading(c::SKILLS_HEADING));
    body.extend(c::SKILLS.iter().map(|l| para(l)));
    body.push(heading(c::EDUCATION_HEADING));
    c::EDUCATION
        .iter()
        .for_each(|e| body.extend(entry_stacked(e)));
    body.push(Block::Para(vec![Run {
        column_break_before: true,
        ..Run::bold(c::SUMMARY_HEADING)
    }]));
    body.extend(c::SUMMARY.iter().map(|l| para(l)));
    body.push(heading(c::EXPERIENCE_HEADING));
    c::ROLES.iter().for_each(|e| body.extend(entry_stacked(e)));
    DocxSpec {
        columns: 2,
        ..spec(body)
    }
}

pub fn table() -> DocxSpec {
    let row = |h: &str, cell: Vec<Block>| vec![vec![heading(h)], cell];
    let rows = vec![
        row(
            c::SUMMARY_HEADING,
            c::SUMMARY.iter().map(|l| para(l)).collect(),
        ),
        row(
            c::EXPERIENCE_HEADING,
            c::ROLES.iter().flat_map(entry_stacked).collect(),
        ),
        row(
            c::EDUCATION_HEADING,
            c::EDUCATION.iter().flat_map(entry_stacked).collect(),
        ),
        row(
            c::SKILLS_HEADING,
            c::SKILLS.iter().map(|l| para(l)).collect(),
        ),
    ];
    let mut body = identity();
    body.push(Block::Table(rows));
    spec(body)
}

pub fn text_box() -> DocxSpec {
    let mut body = identity();
    body.push(Block::TextBox(summary()));
    body.extend(rest());
    spec(body)
}

pub fn header_footer() -> DocxSpec {
    let mut body = summary();
    body.extend(rest());
    DocxSpec {
        header: Some(identity()),
        footer: Some(vec![para("Page 1 of 1")]),
        ..spec(body)
    }
}

pub fn image_only() -> DocxSpec {
    spec(vec![Block::Image])
}

fn with_run(run: Run) -> DocxSpec {
    let mut s = one_column();
    s.body.push(Block::Para(vec![run]));
    s
}

pub fn hidden_white() -> DocxSpec {
    with_run(Run {
        color: Some("FFFFFF"),
        ..Run::plain(c::STUFFED_KEYWORDS)
    })
}

pub fn hidden_vanish() -> DocxSpec {
    with_run(Run {
        vanish: true,
        ..Run::plain(c::STUFFED_KEYWORDS)
    })
}

pub fn hidden_tiny() -> DocxSpec {
    with_run(Run {
        half_points: Some(2),
        ..Run::plain(c::STUFFED_KEYWORDS)
    })
}

pub fn zero_width() -> DocxSpec {
    let mut s = one_column();
    for b in &mut s.body {
        if let Block::Para(runs) = b
            && runs[0].text == c::SKILLS[0]
        {
            runs[0].text = "Ru\u{200B}st, Go, Postgre\u{200B}SQL".to_owned();
        }
    }
    s
}

pub fn metadata_only() -> DocxSpec {
    DocxSpec {
        keywords: Some(c::STUFFED_KEYWORDS.to_owned()),
        ..one_column()
    }
}
