//! Hidden and metadata-only text is flagged with evidence, never scored up, and fails the gate
//! when `fail_on_integrity_flags` is set.

mod common;

use atsgate_extract::Format;
use atsgate_score::{Floors, gate};

macro_rules! fixture {
    ($kind:literal, $name:literal) => {
        include_bytes!(concat!("../../../fixtures/", $kind, "/", $name, ".", $kind)).as_slice()
    };
}

fn flagged() -> Vec<(Format, &'static str, &'static [u8], &'static str)> {
    vec![
        (
            Format::Pdf,
            "hidden-white",
            fixture!("pdf", "hidden-white"),
            "integrity.hidden.white",
        ),
        (
            Format::Pdf,
            "hidden-tiny",
            fixture!("pdf", "hidden-tiny"),
            "integrity.hidden.tiny",
        ),
        (
            Format::Pdf,
            "off-page",
            fixture!("pdf", "off-page"),
            "integrity.hidden.off_page",
        ),
        (
            Format::Pdf,
            "zero-width",
            fixture!("pdf", "zero-width"),
            "integrity.hidden.zero_width",
        ),
        (
            Format::Pdf,
            "metadata-only",
            fixture!("pdf", "metadata-only"),
            "integrity.metadata_only",
        ),
        (
            Format::Docx,
            "hidden-white",
            fixture!("docx", "hidden-white"),
            "integrity.hidden.white",
        ),
        (
            Format::Docx,
            "hidden-tiny",
            fixture!("docx", "hidden-tiny"),
            "integrity.hidden.tiny",
        ),
        (
            Format::Docx,
            "hidden-vanish",
            fixture!("docx", "hidden-vanish"),
            "integrity.hidden.vanished",
        ),
        (
            Format::Docx,
            "zero-width",
            fixture!("docx", "zero-width"),
            "integrity.hidden.zero_width",
        ),
        (
            Format::Docx,
            "metadata-only",
            fixture!("docx", "metadata-only"),
            "integrity.metadata_only",
        ),
    ]
}

/// Floors of zero, so only knockouts and integrity flags can fail the gate.
fn zero_floors() -> Floors {
    Floors::parse("workday=0, taleo=0, icims=0, greenhouse=0, lever=0, successfactors=0")
        .expect("valid floors")
}

#[test]
fn every_integrity_fixture_is_flagged_with_evidence() {
    let blueprint = common::blueprint("software-engineer-backend");
    for (format, name, bytes, rule) in flagged() {
        let r = common::score_bytes(format, bytes, &blueprint);
        let rules: Vec<&str> = r.integrity_flags.iter().map(|f| f.rule.as_str()).collect();
        assert_eq!(rules, [rule], "{format:?} {name}");
        let flag = &r.integrity_flags[0];
        assert!(!flag.evidence.is_empty(), "{format:?} {name}");
        assert!(
            flag.evidence
                .iter()
                .all(|e| e.rule == rule && !e.text.is_empty())
        );
    }
}

#[test]
fn integrity_flags_fail_the_gate_only_when_asked() {
    let blueprint = common::blueprint("software-engineer-backend");
    for (format, name, bytes, rule) in flagged() {
        let r = common::score_bytes(format, bytes, &blueprint);
        let strict = gate(&r, &zero_floors(), true);
        assert!(!strict.passed, "{format:?} {name}");
        assert!(strict.reasons.contains(&format!("integrity flag: {rule}")));
        let lenient = gate(&r, &zero_floors(), false);
        assert!(lenient.passed, "{format:?} {name}: {:?}", lenient.reasons);
    }
}

#[test]
fn a_clean_resume_passes_the_integrity_gate() {
    let blueprint = common::blueprint("software-engineer-backend");
    let r = common::score_bytes(Format::Pdf, fixture!("pdf", "one-column"), &blueprint);
    assert!(gate(&r, &zero_floors(), true).passed);
}

#[test]
fn hidden_keywords_earn_nothing() {
    // The stuffed fixtures hide AWS, Java, Python and Kafka, all preferred skills of this
    // blueprint; with hidden text removed before scoring they score exactly like the clean one.
    let blueprint = common::blueprint("software-engineer-backend");
    let clean = common::score_bytes(Format::Pdf, fixture!("pdf", "one-column"), &blueprint);
    for (format, name, bytes, _) in flagged() {
        let r = common::score_bytes(format, bytes, &blueprint);
        assert_eq!(
            common::numbers(&r)[..6],
            common::numbers(&clean)[..6],
            "{format:?} {name}"
        );
        assert!(
            r.skills
                .iter()
                .all(|s| !["AWS", "Java", "Python", "Apache Kafka"]
                    .contains(&s.canonical.as_str())
                    || s.matched.is_none()),
            "{format:?} {name}"
        );
    }
}
