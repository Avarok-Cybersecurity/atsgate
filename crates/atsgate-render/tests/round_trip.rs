//! The rendered PDF and DOCX read back, through atsgate-extract and atsgate-resume, as exactly
//! the document they were rendered from: on a synthetic career and on the imported tbraun96 one.

use atsgate_career::{Career, PrivateContact, Tailoring};
use atsgate_extract::Format;
use atsgate_render::{
    ContactProfile, DOCX_FILE, PDF_FILE, RenderError, blocks, build, document, render_docx,
    render_pdf, round_trip,
};

fn fixture(path: &str) -> String {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/");
    std::fs::read_to_string(format!("{root}{path}")).expect("fixture")
}

fn inputs(career: &str, tailoring: &str) -> (Career, Tailoring) {
    let career = Career::from_json(&fixture(career)).expect("career");
    let tailoring = Tailoring::from_yaml(&fixture(tailoring)).expect("tailoring");
    (career, tailoring)
}

fn synthetic() -> (Career, Tailoring) {
    inputs("career/synthetic.career.json", "tailoring/synthetic.yaml")
}

fn tbraun96() -> (Career, Tailoring) {
    inputs(
        "import/tbraun96.career.json",
        "tailoring/tbraun96-general.yaml",
    )
}

fn private() -> ContactProfile {
    let text = fixture("contact/synthetic-private.json");
    ContactProfile::Private(PrivateContact::from_json(&text).expect("contact"))
}

#[test]
fn synthetic_round_trips_with_both_profiles() {
    let (career, tailoring) = synthetic();
    for profile in [ContactProfile::Public, private()] {
        let out = build(&career, &tailoring, &profile).expect("build");
        assert!(out.files[PDF_FILE].starts_with(b"%PDF-"));
        assert!(out.files[DOCX_FILE].starts_with(b"PK"));
    }
}

#[test]
fn tbraun96_general_round_trips() {
    let (career, tailoring) = tbraun96();
    build(&career, &tailoring, &ContactProfile::Public).expect("build");
}

#[test]
fn builds_are_byte_for_byte_reproducible() {
    let (career, tailoring) = synthetic();
    let first = build(&career, &tailoring, &ContactProfile::Public).expect("first");
    let second = build(&career, &tailoring, &ContactProfile::Public).expect("second");
    assert_eq!(first, second);
}

#[test]
fn private_details_appear_only_in_the_private_profile() {
    let (career, tailoring) = synthetic();
    let tailored = tailoring.check(&career).expect("tailored");
    let public = document(&tailored, &ContactProfile::Public).expect("public");
    let private = document(&tailored, &private()).expect("private");
    let joined = |d: &atsgate_render::ResumeDocument| d.contact.join(" ");
    assert!(!joined(&public).contains("555"));
    assert!(!joined(&public).contains("Example Street"));
    assert!(joined(&private).contains("+1 555 0100"));
    assert!(joined(&private).contains("1 Example Street"));
}

/// The check is not vacuous: a document that differs from the rendered one fails it, in both
/// formats, naming what differs.
#[test]
fn a_mismatched_document_fails_the_round_trip() {
    let (career, tailoring) = synthetic();
    let tailored = tailoring.check(&career).expect("tailored");
    let doc = document(&tailored, &ContactProfile::Public).expect("doc");
    let layout = blocks(&doc);
    let pdf = render_pdf(&layout, &doc.name, doc.max_pages).expect("pdf");
    let docx = render_docx(&layout).expect("docx");
    let mut other = doc.clone();
    other.sections.swap(1, 2);
    other.title.push_str(" II");
    for (bytes, format) in [(&pdf, Format::Pdf), (&docx, Format::Docx)] {
        round_trip(&doc, bytes, format).expect("the rendered document reads back");
        match round_trip(&other, bytes, format) {
            Err(RenderError::RoundTrip { problems, .. }) => {
                assert!(
                    problems.iter().any(|p| p.starts_with("preamble")),
                    "{problems:?}"
                );
                assert!(
                    problems.iter().any(|p| p.starts_with("section order")),
                    "{problems:?}"
                );
            }
            other => panic!("expected a round-trip failure, got {other:?}"),
        }
    }
}

#[test]
fn a_page_limit_the_resume_exceeds_is_an_error() {
    let (career, tailoring) = tbraun96();
    let tailored = tailoring.check(&career).expect("tailored");
    let doc = document(&tailored, &ContactProfile::Public).expect("doc");
    let result = render_pdf(&blocks(&doc), &doc.name, 1);
    assert!(matches!(
        result,
        Err(RenderError::TooManyPages { max_pages: 1, .. })
    ));
}

#[test]
fn a_phone_in_the_career_file_is_refused() {
    let (mut career, tailoring) = synthetic();
    career.basics.phone = Some("+1 555 0199".to_owned());
    let result = build(&career, &tailoring, &ContactProfile::Public);
    assert!(matches!(
        result,
        Err(RenderError::PrivateFieldInCareer("basics.phone"))
    ));
}
