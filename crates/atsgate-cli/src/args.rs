//! Command-line parsing: arguments in, a [`Command`] out. Pure, so every rule is unit-tested.
//! Every option is required; there are no defaults to discover after the fact.

use crate::options::Options;
use crate::score_args::{self, SCORE_OPTIONS, ScoreArgs};
use std::path::PathBuf;

pub const USAGE: &str = "\
usage:
  atsgate import --from json-resume|tbraun96 --input FILE --out FILE
  atsgate build --career FILE --tailoring FILE --contact public|FILE [--allow-private-contact] --out DIR
  atsgate inject --partials DIR --page FILE --out FILE
  atsgate score --resume FILE SCORING
  atsgate gate --career FILE --tailoring FILE --contact public|FILE [--allow-private-contact] --out DIR SCORING

SCORING: --blueprint builtin:ID|FILE|https://URL|generate:FILE-OR-URL [--floors \"taleo=80, lever=0\"]
         --fail-on-integrity-flags true|false --as-of today|YYYY-MM --report FILE --summary FILE";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    JsonResume,
    Tbraun96,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Contact {
    Public,
    File(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Import {
        from: Source,
        input: PathBuf,
        out: PathBuf,
    },
    Build(BuildArgs),
    Inject {
        partials: PathBuf,
        page: PathBuf,
        out: PathBuf,
    },
    Score {
        resume: PathBuf,
        score: ScoreArgs,
    },
    /// `build`, then `score` of the PDF it rendered.
    Gate {
        build: BuildArgs,
        score: ScoreArgs,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildArgs {
    pub career: PathBuf,
    pub tailoring: PathBuf,
    pub contact: Contact,
    pub allow_private_contact: bool,
    pub out: PathBuf,
}

const BUILD_OPTIONS: [&str; 5] = [
    "--career",
    "--tailoring",
    "--contact",
    "--allow-private-contact",
    "--out",
];

fn build_args(o: &mut Options) -> Result<BuildArgs, String> {
    let contact = match o.take("--contact")? {
        c if c == "public" => Contact::Public,
        path => Contact::File(PathBuf::from(path)),
    };
    Ok(BuildArgs {
        career: o.path("--career")?,
        tailoring: o.path("--tailoring")?,
        contact,
        allow_private_contact: o.flag("--allow-private-contact"),
        out: o.path("--out")?,
    })
}

pub fn parse(args: &[String]) -> Result<Command, String> {
    let (command, rest) = args.split_first().ok_or("no command given")?;
    match command.as_str() {
        "import" => {
            let mut o = Options::parse(rest, &["--from", "--input", "--out"])?;
            let from = match o.take("--from")?.as_str() {
                "json-resume" => Source::JsonResume,
                "tbraun96" => Source::Tbraun96,
                other => {
                    return Err(format!(
                        "--from must be json-resume or tbraun96, not {other:?}"
                    ));
                }
            };
            Ok(Command::Import {
                from,
                input: o.path("--input")?,
                out: o.path("--out")?,
            })
        }
        "build" => {
            let mut o = Options::parse(rest, &BUILD_OPTIONS)?;
            Ok(Command::Build(build_args(&mut o)?))
        }
        "score" => {
            let known: Vec<&str> = ["--resume"].iter().chain(&SCORE_OPTIONS).copied().collect();
            let mut o = Options::parse(rest, &known)?;
            Ok(Command::Score {
                resume: o.path("--resume")?,
                score: score_args::parse(&mut o)?,
            })
        }
        "gate" => {
            let known: Vec<&str> = BUILD_OPTIONS
                .iter()
                .chain(&SCORE_OPTIONS)
                .copied()
                .collect();
            let mut o = Options::parse(rest, &known)?;
            Ok(Command::Gate {
                build: build_args(&mut o)?,
                score: score_args::parse(&mut o)?,
            })
        }
        "inject" => {
            let mut o = Options::parse(rest, &["--partials", "--page", "--out"])?;
            Ok(Command::Inject {
                partials: o.path("--partials")?,
                page: o.path("--page")?,
                out: o.path("--out")?,
            })
        }
        other => Err(format!("unknown command {other:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_owned).collect()
    }

    #[test]
    fn parses_build_with_and_without_the_private_flag() {
        let cmd = parse(&args(
            "build --career c.json --tailoring t.yaml --contact public --out o",
        ));
        assert_eq!(
            cmd,
            Ok(Command::Build(BuildArgs {
                career: "c.json".into(),
                tailoring: "t.yaml".into(),
                contact: Contact::Public,
                allow_private_contact: false,
                out: "o".into(),
            }))
        );
        let line =
            "build --allow-private-contact --contact p.json --career c --tailoring t --out o";
        let Ok(Command::Build(BuildArgs {
            contact,
            allow_private_contact,
            ..
        })) = parse(&args(line))
        else {
            panic!("expected build");
        };
        assert_eq!(contact, Contact::File("p.json".into()));
        assert!(allow_private_contact);
    }

    #[test]
    fn missing_repeated_unknown_and_valueless_options_are_errors() {
        let bad = [
            "",
            "build --career c --tailoring t --out o",
            "build --career c --career d --tailoring t --contact public --out o",
            "build --career c --tailoring t --contact public --out o --force",
            "import --from json-resume --input i --out",
            "import --from europass --input i --out o",
            "inject --partials p --page x.html --out y.html --allow-private-contact",
            "render",
        ];
        for line in bad {
            assert!(parse(&args(line)).is_err(), "{line}");
        }
    }
}
