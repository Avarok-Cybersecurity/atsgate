//! ISO 8601 dates as JSON Resume writes them: `YYYY`, `YYYY-MM` or `YYYY-MM-DD`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const ISO_DATE_PATTERN: &str =
    r"^[1-2][0-9]{3}(-(0[1-9]|1[0-2])(-(0[1-9]|[12][0-9]|3[01]))?)?$";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "String", into = "String")]
#[schemars(extend("pattern" = ISO_DATE_PATTERN), description = "ISO 8601 date: YYYY, YYYY-MM or YYYY-MM-DD.")]
pub struct IsoDate(String);

fn digits(s: &str, n: usize) -> bool {
    s.len() == n && s.bytes().all(|b| b.is_ascii_digit())
}

impl TryFrom<String> for IsoDate {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split('-').collect();
        let in_range =
            |s: &str, lo: u8, hi: u8| s.parse::<u8>().is_ok_and(|v| (lo..=hi).contains(&v));
        let ok = match parts.as_slice() {
            [y] => digits(y, 4),
            [y, m] => digits(y, 4) && digits(m, 2) && in_range(m, 1, 12),
            [y, m, d] => {
                digits(y, 4)
                    && digits(m, 2)
                    && in_range(m, 1, 12)
                    && digits(d, 2)
                    && in_range(d, 1, 31)
            }
            _ => false,
        };
        let year_ok = parts[0].starts_with(['1', '2']);
        if ok && year_ok {
            Ok(Self(value))
        } else {
            Err(format!(
                "{value:?} is not an ISO 8601 date (YYYY, YYYY-MM or YYYY-MM-DD)"
            ))
        }
    }
}

impl From<IsoDate> for String {
    fn from(d: IsoDate) -> String {
        d.0
    }
}

impl IsoDate {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_three_precisions_and_rejects_the_rest() {
        for ok in ["2020", "2020-01", "2020-01-31"] {
            assert!(IsoDate::try_from(ok.to_owned()).is_ok(), "{ok}");
        }
        for bad in [
            "20",
            "2020-13",
            "2020-1",
            "Jan 2020",
            "2020-01-32",
            "3020",
            "",
        ] {
            assert!(IsoDate::try_from(bad.to_owned()).is_err(), "{bad}");
        }
    }
}
