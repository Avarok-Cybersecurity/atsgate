//! The `atsgate` binary end to end: import, build, inject, and the private-contact refusal in CI.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(rel)
}

/// A fresh directory under the target dir, unique to this test.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

/// Runs the binary with `GITHUB_ACTIONS` set as given, never inherited from the test runner.
fn atsgate<A: AsRef<OsStr>>(args: &[A], github_actions: Option<&str>) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_atsgate"));
    cmd.args(args).env_remove("GITHUB_ACTIONS");
    if let Some(value) = github_actions {
        cmd.env("GITHUB_ACTIONS", value);
    }
    cmd.output().expect("run atsgate")
}

fn s(p: &Path) -> &str {
    p.to_str().expect("utf-8 path")
}

fn build_args(contact: &str, out: &Path) -> Vec<String> {
    let career = fixture("career/synthetic.career.json");
    let tailoring = fixture("tailoring/synthetic.yaml");
    let args = [
        "build",
        "--career",
        s(&career),
        "--tailoring",
        s(&tailoring),
    ];
    let rest = ["--contact", contact, "--out", s(out)];
    args.iter().chain(&rest).map(|a| (*a).to_owned()).collect()
}

#[test]
fn import_writes_the_golden_career_file() {
    let dir = scratch("import");
    let out = dir.join("career.json");
    let input = fixture("import/tbraun96-career.json");
    let result = atsgate(
        &[
            "import",
            "--from",
            "tbraun96",
            "--input",
            s(&input),
            "--out",
            s(&out),
        ],
        None,
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let golden = std::fs::read(fixture("import/tbraun96.career.json")).expect("golden");
    assert_eq!(std::fs::read(&out).expect("written"), golden);
}

#[test]
fn build_writes_every_output_and_a_manifest_of_their_hashes() {
    let out = scratch("build");
    let result = atsgate(&build_args("public", &out), Some("true"));
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(out.join("manifest.json")).expect("manifest"))
            .expect("json");
    let files = manifest["files"].as_object().expect("files");
    let names: Vec<&str> = files.keys().map(String::as_str).collect();
    let expected = [
        "partials/education.html",
        "partials/experience.html",
        "partials/jsonld.json",
        "partials/projects.html",
        "partials/skills.html",
        "resume.docx",
        "resume.pdf",
    ];
    assert_eq!(names, expected);
    for (name, hash) in files {
        let bytes = std::fs::read(out.join(name)).expect("output");
        assert_eq!(
            hash.as_str(),
            Some(atsgate_render::sha256_hex(&bytes).as_str()),
            "{name}"
        );
    }
}

#[test]
fn private_contact_is_refused_in_ci_unless_explicitly_allowed() {
    let contact = fixture("contact/synthetic-private.json");
    let out = scratch("private");
    let refused = atsgate(&build_args(s(&contact), &out), Some("true"));
    assert_eq!(refused.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&refused.stderr).contains("--allow-private-contact"));
    assert!(
        !out.join("resume.pdf").exists(),
        "nothing is written when refused"
    );

    let mut allowed = build_args(s(&contact), &out);
    allowed.push("--allow-private-contact".to_owned());
    assert!(atsgate(&allowed, Some("true")).status.success());
    assert!(
        atsgate(&build_args(s(&contact), &scratch("local")), None)
            .status
            .success()
    );
    assert!(
        atsgate(&build_args(s(&contact), &scratch("other")), Some("false"))
            .status
            .success()
    );
}

#[test]
fn inject_fills_a_page_from_built_partials() {
    let out = scratch("inject");
    assert!(atsgate(&build_args("public", &out), None).status.success());
    let page = out.join("page.html");
    let markers: String = ["experience", "projects", "skills", "education"]
        .iter()
        .map(|n| format!("<!-- atsgate:{n} --><!-- /atsgate:{n} -->\n"))
        .collect();
    std::fs::write(&page, format!("<main>\n{markers}</main>\n")).expect("page");
    let filled = out.join("filled.html");
    let partials = out.join("partials");
    let args = [
        "inject",
        "--partials",
        s(&partials),
        "--page",
        s(&page),
        "--out",
        s(&filled),
    ];
    let result = atsgate(&args, None);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let html = std::fs::read_to_string(&filled).expect("filled");
    assert!(html.contains("atsgate-experience-northwind"));

    std::fs::write(&page, "<main></main>\n").expect("page");
    assert_eq!(atsgate(&args, None).status.code(), Some(1));
}

#[test]
fn usage_errors_exit_2() {
    assert_eq!(
        atsgate(&["build", "--career", "x"], None).status.code(),
        Some(2)
    );
}
