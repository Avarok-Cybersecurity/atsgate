//! Every committed fixture raises exactly its expected signals, and the clean ones raise none.

use atsgate_extract::{Document, Format, Region, Thresholds, extract};

macro_rules! fixture {
    ($kind:literal, $name:literal) => {
        include_bytes!(concat!("../../../fixtures/", $kind, "/", $name, ".", $kind)).as_slice()
    };
}

fn codes(doc: &Document) -> Vec<&'static str> {
    let mut c: Vec<&'static str> = doc.signals.iter().map(|s| s.kind.code()).collect();
    c.sort_unstable();
    c.dedup();
    c
}

fn check(format: Format, cases: &[(&str, &[u8], &[&str])]) {
    for (name, bytes, expected) in cases {
        let doc =
            extract(bytes, format, &Thresholds::STANDARD).unwrap_or_else(|e| panic!("{name}: {e}"));
        let mut want = expected.to_vec();
        want.sort_unstable();
        assert_eq!(codes(&doc), want, "{format:?} fixture {name}");
    }
}

#[test]
fn pdf_fixtures_raise_exactly_their_signals() {
    check(
        Format::Pdf,
        &[
            ("one-column", fixture!("pdf", "one-column"), &[]),
            (
                "two-column",
                fixture!("pdf", "two-column"),
                &["layout.multi_column"],
            ),
            ("table", fixture!("pdf", "table"), &["layout.table"]),
            (
                "text-box",
                fixture!("pdf", "text-box"),
                &["layout.text_box", "layout.reading_order_disagreement"],
            ),
            (
                "header-footer",
                fixture!("pdf", "header-footer"),
                &["layout.header_footer_content"],
            ),
            ("page-top-heading", fixture!("pdf", "page-top-heading"), &[]),
            ("browser-print", fixture!("pdf", "browser-print"), &[]),
            (
                "image-only",
                fixture!("pdf", "image-only"),
                &["layout.image_only_page"],
            ),
            (
                "hidden-white",
                fixture!("pdf", "hidden-white"),
                &["integrity.hidden.white"],
            ),
            (
                "hidden-tiny",
                fixture!("pdf", "hidden-tiny"),
                &["integrity.hidden.tiny"],
            ),
            (
                "off-page",
                fixture!("pdf", "off-page"),
                &["integrity.hidden.off_page"],
            ),
            (
                "metadata-only",
                fixture!("pdf", "metadata-only"),
                &["integrity.metadata_only"],
            ),
            (
                "zero-width",
                fixture!("pdf", "zero-width"),
                &["integrity.hidden.zero_width"],
            ),
            (
                "ligature",
                fixture!("pdf", "ligature"),
                &["glyphs.ligature"],
            ),
            (
                "missing-tounicode",
                fixture!("pdf", "missing-tounicode"),
                &["glyphs.missing_tounicode"],
            ),
        ],
    );
}

#[test]
fn docx_fixtures_raise_exactly_their_signals() {
    check(
        Format::Docx,
        &[
            ("one-column", fixture!("docx", "one-column"), &[]),
            (
                "two-column",
                fixture!("docx", "two-column"),
                &["layout.multi_column"],
            ),
            ("table", fixture!("docx", "table"), &["layout.table"]),
            (
                "text-box",
                fixture!("docx", "text-box"),
                &["layout.text_box"],
            ),
            (
                "header-footer",
                fixture!("docx", "header-footer"),
                &["layout.header_footer_content"],
            ),
            (
                "image-only",
                fixture!("docx", "image-only"),
                &["layout.image_only_page"],
            ),
            (
                "hidden-white",
                fixture!("docx", "hidden-white"),
                &["integrity.hidden.white"],
            ),
            (
                "hidden-vanish",
                fixture!("docx", "hidden-vanish"),
                &["integrity.hidden.vanished"],
            ),
            (
                "hidden-tiny",
                fixture!("docx", "hidden-tiny"),
                &["integrity.hidden.tiny"],
            ),
            (
                "zero-width",
                fixture!("docx", "zero-width"),
                &["integrity.hidden.zero_width"],
            ),
            (
                "metadata-only",
                fixture!("docx", "metadata-only"),
                &["integrity.metadata_only"],
            ),
        ],
    );
}

const STUFFED: &str = "Kubernetes Terraform AWS Java Python Spark Kafka";

#[test]
fn hidden_text_evidence_names_the_hidden_words() {
    for (format, bytes) in [
        (Format::Pdf, fixture!("pdf", "hidden-white")),
        (Format::Pdf, fixture!("pdf", "hidden-tiny")),
        (Format::Pdf, fixture!("pdf", "off-page")),
        (Format::Docx, fixture!("docx", "hidden-white")),
        (Format::Docx, fixture!("docx", "hidden-vanish")),
    ] {
        let doc = extract(bytes, format, &Thresholds::STANDARD).expect("fixture parses");
        let signal = doc
            .signals
            .iter()
            .find(|s| s.kind.is_integrity())
            .expect("an integrity flag");
        assert_eq!(
            signal.evidence[0].text, STUFFED,
            "{format:?} {:?}",
            signal.kind
        );
    }
}

#[test]
fn metadata_evidence_lists_only_terms_absent_from_the_page() {
    let doc = extract(
        fixture!("pdf", "metadata-only"),
        Format::Pdf,
        &Thresholds::STANDARD,
    )
    .expect("parses");
    let signal = doc
        .signals
        .iter()
        .find(|s| s.kind.is_integrity())
        .expect("flag");
    assert_eq!(
        signal.evidence[0].text, "aws java python spark kafka",
        "kubernetes and terraform are visible"
    );
}

#[test]
fn thresholds_drive_detection() {
    let lenient = Thresholds {
        tiny_font_pt: 0.5,
        white_min_channel: 1.01,
        ..Thresholds::STANDARD
    };
    for (name, bytes) in [
        ("hidden-tiny", fixture!("pdf", "hidden-tiny")),
        ("hidden-white", fixture!("pdf", "hidden-white")),
    ] {
        let doc = extract(bytes, Format::Pdf, &lenient).expect("parses");
        assert!(
            codes(&doc).is_empty(),
            "{name} must not be flagged above its threshold"
        );
    }
}

#[test]
fn only_band_lines_repeated_across_pages_are_running_headers_and_footers() {
    let region = |bytes: &[u8], text: &str| {
        let doc = extract(bytes, Format::Pdf, &Thresholds::STANDARD).expect("parses");
        let regions: Vec<Region> = doc
            .lines
            .iter()
            .filter(|l| l.text == text)
            .map(|l| l.region)
            .collect();
        assert!(!regions.is_empty(), "{text} is a line");
        regions
    };
    let running = fixture!("pdf", "header-footer");
    assert_eq!(region(running, "Page 1 of 2"), [Region::Footer]);
    assert_eq!(region(running, "Page 2 of 2"), [Region::Footer]);
    assert_eq!(
        region(running, "Jordan Example"),
        [Region::Header, Region::Header]
    );
    // The heading opening page 2 sits in the header band but appears once: page content.
    assert_eq!(
        region(fixture!("pdf", "page-top-heading"), "EDUCATION"),
        [Region::Body]
    );
}
