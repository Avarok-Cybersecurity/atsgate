//! `taxonomy --write | --check | --verify`: builds `crates/atsgate-taxonomy/data/*.jsonl.zst`
//! from pinned O*NET 31.0 and ESCO v1.2.1 downloads.
//!
//! - `--write` downloads (cached in `.cache/taxonomy`), verifies every pin, and writes the data.
//! - `--check` does the same and fails if the committed data differs (a full reproduction).
//! - `--verify` is offline: the committed files decode, their headers carry the pins and credit
//!   lines below, DATA-LICENSES.md quotes the credit lines, and the size budget holds.

mod derive;
mod esco;
mod onet;
mod pins;

use crate::fetch;
use crate::{Mode, root, sync_file};
use atsgate_taxonomy::data::{self, DataHeader, DigestOf, InputPin, Record, SourceName};

/// The committed data may not exceed this many bytes in total.
pub const SIZE_BUDGET_BYTES: usize = 4 * 1024 * 1024;

pub const ONET_PATH: &str = "crates/atsgate-taxonomy/data/onet-31.0.jsonl.zst";
pub const ESCO_PATH: &str = "crates/atsgate-taxonomy/data/esco-v1.2.1.jsonl.zst";

fn compress(header: &DataHeader, records: &[Record]) -> Result<Vec<u8>, String> {
    let text = data::to_jsonl(header, records).map_err(|e| e.to_string())?;
    Ok(ruzstd::encoding::compress_to_vec(
        text.as_bytes(),
        ruzstd::encoding::CompressionLevel::Fastest,
    ))
}

fn header(source: SourceName, version: &str, inputs: Vec<InputPin>, n: usize) -> DataHeader {
    let (license, credit) = pins::credit(source);
    DataHeader {
        format: data::FORMAT_VERSION,
        source,
        version: version.to_owned(),
        license: license.to_owned(),
        credit: credit.to_owned(),
        inputs,
        records: n,
    }
}

fn onet() -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    for (name, sha) in pins::ONET_FILES {
        let url = format!("{}{name}", pins::ONET_BASE);
        let path = fetch::cache_dir(&["taxonomy", "onet-31.0", name]);
        bytes.push(fetch::pinned(&url, sha, &path)?);
    }
    let records = onet::records(&onet::Files {
        content_model: &bytes[0],
        occupations: &bytes[1],
        job_titles: &bytes[2],
        reported_titles: &bytes[3],
        software: &bytes[4],
    })?;
    let inputs = pins::ONET_FILES
        .iter()
        .map(|(name, sha)| InputPin {
            url: format!("{}{name}", pins::ONET_BASE),
            sha256: (*sha).to_owned(),
            digest_of: DigestOf::Bytes,
        })
        .collect();
    compress(
        &header(SourceName::Onet, "31.0", inputs, records.len()),
        &records,
    )
}

fn esco_kind(kind: &str, broader_link: &str, pin: &str) -> Result<Vec<esco::Projected>, String> {
    let dir = ["taxonomy", "esco-v1.2.1"];
    let first_path = fetch::cache_dir(&[dir[0], dir[1], &format!("{kind}-0.json")]);
    let first = fetch::cached(&esco::page_url(kind, 0), &first_path)?;
    let pages = esco::total(&first)?.div_ceil(esco::PAGE_SIZE);
    let jobs: Vec<_> = (0..pages)
        .map(|p| {
            let path = fetch::cache_dir(&[dir[0], dir[1], &format!("{kind}-{p}.json")]);
            (esco::page_url(kind, p), path)
        })
        .collect();
    let mut projected = Vec::new();
    for page in fetch::cached_all(&jobs)? {
        projected.extend(esco::project(&page, broader_link)?);
    }
    let digest = esco::digest(&projected)?;
    if digest != pin {
        return Err(format!(
            "ESCO {kind} projection sha256 {digest} does not match the pin {pin}; \
             clear .cache/taxonomy/esco-v1.2.1 and retry, or review the change and update the pin"
        ));
    }
    Ok(projected)
}

fn esco() -> Result<Vec<u8>, String> {
    let skills = esco_kind("skill", "broaderSkill", pins::ESCO_SKILL_PROJECTION)?;
    let occupations = esco_kind(
        "occupation",
        "broaderOccupation",
        pins::ESCO_OCCUPATION_PROJECTION,
    )?;
    let records = esco::records(&skills, &occupations)?;
    let inputs = [
        ("skill", pins::ESCO_SKILL_PROJECTION),
        ("occupation", pins::ESCO_OCCUPATION_PROJECTION),
    ]
    .iter()
    .map(|(kind, sha)| InputPin {
        url: esco::page_url(kind, 0).replace("&offset=0", "&offset={page}"),
        sha256: (*sha).to_owned(),
        digest_of: DigestOf::Projection,
    })
    .collect();
    compress(
        &header(SourceName::Esco, esco::VERSION, inputs, records.len()),
        &records,
    )
}

fn verify() -> Result<(), String> {
    let licenses = std::fs::read_to_string(root().join("DATA-LICENSES.md"))
        .map_err(|e| format!("DATA-LICENSES.md: {e}"))?;
    let mut total = 0;
    for (path, source) in [(ONET_PATH, SourceName::Onet), (ESCO_PATH, SourceName::Esco)] {
        let bytes = std::fs::read(root().join(path)).map_err(|e| format!("{path}: {e}"))?;
        total += bytes.len();
        let (header, _) = data::decode(&bytes).map_err(|e| format!("{path}: {e}"))?;
        let (license, credit) = pins::credit(source);
        if header.source != source || header.license != license || header.credit != credit {
            return Err(format!(
                "{path}: header license or credit differs from xtask pins"
            ));
        }
        if !licenses.contains(credit) {
            return Err(format!(
                "DATA-LICENSES.md does not quote the {source:?} credit line"
            ));
        }
    }
    if total > SIZE_BUDGET_BYTES {
        return Err(format!(
            "taxonomy data is {total} bytes; budget {SIZE_BUDGET_BYTES}"
        ));
    }
    Ok(())
}

pub fn run(flag: Option<&str>) -> Result<(), String> {
    let mode = match flag {
        Some("--verify") => return verify(),
        other => crate::parse_mode(other)?,
    };
    sync_file(&root().join(ONET_PATH), &onet()?, mode)?;
    sync_file(&root().join(ESCO_PATH), &esco()?, mode)?;
    if mode == Mode::Write {
        verify()?;
    }
    Ok(())
}
