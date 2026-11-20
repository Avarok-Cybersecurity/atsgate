//! Floor overrides and the gate's comparison at the floor.

mod common;

use atsgate_extract::Format;
use atsgate_score::{Floors, FloorsError, Vendor, gate, meets_floor};

const RESUME: &[u8] = include_bytes!("../../../fixtures/pdf/one-column.pdf");

#[test]
fn defaults_are_the_owner_floors() {
    let f = Floors::defaults();
    let got: Vec<(Vendor, u8)> = [
        Vendor::Taleo,
        Vendor::Workday,
        Vendor::SuccessFactors,
        Vendor::Icims,
        Vendor::Greenhouse,
        Vendor::Lever,
    ]
    .map(|v| (v, f.get(v)))
    .to_vec();
    assert_eq!(
        got,
        [
            (Vendor::Taleo, 75),
            (Vendor::Workday, 70),
            (Vendor::SuccessFactors, 65),
            (Vendor::Icims, 60),
            (Vendor::Greenhouse, 50),
            (Vendor::Lever, 50)
        ]
    );
    assert_eq!(Floors::parse("").expect("empty"), f);
}

#[test]
fn overrides_replace_only_the_vendors_they_name() {
    let f = Floors::parse("taleo=80, lever=0").expect("valid");
    assert_eq!(f.get(Vendor::Taleo), 80);
    assert_eq!(f.get(Vendor::Lever), 0);
    assert_eq!(f.get(Vendor::Workday), 70);
    let spaced = Floors::parse(" Workday = 100 ,\nicims=1 ").expect("whitespace and case");
    assert_eq!(
        (spaced.get(Vendor::Workday), spaced.get(Vendor::Icims)),
        (100, 1)
    );
}

#[test]
fn bad_overrides_are_errors() {
    assert_eq!(
        Floors::parse("bamboohr=60"),
        Err(FloorsError::UnknownVendor("bamboohr".into()))
    );
    assert_eq!(
        Floors::parse("taleo"),
        Err(FloorsError::Malformed("taleo".into()))
    );
    for value in ["101", "-1", "7.5", "high", ""] {
        assert!(
            matches!(
                Floors::parse(&format!("lever={value}")),
                Err(FloorsError::BadValue { .. })
            ),
            "{value:?}"
        );
    }
    assert_eq!(
        Floors::parse("lever=1, LEVER=2"),
        Err(FloorsError::Repeated("lever".into()))
    );
}

#[test]
fn a_score_exactly_at_the_floor_passes_and_one_tenth_below_fails() {
    assert!(meets_floor(70.0, 70));
    assert!(!meets_floor(69.9, 70));
    assert!(meets_floor(0.0, 0));
    assert!(meets_floor(100.0, 100));
}

#[test]
fn the_gate_compares_each_vendor_with_its_own_floor() {
    let mut r = common::score_bytes(
        Format::Pdf,
        RESUME,
        &common::blueprint("software-engineer-backend"),
    );
    let workday = r
        .vendors
        .iter()
        .position(|v| v.vendor == Vendor::Workday)
        .expect("workday");
    let everyone_else = "taleo=0, icims=0, greenhouse=0, lever=0, successfactors=0";
    let floors = Floors::parse(&format!("workday=70, {everyone_else}")).expect("valid");
    r.vendors[workday].score = 70.0;
    let at = gate(&r, &floors, true);
    assert!(at.passed, "{:?}", at.reasons);
    r.vendors[workday].score = 69.9;
    let below = gate(&r, &floors, true);
    assert!(!below.passed);
    assert_eq!(below.reasons, ["Workday 69.9 is below its floor 70"]);
    assert_eq!(
        below.min_vendor_score,
        r.vendors
            .iter()
            .map(|v| v.score)
            .fold(f64::INFINITY, f64::min)
    );
}
