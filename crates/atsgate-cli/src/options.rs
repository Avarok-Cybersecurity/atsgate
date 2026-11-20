//! `--name value` options and flags; a repeated or unknown option is an error.

use std::collections::BTreeMap;
use std::path::PathBuf;

const FLAGS: [&str; 1] = ["--allow-private-contact"];

pub struct Options {
    values: BTreeMap<String, String>,
    flags: Vec<String>,
}

impl Options {
    pub fn parse(args: &[String], known: &[&str]) -> Result<Self, String> {
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

    pub fn take(&mut self, name: &str) -> Result<String, String> {
        self.values
            .remove(name)
            .ok_or_else(|| format!("{name} is required"))
    }

    /// An option that may be left out; the caller documents what its absence means.
    pub fn optional(&mut self, name: &str) -> Option<String> {
        self.values.remove(name)
    }

    pub fn path(&mut self, name: &str) -> Result<PathBuf, String> {
        self.take(name).map(PathBuf::from)
    }

    pub fn flag(&self, name: &str) -> bool {
        self.flags.iter().any(|f| f == name)
    }
}
