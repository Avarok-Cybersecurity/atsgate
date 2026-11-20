//! Page layout for positioned spans: regions, frames, reading order and lines.

pub mod frames;
pub mod order;
pub mod xycut;

use crate::integrity::{SignalSet, span_evidence};
use crate::model::{BBox, Line, Region, Span, SpanId};
use crate::signals::{Evidence, SignalKind};
use crate::thresholds::Thresholds;
use frames::{Frame, FrameKind};
use xycut::{Cut, Group, Item};

fn bbox(s: &Span) -> BBox {
    s.bbox.expect("layout only runs on positioned spans")
}

fn assign_regions(
    spans: &mut [Span],
    height: f32,
    frames: &[Frame],
    t: &Thresholds,
) -> Vec<Option<usize>> {
    spans
        .iter_mut()
        .map(|s| {
            let b = bbox(s);
            let frame = frames
                .iter()
                .position(|f| f.bbox.contains_point(b.center()));
            s.region = if b.y0 >= height * (1.0 - t.header_band) {
                Region::Header
            } else if b.y1 <= height * t.footer_band {
                Region::Footer
            } else {
                match frame.map(|i| &frames[i].kind) {
                    Some(FrameKind::Table { .. }) => Region::Table,
                    Some(FrameKind::TextBox) => Region::TextBox,
                    None => Region::Body,
                }
            };
            frame
        })
        .collect()
}

/// Spans grouped into rows that share a baseline, top to bottom, each row left to right.
fn rows(spans: &[Span], mut ids: Vec<SpanId>, page: u32, t: &Thresholds) -> Vec<Line> {
    let at = |id: &SpanId| {
        spans
            .iter()
            .find(|s| s.id == *id)
            .expect("span on this page")
    };
    ids.sort_by(|a, b| {
        bbox(at(b))
            .y0
            .total_cmp(&bbox(at(a)).y0)
            .then(bbox(at(a)).x0.total_cmp(&bbox(at(b)).x0))
    });
    let mut lines: Vec<Line> = Vec::new();
    for id in ids {
        let s = at(&id);
        let b = bbox(s);
        match lines.last_mut() {
            Some(l) if (l.bbox.expect("pdf line").y0 - b.y0).abs() <= t.baseline_tolerance_pt => {
                let prev = l.bbox.expect("pdf line");
                let gap = b.x0 - prev.x1;
                let joiner = if gap > 0.1 * s.font_size.unwrap_or(0.0) && !l.text.ends_with(' ') {
                    " "
                } else {
                    ""
                };
                l.text = format!("{}{joiner}{}", l.text, s.text);
                l.bbox = Some(prev.union(&b));
                l.spans.push(id);
            }
            _ => lines.push(Line {
                text: s.text.clone(),
                page,
                bbox: Some(b),
                spans: vec![id],
                region: s.region,
            }),
        }
    }
    lines
}

fn group_ids(items: &[Item]) -> Vec<SpanId> {
    items
        .iter()
        .filter_map(|i| {
            if let Item::Span { id, .. } = i {
                Some(*id)
            } else {
                None
            }
        })
        .collect()
}

#[derive(Debug, Default)]
pub struct PageLayout {
    pub lines: Vec<Line>,
    pub signals: SignalSet,
}

/// Lays out one page's spans (all on `page`) given the page height and its ruled frames.
pub fn layout_page(
    spans: &mut [Span],
    page: u32,
    height: f32,
    frames: &[Frame],
    t: &Thresholds,
) -> PageLayout {
    let in_frame = assign_regions(spans, height, frames, t);
    let mut items: Vec<Item> = spans
        .iter()
        .zip(&in_frame)
        .filter(|(_, f)| f.is_none())
        .map(|(s, _)| Item::Span {
            id: s.id,
            bbox: bbox(s),
        })
        .collect();
    let used: Vec<usize> = (0..frames.len())
        .filter(|i| in_frame.contains(&Some(*i)))
        .collect();
    items.extend(used.iter().map(|&index| Item::Frame {
        index,
        bbox: frames[index].bbox,
    }));
    let mut cut = Cut::default();
    xycut::cut(items, t, &mut cut);
    let mut out = PageLayout::default();
    for group in &cut.groups {
        match group {
            Group::Leaf(items) => out.lines.extend(rows(spans, group_ids(items), page, t)),
            Group::Frame(index) => {
                let frame = &frames[*index];
                let members: Vec<&Span> = spans
                    .iter()
                    .zip(&in_frame)
                    .filter(|(_, f)| **f == Some(*index))
                    .map(|(s, _)| s)
                    .collect();
                for cell in frame.cells() {
                    let ids = members
                        .iter()
                        .filter(|s| cell.contains_point(bbox(s).center()))
                        .map(|s| s.id)
                        .collect();
                    out.lines.extend(rows(spans, ids, page, t));
                }
                let kind = match frame.kind {
                    FrameKind::Table { .. } => SignalKind::Table,
                    FrameKind::TextBox => SignalKind::TextBox,
                };
                members
                    .iter()
                    .for_each(|s| out.signals.add(kind.clone(), Some(page), span_evidence(s)));
            }
        }
    }
    for (left, right) in &cut.column_splits {
        for id in [left[0], right[0]] {
            let s = spans
                .iter()
                .find(|s| s.id == id)
                .expect("split span on page");
            let evidence = Evidence {
                text: s.text.clone(),
                span: Some(id),
                bbox: s.bbox,
            };
            out.signals
                .add(SignalKind::MultiColumn { columns: 2 }, Some(page), evidence);
        }
    }
    out
}
