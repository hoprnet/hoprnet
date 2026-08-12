//! Attributing return-path loss to the nodes responsible for it.
//!
//! A dead return relay is invisible to the sender's own delivery clock: replies simply stop, and
//! sustained loss says *a* relayer on the return path failed without saying which. Probing finds it
//! eventually, but only after an EMA moves behind a 60 s path cache — tens of seconds during which
//! the session is already degraded.
//!
//! The SURB creator can do better, because it already holds the evidence. `HoprReplyOpener` is
//! `(HoprSurbId, ReplyOpener)` and `create_surb` mints the SURB and its opener together, so the
//! return path each SURB encodes is known at mint time and the id comes back on every
//! successfully-opened reply. Recording that mapping turns each reply into a per-node observation
//! at data-stream rate, with no protocol change and no extra traffic.
//!
//! # What is and is not observable
//!
//! Only *arrivals* are observed. A SURB still sitting unused in the replier's buffer is
//! indistinguishable from one whose reply was lost, so the denominator has to come from somewhere
//! else: an entry that expires without a reply is counted as a failure, and the patience is the
//! cache TTL. That makes the unused fraction a *common-mode* error — the replier pops SURBs in
//! buffer order, which is independent of the path a SURB encodes — so it inflates every node's
//! failure count roughly equally and cancels out of comparisons between nodes. Absolute rates from
//! this module are therefore meaningless; only relative ones are used, which is why
//! [`ReliabilityTable::detect`] scores against the population median rather than a fixed target.
//!
//! # Credit assignment
//!
//! Every node on an arrived path is credited, every node on an expired path is debited. A dead node
//! has zero successes across *all* paths through it, so it separates quickly; a healthy node that
//! merely shares some paths with it keeps the successes from its other paths.
//!
//! This requires overlapping paths to work. Where the candidate set is so thin that a node and its
//! path-mates always travel together, they are mathematically indistinguishable and no amount of
//! data separates them — the same low-connectivity limit that bounds every other return-path
//! mitigation.

use std::collections::HashMap;

use hopr_api::OffchainPublicKey;

/// One-sided CUSUM detector for a sustained drop in a Bernoulli success stream.
///
/// Preferred over an EMA because the two fail differently. An EMA with a time constant short enough
/// to react in a few samples is also short enough to swing on a single unlucky one; lengthen it and
/// the reaction is too slow to matter. A CUSUM accumulates *evidence* instead of tracking a level,
/// so it can tolerate isolated failures indefinitely while still firing within a few samples once
/// failures become persistent — "reacts fast but holds state".
///
/// The statistic is `S_n = max(0, S_{n-1} + (target - x_n) - slack)`, firing when `S_n > threshold`.
/// Flooring at zero is what gives the tolerance: a run of successes pays down accumulated evidence
/// but never banks credit, so a node cannot earn immunity by behaving well for a long time first.
#[derive(Debug, Clone, PartialEq)]
pub struct Cusum {
    /// Deviation tolerated per observation before evidence accumulates.
    ///
    /// Conventionally half the shift worth detecting: to catch a fall from a 0.9 success rate to
    /// 0.5, a slack of ~0.2 ignores ordinary jitter around 0.9 while still accumulating on 0.5.
    slack: f64,
    /// Accumulated evidence at which a change is declared.
    ///
    /// Trades detection delay against false alarms: with `slack` set for the shift of interest,
    /// roughly `threshold / slack` consecutive bad observations are needed to fire.
    threshold: f64,
    sum: f64,
}

impl Cusum {
    pub fn new(slack: f64, threshold: f64) -> Self {
        Self {
            slack,
            threshold,
            sum: 0.0,
        }
    }

    /// Feeds one observation, returning `true` when the accumulated evidence declares a change.
    ///
    /// Firing resets the statistic, so a node that stays bad re-fires periodically rather than
    /// latching once — callers refresh a decaying penalty instead of tracking recovery here.
    pub fn observe(&mut self, success: bool, target: f64) -> bool {
        let observed = if success { 1.0 } else { 0.0 };
        self.sum = (self.sum + (target - observed) - self.slack).max(0.0);

        if self.sum > self.threshold {
            self.sum = 0.0;
            return true;
        }
        false
    }

    /// Accumulated evidence, for tests and diagnostics.
    pub fn evidence(&self) -> f64 {
        self.sum
    }
}

/// Success/failure tally and change detector for one node.
#[derive(Debug, Clone)]
struct NodeRecord {
    successes: u64,
    failures: u64,
    cusum: Cusum,
}

impl NodeRecord {
    fn new(cusum: Cusum) -> Self {
        Self {
            successes: 0,
            failures: 0,
            cusum,
        }
    }

    fn observations(&self) -> u64 {
        self.successes + self.failures
    }

    fn success_rate(&self) -> Option<f64> {
        match self.observations() {
            0 => None,
            n => Some(self.successes as f64 / n as f64),
        }
    }
}

/// Per-node return-path reliability, scored against the population.
#[derive(Debug, Clone)]
pub struct ReliabilityTable {
    nodes: HashMap<OffchainPublicKey, NodeRecord>,
    prototype: Cusum,
    min_observations: u64,
}

impl ReliabilityTable {
    /// `min_observations` guards the early phase, where one node having seen three replies and
    /// another none says nothing about either.
    pub fn new(slack: f64, threshold: f64, min_observations: u64) -> Self {
        Self {
            nodes: HashMap::new(),
            prototype: Cusum::new(slack, threshold),
            min_observations,
        }
    }

    /// Credits every node on a return path whose reply arrived.
    pub fn record_arrival(&mut self, path: &[OffchainPublicKey]) {
        for node in path {
            self.entry(node).successes += 1;
        }
    }

    /// Debits every node on a return path whose SURB expired without a reply.
    pub fn record_expiry(&mut self, path: &[OffchainPublicKey]) {
        for node in path {
            self.entry(node).failures += 1;
        }
    }

    fn entry(&mut self, node: &OffchainPublicKey) -> &mut NodeRecord {
        let prototype = self.prototype.clone();
        self.nodes.entry(*node).or_insert_with(|| NodeRecord::new(prototype))
    }

    /// Median success rate over nodes with enough observations to count.
    ///
    /// The median rather than the mean so a single collapsed node cannot drag the reference down
    /// far enough to excuse itself.
    pub fn population_median(&self) -> Option<f64> {
        let mut rates: Vec<f64> = self
            .nodes
            .values()
            .filter(|r| r.observations() >= self.min_observations)
            .filter_map(|r| r.success_rate())
            .collect();
        if rates.is_empty() {
            return None;
        }
        rates.sort_by(|a, b| a.partial_cmp(b).expect("success rates are never NaN"));
        Some(rates[rates.len() / 2])
    }

    /// Feeds the latest observation for `node` to its detector and reports whether it just became
    /// distinguishable from the population.
    ///
    /// Scoring against the population median makes this a *relative* test: when the whole network
    /// degrades, the median falls with it and nothing fires, because re-routing cannot help. Only a
    /// node falling behind its peers is actionable.
    pub fn detect(&mut self, node: &OffchainPublicKey, success: bool) -> bool {
        let Some(median) = self.population_median() else {
            return false;
        };
        let prototype = self.prototype.clone();
        let record = self.nodes.entry(*node).or_insert_with(|| NodeRecord::new(prototype));
        if record.observations() < self.min_observations {
            return false;
        }
        record.cusum.observe(success, median)
    }

    /// Observed success rate for a node, once it has enough observations.
    pub fn success_rate(&self, node: &OffchainPublicKey) -> Option<f64> {
        self.nodes
            .get(node)
            .filter(|r| r.observations() >= self.min_observations)
            .and_then(|r| r.success_rate())
    }

    pub fn tracked_nodes(&self) -> usize {
        self.nodes.len()
    }
}

#[cfg(test)]
mod tests {
    use hopr_api::types::crypto::prelude::{Keypair, OffchainKeypair};

    use super::*;

    fn node(seed: u8) -> OffchainPublicKey {
        let mut bytes = [1u8; 32];
        bytes[0] = seed;
        *OffchainKeypair::from_secret(&bytes).expect("valid secret").public()
    }

    fn detector() -> Cusum {
        // Against a 0.9 target, one failure contributes `(0.9 - 0) - 0.2 = 0.7` and one success
        // repays `0.3`. A threshold of 2.5 therefore needs four consecutive failures to fire while
        // a lone failure (0.7) is repaid by the successes around it.
        Cusum::new(0.2, 2.5)
    }

    #[test]
    fn cusum_should_ignore_isolated_failures_indefinitely() {
        let mut cusum = detector();
        // One failure in every ten, forever, is jitter — not a change worth re-routing for.
        for round in 0..200 {
            let success = round % 10 != 0;
            assert!(
                !cusum.observe(success, 0.9),
                "fired on jitter at round {round} (evidence {})",
                cusum.evidence()
            );
        }
    }

    #[test]
    fn cusum_should_fire_within_a_few_samples_of_a_sustained_drop() {
        let mut cusum = detector();
        let fired_after = (1..=20).find(|_| cusum.observe(false, 0.9));
        assert_eq!(
            Some(4),
            fired_after,
            "a persistent failure run must be caught in a handful of samples"
        );
    }

    #[test]
    fn cusum_should_reset_after_firing_so_a_bad_node_re_fires() {
        let mut cusum = detector();
        while !cusum.observe(false, 0.9) {}
        assert_eq!(0.0, cusum.evidence(), "firing must clear the evidence");
        // Still bad → fires again, refreshing whatever penalty the caller applies.
        let refired = (1..=10).any(|_| cusum.observe(false, 0.9));
        assert!(refired);
    }

    #[test]
    fn cusum_should_not_bank_credit_for_a_long_healthy_run() {
        let mut cusum = detector();
        // A thousand successes must not buy immunity from the next sustained failure.
        for _ in 0..1000 {
            assert!(!cusum.observe(true, 0.9));
        }
        let fired_after = (1..=20).find(|_| cusum.observe(false, 0.9));
        assert_eq!(
            Some(4),
            fired_after,
            "history must not delay detection — the statistic floors at zero"
        );
    }

    #[test]
    fn table_should_credit_and_debit_every_node_on_a_path() {
        let mut table = ReliabilityTable::new(0.2, 0.6, 1);
        let (a, b) = (node(1), node(2));

        table.record_arrival(&[a, b]);
        table.record_arrival(&[a, b]);
        table.record_expiry(&[a, b]);

        assert_eq!(2, table.tracked_nodes());
        for n in [a, b] {
            let rate = table.success_rate(&n).expect("both nodes have observations");
            assert!((rate - 2.0 / 3.0).abs() < 1e-9, "{rate}");
        }
    }

    #[test]
    fn table_should_separate_a_dead_node_from_its_path_mates() {
        // `dead` is on every failing path; `shared` rides with it half the time but also has its
        // own healthy paths, which is what must keep it out of trouble.
        let mut table = ReliabilityTable::new(0.2, 0.6, 4);
        let (dead, shared, healthy) = (node(1), node(2), node(3));

        for _ in 0..20 {
            table.record_expiry(&[dead, shared]);
            table.record_arrival(&[shared, healthy]);
            table.record_arrival(&[healthy]);
        }

        let dead_rate = table.success_rate(&dead).expect("observed");
        let shared_rate = table.success_rate(&shared).expect("observed");
        let healthy_rate = table.success_rate(&healthy).expect("observed");

        assert_eq!(0.0, dead_rate);
        assert!((shared_rate - 0.5).abs() < 1e-9, "{shared_rate}");
        assert_eq!(1.0, healthy_rate);
        assert!(
            dead_rate < shared_rate && shared_rate < healthy_rate,
            "attribution must order dead < shared < healthy"
        );
    }

    #[test]
    fn table_should_not_fire_before_a_node_has_enough_observations() {
        let mut table = ReliabilityTable::new(0.2, 0.6, 10);
        let fresh = node(1);
        let established = node(2);

        // Give the population a median to score against.
        for _ in 0..20 {
            table.record_arrival(&[established]);
        }
        // A brand-new node failing three times is not yet evidence of anything.
        for _ in 0..3 {
            table.record_expiry(&[fresh]);
            assert!(!table.detect(&fresh, false));
        }
    }

    #[test]
    fn table_should_not_fire_when_the_whole_population_degrades() {
        // Everything halves. Re-routing cannot help, so nothing should fire — this is exactly what
        // scoring against a fixed target would get wrong.
        let mut table = ReliabilityTable::new(0.2, 0.6, 4);
        let nodes = [node(1), node(2), node(3)];

        for _ in 0..10 {
            for n in &nodes {
                table.record_arrival(&[*n]);
                table.record_expiry(&[*n]);
            }
        }
        for round in 0..40 {
            for n in &nodes {
                let success = round % 2 == 0;
                assert!(!table.detect(n, success), "a network-wide dip must not fire for {n}");
            }
        }
    }

    #[test]
    fn table_should_fire_for_a_node_falling_behind_its_peers() {
        let mut table = ReliabilityTable::new(0.2, 0.6, 4);
        let (failing, peer_a, peer_b) = (node(1), node(2), node(3));

        // Establish a healthy population and a history for the node about to fail.
        for _ in 0..20 {
            for n in [failing, peer_a, peer_b] {
                table.record_arrival(&[n]);
            }
        }

        let fired = (0..20).any(|_| {
            table.record_expiry(&[failing]);
            table.detect(&failing, false)
        });
        assert!(fired, "a node falling behind a healthy median must be detected");
    }
}
