//! The `atsgate` command line. It is the only crate that performs I/O.

use std::process::ExitCode;

fn main() -> ExitCode {
    eprintln!("atsgate: no commands are available yet; scoring arrives in v0.1");
    ExitCode::from(2)
}
