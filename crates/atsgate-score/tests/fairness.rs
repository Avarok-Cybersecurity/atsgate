//! Metamorphic fairness: changing what must never be an input leaves every score bit-identical.
//! The name, the school, the graduation year and the time between roles are varied over every
//! built-in blueprint.

mod common;

use atsgate_blueprint::BUILTINS;

struct Person<'a> {
    name: &'a str,
    school: &'a str,
    graduated: &'a str,
    /// The earlier role's dates; moving them back opens a gap before the later role.
    earlier_role: &'a str,
}

const BASE: Person<'static> = Person {
    name: "Jane Doe",
    school: "State University",
    graduated: "2012 - 2016",
    earlier_role: "Jun 2016 - Dec 2019",
};

fn resume(p: &Person<'_>) -> String {
    format!(
        "{name}\n\
         {email}@example.com | Springfield\n\
         \n\
         SUMMARY\n\
         Backend engineer building APIs and data pipelines.\n\
         \n\
         EXPERIENCE\n\
         Senior Software Engineer, Northwind Analytics Jan 2020 - Present\n\
         - Moved batch pipelines to Rust and PostgreSQL, cutting cost 30%.\n\
         - Deployed services on Kubernetes with Docker and Terraform.\n\
         Software Engineer, Contoso Labs {earlier}\n\
         - Built REST APIs in Go for internal billing.\n\
         - Mentored four junior engineers.\n\
         \n\
         EDUCATION\n\
         B.S. Computer Science, {school} {graduated}\n\
         \n\
         SKILLS\n\
         Rust, Go, Python, PostgreSQL, Kubernetes, Git, Linux\n",
        name = p.name,
        email = p.name.to_lowercase().replace(' ', "."),
        earlier = p.earlier_role,
        school = p.school,
        graduated = p.graduated,
    )
}

fn assert_invariant(variant: &Person<'_>, what: &str) {
    for (id, _) in BUILTINS {
        let blueprint = common::blueprint(id);
        let base = common::score_text(&resume(&BASE), &blueprint);
        let changed = common::score_text(&resume(variant), &blueprint);
        assert_eq!(
            common::numbers(&changed),
            common::numbers(&base),
            "{what} changed a score for {id}"
        );
    }
}

#[test]
fn the_name_is_never_an_input() {
    for name in [
        "Mohammed Al-Rashid",
        "Nguyen Thi Minh Khai",
        "Lakisha Washington",
        "Brad Miller",
    ] {
        assert_invariant(&Person { name, ..BASE }, "the name");
    }
}

#[test]
fn the_school_is_never_an_input() {
    for school in [
        "Harvard University",
        "Springfield Community College",
        "Universidad de Chile",
    ] {
        assert_invariant(&Person { school, ..BASE }, "the school");
    }
}

#[test]
fn the_graduation_year_is_never_an_input() {
    for graduated in ["1990 - 1994", "2019 - 2023", "2016"] {
        assert_invariant(&Person { graduated, ..BASE }, "the graduation year");
    }
}

#[test]
fn a_gap_between_roles_is_never_penalized() {
    // The same 43 months, ending three or ten years before the next role starts.
    for earlier_role in ["Jun 2013 - Dec 2016", "Jun 2006 - Dec 2009"] {
        assert_invariant(
            &Person {
                earlier_role,
                ..BASE
            },
            "a gap",
        );
    }
}

#[test]
fn the_metamorphic_resume_exercises_every_dimension() {
    // Guards the tests above against passing vacuously on an unparsed resume.
    let r = common::score_text(
        &resume(&BASE),
        &common::blueprint("software-engineer-backend"),
    );
    let d = &r.dimensions;
    for (name, score) in [
        ("R", d.required_skills.score),
        ("T", d.title.score),
        ("E", d.experience.score),
        ("D", d.education.score),
    ] {
        assert!(score.is_some_and(|s| s > 0.0), "{name} is {score:?}");
    }
    // Jan 2020 to Nov 2026 is 83 months, Jun 2016 to Dec 2019 is 43: 126 months.
    assert_eq!(r.relevant_years, 10.5);
}
