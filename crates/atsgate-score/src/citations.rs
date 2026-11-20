//! Vendor documentation links, read from CITATIONS.md (the single source of truth for sources)
//! at build time, so a URL is never copied into code.

const CITATIONS: &str = include_str!("../../../CITATIONS.md");

/// The URL of the CITATIONS.md entry `id`.
pub fn url(id: &str) -> Option<&'static str> {
    CITATIONS.lines().find_map(|line| {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        // | id | title | authors | date | url | ...: cells[0] is the empty text before the bar.
        (cells.len() > 5 && cells[1] == id).then(|| cells[5])
    })
}
