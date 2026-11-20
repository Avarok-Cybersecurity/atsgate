//! The calendar month of a Unix time (UTC), so `main` can turn the clock into `--as-of today`.

use atsgate_score::YearMonth;

/// Howard Hinnant's `civil_from_days`, restricted to the year and month.
pub fn year_month(unix_seconds: u64) -> YearMonth {
    let days = (unix_seconds / 86_400) as i64 + 719_468;
    let era = days.div_euclid(146_097);
    let doe = days.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    YearMonth {
        year: year as u16,
        month: month as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_instants_map_to_their_month() {
        let ym = |year, month| YearMonth { year, month };
        assert_eq!(year_month(0), ym(1970, 1));
        assert_eq!(year_month(951_782_400), ym(2000, 2)); // 2000-02-29
        assert_eq!(year_month(951_868_799), ym(2000, 2)); // 2000-02-29 23:59:59
        assert_eq!(year_month(951_868_800), ym(2000, 3));
        assert_eq!(year_month(1_795_651_200), ym(2026, 11)); // 2026-11-26
        assert_eq!(year_month(1_798_761_599), ym(2026, 12)); // 2026-12-31 23:59:59
    }
}
