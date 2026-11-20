//! Vendor profiles: re-mapped weights, strictness, labels and documentation links.

mod common;

use atsgate_extract::Format;
use atsgate_score::{PROFILES, SIMULATED_LABEL, Vendor, citations};

#[test]
fn every_weight_vector_sums_to_one_and_drops_quantification() {
    for p in PROFILES {
        let w = p.source.remap();
        let sum = w.f + w.r + w.p + w.t + w.e + w.d;
        assert!((sum - 1.0).abs() < 1e-12, "{} sums to {sum}", p.name);
        let s = p.source;
        let source_sum = s.formatting
            + s.keyword_match
            + s.section_completeness
            + s.experience_relevance
            + s.education_match
            + s.quantification;
        assert!(
            (source_sum - 1.0).abs() < 1e-12,
            "{} source sums to {source_sum}",
            p.name
        );
    }
}

#[test]
fn strictness_follows_ats_screener() {
    let sigma = |v: Vendor| {
        PROFILES
            .iter()
            .find(|p| p.vendor == v)
            .expect("profile")
            .sigma
    };
    assert_eq!(
        [
            Vendor::Workday,
            Vendor::Taleo,
            Vendor::SuccessFactors,
            Vendor::Icims,
            Vendor::Greenhouse,
            Vendor::Lever
        ]
        .map(sigma),
        [0.9, 0.85, 0.85, 0.6, 0.4, 0.35]
    );
}

#[test]
fn every_profile_links_documentation_listed_in_citations() {
    for p in PROFILES {
        let url =
            citations::url(p.citation).unwrap_or_else(|| panic!("{} cites {}", p.name, p.citation));
        assert!(url.starts_with("https://"), "{url}");
    }
    assert_eq!(citations::url("no-such-entry"), None);
}

#[test]
fn reports_label_every_vendor_result_as_simulated() {
    let r = common::score_bytes(
        Format::Pdf,
        include_bytes!("../../../fixtures/pdf/one-column.pdf"),
        &common::blueprint("software-engineer-backend"),
    );
    assert_eq!(r.label, SIMULATED_LABEL);
    assert_eq!(r.vendors.len(), 6);
    for v in &r.vendors {
        assert_eq!(v.label, "simulated parse and keyword risk");
        let silent = matches!(v.vendor, Vendor::Greenhouse | Vendor::Lever);
        assert_eq!(
            v.note.as_deref() == Some("no documented auto-scoring"),
            silent,
            "{}",
            v.name
        );
        assert!(v.citation.url.starts_with("https://"));
    }
    assert_eq!(r.quantification.label, "heuristic, no outcome evidence");
}
