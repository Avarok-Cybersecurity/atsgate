//! The same resume, laid out six ways in two formats, parses to the same sections.

use atsgate_extract::{Format, Thresholds, extract};
use atsgate_resume::{DateEnd, DateRange, PartialDate, Resume, SectionKind, parse};

macro_rules! fixture {
    ($kind:literal, $name:literal) => {
        (
            $name,
            include_bytes!(concat!("../../../fixtures/", $kind, "/", $name, ".", $kind)).as_slice(),
        )
    };
}

/// Title, organization, dates and bullets of one role.
type Role = (String, Option<String>, Option<DateRange>, Vec<String>);

/// What must not change with layout: section content, without evidence positions.
#[derive(Debug, PartialEq)]
struct Content {
    summary: String,
    skills: Vec<String>,
    roles: Vec<Role>,
    education: Vec<(String, Option<String>, Option<DateRange>)>,
}

fn content(r: &Resume) -> Content {
    let section = |k| {
        r.section(k)
            .unwrap_or_else(|| panic!("missing {k:?} in {r:#?}"))
    };
    Content {
        summary: section(SectionKind::Summary).text.clone(),
        skills: section(SectionKind::Skills).items.clone(),
        roles: section(SectionKind::Experience)
            .entries
            .iter()
            .map(|e| {
                (
                    e.title.clone(),
                    e.organization.clone(),
                    e.dates,
                    e.bullets.clone(),
                )
            })
            .collect(),
        education: section(SectionKind::Education)
            .entries
            .iter()
            .map(|e| (e.title.clone(), e.organization.clone(), e.dates))
            .collect(),
    }
}

fn parsed(format: Format, bytes: &[u8]) -> Resume {
    parse(&extract(bytes, format, &Thresholds::STANDARD).expect("fixture parses"))
}

fn ym(year: u16, month: Option<u8>) -> PartialDate {
    PartialDate { year, month }
}

fn expected() -> Content {
    let s = |v: &str| v.to_owned();
    Content {
        summary: s(
            "Backend engineer with eight years of experience building data pipelines and APIs for finance.",
        ),
        skills: ["Rust", "Go", "PostgreSQL", "Kubernetes", "Terraform"]
            .map(s)
            .to_vec(),
        roles: vec![
            (
                s("Senior Software Engineer"),
                Some(s("Northwind Analytics")),
                Some(DateRange {
                    start: ym(2020, Some(1)),
                    end: DateEnd::Present,
                }),
                vec![
                    s("Moved batch pipelines to Rust, cutting cost 30%."),
                    s("Designed a PostgreSQL schema for 2M requests/day."),
                ],
            ),
            (
                s("Software Engineer"),
                Some(s("Contoso Labs")),
                Some(DateRange {
                    start: ym(2016, Some(6)),
                    end: DateEnd::Date(ym(2019, Some(12))),
                }),
                vec![
                    s("Built REST APIs in Go for internal billing."),
                    s("Mentored four junior engineers."),
                ],
            ),
        ],
        education: vec![(
            s("B.S. Computer Science"),
            Some(s("State University")),
            Some(DateRange {
                start: ym(2012, None),
                end: DateEnd::Date(ym(2016, None)),
            }),
        )],
    }
}

#[test]
fn every_text_layout_parses_to_the_same_sections() {
    let pdfs = [
        fixture!("pdf", "one-column"),
        fixture!("pdf", "two-column"),
        fixture!("pdf", "table"),
        fixture!("pdf", "text-box"),
        fixture!("pdf", "header-footer"),
        fixture!("pdf", "page-top-heading"),
        fixture!("pdf", "browser-print"),
    ];
    let docxs = [
        fixture!("docx", "one-column"),
        fixture!("docx", "two-column"),
        fixture!("docx", "table"),
        fixture!("docx", "text-box"),
        fixture!("docx", "header-footer"),
    ];
    let cases = pdfs
        .iter()
        .map(|c| (Format::Pdf, c))
        .chain(docxs.iter().map(|c| (Format::Docx, c)));
    for (format, (name, bytes)) in cases {
        assert_eq!(
            content(&parsed(format, bytes)),
            expected(),
            "{format:?} {name}"
        );
    }
}

#[test]
fn header_and_footer_lines_stay_out_of_sections() {
    let r = parsed(
        Format::Pdf,
        include_bytes!("../../../fixtures/pdf/header-footer.pdf"),
    );
    assert_eq!(
        r.header_footer.last().map(String::as_str),
        Some("Page 2 of 2")
    );
    assert!(
        r.preamble.is_empty(),
        "name and contact sit in the header band"
    );
}

#[test]
fn an_image_only_resume_has_no_sections() {
    let r = parsed(
        Format::Pdf,
        include_bytes!("../../../fixtures/pdf/image-only.pdf"),
    );
    assert!(r.sections.is_empty() && r.preamble.is_empty());
}

#[test]
fn ligatures_and_zero_width_characters_do_not_change_content() {
    let clean = content(&parsed(
        Format::Pdf,
        include_bytes!("../../../fixtures/pdf/one-column.pdf"),
    ));
    for bytes in [
        include_bytes!("../../../fixtures/pdf/ligature.pdf").as_slice(),
        include_bytes!("../../../fixtures/pdf/zero-width.pdf"),
    ] {
        assert_eq!(content(&parsed(Format::Pdf, bytes)), clean);
    }
}

#[test]
fn every_entry_points_back_to_its_lines() {
    let pdf = include_bytes!("../../../fixtures/pdf/one-column.pdf");
    let doc = extract(pdf, Format::Pdf, &Thresholds::STANDARD).expect("parses");
    let r = parse(&doc);
    let role = &r
        .section(SectionKind::Experience)
        .expect("experience")
        .entries[0];
    let first = &role.evidence[0];
    assert_eq!(
        doc.lines[first.index].text,
        "Senior Software Engineer, Northwind Analytics Jan 2020 \u{2013} Present"
    );
    assert_eq!(first.page, 1);
    assert!(first.bbox.is_some());
}
