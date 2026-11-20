//! The `atsgate` command line. It is the only crate that performs I/O, and this file is the only
//! place that reads the environment.

mod args;
mod run;

use std::process::ExitCode;

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let command = match args::parse(&argv) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("atsgate: {message}\n{}", args::USAGE);
            return ExitCode::from(2);
        }
    };
    let env = run::Environment {
        in_ci: std::env::var("GITHUB_ACTIONS").is_ok_and(|v| v == "true"),
    };
    match run::run(&command, env) {
        Ok(lines) => {
            lines.iter().for_each(|line| println!("{line}"));
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("atsgate: {message}");
            ExitCode::FAILURE
        }
    }
}
