//! What `atsgate score` and `atsgate gate` write: the report, the verdict, and which resume and
//! blueprint they belong to (`schemas/report.schema.json` is generated from this type).

use crate::gate::Verdict;
use crate::report::Report;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct ResumeInfo {
    pub path: String,
    pub format: String,
    /// SHA-256 of the exact bytes scored: with `atsgate gate`, the PDF that ships.
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[schemars(
    title = "ATSGate score report",
    description = "A simulated parse and keyword risk report: dimensions with evidence, vendor results and the gate verdict."
)]
pub struct ScoreOutput {
    pub atsgate_version: String,
    pub resume: ResumeInfo,
    /// The `--blueprint` argument as given (`builtin:<id>`, a path, a URL or `generate:<...>`).
    pub blueprint_spec: String,
    pub report: Report,
    pub verdict: Verdict,
}
