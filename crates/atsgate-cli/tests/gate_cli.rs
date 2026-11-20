//! `atsgate gate` end to end, and the step outputs the action reads from GITHUB_OUTPUT.

mod support;

use serde_json::Value;
use std::process::Command;
use support::*;

#[test]
fn gate_scores_exactly_the_pdf_it_ships() {
    let dir = scratch("gate");
    let out_dir = dir.join("dist");
    let mut args: Vec<String> = [
        "gate",
        "--career",
        &s(&root("fixtures/career/synthetic.career.json")),
        "--tailoring",
        &s(&root("fixtures/tailoring/synthetic.yaml")),
        "--contact",
        "public",
        "--out",
        &s(&out_dir),
    ]
    .iter()
    .map(|a| (*a).to_owned())
    .collect();
    args.extend(scoring(&dir, "builtin:software-engineer-backend", ZERO));
    let out = atsgate(&args, None);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(out_dir.join("manifest.json")).expect("manifest"))
            .expect("json");
    let r = report(&dir);
    assert_eq!(r["resume"]["sha256"], manifest["files"]["resume.pdf"]);
    assert_eq!(r["resume"]["path"], s(&out_dir.join("resume.pdf")));
    let shipped = std::fs::read(out_dir.join("resume.pdf")).expect("pdf");
    assert_eq!(r["resume"]["sha256"], atsgate_render::sha256_hex(&shipped));
}

#[test]
fn action_outputs_are_appended_to_github_output() {
    let dir = scratch("outputs");
    let output = dir.join("github-output");
    let mut args = vec![
        "score".to_owned(),
        "--resume".to_owned(),
        s(&root("fixtures/pdf/one-column.pdf")),
    ];
    args.extend(scoring(&dir, "builtin:software-engineer-backend", ZERO));
    let status = Command::new(env!("CARGO_BIN_EXE_atsgate"))
        .args(&args)
        .env_remove("GITHUB_ACTIONS")
        .env_remove("GITHUB_STEP_SUMMARY")
        .env("GITHUB_OUTPUT", &output)
        .status()
        .expect("run");
    assert!(status.success());
    let text = std::fs::read_to_string(&output).expect("outputs");
    let names: Vec<&str> = text
        .lines()
        .map(|l| l.split_once('=').expect("name=value").0)
        .collect();
    assert_eq!(
        names,
        [
            "score-json",
            "min-vendor-score",
            "passed",
            "report-json",
            "resume-sha256"
        ]
    );
    let score_json = text
        .lines()
        .next()
        .expect("first")
        .split_once('=')
        .expect("pair")
        .1;
    let verdict: Value = serde_json::from_str(score_json).expect("compact json");
    assert_eq!(verdict["passed"], true);
    assert!(text.contains("passed=true\n"));
}
