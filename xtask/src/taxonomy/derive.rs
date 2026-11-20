//! Aliases derived mechanically from source labels. Each rule is documented in DATA-LICENSES.md
//! as a modification of the source data.
//!
//! O*NET's trailing " software" ("HubSpot software") is deliberately *not* stripped: it turns
//! "Chat software" and "Music software" into the ordinary words "Chat" and "Music". Brand names
//! get their short form from the curated table instead.

/// "Python (computer programming)" -> "Python".
pub fn strip_parenthetical(label: &str) -> Option<String> {
    let open = label.rfind(" (")?;
    label
        .ends_with(')')
        .then(|| label[..open].trim().to_owned())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_only_from_a_trailing_qualifier() {
        assert_eq!(
            strip_parenthetical("Python (computer programming)"),
            Some("Python".into())
        );
        assert_eq!(
            strip_parenthetical("CI (continuous integration) tools"),
            None
        );
        assert_eq!(strip_parenthetical("(x)"), None);
    }
}
