//! Standard section headings and their common variants.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SectionKind {
    Summary,
    Experience,
    Education,
    Skills,
    Projects,
    Certifications,
    Languages,
    Awards,
    Publications,
    Volunteer,
}

impl SectionKind {
    /// Sections whose body is a list of dated entries.
    pub fn has_entries(self) -> bool {
        matches!(
            self,
            Self::Experience | Self::Education | Self::Projects | Self::Volunteer
        )
    }
}

const HEADINGS: [(SectionKind, &[&str]); 10] = [
    (
        SectionKind::Summary,
        &[
            "summary",
            "professional summary",
            "profile",
            "professional profile",
            "about",
            "about me",
            "objective",
            "career objective",
        ],
    ),
    (
        SectionKind::Experience,
        &[
            "experience",
            "work experience",
            "professional experience",
            "employment",
            "employment history",
            "work history",
            "career history",
            "relevant experience",
        ],
    ),
    (
        SectionKind::Education,
        &[
            "education",
            "academic background",
            "education and training",
            "education & training",
        ],
    ),
    (
        SectionKind::Skills,
        &[
            "skills",
            "technical skills",
            "core competencies",
            "competencies",
            "key skills",
            "skills & tools",
            "skills and tools",
            "technologies",
        ],
    ),
    (
        SectionKind::Projects,
        &[
            "projects",
            "selected projects",
            "personal projects",
            "key projects",
        ],
    ),
    (
        SectionKind::Certifications,
        &[
            "certifications",
            "certificates",
            "licenses & certifications",
            "licenses and certifications",
        ],
    ),
    (SectionKind::Languages, &["languages"]),
    (
        SectionKind::Awards,
        &[
            "awards",
            "honors",
            "honours",
            "awards & honors",
            "awards and honors",
        ],
    ),
    (SectionKind::Publications, &["publications", "papers"]),
    (
        SectionKind::Volunteer,
        &["volunteer", "volunteering", "volunteer experience"],
    ),
];

/// The section a normalized line names, if it is a standard heading.
pub fn standard(normalized: &str) -> Option<SectionKind> {
    let key = normalized.trim_end_matches(':').trim().to_lowercase();
    HEADINGS
        .iter()
        .find(|(_, names)| names.contains(&key.as_str()))
        .map(|(kind, _)| *kind)
}

/// A short all-capitals line that is not a standard heading: likely a non-standard heading.
pub fn looks_like_heading(normalized: &str) -> bool {
    let words = normalized.split_whitespace().count();
    let letters: Vec<char> = normalized.chars().filter(|c| c.is_alphabetic()).collect();
    (1..=4).contains(&words) && letters.len() >= 3 && letters.iter().all(|c| c.is_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_variants_case_and_colon_insensitively() {
        assert_eq!(standard("WORK EXPERIENCE"), Some(SectionKind::Experience));
        assert_eq!(standard("Technical Skills:"), Some(SectionKind::Skills));
        assert_eq!(standard("Experience at Acme"), None);
    }

    #[test]
    fn flags_unknown_all_caps_lines_only() {
        assert!(looks_like_heading("MY JOURNEY"));
        assert!(!looks_like_heading("My journey"));
        assert!(!looks_like_heading("A B"));
    }
}
