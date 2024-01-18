use crate::channel_graph::{ChannelEdge, ChannelGraph};
use crate::errors::{PathError, Result};
use crate::path::ChannelPath;
use crate::selectors::{EdgeWeighting, PathSelector};
use hopr_crypto_random::random_float;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use log::warn;
use petgraph::visit::EdgeRef;
use std::cmp::{max, Ordering};
use std::collections::BinaryHeap;
use std::marker::PhantomData;

/// Holds a weighted channel path and auxiliary information for graph traversal.
#[derive(Clone, Debug, PartialEq, Eq)]
struct WeightedChannelPath {
    path: Vec<Address>,
    weight: U256,
    fully_explored: bool,
}

impl PartialOrd for WeightedChannelPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WeightedChannelPath {
    /// Favors unexplored paths over fully explored paths even when a better
    /// alternative exists.
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
                Ordering::Greater => Ordering::Greater,
                Ordering::Less => Ordering::Less,
            }
        } else if other.fully_explored {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

/// Assigns each channel a weight.
/// The weight is randomized such that not always the same
/// nodes get selected. This is necessary to achieve privacy.
/// It also favors nodes with higher stake.
pub struct RandomizedEdgeWeighting;

impl EdgeWeighting<U256> for RandomizedEdgeWeighting {
    /// Multiply all channel stake with a random float in the interval (0,1].
    /// Given that the floats are uniform, nodes with higher stake have a higher
    /// probability of reaching a higher value.
    ///
    /// Sorting the list of weights thus moves nodes with higher stakes more
    /// often to the front.
    fn calculate_weight(channel: &ChannelEntry) -> U256 {
        max(
            U256::one(),
            channel
                .balance
                .amount()
                .mul_f64(random_float())
                .expect("Could not multiply edge weight with float"),
        )
    }
}

/// Legacy path selector using depth-first search of the channel graph.
#[derive(Clone, Debug)]
pub struct DfsPathSelector<CW>
where
    CW: EdgeWeighting<U256>,
{
    /// Maximum number of iterations before a path selection fails
    /// Default is 100
    pub max_iterations: usize,
    /// Peer quality threshold for a channel to be taken into account.
    /// Default is 0.5
    pub quality_threshold: f64,
    cw: PhantomData<CW>,
}

impl<CW> Default for DfsPathSelector<CW>
where
    CW: EdgeWeighting<U256>,
{
    fn default() -> Self {
        Self {
            max_iterations: 100,
            quality_threshold: 0.5_f64,
            cw: PhantomData,
        }
    }
}

impl<CW> DfsPathSelector<CW>
where
    CW: EdgeWeighting<U256>,
{
    /// Determines whether a node is considered "interesting".
    ///
    /// To achieve privacy, we need at least one honest node along
    /// the path. Each additional node decreases the probability of
    /// having only malicious nodes, so we can sort out many nodes.
    /// Nodes that have shown to be unreliable are of no use, so
    /// drop them.
    fn filter_channel(
        &self,
        channel: &ChannelEdge,
        source: &Address,
        destination: &Address,
        current_path: &[Address],
    ) -> bool {
        if source.eq(&channel.channel.destination) {
            // looping back to self does not give any privacy
            return false;
        }

        if destination.eq(&channel.channel.destination) {
            // We cannot use destination as last intermediate hop as
            // this would be a loopback which does not give any privacy
            return false;
        }

        // Check node reliability, new nodes are considered reliable
        // unless the opposite has been observed
        if channel.quality.unwrap_or(1.0f64) < self.quality_threshold {
            // Only use nodes that have shown to be somewhat reliable
            return false;
        }

        if current_path.contains(&channel.channel.destination) {
            // At the moment, we do not allow circles because they
            // do not give additional privacy
            return false;
        }

        if U256::zero().eq(&channel.channel.balance.amount()) {
            // We cannot use channels with zero stake
            return false;
        }

        true
    }
}

impl<CW> PathSelector<CW> for DfsPathSelector<CW>
where
    CW: EdgeWeighting<U256>,
{
    /// Attempts to find a path with at least `min_hops` hops and at most `max_hops` hops
    /// that goes from `source` to `destination`. There does not need to be a
    /// a payment channel to to `destination`, so the path only includes intermediate hops.
    ///
    /// Implements a randomized best-first search through the path space. The graph
    /// traversal is bounded by `self.max_iterations` to prevent from long-running path
    /// selection runs.
    fn select_path(
        &self,
        graph: &ChannelGraph,
        source: Address,
        destination: Address,
        min_hops: usize,
        max_hops: usize,
    ) -> Result<ChannelPath> {
        if !(1..=INTERMEDIATE_HOPS).contains(&max_hops) {
            return Err(GeneralError::InvalidInput.into());
        }

        if min_hops > max_hops || min_hops == 0 {
            return Err(GeneralError::InvalidInput.into());
        }

        let mut queue = BinaryHeap::new();

        queue.extend(graph.open_channels_from(source).filter_map(|channel| {
            let w = channel.weight();
            self.filter_channel(w, &source, &destination, &[])
                .then(|| WeightedChannelPath {
                    path: vec![w.channel.destination],
                    weight: CW::calculate_weight(&w.channel),
                    fully_explored: false,
                })
        }));

        let mut iters = 0;
        while let Some(mut current_path) = queue.pop() {
            // This should not happen. Retrying can help here.
            if iters > self.max_iterations {
                warn!("Could not find a path from {} to {} with at least {} hops and at most {} hops within {} iterations", source, destination, min_hops, max_hops, self.max_iterations);
                break;
            }

            let current_path_len = current_path.path.len();

            if current_path_len >= max_hops {
                return Ok(ChannelPath::new_valid(current_path.path));
            }

            if current_path.fully_explored {
                if current_path_len >= min_hops {
                    return Ok(ChannelPath::new_valid(current_path.path));
                } else {
                    return Err(PathError::PathNotFound(
                        max_hops,
                        source.to_string(),
                        destination.to_string(),
                    ));
                }
            }

            let last_peer = *current_path.path.last().unwrap();
            let mut new_channels = graph
                .open_channels_from(last_peer)
                .filter(|channel| self.filter_channel(channel.weight(), &source, &destination, &current_path.path))
                .peekable();

            if new_channels.peek().is_some() {
                queue.extend(new_channels.map(|new_channel| {
                    let mut next_path_variant = WeightedChannelPath {
                        path: Vec::with_capacity(current_path_len + 1),
                        weight: current_path.weight + CW::calculate_weight(&new_channel.weight().channel),
                        fully_explored: false,
                    };
                    next_path_variant.path.extend(&current_path.path);
                    next_path_variant.path.push(new_channel.weight().channel.destination);

                    next_path_variant
                }));
            } else {
                current_path.fully_explored = true;
                // Keep the current path in case we do not find anything better
                queue.push(current_path);
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

/// Legacy DFS path selector with channel weighting function
/// that uses randomized channel stakes as edge weights.
pub type LegacyPathSelector = DfsPathSelector<RandomizedEdgeWeighting>;

#[cfg(test)]
mod tests {
    use crate::channel_graph::ChannelGraph;
    use crate::path::{ChannelPath, Path};
    use crate::selectors::legacy::DfsPathSelector;
    use crate::selectors::legacy::RandomizedEdgeWeighting;
    use crate::selectors::{EdgeWeighting, PathSelector};
    use core::panic;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use lazy_static::lazy_static;
    use regex::Regex;
    use std::str::FromStr;

    lazy_static! {
        static ref ADDRESSES: [Address; 6] = [
            Address::from_str("0xafe8c178cf70d966be0a798e666ce2782c7b2288").unwrap(),
            Address::from_str("0x1223d5786d9e6799b3297da1ad55605b91e2c882").unwrap(),
            Address::from_str("0x0e3e60ddced1e33c9647a71f4fc2cf4ed33e4a9d").unwrap(),
            Address::from_str("0x27644105095c8c10f804109b4d1199a9ac40ed46").unwrap(),
            Address::from_str("0x4701a288c38fa8a0f4b79127747257af4a03a623").unwrap(),
            Address::from_str("0xfddd2f462ec709cf181bbe44a7e952487bd4591d").unwrap(),
        ];
    }

    fn create_channel(src: Address, dst: Address, status: ChannelStatus, stake: Balance) -> ChannelEntry {
        ChannelEntry::new(src, dst, stake, U256::zero(), status, U256::zero(), U256::zero())
    }

    fn check_path(path: &ChannelPath, graph: &ChannelGraph, dst: Address) {
        let other = ChannelPath::new(path.hops().into(), graph).expect("path must be valid");
        assert_eq!(other, path.clone(), "valid paths must be equal");
        assert!(!path.contains_cycle(), "path must not be cyclic");
        assert!(!path.hops().contains(&dst), "path must not contain destination");
    }

    /// Quickly define a graph with edge weights.
    /// Syntax:
    /// `0 [1] -> 1` => edge from `0` to `1` with edge weight `1`
    /// `0 <- [1] 1` => edge from `1` to `0` with edge weight `1`
    /// `0 [1] <-> [2] 1` => edge from `0` to `1` with edge weight `1` and edge from `1` to `0` with edge weight `2`
    /// ```
    /// let star = define_graph(
    ///     "0 [1] <-> [2] 1, 0 [1] <-> [3] 2, 0 [1] <-> [4] 3, 0 [1] <-> [5] 4",
    ///     ADDRESSES[1],
    ///     |_, _| 1_f64,
    /// );
    /// ```
    fn define_graph<Q>(def: &str, me: Address, quality: Q) -> ChannelGraph
    where
        Q: Fn(Address, Address) -> f64,
    {
        let mut graph = ChannelGraph::new(me);

        if def.is_empty() {
            return graph;
        }

        let re: Regex = Regex::new(r"^\s*(\d+)\s*(\[\d+\])?\s*(<?->?)\s*(\[\d+\])?\s*(\d+)\s*$").unwrap();
        let re_stake = Regex::new(r"^\[(\d+)\]$").unwrap();

        let mut match_stake_and_update_channel = |src: Address, dest: Address, stake_str: &str| {
            let stake_caps = re_stake.captures(stake_str).unwrap();

            if stake_caps.get(0).is_none() {
                panic!("no matching stake. got {}", stake_str);
            }
            graph.update_channel(create_channel(
                src,
                dest,
                ChannelStatus::Open,
                Balance::new(
                    U256::from_str(stake_caps.get(1).unwrap().as_str())
                        .expect("failed to create U256 from given stake"),
                    BalanceType::HOPR,
                ),
            ));

            graph.update_channel_quality(src, dest, quality(src, dest));
        };

        for edge in def.split(",") {
            let caps = re.captures(edge).unwrap();

            if caps.get(0).is_none() {
                panic!("no matching edge. got `{}`", edge);
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

    pub struct TestWeights;
    impl EdgeWeighting<U256> for TestWeights {
        fn calculate_weight(channel: &ChannelEntry) -> U256 {
            channel.balance.amount() + 1u32
        }
    }

    #[test]
    fn test_should_not_find_path_if_isolated() {
        let isolated = define_graph("", ADDRESSES[0], |_, _| 1_f64);

        let selector = DfsPathSelector::<TestWeights>::default();

        selector
            .select_path(&isolated, ADDRESSES[0], ADDRESSES[5], 1, 2)
            .expect_err("should not find a path");
    }

    #[test]
    fn test_should_not_find_zero_path() {
        let isolated = define_graph("0 [0] -> 1", ADDRESSES[0], |_, _| 1_f64);

        let selector = DfsPathSelector::<TestWeights>::default();

        selector
            .select_path(&isolated, ADDRESSES[0], ADDRESSES[5], 1, 1)
            .expect_err("should not find a path");
    }

    #[test]
    fn test_should_not_find_unreliable_path() {
        let isolated = define_graph("0 [1] -> 1", ADDRESSES[0], |_, _| 0_f64);

        let selector = DfsPathSelector::<TestWeights>::default();

        selector
            .select_path(&isolated, ADDRESSES[0], ADDRESSES[5], 1, 1)
            .expect_err("should not find a path");
    }

    #[test]
    fn test_should_not_find_loopback_path() {
        let isolated = define_graph("0 [1] <-> [1] 1", ADDRESSES[0], |_, _| 1_f64);

        let selector = DfsPathSelector::<TestWeights>::default();

        selector
            .select_path(&isolated, ADDRESSES[0], ADDRESSES[5], 2, 2)
            .expect_err("should not find a path");
    }

    #[test]
    fn test_should_not_include_destination_in_path() {
        let isolated = define_graph("0 [1] -> 1", ADDRESSES[0], |_, _| 1_f64);

        let selector = DfsPathSelector::<TestWeights>::default();

        selector
            .select_path(&isolated, ADDRESSES[0], ADDRESSES[1], 1, 1)
            .expect_err("should not find a path");
    }

    #[test]
    fn test_dfs_should_find_path_in_reliable_star() {
        let star = define_graph(
            "0 [1] <-> [2] 1, 0 [1] <-> [3] 2, 0 [1] <-> [4] 3, 0 [1] <-> [5] 4",
            ADDRESSES[1],
            |_, _| 1_f64,
        );

        let selector = DfsPathSelector::<TestWeights>::default();
        let path = selector
            .select_path(&star, ADDRESSES[1], ADDRESSES[5], 1, 2)
            .expect("should find a path");

        check_path(&path, &star, ADDRESSES[5]);
        assert_eq!(2, path.length(), "should have 2 hops");
    }

    #[test]
    fn test_dfs_should_find_path_in_reliable_arrow_with_lower_weight() {
        let arrow = define_graph(
            "0 [1] -> 1, 1 [1] -> 2, 2 [1] -> 3, 1 [1] -> 3",
            ADDRESSES[0],
            |_, _| 1_f64,
        );
        let selector = DfsPathSelector::<TestWeights>::default();

        let path = selector
            .select_path(&arrow, ADDRESSES[0], ADDRESSES[5], 3, 3)
            .expect("should find a path");
        check_path(&path, &arrow, ADDRESSES[5]);
        assert_eq!(3, path.length(), "should have 3 hops");
    }

    #[test]
    fn test_dfs_should_find_path_in_reliable_arrow_with_higher_weight() {
        let arrow = define_graph(
            "0 [1] -> 1, 1 [2] -> 2, 2 [3] -> 3, 1 [2] -> 3",
            ADDRESSES[0],
            |_, _| 1_f64,
        );
        let selector = DfsPathSelector::<TestWeights>::default();

        let path = selector
            .select_path(&arrow, ADDRESSES[0], ADDRESSES[5], 3, 3)
            .expect("should find a path");
        check_path(&path, &arrow, ADDRESSES[5]);
        assert_eq!(3, path.length(), "should have 3 hops");
    }

    #[test]
    fn test_dfs_should_find_path_in_reliable_arrow_with_random_weight() {
        let arrow = define_graph(
            "0 [29] -> 1, 1 [5] -> 2, 2 [31] -> 3, 1 [2] -> 3",
            ADDRESSES[0],
            |_, _| 1_f64,
        );
        let selector = DfsPathSelector::<RandomizedEdgeWeighting>::default();

        let path = selector
            .select_path(&arrow, ADDRESSES[0], ADDRESSES[5], 3, 3)
            .expect("should find a path");
        check_path(&path, &arrow, ADDRESSES[5]);
        assert_eq!(3, path.length(), "should have 3 hops");
    }
}
