//! Experience E = min(1, relevant years / min_years).
//!
//! Relevant years are the **union** of the dated intervals of relevant roles: overlapping roles
//! count once, and the time between roles is never subtracted. A gap therefore never changes the
//! score. A role is relevant when its title names the blueprint's occupation or its text
//! evidences one of the blueprint's skills.
//!
//! Months: a year written without a month starts in January and ends at the start of that year
//! (so "2012 - 2016" is 48 months, never more than the text proves). A month-dated end counts
//! through that month. "Present" counts through the `as_of` month, which the caller supplies:
//! the core never reads a clock.

use crate::evidence::Evidence;
use crate::units::EntryId;
use atsgate_extract::Document;
use atsgate_resume::{DateEnd, DateRange, PartialDate, Resume, SectionKind};
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::BTreeSet;

/// A calendar month, the resolution of every date the scorer reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, JsonSchema)]
pub struct YearMonth {
    pub year: u16,
    /// 1-12.
    pub month: u8,
}

impl YearMonth {
    /// Parses `YYYY-MM`.
    pub fn parse(text: &str) -> Option<Self> {
        let (y, m) = text.split_once('-')?;
        let (year, month) = (y.parse().ok()?, m.parse().ok()?);
        (y.len() == 4 && m.len() == 2 && (1..=12).contains(&month)).then_some(Self { year, month })
    }
    fn index(self) -> i64 {
        i64::from(self.year) * 12 + i64::from(self.month) - 1
    }
}

impl std::fmt::Display for YearMonth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}", self.year, self.month)
    }
}

fn start_index(d: PartialDate) -> i64 {
    i64::from(d.year) * 12 + i64::from(d.month.unwrap_or(1)) - 1
}

fn end_index(d: PartialDate) -> i64 {
    match d.month {
        Some(m) => i64::from(d.year) * 12 + i64::from(m),
        None => i64::from(d.year) * 12,
    }
}

/// The half-open month interval `[start, end)` of a range, clipped to `as_of`.
pub fn interval(range: DateRange, as_of: YearMonth) -> Option<(i64, i64)> {
    let limit = as_of.index() + 1;
    let start = start_index(range.start);
    let end = match range.end {
        DateEnd::Present => limit,
        DateEnd::Date(d) => end_index(d).min(limit),
    };
    (start < end).then_some((start, end))
}

/// Total months covered by the union of `intervals`.
pub fn union_months(mut intervals: Vec<(i64, i64)>) -> i64 {
    intervals.sort_unstable();
    let mut total = 0;
    let mut current: Option<(i64, i64)> = None;
    for (s, e) in intervals {
        current = match current {
            Some((cs, ce)) if s <= ce => Some((cs, ce.max(e))),
            Some((cs, ce)) => {
                total += ce - cs;
                Some((s, e))
            }
            None => Some((s, e)),
        };
    }
    total + current.map_or(0, |(s, e)| e - s)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExperienceScore {
    pub score: f64,
    pub relevant_years: f64,
    /// Roles with dates at all, relevant or not: a knockout on years is unknown without them.
    pub dated_roles: usize,
    pub evidence: Vec<Evidence>,
}

pub fn score(
    doc: &Document,
    resume: &Resume,
    relevant: &BTreeSet<EntryId>,
    min_years: f64,
    as_of: YearMonth,
) -> ExperienceScore {
    let mut intervals = Vec::new();
    let mut evidence = Vec::new();
    let mut dated_roles = 0;
    for (si, section) in resume.sections.iter().enumerate() {
        if section.kind != SectionKind::Experience {
            continue;
        }
        for (ei, entry) in section.entries.iter().enumerate() {
            let Some(range) = entry.dates.and_then(|r| interval(r, as_of).map(|i| (r, i))) else {
                continue;
            };
            dated_roles += 1;
            if !relevant.contains(&(si, ei)) {
                continue;
            }
            intervals.push(range.1);
            let text = format!("{} ({} months)", entry.title, range.1.1 - range.1.0);
            evidence.push(match entry.evidence.first() {
                Some(at) => Evidence::at_line("experience.interval", doc, at, text),
                None => Evidence::note("experience.interval", text),
            });
        }
    }
    let relevant_years = union_months(intervals) as f64 / 12.0;
    let score = if min_years > 0.0 {
        (relevant_years / min_years).min(1.0)
    } else {
        evidence.push(Evidence::note(
            "experience.no_minimum",
            "the blueprint sets no minimum years",
        ));
        1.0
    };
    ExperienceScore {
        score,
        relevant_years,
        dated_roles,
        evidence,
    }
}
