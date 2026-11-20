//! Pinned inputs and the credit lines every distribution of the derived data carries.
//! DATA-LICENSES.md quotes the same lines; `taxonomy --verify` fails when they drift.

use atsgate_taxonomy::SourceName;

pub const ONET_BASE: &str = "https://www.onetcenter.org/dl_files/database/db_31_0_csv/";

/// O*NET 31.0 CSV files and their sha256 (order matters: see `onet::Files`).
pub const ONET_FILES: [(&str, &str); 5] = [
    (
        "content_model_reference.csv",
        "370033744006dbd5df048431d42a402f6741a00a229b6d0f4e4f4249b6399c04",
    ),
    (
        "occupation_data.csv",
        "a09eae1d6609686e44e05b7290993a1c8b523d8ca224bc0eedc194c855c3ee02",
    ),
    (
        "job_titles.csv",
        "75a3b46d6edc5996117c10c9ed488c9a06f7495d02969e9d12873ab14a21540c",
    ),
    (
        "sample_of_reported_titles.csv",
        "5d76040b7b0602ea217660648f8363e7980edd1870f96574a2eeaa6a822d2666",
    ),
    (
        "software_skills.csv",
        "6aabb96b464288db849580e6510530efff3f80e25dde2bb23f1fc36bb526b016",
    ),
];

/// sha256 of the canonical projection of ESCO v1.2.1 released English skills / occupations.
pub const ESCO_SKILL_PROJECTION: &str =
    "14f824861b47c942c867394b5e0263dba1a6ced50d7fae06f2adfe6f78820923";
pub const ESCO_OCCUPATION_PROJECTION: &str =
    "c36995d5f2262ff6e51b0c65a72191ad6ae314144ced42c3182bdc131cc20f59";

pub const ONET_CREDIT: &str = "This product includes information from the O*NET 31.0 Database by \
the U.S. Department of Labor, Employment and Training Administration (USDOL/ETA). Used under the \
CC BY 4.0 license. O*NET® is a trademark of USDOL/ETA. Avarok Cybersecurity has modified all or \
some of this information. USDOL/ETA has not approved, endorsed, or tested these modifications.";

pub const ESCO_CREDIT: &str = "This service uses the ESCO classification of the European \
Commission (ESCO v1.2.1), reused under Commission Decision 2011/833/EU on the reuse of Commission \
documents. This is a modified version: Avarok Cybersecurity selected the English labels and \
links and derived alias tables from them. The European Commission does not endorse this product.";

/// (SPDX license id, credit line) for a source.
pub fn credit(source: SourceName) -> (&'static str, &'static str) {
    match source {
        SourceName::Onet => ("CC-BY-4.0", ONET_CREDIT),
        SourceName::Esco => ("CC-BY-4.0", ESCO_CREDIT),
        SourceName::Curated => ("MIT OR Apache-2.0", "ATSGate curated alias table."),
    }
}
