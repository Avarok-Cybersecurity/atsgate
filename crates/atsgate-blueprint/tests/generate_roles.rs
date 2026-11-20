//! Hacker News "Who's Hiring" postings: a company header, several roles in a list, and a
//! lowercase stack line. Synthetic postings with the shapes of real ones.

use atsgate_blueprint::generate::{GenerateError, GenerateOptions, generate};
use atsgate_blueprint::{Blueprint, Seniority};
use atsgate_taxonomy::{Gazetteer, Scope, Taxonomy};

fn run(text: &str) -> Result<Blueprint, GenerateError> {
    let gazetteer = Gazetteer::new(Taxonomy::builtin().expect("taxonomy"), Scope::Postings);
    let options = GenerateOptions {
        id: "test".to_owned(),
        version: "0.1.0".to_owned(),
        occupational_category: None,
    };
    generate(text, &options, &gazetteer)
}

const DASHED_ROLES: &str = "Acme | US-Based / Remote | Full-time – https://acme.test/\n\
    \n\
    Acme is a startup powering real-time data replication, from CDC to OLAP data flows.\n\
    \n\
    We're hiring for:\n\
    \n\
    Head of Engineering — Own the engineering function end-to-end as a partner to our CTO.\n\
    \n\
    Senior Systems Engineer (Control & Data Plane) — Build the orchestration and stream \
    transport. You'll work primarily in Rust on fault tolerance, consensus, and networking.\n\
    \n\
    Product Manager — Own product areas end-to-end.\n";

#[test]
fn a_multi_role_posting_takes_the_first_listed_role_that_names_a_title() {
    let b = run(DASHED_ROLES).expect("generates");
    assert_eq!(b.title, "Senior Systems Engineer");
    assert_eq!(b.seniority, Seniority::Senior);
}

const BULLETED_ROLES: &str = "Acme | Security, Machine Learning | REMOTE / Brno | Equity\n\
    \n\
    We protect end users from financial fraud.\n\
    \n\
    Stack: [python, typescript, rust], [redis, postgresql], [aws, terraform] ...\n\
    \n\
    We are looking for exceptional engineers to build with us:\n\
    \n\
    - Backend Engineer [rust, python] - low-latency stream processing engine\n\
    \n\
    - DevOps Engineer (IaaC in AWS)\n";

#[test]
fn a_bulleted_role_list_gives_the_title_without_its_brackets() {
    let b = run(BULLETED_ROLES).expect("generates");
    assert_eq!(b.title, "Backend Engineer");
    assert_eq!(b.seniority, Seniority::Unspecified);
}

#[test]
fn a_stack_line_names_its_items_whatever_their_case() {
    let b = run(BULLETED_ROLES).expect("generates");
    let required: Vec<&str> = b
        .required_skills
        .iter()
        .map(|s| s.canonical.as_str())
        .collect();
    for skill in [
        "Python",
        "TypeScript",
        "Rust",
        "Redis",
        "PostgreSQL",
        "AWS",
        "Terraform",
    ] {
        assert!(required.contains(&skill), "{skill} missing from {required:?}");
    }
}

#[test]
fn prose_about_hiring_engineers_is_not_a_title() {
    let text = "Acme | San Francisco | Remote | Equity\n\
        \n\
        We build a governed AI workspace.\n\
        \n\
        Looking for engineers (security, local-first systems) and GTM people.\n\
        \n\
        We build on Rust and Svelte.\n";
    assert!(matches!(run(text), Err(GenerateError::NoTitle)));
}
