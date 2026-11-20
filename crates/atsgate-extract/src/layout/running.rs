//! Running headers and footers: a line in the header or footer band is one only when the same
//! text (numbers aside, so "Page 1 of 2" matches "Page 2 of 2") sits at the same height on
//! another page. A band line that appears once is page content, such as the section heading a
//! browser prints at the top of page 2 inside its half-inch margin.

use crate::model::{Line, Region, Span};
use crate::thresholds::Thresholds;

/// The text with every run of digits replaced by `#`, lowercased and trimmed.
fn masked(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.trim().chars() {
        if c.is_ascii_digit() {
            if !out.ends_with('#') {
                out.push('#');
            }
        } else {
            out.extend(c.to_lowercase());
        }
    }
    out
}

fn in_band(line: &Line) -> bool {
    matches!(line.region, Region::Header | Region::Footer)
}

fn repeats(lines: &[Line], i: usize, t: &Thresholds) -> bool {
    let line = &lines[i];
    let (Some(b), key) = (line.bbox, masked(&line.text)) else {
        return false;
    };
    lines.iter().any(|o| {
        o.page != line.page
            && o.region == line.region
            && o.bbox
                .is_some_and(|ob| (ob.y0 - b.y0).abs() <= t.running_position_tolerance_pt)
            && masked(&o.text) == key
    })
}

/// Moves band lines that do not repeat across pages, and their spans, back to the body.
pub fn confirm(lines: &mut [Line], spans: &mut [Span], t: &Thresholds) {
    let lone: Vec<usize> = (0..lines.len())
        .filter(|&i| in_band(&lines[i]) && !repeats(lines, i, t))
        .collect();
    for i in lone {
        lines[i].region = Region::Body;
        for id in &lines[i].spans {
            if let Some(s) = spans.iter_mut().find(|s| s.id == *id) {
                s.region = Region::Body;
            }
        }
    }
}
