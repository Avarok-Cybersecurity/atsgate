//! Dates as the resume prints them: "Mar 2026", "Feb 2023 - Mar 2026", "Mar 2026 - Present",
//! or a bare year when that is all the career file knows. ASCII hyphen, spaces around it.

use atsgate_career::IsoDate;
use atsgate_resume::{DateEnd, DateRange, PartialDate};

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
pub(crate) const PRESENT: &str = "Present";
pub(crate) const RANGE_SEPARATOR: &str = " - ";

/// The dates of one entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dates {
    /// A start and an end; no end means the entry is current.
    Range {
        start: IsoDate,
        end: Option<IsoDate>,
    },
    /// A single date: a graduation, a publication, an award.
    Point(IsoDate),
}

fn parts(d: &IsoDate) -> PartialDate {
    let mut it = d.as_str().split('-');
    let year = it.next().and_then(|y| y.parse().ok());
    let month = it.next().and_then(|m| m.parse().ok());
    PartialDate {
        year: year.expect("IsoDate always starts with a four-digit year"),
        month,
    }
}

/// "Mar 2026", or "2020" when only the year is known.
pub fn month_year(d: &IsoDate) -> String {
    let p = parts(d);
    match p.month {
        Some(m) => format!("{} {}", MONTHS[usize::from(m) - 1], p.year),
        None => p.year.to_string(),
    }
}

impl Dates {
    /// `start` and `end` as a career entry stores them; an end alone is a point in time. A start
    /// alone is a current range ("- Present") when `open_is_current`, otherwise a point.
    pub fn of(
        start: Option<&IsoDate>,
        end: Option<&IsoDate>,
        open_is_current: bool,
    ) -> Option<Self> {
        match (start, end) {
            (Some(s), Some(e)) => Some(Self::Range {
                start: s.clone(),
                end: Some(e.clone()),
            }),
            (Some(s), None) if open_is_current => Some(Self::Range {
                start: s.clone(),
                end: None,
            }),
            (Some(s), None) => Some(Self::Point(s.clone())),
            (None, Some(e)) => Some(Self::Point(e.clone())),
            (None, None) => None,
        }
    }

    pub fn text(&self) -> String {
        match self {
            Self::Range { start, end } => {
                let end = end.as_ref().map_or_else(|| PRESENT.to_owned(), month_year);
                format!("{}{RANGE_SEPARATOR}{end}", month_year(start))
            }
            Self::Point(d) => month_year(d),
        }
    }

    /// What atsgate-resume should read back from [`Dates::text`].
    pub fn parsed(&self) -> DateRange {
        match self {
            Self::Range { start, end } => DateRange {
                start: parts(start),
                end: end
                    .as_ref()
                    .map_or(DateEnd::Present, |e| DateEnd::Date(parts(e))),
            },
            Self::Point(d) => DateRange {
                start: parts(d),
                end: DateEnd::Date(parts(d)),
            },
        }
    }

    /// The machine-readable start and end for HTML `<time>` and JSON-LD.
    pub fn iso(&self) -> (&str, Option<&str>) {
        match self {
            Self::Range { start, end } => (start.as_str(), end.as_ref().map(IsoDate::as_str)),
            Self::Point(d) => (d.as_str(), None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> IsoDate {
        IsoDate::try_from(s.to_owned()).expect("date")
    }

    #[test]
    fn formats_months_years_and_present() {
        let current = Dates::of(Some(&d("2026-03")), None, true).expect("dates");
        assert_eq!(current.text(), "Mar 2026 - Present");
        let past = Dates::of(Some(&d("2023-02")), Some(&d("2026-03-15")), true).expect("dates");
        assert_eq!(past.text(), "Feb 2023 - Mar 2026");
        assert_eq!(
            Dates::of(None, Some(&d("2020")), false).expect("d").text(),
            "2020"
        );
        assert_eq!(
            Dates::of(Some(&d("2023-04")), None, false)
                .expect("d")
                .text(),
            "Apr 2023"
        );
        assert_eq!(Dates::of(None, None, true), None);
    }
}
