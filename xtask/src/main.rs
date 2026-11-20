//! Repository maintenance tasks. This binary is the I/O boundary; the generators it calls are pure.

mod citations;
mod eval;
mod examples;
mod fetch;
mod fixtures;
mod schemas;
mod site;
mod taxonomy;

use std::path::Path;
use std::process::ExitCode;

/// Whether a task writes its generated output or only verifies the committed copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Write,
    Check,
}

pub fn parse_mode(flag: Option<&str>) -> Result<Mode, String> {
    match flag {
        Some("--write") => Ok(Mode::Write),
        Some("--check") => Ok(Mode::Check),
        other => Err(format!("expected --write or --check, got {other:?}")),
    }
}

pub fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below the repository root")
}

/// Writes `expected` to `path`, or in check mode fails when the committed file differs or is missing.
pub fn sync_file(path: &Path, expected: &[u8], mode: Mode) -> Result<(), String> {
    let current = match std::fs::read(path) {
        Ok(bytes) => Some(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    if current.as_deref() == Some(expected) {
        return Ok(());
    }
    match mode {
        Mode::Check => Err(format!(
            "{} is stale or missing; run `cargo run -p xtask -- <task> --write`",
            path.display()
        )),
        Mode::Write => {
            let parent = path.parent().ok_or("generated path has no parent")?;
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
            std::fs::write(path, expected).map_err(|e| format!("{}: {e}", path.display()))
        }
    }
}

fn sync_all(files: Vec<(String, Vec<u8>)>, mode: Mode) -> Result<(), String> {
    files
        .iter()
        .try_for_each(|(rel, bytes)| sync_file(&root().join(rel), bytes, mode))
}

fn run_citations(mode: Mode) -> Result<(), String> {
    let source = std::fs::read_to_string(root().join("CITATIONS.md")).map_err(|e| e.to_string())?;
    let entries = citations::parse(&source)?;
    let readme_path = root().join("README.md");
    let readme = std::fs::read_to_string(&readme_path).map_err(|e| e.to_string())?;
    let updated = citations::splice_readme(&readme, &citations::render(&entries))?;
    sync_file(&readme_path, updated.as_bytes(), mode)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let task = args.first().map(String::as_str);
    let flag = args.get(1).map(String::as_str);
    let result = match task {
        Some("taxonomy") => taxonomy::run(flag),
        _ => parse_mode(flag).and_then(|mode| match task {
            Some("citations") => run_citations(mode),
            Some("fixtures") => fixtures::all().and_then(|f| sync_all(f, mode)),
            Some("schemas") => schemas::all().and_then(|f| sync_all(f, mode)),
            Some("examples") => examples::all().and_then(|f| sync_all(f, mode)),
            Some("eval") => eval::run(mode),
            Some("site") => site::all().and_then(|f| sync_all(f, mode)),
            other => Err(format!(
                "unknown task {other:?}; tasks: citations, eval, examples, fixtures, schemas, site, taxonomy"
            )),
        }),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("xtask: {message}");
            ExitCode::FAILURE
        }
    }
}
