//! Plain JSON Resume to a career file: the same document with an `x-atsgate` block added at the
//! top and on every entry a tailoring file can select. Fields JSON Resume does not define are
//! errors (reported by the career parser), never dropped.

use super::ImportError;
use super::ids::Ids;
use crate::{Career, CareerError, EXTENSION_VERSION};
use serde_json::{Map, Value, json};

/// Each selectable section, and the field that names its entries.
const SECTIONS: [(&str, &str); 6] = [
    ("work", "name"),
    ("volunteer", "organization"),
    ("education", "institution"),
    ("projects", "name"),
    ("publications", "name"),
    ("awards", "title"),
];

const EXTENSION_KEY: &str = "x-atsgate";

fn entries<'a>(root: &'a mut Map<String, Value>, section: &str) -> Vec<&'a mut Map<String, Value>> {
    match root.get_mut(section) {
        Some(Value::Array(items)) => items.iter_mut().filter_map(Value::as_object_mut).collect(),
        _ => Vec::new(),
    }
}

pub fn convert(mut doc: Value) -> Result<Career, ImportError> {
    let root = doc.as_object_mut().ok_or_else(|| {
        ImportError::unrepresentable("document", "the top level is not an object")
    })?;
    if root.contains_key(EXTENSION_KEY) {
        return Err(ImportError::AlreadyCareer);
    }
    let mut ids = Ids::default();
    for (section, name_field) in SECTIONS {
        for entry in entries(root, section) {
            if entry.contains_key(EXTENSION_KEY) {
                return Err(ImportError::AlreadyCareer);
            }
            let name = entry.get(name_field).and_then(Value::as_str).unwrap_or("");
            let id = ids.derive(name, section)?;
            entry.insert(EXTENSION_KEY.to_owned(), json!({ "id": id, "sources": [] }));
        }
    }
    root.insert(
        EXTENSION_KEY.to_owned(),
        json!({
            "version": EXTENSION_VERSION,
            "sources": {},
            "families": [],
            "variants": { "summaries": {}, "titles": {} }
        }),
    );
    Ok(serde_json::from_value(doc).map_err(CareerError::Parse)?)
}
