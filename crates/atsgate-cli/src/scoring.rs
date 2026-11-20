//! `score` and the scoring half of `gate`: resolve the blueprint, extract and score the resume
//! bytes, apply the gate, and write the JSON report, the markdown summary and, in GitHub
//! Actions, the job summary.

use crate::io::Io;
use crate::run::write;
use crate::score_args::{AsOf, ScoreArgs};
use crate::spec;
use atsgate_extract::{Format, Thresholds, extract};
use atsgate_score::{Floors, Input, ResumeInfo, ScoreOutput, YearMonth, gate, markdown, score};
use atsgate_taxonomy::{Gazetteer, Taxonomy};
use std::io::Write as _;
use std::path::{Path, PathBuf};

/// Where the scoring step runs; built by `main` from the clock and the environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub today: YearMonth,
    /// `GITHUB_STEP_SUMMARY`, when set.
    pub step_summary: Option<PathBuf>,
}

/// The format of a resume file, from its extension. Anything else is an error.
pub fn resume_format(path: &Path) -> Result<Format, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    match ext.as_deref() {
        Some("pdf") => Ok(Format::Pdf),
        Some("docx") => Ok(Format::Docx),
        Some("html" | "htm") => Ok(Format::Html),
        Some("md" | "markdown") => Ok(Format::Markdown),
        Some("txt") => Ok(Format::Text),
        _ => Err(format!(
            "{}: the resume must be .pdf, .docx, .html, .md or .txt",
            path.display()
        )),
    }
}

/// The file a generated blueprint is written to: beside the report.
pub fn generated_path(report: &Path) -> PathBuf {
    report.with_file_name("generated-blueprint.yaml")
}

pub struct Scored {
    pub lines: Vec<String>,
    pub passed: bool,
    pub outputs: Vec<(String, String)>,
}

/// The compact `score-json` output: the verdict and each vendor's score and floor.
fn score_json(v: &atsgate_score::Verdict) -> Result<String, String> {
    serde_json::to_string(v).map_err(|e| e.to_string())
}

pub fn run(
    bytes: &[u8],
    path: &Path,
    args: &ScoreArgs,
    io: &dyn Io,
    ctx: &Context,
) -> Result<Scored, String> {
    let floors = Floors::parse(&args.floors).map_err(|e| format!("--floors: {e}"))?;
    let format = resume_format(path)?;
    let taxonomy = Taxonomy::builtin().map_err(|e| e.to_string())?;
    let gazetteer = Gazetteer::skills(taxonomy);
    let resolved = spec::resolve(&args.blueprint, io, &gazetteer)?;
    let document = extract(bytes, format, &Thresholds::STANDARD)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let as_of = match args.as_of {
        AsOf::Today => ctx.today,
        AsOf::Month(m) => m,
    };
    let report = score(&Input {
        document: &document,
        blueprint: &resolved.blueprint,
        gazetteer: &gazetteer,
        as_of,
    });
    let verdict = gate(&report, &floors, args.fail_on_integrity_flags);
    let output = ScoreOutput {
        atsgate_version: env!("CARGO_PKG_VERSION").to_owned(),
        resume: ResumeInfo {
            path: path.display().to_string(),
            format: format!("{format:?}").to_lowercase(),
            sha256: atsgate_render::sha256_hex(bytes),
        },
        blueprint_spec: args.blueprint.clone(),
        report,
        verdict,
    };
    let mut lines = Vec::new();
    if let Some(yaml) = &resolved.generated_yaml {
        let target = generated_path(&args.report);
        write(&target, yaml.as_bytes())?;
        lines.push(format!(
            "wrote {} (generated, reviewed: false)",
            target.display()
        ));
    }
    let mut json = serde_json::to_string_pretty(&output).map_err(|e| e.to_string())?;
    json.push('\n');
    write(&args.report, json.as_bytes())?;
    let summary = markdown::summary(&output);
    write(&args.summary, summary.as_bytes())?;
    lines.push(format!("wrote {}", args.report.display()));
    lines.push(format!("wrote {}", args.summary.display()));
    if let Some(step) = &ctx.step_summary {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(step)
            .map_err(|e| format!("{}: {e}", step.display()))?;
        file.write_all(summary.as_bytes())
            .map_err(|e| format!("{}: {e}", step.display()))?;
    }
    let v = &output.verdict;
    lines.push(format!("min-vendor-score={:.1}", v.min_vendor_score));
    lines.push(format!("passed={}", v.passed));
    lines.extend(v.reasons.iter().map(|r| format!("  {r}")));
    let outputs = vec![
        ("score-json".to_owned(), score_json(v)?),
        (
            "min-vendor-score".to_owned(),
            format!("{:.1}", v.min_vendor_score),
        ),
        ("passed".to_owned(), v.passed.to_string()),
        ("report-json".to_owned(), args.report.display().to_string()),
        ("resume-sha256".to_owned(), output.resume.sha256.clone()),
    ];
    Ok(Scored {
        lines,
        passed: v.passed,
        outputs,
    })
}
