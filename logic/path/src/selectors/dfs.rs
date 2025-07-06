use std::{cmp::Ordering, collections::BinaryHeap, marker::PhantomData, time::Duration};

use async_trait::async_trait;
use hopr_crypto_random::random_float;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use tracing::trace;

use crate::{
    ChannelPath,
    channel_graph::{ChannelEdge, ChannelGraph, Node},
    errors::{PathError, Result},
    selectors::{EdgeWeighting, PathSelector},
};

/// Holds a weighted channel path and auxiliary information for the graph traversal.
#[derive(Clone, Debug, PartialEq, Eq)]
struct WeightedChannelPath {
    path: Vec<Address>,
    weight: U256,
    fully_explored: bool,
}

impl WeightedChannelPath {
    /// Extend this path with another [`ChannelEdge`] and return a new [`WeightedChannelPath`].
    fn extend<CW: EdgeWeighting<U256>>(mut self, edge: &ChannelEdge) -> Self {
        if !self.fully_explored {
            self.path.push(edge.channel.destination);
            self.weight += CW::calculate_weight(edge);
        }
        self
    }
}

impl Default for WeightedChannelPath {
    fn default() -> Self {
        Self {
            path: Vec::with_capacity(INTERMEDIATE_HOPS),
            weight: U256::zero(),
            fully_explored: false,
        }
    }
}

impl PartialOrd for WeightedChannelPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WeightedChannelPath {
    /// Favors unexplored paths over fully explored paths even
    /// when a better alternative exists.
    ///
    /// Favors longer paths over shorter paths, longer path
    /// means more privacy.
    ///
    /// If both parts are of the same length, favors paths
    /// with higher weights.
    fn cmp(&self, other: &Self) -> Ordering {
        if other.fully_explored == self.fully_explored {
            match self.path.len().cmp(&other.path.len()) {
                Ordering::Equal => self.weight.cmp(&other.weight),
                o => o,
            }
        } else if other.fully_explored {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}

/// Assigns each channel a weight.
/// The weight is randomized such that the same
/// nodes get not always selected.
/// This is necessary to achieve privacy.
/// It also favors nodes with higher stake.
#[derive(Clone, Copy, Debug, Default)]
pub struct RandomizedEdgeWeighting;

impl EdgeWeighting<U256> for RandomizedEdgeWeighting {
    /// Multiply channel stake with a random float in the interval [0,1).
    /// Given that the floats are uniformly distributed, nodes with higher
    /// stake have a higher probability of reaching a higher value.
    ///
    /// Sorting the list of weights thus moves nodes with higher stakes more
    /// often to the front.
    fn calculate_weight(edge: &ChannelEdge) -> U256 {
        edge.channel
            .balance
            .amount()
            .mul_f64(random_float())
            .expect("Could not multiply edge weight with float")
            .max(1.into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, smart_default::SmartDefault)]
pub struct DfsPathSelectorConfig {
    /// The maximum number of iterations before a path selection fails
    /// Default is 100
    #[default(100)]
    pub max_iterations: usize,
    /// Peer quality threshold for a node to be taken into account.
    /// Default is 0.5
    #[default(0.5)]
    pub node_score_threshold: f64,
    /// Channel score threshold for a channel to be taken into account.
    /// Default is 0
    #[default(0.0)]
    pub edge_score_threshold: f64,
    /// The maximum latency of the first hop
    /// Default is 100 ms
    #[default(Some(Duration::from_millis(100)))]
    pub max_first_hop_latency: Option<Duration>,
    /// If true, include paths with payment channels, which have no
    /// funds in it. By default, that option is set to `false` to
    /// prevent tickets being dropped immediately.
    /// Defaults to false.
    #[default(false)]
    pub allow_zero_edge_weight: bool,
}

/// Path selector using depth-first search of the channel graph.
#[derive(Clone, Debug)]
pub struct DfsPathSelector<CW> {
    graph: std::sync::Arc<async_lock::RwLock<ChannelGraph>>,
    cfg: DfsPathSelectorConfig,
    _cw: PhantomData<CW>,
}

impl<CW: EdgeWeighting<U256>> DfsPathSelector<CW> {
    /// Creates a new path selector with the given [config](DfsPathSelectorConfig) and
    /// [`ChannelGraph`].
    pub fn new(graph: std::sync::Arc<async_lock::RwLock<ChannelGraph>>, cfg: DfsPathSelectorConfig) -> Self {
        Self {
            graph,
            cfg,
            _cw: PhantomData,
        }
    }

    /// Determines whether a `next_hop` node is considered "interesting".
    ///
    /// To achieve privacy, we need at least one honest node along
    /// the path. Each additional node decreases the probability of
    /// having only malicious nodes, so we can sort out many nodes.
    /// Nodes that have shown to be unreliable are of no use, so
    /// drop them.
    ///
    /// ## Arguments
    /// * `next_hop`: node in the channel graph that we're trying to add to the path
    /// * `edge`: the information about the corresponding graph edge ([`ChannelEntry`] and `score`).
    /// * `initial_source`: the initial node on the path
    /// * `final_destination`: the desired destination node (will not be part of the path)
    /// * `current_path`: currently selected relayers
    #[tracing::instrument(level = "trace", skip(self))]
    fn is_next_hop_usable(
        &self,
        next_hop: &Node,
        edge: &ChannelEdge,
        initial_source: &Address,
        final_destination: &Address,
        current_path: &[Address],
    ) -> bool {
        debug_assert_eq!(next_hop.address, edge.channel.destination);

        // Looping back to self does not give any privacy
        if next_hop.address.eq(initial_source) {
            trace!("source loopback not allowed");
            return false;
        }

        // We cannot use `final_destination` as the last intermediate hop as
        // this would be a loopback that does not give any privacy
        if next_hop.address.eq(final_destination) {
            trace!("destination loopback not allowed");
            return false;
        }

        // Only use nodes that have shown to be somewhat reliable
        if next_hop.node_score < self.cfg.node_score_threshold {
            trace!("node quality threshold not satisfied");
            return false;
        }

        // Edges which have score and is below the threshold will not be considered
        if edge
            .edge_score
            .is_some_and(|score| score < self.cfg.edge_score_threshold)
        {
            trace!("channel score threshold not satisfied");
            return false;
        }

        // If this is the first hop, check if the latency is not too high
        if current_path.is_empty()
            && self
                .cfg
                .max_first_hop_latency
                .is_some_and(|limit| next_hop.latency.average().is_none_or(|avg_latency| avg_latency > limit))
        {
            trace!("first hop latency too high");
            return false;
        }

        // At the moment, we do not allow circles because they
        // do not give additional privacy
        if current_path.contains(&next_hop.address) {
            trace!("circles not allowed");
            return false;
        }

        // We cannot use channels with zero stake, if configure so
        if !self.cfg.allow_zero_edge_weight && edge.channel.balance.is_zero() {
            trace!(%next_hop, "zero stake channels not allowed");
            return false;
        }

        trace!("usable node");
        true
    }
}

#[async_trait]
impl<CW> PathSelector for DfsPathSelector<CW>
where
    CW: EdgeWeighting<U256> + Send + Sync,
{
    /// Attempts to find a path with at least `min_hops` hops and at most `max_hops` hops
    /// that goes from `source` to `destination`. There does not need to be
    /// a payment channel to `destination`, so the path only includes intermediate hops.
    ///
    /// The function implements a randomized best-first search through the path space. The graph
    /// traversal is bounded by `self.max_iterations` to prevent from long-running path
    /// selection runs.
    async fn select_path(
        &self,
        source: Address,
        destination: Address,
        min_hops: usize,
        max_hops: usize,
    ) -> Result<ChannelPath> {
        // The protocol does not support >3 hop paths and will presumably never do,
        // so we can exclude it here.
        if !(1..=INTERMEDIATE_HOPS).contains(&max_hops) || !(1..=max_hops).contains(&min_hops) {
            return Err(GeneralError::InvalidInput.into());
        }

        let graph = self.graph.read().await;

        // Populate the queue with possible initial path offsprings
        let mut queue = graph
            .open_channels_from(source)
            .filter(|(node, edge)| self.is_next_hop_usable(node, edge, &source, &destination, &[]))
            .map(|(_, edge)| WeightedChannelPath::default().extend::<CW>(edge))
            .collect::<BinaryHeap<_>>();

        trace!(last_peer = %source, queue_len = queue.len(), "got next possible steps");

        let mut iters = 0;
        while let Some(mut current) = queue.pop() {
            let current_len = current.path.len();

            trace!(
                ?current,
                ?queue,
                queue_len = queue.len(),
                iters,
                min_hops,
                max_hops,
                "testing next path in queue"
            );
            if current_len == max_hops || current.fully_explored || iters > self.cfg.max_iterations {
                return if current_len >= min_hops && current_len <= max_hops {
                    Ok(ChannelPath::from_iter(current.path))
                } else {
                    trace!(current_len, min_hops, max_hops, iters, "path not found");
                    Err(PathError::PathNotFound(
                        max_hops,
                        source.to_string(),
                        destination.to_string(),
                    ))
                };
            }

            // Check for any acceptable path continuations
            let last_peer = *current.path.last().unwrap();
            let mut new_channels = graph
                .open_channels_from(last_peer)
                .filter(|(next_hop, edge)| {
                    self.is_next_hop_usable(next_hop, edge, &source, &destination, &current.path)
                })
                .peekable();

            // If there are more usable path continuations, add them all to the queue
            if new_channels.peek().is_some() {
                queue.extend(new_channels.map(|(_, edge)| current.clone().extend::<CW>(edge)));
                trace!(%last_peer, queue_len = queue.len(), "got next possible steps");
            } else {
                // No more possible continuations, mark this path as fully explored,
                // but push it into the queue
                // if we don't manage to find anything better
                current.fully_explored = true;
                trace!(path = ?current, "fully explored");
                queue.push(current);
            }

            iters += 1;
        }

        Err(PathError::PathNotFound(
            max_hops,
            source.to_string(),
            destination.to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::{ops::Deref, str::FromStr, sync::Arc};

    use async_lock::RwLock;
    use regex::Regex;

    use super::*;
    use crate::{
        ChainPath, Path, ValidatedPath,
        channel_graph::NodeScoreUpdate,
        tests::{ADDRESSES, PATH_ADDRS},
    };

    fn create_channel(src: Address, dst: Address, status: ChannelStatus, stake: HoprBalance) -> ChannelEntry {
        ChannelEntry::new(src, dst, stake, U256::zero(), status, U256::zero())
    }

    async fn check_path(path: &ChannelPath, graph: &ChannelGraph, dst: Address) -> anyhow::Result<()> {
        let _ = ValidatedPath::new(
            graph.my_address(),
            ChainPath::from_channel_path(path.clone(), dst),
            graph,
            PATH_ADDRS.deref(),
        )
        .await?;

        assert!(!path.contains_cycle(), "path must not be cyclic");
        assert!(!path.hops().contains(&dst), "path must not contain destination");

        Ok(())
    }

    /// Quickly define a graph with edge weights (channel stakes).
    /// Additional closures allow defining node qualities and edge scores.
    ///
    /// Syntax:
    /// `0 [1] -> 1` => edge from `0` to `1` with edge weight `1`
    /// `0 <- [1] 1` => edge from `1` to `0` with edge weight `1`
    /// `0 [1] <-> [2] 1` => edge from `0` to `1` with edge weight `1` and edge from `1` to `0` with edge weight `2`
    ///
    /// # Example
    /// ```ignore
    /// let star = define_graph(
    ///     "0 [1] <-> [2] 1, 0 [1] <-> [3] 2, 0 [1] <-> [4] 3, 0 [1] <-> [5] 4",
    ///     "0x1223d5786d9e6799b3297da1ad55605b91e2c882".parse().unwrap(),
    ///     |_| 1_f64,
    ///     None,
    /// );
    /// ```
    fn define_graph<Q, S>(def: &str, me: Address, quality: Q, score: S) -> ChannelGraph
    where
        Q: Fn(Address) -> f64,
        S: Fn(Address, Address) -> f64,
    {
        let mut graph = ChannelGraph::new(me, Default::default());

        if def.is_empty() {
            return graph;
        }

        let re: Regex = Regex::new(r"^\s*(\d+)\s*(\[\d+\])?\s*(<?->?)\s*(\[\d+\])?\s*(\d+)\s*$").unwrap();
        let re_stake = Regex::new(r"^\[(\d+)\]$").unwrap();

        let mut match_stake_and_update_channel = |src: Address, dest: Address, stake_str: &str| {
            let stake_caps = re_stake.captures(stake_str).unwrap();

            if stake_caps.get(0).is_none() {
                panic!("no matching stake. got {stake_str}");
            }
            graph.update_channel(create_channel(
                src,
                dest,
                ChannelStatus::Open,
                U256::from_str(stake_caps.get(1).unwrap().as_str())
                    .expect("failed to create U256 from given stake")
                    .into(),
            ));

            graph.update_node_score(
                &src,
                NodeScoreUpdate::Initialize(Duration::from_millis(10), quality(src)),
            );
            graph.update_node_score(
                &dest,
                NodeScoreUpdate::Initialize(Duration::from_millis(10), quality(dest)),
            );

            graph.update_channel_score(&src, &dest, score(src, dest));
        };

        for edge in def.split(",") {
            let caps = re.captures(edge).unwrap();

            if caps.get(0).is_none() {
                panic!("no matching edge. got `{edge}`");
            }

            let addr_a = ADDRESSES[usize::from_str(caps.get(1).unwrap().as_str()).unwrap()];
            let addr_b = ADDRESSES[usize::from_str(caps.get(5).unwrap().as_str()).unwrap()];

            let dir = caps.get(3).unwrap().as_str();

            match dir {
                "->" => {
                    if let Some(stake_b) = caps.get(4) {
                        panic!(
                            "Cannot assign stake for counterparty because channel is unidirectional. Got `{}`",
                            stake_b.as_str()
                        );
                    }

                    let stake_opt_a = caps.get(2).ok_or("missing stake for initiator").unwrap();

                    match_stake_and_update_channel(addr_a, addr_b, stake_opt_a.as_str());
                }
                "<-" => {
                    if let Some(stake_a) = caps.get(2) {
                        panic!(
                            "Cannot assign stake for counterparty because channel is unidirectional. Got `{}`",
                            stake_a.as_str()
                        );
                    }

                    let stake_opt_b = caps.get(4).ok_or("missing stake for counterparty").unwrap();

                    match_stake_and_update_channel(addr_b, addr_a, stake_opt_b.as_str());
                }
                "<->" => {
                    let stake_opt_a = caps.get(2).ok_or("missing stake for initiator").unwrap();

                    match_stake_and_update_channel(addr_a, addr_b, stake_opt_a.as_str());

                    let stake_opt_b = caps.get(4).ok_or("missing stake for counterparty").unwrap();

                    match_stake_and_update_channel(addr_b, addr_a, stake_opt_b.as_str());
                }
                _ => panic!("unknown direction infix"),
            };
        }

        graph
    }

    #[derive(Default)]
    pub struct TestWeights;
    impl EdgeWeighting<U256> for TestWeights {
        fn calculate_weight(edge: &ChannelEdge) -> U256 {
            edge.channel.balance.amount() + 1u32
        }
    }

    #[tokio::test]
    async fn test_should_not_find_path_if_isolated() {
        let graph = Arc::new(RwLock::new(define_graph("", ADDRESSES[0], |_| 1.0, |_, _| 0.0)));

        let selector = DfsPathSelector::<TestWeights>::new(graph.clone(), Default::default());

        selector
            .select_path(ADDRESSES[0], ADDRESSES[5], 1, 2)
            .await
            .expect_err("should not find a path");
    }

    #[tokio::test]
    async fn test_should_not_find_zero_weight_path() {
        let graph = Arc::new(RwLock::new(define_graph(
            "0 [0] -> 1",
            ADDRESSES[0],
            |_| 1.0,
            |_, _| 0.0,
        )));

        let selector = DfsPathSelector::<TestWeights>::new(graph.clone(), Default::default());

        selector
            .select_path(ADDRESSES[0], ADDRESSES[5], 1, 1)
            .await
            .expect_err("should not find a path");
    }

    #[tokio::test]
    async fn test_should_not_find_one_hop_path_when_unrelated_channels_are_in_the_graph() {
        let graph = Arc::new(RwLock::new(define_graph(
            "1 [1] -> 2",
            ADDRESSES[0],
            |_| 1.0,
            |_, _| 0.0,
        )));

        let selector = DfsPathSelector::<TestWeights>::new(graph.clone(), Default::default());

        selector
            .select_path(ADDRESSES[0], ADDRESSES[5], 1, 1)
            .await
            .expect_err("should not find a path");
    }

    #[tokio::test]
    async fn test_should_not_find_one_hop_path_in_empty_graph() {
        let graph = Arc::new(RwLock::new(define_graph("", ADDRESSES[0], |_| 1.0, |_, _| 0.0)));

        let selector = DfsPathSelector::<TestWeights>::new(graph.clone(), Default::default());

        selector
            .select_path(ADDRESSES[0], ADDRESSES[5], 1, 1)
            .await
            .expect_err("should not find a path");
    }

    #[tokio::test]
    async fn test_should_not_find_path_with_unreliable_node() {
        let graph = Arc::new(RwLock::new(define_graph(
            "0 [1] -> 1",
            ADDRESSES[0],
            |_| 0_f64,
            |_, _| 0.0,
        )));

        let selector = DfsPathSelector::<TestWeights>::new(graph.clone(), Default::default());

        selector
            .select_path(ADDRESSES[0], ADDRESSES[5], 1, 1)
            .await
            .expect_err("should not find a path");
    }

    #[tokio::test]
    async fn test_should_not_find_loopback_path() {
        let graph = Arc::new(RwLock::new(define_graph(
            "0 [1] <-> [1] 1",
            ADDRESSES[0],
            |_| 1_f64,
            |_, _| 0.0,
        )));

        let selector = DfsPathSelector::<TestWeights>::new(graph.clone(), Default::default());

        selector
            .select_path(ADDRESSES[0], ADDRESSES[5], 2, 2)
            .await
            .expect_err("should not find a path");
    }

    #[tokio::test]
    async fn test_should_not_include_destination_in_path() {
        let graph = Arc::new(RwLock::new(define_graph(
            "0 [1] -> 1",
            ADDRESSES[0],
            |_| 1_f64,
            |_, _| 0.0,
        )));

        let selector = DfsPathSelector::<TestWeights>::new(graph.clone(), Default::default());

        selector
            .select_path(ADDRESSES[0], ADDRESSES[1], 1, 1)
            .await
            .expect_err("should not find a path");
    }

    #[tokio::test]
    async fn test_should_find_path_in_reliable_star() -> anyhow::Result<()> {
        let graph = Arc::new(RwLock::new(define_graph(
            "0 [1] <-> [2] 1, 0 [1] <-> [3] 2, 0 [1] <-> [4] 3, 0 [1] <-> [5] 4",
            ADDRESSES[1],
            |_| 1_f64,
            |_, _| 0.0,
        )));

        let selector = DfsPathSelector::<TestWeights>::new(graph.clone(), Default::default());
        let path = selector.select_path(ADDRESSES[1], ADDRESSES[5], 1, 2).await?;

        check_path(&path, graph.read().await.deref(), ADDRESSES[5]).await?;
        assert_eq!(2, path.num_hops(), "should have 2 hops");

        Ok(())
    }

    #[tokio::test]
    async fn test_should_find_path_in_reliable_arrow_with_lower_weight() -> anyhow::Result<()> {
        let graph = Arc::new(RwLock::new(define_graph(
            "0 [1] -> 1, 1 [1] -> 2, 2 [1] -> 3, 1 [1] -> 3",
            ADDRESSES[0],
            |_| 1_f64,
            |_, _| 0.0,
        )));
        let selector = DfsPathSelector::<TestWeights>::new(graph.clone(), Default::default());

        let path = selector.select_path(ADDRESSES[0], ADDRESSES[5], 3, 3).await?;
        check_path(&path, graph.read().await.deref(), ADDRESSES[5]).await?;
        assert_eq!(3, path.num_hops(), "should have 3 hops");

        Ok(())
    }

    #[tokio::test]
    async fn test_should_find_path_in_reliable_arrow_with_higher_weight() -> anyhow::Result<()> {
        let graph = Arc::new(RwLock::new(define_graph(
            "0 [1] -> 1, 1 [2] -> 2, 2 [3] -> 3, 1 [2] -> 3",
            ADDRESSES[0],
            |_| 1_f64,
            |_, _| 0.0,
        )));
        let selector = DfsPathSelector::<TestWeights>::new(graph.clone(), Default::default());

        let path = selector.select_path(ADDRESSES[0], ADDRESSES[5], 3, 3).await?;
        check_path(&path, graph.read().await.deref(), ADDRESSES[5]).await?;
        assert_eq!(3, path.num_hops(), "should have 3 hops");

        Ok(())
    }

    #[tokio::test]
    async fn test_should_find_path_in_reliable_arrow_with_random_weight() -> anyhow::Result<()> {
        let graph = Arc::new(RwLock::new(define_graph(
            "0 [29] -> 1, 1 [5] -> 2, 2 [31] -> 3, 1 [2] -> 3",
            ADDRESSES[0],
            |_| 1_f64,
            |_, _| 0.0,
        )));
        let selector = DfsPathSelector::<RandomizedEdgeWeighting>::new(graph.clone(), Default::default());

        let path = selector.select_path(ADDRESSES[0], ADDRESSES[5], 3, 3).await?;
        check_path(&path, graph.read().await.deref(), ADDRESSES[5]).await?;
        assert_eq!(3, path.num_hops(), "should have 3 hops");

        Ok(())
    }
}
