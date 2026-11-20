//! HTML: block elements become lines; `hidden`, `display:none` and `visibility:hidden` are flagged.

use super::{FlowBuilder, Run};
use crate::error::ExtractError;
use crate::model::{Document, Format, Region};
use crate::thresholds::Thresholds;
use scraper::{ElementRef, Html, node::Node};

const SKIPPED: [&str; 6] = ["head", "script", "style", "template", "noscript", "svg"];
const BLOCKS: [&str; 22] = [
    "p",
    "div",
    "li",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "tr",
    "br",
    "section",
    "article",
    "header",
    "footer",
    "ul",
    "ol",
    "table",
    "dt",
    "dd",
    "blockquote",
    "pre",
];

fn is_hidden(e: &ElementRef) -> bool {
    let el = e.value();
    let style = el
        .attr("style")
        .unwrap_or("")
        .replace(' ', "")
        .to_lowercase();
    el.attr("hidden").is_some()
        || style.contains("display:none")
        || style.contains("visibility:hidden")
}

struct Walker {
    builder: FlowBuilder,
    runs: Vec<Run>,
}

impl Walker {
    fn flush(&mut self, region: Region) {
        let runs = std::mem::take(&mut self.runs);
        self.builder.paragraph(runs, region);
    }

    /// Adds a text node with HTML whitespace collapsing: runs of whitespace become one space,
    /// and no space is doubled across node boundaries.
    fn text(&mut self, raw: &str, hidden: bool) {
        let mut text = String::new();
        for (i, word) in raw.split_whitespace().enumerate() {
            if i > 0 {
                text.push(' ');
            }
            text.push_str(word);
        }
        let after_space = self.runs.last().is_none_or(|r| r.text.ends_with(' '));
        if raw.starts_with(char::is_whitespace) && !after_space && !text.is_empty() {
            text.insert(0, ' ');
        }
        if raw.ends_with(char::is_whitespace) && !text.is_empty() {
            text.push(' ');
        }
        if !text.is_empty() {
            self.runs.push(Run {
                text,
                vanished: hidden,
                ..Run::default()
            });
        }
    }

    fn walk(&mut self, e: ElementRef, hidden: bool, region: Region) {
        let name = e.value().name();
        if SKIPPED.contains(&name) {
            return;
        }
        let hidden = hidden || is_hidden(&e);
        let region = if matches!(name, "td" | "th") {
            Region::Table
        } else {
            region
        };
        let block = BLOCKS.contains(&name);
        if block {
            self.flush(region);
        }
        if name == "li" {
            self.runs.push(Run {
                text: "\u{2022} ".into(),
                vanished: hidden,
                ..Run::default()
            });
        }
        if name == "img" {
            self.builder.images += 1;
        }
        for child in e.children() {
            match child.value() {
                Node::Text(t) => self.text(t, hidden),
                Node::Element(_) => {
                    if let Some(child) = ElementRef::wrap(child) {
                        self.walk(child, hidden, region);
                    }
                }
                _ => {}
            }
        }
        if block || matches!(name, "td" | "th") {
            self.flush(region);
        }
    }
}

pub fn html(input: &str, t: &Thresholds) -> Result<Document, ExtractError> {
    let doc = Html::parse_document(input);
    let mut w = Walker {
        builder: FlowBuilder::default(),
        runs: Vec::new(),
    };
    w.walk(doc.root_element(), false, Region::Body);
    w.flush(Region::Body);
    Ok(w.builder.finish(Format::Html, Vec::new(), t))
}
