//! How far reading order departs from stored order.

use crate::model::SpanId;

/// Spans that must move to turn `stored` order into `reading` order: n minus the longest
/// common subsequence. Both are permutations of the same ids, so the LCS is a longest
/// increasing subsequence of stored positions, found in O(n log n).
pub fn displaced(stored: &[SpanId], reading: &[SpanId]) -> u32 {
    let position: std::collections::HashMap<SpanId, usize> =
        stored.iter().enumerate().map(|(i, id)| (*id, i)).collect();
    let mut tails: Vec<usize> = Vec::new();
    for id in reading {
        let p = position[id];
        let at = tails.partition_point(|&t| t < p);
        if at == tails.len() {
            tails.push(p);
        } else {
            tails[at] = p;
        }
    }
    (reading.len() - tails.len()) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(v: &[u32]) -> Vec<SpanId> {
        v.iter().copied().map(SpanId).collect()
    }

    #[test]
    fn identical_orders_displace_nothing() {
        assert_eq!(displaced(&ids(&[0, 1, 2, 3]), &ids(&[0, 1, 2, 3])), 0);
    }

    #[test]
    fn a_block_drawn_last_but_read_early_displaces_its_spans() {
        assert_eq!(
            displaced(&ids(&[0, 1, 2, 3, 4, 5]), &ids(&[0, 4, 5, 1, 2, 3])),
            2
        );
    }
}
