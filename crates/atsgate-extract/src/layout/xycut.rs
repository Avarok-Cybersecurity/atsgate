//! Recursive XY-cut over spans and frames, with a guard that keeps same-row text together.
//!
//! A vertical cut is tried first and accepted only when both sides hold rows that the other side
//! does not: right-aligned dates share their title's baseline and so never become a "column".
//! Otherwise the topmost row band is peeled off and the rest is cut again.

use crate::model::{BBox, SpanId};
use crate::thresholds::Thresholds;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Item {
    Span { id: SpanId, bbox: BBox },
    Frame { index: usize, bbox: BBox },
}

impl Item {
    fn bbox(&self) -> BBox {
        match self {
            Item::Span { bbox, .. } | Item::Frame { bbox, .. } => *bbox,
        }
    }
}

/// A run of items read as one unit: a leaf block of spans, or one frame.
#[derive(Debug, Clone, PartialEq)]
pub enum Group {
    Leaf(Vec<Item>),
    Frame(usize),
}

#[derive(Debug, Default)]
pub struct Cut {
    pub groups: Vec<Group>,
    /// Each accepted column split, as the span ids left and right of the gutter.
    pub column_splits: Vec<(Vec<SpanId>, Vec<SpanId>)>,
}

fn baselines(items: &[Item]) -> Vec<f32> {
    items
        .iter()
        .filter_map(|i| match i {
            Item::Span { bbox, .. } => Some(bbox.y0),
            Item::Frame { .. } => None,
        })
        .collect()
}

fn unmatched_rows(side: &[f32], other: &[f32], tol: f32) -> usize {
    let mut rows: Vec<f32> = side
        .iter()
        .copied()
        .filter(|y| !other.iter().any(|o| (o - y).abs() <= tol))
        .collect();
    rows.sort_by(f32::total_cmp);
    rows.dedup_by(|a, b| (*a - *b).abs() <= tol);
    rows.len()
}

fn span_ids(items: &[Item]) -> Vec<SpanId> {
    items
        .iter()
        .filter_map(|i| match i {
            Item::Span { id, .. } => Some(*id),
            Item::Frame { .. } => None,
        })
        .collect()
}

/// Gutters (start x, end x) wider than the column gap, widest first.
fn gutters(items: &[Item], min_gap: f32) -> Vec<(f32, f32)> {
    let mut spans: Vec<(f32, f32)> = items.iter().map(|i| (i.bbox().x0, i.bbox().x1)).collect();
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut out = Vec::new();
    let mut reach = spans[0].1;
    for &(x0, x1) in &spans[1..] {
        if x0 - reach >= min_gap {
            out.push((reach, x0));
        }
        reach = reach.max(x1);
    }
    out.sort_by(|a, b| (b.1 - b.0).total_cmp(&(a.1 - a.0)));
    out
}

fn vertical_split(items: &[Item], t: &Thresholds) -> Option<(Vec<Item>, Vec<Item>)> {
    for (start, _) in gutters(items, t.column_gap_pt) {
        let (left, right): (Vec<Item>, Vec<Item>) =
            items.iter().partition(|i| i.bbox().x1 <= start);
        let (lb, rb) = (baselines(&left), baselines(&right));
        let tol = t.baseline_tolerance_pt;
        let is_columns = unmatched_rows(&lb, &rb, tol) >= t.min_column_rows as usize
            && unmatched_rows(&rb, &lb, tol) >= t.min_column_rows as usize;
        let frames_only = lb.is_empty() || rb.is_empty();
        if is_columns || frames_only {
            return Some((left, right));
        }
    }
    None
}

/// The topmost band of vertically overlapping items, and everything below it.
fn peel_top(items: &[Item]) -> Option<(Vec<Item>, Vec<Item>)> {
    let mut sorted = items.to_vec();
    sorted.sort_by(|a, b| b.bbox().y1.total_cmp(&a.bbox().y1));
    let mut floor = sorted[0].bbox().y0;
    let mut cut = sorted.len();
    for (i, item) in sorted.iter().enumerate().skip(1) {
        if item.bbox().y1 <= floor {
            cut = i;
            break;
        }
        floor = floor.min(item.bbox().y0);
    }
    (cut < sorted.len()).then(|| {
        let rest = sorted.split_off(cut);
        (sorted, rest)
    })
}

/// An uncuttable block: spans stay together, and any frame inside it is read where its top edge
/// falls, between the spans above and below it.
fn leaf(mut items: Vec<Item>, out: &mut Cut) {
    items.sort_by(|a, b| b.bbox().y1.total_cmp(&a.bbox().y1));
    let mut spans = Vec::new();
    for item in items {
        match item {
            Item::Span { .. } => spans.push(item),
            Item::Frame { index, .. } => {
                if !spans.is_empty() {
                    out.groups.push(Group::Leaf(std::mem::take(&mut spans)));
                }
                out.groups.push(Group::Frame(index));
            }
        }
    }
    if !spans.is_empty() {
        out.groups.push(Group::Leaf(spans));
    }
}

pub fn cut(items: Vec<Item>, t: &Thresholds, out: &mut Cut) {
    match items.as_slice() {
        [] => {}
        [_] => leaf(items, out),
        _ => {
            if let Some((left, right)) = vertical_split(&items, t) {
                let (l, r) = (span_ids(&left), span_ids(&right));
                if !l.is_empty() && !r.is_empty() {
                    out.column_splits.push((l, r));
                }
                cut(left, t, out);
                cut(right, t, out);
            } else if let Some((top, rest)) = peel_top(&items) {
                cut(top, t, out);
                cut(rest, t, out);
            } else {
                leaf(items, out);
            }
        }
    }
}
