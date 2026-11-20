//! Metadata the page does not show is an integrity flag only when it carries content: a
//! blueprint skill or keyword, or more text than a title or author line holds.

mod common;

use atsgate_extract::Format;
use atsgate_score::metadata::METADATA_LONG_TERMS;

const ONE_COLUMN: &[u8] = include_bytes!("../../../fixtures/pdf/one-column.pdf");

/// The one-column fixture with its document Title replaced by `title`.
fn with_title(title: &str) -> Vec<u8> {
    let mut doc = lopdf::Document::load_mem(ONE_COLUMN).expect("fixture loads");
    let info = doc
        .trailer
        .get(b"Info")
        .expect("Info")
        .as_reference()
        .expect("ref");
    let dict = doc
        .get_object_mut(info)
        .expect("info")
        .as_dict_mut()
        .expect("dict");
    dict.set("Title", lopdf::Object::string_literal(title));
    let mut out = Vec::new();
    doc.save_to(&mut out).expect("saves");
    out
}

fn flags(title: &str) -> Vec<String> {
    let b = common::blueprint("software-engineer-backend");
    let r = common::score_bytes(Format::Pdf, &with_title(title), &b);
    r.integrity_flags.into_iter().map(|f| f.rule).collect()
}

#[test]
fn an_ordinary_title_absent_from_the_page_is_not_a_flag() {
    assert!(flags("resume").is_empty());
    assert!(flags("Curriculum Vitae (final draft)").is_empty());
}

#[test]
fn a_blueprint_skill_only_in_metadata_is_a_flag() {
    // AWS is a preferred skill of the backend blueprint and appears nowhere on the page.
    assert_eq!(flags("resume AWS"), ["integrity.metadata_only"]);
}

#[test]
fn long_metadata_text_absent_from_the_page_is_a_flag() {
    let words: Vec<String> = (0..METADATA_LONG_TERMS)
        .map(|i| format!("word{i}x"))
        .collect();
    assert_eq!(flags(&words.join(" ")), ["integrity.metadata_only"]);
    let short = &words[..METADATA_LONG_TERMS - 1];
    assert!(flags(&short.join(" ")).is_empty());
}
