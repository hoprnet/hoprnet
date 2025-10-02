use std::{
    collections::{HashMap, hash_map::Entry},
    fmt::{Debug, Formatter},
    time::Duration,
};

use hopr_internal_types::prelude::*;
use hopr_primitive_types::{prelude::SMA, primitives::Address, sma::SingleSumSMA};
use petgraph::{
    Direction,
    algo::has_path_connecting,
    dot::Dot,
    prelude::StableDiGraph,
    stable_graph::NodeIndex,
    visit::{EdgeFiltered, EdgeRef, NodeFiltered},
};
use tracing::{debug, warn};
#[cfg(all(feature = "prometheus", not(test)))]
use {hopr_internal_types::channels::ChannelDirection, hopr_metrics::metrics::MultiGauge};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_NUMBER_OF_CHANNELS: MultiGauge = MultiGauge::new(
        "hopr_channels_count",
        "Number of channels per direction",
        &["direction"]
    ).unwrap();
}

/// Structure that adds additional data to a `ChannelEntry`, which
/// can be used to compute edge weights and traverse the `ChannelGraph`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChannelEdge {
    /// Underlying channel
    pub channel: ChannelEntry,
    /// Optional scoring of the edge that might be used for path planning.
    pub edge_score: Option<f64>,
}

impl std::fmt::Display for ChannelEdge {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}; stake={}; score={:?}; status={};",
            self.channel,
            self.channel.balance,
            self.edge_score,
            self.channel.status.to_string().to_lowercase()
        )
    }
}

/// Represents a node in the Channel Graph.
/// This is typically represented by an on-chain address and ping quality, which
/// represents some kind of node's liveness as perceived by us.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Node {
    /// Node's on-chain address.
    pub address: Address,
    /// Liveness of the node.
    pub node_score: f64,
    /// Average node latency
    pub latency: SingleSumSMA<std::time::Duration, u32>,
}

impl Node {
    pub fn new(address: Address, latency_window_length: usize) -> Self {
        Self {
            address,
            node_score: 0.0,
            latency: SingleSumSMA::new(latency_window_length),
        }
    }

    /// Update the score using the [`NodeScoreUpdate`] and [`ChannelGraphConfig`].
    ///
    /// The function will ensure additive (slow) ramp-up, but exponential (fast)
    /// ramp-down of the node's score, depending on whether it was reachable or not.
    /// The ramp-down has a cut-off at `offline_node_score_threshold`, below which
    /// is the score set to zero.
    pub fn update_score(&mut self, score_update: NodeScoreUpdate, cfg: ChannelGraphConfig) -> f64 {
        match score_update {
            NodeScoreUpdate::Reachable(latency) => {
                self.node_score = 1.0_f64.min(self.node_score + cfg.node_score_step_up);
                self.latency.push(latency);
            }
            NodeScoreUpdate::Unreachable => {
                self.node_score /= cfg.node_score_decay;
                self.latency.clear();
                if self.node_score < cfg.offline_node_score_threshold {
                    self.node_score = 0.0;
                }
            }
            NodeScoreUpdate::Initialize(latency, node_score) => {
                self.latency.clear();
                self.latency.push(latency);
                self.node_score = node_score.clamp(0.0, 1.0);
            }
        }
        self.node_score
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}; score={}", self.address, self.node_score)
    }
}

/// Configuration for the [`ChannelGraph`].
#[derive(Clone, Copy, Debug, PartialEq, smart_default::SmartDefault)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChannelGraphConfig {
    /// Length of the Simple Moving Average window for node latencies.
    #[default(20)]
    pub latency_sma_window_length: usize,
    /// Additive node score modifier when the node is reachable.
    #[default(0.1)]
    pub node_score_step_up: f64,
    /// Node score divisor when the node is unreachable.
    #[default(4.0)]
    pub node_score_decay: f64,
    /// If a node is unreachable and because of that it reaches a score
    /// lower than this threshold, it is considered offline (and in some situations
    /// can be removed from the graph).
    #[default(0.1)]
    pub offline_node_score_threshold: f64,
}

/// Describes an update of the [`Node`]'s score.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NodeScoreUpdate {
    /// Node is reachable with the given latency.
    Reachable(Duration),
    /// Node is unreachable.
    Unreachable,
    /// Initializes a node's score to the given latency and quality.
    /// This is useful during loading data from the persistent storage.
    Initialize(Duration, f64),
}

impl<T> From<Result<Duration, T>> for NodeScoreUpdate {
    fn from(result: Result<Duration, T>) -> Self {
        match result {
            Ok(duration) => NodeScoreUpdate::Reachable(duration),
            Err(_) => NodeScoreUpdate::Unreachable,
        }
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
/// Per default, the graph does not track channels in the `Closed` state and therefore
/// cannot detect channel re-openings.
///
/// When a node reaches zero [quality](Node) and there are no edges (channels) containing this node,
/// it is removed from the graph entirely.
#[cfg_attr(feature = "serde", cfg_eval::cfg_eval, serde_with::serde_as)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct ChannelGraph {
    me: Address,
    #[cfg_attr(feature = "serde", serde_as(as = "Vec<(_, _)>"))]
    indices: HashMap<Address, u32>,
    graph: StableDiGraph<Node, ChannelEdge>,
    cfg: ChannelGraphConfig,
}

impl ChannelGraph {
    /// The maximum number of intermediate hops the automatic path-finding algorithm will look for.
    pub const INTERMEDIATE_HOPS: usize = 3;

    /// Creates a new instance with the given self `Address`.
    pub fn new(me: Address, cfg: ChannelGraphConfig) -> Self {
        #[cfg(all(feature = "prometheus", not(test)))]
        {
            lazy_static::initialize(&METRIC_NUMBER_OF_CHANNELS);
        }

        let mut ret = Self {
            me,
            cfg,
            indices: HashMap::new(),
            graph: StableDiGraph::default(),
        };
        ret.indices.insert(
            me,
            ret.graph
                .add_node(Node {
                    address: me,
                    node_score: 1.0,
                    latency: SingleSumSMA::new_with_samples(
                        cfg.latency_sma_window_length,
                        vec![Duration::ZERO; cfg.latency_sma_window_length],
                    ),
                })
                .index() as u32,
        );
        ret
    }

    /// Number of channels (edges) in the graph.
    pub fn count_channels(&self) -> usize {
        self.graph.edge_count()
    }

    /// Number of nodes in the graph.
    pub fn count_nodes(&self) -> usize {
        self.graph.node_count()
    }

    /// Checks if the channel is incoming to or outgoing from this node
    pub fn is_own_channel(&self, channel: &ChannelEntry) -> bool {
        channel.destination == self.me || channel.source == self.me
    }

    /// Convenience method to get this node's own address
    pub fn my_address(&self) -> Address {
        self.me
    }

    fn get_edge(&self, src: &Address, dst: &Address) -> Option<petgraph::stable_graph::EdgeReference<'_, ChannelEdge>> {
        let (src_idx, dst_idx) = self
            .indices
            .get(src)
            .and_then(|src| self.indices.get(dst).map(|dst| (*src, *dst)))?;
        self.graph.edges_connecting(src_idx.into(), dst_idx.into()).next()
    }

    /// Looks up an `Open` or `PendingToClose` channel given the source and destination.
    /// Returns `None` if no such edge exists in the graph.
    pub fn get_channel(&self, source: &Address, destination: &Address) -> Option<&ChannelEntry> {
        self.get_edge(source, destination).map(|e| &e.weight().channel)
    }

    /// Gets the node information.
    /// Returns `None` if no such node exists in the graph.
    pub fn get_node(&self, node: &Address) -> Option<&Node> {
        self.indices
            .get(node)
            .and_then(|index| self.graph.node_weight((*index).into()))
    }

    /// Gets all `Open` outgoing channels going from the given [source](Address).
    pub fn open_channels_from(&self, source: Address) -> impl Iterator<Item = (&Node, &ChannelEdge)> {
        // If the source does not exist, select an impossible index to result in empty iterator.
        let idx = self
            .indices
            .get(&source)
            .cloned()
            .unwrap_or(self.graph.node_count() as u32);
        self.graph
            .edges_directed(idx.into(), Direction::Outgoing)
            .filter(|c| c.weight().channel.status == ChannelStatus::Open)
            .map(|e| (&self.graph[e.target()], e.weight()))
    }

    /// Checks whether there is any path via Open channels that connects `source` and `destination`
    /// This does not need to be necessarily a multi-hop path.
    pub fn has_path(&self, source: &Address, destination: &Address) -> bool {
        let only_open_graph = EdgeFiltered::from_fn(&self.graph, |e| e.weight().channel.status == ChannelStatus::Open);
        if let Some((src_idx, dst_idx)) = self
            .indices
            .get(source)
            .and_then(|src| self.indices.get(destination).map(|dst| (*src, *dst)))
        {
            has_path_connecting(&only_open_graph, src_idx.into(), dst_idx.into(), None)
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
                        }
                        ChannelStatus::Open => {
                            METRIC_NUMBER_OF_CHANNELS.increment(&["out"], 1.0);
                        }
                        ChannelStatus::PendingToClose(_) => {}
                    },
                    ChannelDirection::Incoming => match channel.status {
                        ChannelStatus::Closed => {
                            METRIC_NUMBER_OF_CHANNELS.decrement(&["in"], 1.0);
                        }
                        ChannelStatus::Open => {
                            METRIC_NUMBER_OF_CHANNELS.increment(&["in"], 1.0);
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
                self.graph
                    .add_node(Node::new(channel.source, self.cfg.latency_sma_window_length))
                    .index() as u32
            });

            let dst = *self.indices.entry(channel.destination).or_insert_with(|| {
                self.graph
                    .add_node(Node::new(channel.destination, self.cfg.latency_sma_window_length))
                    .index() as u32
            });

            let weighted = ChannelEdge {
                channel,
                edge_score: None,
            };

            self.graph.add_edge(src.into(), dst.into(), weighted);
            debug!("new {channel}");

            None
        }
    }

    /// Updates the quality of a node (inserting it into the graph if it does not exist yet),
    /// based on the given [`NodeScoreUpdate`].
    pub fn update_node_score(&mut self, address: &Address, score_update: NodeScoreUpdate) {
        if !self.me.eq(address) {
            match self.indices.entry(*address) {
                // The node exists
                Entry::Occupied(existing) => {
                    let existing_idx: NodeIndex = (*existing.get()).into();
                    // NOTE: we cannot remove offline nodes that still have edges,
                    // as we would lose the ability to track changes on those edges if they
                    // were removed early.
                    if score_update != NodeScoreUpdate::Unreachable
                        || self.graph.neighbors_undirected(existing_idx).count() > 0
                    {
                        // We are for sure updating to a greater-than-zero score
                        if let Some(node) = self.graph.node_weight_mut(existing_idx) {
                            let updated_quality = node.update_score(score_update, self.cfg);
                            debug!(%address, updated_quality, "updated node quality");
                        } else {
                            // This should not happen
                            warn!(%address, "removed dangling node index from channel graph");
                            existing.remove();
                        }
                    } else {
                        // If the node has no neighbors, is unreachable, and reached very low
                        // score, just remove it from the graph
                        if self
                            .graph
                            .node_weight_mut(existing_idx)
                            .map(|node| node.update_score(score_update, self.cfg))
                            .is_some_and(|updated_quality| updated_quality < self.cfg.offline_node_score_threshold)
                        {
                            self.graph.remove_node(existing.remove().into());
                            debug!(%address, "removed offline node with no channels");
                        }
                    }
                }
                // The node does not exist, and we are updating to greater-than-zero quality
                Entry::Vacant(new_node) if score_update != NodeScoreUpdate::Unreachable => {
                    let mut inserted_node = Node::new(*address, self.cfg.latency_sma_window_length);
                    let updated_quality = inserted_node.update_score(score_update, self.cfg);
                    new_node.insert(self.graph.add_node(inserted_node).index() as u32);
                    debug!(%address, updated_quality, "added new node");
                }
                // We do not want to add unreachable nodes to the graph, so do nothing otherwise
                Entry::Vacant(_) => {}
            }
        }
    }

    /// Updates the score value of network connection between `source` and `destination`
    /// The given score value must always be non-negative.
    pub fn update_channel_score(&mut self, source: &Address, destination: &Address, score: f64) {
        assert!(score >= 0_f64, "score must be non-negative");
        let maybe_edge_id = self.get_edge(source, destination).map(|e| e.id());
        if let Some(channel) = maybe_edge_id.and_then(|id| self.graph.edge_weight_mut(id)) {
            if score != channel.edge_score.unwrap_or(-1_f64) {
                channel.edge_score = Some(score);
                debug!("updated score of {} to {score}", channel.channel);
            }
        }
    }

    /// Gets quality of the given channel. Returns `None` if no such channel exists, or no
    /// quality has been set for that channel.
    pub fn get_channel_score(&self, source: &Address, destination: &Address) -> Option<f64> {
        self.get_edge(source, destination)
            .and_then(|e| self.graph.edge_weight(e.id()))
            .and_then(|e| e.edge_score)
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

            let me_idx: NodeIndex = (*self.indices.get(&self.me).expect("graph must contain self")).into();

            Dot::new(&NodeFiltered::from_fn(&self.graph, |n| {
                // Include only nodes that have non-zero quality,
                // and there is a path to them in an Open channel graph
                self.graph.node_weight(n).is_some_and(|n| n.node_score > 0_f64)
                    && has_path_connecting(&only_open_graph, me_idx, n, None)
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
        } else if cfg.only_3_hop_accessible_nodes {
            // Keep only those nodes that are accessible from via less than 3 hop paths
            let me_idx: NodeIndex = (*self.indices.get(&self.me).expect("graph must contain self")).into();
            let distances = petgraph::algo::dijkstra(&self.graph, me_idx, None, |e| {
                if e.weight().channel.status == ChannelStatus::Open {
                    1
                } else {
                    100
                }
            });

            Dot::new(&NodeFiltered::from_fn(&self.graph, |a| {
                distances.get(&a).map(|d| *d <= 3).unwrap_or(false)
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
    /// Show only nodes that are accessible via 3-hops (via open channels) from this node.
    pub only_3_hop_accessible_nodes: bool,
}

#[cfg(test)]
mod tests {
    use std::{
        ops::Add,
        time::{Duration, SystemTime},
    };

    use anyhow::{Context, anyhow};
    use hopr_internal_types::channels::{ChannelChange, ChannelStatus};
    use hopr_primitive_types::prelude::*;

    use super::*;
    use crate::{
        channel_graph::ChannelGraph,
        tests::{ADDRESSES, dummy_channel},
    };

    #[test]
    fn channel_graph_self_addr() {
        let cg = ChannelGraph::new(ADDRESSES[0], Default::default());
        assert_eq!(ADDRESSES[0], cg.my_address(), "must produce correct self address");

        assert!(cg.contains_node(&ADDRESSES[0]), "must contain self address");

        assert_eq!(
            cg.get_node(&ADDRESSES[0]).cloned(),
            Some(Node {
                address: ADDRESSES[0],
                node_score: 1.0,
                latency: SingleSumSMA::new_with_samples(
                    cg.cfg.latency_sma_window_length,
                    vec![Duration::ZERO; cg.cfg.latency_sma_window_length]
                )
            }),
            "must contain self node with quality 1"
        );
    }

    #[test]
    fn channel_graph_has_path() {
        let mut cg = ChannelGraph::new(ADDRESSES[0], Default::default());

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
    fn channel_graph_update_node_quality() -> anyhow::Result<()> {
        let mut cg = ChannelGraph::new(
            ADDRESSES[0],
            ChannelGraphConfig {
                node_score_step_up: 0.1,
                node_score_decay: 4.0,
                offline_node_score_threshold: 0.1,
                ..Default::default()
            },
        );

        cg.update_node_score(&ADDRESSES[1], NodeScoreUpdate::Reachable(Duration::from_millis(100)));
        let node = cg.get_node(&ADDRESSES[1]).cloned().ok_or(anyhow!("node must exist"))?;
        assert_eq!(node.node_score, 0.1);
        assert_eq!(node.latency.average(), Some(Duration::from_millis(100)));

        assert!(!cg.has_path(&ADDRESSES[0], &ADDRESSES[1]));

        cg.update_node_score(&ADDRESSES[1], NodeScoreUpdate::Reachable(Duration::from_millis(50)));
        let node = cg.get_node(&ADDRESSES[1]).cloned().ok_or(anyhow!("node must exist"))?;
        assert_eq!(node.node_score, 0.2);
        assert_eq!(node.latency.average(), Some(Duration::from_millis(75)));

        cg.update_node_score(&ADDRESSES[1], NodeScoreUpdate::Reachable(Duration::from_millis(30)));
        cg.update_node_score(&ADDRESSES[1], NodeScoreUpdate::Reachable(Duration::from_millis(20)));

        let node = cg.get_node(&ADDRESSES[1]).cloned().ok_or(anyhow!("node must exist"))?;
        assert_eq!(node.node_score, 0.4);
        assert_eq!(node.latency.average(), Some(Duration::from_millis(50)));

        cg.update_node_score(&ADDRESSES[1], NodeScoreUpdate::Unreachable);
        let node = cg.get_node(&ADDRESSES[1]).cloned().ok_or(anyhow!("node must exist"))?;
        assert_eq!(node.node_score, 0.1);
        assert!(node.latency.average().is_none());

        cg.update_node_score(&ADDRESSES[1], NodeScoreUpdate::Unreachable);

        // At this point the node will be removed because there are no channels with it
        assert_eq!(cg.get_node(&ADDRESSES[1]), None);

        Ok(())
    }

    #[test]
    fn channel_graph_update_node_quality_should_not_remove_nodes_with_zero_quality_and_path() -> anyhow::Result<()> {
        let mut cg = ChannelGraph::new(
            ADDRESSES[0],
            ChannelGraphConfig {
                node_score_step_up: 0.1,
                node_score_decay: 4.0,
                offline_node_score_threshold: 0.1,
                ..Default::default()
            },
        );

        let c = dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::Open);
        cg.update_channel(c);

        let node = cg.get_node(&ADDRESSES[1]).cloned().ok_or(anyhow!("node must exist"))?;
        assert_eq!(node.address, ADDRESSES[1]);
        assert_eq!(node.node_score, 0.0);
        assert!(node.latency.is_empty());

        cg.update_node_score(&ADDRESSES[1], NodeScoreUpdate::Reachable(Duration::from_millis(50)));

        let node = cg.get_node(&ADDRESSES[1]).cloned().ok_or(anyhow!("node must exist"))?;
        assert_eq!(node.address, ADDRESSES[1]);
        assert_eq!(node.node_score, 0.1);

        cg.update_node_score(&ADDRESSES[1], NodeScoreUpdate::Reachable(Duration::from_millis(50)));
        cg.update_node_score(&ADDRESSES[1], NodeScoreUpdate::Reachable(Duration::from_millis(50)));
        cg.update_node_score(&ADDRESSES[1], NodeScoreUpdate::Reachable(Duration::from_millis(50)));

        let node = cg.get_node(&ADDRESSES[1]).cloned().ok_or(anyhow!("node must exist"))?;
        assert_eq!(node.address, ADDRESSES[1]);
        assert_eq!(node.node_score, 0.4);

        assert!(cg.has_path(&ADDRESSES[0], &ADDRESSES[1]));

        cg.update_node_score(&ADDRESSES[1], NodeScoreUpdate::Unreachable);

        let node = cg.get_node(&ADDRESSES[1]).cloned().ok_or(anyhow!("node must exist"))?;
        assert_eq!(node.address, ADDRESSES[1]);
        assert_eq!(node.node_score, 0.1);

        assert!(cg.has_path(&ADDRESSES[0], &ADDRESSES[1]));

        cg.update_node_score(&ADDRESSES[1], NodeScoreUpdate::Unreachable);

        let node = cg.get_node(&ADDRESSES[1]).cloned().ok_or(anyhow!("node must exist"))?;
        assert_eq!(node.address, ADDRESSES[1]);
        assert_eq!(node.node_score, 0.0);

        assert!(cg.has_path(&ADDRESSES[0], &ADDRESSES[1]));

        Ok(())
    }

    #[test]
    fn channel_graph_update_channel_score() -> anyhow::Result<()> {
        let mut cg = ChannelGraph::new(ADDRESSES[0], Default::default());

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
        let mut cg = ChannelGraph::new(ADDRESSES[0], Default::default());

        let c1 = dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::Open);
        let c2 = dummy_channel(ADDRESSES[1], ADDRESSES[2], ChannelStatus::Open);
        cg.update_channel(c1);
        cg.update_channel(c2);

        assert!(cg.is_own_channel(&c1), "must detect as own channel");
        assert!(!cg.is_own_channel(&c2), "must not detect as own channel");
    }

    #[test]
    fn channel_graph_update_changes() -> anyhow::Result<()> {
        let mut cg = ChannelGraph::new(ADDRESSES[0], Default::default());

        let mut c = dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::Open);

        let changes = cg.update_channel(c);
        assert!(changes.is_none(), "must not produce changes for a new channel");

        let cr = cg
            .get_channel(&ADDRESSES[0], &ADDRESSES[1])
            .context("must contain channel")?;
        assert!(c.eq(cr), "channels must be equal");

        let ts = SystemTime::now().add(Duration::from_secs(10));
        c.balance = 0.into();
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
                    assert_eq!(HoprBalance::from(1), left, "previous balance does not match");
                    assert_eq!(HoprBalance::zero(), right, "new balance does not match");
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
        let mut cg = ChannelGraph::new(ADDRESSES[0], Default::default());

        let ts = SystemTime::now().add(Duration::from_secs(10));
        let mut c = dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::PendingToClose(ts));

        let changes = cg.update_channel(c);
        assert!(changes.is_none(), "must not produce changes for a new channel");

        let cr = cg
            .get_channel(&ADDRESSES[0], &ADDRESSES[1])
            .context("must contain channel")?;
        assert!(c.eq(cr), "channels must be equal");

        c.balance = 0.into();
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
                    assert_eq!(HoprBalance::from(1), left, "previous balance does not match");
                    assert_eq!(HoprBalance::zero(), right, "new balance does not match");
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
        let mut cg = ChannelGraph::new(ADDRESSES[0], Default::default());
        let changes = cg.update_channel(dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::Closed));
        assert!(changes.is_none(), "must not produce changes for a closed channel");

        let c = cg.get_channel(&ADDRESSES[0], &ADDRESSES[1]);
        assert!(c.is_none(), "must not allow adding closed channels");
    }

    #[test]
    fn channel_graph_update_should_allow_pending_to_close_channels() -> anyhow::Result<()> {
        let mut cg = ChannelGraph::new(ADDRESSES[0], Default::default());
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
