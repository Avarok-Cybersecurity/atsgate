//! Years as the union of dated intervals, and knockouts met, unmet or unknown.

mod common;

use atsgate_blueprint::{Knockout, KnockoutType, UnknownPolicy};
use atsgate_resume::{DateEnd, DateRange, PartialDate};
use atsgate_score::experience::{interval, union_months};
use atsgate_score::knockouts::{Outcome, Status};
use atsgate_score::{Floors, YearMonth, gate};

fn ym(year: u16, month: Option<u8>) -> PartialDate {
    PartialDate { year, month }
}

#[test]
fn months_are_counted_without_rounding_up() {
    let as_of = YearMonth {
        year: 2026,
        month: 11,
    };
    let range = |s, e| DateRange { start: s, end: e };
    assert_eq!(
        interval(range(ym(2012, None), DateEnd::Date(ym(2016, None))), as_of),
        Some((2012 * 12, 2016 * 12))
    );
    let jun_dec = interval(
        range(ym(2016, Some(6)), DateEnd::Date(ym(2019, Some(12)))),
        as_of,
    )
    .expect("range");
    assert_eq!(jun_dec.1 - jun_dec.0, 43);
    let present = interval(range(ym(2026, Some(1)), DateEnd::Present), as_of).expect("range");
    assert_eq!(present.1 - present.0, 11);
    assert_eq!(
        interval(range(ym(2027, Some(1)), DateEnd::Present), as_of),
        None,
        "future start"
    );
    assert_eq!(
        interval(range(ym(2020, None), DateEnd::Date(ym(2018, None))), as_of),
        None,
        "reversed"
    );
}

#[test]
fn overlapping_roles_count_once_and_gaps_subtract_nothing() {
    assert_eq!(union_months(vec![(0, 12), (6, 18)]), 18);
    assert_eq!(union_months(vec![(0, 12), (24, 36)]), 24);
    assert_eq!(union_months(vec![(24, 36), (0, 12), (100, 112)]), 36);
    assert_eq!(union_months(vec![(0, 12), (12, 24)]), 24);
    assert_eq!(union_months(Vec::new()), 0);
}

#[test]
fn year_months_parse_strictly() {
    assert_eq!(
        YearMonth::parse("2026-11"),
        Some(YearMonth {
            year: 2026,
            month: 11
        })
    );
    for bad in ["2026-13", "2026-1", "26-11", "2026/11", "today", ""] {
        assert_eq!(YearMonth::parse(bad), None, "{bad}");
    }
}

fn with_knockouts(knockouts: Vec<Knockout>) -> atsgate_blueprint::Blueprint {
    let mut b = common::blueprint("software-engineer-backend");
    b.knockouts = knockouts;
    b
}

fn ko(kind: KnockoutType, value: &str, policy: UnknownPolicy) -> Knockout {
    Knockout {
        kind,
        value: value.to_owned(),
        unknown_policy: policy,
    }
}

const RESUME: &str = "EXPERIENCE\n\
    Software Engineer, Acme Jan 2020 - Dec 2023\n\
    - Built services in Go and PostgreSQL.\n\
    \n\
    EDUCATION\n\
    B.S. Computer Science, State University 2012 - 2016\n\
    \n\
    CERTIFICATIONS\n\
    CKA\n";

#[test]
fn knockouts_resolve_to_met_unmet_or_unknown() {
    let b = with_knockouts(vec![
        ko(KnockoutType::MinYears, "3", UnknownPolicy::Fail),
        ko(KnockoutType::MinYears, "5", UnknownPolicy::Pass),
        ko(KnockoutType::Degree, "Bachelor", UnknownPolicy::Fail),
        ko(KnockoutType::Degree, "PhD", UnknownPolicy::Pass),
        ko(KnockoutType::Certification, "CKA", UnknownPolicy::Fail),
        ko(KnockoutType::Clearance, "TS/SCI", UnknownPolicy::Review),
        ko(
            KnockoutType::Authorization,
            "US work authorization",
            UnknownPolicy::Pass,
        ),
    ]);
    let r = common::score_text(RESUME, &b);
    let got: Vec<(Status, Outcome)> = r.knockouts.iter().map(|k| (k.status, k.outcome)).collect();
    assert_eq!(
        got,
        [
            (Status::Met, Outcome::Pass),
            (Status::Unmet, Outcome::Fail),
            (Status::Met, Outcome::Pass),
            (Status::Unmet, Outcome::Fail),
            (Status::Met, Outcome::Pass),
            (Status::Unknown, Outcome::Review),
            (Status::Unknown, Outcome::Pass),
        ]
    );
    let zero =
        Floors::parse("workday=0, taleo=0, icims=0, greenhouse=0, lever=0, successfactors=0")
            .expect("floors");
    let v = gate(&r, &zero, true);
    assert!(!v.passed);
    assert_eq!(v.reasons.len(), 2, "{:?}", v.reasons);
    assert_eq!(v.review, ["Clearance TS/SCI"]);
}

#[test]
fn an_unknown_knockout_follows_its_policy() {
    // No dated role: years are unknown, so the policy decides.
    let undated = "EXPERIENCE\nSoftware Engineer, Acme\n- Built services in Go.\n";
    for (policy, outcome) in [
        (UnknownPolicy::Pass, Outcome::Pass),
        (UnknownPolicy::Fail, Outcome::Fail),
        (UnknownPolicy::Review, Outcome::Review),
    ] {
        let r = common::score_text(
            undated,
            &with_knockouts(vec![ko(KnockoutType::MinYears, "2", policy)]),
        );
        assert_eq!(
            (r.knockouts[0].status, r.knockouts[0].outcome),
            (Status::Unknown, outcome)
        );
    }
}

#[test]
fn an_authorization_the_resume_never_evidences_is_unknown_never_unmet() {
    // A posting-worded value names a requirement, not a phrase a resume would contain, so it
    // can only be met by that phrase or left to the policy; it never fails the gate on its own.
    for policy in [UnknownPolicy::Pass, UnknownPolicy::Review] {
        let r = common::score_text(
            RESUME,
            &with_knockouts(vec![ko(
                KnockoutType::Authorization,
                "No visa sponsorship",
                policy,
            )]),
        );
        assert_eq!(r.knockouts[0].status, Status::Unknown);
        assert_ne!(r.knockouts[0].outcome, Outcome::Fail);
    }
    let evidenced = "SUMMARY\nAuthorized to work in the US.\n".to_owned() + RESUME;
    let r = common::score_text(
        &evidenced,
        &with_knockouts(vec![ko(
            KnockoutType::Authorization,
            "authorized to work in the US",
            UnknownPolicy::Fail,
        )]),
    );
    assert_eq!(
        (r.knockouts[0].status, r.knockouts[0].outcome),
        (Status::Met, Outcome::Pass)
    );
}
