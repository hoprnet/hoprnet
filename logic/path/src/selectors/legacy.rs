use crate::channel_graph::{ChannelEdge, ChannelGraph};
use crate::errors::{PathError, Result};
use crate::path::ChannelPath;
use crate::selectors::{EdgeWeighting, PathSelector};
use core_types::channels::ChannelEntry;
use core_types::protocol::INTERMEDIATE_HOPS;
use hopr_crypto::random::random_float;
use petgraph::visit::EdgeRef;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::marker::PhantomData;
use utils_types::errors::GeneralError::InvalidInput;
use utils_types::primitives::{Address, U256};

/// Holds a weighted channel path and auxiliary information
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
    /// Favor not expored paths over fully explored paths,
    /// there could be better ones.
    ///
    /// Favor longer paths over shorter paths, longer path
    /// means more privacy.
    ///
    /// If both parts are of the same length, favor paths
    /// with higher weights.
    fn cmp(&self, other: &Self) -> Ordering {
        if other.fully_explored == self.fully_explored {
            match self.path.len().cmp(&other.path.len()) {
                Ordering::Equal => self.weight.cmp(&other.weight),
                Ordering::Greater => Ordering::Greater,
                Ordering::Less => Ordering::Less,
            }
        } else if other.fully_explored && !self.fully_explored {
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
    /// Given that the float are uniform, nodes with higher stake have a higher
    /// probability of reaching a higher value.
    ///
    /// Sorting the list of weights thus moves nodes with higher stakes more
    /// often to the front.
    fn calculate_weight(channel: &ChannelEntry) -> U256 {
        channel
            .balance
            .value()
            .multiply_f64(random_float())
            .expect("Could not multiply edge weight with float")
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

        true
    }
}

impl<CW> PathSelector<CW> for DfsPathSelector<CW>
where
    CW: EdgeWeighting<U256>,
{
    fn select_path(
        &self,
        graph: &ChannelGraph,
        source: Address,
        destination: Address,
        min_hops: usize,
        max_hops: usize,
    ) -> Result<ChannelPath> {
        if !(1..=INTERMEDIATE_HOPS).contains(&max_hops) {
            return Err(InvalidInput.into());
        }

        let mut queue = BinaryHeap::new();

        graph
            .open_channels_from(source)
            .filter_map(|channel| {
                let w = channel.weight();
                self.filter_channel(w, &source, &destination, &[])
                    .then(|| WeightedChannelPath {
                        path: vec![w.channel.destination],
                        weight: CW::calculate_weight(&w.channel),
                        fully_explored: false,
                    })
            })
            .for_each(|wcp| queue.push(wcp));

        let mut iters = 0;
        while !queue.is_empty() && iters < self.max_iterations {
            let current_path = queue.peek().unwrap().clone();
            let current_path_len = current_path.path.len();

            if current_path_len >= max_hops {
                return Ok(ChannelPath::new_valid(queue.pop().unwrap().path));
            }

            if current_path.fully_explored {
                if current_path_len >= min_hops {
                    return Ok(ChannelPath::new_valid(queue.pop().unwrap().path));
                } else {
                    return Err(PathError::PathNotFound(
                        max_hops,
                        source.to_string(),
                        destination.to_string(),
                    ));
                }
            }

            let last_peer = *current_path.path.last().unwrap();
            let new_channels = graph
                .open_channels_from(last_peer)
                .filter(|channel| self.filter_channel(channel.weight(), &source, &destination, &current_path.path))
                .collect::<Vec<_>>();

            if !new_channels.is_empty() {
                // There exists a longer paths, so drop current path
                queue.pop();

                for new_channel in new_channels {
                    let mut next_path_variant = current_path.clone();
                    next_path_variant.path.push(new_channel.weight().channel.destination);
                    next_path_variant.weight += CW::calculate_weight(&new_channel.weight().channel);
                    next_path_variant.fully_explored = false;

                    queue.push(next_path_variant);
                }
            } else {
                // Keep the current path in case we do not find anything better
                let mut current_path = queue.peek_mut().unwrap();
                current_path.fully_explored = true;
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
    use crate::selectors::{EdgeWeighting, PathSelector};
    use core_types::channels::{ChannelEntry, ChannelStatus};
    use lazy_static::lazy_static;
    use std::str::FromStr;
    use utils_types::primitives::{Address, Balance, BalanceType, U256};

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

    fn initialize_star_graph<Q, B>(me: Address, stake: B, quality: Q) -> ChannelGraph
    where
        Q: Fn(Address, Address) -> f64,
        B: Fn(Address, Address) -> Balance,
    {
        let mut graph = ChannelGraph::new(me);

        graph.update_channel(create_channel(
            ADDRESSES[1],
            ADDRESSES[0],
            ChannelStatus::Open,
            stake(ADDRESSES[1], ADDRESSES[0]),
        ));
        graph.update_channel_quality(ADDRESSES[1], ADDRESSES[0], quality(ADDRESSES[1], ADDRESSES[0]));

        graph.update_channel(create_channel(
            ADDRESSES[2],
            ADDRESSES[0],
            ChannelStatus::Open,
            stake(ADDRESSES[2], ADDRESSES[0]),
        ));
        graph.update_channel_quality(ADDRESSES[2], ADDRESSES[0], quality(ADDRESSES[2], ADDRESSES[0]));

        graph.update_channel(create_channel(
            ADDRESSES[3],
            ADDRESSES[0],
            ChannelStatus::Open,
            stake(ADDRESSES[3], ADDRESSES[0]),
        ));
        graph.update_channel_quality(ADDRESSES[3], ADDRESSES[0], quality(ADDRESSES[3], ADDRESSES[0]));

        graph.update_channel(create_channel(
            ADDRESSES[4],
            ADDRESSES[0],
            ChannelStatus::Open,
            stake(ADDRESSES[4], ADDRESSES[0]),
        ));
        graph.update_channel_quality(ADDRESSES[4], ADDRESSES[0], quality(ADDRESSES[4], ADDRESSES[0]));

        graph.update_channel(create_channel(
            ADDRESSES[0],
            ADDRESSES[1],
            ChannelStatus::Open,
            stake(ADDRESSES[0], ADDRESSES[1]),
        ));
        graph.update_channel_quality(ADDRESSES[0], ADDRESSES[1], quality(ADDRESSES[0], ADDRESSES[1]));

        graph.update_channel(create_channel(
            ADDRESSES[0],
            ADDRESSES[2],
            ChannelStatus::Open,
            stake(ADDRESSES[0], ADDRESSES[2]),
        ));
        graph.update_channel_quality(ADDRESSES[0], ADDRESSES[2], quality(ADDRESSES[0], ADDRESSES[2]));

        graph.update_channel(create_channel(
            ADDRESSES[0],
            ADDRESSES[3],
            ChannelStatus::Open,
            stake(ADDRESSES[0], ADDRESSES[3]),
        ));
        graph.update_channel_quality(ADDRESSES[0], ADDRESSES[3], quality(ADDRESSES[0], ADDRESSES[3]));

        graph.update_channel(create_channel(
            ADDRESSES[0],
            ADDRESSES[4],
            ChannelStatus::Open,
            stake(ADDRESSES[0], ADDRESSES[4]),
        ));
        graph.update_channel_quality(ADDRESSES[0], ADDRESSES[4], quality(ADDRESSES[0], ADDRESSES[4]));

        graph
    }

    fn initialize_arrow_graph<Q, B>(me: Address, stake: B, quality: Q) -> ChannelGraph
    where
        Q: Fn(Address, Address) -> f64,
        B: Fn(Address, Address) -> Balance,
    {
        let mut graph = ChannelGraph::new(me);

        graph.update_channel(create_channel(
            ADDRESSES[0],
            ADDRESSES[1],
            ChannelStatus::Open,
            stake(ADDRESSES[0], ADDRESSES[1]),
        ));
        graph.update_channel_quality(ADDRESSES[0], ADDRESSES[1], quality(ADDRESSES[0], ADDRESSES[1]));

        graph.update_channel(create_channel(
            ADDRESSES[1],
            ADDRESSES[2],
            ChannelStatus::Open,
            stake(ADDRESSES[1], ADDRESSES[2]),
        ));
        graph.update_channel_quality(ADDRESSES[1], ADDRESSES[2], quality(ADDRESSES[1], ADDRESSES[2]));

        graph.update_channel(create_channel(
            ADDRESSES[2],
            ADDRESSES[3],
            ChannelStatus::Open,
            stake(ADDRESSES[2], ADDRESSES[3]),
        ));
        graph.update_channel_quality(ADDRESSES[2], ADDRESSES[3], quality(ADDRESSES[2], ADDRESSES[3]));

        graph
    }

    fn check_path(path: &ChannelPath, graph: &ChannelGraph, dst: Address) {
        let other = ChannelPath::new(path.hops().into(), graph).expect("path must be valid");
        assert_eq!(other, path.clone(), "valid paths must be equal");
        assert!(!path.contains_cycle(), "path must not be cyclic");
        assert!(!path.hops().contains(&dst), "path must not contain destination");
    }

    pub struct TestWeights;
    impl EdgeWeighting<U256> for TestWeights {
        fn calculate_weight(channel: &ChannelEntry) -> U256 {
            *channel.balance.value() + 1u32
        }
    }

    #[test]
    fn test_dfs_should_find_path_in_reliable_star() {
        let star = initialize_star_graph(
            ADDRESSES[1],
            |_, _| Balance::new(1u32.into(), BalanceType::HOPR),
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
    fn test_dfs_should_find_most_valuable_path_in_reliable_star() {
        let star = initialize_star_graph(
            ADDRESSES[1],
            |_, b| {
                Balance::new(
                    (ADDRESSES.iter().position(|a| b.eq(a)).unwrap() as u32 + 1).into(),
                    BalanceType::HOPR,
                )
            },
            |_, _| 1_f64,
        );

        let selector = DfsPathSelector::<TestWeights>::default();
        let path = selector
            .select_path(&star, ADDRESSES[1], ADDRESSES[5], 1, 2)
            .expect("should find a path");

        check_path(&path, &star, ADDRESSES[5]);
        assert_eq!(2, path.length(), "should have 2 hops");
        assert_eq!(path.hops()[1], ADDRESSES[4], "last hop should be the most valuable one");
    }

    #[test]
    fn test_dfs_should_not_find_path_when_does_not_exist() {
        let star = initialize_star_graph(
            ADDRESSES[1],
            |_, _| Balance::new(1u32.into(), BalanceType::HOPR),
            |_, _| 1_f64,
        );
        let selector = DfsPathSelector::<TestWeights>::default();

        selector
            .select_path(&star, ADDRESSES[1], ADDRESSES[5], 1, 3)
            .expect_err("should not find a path 1");
        selector
            .select_path(&star, ADDRESSES[5], ADDRESSES[0], 1, 3)
            .expect_err("should not find a path 2");
    }

    #[test]
    fn test_dfs_should_find_path_in_reliable_arrow() {
        let arrow = initialize_arrow_graph(
            ADDRESSES[0],
            |_, _| Balance::new(1u32.into(), BalanceType::HOPR),
            |_, _| 1_f64,
        );
        let selector = DfsPathSelector::<TestWeights>::default();

        let path = selector
            .select_path(&arrow, ADDRESSES[0], ADDRESSES[5], 1, 3)
            .expect("should find a path");
        check_path(&path, &arrow, ADDRESSES[5]);
        assert_eq!(3, path.length(), "should have 3 hops");
    }

    #[test]
    fn test_dfs_should_not_find_path_if_unreliable_node_in_arrow() {
        let arrow = initialize_arrow_graph(
            ADDRESSES[0],
            |_, _| Balance::new(1u32.into(), BalanceType::HOPR),
            |_, dst| if dst == ADDRESSES[3] { 0.1_f64 } else { 1_f64 },
        ); // node 3 is unreliable

        let selector = DfsPathSelector::<TestWeights>::default();
        selector
            .select_path(&arrow, ADDRESSES[0], ADDRESSES[5], 1, 3)
            .expect_err("should not find a path");
    }
}
