//! HTML, Markdown and plain text: document order, bullets, tables and hidden markup.

use atsgate_extract::{Format, Thresholds, extract};

fn run(format: Format, input: &str) -> (Vec<String>, Vec<&'static str>) {
    let doc = extract(input.as_bytes(), format, &Thresholds::STANDARD).expect("parses");
    (
        doc.lines.iter().map(|l| l.text.clone()).collect(),
        doc.signals.iter().map(|s| s.kind.code()).collect(),
    )
}

#[test]
fn html_blocks_become_lines_and_list_items_bullets() {
    let (lines, signals) = run(
        Format::Html,
        "<h2>SKILLS</h2><ul><li>Rust</li><li>Go</li></ul><p>Done <b>well</b>.</p>",
    );
    assert_eq!(
        lines,
        ["SKILLS", "\u{2022} Rust", "\u{2022} Go", "Done well."]
    );
    assert!(signals.is_empty());
}

#[test]
fn html_hidden_elements_are_flagged() {
    for markup in [
        "<p hidden>AWS</p>",
        "<p style=\"display: none\">AWS</p>",
        "<div style=\"visibility:hidden\"><span>AWS</span></div>",
    ] {
        let (_, signals) = run(Format::Html, &format!("<p>Visible</p>{markup}"));
        assert_eq!(signals, ["integrity.hidden.vanished"], "{markup}");
    }
}

#[test]
fn html_scripts_and_styles_are_not_text() {
    let (lines, _) = run(
        Format::Html,
        "<head><style>p{}</style></head><script>x()</script><p>Text</p>",
    );
    assert_eq!(lines, ["Text"]);
}

#[test]
fn markdown_lists_tables_and_headings() {
    let (lines, signals) = run(
        Format::Markdown,
        "## Skills\n\n- Rust\n- Go\n\n| a | b |\n|---|---|\n| c | d |\n",
    );
    assert_eq!(&lines[..3], ["Skills", "\u{2022} Rust", "\u{2022} Go"]);
    assert!(signals.contains(&"layout.table"));
}

#[test]
fn plain_text_zero_width_is_flagged() {
    let (lines, signals) = run(Format::Text, "Ru\u{200B}st\n\nGo\n");
    assert_eq!(lines.len(), 2);
    assert_eq!(signals, ["integrity.hidden.zero_width"]);
    assert!(
        run(Format::Text, "Rust\nGo\n").1.is_empty(),
        "negative control"
    );
}

#[test]
fn malformed_inputs_fail_instead_of_guessing() {
    assert!(extract(b"not a pdf", Format::Pdf, &Thresholds::STANDARD).is_err());
    assert!(extract(b"not a zip", Format::Docx, &Thresholds::STANDARD).is_err());
    assert!(extract(&[0xFF, 0xFE], Format::Text, &Thresholds::STANDARD).is_err());
}
