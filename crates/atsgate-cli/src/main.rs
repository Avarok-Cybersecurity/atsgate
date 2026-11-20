//! The `atsgate` command line. It is the only crate that performs I/O, and this file is the only
//! place that reads the environment.

mod args;
mod clock;
mod io;
mod options;
mod run;
mod score_args;
mod scoring;
mod spec;

use std::process::ExitCode;

/// A blueprint or job description fetch gives up after this long.
const FETCH_TIMEOUT_SECONDS: u64 = 30;

/// Appends `name=value` lines for a GitHub Actions step.
fn append_outputs(path: &std::path::Path, outputs: &[(String, String)]) -> std::io::Result<()> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    for (name, value) in outputs {
        writeln!(file, "{name}={value}")?;
    }
    Ok(())
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let command = match args::parse(&argv) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("atsgate: {message}\n{}", args::USAGE);
            return ExitCode::from(2);
        }
    };
    let now = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d.as_secs(),
        Err(e) => {
            eprintln!("atsgate: the system clock is before 1970: {e}");
            return ExitCode::FAILURE;
        }
    };
    let env = run::Environment {
        in_ci: std::env::var("GITHUB_ACTIONS").is_ok_and(|v| v == "true"),
        scoring: scoring::Context {
            today: clock::year_month(now),
            step_summary: std::env::var_os("GITHUB_STEP_SUMMARY")
                .filter(|v| !v.is_empty())
                .map(std::path::PathBuf::from),
        },
    };
    let system = io::System {
        timeout: std::time::Duration::from_secs(FETCH_TIMEOUT_SECONDS),
    };
    match run::run(&command, &env, &system) {
        Ok(outcome) => {
            outcome.lines.iter().for_each(|line| println!("{line}"));
            if let Some(path) = std::env::var_os("GITHUB_OUTPUT").filter(|v| !v.is_empty())
                && let Err(e) = append_outputs(std::path::Path::new(&path), &outcome.outputs)
            {
                eprintln!("atsgate: GITHUB_OUTPUT: {e}");
                return ExitCode::FAILURE;
            }
            if outcome.passed {
                ExitCode::SUCCESS
            } else {
                eprintln!("atsgate: the gate failed");
                ExitCode::FAILURE
            }
        }
        Err(message) => {
            eprintln!("atsgate: {message}");
            ExitCode::FAILURE
        }
    }
}
