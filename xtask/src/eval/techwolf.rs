//! TechWolf Synthetic-ESCO-skill-sentences (CC BY 4.0): each sentence was written to express
//! one ESCO skill. Recall is how often the gazetteer finds that skill (or a concept one link
//! broader or narrower); precision is how many found skills are the target, a lower bound
//! because a sentence may truly mention other skills.

use atsgate_taxonomy::{Gazetteer, fold};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub sentence: String,
    pub skill: String,
}

pub fn parse(csv_bytes: &[u8]) -> Result<Vec<Row>, String> {
    let mut reader = csv::Reader::from_reader(csv_bytes);
    reader
        .records()
        .map(|r| {
            let r = r.map_err(|e| format!("TechWolf CSV: {e}"))?;
            Ok(Row {
                sentence: r[0].to_owned(),
                skill: r[1].to_owned(),
            })
        })
        .collect()
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Counts {
    pub sentences: usize,
    /// Target skill label resolves in the taxonomy (others are skipped).
    pub resolvable: usize,
    pub found_exact: usize,
    pub found_related: usize,
    pub predicted: usize,
    pub predicted_target: usize,
}

pub fn evaluate(rows: &[Row], gazetteer: &Gazetteer<'_>) -> Counts {
    let t = gazetteer.taxonomy();
    let mut c = Counts::default();
    for row in rows {
        c.sentences += 1;
        let Some(target) = t.resolve_skill(&row.skill) else {
            continue;
        };
        c.resolvable += 1;
        let key = fold(&target.canonical);
        let found = gazetteer.find(&row.sentence);
        c.predicted += found.len();
        let mut exact = false;
        let mut related = false;
        for m in &found {
            let concept = t.concept(m.concept);
            if m.concept == target.concept || fold(concept.canonical()) == key {
                exact = true;
                c.predicted_target += 1;
            } else if concept
                .broader()
                .chain(concept.narrower())
                .any(|n| n.id() == target.concept)
            {
                related = true;
            }
        }
        c.found_exact += usize::from(exact);
        c.found_related += usize::from(exact || related);
    }
    c
}
