//! Assembles pdf_oxide's positioned glyphs into styled runs, in content-stream order.
//!
//! Glyphs are used rather than pdf_oxide's spans because its span post-processing drops text it
//! judges duplicated (on our text-box fixture it dropped the whole boxed summary) and normalizes
//! away zero-width characters and ligatures, both of which ATSGate must see.

use super::content::TextShow;
use crate::model::{BBox, Rgb};
use pdf_oxide::layout::TextChar;

#[derive(Debug, Clone)]
pub struct GlyphRun {
    pub text: String,
    pub bbox: BBox,
    pub baseline: f32,
    pub font_size: f32,
    pub color: Rgb,
    /// Index of the text-showing operation that drew the run's first glyph.
    pub stream_index: usize,
}

fn rgb(c: &TextChar) -> Rgb {
    Rgb {
        r: c.color.r,
        g: c.color.g,
        b: c.color.b,
    }
}

fn bbox(c: &TextChar) -> BBox {
    BBox {
        x0: c.bbox.x,
        y0: c.bbox.y,
        x1: c.bbox.x + c.bbox.width,
        y1: c.bbox.y + c.bbox.height,
    }
}

fn continues(run: &GlyphRun, font: &str, run_font: &str, c: &TextChar, tol: f32) -> bool {
    let gap = c.bbox.x - run.bbox.x1;
    font == run_font
        && (c.font_size - run.font_size).abs() < 0.01
        && rgb(c) == run.color
        && (c.origin_y - run.baseline).abs() <= tol
        && gap >= -tol
        && gap <= c.font_size
}

/// The first text-showing operation that starts on `run`'s baseline within its horizontal extent.
fn stream_index(run: &GlyphRun, shows: &[TextShow], tol: f32) -> usize {
    shows
        .iter()
        .position(|s| {
            (s.origin.1 - run.baseline).abs() <= tol
                && s.origin.0 >= run.bbox.x0 - tol
                && s.origin.0 <= run.bbox.x1
        })
        .unwrap_or(usize::MAX)
}

/// Runs of glyphs that share font, size, colour and baseline and sit next to each other.
pub fn assemble(chars: &[TextChar], shows: &[TextShow], tol: f32) -> Vec<GlyphRun> {
    let mut runs: Vec<(GlyphRun, &str)> = Vec::new();
    for c in chars {
        match runs.last_mut() {
            Some((run, font)) if continues(run, &c.font_name, font, c, tol) => {
                run.text.push(c.char);
                run.bbox = run.bbox.union(&bbox(c));
            }
            _ => {
                let run = GlyphRun {
                    text: c.char.to_string(),
                    bbox: bbox(c),
                    baseline: c.origin_y,
                    font_size: c.font_size,
                    color: rgb(c),
                    stream_index: 0,
                };
                runs.push((run, c.font_name.as_str()));
            }
        }
    }
    let mut out: Vec<GlyphRun> = runs
        .into_iter()
        .map(|(mut run, _)| {
            run.stream_index = stream_index(&run, shows, tol);
            run
        })
        .filter(|r| !r.text.trim().is_empty())
        .collect();
    out.sort_by_key(|r| r.stream_index);
    out
}
