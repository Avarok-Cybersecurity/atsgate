# Data licenses

ATSGate's code is dual-licensed MIT OR Apache-2.0. Data files derived from third-party
occupational data are **separate works** with their own licenses, kept in their own files with
their own attribution and version. Each compiled file also carries its credit line, license and
pinned inputs in its header line.

| File | Source | Version | License | Size (zstd) |
|---|---|---|---|---|
| `crates/atsgate-taxonomy/data/onet-31.0.jsonl.zst` | O*NET 31.0 Database (USDOL/ETA) | 31.0 | CC BY 4.0 | ~0.8 MB |
| `crates/atsgate-taxonomy/data/esco-v1.2.1.jsonl.zst` | ESCO classification (European Commission) | v1.2.1 | Commission reuse policy (CC BY 4.0) | ~2.0 MB |
| `crates/atsgate-taxonomy/data/curated.yaml` | ATSGate (hand-written) | dated in file | MIT OR Apache-2.0 | 8 KB |

The compiled files are built by `cargo run -p xtask -- taxonomy --write` from downloads pinned by
URL and sha256 (`xtask/src/taxonomy/pins.rs`); `taxonomy --check` rebuilds them and fails on any
difference, and CI runs the offline `taxonomy --verify` (headers, these credit lines, and the
4 MiB size budget). **No Lightcast data is used.**

## O*NET 31.0 Database

Licensed under [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/).
Terms: <https://www.onetcenter.org/license_db.html> (opened 2026-11-20). Every distribution of
O*NET-derived data, and the docs site, carries this credit line:

> This product includes information from the O*NET 31.0 Database by the U.S. Department of Labor, Employment and Training Administration (USDOL/ETA). Used under the CC BY 4.0 license. O*NET® is a trademark of USDOL/ETA. Avarok Cybersecurity has modified all or some of this information. USDOL/ETA has not approved, endorsed, or tested these modifications.

"O*NET" is used only as an adjective before a generic noun ("O*NET data"), as the trademark terms
require, and never in the possessive or plural.

**Inputs** (from `https://www.onetcenter.org/dl_files/database/db_31_0_csv/`):
`content_model_reference.csv`, `occupation_data.csv`, `job_titles.csv`,
`sample_of_reported_titles.csv`, `software_skills.csv`.

**Modifications:** content-model elements 2.A, 2.B, 2.C and 2.E become skill and skill-group
concepts linked to their parent element; each Software Skills "Workplace Example" becomes a skill
linked to its category; occupations keep their alternate and reported titles as aliases and their
software examples as linked skills. No alias is derived mechanically (short forms such as
"HubSpot" for "HubSpot software" come from the curated table). Descriptions, ratings and all
other columns are not included.

## ESCO v1.2.1

ESCO is published by the European Commission. Its FAQ (<https://esco.ec.europa.eu/en/about-esco/faq>,
opened 2026-11-20) states that "the ESCO classification can be downloaded, used, reproduced and
reused for any purpose and by any interested party free of charge", under the Commission
Decision of 12 December 2011 on the reuse of Commission documents (2011/833/EU), that the use of
ESCO "shall be acknowledged by publishing the statement" *This service uses the ESCO
classification of the European Commission*, and that "any modified or adapted version of ESCO
must be clearly indicated as such". The Commission's [legal notice](https://commission.europa.eu/legal-notice_en)
(opened 2026-11-20) implements that decision with CC BY 4.0. Every distribution of ESCO-derived
data carries:

> This service uses the ESCO classification of the European Commission (ESCO v1.2.1), reused under Commission Decision 2011/833/EU on the reuse of Commission documents. This is a modified version: Avarok Cybersecurity selected the English labels and links and derived alias tables from them. The European Commission does not endorse this product.

ESCO trade marks and logos are excluded from the reuse policy and are not used.

**Inputs:** the bulk download is delivered only through an e-mailed link, so the import pages
through the official ESCO web API pinned to the release:
`https://ec.europa.eu/esco/api/search?language=en&type={skill|occupation}&limit=100&offset={page}&selectedVersion=v1.2.1&full=true`.
The API's bytes differ between identical requests (link order), so each kind is pinned by the
sha256 of the canonical projection the import keeps (URI, English preferred label, English
alternative labels, broader links), which is stable.

**Modifications:** only released concepts with an English label; English alternative labels as
aliases; `broaderSkill` and `broaderOccupation` links; one derived alias, a trailing
parenthetical dropped ("Python (computer programming)" also matches "Python").

## Evaluation data

The evaluation (`cargo run -p xtask -- eval`) downloads these datasets at pinned revisions into
the gitignored `.cache/eval/`. Small samples are committed under `tests/fixtures/eval/` for the
offline regression tests, each under its own license:

| Sample | Source | License | Credit |
|---|---|---|---|
| `skillspan-test-sample.jsonl` (every 10th test sentence) | SkillSpan, Zhang, Jensen, Sonniks and Plank (NAACL 2022), <https://github.com/kris927b/SkillSpan> at `2ccf3de` | MIT (copyright notice in `tests/fixtures/eval/LICENSE-SkillSpan`) | Unmodified rows |
| `techwolf-sample.csv` (every 100th row) | TechWolf, Synthetic-ESCO-skill-sentences, <https://huggingface.co/datasets/TechWolf/Synthetic-ESCO-skill-sentences> at `09fc487` | CC BY 4.0 | Unmodified rows; subset selected by ATSGate |
| `talentclef-taskA-en-validation/` | TalentCLEF 2025 corpus, Task A, English validation, <https://zenodo.org/records/15292308> (v0.5.0) | CC BY 4.0 | Unmodified files |

## Not used

- **Lightcast Open Skills**: its terms depend on the licence requested and are not stated openly
  ([lightcast-faq](CITATIONS.md)), so its data is not embedded.

## Test fixtures

The resumes under `fixtures/` describe an invented person and are generated by this repository's
own code (`cargo run -p xtask -- fixtures --write`). They are covered by the code license.

`tests/fixtures/jd/artcraft-hn-2026-11-02.txt` is a public Hacker News job posting quoted with its
source, author and retrieval date in its header, as a test input for the generator.
