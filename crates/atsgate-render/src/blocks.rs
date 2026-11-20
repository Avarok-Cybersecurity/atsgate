//! The resume as a flat list of text blocks: the one layout both the PDF and the DOCX draw, so
//! the two files always carry the same text in the same order.

use crate::model::{Body, Entry, ResumeDocument, heading};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Block {
    Name {
        text: String,
    },
    Title {
        text: String,
    },
    Contact {
        text: String,
    },
    Heading {
        text: String,
    },
    Paragraph {
        text: String,
    },
    /// An entry's first line: the title in bold, then `rest` (organization and dates).
    Entry {
        title: String,
        rest: String,
    },
    Bullet {
        text: String,
    },
    /// A labelled line such as a skill group: `label: text`.
    Labeled {
        label: String,
        text: String,
    },
}

/// The bullet marker every renderer writes, followed by one space.
pub const BULLET: &str = "\u{2022} ";

/// Separates the parts of an entry line; atsgate-resume splits titles from organizations on it.
pub const PART_SEPARATOR: &str = " | ";

/// An entry's first line after the title: `" | organization | dates"`.
pub fn entry_rest(e: &Entry) -> String {
    let dates = e.dates.as_ref().map(|d| d.text());
    [e.organization.clone(), dates]
        .into_iter()
        .flatten()
        .map(|part| format!("{PART_SEPARATOR}{part}"))
        .collect()
}

/// The document as blocks, in reading order.
pub fn blocks(doc: &ResumeDocument) -> Vec<Block> {
    let mut out = vec![
        Block::Name {
            text: doc.name.clone(),
        },
        Block::Title {
            text: doc.title.clone(),
        },
        Block::Contact {
            text: doc.contact.join(PART_SEPARATOR),
        },
    ];
    for section in &doc.sections {
        out.push(Block::Heading {
            text: heading(section.kind).to_owned(),
        });
        match &section.body {
            Body::Text(text) => out.push(Block::Paragraph { text: text.clone() }),
            Body::Groups(groups) => out.extend(groups.iter().map(|g| Block::Labeled {
                label: g.label.clone(),
                text: g.items.join(", "),
            })),
            Body::Entries(entries) => {
                for e in entries {
                    out.push(Block::Entry {
                        title: e.title.clone(),
                        rest: entry_rest(e),
                    });
                    out.extend(e.bullets.iter().map(|b| Block::Bullet { text: b.clone() }));
                }
            }
        }
    }
    out
}
