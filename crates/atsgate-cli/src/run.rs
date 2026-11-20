//! Runs a parsed command: reads its inputs, calls the pure crates, writes the outputs. The CLI
//! crate is the only part of ATSGate that touches the file system or the network: inputs go
//! through [`crate::io::Io`], outputs through [`write`].

use crate::args::{BuildArgs, Command, Contact, Source};
use crate::io::Io;
use crate::scoring::{self, Context};
use atsgate_career::{Career, ImportFormat, PrivateContact, Tailoring, import};
use atsgate_render::{
    Artifacts, ContactProfile, DOCX_FILE, PARTIALS, PDF_FILE, build, guard_private_contact, inject,
};
use std::collections::BTreeMap;
use std::path::Path;

/// What the process knows about where it runs; read from the environment by `main` only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    pub in_ci: bool,
    pub scoring: Context,
}

/// What a command printed, whether its gate (if any) passed, and the `name=value` outputs a
/// GitHub Action step exposes (single-line values).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub lines: Vec<String>,
    pub passed: bool,
    pub outputs: Vec<(String, String)>,
}

fn done(lines: Vec<String>) -> Outcome {
    Outcome {
        lines,
        passed: true,
        outputs: Vec::new(),
    }
}

fn build_outputs(b: &BuildArgs) -> Vec<(String, String)> {
    [
        ("resume-pdf", b.out.join(PDF_FILE)),
        ("resume-docx", b.out.join(DOCX_FILE)),
        ("partials-dir", b.out.join("partials")),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v.display().to_string()))
    .collect()
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    std::fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
}

fn contact_profile(contact: &Contact) -> Result<ContactProfile, String> {
    match contact {
        Contact::Public => Ok(ContactProfile::Public),
        Contact::File(path) => PrivateContact::from_json(&read(path)?)
            .map(ContactProfile::Private)
            .map_err(|e| format!("{}: {e}", path.display())),
    }
}

fn build_files(b: &BuildArgs, in_ci: bool) -> Result<(Artifacts, Vec<String>), String> {
    let profile = contact_profile(&b.contact)?;
    guard_private_contact(&profile, in_ci, b.allow_private_contact).map_err(|e| e.to_string())?;
    let career =
        Career::from_json(&read(&b.career)?).map_err(|e| format!("{}: {e}", b.career.display()))?;
    let tailoring = Tailoring::from_yaml(&read(&b.tailoring)?)
        .map_err(|e| format!("{}: {e}", b.tailoring.display()))?;
    let artifacts = build(&career, &tailoring, &profile).map_err(|e| e.to_string())?;
    let mut lines = Vec::new();
    for (rel, bytes) in &artifacts.files {
        let path = b.out.join(rel);
        write(&path, bytes)?;
        lines.push(format!("wrote {}", path.display()));
    }
    Ok((artifacts, lines))
}

/// Runs `command` and returns what to print.
pub fn run(command: &Command, env: &Environment, io: &dyn Io) -> Result<Outcome, String> {
    match command {
        Command::Import { from, input, out } => {
            let format = match from {
                Source::JsonResume => ImportFormat::JsonResume,
                Source::Tbraun96 => ImportFormat::Tbraun96,
            };
            let career = import(&read(input)?, format).map_err(|e| e.to_string())?;
            write(out, career.to_json().map_err(|e| e.to_string())?.as_bytes())?;
            Ok(done(vec![format!("wrote {}", out.display())]))
        }
        Command::Build(b) => {
            let lines = build_files(b, env.in_ci)?.1;
            Ok(Outcome {
                outputs: build_outputs(b),
                ..done(lines)
            })
        }
        Command::Score { resume, score } => {
            let bytes = io.read(resume)?;
            let scored = scoring::run(&bytes, resume, score, io, &env.scoring)?;
            Ok(Outcome {
                lines: scored.lines,
                passed: scored.passed,
                outputs: scored.outputs,
            })
        }
        Command::Gate { build, score } => {
            let (artifacts, mut lines) = build_files(build, env.in_ci)?;
            let pdf = &artifacts.files[PDF_FILE];
            let path = build.out.join(PDF_FILE);
            let scored = scoring::run(pdf, &path, score, io, &env.scoring)?;
            lines.extend(scored.lines);
            let mut outputs = build_outputs(build);
            outputs.extend(scored.outputs);
            Ok(Outcome {
                lines,
                passed: scored.passed,
                outputs,
            })
        }
        Command::Inject {
            partials,
            page,
            out,
        } => {
            let mut fragments = BTreeMap::new();
            for (name, _) in PARTIALS {
                let fragment = read(&partials.join(format!("{name}.html")))?;
                fragments.insert(name.to_owned(), fragment);
            }
            let filled = inject(&read(page)?, &fragments).map_err(|e| e.to_string())?;
            write(out, filled.as_bytes())?;
            Ok(done(vec![format!("wrote {}", out.display())]))
        }
    }
}
