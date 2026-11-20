//! Parses CITATIONS.md (the SSOT) and renders the README's generated Citations section.

const COLUMNS: [&str; 8] = [
    "id", "title", "authors", "date", "url", "accessed", "grade", "supports",
];
const GRADES: [&str; 6] = ["PR", "AX", "VD", "IB", "CR", "LG"];
pub const BEGIN: &str = "<!-- citations:begin (generated from CITATIONS.md; do not edit) -->";
pub const END: &str = "<!-- citations:end -->";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Citation {
    pub id: String,
    pub title: String,
    pub authors: String,
    pub date: String,
    pub url: String,
    pub accessed: String,
    pub grade: String,
    pub supports: String,
}

fn cells(line: &str) -> Vec<String> {
    let inner = line.trim().trim_start_matches('|').trim_end_matches('|');
    inner.split('|').map(|c| c.trim().to_owned()).collect()
}

fn is_iso_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

fn validate(c: &Citation, seen: &mut Vec<String>) -> Result<(), String> {
    let fields = [
        &c.id,
        &c.title,
        &c.authors,
        &c.date,
        &c.url,
        &c.accessed,
        &c.grade,
        &c.supports,
    ];
    if let Some(i) = fields.iter().position(|f| f.is_empty()) {
        return Err(format!("citation {:?}: empty {}", c.id, COLUMNS[i]));
    }
    if !c.url.starts_with("https://") {
        return Err(format!("citation {}: url must be https", c.id));
    }
    if !is_iso_date(&c.accessed) {
        return Err(format!("citation {}: accessed must be YYYY-MM-DD", c.id));
    }
    if !GRADES.contains(&c.grade.as_str()) {
        return Err(format!("citation {}: unknown grade {}", c.id, c.grade));
    }
    if seen.contains(&c.id) {
        return Err(format!("citation {}: duplicate id", c.id));
    }
    seen.push(c.id.clone());
    Ok(())
}

/// Parses the first table whose header row is exactly [`COLUMNS`].
pub fn parse(markdown: &str) -> Result<Vec<Citation>, String> {
    let mut lines = markdown.lines();
    lines
        .by_ref()
        .find(|l| l.starts_with('|') && cells(l) == COLUMNS)
        .ok_or("CITATIONS.md has no citation table with the expected header")?;
    lines
        .next()
        .filter(|l| l.starts_with("|---"))
        .ok_or("missing table separator row")?;
    let mut seen = Vec::new();
    let mut out = Vec::new();
    for line in lines.take_while(|l| l.starts_with('|')) {
        let c = cells(line);
        let [id, title, authors, date, url, accessed, grade, supports]: [String; 8] = c
            .try_into()
            .map_err(|c: Vec<String>| format!("row has {} cells, expected 8: {line}", c.len()))?;
        let entry = Citation {
            id,
            title,
            authors,
            date,
            url,
            accessed,
            grade,
            supports,
        };
        validate(&entry, &mut seen)?;
        out.push(entry);
    }
    if out.is_empty() {
        return Err("citation table is empty".into());
    }
    Ok(out)
}

/// Renders the citation list that sits between the README markers.
pub fn render(entries: &[Citation]) -> String {
    let mut s = String::new();
    for c in entries {
        s.push_str(&format!(
            "- <a id=\"cite-{id}\"></a>**{id}**: {authors}. *{title}*. {date}. <{url}> ({grade}; accessed {accessed}). Supports: {supports}.\n",
            id = c.id,
            authors = c.authors,
            title = c.title,
            date = c.date,
            url = c.url,
            grade = c.grade,
            accessed = c.accessed,
            supports = c.supports.trim_end_matches('.'),
        ));
    }
    s
}

/// Replaces everything between [`BEGIN`] and [`END`] in `readme` with `body`.
pub fn splice_readme(readme: &str, body: &str) -> Result<String, String> {
    let start = readme
        .find(BEGIN)
        .ok_or("README.md lacks the citations begin marker")?;
    let end = readme
        .find(END)
        .ok_or("README.md lacks the citations end marker")?;
    if end < start {
        return Err("README.md citation markers are out of order".into());
    }
    let head = &readme[..start + BEGIN.len()];
    Ok(format!("{head}\n{body}{}", &readme[end..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "| id | title | authors | date | url | accessed | grade | supports |\n|---|---|---|---|---|---|---|---|\n";

    fn table(rows: &str) -> String {
        format!("# C\n\n{HEADER}{rows}\nafter\n")
    }

    const ROW: &str = "| a | T | A | 2025 | https://x.org | 2026-11-20 | AX | S |\n";

    #[test]
    fn parses_a_valid_row() {
        let got = parse(&table(ROW)).expect("valid");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].url, "https://x.org");
    }

    #[test]
    fn rejects_duplicate_ids() {
        assert!(
            parse(&table(&format!("{ROW}{ROW}")))
                .unwrap_err()
                .contains("duplicate")
        );
    }

    #[test]
    fn rejects_bad_grade_url_date_and_empty_cells() {
        for bad in [
            ROW.replace("AX", "ZZ"),
            ROW.replace("https://x.org", "http://x.org"),
            ROW.replace("2026-11-20", "Nov 2026"),
            ROW.replace("| S |", "|  |"),
            ROW.replace("| S |", "| S | extra |"),
        ] {
            assert!(parse(&table(&bad)).is_err(), "accepted {bad}");
        }
    }

    #[test]
    fn splice_replaces_only_the_generated_region() {
        let readme = format!("intro\n{BEGIN}\nold\n{END}\noutro\n");
        let got = splice_readme(&readme, "new\n").expect("markers present");
        assert_eq!(got, format!("intro\n{BEGIN}\nnew\n{END}\noutro\n"));
        assert!(splice_readme("no markers", "x").is_err());
    }

    #[test]
    fn render_links_each_entry_by_anchor() {
        let got = render(&parse(&table(ROW)).expect("valid"));
        assert!(got.starts_with("- <a id=\"cite-a\"></a>**a**: A. *T*. 2025. <https://x.org>"));
    }
}
