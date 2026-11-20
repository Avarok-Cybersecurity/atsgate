//! The same resume in five layouts and two formats: R, P, T, E and D are identical, and F is
//! charged for exactly the layout risk each one carries.

mod common;

use atsgate_extract::Format;
use atsgate_score::Report;

macro_rules! fixture {
    ($kind:literal, $name:literal) => {
        include_bytes!(concat!("../../../fixtures/", $kind, "/", $name, ".", $kind)).as_slice()
    };
}

/// (format, layout, bytes, the parse penalties it must carry).
fn cases() -> Vec<(Format, &'static str, &'static [u8], Vec<&'static str>)> {
    vec![
        (
            Format::Pdf,
            "one-column",
            fixture!("pdf", "one-column"),
            vec![],
        ),
        (
            Format::Pdf,
            "two-column",
            fixture!("pdf", "two-column"),
            vec!["layout.multi_column"],
        ),
        (
            Format::Pdf,
            "table",
            fixture!("pdf", "table"),
            vec!["layout.table"],
        ),
        (
            Format::Pdf,
            "text-box",
            fixture!("pdf", "text-box"),
            vec!["layout.reading_order_disagreement", "layout.text_box"],
        ),
        (
            Format::Pdf,
            "header-footer",
            fixture!("pdf", "header-footer"),
            vec!["layout.header_footer_content"],
        ),
        (
            Format::Docx,
            "one-column",
            fixture!("docx", "one-column"),
            vec![],
        ),
        (
            Format::Docx,
            "two-column",
            fixture!("docx", "two-column"),
            vec!["layout.multi_column"],
        ),
        (
            Format::Docx,
            "table",
            fixture!("docx", "table"),
            vec!["layout.table"],
        ),
        (
            Format::Docx,
            "text-box",
            fixture!("docx", "text-box"),
            vec!["layout.text_box"],
        ),
        (
            Format::Docx,
            "header-footer",
            fixture!("docx", "header-footer"),
            vec!["layout.header_footer_content"],
        ),
    ]
}

fn content_scores(r: &Report) -> [Option<u64>; 6] {
    let d = &r.dimensions;
    [
        d.required_skills.score,
        d.preferred_skills.score,
        d.title.score,
        d.experience.score,
        d.education.score,
        Some(r.relevant_years),
    ]
    .map(|x| x.map(f64::to_bits))
}

fn rules(r: &Report) -> Vec<&str> {
    let mut v: Vec<&str> = r.parse_penalties.iter().map(|p| p.rule.as_str()).collect();
    v.sort_unstable();
    v
}

#[test]
fn content_dimensions_do_not_depend_on_layout() {
    for id in [
        "software-engineer-backend",
        "software-engineer-systems-rust",
        "accountant",
    ] {
        let blueprint = common::blueprint(id);
        let reference = common::score_bytes(Format::Pdf, fixture!("pdf", "one-column"), &blueprint);
        for (format, name, bytes, _) in cases() {
            let r = common::score_bytes(format, bytes, &blueprint);
            assert_eq!(
                content_scores(&r),
                content_scores(&reference),
                "{id}: {format:?} {name}"
            );
            let credits = |r: &Report| {
                r.skills
                    .iter()
                    .map(|s| s.credit.to_bits())
                    .collect::<Vec<_>>()
            };
            assert_eq!(credits(&r), credits(&reference), "{id}: {format:?} {name}");
        }
    }
}

#[test]
fn parse_integrity_charges_exactly_the_risky_layouts() {
    let blueprint = common::blueprint("software-engineer-backend");
    for (format, name, bytes, expected) in cases() {
        let r = common::score_bytes(format, bytes, &blueprint);
        assert_eq!(rules(&r), expected, "{format:?} {name}");
        let clean = expected.is_empty();
        for v in &r.vendors {
            assert_eq!(
                v.parse_score == 100.0,
                clean,
                "{format:?} {name} {}",
                v.name
            );
        }
        assert!(r.integrity_flags.is_empty(), "{format:?} {name}");
    }
}

#[test]
fn a_stricter_parser_charges_more_for_the_same_layout() {
    let blueprint = common::blueprint("software-engineer-backend");
    let r = common::score_bytes(Format::Pdf, fixture!("pdf", "two-column"), &blueprint);
    let f = |name: &str| {
        r.vendors
            .iter()
            .find(|v| v.name == name)
            .expect(name)
            .parse_score
    };
    // 100 - 15 points x sigma.
    assert_eq!(f("Workday"), 86.5);
    assert_eq!(f("Lever"), 94.8);
    assert!(f("Workday") < f("iCIMS") && f("iCIMS") < f("Greenhouse"));
}

#[test]
fn an_image_only_resume_loses_its_parse_score_and_its_content() {
    let blueprint = common::blueprint("software-engineer-backend");
    let r = common::score_bytes(Format::Pdf, fixture!("pdf", "image-only"), &blueprint);
    assert_eq!(rules(&r), ["layout.image_only_page"]);
    for v in &r.vendors {
        // 100 points at the vendor's strictness: nothing is left to parse.
        assert_eq!(
            v.parse_score,
            ((100.0 - 100.0 * v.sigma) * 10.0_f64).round() / 10.0
        );
    }
    assert_eq!(r.dimensions.required_skills.score, Some(0.0));
}
