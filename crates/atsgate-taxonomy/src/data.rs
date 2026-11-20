//! The compiled data format: one zstd-compressed JSON Lines file per source. The first line is a
//! [`DataHeader`]; every later line is a [`Record`]. `xtask taxonomy` writes these files with the
//! same types, so the writer and the reader cannot disagree on the layout.

use serde::{Deserialize, Serialize};

/// The ESCO URI prefix; records store ESCO references without it to keep the data small.
pub const ESCO_URI_PREFIX: &str = "http://data.europa.eu/esco/";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceName {
    /// Hand-written alias tables shipped with this crate.
    Curated,
    Onet,
    Esco,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConceptKind {
    /// A concrete skill, tool or body of knowledge.
    Skill,
    /// A category of skills (an O*NET content-model element such as "Programming").
    SkillGroup,
    Occupation,
}

/// How a pinned input's sha256 was computed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigestOf {
    /// The downloaded bytes.
    Bytes,
    /// The canonical projection of the records taken from it (for API responses whose bytes vary).
    Projection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputPin {
    pub url: String,
    pub sha256: String,
    pub digest_of: DigestOf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataHeader {
    pub format: u32,
    pub source: SourceName,
    pub version: String,
    /// SPDX identifier of the data license.
    pub license: String,
    /// The attribution line every distribution must carry (see DATA-LICENSES.md).
    pub credit: String,
    pub inputs: Vec<InputPin>,
    pub records: usize,
}

pub const FORMAT_VERSION: u32 = 1;

/// One concept. Indices in `broader` and `skills` point at other records of the same file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    #[serde(rename = "k")]
    pub kind: ConceptKind,
    /// The taxonomy reference (ESCO URIs without [`ESCO_URI_PREFIX`]).
    #[serde(rename = "r")]
    pub reference: String,
    #[serde(rename = "l")]
    pub label: String,
    #[serde(rename = "a")]
    pub aliases: Vec<String>,
    #[serde(rename = "b")]
    pub broader: Vec<u32>,
    /// For occupations: the skills the source links to it (O*NET Technology Skills).
    #[serde(rename = "s")]
    pub skills: Vec<u32>,
}

#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("taxonomy data is not valid zstd: {0}")]
    Zstd(String),
    #[error("taxonomy data is not UTF-8")]
    Utf8,
    #[error("taxonomy data line {line}: {source}")]
    Json {
        line: usize,
        source: serde_json::Error,
    },
    #[error("taxonomy data is empty")]
    Empty,
    #[error("taxonomy data format {0} is not supported (expected {FORMAT_VERSION})")]
    Format(u32),
    #[error("taxonomy data header says {expected} records, found {found}")]
    Count { expected: usize, found: usize },
    #[error("record {0} links to a record that does not exist")]
    Dangling(usize),
}

/// Renders a header and records as JSON Lines (uncompressed).
pub fn to_jsonl(header: &DataHeader, records: &[Record]) -> Result<String, serde_json::Error> {
    let mut out = serde_json::to_string(header)?;
    out.push('\n');
    for r in records {
        out.push_str(&serde_json::to_string(r)?);
        out.push('\n');
    }
    Ok(out)
}

/// Parses JSON Lines written by [`to_jsonl`] and checks the record count and internal links.
pub fn from_jsonl(text: &str) -> Result<(DataHeader, Vec<Record>), DataError> {
    let mut lines = text.lines().enumerate();
    let (_, first) = lines.next().ok_or(DataError::Empty)?;
    let header: DataHeader =
        serde_json::from_str(first).map_err(|source| DataError::Json { line: 1, source })?;
    if header.format != FORMAT_VERSION {
        return Err(DataError::Format(header.format));
    }
    let records = lines
        .map(|(i, l)| {
            serde_json::from_str::<Record>(l).map_err(|source| DataError::Json {
                line: i + 1,
                source,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if records.len() != header.records {
        return Err(DataError::Count {
            expected: header.records,
            found: records.len(),
        });
    }
    let n = records.len() as u32;
    if let Some(bad) = records
        .iter()
        .position(|r| r.broader.iter().chain(&r.skills).any(|&i| i >= n))
    {
        return Err(DataError::Dangling(bad));
    }
    Ok((header, records))
}

/// Decompresses a `.jsonl.zst` file and parses it.
pub fn decode(compressed: &[u8]) -> Result<(DataHeader, Vec<Record>), DataError> {
    use std::io::Read;
    let mut decoder = ruzstd::decoding::StreamingDecoder::new(compressed)
        .map_err(|e| DataError::Zstd(e.to_string()))?;
    let mut bytes = Vec::new();
    decoder
        .read_to_end(&mut bytes)
        .map_err(|e| DataError::Zstd(e.to_string()))?;
    let text = String::from_utf8(bytes).map_err(|_| DataError::Utf8)?;
    from_jsonl(&text)
}
