//! Repository maintenance tasks. This binary is the I/O boundary; the generators it calls are pure.

mod citations;

use std::path::Path;
use std::process::ExitCode;

/// Whether a task writes its generated output or only verifies the committed copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Write,
    Check,
}

fn parse_mode(flag: Option<&str>) -> Result<Mode, String> {
    match flag {
        Some("--write") => Ok(Mode::Write),
        Some("--check") => Ok(Mode::Check),
        other => Err(format!("expected --write or --check, got {other:?}")),
    }
}

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below the repository root")
}

/// Writes `expected` to `path`, or in check mode fails when the committed file differs.
fn sync_file(path: &Path, expected: &str, mode: Mode) -> Result<(), String> {
    let current = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if current == expected {
        return Ok(());
    }
    match mode {
        Mode::Check => Err(format!(
            "{} is stale; run `cargo run -p xtask -- <task> --write`",
            path.display()
        )),
        Mode::Write => {
            std::fs::write(path, expected).map_err(|e| format!("{}: {e}", path.display()))
        }
    }
}

fn run_citations(mode: Mode) -> Result<(), String> {
    let source = std::fs::read_to_string(root().join("CITATIONS.md")).map_err(|e| e.to_string())?;
    let entries = citations::parse(&source)?;
    let readme_path = root().join("README.md");
    let readme = std::fs::read_to_string(&readme_path).map_err(|e| e.to_string())?;
    let updated = citations::splice_readme(&readme, &citations::render(&entries))?;
    sync_file(&readme_path, &updated, mode)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let task = args.first().map(String::as_str);
    let result = parse_mode(args.get(1).map(String::as_str)).and_then(|mode| match task {
        Some("citations") => run_citations(mode),
        other => Err(format!("unknown task {other:?}; tasks: citations")),
    });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("xtask: {message}");
            ExitCode::FAILURE
        }
    }
}
