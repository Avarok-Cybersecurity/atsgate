//! The options `score` and `gate` share.

use crate::options::Options;
use atsgate_score::YearMonth;
use std::path::PathBuf;

pub const SCORE_OPTIONS: [&str; 6] = [
    "--blueprint",
    "--floors",
    "--fail-on-integrity-flags",
    "--as-of",
    "--report",
    "--summary",
];

/// The month "Present" counts through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsOf {
    /// The current month, read from the clock by `main`.
    Today,
    Month(YearMonth),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreArgs {
    /// `builtin:<id>`, a path, an `https://` URL, or `generate:<path-or-url>`.
    pub blueprint: String,
    /// Floor overrides (`taleo=80, lever=0`); absent means the documented default floors.
    pub floors: String,
    pub fail_on_integrity_flags: bool,
    pub as_of: AsOf,
    pub report: PathBuf,
    pub summary: PathBuf,
}

pub fn parse(o: &mut Options) -> Result<ScoreArgs, String> {
    let fail_on_integrity_flags = match o.take("--fail-on-integrity-flags")?.as_str() {
        "true" => true,
        "false" => false,
        other => {
            return Err(format!(
                "--fail-on-integrity-flags must be true or false, not {other:?}"
            ));
        }
    };
    let as_of = match o.take("--as-of")?.as_str() {
        "today" => AsOf::Today,
        text => AsOf::Month(
            YearMonth::parse(text)
                .ok_or_else(|| format!("--as-of must be today or YYYY-MM, not {text:?}"))?,
        ),
    };
    Ok(ScoreArgs {
        blueprint: o.take("--blueprint")?,
        floors: o.optional("--floors").unwrap_or_default(),
        fail_on_integrity_flags,
        as_of,
        report: o.path("--report")?,
        summary: o.path("--summary")?,
    })
}

#[cfg(test)]
mod tests {
    use crate::args::{Command, parse};
    use crate::score_args::AsOf;

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_owned).collect()
    }

    const SCORE: &str = "score --resume r.pdf --blueprint builtin:accountant \
        --fail-on-integrity-flags true --as-of 2026-11 --report o.json --summary o.md";

    #[test]
    fn score_and_gate_parse_with_every_required_option() {
        let Ok(Command::Score { resume, score }) = parse(&args(SCORE)) else {
            panic!("expected score");
        };
        assert_eq!(resume, std::path::PathBuf::from("r.pdf"));
        assert!(score.fail_on_integrity_flags && score.floors.is_empty());
        assert!(matches!(score.as_of, AsOf::Month(m) if m.year == 2026 && m.month == 11));
        let gate = "gate --career c --tailoring t --contact public --out o --blueprint builtin:x \
            --floors taleo=80 --fail-on-integrity-flags false --as-of today --report r --summary s";
        let Ok(Command::Gate { score, .. }) = parse(&args(gate)) else {
            panic!("expected gate");
        };
        assert_eq!(
            (score.floors.as_str(), score.as_of),
            ("taleo=80", AsOf::Today)
        );
    }

    #[test]
    fn scoring_options_have_no_defaults_to_discover() {
        for (drop, replace) in [
            ("--fail-on-integrity-flags true", ""),
            (
                "--fail-on-integrity-flags true",
                "--fail-on-integrity-flags yes",
            ),
            ("--as-of 2026-11", ""),
            ("--as-of 2026-11", "--as-of 2026-13"),
            ("--summary o.md", ""),
            ("--blueprint builtin:accountant", ""),
            ("--report o.json", "--report o.json --career c"),
        ] {
            let line = SCORE.replace(drop, replace);
            assert!(parse(&args(&line)).is_err(), "{line}");
        }
    }
}
