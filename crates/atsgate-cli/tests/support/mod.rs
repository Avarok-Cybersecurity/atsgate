//! Helpers shared by the `score` and `gate` end-to-end tests.
#![allow(dead_code)]

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub fn root(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel)
}

pub fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

pub fn s(p: &Path) -> String {
    p.to_str().expect("utf-8 path").to_owned()
}

/// Runs the binary with no inherited GitHub variables; `summary` sets GITHUB_STEP_SUMMARY.
pub fn atsgate(args: &[String], summary: Option<&Path>) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_atsgate"));
    cmd.args(args)
        .env_remove("GITHUB_ACTIONS")
        .env_remove("GITHUB_STEP_SUMMARY")
        .env_remove("GITHUB_OUTPUT");
    if let Some(path) = summary {
        cmd.env("GITHUB_STEP_SUMMARY", path);
    }
    cmd.output().expect("run atsgate")
}

/// The scoring options, with floors of zero unless `floors` says otherwise.
pub fn scoring(dir: &Path, blueprint: &str, floors: &str) -> Vec<String> {
    [
        "--blueprint",
        blueprint,
        "--floors",
        floors,
        "--fail-on-integrity-flags",
        "true",
        "--as-of",
        "2026-11",
    ]
    .iter()
    .map(|a| (*a).to_owned())
    .chain([
        "--report".to_owned(),
        s(&dir.join("report.json")),
        "--summary".to_owned(),
        s(&dir.join("summary.md")),
    ])
    .collect()
}

pub const ZERO: &str = "workday=0, taleo=0, icims=0, greenhouse=0, lever=0, successfactors=0";

pub fn report(dir: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(dir.join("report.json")).expect("report")).expect("json")
}

pub fn score(
    resume: &str,
    dir: &Path,
    blueprint: &str,
    floors: &str,
    summary: Option<&Path>,
) -> Output {
    let mut args = vec!["score".to_owned(), "--resume".to_owned(), s(&root(resume))];
    args.extend(scoring(dir, blueprint, floors));
    atsgate(&args, summary)
}
