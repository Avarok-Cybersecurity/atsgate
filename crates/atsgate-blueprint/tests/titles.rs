//! Job titles to O*NET occupations: seniority and technology qualifiers removed, ambiguous
//! titles read in the context of the occupation they are scored against.

use atsgate_blueprint::generate::resolve_occupation;
use atsgate_taxonomy::Taxonomy;

fn code(title: &str, context: Option<&str>) -> Option<String> {
    let t = Taxonomy::builtin().expect("taxonomy");
    resolve_occupation(title, t, context).map(|r| r.taxonomy_ref)
}

const SOFTWARE: &str = "onet-soc:15-1252.00";

#[test]
fn technology_qualified_titles_resolve_to_software_developers() {
    for title in [
        "Rust Software Engineer",
        "Principal Rust Software Engineer",
        "Senior Rust Systems Engineer",
        "Rust Developer",
    ] {
        assert_eq!(code(title, None).as_deref(), Some(SOFTWARE), "{title}");
    }
}

#[test]
fn ambiguous_titles_read_as_software_titles_in_a_software_context() {
    let ctx = Some("15-1252.00");
    for title in [
        "Systems Engineer",
        "Staff Engineer",
        "Senior Engineer",
        "Principal Engineer",
        "Founding Engineer",
    ] {
        assert_eq!(code(title, ctx).as_deref(), Some(SOFTWARE), "{title}");
    }
    assert_eq!(
        code("Security Engineer", ctx).as_deref(),
        Some("onet-soc:15-1299.05")
    );
}

#[test]
fn context_never_overrides_a_specific_title() {
    let ctx = Some("15-1252.00");
    assert_eq!(
        code("Civil Engineer", ctx).as_deref(),
        Some("onet-soc:17-2051.00")
    );
    // Outside software, an ambiguous title keeps the meaning that context names.
    assert_eq!(
        code("Systems Engineer", Some("17-2112.00")).as_deref(),
        Some("onet-soc:17-2112.00")
    );
    // A bare seniority title with no context names no occupation.
    assert_eq!(code("Senior Engineer", None), None);
}
