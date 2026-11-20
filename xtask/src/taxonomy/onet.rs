//! O*NET 31.0 CSV files to taxonomy records (pure).
//!
//! - Content-model elements 2.A (essential skills), 2.B (transferable skills), 2.C (knowledge)
//!   and 2.E (software categories) become `onet:<element id>`, linked to their parent element.
//! - Each Software Skills "Workplace Example" becomes `onet-tech:<example>`, linked to the
//!   element(s) it is filed under.
//! - Each occupation becomes `onet-soc:<code>` with its alternate and reported titles as aliases
//!   and its software examples as linked skills.

use atsgate_taxonomy::ConceptKind;
use atsgate_taxonomy::data::Record;
use std::collections::{BTreeMap, BTreeSet};

pub struct Files<'a> {
    pub content_model: &'a [u8],
    pub occupations: &'a [u8],
    pub job_titles: &'a [u8],
    pub reported_titles: &'a [u8],
    pub software: &'a [u8],
}

fn rows(bytes: &[u8], columns: &[&str]) -> Result<Vec<Vec<String>>, String> {
    let mut reader = csv::Reader::from_reader(bytes);
    let headers = reader.headers().map_err(|e| e.to_string())?.clone();
    let idx = columns
        .iter()
        .map(|c| {
            headers
                .iter()
                .position(|h| h == *c)
                .ok_or(format!("O*NET file lacks column {c:?}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    reader
        .records()
        .map(|r| {
            let r = r.map_err(|e| e.to_string())?;
            Ok(idx.iter().map(|&i| r[i].trim().to_owned()).collect())
        })
        .collect()
}

const ELEMENT_ROOTS: [&str; 4] = ["2.A", "2.B", "2.C", "2.E"];

fn element_kind(id: &str) -> ConceptKind {
    let depth = id.matches('.').count();
    if id.starts_with("2.E") || depth < 3 {
        ConceptKind::SkillGroup
    } else {
        ConceptKind::Skill
    }
}

fn parent(id: &str) -> Option<&str> {
    id.rsplit_once('.').map(|(p, _)| p)
}

struct Builder {
    records: Vec<Record>,
    index: BTreeMap<String, u32>,
}

impl Builder {
    fn add(&mut self, kind: ConceptKind, reference: String, label: String) -> u32 {
        let i = self.records.len() as u32;
        self.index.insert(reference.clone(), i);
        self.records.push(Record {
            kind,
            reference,
            label,
            aliases: Vec::new(),
            broader: Vec::new(),
            skills: Vec::new(),
        });
        i
    }
}

fn set_aliases(record: &mut Record, aliases: BTreeSet<String>) {
    let label = atsgate_taxonomy::fold(&record.label);
    record.aliases = aliases
        .into_iter()
        .filter(|a| !a.is_empty() && atsgate_taxonomy::fold(a) != label)
        .collect();
}

pub fn records(files: &Files) -> Result<Vec<Record>, String> {
    let mut b = Builder {
        records: Vec::new(),
        index: BTreeMap::new(),
    };
    let elements: BTreeMap<String, String> =
        rows(files.content_model, &["Element ID", "Element Name"])?
            .into_iter()
            .filter(|r| {
                ELEMENT_ROOTS
                    .iter()
                    .any(|root| r[0] == *root || r[0].starts_with(&format!("{root}.")))
            })
            .map(|r| (r[0].clone(), r[1].clone()))
            .collect();
    for (id, name) in &elements {
        b.add(element_kind(id), format!("onet:{id}"), name.clone());
    }
    for id in elements.keys() {
        if let Some(p) = parent(id).and_then(|p| b.index.get(&format!("onet:{p}")).copied()) {
            let i = b.index[&format!("onet:{id}")];
            b.records[i as usize].broader.push(p);
        }
    }
    let software = rows(
        files.software,
        &["O*NET-SOC Code", "Workplace Example", "Element ID"],
    )?;
    let mut examples: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for r in &software {
        examples
            .entry(r[1].clone())
            .or_default()
            .insert(r[2].clone());
    }
    for (example, element_ids) in &examples {
        let i = b.add(
            ConceptKind::Skill,
            format!("onet-tech:{example}"),
            example.clone(),
        );
        let broader = element_ids
            .iter()
            .map(|e| {
                b.index
                    .get(&format!("onet:{e}"))
                    .copied()
                    .ok_or(format!("unknown element {e}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let record = &mut b.records[i as usize];
        record.broader = broader;
    }
    let mut titles: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for r in rows(
        files.job_titles,
        &["O*NET-SOC Code", "Job Title", "Short Title"],
    )? {
        let entry = titles.entry(r[0].clone()).or_default();
        entry.insert(r[1].clone());
        entry.insert(r[2].clone());
    }
    for r in rows(
        files.reported_titles,
        &["O*NET-SOC Code", "Reported Job Title"],
    )? {
        titles.entry(r[0].clone()).or_default().insert(r[1].clone());
    }
    let mut skills: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
    for r in &software {
        let id = b.index[&format!("onet-tech:{}", r[1])];
        skills.entry(r[0].clone()).or_default().insert(id);
    }
    for r in rows(files.occupations, &["O*NET-SOC Code", "Title"])? {
        let i = b.add(
            ConceptKind::Occupation,
            format!("onet-soc:{}", r[0]),
            r[1].clone(),
        );
        let record = &mut b.records[i as usize];
        set_aliases(record, titles.remove(&r[0]).unwrap_or_default());
        record.skills = skills
            .remove(&r[0])
            .unwrap_or_default()
            .into_iter()
            .collect();
    }
    Ok(b.records)
}
