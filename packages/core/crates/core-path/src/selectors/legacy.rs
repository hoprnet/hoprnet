use crate::channel_graph::{ChannelEdge, ChannelGraph};
use crate::errors::{PathError, Result};
use crate::path::ChannelPath;
use crate::selectors::{EdgeWeighting, PathSelector};
use core_crypto::random::random_float;
use core_types::channels::ChannelEntry;
use core_types::protocol::INTERMEDIATE_HOPS;
use petgraph::visit::EdgeRef;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::marker::PhantomData;
use std::ops::Add;
use utils_types::errors::GeneralError::InvalidInput;
use utils_types::primitives::{Address, U256};

#[derive(Clone, Debug, PartialEq, Eq)]
struct WeightedChannelPath(Vec<Address>, U256);

impl PartialOrd for WeightedChannelPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.1.partial_cmp(&other.1)
    }
}

impl Ord for WeightedChannelPath {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// Calculates the channel weight as (channel stake + 1) * (1 + R)
/// where R is uniform random in (0,1]
pub struct RandomizedEdgeWeighting;

impl EdgeWeighting<U256> for RandomizedEdgeWeighting {
    fn calculate_weight(channel: &ChannelEntry) -> U256 {
        const PATH_RANDOMNESS: f64 = 0.1;

        let r = random_float() * PATH_RANDOMNESS;
        let base = channel.balance.value().addn(1);

        base.add(base.multiply_f64(r).unwrap())
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
    fn filter_channel(
        &self,
        channel: &ChannelEdge,
        destination: &Address,
        current: &[Address],
        dead_ends: &HashSet<Address>,
    ) -> bool {
        !destination.eq(&channel.channel.destination) &&               // last hop does not need channel
            channel.quality.unwrap_or(0_f64) > self.quality_threshold &&  // quality threshold
            !current.contains(&channel.channel.destination) &&     // must not be in the path already (no loops allowed)
            !dead_ends.contains(&channel.channel.destination) // must not be in the dead end list
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
        max_hops: usize,
    ) -> Result<ChannelPath> {
        if !(1..=INTERMEDIATE_HOPS).contains(&max_hops) {
            return Err(InvalidInput.into())
        }

        let mut queue = BinaryHeap::new();
        let mut dead_ends = HashSet::new();

        graph
            .open_channels_from(source)
            .filter_map(|channel| {
                let w = channel.weight();
                self.filter_channel(w, &destination, &[], &dead_ends)
                    .then(|| WeightedChannelPath(vec![w.channel.destination], CW::calculate_weight(&w.channel)))
            })
            .for_each(|wcp| queue.push(wcp));

        let mut iters = 0;
        while !queue.is_empty() && iters < self.max_iterations {
            let current_path = queue.peek().unwrap();
            let current_path_len = current_path.0.len();

            if current_path_len >= max_hops && current_path_len <= INTERMEDIATE_HOPS {
                let mut res = queue.pop().unwrap().0;
                res.push(destination); // Need to add the destination address
                return Ok(ChannelPath::new_valid(res));
            }

            let last_peer = *current_path.0.last().unwrap();
            let new_channels = graph
                .open_channels_from(last_peer)
                .filter(|channel| self.filter_channel(channel.weight(), &destination, &current_path.0, &dead_ends))
                .collect::<Vec<_>>();

            if !new_channels.is_empty() {
                let current_path_clone = current_path.clone();
                for new_channel in new_channels {
                    let mut next_path_variant = current_path_clone.clone();
                    next_path_variant.0.push(new_channel.weight().channel.destination);
                    next_path_variant.1 += CW::calculate_weight(&new_channel.weight().channel);
                    queue.push(next_path_variant);
                }
            } else {
                queue.pop();
                dead_ends.insert(last_peer);
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
    use core_types::channels::{ChannelEntry, ChannelStatus};
    use lazy_static::lazy_static;
    use std::str::FromStr;
    use utils_types::primitives::{Address, Balance, U256};

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

    fn initialize_star_graph<F>(stake: Balance, quality: F) -> ChannelGraph
    where
        F: Fn(Address, Address) -> f64,
    {
        let mut graph = ChannelGraph::new(ADDRESSES[0]);

        graph.update_channel(create_channel(ADDRESSES[1], ADDRESSES[0], ChannelStatus::Open, stake));
        graph.update_channel_quality(ADDRESSES[1], ADDRESSES[0], quality(ADDRESSES[1], ADDRESSES[0]));

        graph.update_channel(create_channel(ADDRESSES[2], ADDRESSES[0], ChannelStatus::Open, stake));
        graph.update_channel_quality(ADDRESSES[2], ADDRESSES[0], quality(ADDRESSES[2], ADDRESSES[0]));

        graph.update_channel(create_channel(ADDRESSES[3], ADDRESSES[0], ChannelStatus::Open, stake));
        graph.update_channel_quality(ADDRESSES[3], ADDRESSES[0], quality(ADDRESSES[3], ADDRESSES[0]));

        graph.update_channel(create_channel(ADDRESSES[4], ADDRESSES[0], ChannelStatus::Open, stake));
        graph.update_channel_quality(ADDRESSES[4], ADDRESSES[0], quality(ADDRESSES[4], ADDRESSES[0]));

        graph.update_channel(create_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::Open, stake));
        graph.update_channel_quality(ADDRESSES[0], ADDRESSES[1], quality(ADDRESSES[0], ADDRESSES[1]));

        graph.update_channel(create_channel(ADDRESSES[0], ADDRESSES[2], ChannelStatus::Open, stake));
        graph.update_channel_quality(ADDRESSES[0], ADDRESSES[2], quality(ADDRESSES[0], ADDRESSES[2]));

        graph.update_channel(create_channel(ADDRESSES[0], ADDRESSES[3], ChannelStatus::Open, stake));
        graph.update_channel_quality(ADDRESSES[0], ADDRESSES[3], quality(ADDRESSES[0], ADDRESSES[3]));

        graph.update_channel(create_channel(ADDRESSES[0], ADDRESSES[4], ChannelStatus::Open, stake));
        graph.update_channel_quality(ADDRESSES[0], ADDRESSES[4], quality(ADDRESSES[0], ADDRESSES[4]));

        graph
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::channel_graph::wasm::ChannelGraph;
    use crate::path::wasm::PathResolver;
    use crate::path::Path;
    use crate::selectors::legacy::LegacyPathSelector;
    use crate::selectors::PathSelector;
    use core_ethereum_db::db::wasm::Database;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::Address;
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    pub async fn legacy_path_select(
        graph: &ChannelGraph,
        database: &Database,
        destination: &Address,
        max_hops: u32,
    ) -> JsResult<Path> {
        let selector = LegacyPathSelector::default();
        let cp = {
            let cgraph = graph.as_ref_counted();
            let cg = cgraph.read().await;
            selector.select_path(&*cg, cg.my_address(), *destination, max_hops as usize)?
        };

        let database = database.as_ref_counted();
        let g = database.read().await;

        Ok(cp.to_path(&PathResolver(&*g)).await?)
    }
}
