//! Date ranges as resumes write them: "Jan 2020 - Present", "06/2016 - 12/2019", "2012 - 2016",
//! "2016-06 to 2019-12", "Sept. 2019 - now", or a single date such as a graduation year.

/// A year with an optional month (1-12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PartialDate {
    pub year: u16,
    pub month: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateEnd {
    Date(PartialDate),
    Present,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateRange {
    pub start: PartialDate,
    pub end: DateEnd,
}

const MONTHS: [&str; 12] = [
    "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
];
const PRESENT: [&str; 5] = ["present", "current", "now", "today", "ongoing"];
const SEPARATORS: [&str; 4] = ["-", "to", "until", "till"];
const YEARS: std::ops::RangeInclusive<u16> = 1950..=2100;

fn month(word: &str) -> Option<u8> {
    let w = word.trim_end_matches('.').to_lowercase();
    if w.len() < 3 {
        return None;
    }
    let full = [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ];
    MONTHS
        .iter()
        .zip(full)
        .position(|(abbr, name)| name.starts_with(&w) && w.starts_with(abbr))
        .map(|i| i as u8 + 1)
}

fn year(word: &str) -> Option<u16> {
    let w = word.trim_end_matches([',', '.', ')']);
    (w.len() == 4)
        .then(|| w.parse().ok())
        .flatten()
        .filter(|y| YEARS.contains(y))
}

/// "06/2016", "6/2016" or "2016-06": a month and year in one token.
fn numeric_month(word: &str) -> Option<PartialDate> {
    let w = word.trim_end_matches([',', '.']);
    let (m, y) = match (w.split_once('/'), w.split_once('-')) {
        (Some((m, y)), _) => (m, y),
        (None, Some((y, m))) if y.len() == 4 && m.len() == 2 => (m, y),
        _ => return None,
    };
    let month: u8 = m.parse().ok().filter(|m| (1..=12).contains(m))?;
    Some(PartialDate {
        year: year(y)?,
        month: Some(month),
    })
}

/// Splits words, and splits dashes off everything but an ISO year-month ("2016-06").
fn tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for word in text.split_whitespace() {
        if numeric_month(word).is_some() {
            out.push(word.to_owned());
            continue;
        }
        for (i, part) in word.split('-').enumerate() {
            if i > 0 {
                out.push("-".to_owned());
            }
            if !part.is_empty() {
                out.push(part.to_owned());
            }
        }
    }
    out
}

/// A date starting at `t[i]`, with the number of tokens it used.
fn date_at(t: &[String], i: usize) -> Option<(PartialDate, usize)> {
    let word = t.get(i)?;
    if let (Some(m), Some(y)) = (month(word), t.get(i + 1).and_then(|w| year(w))) {
        return Some((
            PartialDate {
                year: y,
                month: Some(m),
            },
            2,
        ));
    }
    numeric_month(word).map(|d| (d, 1)).or_else(|| {
        year(word).map(|y| {
            (
                PartialDate {
                    year: y,
                    month: None,
                },
                1,
            )
        })
    })
}

fn end_at(t: &[String], i: usize) -> Option<(DateEnd, usize)> {
    let word = t.get(i)?.trim_end_matches(['.', ',', ')']).to_lowercase();
    if PRESENT.contains(&word.as_str()) {
        return Some((DateEnd::Present, 1));
    }
    date_at(t, i).map(|(d, n)| (DateEnd::Date(d), n))
}

/// The first date range in `text`, and the text with it removed.
pub fn find_range(text: &str) -> Option<(DateRange, String)> {
    let t = tokens(text);
    for i in 0..t.len() {
        let Some((start, used)) = date_at(&t, i) else {
            continue;
        };
        let sep = i + used;
        if !t
            .get(sep)
            .is_some_and(|s| SEPARATORS.contains(&s.to_lowercase().as_str()))
        {
            continue;
        }
        if let Some((end, used_end)) = end_at(&t, sep + 1) {
            let rest = [&t[..i], &t[sep + 1 + used_end..]].concat().join(" ");
            return Some((DateRange { start, end }, rest));
        }
    }
    None
}

/// A single trailing date ("2016", "May 2016") as a one-point range, for header lines only.
pub fn find_single(text: &str) -> Option<(DateRange, String)> {
    let t = tokens(text);
    (0..t.len()).rev().find_map(|i| {
        let (d, used) = date_at(&t, i).filter(|(_, used)| i + used == t.len())?;
        let _ = used;
        Some((
            DateRange {
                start: d,
                end: DateEnd::Date(d),
            },
            t[..i].join(" "),
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ym(year: u16, month: Option<u8>) -> PartialDate {
        PartialDate { year, month }
    }

    #[test]
    fn parses_common_range_formats() {
        let cases = [
            (
                "Engineer, Acme Jan 2020 - Present",
                ym(2020, Some(1)),
                DateEnd::Present,
                "Engineer, Acme",
            ),
            (
                "06/2016 - 12/2019",
                ym(2016, Some(6)),
                DateEnd::Date(ym(2019, Some(12))),
                "",
            ),
            (
                "2012-2016",
                ym(2012, None),
                DateEnd::Date(ym(2016, None)),
                "",
            ),
            (
                "2016-06 to 2019-12",
                ym(2016, Some(6)),
                DateEnd::Date(ym(2019, Some(12))),
                "",
            ),
            ("Sept. 2019 - now", ym(2019, Some(9)), DateEnd::Present, ""),
            (
                "March 2018 until May 2019 Lead",
                ym(2018, Some(3)),
                DateEnd::Date(ym(2019, Some(5))),
                "Lead",
            ),
        ];
        for (text, start, end, rest) in cases {
            let (range, left) = find_range(text).unwrap_or_else(|| panic!("no range in {text:?}"));
            assert_eq!(
                (range.start, range.end, left.as_str()),
                (start, end, rest),
                "{text}"
            );
        }
    }

    #[test]
    fn ignores_numbers_that_are_not_dates() {
        for text in [
            "Cut cost 30% - saved 2M",
            "Handled 2,000 - 3,000 requests",
            "Room 1200 - 1300",
            "May the best - win",
        ] {
            assert_eq!(find_range(text), None, "{text}");
        }
    }

    #[test]
    fn a_trailing_single_date_is_a_point() {
        let (range, rest) = find_single("B.S. Physics, State University 2016").expect("year");
        assert_eq!(range.start, ym(2016, None));
        assert_eq!(rest, "B.S. Physics, State University");
        assert_eq!(find_single("Mentored 4 engineers"), None);
    }
}
