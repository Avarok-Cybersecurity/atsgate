//! Command-line parsing: arguments in, a [`Command`] out. Pure, so every rule is unit-tested.
//! Every option is required; there are no defaults to discover after the fact.

use std::collections::BTreeMap;
use std::path::PathBuf;

pub const USAGE: &str = "\
usage:
  atsgate import --from json-resume|tbraun96 --input FILE --out FILE
  atsgate build --career FILE --tailoring FILE --contact public|FILE [--allow-private-contact] --out DIR
  atsgate inject --partials DIR --page FILE --out FILE";

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
    Build {
        career: PathBuf,
        tailoring: PathBuf,
        contact: Contact,
        allow_private_contact: bool,
        out: PathBuf,
    },
    Inject {
        partials: PathBuf,
        page: PathBuf,
        out: PathBuf,
    },
}

const FLAGS: [&str; 1] = ["--allow-private-contact"];

/// Options as `--name value` pairs, and flags; a repeated or unknown option is an error.
struct Options {
    values: BTreeMap<String, String>,
    flags: Vec<String>,
}

impl Options {
    fn parse(args: &[String], known: &[&str]) -> Result<Self, String> {
        let (mut values, mut flags) = (BTreeMap::new(), Vec::new());
        let mut it = args.iter();
        while let Some(arg) = it.next() {
            if FLAGS.contains(&arg.as_str()) && known.contains(&arg.as_str()) {
                if flags.contains(arg) {
                    return Err(format!("{arg} is given twice"));
                }
                flags.push(arg.clone());
                continue;
            }
            if !known.contains(&arg.as_str()) {
                return Err(format!("unexpected argument {arg:?}"));
            }
            let value = it.next().ok_or_else(|| format!("{arg} needs a value"))?;
            if values.insert(arg.clone(), value.clone()).is_some() {
                return Err(format!("{arg} is given twice"));
            }
        }
        Ok(Self { values, flags })
    }

    fn take(&mut self, name: &str) -> Result<String, String> {
        self.values
            .remove(name)
            .ok_or_else(|| format!("{name} is required"))
    }

    fn path(&mut self, name: &str) -> Result<PathBuf, String> {
        self.take(name).map(PathBuf::from)
    }

    fn flag(&self, name: &str) -> bool {
        self.flags.iter().any(|f| f == name)
    }
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
            let known = [
                "--career",
                "--tailoring",
                "--contact",
                "--allow-private-contact",
                "--out",
            ];
            let mut o = Options::parse(rest, &known)?;
            let contact = match o.take("--contact")? {
                c if c == "public" => Contact::Public,
                path => Contact::File(PathBuf::from(path)),
            };
            Ok(Command::Build {
                career: o.path("--career")?,
                tailoring: o.path("--tailoring")?,
                contact,
                allow_private_contact: o.flag("--allow-private-contact"),
                out: o.path("--out")?,
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
            Ok(Command::Build {
                career: "c.json".into(),
                tailoring: "t.yaml".into(),
                contact: Contact::Public,
                allow_private_contact: false,
                out: "o".into(),
            })
        );
        let line =
            "build --allow-private-contact --contact p.json --career c --tailoring t --out o";
        let Ok(Command::Build {
            contact,
            allow_private_contact,
            ..
        }) = parse(&args(line))
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
