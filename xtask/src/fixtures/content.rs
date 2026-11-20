//! The synthetic resume every fixture renders. The person is invented; no real PII.
//!
//! Lines are pre-wrapped to fit the narrowest layout, so every layout carries identical text.

pub const NAME: &str = "Jordan Example";
pub const CONTACT: [&str; 3] = ["jordan@example.com", "+1 555 0100", "Springfield, USA"];

pub const SUMMARY_HEADING: &str = "SUMMARY";
pub const SUMMARY: [&str; 2] = [
    "Backend engineer with eight years of experience",
    "building data pipelines and APIs for finance.",
];

pub const EXPERIENCE_HEADING: &str = "EXPERIENCE";
pub const EDUCATION_HEADING: &str = "EDUCATION";
pub const SKILLS_HEADING: &str = "SKILLS";
pub const SKILLS: [&str; 2] = ["Rust, Go, PostgreSQL", "Kubernetes, Terraform"];

/// A dated entry: a role in Experience or a degree in Education.
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    pub title: &'static str,
    pub dates: &'static str,
    pub bullets: &'static [&'static str],
}

pub const ROLES: [Entry; 2] = [
    Entry {
        title: "Senior Software Engineer, Northwind Analytics",
        dates: "Jan 2020 \u{2013} Present",
        bullets: &[
            "Moved batch pipelines to Rust, cutting cost 30%.",
            "Designed a PostgreSQL schema for 2M requests/day.",
        ],
    },
    Entry {
        title: "Software Engineer, Contoso Labs",
        dates: "06/2016 \u{2013} 12/2019",
        bullets: &[
            "Built REST APIs in Go for internal billing.",
            "Mentored four junior engineers.",
        ],
    },
];

pub const EDUCATION: [Entry; 1] = [Entry {
    title: "B.S. Computer Science, State University",
    dates: "2012 \u{2013} 2016",
    bullets: &[],
}];

pub const BULLET: &str = "\u{2022} ";

/// Keywords a stuffed resume hides; none of them appear in the visible resume.
pub const STUFFED_KEYWORDS: &str = "Kubernetes Terraform AWS Java Python Spark Kafka";
