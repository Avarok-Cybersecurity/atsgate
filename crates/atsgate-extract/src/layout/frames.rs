//! Ruled frames from vector paths: grids of cells are tables, a lone ruled box is a text box.

use crate::model::BBox;

/// An axis-aligned ruling segment: horizontal when `y0 == y1`, vertical when `x0 == x1`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl Segment {
    fn is_horizontal(&self) -> bool {
        (self.y0 - self.y1).abs() <= TOUCH
    }
    fn bbox(&self) -> BBox {
        BBox {
            x0: self.x0.min(self.x1),
            y0: self.y0.min(self.y1),
            x1: self.x0.max(self.x1),
            y1: self.y0.max(self.y1),
        }
    }
}

/// Rulings closer than this (points) touch; also the tolerance for "same x" or "same y".
const TOUCH: f32 = 1.0;

#[derive(Debug, Clone, PartialEq)]
pub enum FrameKind {
    /// Cell boundaries: distinct x positions left to right and y positions top to bottom.
    Table {
        xs: Vec<f32>,
        ys: Vec<f32>,
    },
    TextBox,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub bbox: BBox,
    pub kind: FrameKind,
}

impl Frame {
    /// The cells of the frame, top row first and left to right within a row.
    pub fn cells(&self) -> Vec<BBox> {
        match &self.kind {
            FrameKind::TextBox => vec![self.bbox],
            FrameKind::Table { xs, ys } => ys
                .windows(2)
                .flat_map(|y| {
                    xs.windows(2).map(move |x| BBox {
                        x0: x[0],
                        y0: y[1],
                        x1: x[1],
                        y1: y[0],
                    })
                })
                .collect(),
        }
    }
}

fn touches(a: &BBox, b: &BBox) -> bool {
    a.x0 <= b.x1 + TOUCH && b.x0 <= a.x1 + TOUCH && a.y0 <= b.y1 + TOUCH && b.y0 <= a.y1 + TOUCH
}

fn distinct(mut values: Vec<f32>) -> Vec<f32> {
    values.sort_by(f32::total_cmp);
    values.dedup_by(|a, b| (*a - *b).abs() <= TOUCH);
    values
}

/// Groups touching segments; a group with 2 x 2 rulings is a box, more is a table grid.
pub fn detect(segments: &[Segment]) -> Vec<Frame> {
    let boxes: Vec<BBox> = segments.iter().map(Segment::bbox).collect();
    let mut group: Vec<usize> = (0..segments.len()).collect();
    fn root(g: &mut [usize], mut i: usize) -> usize {
        while g[i] != i {
            g[i] = g[g[i]];
            i = g[i];
        }
        i
    }
    for i in 0..boxes.len() {
        for j in i + 1..boxes.len() {
            if touches(&boxes[i], &boxes[j]) {
                let (a, b) = (root(&mut group, i), root(&mut group, j));
                group[a] = b;
            }
        }
    }
    let mut frames = Vec::new();
    for r in 0..segments.len() {
        let members: Vec<usize> = (0..segments.len())
            .filter(|&i| root(&mut group, i) == r)
            .collect();
        if members.is_empty() {
            continue;
        }
        let ys = distinct(
            members
                .iter()
                .filter(|&&i| segments[i].is_horizontal())
                .map(|&i| segments[i].y0)
                .collect(),
        );
        let xs = distinct(
            members
                .iter()
                .filter(|&&i| !segments[i].is_horizontal())
                .map(|&i| segments[i].x0)
                .collect(),
        );
        if xs.len() < 2 || ys.len() < 2 {
            continue;
        }
        let bbox = members
            .iter()
            .map(|&i| boxes[i])
            .reduce(|a, b| a.union(&b))
            .expect("non-empty group");
        let kind = if xs.len() == 2 && ys.len() == 2 {
            FrameKind::TextBox
        } else {
            FrameKind::Table {
                xs,
                ys: ys.into_iter().rev().collect(),
            }
        };
        frames.push(Frame { bbox, kind });
    }
    frames
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> [Segment; 4] {
        let s = |x0, y0, x1, y1| Segment { x0, y0, x1, y1 };
        [
            s(x, y, x + w, y),
            s(x, y + h, x + w, y + h),
            s(x, y, x, y + h),
            s(x + w, y, x + w, y + h),
        ]
    }

    #[test]
    fn a_lone_rectangle_is_a_text_box() {
        let frames = detect(&rect(10.0, 10.0, 100.0, 50.0));
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].kind, FrameKind::TextBox);
    }

    #[test]
    fn adjacent_cells_form_one_table_with_row_major_cells() {
        let mut segs = rect(0.0, 50.0, 40.0, 50.0).to_vec();
        segs.extend(rect(40.0, 50.0, 60.0, 50.0));
        segs.extend(rect(0.0, 0.0, 40.0, 50.0));
        segs.extend(rect(40.0, 0.0, 60.0, 50.0));
        let frames = detect(&segs);
        assert_eq!(frames.len(), 1);
        let cells = frames[0].cells();
        assert_eq!(cells.len(), 4);
        assert_eq!(
            (cells[0].x0, cells[0].y1),
            (0.0, 100.0),
            "top-left cell first"
        );
        assert_eq!((cells[1].x0, cells[1].y1), (40.0, 100.0));
    }

    #[test]
    fn a_single_rule_is_not_a_frame() {
        assert!(
            detect(&[Segment {
                x0: 0.0,
                y0: 5.0,
                x1: 500.0,
                y1: 5.0
            }])
            .is_empty()
        );
    }
}
