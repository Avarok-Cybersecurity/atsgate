<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="brand/logo-dark.svg">
    <img src="brand/logo-light.svg" alt="ATSGate logo: a gate with a check mark passing through it" width="96" height="96">
  </picture>
</p>

<h1 align="center">ATSGate</h1>

<p align="center">A deterministic, evidence-carrying resume scorer in pure Rust, shipped as a GitHub Action.<br>
Docs: <a href="https://ats.express">ats.express</a> (coming with v0.1)</p>

## Contents

- [What it is](#what-it-is)
- [Quick start](#quick-start)
- [GitHub Action](#github-action)
- [Formats](#formats)
- [Schemas](#schemas)
- [Rendering](#rendering)
- [Blueprints and taxonomy](#blueprints-and-taxonomy)
- [Test fixtures](#test-fixtures)
- [Scoring](#scoring)
- [Fairness](#fairness)
- [Citations](#citations)
- [License](#license)

## What it is

ATSGate reads a resume (PDF, DOCX, HTML, Markdown or plain text) and scores it against a
standardized **job blueprint**. It runs in CI, so a resume that is generated from a career file
can be gated on parse risk and keyword coverage before it ships.

- **Deterministic.** The same input always gives the same score. No language model sits in the
  scoring path.
- **Evidence-carrying.** Every point names the rule that awarded it and the page, bounding box and
  text span it came from.
- **Honest about vendors.** Real applicant tracking systems mostly do not auto-score
  ([gh-tm-faq](#cite-gh-tm-faq), [ats-screener-research](#cite-ats-screener-research)). ATSGate
  reports a *simulated parse and keyword risk* per vendor, never a vendor score.

The crates are pure: they take bytes and return values. Only `atsgate-cli` reads files or the
network.

| Crate | Role | Status |
|---|---|---|
| `atsgate-extract` | Bytes to a `Document`: spans, bounding boxes, reading order, layout and hidden-text signals | Phase 2 |
| `atsgate-resume` | `Document` to `Resume`: sections, dated roles and bullets with evidence spans | Phase 2 |
| `atsgate-career` | The career SSOT format (JSON Resume plus `x-atsgate`) | Phase 2b |
| `atsgate-blueprint` | The job blueprint schema, built-ins and generator | Phase 2b / 3 |
| `atsgate-taxonomy` | Skill and title alias tables from O*NET and ESCO | Phase 3 |
| `atsgate-score` | Resume x blueprint to an evidence-carrying report, six simulated vendor scores and the gate | Phase 4 |
| `atsgate-render` | Career SSOT and tailoring file to resume PDF/DOCX, site partials and JSON-LD, each PDF and DOCX checked by a parse round trip | Phase 2b |
| `atsgate-cli` | The `atsgate` command: the only crate that performs I/O (`import`, `build`, `inject`, `score`, `gate`) | Phase 2b / 4 |

## Quick start

Render a resume from a career file, score exactly the PDF that was rendered, and fail the job if
any simulated vendor falls below its floor:

```yaml
- uses: Avarok-Cybersecurity/atsgate@<release tag or commit> # see "Release binaries" below
  id: atsgate
  with:
    mode: gate
    career: career.json
    tailoring: tailoring/general.yaml
    contact: public
    out: dist
    blueprint: builtin:software-engineer-backend
    fail-on-integrity-flags: "true"
    report: atsgate/report.json
- uses: actions/upload-artifact@<sha>
  if: always()
  with:
    name: resume
    path: |
      dist/
      atsgate/
```

The same from a shell:

```sh
atsgate gate --career career.json --tailoring tailoring/general.yaml --contact public --out dist \
  --blueprint builtin:software-engineer-backend --fail-on-integrity-flags true \
  --as-of today --report atsgate/report.json --summary atsgate/report.md
atsgate score --resume my-resume.pdf --blueprint generate:posting.txt --floors "taleo=80" \
  --fail-on-integrity-flags true --as-of today --report report.json --summary report.md
```

Exit status: 0 when the gate passes, 1 when it fails (the report is still written) or on an
error, 2 on a usage error.

## GitHub Action

| Input | Modes | Required | Meaning |
|---|---|---|---|
| `mode` | all | yes, no default | `build` (render only), `score` (score an existing resume) or `gate` (build, then score the PDF it rendered) |
| `resume` | score | yes | The PDF or DOCX to score |
| `career`, `tailoring` | build, gate | yes | The career JSON and the tailoring YAML |
| `contact` | build, gate | yes | `public`, or a private contact JSON file |
| `out` | build, gate | yes | Output directory |
| `allow-private-contact` | build, gate | no (`false`) | A private contact file is refused in CI unless this is `true` |
| `blueprint` | score, gate | yes | `builtin:ID`, a blueprint file, an `https://` URL, or `generate:FILE-OR-URL` (a job posting; the generated blueprint is written beside the report and marked `reviewed: false`) |
| `floors` | score, gate | no | Overrides such as `taleo=80, lever=0`; vendors not named keep the default floors below |
| `fail-on-integrity-flags` | score, gate | yes, no default | `true` or `false`: whether hidden or metadata-only text fails the gate |
| `report` | score, gate | yes | The JSON report path, ending in `.json`; the markdown summary goes beside it as `.md` and is appended to the job summary |

| Output | Meaning |
|---|---|
| `passed` | `true` or `false` |
| `min-vendor-score` | The lowest of the six vendor scores |
| `score-json` | The verdict as compact JSON: reasons, and each vendor's score and floor |
| `report-json` | The report path |
| `resume-sha256` | The sha256 of the exact file that was scored |
| `resume-pdf`, `resume-docx`, `partials-dir` | What `build` and `gate` rendered |

"Present" counts through the month the job runs in.

**Release binaries.** Outside this repository the action downloads the `atsgate` binary for the
runner (Linux x86_64 and arm64, macOS arm64, Windows x86_64) from the GitHub release named in
[`action/release-pins.txt`](action/release-pins.txt) and refuses it unless its sha256 matches the
pin. Each binary is built by [`release.yml`](.github/workflows/release.yml) on a `v*` tag with a
build-provenance attestation (`gh attestation verify <file> --repo Avarok-Cybersecurity/atsgate`).
A revision with no pins refuses to run rather than fetch an unverified binary. Within this
repository (`uses: ./`) the action builds from source, which is how CI tests it.

## Formats

| Input | Reader | What it detects |
|---|---|---|
| PDF | `pdf_oxide` (positioned glyphs) and `lopdf` (content-stream order, rulings, images, fonts, ToUnicode, metadata); reading order is ATSGate's own XY-cut | Columns, out-of-flow text boxes, tables, header and footer content, image-only pages, unmapped glyphs and ligatures, white, tiny, zero-width, off-page and metadata-only text |
| DOCX | `zip` and `quick-xml` | Tables, text boxes, header and footer content, hidden (`w:vanish`), white and tiny runs, zero-width characters, metadata-only text |
| HTML | `scraper` | Text in document order |
| Markdown | `pulldown-cmark` | Text in document order |
| Plain text | built in | Text |

## Schemas

| File | What it describes |
|---|---|
| [`schemas/career.schema.json`](schemas/career.schema.json) | The career SSOT: a JSON Resume document plus an `x-atsgate` block (sources, statistics with a required source, families, variants) |
| [`schemas/blueprint.schema.json`](schemas/blueprint.schema.json) | A job blueprint, aligned with schema.org JobPosting. Every key is required; a value that does not apply is `null` or `[]` |
| [`schemas/tailoring.schema.json`](schemas/tailoring.schema.json) | A tailoring file (`tailoring/*.yaml`): which career entries, highlights, statistics and variants one resume shows, in order. It only selects, orders and chooses; it has no field that could carry a new fact |
| [`schemas/contact.schema.json`](schemas/contact.schema.json) | A private contact file: the phone and street that never live in the career file |
| [`schemas/report.schema.json`](schemas/report.schema.json) | The JSON report of `score` and `gate`: the resume's sha256, every dimension with its evidence, the six vendor scores, knockouts, integrity flags and the verdict |

The schemas are generated from the Rust types in `atsgate-career`, `atsgate-blueprint` and `atsgate-score` (`cargo run -p xtask -- schemas --write`), so the schema and the parser cannot drift: CI fails if the committed copy differs. Unknown fields are errors in all of them.

## Rendering

```sh
# Convert a JSON Resume (or the tbraun96 site format) into a career file; every field is mapped.
atsgate import --from json-resume --input resume.json --out career.json

# Render one tailoring of it. --contact is `public` or a private contact file.
atsgate build --career career.json --tailoring tailoring/general.yaml --contact public --out dist

# Fill <!-- atsgate:NAME --> ... <!-- /atsgate:NAME --> markers in a page with the partials.
# (score and gate: see Quick start.)
atsgate inject --partials dist/partials --page site/index.html --out site/index.html
```

`build` writes `resume.pdf`, `resume.docx`, `partials/{experience,projects,skills,education}.html`, `partials/jsonld.json` (schema.org `Person` with `hasOccupation`, `worksFor` and `alumniOf`) and `manifest.json` with the SHA-256 of each file. Output is byte-for-byte reproducible.

- **One column, real text.** The PDF is typeset by [Typst](https://typst.app) embedded as a library, with one built-in template: standard headings, contact details in the body, no tables, dates as `Mon YYYY - Mon YYYY` or `Mon YYYY - Present`. Career text reaches the template only as data, never as markup.
- **Round trip.** Before anything is written, the PDF and the DOCX are read back through `atsgate-extract` and `atsgate-resume`. Every section, entry, date, bullet and skill must come back in order, with no integrity flag, no layout-risk signal and no unrecognized heading, or the build fails.
- **Contact profiles.** `public` prints city and region, email, website and identity profiles. A private contact file adds the phone and street. A phone, street or postal code in the career file itself is an error. In CI (`GITHUB_ACTIONS=true`) a private contact file is refused unless `--allow-private-contact` is passed.

## Blueprints and taxonomy

`atsgate-taxonomy` embeds O*NET 31.0 and ESCO v1.2.1 (about 2.9 MB compressed, against a 4 MiB
budget) plus a curated alias table, [`curated.yaml`](crates/atsgate-taxonomy/data/curated.yaml).
A lookup returns the canonical name, the taxonomy reference and how it matched (`exact`, `alias`,
`broader` or `narrower`). `cargo run -p xtask -- taxonomy --write` rebuilds the data from
downloads pinned by sha256, and `--verify` checks the committed data offline. Terms and
attribution: [DATA-LICENSES.md](DATA-LICENSES.md).

Ten reviewed built-in blueprints live in [`blueprints/`](blueprints) (`builtin:<id>`):
`software-engineer-backend`, `software-engineer-systems-rust`, `ml-inference-engineer`,
`software-engineer-frontend`, `data-scientist`, `product-designer`, `marketing-manager`,
`sales-representative`, `accountant` and `paralegal`. Every skill names a taxonomy concept, and
a test fails if any stops resolving.

`atsgate_blueprint::generate` turns a job description into a candidate blueprint. It is
deterministic: section cues decide required versus preferred, and it also reads years, degrees,
certifications and the seniority in the title, writing `seniority: unspecified` when the posting
states none. In a posting that lists several roles (Hacker News "Who's Hiring") it takes the first
listed role that names a title, and it reads each item of a `Stack:` line as a skill whatever its
case. Its gazetteer reports less of ESCO than the scorer's (multi-word preferred labels, curated
aliases and an allow-list; no transversal or attitude skills), because every mention becomes a
requirement. Its output is always `reviewed: false`.
[`blueprints/examples/artcraft-senior-engineer.yaml`](blueprints/examples/artcraft-senior-engineer.yaml)
is a worked example from a real posting. Measured quality, and its limits:
[docs/evaluation.md](docs/evaluation.md).

## Test fixtures

The resumes under [`fixtures/`](fixtures) are generated by `cargo run -p xtask -- fixtures --write` from one invented person. The generator writes the PDFs directly with `lopdf` and the DOCX files as stored zip packages, so each layout (one column, two columns, table, text box, header and footer, image only) and each integrity variant (white, tiny, hidden, off-page, zero-width, ligature, missing ToUnicode, metadata-only text) is exactly what its name says. Output is byte-for-byte deterministic; CI runs `fixtures --check` and needs no external tool.

## Scoring

Text a reader cannot see (white, tiny, off-page, zero-width or hidden runs) is removed before the
resume is parsed, so it earns nothing; it is reported as an **integrity flag**. Metadata text
absent from the page is flagged only when it names a blueprint skill or domain keyword, or runs to
12 or more terms: an ordinary document title is not concealment. Header and footer content is a
layout signal only when the same text repeats in the top or bottom band of another page. Every number in
the report names its rule and the page, bounding box and text it came from.

| Dimension | Formula |
|---|---|
| R, P: required and preferred skills | Σ w·m·c / Σ w, each skill counted once at its best occurrence. m = 1 canonical name, 0.85 taxonomy alias, 0.5 broader or narrower concept. c = 1 where the blueprint's `evidence_scope` asks for it; 0.8 for a skill an `experience_or_projects` blueprint finds only in the skills list or summary |
| T: title | The best title match (1 the same words, 0.85 a title of the blueprint's O*NET occupation, 0.5 one in the same SOC broad group) × (1 − k · \|seniority steps apart\| / 9), k = 1 below the blueprint's seniority and 0.5 above it; no seniority factor when the blueprint says `unspecified`. Technology qualifiers ("Rust Software Engineer") are dropped before matching, and an ambiguous title ("Engineer", "Founding Engineer") is read in the blueprint's occupation |
| E: years | min(1, relevant years / the blueprint's minimum), years being the union of relevant dated roles (overlaps once, **gaps never subtract**). A role is relevant when its title matches or it evidences any required or preferred skill |
| D: education | Level and field; institutions and dates are never read |
| F: parse integrity | max(0, 100 − Σ penalty · σ), penalties per layout signal (image-only page 100, columns 15, tables 12, header or footer content 10, and so on; see `parse.rs`) |
| Q: quantification | Share of bullets with a number. Reported as a heuristic with no outcome evidence; never scored |
| Knockouts | Met, unmet or unknown, never points. Unmet fails the gate unless the blueprint says `review`. Phrase knockouts (certification, clearance, authorization, language) are met when the resume states `value` and otherwise unknown, never unmet, so a work-authorization knockout the resume says nothing about resolves by its `unknown_policy` |

Each vendor's score is 100 · Σ weight · dimension over the dimensions the blueprint uses,
renormalized, rounded to 0.1. The weights and parser strictness σ are adapted from
[ats-screener](#cite-ats-screener-method) (MIT; see [DATA-LICENSES.md](DATA-LICENSES.md)), and
the default floors are its `thresholds.md` values, because its own profiles disagree with them:

| Vendor | σ | Default floor | Note |
|---|---|---|---|
| Taleo | 0.85 | 75 | |
| Workday | 0.9 | 70 | |
| SuccessFactors | 0.85 | 65 | |
| iCIMS | 0.6 | 60 | |
| Greenhouse | 0.4 | 50 | no documented auto-scoring |
| Lever | 0.35 | 50 | no documented auto-scoring |

**Every vendor result is a simulated parse and keyword risk**, not a vendor's score: no vendor
publishes its scoring. The gate fails when any vendor is below its floor (a score equal to the
floor passes), any knockout fails, or, with `fail-on-integrity-flags: true`, any integrity flag is
raised. The weights live in [`vendor.rs`](crates/atsgate-score/src/vendor.rs); a test keeps the
table above equal to them.

## Fairness

LLM screeners carry name, school and gap bias ([faire](#cite-faire), [iso-bias](#cite-iso-bias),
[karvonen](#cite-karvonen), [llm-hire-fairly](#cite-llm-hire-fairly)). ATSGate never reads a
name, photo, address, school name, graduation year or employment gap as a scoring input, and
metamorphic tests ([`fairness.rs`](crates/atsgate-score/tests/fairness.rs)) show that changing
any of them leaves every score bit-identical, over every built-in blueprint.
Because it is deterministic, those properties are checkable rather than statistical.

## Citations

Generated from [CITATIONS.md](CITATIONS.md), the single source of truth for sources; CI fails if
this list drifts.

<!-- citations:begin (generated from CITATIONS.md; do not edit) -->
- <a id="cite-ats-screener-method"></a>**ats-screener-method**: S. Patel. *ATS Screener scoring methodology, dimensions and thresholds*. 2026. <https://github.com/sunnypatell/ats-screener/tree/main/docs/src/content/docs/scoring> (CR; accessed 2026-11-20). Supports: The six vendor profiles' weights and strictness (MIT, at 4fd3009, re-mapped onto ATSGate's dimensions) and default floors from thresholds.md; its pass floors contradict each other between methodology.md and thresholds.md.
- <a id="cite-ats-screener-research"></a>**ats-screener-research**: S. Patel. *ATS parsing, scoring and platform research notes*. 2026-02. <https://github.com/sunnypatell/ats-screener/tree/main/research> (CR; accessed 2026-11-20). Supports: Vendor behaviour caveats: Lever and Greenhouse do not auto-score or auto-reject.
- <a id="cite-confit-v2"></a>**confit-v2**: Yu, Xu, Xue, Zhang, Ma, Z. Yu. *ConFit v2: Improving Resume-Job Matching using Hypothetical Resume Embedding and Runner-Up Hard-Negative Mining*. 2025-02. <https://arxiv.org/abs/2502.12361> (AX; accessed 2026-11-20). Supports: BM25 trails dense retrieval on full resume-to-job ranking.
- <a id="cite-confit-v3"></a>**confit-v3**: Yu et al.. *ConFit v3: Improving Resume-Job Matching with LLM-based Re-Ranking*. 2026-05. <https://arxiv.org/abs/2605.09760> (AX; accessed 2026-11-20). Supports: State-of-the-art context; LLMs kept out of deterministic scoring.
- <a id="cite-pjb"></a>**pjb**: Wang et al.. *PJB: A Reasoning-Aware Benchmark for Person-Job Retrieval*. 2026-03. <https://arxiv.org/abs/2603.17386> (AX; accessed 2026-11-20). Supports: Domain variance exceeds model gains.
- <a id="cite-workrb"></a>**workrb**: De Lange et al.. *WorkRB: A Community-Driven Evaluation Framework for AI in the Work Domain*. 2026-03. <https://arxiv.org/abs/2604.13055> (AX; accessed 2026-11-20). Supports: Lexical baselines hold on skill normalization (BM25 47.4 MAP) but collapse on titles (2.5).
- <a id="cite-lcd-pjf"></a>**lcd-pjf**: Jouanneau, Jouffroy, Palyart. *An Efficient Long-Context Ranking Architecture With Calibrated LLM Distillation: Application to Person-Job Fit*. 2026-01. <https://arxiv.org/abs/2601.10321> (AX; accessed 2026-11-20). Supports: Skill-fit scores and calibration.
- <a id="cite-review-recruit"></a>**review-recruit**: Zhao, Wei. *From Matching Models to Recruiting Agents: A Systematized Narrative Review of AI Recruitment Systems, Evaluation, and Governance*. 2026-09. <https://arxiv.org/abs/2609.04286> (AX; accessed 2026-11-20). Supports: Evidence-level evaluation and contestability.
- <a id="cite-hybrid-emb"></a>**hybrid-emb**: Yashwant et al.. *Semantic Candidate-Job Matching: A Comparative Evaluation of Dense Embedding Models in Hybrid Retrieval*. 2026-09. <https://arxiv.org/abs/2609.23307> (AX; accessed 2026-11-20). Supports: Hybrid lexical and dense retrieval.
- <a id="cite-talentclef25"></a>**talentclef25**: Gasco et al.. *Overview of the TalentCLEF 2025: Skill and Job Title Intelligence for Human Capital Management*. 2025-07. <https://arxiv.org/abs/2507.13275> (AX; accessed 2026-11-20). Supports: Job-title and skill benchmark; training strategy beat model size.
- <a id="cite-talentclef26"></a>**talentclef26**: Gasco et al.. *Overview of the TalentCLEF 2026: Skill and Job Title Intelligence for Human Capital Management*. 2026-06. <https://arxiv.org/abs/2606.31692> (AX; accessed 2026-11-20). Supports: Job-person matching task.
- <a id="cite-nlpnorth"></a>**nlpnorth**: Zhang, van der Goot. *NLPnorth @ TalentCLEF 2025: Comparing Discriminative, Contrastive, and Prompt-Based Methods for Job Title and Skill Matching*. 2025-06. <https://arxiv.org/abs/2506.19058> (AX; accessed 2026-11-20). Supports: Method comparison on title and skill matching.
- <a id="cite-esco-linking"></a>**esco-linking**: Saroglou et al.. *Enhancing Job Matching: Occupation, Skill and Qualification Linking with the ESCO and EQF taxonomies*. 2025-12. <https://arxiv.org/abs/2512.03195> (AX; accessed 2026-11-20). Supports: Entity linking suits skills; sentence linking suits occupations.
- <a id="cite-skillspan"></a>**skillspan**: Zhang, Jensen, Sonniks, Plank. *SkillSpan: Hard and Soft Skill Extraction from English Job Postings*. 2022. <https://arxiv.org/abs/2204.12811> (PR; accessed 2026-11-20). Supports: Span-annotated job-description dataset (NAACL 2022).
- <a id="cite-rethink-se"></a>**rethink-se**: Nguyen, Zhang, Montariol, Bosselut. *Rethinking Skill Extraction in the Job Market Domain using Large Language Models*. 2024. <https://arxiv.org/abs/2402.03832> (PR; accessed 2026-11-20). Supports: In-context LLM extraction still trails supervised taggers (workshop paper).
- <a id="cite-decorte-xml"></a>**decorte-xml**: Decorte et al.. *Extreme Multi-Label Skill Extraction Training using Large Language Models*. 2023. <https://arxiv.org/abs/2307.10778> (PR; accessed 2026-11-20). Supports: Synthetic-data contrastive models beat literal-match supervision (workshop paper).
- <a id="cite-senger-survey"></a>**senger-survey**: Senger, Zhang, van der Goot, Plank. *Deep Learning-based Computational Job Market Analysis: A Survey on Skill Extraction and Classification from Job Postings*. 2024. <https://arxiv.org/abs/2402.05617> (PR; accessed 2026-11-20). Supports: Catalogue of skill-extraction datasets (workshop paper).
- <a id="cite-resumebench"></a>**resumebench**: Ling et al.. *Beyond Human Labels: A Multi-Linguistic Auto-Generated Benchmark for Evaluating Large Language Models on Resume Parsing*. 2025-11. <https://aclanthology.org/2025.emnlp-main.1626> (PR; accessed 2026-11-20). Supports: Resume parsing benchmark (EMNLP 2025, synthetic).
- <a id="cite-layout-resume"></a>**layout-resume**: Zhu et al.. *Layout-Aware Parsing Meets Efficient LLMs: A Unified, Scalable Framework for Resume Information Extraction and Evaluation*. 2025-10. <https://arxiv.org/abs/2510.09722> (AX; accessed 2026-11-20). Supports: Normalize layout before extracting resume fields.
- <a id="cite-pdf-tools"></a>**pdf-tools**: Adhikari, Agarwal. *A Comparative Study of PDF Parsing Tools Across Diverse Document Categories*. 2024. <https://arxiv.org/abs/2410.09871> (AX; accessed 2026-11-20). Supports: Rule-based extractor choice (PyMuPDF and pdfium strongest).
- <a id="cite-tk-columns"></a>**tk-columns**: Textkernel. *Improving extraction from column resumes*. n.d.. <https://textkernel.com/learn-support/blog/improving-extraction-from-column-resumes> (VD; accessed 2026-11-20). Supports: At least 15% of CVs use columns; naive stream order interleaves them.
- <a id="cite-faire"></a>**faire**: Wen et al.. *FAIRE: Assessing Racial and Gender Bias in AI-Driven Resume Evaluations*. 2025-04. <https://arxiv.org/abs/2504.01420> (AX; accessed 2026-11-20). Supports: Name-based bias in LLM screeners varies by model.
- <a id="cite-iso-bias"></a>**iso-bias**: Iso et al.. *Evaluating Bias in LLMs for Job-Resume Matching: Gender, Race, and Education*. 2025-03. <https://arxiv.org/abs/2503.19182> (PR; accessed 2026-11-20). Supports: Education bias persists (NAACL 2025 Industry).
- <a id="cite-karvonen"></a>**karvonen**: Karvonen, Marks. *Robustly Improving LLM Fairness in Realistic Settings via Interpretability*. 2025-06. <https://arxiv.org/abs/2506.10922> (AX; accessed 2026-11-20). Supports: College names act as a demographic proxy despite anti-bias prompts.
- <a id="cite-fairness-not-enough"></a>**fairness-not-enough**: Webster. *Fairness Is Not Enough: Auditing Competence and Intersectional Bias in AI-powered Resume Screening*. 2025-07. <https://arxiv.org/abs/2507.11548> (AX; accessed 2026-11-20). Supports: Audit competence as well as bias.
- <a id="cite-popresume"></a>**popresume**: Yu, Park, Moon. *PopResume: Causal Fairness Evaluation of LLM/VLM Resume Screeners with Population-Representative Dataset*. 2026-03. <https://arxiv.org/abs/2603.22714> (AX; accessed 2026-11-20). Supports: Causal fairness audit dataset.
- <a id="cite-llm-hire-fairly"></a>**llm-hire-fairly**: Gao, Jiang, Yan. *Can LLMs Hire Fairly? Racial Bias in Resume Screening*. 2026-06. <https://arxiv.org/abs/2606.28978> (AX; accessed 2026-11-20). Supports: Bias direction flips across model generations.
- <a id="cite-bias-tails"></a>**bias-tails**: Nghiem et al.. *Bias in the Tails: How Name-conditioned Evaluative Framing in Resume Summaries Destabilizes LLM-based Hiring*. 2026-04. <https://arxiv.org/abs/2604.19984> (AX; accessed 2026-11-20). Supports: Name-driven variation hides in score-distribution tails.
- <a id="cite-audit-verdict"></a>**audit-verdict**: Vohra, Ravikiran. *The Audit Decides the Verdict: Instrument Effects Rival Demographic Bias in LLM Decision Audits*. 2026-09. <https://arxiv.org/abs/2609.09048> (AX; accessed 2026-11-20). Supports: Audit construction changes the finding (workshop paper).
- <a id="cite-task-audit"></a>**task-audit**: Braun et al.. *Hiring Discrimination and the Task Content of Jobs: Evidence from a Large-Scale Resume Audit*. 2026-04. <https://arxiv.org/abs/2604.01933> (AX; accessed 2026-11-20). Supports: Discretion widens gaps; objective rules narrow them.
- <a id="cite-inj-measure"></a>**inj-measure**: Zhang et al.. *Measuring Real-World Prompt Injection Attacks in LLM-based Resume Screening*. 2026-05. <https://arxiv.org/abs/2605.28999> (AX; accessed 2026-11-20). Supports: About 1% of real resumes carry hidden injections (USENIX Security 2026 listed).
- <a id="cite-resumeshield"></a>**resumeshield**: Barach. *ResumeShield: Channel Separation and an Open Benchmark for Indirect Prompt Injection in AI Resume Screening*. 2026-07. <https://arxiv.org/abs/2609.20188> (AX; accessed 2026-11-20). Supports: Concealment is itself a detectable signal (precision 1.0, recall 0.944).
- <a id="cite-inj-multi"></a>**inj-multi**: Baxi et al.. *Prompt Injection in Automated Résumé Screening with Large Language Models: Single and Multi-Injection Settings*. 2026-06. <https://arxiv.org/abs/2606.27287> (AX; accessed 2026-11-20). Supports: Manipulation dynamics across screeners (ACL Findings listed).
- <a id="cite-ai-sec-resume"></a>**ai-sec-resume**: Mu et al.. *AI Security Beyond Core Domains: Resume Screening as a Case Study of Adversarial Vulnerabilities in Specialized LLM Applications*. 2025-12. <https://arxiv.org/abs/2512.20164> (AX; accessed 2026-11-20). Supports: Invisible-keyword attacks.
- <a id="cite-hidden-workers"></a>**hidden-workers**: Fuller, Raman, Sage-Gavin, Hines; Harvard Business School and Accenture. *Hidden Workers: Untapped Talent (Executive Summary)*. 2021-09. <https://pubwww.hbs.edu/managing-the-future-of-work/Documents/research/Hidden%20Workers%20Executive%20Summary.pdf> (IB; accessed 2026-11-20). Supports: Recruiting systems exclude candidates for a gap in employment; ATSGate never penalizes gaps.
- <a id="cite-kristal-gaps"></a>**kristal-gaps**: Kristal, Nicks, Gloor, Hauser. *Reducing discrimination against job seekers with and without employment gaps*. 2022-12. <https://europepmc.org/article/PMC/PMC7614241> (PR; accessed 2026-11-20). Supports: Listing years worked instead of dates raised callbacks about 8% (vs no gap) and 15% (vs gap); Nature Human Behaviour 7(2):211-218.
- <a id="cite-gh-tm-faq"></a>**gh-tm-faq**: Greenhouse. *Talent Matching Data Processing FAQ*. n.d.. <https://support.greenhouse.io/hc/en-us/articles/41131616864283> (VD; accessed 2026-11-20). Supports: No auto-reject; names and contact details excluded.
- <a id="cite-gh-import"></a>**gh-import**: Greenhouse. *Why didn't the resume import correctly?*. n.d.. <https://support.greenhouse.io/hc/en-us/articles/200721684> (VD; accessed 2026-11-20). Supports: Unrecognized characters break parsing.
- <a id="cite-gh-types"></a>**gh-types**: Greenhouse. *Supported file types*. n.d.. <https://support.greenhouse.io/hc/en-us/articles/360052218132> (VD; accessed 2026-11-20). Supports: Accepted input formats.
- <a id="cite-wd-parse"></a>**wd-parse**: Workday. *Set Up Prospects and Candidates*. n.d.. <https://doc.workday.com/admin-guide/en-us/human-capital-management/recruiting/candidates/set-up-prospects-and-candidates/hdc1552497830785.html> (VD; accessed 2026-11-20). Supports: Parse results vary with format and word order; avoid image resumes.
- <a id="cite-taleo-attach"></a>**taleo-attach**: Oracle. *Attachments*. 22D. <https://docs.oracle.com/en/cloud/saas/taleo-enterprise/22d/otrcg/c-attachment.html> (VD; accessed 2026-11-20). Supports: Accepted attachment formats.
- <a id="cite-icims-blog"></a>**icims-blog**: iCIMS. *What is CV/Resume parsing?*. 2020-09. <https://www.icims.com/blog/what-is-cv-resume-parsing/> (VD; accessed 2026-11-20). Supports: Context only (vendor blog).
- <a id="cite-sap-kba"></a>**sap-kba**: SAP. *KBA 2951470*. n.d.. <https://userapps.support.sap.com/sap/support/knowledge/en/2951470> (VD; accessed 2026-11-20). Supports: Resume field-mapping limits.
- <a id="cite-onet-lic"></a>**onet-lic**: U.S. Department of Labor, Employment and Training Administration. *O*NET Database License*. 31.0. <https://www.onetcenter.org/license_db.html> (LG; accessed 2026-11-20). Supports: Embedding O*NET data is allowed under CC BY 4.0 with attribution.
- <a id="cite-esco-dl"></a>**esco-dl**: European Commission. *Download ESCO*. n.d.. <https://esco.ec.europa.eu/en/use-esco/download> (LG; accessed 2026-11-20). Supports: Bulk download sits behind a form, so ATSGate imports ESCO v1.2.1 through the public API instead; reuse terms are in esco-faq.
- <a id="cite-esco-faq"></a>**esco-faq**: European Commission. *ESCO frequently asked questions: reuse of ESCO*. n.d.. <https://esco.ec.europa.eu/en/about-esco/faq> (LG; accessed 2026-11-20). Supports: Free reuse for any purpose under Decision 2011/833/EU; services state "This service uses the ESCO classification of the European Commission"; modified versions must say so.
- <a id="cite-onet-db"></a>**onet-db**: U.S. Department of Labor, Employment and Training Administration. *O*NET 31.0 Database*. 31.0. <https://www.onetcenter.org/database.html> (LG; accessed 2026-11-20). Supports: Current release is 31.0, licensed CC BY 4.0; source of the embedded occupations, skills and technology examples.
- <a id="cite-skillspan-data"></a>**skillspan-data**: Zhang et al.. *SkillSpan data repository*. 2023-08. <https://github.com/kris927b/SkillSpan> (CR; accessed 2026-11-20). Supports: MIT-licensed test split (JSON, BIO skill and knowledge tags, house and tech subsets) used in the evaluation.
- <a id="cite-techwolf-sentences"></a>**techwolf-sentences**: Decorte, Verlinden, Van Hautte, Deleu, Develder, Demeester (TechWolf). *Synthetic-ESCO-skill-sentences*. 2023. <https://huggingface.co/datasets/TechWolf/Synthetic-ESCO-skill-sentences> (CR; accessed 2026-11-20). Supports: 138,260 synthetic sentences, ten per ESCO v1.1.0 skill; CC BY 4.0.
- <a id="cite-talentclef25-corpus"></a>**talentclef25-corpus**: Gascó et al. (Avature). *TalentCLEF 2025 corpus: Skill and Job Title Intelligence for Human Capital Management*. 2025-04. <https://zenodo.org/records/15292308> (CR; accessed 2026-11-20). Supports: Task A job-title matching data (v0.5.0), CC BY 4.0; the evaluation uses its English validation split.
- <a id="cite-hn-artcraft"></a>**hn-artcraft**: possibilistic. *ArtCraft: Senior Engineers, onsite San Francisco or Atlanta (Ask HN: Who is hiring? November 2026)*. 2026-11-02. <https://news.ycombinator.com/item?id=50342041> (IB; accessed 2026-11-20). Supports: Real posting used as the generator's worked example.
- <a id="cite-lightcast-faq"></a>**lightcast-faq**: Lightcast. *Lightcast Skill Taxonomy FAQs*. n.d.. <https://lightcast.io/our-data/taxonomies/open-skills/faqs> (VD; accessed 2026-11-20). Supports: Use is set by the licence type requested; terms are not stated openly, so ATSGate does not embed it.
- <a id="cite-schema-jp"></a>**schema-jp**: schema.org. *JobPosting*. current. <https://schema.org/JobPosting> (LG; accessed 2026-11-20). Supports: Blueprint field alignment.
- <a id="cite-jsonresume"></a>**jsonresume**: JSON Resume. *resume-schema*. archived. <https://github.com/jsonresume/resume-schema> (CR; accessed 2026-11-20). Supports: Career SSOT base format (MIT).
- <a id="cite-ll144"></a>**ll144**: NYC Department of Consumer and Worker Protection. *Automated Employment Decision Tools: Frequently Asked Questions*. 2023. <https://www.nyc.gov/site/dca/businesses/faq-automated-employment-decision-tools.page> (LG; accessed 2026-11-20). Supports: Bias audits compute selection or scoring rates and impact ratios across sex, race/ethnicity and intersectional categories.
- <a id="cite-eu-aia-a3"></a>**eu-aia-a3**: artificialintelligenceact.eu. *AI Act Annex III: High-Risk AI Systems*. 2024. <https://artificialintelligenceact.eu/annex/3/> (LG; accessed 2026-11-20). Supports: Recruitment AI is high-risk (Annex III 4(a)).
<!-- citations:end -->

## License

Code: dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
Data: see [DATA-LICENSES.md](DATA-LICENSES.md) for the O*NET and ESCO terms that taxonomy data
will ship under.
