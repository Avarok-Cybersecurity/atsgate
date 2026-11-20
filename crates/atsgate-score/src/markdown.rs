//! The markdown summary: verdict, vendor table with floors, dimensions, knockouts, the top
//! missing required skills with the spellings that would match, parse risks and integrity flags.

use crate::output::ScoreOutput;
use crate::skills::Requirement;
use std::fmt::Write;

/// How many missing skills and evidence lines the summary lists before it says "and N more".
pub const TOP: usize = 10;

fn pct(x: Option<f64>) -> String {
    x.map_or("n/a".to_owned(), |v| format!("{:.0}%", v * 100.0))
}

fn cell(text: &str) -> String {
    text.replace('|', "\\|").replace('\n', " ")
}

pub fn summary(o: &ScoreOutput) -> String {
    let (r, v) = (&o.report, &o.verdict);
    let mut s = String::new();
    let status = if v.passed { "passed" } else { "failed" };
    let _ = writeln!(s, "## ATSGate: {status}\n");
    let _ = writeln!(s, "_{}; no vendor publishes its scoring._\n", r.label);
    let reviewed = if r.blueprint.reviewed {
        ""
    } else {
        " — **reviewed: false**"
    };
    let _ = writeln!(
        s,
        "Blueprint `{}` {} ({:?}){reviewed}. Resume `{}` sha256 `{}`. As of {}.\n",
        r.blueprint.id,
        r.blueprint.version,
        r.blueprint.source,
        o.resume.path,
        o.resume.sha256,
        r.as_of
    );
    for reason in &v.reasons {
        let _ = writeln!(s, "- {}", cell(reason));
    }
    if !v.reasons.is_empty() {
        s.push('\n');
    }
    let _ = writeln!(
        s,
        "| Vendor | Score | Floor | Result | F | Note |\n|---|---|---|---|---|---|"
    );
    for (vv, vr) in v.vendors.iter().zip(&r.vendors) {
        let result = if vv.passed { "pass" } else { "below floor" };
        let note = vr.note.as_deref().unwrap_or("");
        let _ = writeln!(
            s,
            "| [{}]({}) | {:.1} | {} | {result} | {:.1} | {note} |",
            vv.name, vr.citation.url, vv.score, vv.floor, vr.parse_score
        );
    }
    let d = &r.dimensions;
    let _ = writeln!(s, "\n| Dimension | Score |\n|---|---|");
    for (name, x) in [
        ("R required skills", d.required_skills.score),
        ("P preferred skills", d.preferred_skills.score),
        ("T title and seniority", d.title.score),
        ("E experience", d.experience.score),
        ("D education", d.education.score),
    ] {
        let _ = writeln!(s, "| {name} | {} |", pct(x));
    }
    let _ = writeln!(
        s,
        "| Q quantification ({}) | {} |",
        r.quantification.label,
        pct(r.quantification.score)
    );
    let _ = writeln!(
        s,
        "\nRelevant experience: {:.1} years (gaps are never counted against it).",
        r.relevant_years
    );
    if !r.knockouts.is_empty() {
        let _ = writeln!(s, "\n| Knockout | Status | Outcome |\n|---|---|---|");
        for k in &r.knockouts {
            let _ = writeln!(
                s,
                "| {:?} {} | {:?} | {:?} |",
                k.kind,
                cell(&k.value),
                k.status,
                k.outcome
            );
        }
    }
    let missing: Vec<_> = r
        .gaps
        .iter()
        .filter(|g| g.requirement == Requirement::Required)
        .collect();
    if !missing.is_empty() {
        let _ = writeln!(
            s,
            "\n**Missing required skills** (would match as written):\n"
        );
        for g in missing.iter().take(TOP) {
            let _ = writeln!(
                s,
                "- {} (weight {}): {}",
                g.canonical,
                g.weight,
                g.would_match.join(", ")
            );
        }
        if missing.len() > TOP {
            let _ = writeln!(s, "- and {} more", missing.len() - TOP);
        }
    }
    for (title, items) in [
        (
            "Parse risks",
            r.parse_penalties
                .iter()
                .map(|p| (&p.rule, p.points, &p.evidence))
                .collect::<Vec<_>>(),
        ),
        (
            "Integrity flags",
            r.integrity_flags
                .iter()
                .map(|f| (&f.rule, 0.0, &f.evidence))
                .collect(),
        ),
    ] {
        if items.is_empty() {
            continue;
        }
        let _ = writeln!(s, "\n**{title}:**\n");
        for (rule, points, evidence) in items {
            let first = evidence.first().map_or(String::new(), |e| {
                let page = e.page.map_or(String::new(), |p| format!(" (page {p})"));
                format!(": \"{}\"{page}", cell(&e.text))
            });
            let charge = if points > 0.0 {
                format!(" −{points} × σ")
            } else {
                String::new()
            };
            let _ = writeln!(s, "- `{rule}`{charge}{first}");
        }
    }
    s
}
