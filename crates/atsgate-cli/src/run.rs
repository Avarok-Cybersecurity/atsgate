//! Runs a parsed command: reads its inputs, calls the pure crates, writes the outputs. This and
//! `main` are the only places in ATSGate that touch the file system.

use crate::args::{Command, Contact, Source};
use atsgate_career::{Career, ImportFormat, PrivateContact, Tailoring, import};
use atsgate_render::{ContactProfile, PARTIALS, build, guard_private_contact, inject};
use std::collections::BTreeMap;
use std::path::Path;

/// What the process knows about where it runs; read from the environment by `main` only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Environment {
    pub in_ci: bool,
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
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

/// Runs `command` and returns the lines to print on success.
pub fn run(command: &Command, env: Environment) -> Result<Vec<String>, String> {
    match command {
        Command::Import { from, input, out } => {
            let format = match from {
                Source::JsonResume => ImportFormat::JsonResume,
                Source::Tbraun96 => ImportFormat::Tbraun96,
            };
            let career = import(&read(input)?, format).map_err(|e| e.to_string())?;
            write(out, career.to_json().map_err(|e| e.to_string())?.as_bytes())?;
            Ok(vec![format!("wrote {}", out.display())])
        }
        Command::Build {
            career,
            tailoring,
            contact,
            allow_private_contact,
            out,
        } => {
            let profile = contact_profile(contact)?;
            guard_private_contact(&profile, env.in_ci, *allow_private_contact)
                .map_err(|e| e.to_string())?;
            let career_file = Career::from_json(&read(career)?)
                .map_err(|e| format!("{}: {e}", career.display()))?;
            let tailoring_file = Tailoring::from_yaml(&read(tailoring)?)
                .map_err(|e| format!("{}: {e}", tailoring.display()))?;
            let artifacts =
                build(&career_file, &tailoring_file, &profile).map_err(|e| e.to_string())?;
            let mut lines = Vec::new();
            for (rel, bytes) in &artifacts.files {
                let path = out.join(rel);
                write(&path, bytes)?;
                lines.push(format!("wrote {}", path.display()));
            }
            Ok(lines)
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
            Ok(vec![format!("wrote {}", out.display())])
        }
    }
}
