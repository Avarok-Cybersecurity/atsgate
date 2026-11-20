//! `atsgate score` and `atsgate gate` end to end: real files, real scoring, exit codes, the JSON
//! report, the markdown summary and the GitHub job summary.
mod support;

use support::*;

#[test]
fn score_writes_the_report_summary_and_job_summary() {
    let dir = scratch("score-pass");
    let step = dir.join("step.md");
    std::fs::write(&step, "earlier step\n").expect("step summary");
    let out = score(
        "fixtures/pdf/one-column.pdf",
        &dir,
        "builtin:software-engineer-backend",
        ZERO,
        Some(&step),
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let r = report(&dir);
    let bytes = std::fs::read(root("fixtures/pdf/one-column.pdf")).expect("fixture");
    assert_eq!(r["resume"]["sha256"], atsgate_render::sha256_hex(&bytes));
    assert_eq!(r["verdict"]["passed"], true);
    assert_eq!(r["report"]["label"], "simulated parse and keyword risk");
    assert_eq!(r["report"]["vendors"].as_array().expect("vendors").len(), 6);
    let summary = std::fs::read_to_string(dir.join("summary.md")).expect("summary");
    assert!(summary.contains("| Vendor | Score | Floor |"));
    let step_text = std::fs::read_to_string(&step).expect("step");
    assert_eq!(
        step_text,
        format!("earlier step\n{summary}"),
        "appended, not replaced"
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("passed=true"));
}

#[test]
fn a_failing_gate_exits_1_and_still_writes_its_report() {
    let dir = scratch("score-fail");
    let out = score(
        "fixtures/pdf/image-only.pdf",
        &dir,
        "builtin:software-engineer-backend",
        "",
        None,
    );
    assert_eq!(out.status.code(), Some(1));
    let r = report(&dir);
    assert_eq!(r["verdict"]["passed"], false);
    assert!(
        !r["verdict"]["reasons"]
            .as_array()
            .expect("reasons")
            .is_empty()
    );
}

#[test]
fn an_integrity_flag_fails_the_gate_even_at_zero_floors() {
    let dir = scratch("score-hidden");
    let out = score(
        "fixtures/docx/hidden-white.docx",
        &dir,
        "builtin:software-engineer-backend",
        ZERO,
        None,
    );
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        report(&dir)["verdict"]["reasons"][0],
        "integrity flag: integrity.hidden.white"
    );
}

#[test]
fn an_unknown_vendor_is_an_error_before_anything_is_written() {
    let dir = scratch("score-floors");
    let out = score(
        "fixtures/pdf/one-column.pdf",
        &dir,
        "builtin:accountant",
        "bamboohr=50",
        None,
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("unknown vendor \"bamboohr\""));
    assert!(!dir.join("report.json").exists());
}

#[test]
fn a_generated_blueprint_is_written_beside_the_report_unreviewed() {
    let dir = scratch("score-generate");
    // The posting below its provenance header, as a person would save it.
    let fixture =
        std::fs::read_to_string(root("tests/fixtures/jd/artcraft-hn-2026-11-02.txt")).expect("jd");
    let posting = dir.join("artcraft.txt");
    let body = fixture.split_once("\n---\n").expect("header").1;
    std::fs::write(&posting, body).expect("posting");
    let spec = format!("generate:{}", s(&posting));
    let out = score("fixtures/pdf/one-column.pdf", &dir, &spec, ZERO, None);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(report(&dir)["report"]["blueprint"]["reviewed"], false);
    let yaml = std::fs::read_to_string(dir.join("generated-blueprint.yaml")).expect("generated");
    assert!(yaml.contains("reviewed: false") && yaml.contains("id: artcraft"));
    let summary = std::fs::read_to_string(dir.join("summary.md")).expect("summary");
    assert!(summary.contains("**reviewed: false**"));
}
