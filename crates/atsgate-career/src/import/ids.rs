//! Ids for entries that arrive without one: a slug of the entry's name, made unique.

use super::ImportError;
use crate::extension::Id;
use std::collections::BTreeSet;

/// Lowercase ASCII letters and digits, every other run of characters collapsed to one hyphen.
pub fn slug(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_owned()
}

/// Hands out ids that are unique across the whole career file.
#[derive(Debug, Default)]
pub struct Ids {
    taken: BTreeSet<String>,
}

impl Ids {
    /// Claims an id the source already gave; a malformed or repeated one is an error.
    pub fn given(&mut self, item: &str, id: String) -> Result<Id, ImportError> {
        let parsed = Id::try_from(id.clone()).map_err(|e| ImportError::unrepresentable(item, e))?;
        if !self.taken.insert(id.clone()) {
            return Err(ImportError::unrepresentable(
                item,
                format!("id {id:?} repeats"),
            ));
        }
        Ok(parsed)
    }

    /// A new id from `name`, suffixed `-2`, `-3`... on collision; `fallback` when the slug is empty.
    pub fn derive(&mut self, name: &str, fallback: &str) -> Result<Id, ImportError> {
        let base = Some(slug(name))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| fallback.to_owned());
        let mut candidate = base.clone();
        let mut n = 2;
        while self.taken.contains(&candidate) {
            candidate = format!("{base}-{n}");
            n += 1;
        }
        self.given(name, candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_and_suffixes() {
        assert_eq!(slug("Oregon State University"), "oregon-state-university");
        assert_eq!(slug("US Patent 12,224,993 B2"), "us-patent-12-224-993-b2");
        assert_eq!(slug("  --  "), "");
        let mut ids = Ids::default();
        assert_eq!(ids.derive("Acme", "work").unwrap().as_str(), "acme");
        assert_eq!(ids.derive("ACME", "work").unwrap().as_str(), "acme-2");
        assert_eq!(ids.derive("***", "work").unwrap().as_str(), "work");
        assert!(ids.given("x", "acme".to_owned()).is_err());
        assert!(ids.given("x", "Not An Id".to_owned()).is_err());
    }
}
