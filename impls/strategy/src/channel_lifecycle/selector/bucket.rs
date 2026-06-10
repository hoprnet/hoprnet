//! Anonymity bucket grid: (LatencyBucket, SubnetBucket) cells for open channels.

use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use hopr_api::types::internal::prelude::ChannelId;

use super::subnet::SubnetBucket;

/// Four RTT latency tiers.  Mirrors the threshold values used in the graph weight module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LatencyBucket {
    /// ≤ 75 ms
    Fast,
    /// 76–125 ms
    Medium,
    /// 126–200 ms
    Slow,
    /// > 200 ms, or latency unknown
    VerySlow,
}

impl LatencyBucket {
    pub fn from_latency(d: Option<Duration>) -> Self {
        match d.map(|d| d.as_millis()) {
            Some(ms) if ms <= 75 => Self::Fast,
            Some(ms) if ms <= 125 => Self::Medium,
            Some(ms) if ms <= 200 => Self::Slow,
            _ => Self::VerySlow,
        }
    }
}

/// A (latency, subnet) grid cell.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BucketCell(pub LatencyBucket, pub SubnetBucket);

/// Snapshot of the (latency, subnet) cells for all currently-open channels.
/// Computed once per tick and passed into `SelectorContext`.
pub struct BucketView {
    cells: HashMap<ChannelId, BucketCell>,
}

impl BucketView {
    pub fn new(cells: HashMap<ChannelId, BucketCell>) -> Self {
        Self { cells }
    }

    pub fn empty() -> Self {
        Self { cells: HashMap::new() }
    }

    pub fn cell_for(&self, id: &ChannelId) -> Option<&BucketCell> {
        self.cells.get(id)
    }

    pub fn cell_count(&self, cell: &BucketCell) -> usize {
        self.cells.values().filter(|c| *c == cell).count()
    }

    pub fn distinct_cell_count(&self) -> usize {
        let unique: HashSet<&BucketCell> = self.cells.values().collect();
        unique.len()
    }

    /// Shannon entropy H = -Σ p_i log₂(p_i) over populated bucket cells.
    pub fn shannon_entropy(&self) -> f64 {
        let total = self.cells.len();
        if total == 0 {
            return 0.0;
        }
        let mut counts: HashMap<&BucketCell, usize> = HashMap::new();
        for cell in self.cells.values() {
            *counts.entry(cell).or_insert(0) += 1;
        }
        counts.values().fold(0.0_f64, |acc, &count| {
            let p = count as f64 / total as f64;
            acc - p * p.log2()
        })
    }

    /// 2^H — effective number of distinct buckets weighted by distribution.
    pub fn effective_buckets(&self) -> f64 {
        2.0_f64.powf(self.shannon_entropy())
    }

    /// Relative occupancy of `cell` among all open channels, in [0, 1].
    pub fn bucket_coverage(&self, cell: &BucketCell) -> f64 {
        let total = self.cells.len();
        if total == 0 {
            return 0.0;
        }
        self.cell_count(cell) as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ch(seed: u8) -> ChannelId {
        ChannelId::create(&[&[seed]])
    }

    fn subnet(n: u8) -> SubnetBucket {
        SubnetBucket::V4Prefix([n, 0, 0])
    }

    fn view(entries: Vec<(ChannelId, LatencyBucket, SubnetBucket)>) -> BucketView {
        let map = entries
            .into_iter()
            .map(|(id, lat, sub)| (id, BucketCell(lat, sub)))
            .collect();
        BucketView::new(map)
    }

    #[test]
    fn empty_view() {
        let v = BucketView::empty();
        assert_eq!(v.shannon_entropy(), 0.0);
        assert!((v.effective_buckets() - 1.0).abs() < 1e-9, "2^0 = 1");
        assert_eq!(v.distinct_cell_count(), 0);
    }

    #[test]
    fn uniform_two_cells_entropy_equals_one() {
        let v = view(vec![
            (ch(1), LatencyBucket::Fast, subnet(1)),
            (ch(2), LatencyBucket::Slow, subnet(2)),
        ]);
        assert!((v.shannon_entropy() - 1.0).abs() < 1e-9);
        assert!((v.effective_buckets() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn skewed_entropy_less_than_max() {
        let v = view(vec![
            (ch(1), LatencyBucket::Fast, subnet(1)),
            (ch(2), LatencyBucket::Fast, subnet(1)),
            (ch(3), LatencyBucket::Fast, subnet(1)),
            (ch(4), LatencyBucket::Slow, subnet(2)),
        ]);
        assert!(v.shannon_entropy() < 1.0);
        assert_eq!(v.distinct_cell_count(), 2);
    }

    #[test]
    fn bucket_coverage_proportional() {
        let cell_a = BucketCell(LatencyBucket::Fast, subnet(1));
        let cell_b = BucketCell(LatencyBucket::Slow, subnet(2));
        let v = view(vec![
            (ch(1), LatencyBucket::Fast, subnet(1)),
            (ch(2), LatencyBucket::Fast, subnet(1)),
            (ch(3), LatencyBucket::Slow, subnet(2)),
        ]);
        assert!((v.bucket_coverage(&cell_a) - 2.0 / 3.0).abs() < 1e-9);
        assert!((v.bucket_coverage(&cell_b) - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn latency_bucket_thresholds() {
        assert_eq!(
            LatencyBucket::from_latency(Some(Duration::from_millis(0))),
            LatencyBucket::Fast
        );
        assert_eq!(
            LatencyBucket::from_latency(Some(Duration::from_millis(75))),
            LatencyBucket::Fast
        );
        assert_eq!(
            LatencyBucket::from_latency(Some(Duration::from_millis(76))),
            LatencyBucket::Medium
        );
        assert_eq!(
            LatencyBucket::from_latency(Some(Duration::from_millis(125))),
            LatencyBucket::Medium
        );
        assert_eq!(
            LatencyBucket::from_latency(Some(Duration::from_millis(126))),
            LatencyBucket::Slow
        );
        assert_eq!(
            LatencyBucket::from_latency(Some(Duration::from_millis(200))),
            LatencyBucket::Slow
        );
        assert_eq!(
            LatencyBucket::from_latency(Some(Duration::from_millis(201))),
            LatencyBucket::VerySlow
        );
        assert_eq!(LatencyBucket::from_latency(None), LatencyBucket::VerySlow);
    }
}
