//! DOCX: the OPC package is unzipped in memory; document, header and footer parts are walked.

mod body;

use crate::error::ExtractError;
use crate::flow::FlowBuilder;
use crate::integrity::SignalSet;
use crate::model::{Document, Format, MetadataField, Region};
use crate::signals::{Evidence, SignalKind};
use crate::thresholds::Thresholds;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::{Cursor, Read};

type Archive<'a> = zip::ZipArchive<Cursor<&'a [u8]>>;

fn read_part(zip: &mut Archive, name: &str, limit: u64) -> Result<Option<String>, ExtractError> {
    let mut file = match zip.by_name(name) {
        Ok(f) => f,
        Err(zip::result::ZipError::FileNotFound) => return Ok(None),
        Err(e) => return Err(ExtractError::Docx(e.to_string())),
    };
    let mut bytes = Vec::new();
    (&mut file)
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| ExtractError::Docx(e.to_string()))?;
    if bytes.len() as u64 > limit {
        return Err(ExtractError::Docx(format!(
            "{name} is larger than {limit} bytes"
        )));
    }
    Ok(Some(
        String::from_utf8(bytes).map_err(|e| ExtractError::Docx(e.to_string()))?,
    ))
}

/// Header and footer part names from the document relationships, in document order.
fn header_footer_parts(rels: &str) -> Result<Vec<(String, Region)>, ExtractError> {
    let mut reader = Reader::from_str(rels);
    let mut out = Vec::new();
    loop {
        match reader
            .read_event()
            .map_err(|e| ExtractError::Xml(e.to_string()))?
        {
            Event::Empty(e) | Event::Start(e) if e.name().as_ref() == "Relationship" => {
                let get = |k: &str| {
                    e.attributes()
                        .flatten()
                        .find(|a| a.key.as_ref() == k)
                        .map(|a| a.value.to_string())
                };
                let (Some(kind), Some(target)) = (get("Type"), get("Target")) else {
                    continue;
                };
                let region = if kind.ends_with("/header") {
                    Region::Header
                } else if kind.ends_with("/footer") {
                    Region::Footer
                } else {
                    continue;
                };
                out.push((
                    format!(
                        "word/{}",
                        target.trim_start_matches('/').trim_start_matches("word/")
                    ),
                    region,
                ));
            }
            Event::Eof => return Ok(out),
            _ => {}
        }
    }
}

fn core_properties(xml: &str) -> Result<Vec<MetadataField>, ExtractError> {
    const FIELDS: [(&str, &str); 4] = [
        ("dc:title", "docx.title"),
        ("dc:subject", "docx.subject"),
        ("cp:keywords", "docx.keywords"),
        ("dc:description", "docx.description"),
    ];
    let mut reader = Reader::from_str(xml);
    let (mut out, mut current) = (Vec::new(), None);
    loop {
        match reader
            .read_event()
            .map_err(|e| ExtractError::Xml(e.to_string()))?
        {
            Event::Start(e) => {
                current = FIELDS
                    .iter()
                    .find(|(tag, _)| e.name().as_ref() == *tag)
                    .map(|(_, n)| *n)
            }
            Event::Text(t) => {
                if let Some(name) = current {
                    let value = AsRef::<str>::as_ref(&t).to_owned();
                    out.push(MetadataField { name, value });
                }
            }
            Event::End(_) => current = None,
            Event::Eof => return Ok(out),
            _ => {}
        }
    }
}

pub fn extract(bytes: &[u8], t: &Thresholds) -> Result<Document, ExtractError> {
    let mut zip =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| ExtractError::Docx(e.to_string()))?;
    let limit = t.max_docx_part_bytes;
    let document = read_part(&mut zip, "word/document.xml", limit)?
        .ok_or(ExtractError::MissingPart("word/document.xml"))?;
    let rels = read_part(&mut zip, "word/_rels/document.xml.rels", limit)?.unwrap_or_default();
    let parts = if rels.is_empty() {
        Vec::new()
    } else {
        header_footer_parts(&rels)?
    };
    let mut b = FlowBuilder::default();
    for (name, region) in parts.iter().filter(|(_, r)| *r == Region::Header) {
        let xml = read_part(&mut zip, name, limit)?.ok_or(ExtractError::MissingPart("header"))?;
        body::walk(&xml, *region, &mut b)?;
    }
    let columns = body::walk(&document, Region::Body, &mut b)?;
    for (name, region) in parts.iter().filter(|(_, r)| *r == Region::Footer) {
        let xml = read_part(&mut zip, name, limit)?.ok_or(ExtractError::MissingPart("footer"))?;
        body::walk(&xml, *region, &mut b)?;
    }
    let mut extra = SignalSet::default();
    if let Some(n) = columns.filter(|n| *n >= 2) {
        let evidence = Evidence {
            text: format!("w:cols w:num=\"{n}\""),
            span: None,
            bbox: None,
        };
        extra.add(SignalKind::MultiColumn { columns: n }, Some(1), evidence);
    }
    let metadata = match read_part(&mut zip, "docProps/core.xml", limit)? {
        Some(xml) => core_properties(&xml)?,
        None => Vec::new(),
    };
    b.signals.extend(extra);
    Ok(b.finish(Format::Docx, metadata, t))
}
