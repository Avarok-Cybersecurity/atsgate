//! Font checks from the lopdf pass: composite fonts used without a ToUnicode map.

use super::content::PageScan;
use crate::integrity::SignalSet;
use crate::signals::{Evidence, SignalKind};
use lopdf::{Dictionary, Object};
use std::collections::BTreeMap;

fn name<'a>(d: &'a Dictionary, key: &[u8]) -> Option<&'a [u8]> {
    d.get(key).and_then(Object::as_name).ok()
}

/// Composite (Type0) and Type3 fonts carry codes with no standard meaning: without a ToUnicode
/// map a parser cannot turn their glyphs back into text. Simple fonts with a standard encoding
/// can be decoded without one, so they are not flagged.
fn lacks_unicode(font: &Dictionary) -> bool {
    let composite = matches!(name(font, b"Subtype"), Some(b"Type0" | b"Type3"));
    composite && !font.has(b"ToUnicode")
}

pub fn missing_to_unicode(
    scan: &PageScan,
    fonts: &BTreeMap<Vec<u8>, &Dictionary>,
    page: u32,
    out: &mut SignalSet,
) {
    let mut used: Vec<&Vec<u8>> = scan.shows.iter().map(|s| &s.font).collect();
    used.sort();
    used.dedup();
    for resource in used {
        let Some(font) = fonts.get(resource) else {
            continue;
        };
        if lacks_unicode(font) {
            let base = name(font, b"BaseFont")
                .map(|b| String::from_utf8_lossy(b).into_owned())
                .unwrap_or_default();
            let evidence = Evidence {
                text: base.clone(),
                span: None,
                bbox: None,
            };
            out.add(
                SignalKind::MissingToUnicode { font: base },
                Some(page),
                evidence,
            );
        }
    }
}
