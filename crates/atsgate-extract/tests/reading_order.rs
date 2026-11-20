//! Reading order: columns are read whole, cells row by row, and boxed text where it sits.

use atsgate_extract::{Format, Region, Thresholds, extract};

fn lines(bytes: &[u8], format: Format) -> Vec<String> {
    let doc = extract(bytes, format, &Thresholds::STANDARD).expect("fixture parses");
    doc.lines.iter().map(|l| l.text.clone()).collect()
}

fn position(lines: &[String], text: &str) -> usize {
    lines
        .iter()
        .position(|l| l == text)
        .unwrap_or_else(|| panic!("no line {text:?} in {lines:#?}"))
}

#[test]
fn two_columns_are_read_one_after_the_other() {
    let l = lines(
        include_bytes!("../../../fixtures/pdf/two-column.pdf"),
        Format::Pdf,
    );
    let left_end = position(&l, "2012 \u{2013} 2016");
    let right_start = position(&l, "SUMMARY");
    assert!(
        left_end < right_start,
        "left column finishes before the right begins"
    );
    assert_eq!(
        l[right_start + 1],
        "Backend engineer with eight years of experience"
    );
}

#[test]
fn right_aligned_dates_stay_on_their_title_row() {
    let l = lines(
        include_bytes!("../../../fixtures/pdf/one-column.pdf"),
        Format::Pdf,
    );
    position(
        &l,
        "Senior Software Engineer, Northwind Analytics Jan 2020 \u{2013} Present",
    );
}

#[test]
fn a_late_drawn_text_box_is_read_where_it_sits() {
    let pdf = include_bytes!("../../../fixtures/pdf/text-box.pdf");
    let doc = extract(pdf, Format::Pdf, &Thresholds::STANDARD).expect("parses");
    let l: Vec<&str> = doc.lines.iter().map(|l| l.text.as_str()).collect();
    assert_eq!(l[2], "SUMMARY");
    assert_eq!(doc.lines[2].region, Region::TextBox);
    assert!(
        doc.stream_text()
            .ends_with("building data pipelines and APIs for finance."),
        "stored last"
    );
}

#[test]
fn table_cells_are_read_row_by_row() {
    let l = lines(
        include_bytes!("../../../fixtures/pdf/table.pdf"),
        Format::Pdf,
    );
    assert_eq!(
        l[position(&l, "SUMMARY") + 1],
        "Backend engineer with eight years of experience"
    );
    assert_eq!(l[position(&l, "SKILLS") + 1], "Rust, Go, PostgreSQL");
}
