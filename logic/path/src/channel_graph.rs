use hopr_internal_types::prelude::*;
use hopr_primitive_types::primitives::Address;
use petgraph::algo::has_path_connecting;
use petgraph::dot::Dot;
use petgraph::prelude::StableDiGraph;
use petgraph::stable_graph::NodeIndex;
use petgraph::visit::{EdgeFiltered, EdgeRef, NodeFiltered};
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use tracing::{debug, warn};

#[cfg(all(feature = "prometheus", not(test)))]
use {
    hopr_internal_types::channels::ChannelDirection, hopr_metrics::metrics::MultiGauge,
    hopr_primitive_types::traits::ToHex,
};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_NUMBER_OF_CHANNELS: MultiGauge = MultiGauge::new(
        "hopr_channels_count",
        "Number of channels per direction",
        &["direction"]
    ).unwrap();
    static ref METRIC_CHANNEL_BALANCES: MultiGauge = MultiGauge::new(
        "hopr_channel_balances",
        "Balances on channels per counterparty",
        &["counterparty", "direction"]
    ).unwrap();
}

/// Structure that adds additional data to a `ChannelEntry`, which
/// can be used to compute edge weights and traverse the `ChannelGraph`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChannelEdge {
    /// Underlying channel
    pub channel: ChannelEntry,
    /// Optional scoring of the edge, that might be used for path planning.
    pub score: Option<f64>,
}

impl std::fmt::Display for ChannelEdge {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}; stake={}; score={:?}",
            self.channel, self.channel.balance, self.score
        )
    }
}

/// Represents a node in the Channel Graph.
/// This is typically represented by an on-chain address and ping quality, which
/// represents some kind of node's liveness as perceived by us.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Node {
    /// Node's on-chain address.
    pub address: Address,
    /// Liveness of the node.
    pub quality: f64,
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}; q={}", self.address, self.quality)
    }
}

/// Implements a HOPR payment channel graph (directed) cached in-memory.
///
/// This structure is useful for tracking channel state changes and
/// packet path finding.
///
/// The edges are updated only from the Indexer, and therefore the graph contains only
/// the channels *seen* on-chain.
/// The nodes and their qualities are updated as they are observed on the network.
///
/// Using this structure is much faster than querying the DB and therefore
/// is preferred for per-packet path-finding computations.
/// Per default, the graph does not track channels in `Closed` state, and therefore
/// cannot detect channel re-openings.
///
/// When a node reaches zero [quality](Node) and there are no edges (channels) containing this node,
/// it is removed from the graph entirely.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelGraph {
    me: Address,
    indices: HashMap<Address, NodeIndex>,
    graph: StableDiGraph<Node, ChannelEdge>,
}

impl ChannelGraph {
    /// The maximum number of intermediate hops the automatic path finding algorithm will look for.
    pub const INTERMEDIATE_HOPS: usize = 3;

    /// Creates a new instance with the given self `Address`.
    pub fn new(me: Address) -> Self {
        #[cfg(all(feature = "prometheus", not(test)))]
        {
            lazy_static::initialize(&METRIC_NUMBER_OF_CHANNELS);
        }

        let mut ret = Self {
            me,
            indices: HashMap::new(),
            graph: StableDiGraph::default(),
        };
        ret.indices.insert(
            me,
            ret.graph.add_node(Node {
                address: me,
                quality: 1.0,
            }),
        );
        ret
    }

    /// Checks if the channel is incoming to or outgoing from this node
    pub fn is_own_channel(&self, channel: &ChannelEntry) -> bool {
        channel.destination == self.me || channel.source == self.me
    }

    /// Convenience method to get this node's own address
    pub fn my_address(&self) -> Address {
        self.me
    }

    fn get_edge(&self, src: &Address, dst: &Address) -> Option<petgraph::stable_graph::EdgeReference<ChannelEdge>> {
        let (src_idx, dst_idx) = self
            .indices
            .get(src)
            .and_then(|src| self.indices.get(dst).map(|dst| (*src, *dst)))?;
        self.graph.edges_connecting(src_idx, dst_idx).next()
    }

    /// Looks up an `Open` or `PendingToClose` channel given the source and destination.
    /// Returns `None` if no such edge exists in the graph.
    pub fn get_channel(&self, source: &Address, destination: &Address) -> Option<&ChannelEntry> {
        self.get_edge(source, destination).map(|e| &e.weight().channel)
    }

    /// Gets the node information.
    /// Returns `None` if no such node exists in the graph.
    pub fn get_node(&self, node: &Address) -> Option<&Node> {
        self.indices.get(node).and_then(|index| self.graph.node_weight(*index))
    }

    /// Gets all `Open` outgoing channels going from the given [source](Address).
    pub fn open_channels_from(&self, source: Address) -> impl Iterator<Item = (&Node, &ChannelEdge)> {
        // If the source does not exist, select an impossible index to result in empty iterator.
        let idx = self
            .indices
            .get(&source)
            .cloned()
            .unwrap_or((self.graph.node_count() as u32).into());
        self.graph
            .edges_directed(idx, Direction::Outgoing)
            .filter(|c| c.weight().channel.status == ChannelStatus::Open)
            .map(|e| (&self.graph[e.target()], e.weight()))
    }

    /// Checks whether there's any path via Open channels that connects `source` and `destination`
    /// This does not need to be necessarily a multi-hop path.
    pub fn has_path(&self, source: &Address, destination: &Address) -> bool {
        let only_open_graph = EdgeFiltered::from_fn(&self.graph, |e| e.weight().channel.status == ChannelStatus::Open);
        if let Some((src_idx, dst_idx)) = self
            .indices
            .get(source)
            .and_then(|src| self.indices.get(destination).map(|dst| (*src, *dst)))
        {
            has_path_connecting(&only_open_graph, src_idx, dst_idx, None)
        } else {
            false
        }
    }

    /// Inserts or updates the given channel in the channel graph.
    /// Returns a set of changes if the channel was already present in the graphs or
    /// None if the channel was not previously present in the channel graph.
    pub fn update_channel(&mut self, channel: ChannelEntry) -> Option<Vec<ChannelChange>> {
        #[cfg(all(feature = "prometheus", not(test)))]
        {
            if let Some(direction) = channel.direction(&self.me) {
                match direction {
                    ChannelDirection::Outgoing => match channel.status {
                        ChannelStatus::Closed => {
                            METRIC_NUMBER_OF_CHANNELS.decrement(&["out"], 1.0);
                            METRIC_CHANNEL_BALANCES.set(&[channel.destination.to_hex().as_str(), "out"], 0.0);
                        }
                        ChannelStatus::Open => {
                            METRIC_NUMBER_OF_CHANNELS.increment(&["out"], 1.0);
                            METRIC_CHANNEL_BALANCES.set(
                                &[channel.destination.to_hex().as_str(), "out"],
                                channel
                                    .balance
                                    .amount_base_units()
                                    .parse::<f64>()
                                    .unwrap_or(f64::INFINITY),
                            );
                        }
                        ChannelStatus::PendingToClose(_) => {}
                    },
                    ChannelDirection::Incoming => match channel.status {
                        ChannelStatus::Closed => {
                            METRIC_NUMBER_OF_CHANNELS.decrement(&["in"], 1.0);
                            METRIC_CHANNEL_BALANCES.set(&[channel.source.to_hex().as_str(), "in"], 0.0);
                        }
                        ChannelStatus::Open => {
                            METRIC_NUMBER_OF_CHANNELS.increment(&["in"], 1.0);
                            METRIC_CHANNEL_BALANCES.set(
                                &[channel.source.to_hex().as_str(), "in"],
                                channel
                                    .balance
                                    .amount_base_units()
                                    .parse::<f64>()
                                    .unwrap_or(f64::INFINITY),
                            );
                        }
                        ChannelStatus::PendingToClose(_) => {}
                    },
                }
            }
        }

        let maybe_edge_id = self.get_edge(&channel.source, &channel.destination).map(|e| e.id());

        // Remove the edge since we don't allow Closed channels
        if channel.status == ChannelStatus::Closed {
            return maybe_edge_id
                .and_then(|id| self.graph.remove_edge(id))
                .inspect(|c| debug!("removed {}", c.channel))
                .map(|old_value| ChannelChange::diff_channels(&old_value.channel, &channel));
        }

        // If an edge already exists, update it and compute ChannelDiff
        if let Some(old_value) = maybe_edge_id.and_then(|id| self.graph.edge_weight_mut(id)) {
            let old_channel = old_value.channel;
            old_value.channel = channel;

            let ret = ChannelChange::diff_channels(&old_channel, &channel);
            debug!(
                "updated {channel}: {}",
                ret.iter().map(ChannelChange::to_string).collect::<Vec<_>>().join(",")
            );
            Some(ret)
        } else {
            // Otherwise, create a new edge and add the nodes with 0 quality if they don't yet exist
            let src = *self.indices.entry(channel.source).or_insert_with(|| {
                self.graph.add_node(Node {
                    address: channel.source,
                    quality: 0.0,
                })
            });

            let dst = *self.indices.entry(channel.destination).or_insert_with(|| {
                self.graph.add_node(Node {
                    address: channel.destination,
                    quality: 0.0,
                })
            });

            let weighted = ChannelEdge { channel, score: None };

            self.graph.add_edge(src, dst, weighted);
            debug!("new {channel}");

            None
        }
    }

    /// Updates the quality of a node (inserting it into the graph if it does not exist yet).
    /// The given quality value must always be non-negative.
    pub fn update_node_quality(&mut self, address: &Address, quality: f64) {
        assert!(quality >= 0_f64, "quality must be non-negative");
        if !self.me.eq(address) {
            match self.indices.entry(*address) {
                // The node exists and we're updating to greater-than-zero quality
                Entry::Occupied(existing) => {
                    if quality > 0.0 || self.graph.neighbors_undirected(*existing.get()).count() > 0 {
                        if let Some(node) = self.graph.node_weight_mut(*existing.get()) {
                            node.quality = quality;
                            debug!("updated quality of {address} to {quality}");
                        } else {
                            warn!("removed dangling node {address} in channel graph");
                            existing.remove();
                        }
                    } else {
                        // If the node has no neighbors and 0 quality, remove it from the graph
                        self.graph.remove_node(existing.remove());
                        debug!("removed solitary node {address} with zero quality");
                    }
                }
                // The node does not exist, and we're updating to greater-than-zero quality
                Entry::Vacant(new_node) if quality > 0_f64 => {
                    new_node.insert(self.graph.add_node(Node {
                        address: *address,
                        quality,
                    }));
                    debug!("added node {address} with {quality}");
                }
                // Do nothing otherwise.
                Entry::Vacant(_) => {}
            }
        }
    }

    /// Updates the score value of network connection between `source` and `destination`
    /// The given quality value must always be non-negative.
    pub fn update_channel_score(&mut self, source: &Address, destination: &Address, score: f64) {
        assert!(score >= 0_f64, "quality must be non-negative");
        let maybe_edge_id = self.get_edge(source, destination).map(|e| e.id());
        if let Some(channel) = maybe_edge_id.and_then(|id| self.graph.edge_weight_mut(id)) {
            if score != channel.score.unwrap_or(-1_f64) {
                channel.score = Some(score);
                debug!("updated score of {} to {score}", channel.channel);
            }
        }
    }

    /// Gets quality of the given channel. Returns `None` if no such channel exists or no
    /// quality has been set for that channel.
    pub fn get_channel_score(&self, source: &Address, destination: &Address) -> Option<f64> {
        self.get_edge(source, destination)
            .and_then(|e| self.graph.edge_weight(e.id()))
            .and_then(|e| e.score)
    }

    /// Checks whether the given channel is in the graph already.
    pub fn contains_channel(&self, channel: &ChannelEntry) -> bool {
        self.get_channel(&channel.source, &channel.destination).is_some()
    }

    /// Checks whether the given node is in the channel graph.
    pub fn contains_node(&self, address: &Address) -> bool {
        self.get_node(address).is_some()
    }

    /// Outputs the channel graph in the DOT (graphviz) format with the given `config`.
    pub fn as_dot(&self, cfg: GraphExportConfig) -> String {
        if cfg.ignore_disconnected_components {
            let only_open_graph =
                EdgeFiltered::from_fn(&self.graph, |e| e.weight().channel.status == ChannelStatus::Open);

            let me_idx = self.indices.get(&self.me).expect("graph must contain self");

            Dot::new(&NodeFiltered::from_fn(&self.graph, |n| {
                // Include only nodes that have non-zero quality,
                // and there is a path to them in an Open channel graph
                self.graph.node_weight(n).is_some_and(|n| n.quality > 0_f64)
                    && has_path_connecting(&only_open_graph, *me_idx, n, None)
            }))
            .to_string()
        } else if cfg.ignore_non_opened_channels {
            // Keep nodes that have at least one incoming or outgoing Open channel
            Dot::new(&NodeFiltered::from_fn(&self.graph, |a| {
                self.graph
                    .edges_directed(a, Direction::Outgoing)
                    .any(|e| e.weight().channel.status == ChannelStatus::Open)
                    || self
                        .graph
                        .edges_directed(a, Direction::Incoming)
                        .any(|e| e.weight().channel.status == ChannelStatus::Open)
            }))
            .to_string()
        } else {
            Dot::new(&self.graph).to_string()
        }
    }
}

/// Configuration for the DOT export of the [`ChannelGraph`].
///
/// See [`ChannelGraph::as_dot`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GraphExportConfig {
    /// If set, nodes that are not connected to this node (via open channels) will not be exported.
    ///
    /// This setting automatically implies `ignore_non_opened_channels`.
    pub ignore_disconnected_components: bool,
    /// Do not export channels that are not in the [`ChannelStatus::Open`] state.
    pub ignore_non_opened_channels: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::channel_graph::ChannelGraph;
    use anyhow::Context;
    use hopr_internal_types::channels::{ChannelChange, ChannelEntry, ChannelStatus};
    use hopr_primitive_types::prelude::*;
    use lazy_static::lazy_static;
    use std::ops::Add;
    use std::str::FromStr;
    use std::time::{Duration, SystemTime};

    lazy_static! {
        static ref ADDRESSES: [Address; 6] = [
            Address::from_str("0xafe8c178cf70d966be0a798e666ce2782c7b2288")
                .expect("lazy static address should be valid"),
            Address::from_str("0x1223d5786d9e6799b3297da1ad55605b91e2c882")
                .expect("lazy static address should be valid"),
            Address::from_str("0x0e3e60ddced1e33c9647a71f4fc2cf4ed33e4a9d")
                .expect("lazy static address should be valid"),
            Address::from_str("0x27644105095c8c10f804109b4d1199a9ac40ed46")
                .expect("lazy static address should be valid"),
            Address::from_str("0x4701a288c38fa8a0f4b79127747257af4a03a623")
                .expect("lazy static address should be valid"),
            Address::from_str("0xfddd2f462ec709cf181bbe44a7e952487bd4591d")
                .expect("lazy static address should be valid"),
        ];
    }

    fn dummy_channel(src: Address, dst: Address, status: ChannelStatus) -> ChannelEntry {
        ChannelEntry::new(
            src,
            dst,
            Balance::new_from_str("1", BalanceType::HOPR),
            1u32.into(),
            status,
            1u32.into(),
        )
    }

    #[test]
    fn channel_graph_self_addr() {
        let cg = ChannelGraph::new(ADDRESSES[0]);
        assert_eq!(ADDRESSES[0], cg.my_address(), "must produce correct self address");

        assert!(cg.contains_node(&ADDRESSES[0]), "must contain self address");

        assert_eq!(
            cg.get_node(&ADDRESSES[0]).cloned(),
            Some(Node {
                address: ADDRESSES[0],
                quality: 1.0
            }),
            "must contain self node with quality 1"
        );
    }

    #[test]
    fn channel_graph_has_path() {
        let mut cg = ChannelGraph::new(ADDRESSES[0]);

        let c = dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::Open);
        cg.update_channel(c);

        assert!(cg.contains_channel(&c), "must contain channel");

        assert!(cg.contains_node(&ADDRESSES[0]), "must contain channel source");

        assert!(cg.contains_node(&ADDRESSES[1]), "must contain channel destination");

        assert!(cg.has_path(&ADDRESSES[0], &ADDRESSES[1]), "must have simple path");

        assert!(
            !cg.has_path(&ADDRESSES[0], &ADDRESSES[2]),
            "must not have non existent path"
        );
    }

    #[test]
    fn channel_graph_update_node_quality() {
        let mut cg = ChannelGraph::new(ADDRESSES[0]);

        cg.update_node_quality(&ADDRESSES[1], 0.5);
        assert_eq!(
            cg.get_node(&ADDRESSES[1]).cloned(),
            Some(Node {
                address: ADDRESSES[1],
                quality: 0.5
            })
        );

        cg.update_node_quality(&ADDRESSES[1], 0.3);
        assert_eq!(
            cg.get_node(&ADDRESSES[1]).cloned(),
            Some(Node {
                address: ADDRESSES[1],
                quality: 0.3
            })
        );

        assert!(!cg.has_path(&ADDRESSES[0], &ADDRESSES[1]));

        cg.update_node_quality(&ADDRESSES[1], 0.0);
        assert_eq!(cg.get_node(&ADDRESSES[1]), None);
    }

    #[test]
    fn channel_graph_update_node_quality_should_not_remove_nodes_with_zero_quality_and_path() {
        let mut cg = ChannelGraph::new(ADDRESSES[0]);

        let c = dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::Open);
        cg.update_channel(c);

        assert_eq!(
            cg.get_node(&ADDRESSES[1]).cloned(),
            Some(Node {
                address: ADDRESSES[1],
                quality: 0.0
            })
        );

        cg.update_node_quality(&ADDRESSES[1], 0.5);
        assert_eq!(
            cg.get_node(&ADDRESSES[1]).cloned(),
            Some(Node {
                address: ADDRESSES[1],
                quality: 0.5
            })
        );

        cg.update_node_quality(&ADDRESSES[1], 0.3);
        assert_eq!(
            cg.get_node(&ADDRESSES[1]).cloned(),
            Some(Node {
                address: ADDRESSES[1],
                quality: 0.3
            })
        );

        assert!(cg.has_path(&ADDRESSES[0], &ADDRESSES[1]));

        cg.update_node_quality(&ADDRESSES[1], 0.0);
        assert_eq!(
            cg.get_node(&ADDRESSES[1]).cloned(),
            Some(Node {
                address: ADDRESSES[1],
                quality: 0.0
            })
        );
    }

    #[test]
    fn channel_graph_update_channel_score() -> anyhow::Result<()> {
        let mut cg = ChannelGraph::new(ADDRESSES[0]);

        let c = dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::Open);
        cg.update_channel(c);

        assert!(cg.contains_channel(&c), "must contain channel");
        assert!(
            cg.get_channel_score(&ADDRESSES[0], &ADDRESSES[1]).is_none(),
            "must start with no quality info"
        );

        cg.update_channel_score(&ADDRESSES[0], &ADDRESSES[1], 0.5_f64);

        let q = cg
            .get_channel_score(&ADDRESSES[0], &ADDRESSES[1])
            .context("must have quality when set")?;
        assert_eq!(0.5_f64, q, "quality must be equal");

        Ok(())
    }

    #[test]
    fn channel_graph_is_own_channel() {
        let mut cg = ChannelGraph::new(ADDRESSES[0]);

        let c1 = dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::Open);
        let c2 = dummy_channel(ADDRESSES[1], ADDRESSES[2], ChannelStatus::Open);
        cg.update_channel(c1);
        cg.update_channel(c2);

        assert!(cg.is_own_channel(&c1), "must detect as own channel");
        assert!(!cg.is_own_channel(&c2), "must not detect as own channel");
    }

    #[test]
    fn channel_graph_update_changes() -> anyhow::Result<()> {
        let mut cg = ChannelGraph::new(ADDRESSES[0]);

        let mut c = dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::Open);

        let changes = cg.update_channel(c);
        assert!(changes.is_none(), "must not produce changes for a new channel");

        let cr = cg
            .get_channel(&ADDRESSES[0], &ADDRESSES[1])
            .context("must contain channel")?;
        assert!(c.eq(cr), "channels must be equal");

        let ts = SystemTime::now().add(Duration::from_secs(10));
        c.balance = Balance::zero(BalanceType::HOPR);
        c.status = ChannelStatus::PendingToClose(ts);
        let changes = cg.update_channel(c).context("should contain channel changes")?;
        assert_eq!(2, changes.len(), "must contain 2 changes");

        for change in changes {
            match change {
                ChannelChange::Status { left, right } => {
                    assert_eq!(ChannelStatus::Open, left, "previous status does not match");
                    assert_eq!(ChannelStatus::PendingToClose(ts), right, "new status does not match");
                }
                ChannelChange::CurrentBalance { left, right } => {
                    assert_eq!(
                        Balance::new(1_u32, BalanceType::HOPR),
                        left,
                        "previous balance does not match"
                    );
                    assert_eq!(Balance::zero(BalanceType::HOPR), right, "new balance does not match");
                }
                _ => panic!("unexpected change"),
            }
        }

        let cr = cg
            .get_channel(&ADDRESSES[0], &ADDRESSES[1])
            .context("must contain channel")?;
        assert!(c.eq(cr), "channels must be equal");

        Ok(())
    }

    #[test]
    fn channel_graph_update_changes_on_close() -> anyhow::Result<()> {
        let mut cg = ChannelGraph::new(ADDRESSES[0]);

        let ts = SystemTime::now().add(Duration::from_secs(10));
        let mut c = dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::PendingToClose(ts));

        let changes = cg.update_channel(c);
        assert!(changes.is_none(), "must not produce changes for a new channel");

        let cr = cg
            .get_channel(&ADDRESSES[0], &ADDRESSES[1])
            .context("must contain channel")?;
        assert!(c.eq(cr), "channels must be equal");

        c.balance = Balance::zero(BalanceType::HOPR);
        c.status = ChannelStatus::Closed;
        let changes = cg.update_channel(c).context("must contain changes")?;
        assert_eq!(2, changes.len(), "must contain 2 changes");

        for change in changes {
            match change {
                ChannelChange::Status { left, right } => {
                    assert_eq!(
                        ChannelStatus::PendingToClose(ts),
                        left,
                        "previous status does not match"
                    );
                    assert_eq!(ChannelStatus::Closed, right, "new status does not match");
                }
                ChannelChange::CurrentBalance { left, right } => {
                    assert_eq!(
                        Balance::new(1_u32, BalanceType::HOPR),
                        left,
                        "previous balance does not match"
                    );
                    assert_eq!(Balance::zero(BalanceType::HOPR), right, "new balance does not match");
                }
                _ => panic!("unexpected change"),
            }
        }

        let cr = cg.get_channel(&ADDRESSES[0], &ADDRESSES[1]);
        assert!(cr.is_none(), "must not contain channel after closing");

        Ok(())
    }

    #[test]
    fn channel_graph_update_should_not_allow_closed_channels() {
        let mut cg = ChannelGraph::new(ADDRESSES[0]);
        let changes = cg.update_channel(dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::Closed));
        assert!(changes.is_none(), "must not produce changes for a closed channel");

        let c = cg.get_channel(&ADDRESSES[0], &ADDRESSES[1]);
        assert!(c.is_none(), "must not allow adding closed channels");
    }

    #[test]
    fn channel_graph_update_should_allow_pending_to_close_channels() -> anyhow::Result<()> {
        let mut cg = ChannelGraph::new(ADDRESSES[0]);
        let ts = SystemTime::now().add(Duration::from_secs(10));
        let changes = cg.update_channel(dummy_channel(
            ADDRESSES[0],
            ADDRESSES[1],
            ChannelStatus::PendingToClose(ts),
        ));
        assert!(changes.is_none(), "must not produce changes for a closed channel");

        cg.get_channel(&ADDRESSES[0], &ADDRESSES[1])
            .context("must allow PendingToClose channels")?;

        Ok(())
    }
}
