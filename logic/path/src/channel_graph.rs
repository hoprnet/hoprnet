use std::cmp::Ordering;
use std::collections::HashMap;
use crate::errors::Result;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::primitives::Address;
use petgraph::algo::{has_path_connecting, DfsSpace};
use petgraph::dot::Dot;
use petgraph::graphmap::DiGraphMap;
use petgraph::visit::{EdgeFiltered, EdgeRef, NodeFiltered};
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use futures::sink::drain;
use petgraph::graph::{EdgeReference, IndexType};
use petgraph::prelude::{NodeIndex, StableDiGraph};
use tracing::{debug, info};

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
    /// Network quality of this channel at the transport level (if any).
    pub quality: Option<f64>,
}

impl std::fmt::Display for ChannelEdge {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}; stake {}; quality: {}",
            self.channel,
            self.channel.balance,
            self.quality.unwrap_or(-1_f64)
        )
    }
}

/// Represents a node in the Channel Graph.
/// This is typically represented by an on-chain address and ping quality, which
/// represents some kind of node's liveness as perceived by us.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Node {
    /// Node's on-chain address.
    pub address: Address,
    /// Liveness of the node.
    pub quality: f64,
}


/// Implements a HOPR payment channel graph (directed) cached in-memory.
///
/// This structure is useful for tracking channel state changes and
/// packet path finding.
/// The structure is updated only from the Indexer and therefore contains only
/// the channels *seen* on-chain. The network qualities are also
/// added to the graph on the fly.
/// Using this structure is much faster than querying the DB and therefore
/// is preferred for per-packet path-finding computations.
/// Per default, the graph does not track channels in `Closed` state, and therefore
/// cannot detect channel re-openings.
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
            METRIC_NUMBER_OF_CHANNELS.decrement(&["out"], 0.0);
            METRIC_NUMBER_OF_CHANNELS.decrement(&["in"], 0.0);
        }

        Self {
            me,
            indices: HashMap::new(),
            graph: StableDiGraph::default(),
        }
    }

    /// Checks if the channel is incoming to or outgoing from this node
    pub fn is_own_channel(&self, channel: &ChannelEntry) -> bool {
        channel.destination == self.me || channel.source == self.me
    }

    /// Convenience method to get this node's own address
    pub fn my_address(&self) -> Address {
        self.me
    }

    fn get_edge(&self, src: Address, dst: Address) -> Option<petgraph::stable_graph::EdgeReference<ChannelEdge>> {
        let (src_idx, dst_idx) = self.indices.get(&src)
            .and_then(|src| self.indices.get(&dst).map(|dst| (*src, *dst)))?;
        self.graph
            .edges_connecting(src_idx, dst_idx)
            .next()
    }

    /// Looks up an `Open` or `PendingToClose` channel given the source and destination.
    /// Returns `None` if no such edge exists in the graph.
    pub fn get_channel(&self, source: Address, destination: Address) -> Option<&ChannelEntry> {
        self.get_edge(source, destination)
            .map(|e| &e.weight().channel)
        // self.graph.edge_weight(src_idx, dst_idx).map(|w| &w.channel)
    }

    /// Gets all `Open` outgoing channels going from the given `Address`
    pub fn open_channels_from(&self, source: Address) -> impl Iterator<Item = (Node, Node, &ChannelEdge)> {
        if let Some(idx) = self.indices.get(&source) {
            self.graph
                .edges_directed(*idx, Direction::Outgoing)
                .filter(|c| c.weight().channel.status == ChannelStatus::Open)
        } else {
            vec![].into_iter()
        }
    }

    /// Checks whether there's any path via Open channels that connects `source` and `destination`
    /// This does not need to be necessarily a multi-hop path.
    pub fn has_path(&self, source: Address, destination: Address) -> bool {
        let only_open_graph = EdgeFiltered::from_fn(&self.graph, |e| e.weight().channel.status == ChannelStatus::Open);
        has_path_connecting(&only_open_graph, source, destination, None)
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

        // Remove the edge since we don't allow Closed channels
        if channel.status == ChannelStatus::Closed {
            let ret = self.get_edge(channel.source, channel.destination)
                .and_then(|e| self.graph.remove_edge(e.id()))
                .map(|old_value| ChannelChange::diff_channels(&old_value.channel, &channel));

            debug!("removed {channel}");

            return ret;
        }

        if let Some(old_value) = self.get_edge(channel.destination, channel.destination)
            .and_then(|e| self.graph.edge_weight_mut(e.id())) {
            let old_channel = old_value.channel;
            old_value.channel = channel;

            let ret = ChannelChange::diff_channels(&old_channel, &channel);
            debug!(
                "updated {channel}: {}",
                ret.iter().map(ChannelChange::to_string).collect::<Vec<_>>().join(",")
            );
            Some(ret)
        } else {
            let src = *self.indices.entry(channel.source)
                .or_insert_with(|| self.graph.add_node(Node { address: channel.source, quality: 0.0 }));

            let dst = *self.indices.entry(channel.destination)
                .or_insert_with(|| self.graph.add_node(Node { address: channel.destination, quality: 0.0 }));

            let weighted = ChannelEdge { channel, quality: None };

            self.graph.add_edge(src, dst, weighted);
            debug!("new {channel}");

            None
        }
    }

    /// Updates the quality of a node.
    /// The given quality value must be always non-negative
    pub fn update_node_quality(&mut self, address: Address, quality: f64) {
        assert!(quality >= 0_f64, "quality must be non-negative");
        if let Some(node) = self.indices.get(&address).and_then(|node| self.graph.node_weight_mut(*node)) {
            node.quality = quality;
        } else {
            self.indices.insert(address, self.graph.add_node(Node { address, quality }));
        }
        debug!("updated quality of {address} to {quality}");
    }

    /// Updates the quality value of network connection between `source` and `destination`
    /// The given quality value must be always non-negative
    pub fn update_channel_quality(&mut self, source: Address, destination: Address, quality: f64) {
        assert!(quality >= 0_f64, "quality must be non-negative");
        if let Some(channel) = self
            .get_edge(source, destination)
            .and_then(|e| self.graph.edge_weight_mut(e.id())) {
            if quality != channel.quality.unwrap_or(-1_f64) {
                channel.quality = Some(quality);
                debug!("updated quality of {} to {quality}", channel.channel);
            }
        }
    }

    /// Gets quality of the given channel. Returns `None` if no such channel exists or no
    /// quality has been set for that channel.
    pub fn get_channel_quality(&self, source: Address, destination: Address) -> Option<f64> {
        self.get_edge(source, destination)
            .and_then(|e| self.graph.edge_weight(e.id()))
            .and_then(|e| e.quality)
    }

    /// Synchronizes the channel entries in this graph with the database.
    ///
    /// The synchronization is one-way from DB to the graph, not vice versa.
    pub fn sync_channels<I>(&mut self, channels: I) -> Result<()>
    where
        I: IntoIterator<Item = ChannelEntry>,
    {
        self.graph.clear();

        let now = hopr_platform::time::native::current_time();
        let changes: usize = channels
            .into_iter()
            .filter(|c| !c.closure_time_passed(now))
            .map(|c| self.update_channel(c).map(|v| v.len()).unwrap_or(0))
            .sum();
        info!(
            edge_count = self.graph.edge_count(),
            total_changes = changes,
            "channel graph synced",
        );
        Ok(())
    }

    /// Checks whether the given channel is in the graph already.
    pub fn contains_channel(&self, channel: &ChannelEntry) -> bool {
        self.get_channel(channel.source, channel.destination).is_some()
    }

    /// Outputs the channel graph in the DOT (graphviz) format with the given `config`.
    pub fn as_dot(&self, cfg: GraphExportConfig) -> String {
        if cfg.ignore_disconnected_components {
            let only_open_graph =
                EdgeFiltered::from_fn(&self.graph, |e| e.weight().channel.status == ChannelStatus::Open);

            Dot::new(&NodeFiltered::from_fn(&self.graph, |n| {
                let mut dfs_state = DfsSpace::new(&only_open_graph);

                (has_path_connecting(&only_open_graph, self.me, n, Some(&mut dfs_state))
                    && self
                        .graph
                        .edge_weight(self.me, n)
                        .is_some_and(|v| v.quality.unwrap_or(-1f64) > 0_f64))
                    || (has_path_connecting(&only_open_graph, n, self.me, Some(&mut dfs_state))
                        && self
                            .graph
                            .edge_weight(n, self.me)
                            .is_some_and(|v| v.quality.unwrap_or(-1f64) > 0_f64))
            }))
            .to_string()
        } else if cfg.ignore_non_opened_channels {
            Dot::new(&NodeFiltered::from_fn(&self.graph, |a| {
                self.graph.neighbors_directed(a, Direction::Outgoing).any(|b| {
                    self.graph
                        .edge_weight(a, b)
                        .is_some_and(|w| w.channel.status == ChannelStatus::Open)
                }) || self.graph.neighbors_directed(a, Direction::Incoming).any(|b| {
                    self.graph
                        .edge_weight(a, b)
                        .is_some_and(|w| w.channel.status == ChannelStatus::Open)
                })
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
    use crate::channel_graph::ChannelGraph;
    use anyhow::Context;
    use hopr_crypto_types::prelude::{ChainKeypair, Keypair};
    use hopr_db_sql::channels::HoprDbChannelOperations;
    use hopr_internal_types::channels::{ChannelChange, ChannelEntry, ChannelStatus};
    use hopr_primitive_types::prelude::*;
    use lazy_static::lazy_static;
    use std::ops::{Add, Sub};
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
    fn test_channel_graph_self_addr() {
        let cg = ChannelGraph::new(ADDRESSES[0]);
        assert_eq!(ADDRESSES[0], cg.my_address(), "must produce correct self address");
    }

    #[test]
    fn test_channel_graph_has_path() {
        let mut cg = ChannelGraph::new(ADDRESSES[0]);

        let c = dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::Open);
        cg.update_channel(c);

        assert!(cg.contains_channel(&c), "must contain channel");
        assert!(cg.has_path(ADDRESSES[0], ADDRESSES[1]), "must have simple path");
        assert!(
            !cg.has_path(ADDRESSES[0], ADDRESSES[2]),
            "must not have non existent path"
        );
    }

    #[test]
    fn test_channel_graph_update_quality() -> anyhow::Result<()> {
        let mut cg = ChannelGraph::new(ADDRESSES[0]);

        let c = dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::Open);
        cg.update_channel(c);

        assert!(cg.contains_channel(&c), "must contain channel");
        assert!(
            cg.get_channel_quality(ADDRESSES[0], ADDRESSES[1]).is_none(),
            "must start with no quality info"
        );

        cg.update_channel_quality(ADDRESSES[0], ADDRESSES[1], 0.5_f64);

        let q = cg
            .get_channel_quality(ADDRESSES[0], ADDRESSES[1])
            .context("must have quality when set")?;
        assert_eq!(0.5_f64, q, "quality must be equal");

        Ok(())
    }

    #[test]
    fn test_channel_graph_is_own_channel() {
        let mut cg = ChannelGraph::new(ADDRESSES[0]);

        let c1 = dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::Open);
        let c2 = dummy_channel(ADDRESSES[1], ADDRESSES[2], ChannelStatus::Open);
        cg.update_channel(c1);
        cg.update_channel(c2);

        assert!(cg.is_own_channel(&c1), "must detect as own channel");
        assert!(!cg.is_own_channel(&c2), "must not detect as own channel");
    }

    #[test]
    fn test_channel_graph_update_changes() -> anyhow::Result<()> {
        let mut cg = ChannelGraph::new(ADDRESSES[0]);

        let mut c = dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::Open);

        let changes = cg.update_channel(c);
        assert!(changes.is_none(), "must not produce changes for a new channel");

        let cr = cg
            .get_channel(ADDRESSES[0], ADDRESSES[1])
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
            .get_channel(ADDRESSES[0], ADDRESSES[1])
            .context("must contain channel")?;
        assert!(c.eq(cr), "channels must be equal");

        Ok(())
    }

    #[test]
    fn test_channel_graph_update_changes_on_close() -> anyhow::Result<()> {
        let mut cg = ChannelGraph::new(ADDRESSES[0]);

        let ts = SystemTime::now().add(Duration::from_secs(10));
        let mut c = dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::PendingToClose(ts));

        let changes = cg.update_channel(c);
        assert!(changes.is_none(), "must not produce changes for a new channel");

        let cr = cg
            .get_channel(ADDRESSES[0], ADDRESSES[1])
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

        let cr = cg.get_channel(ADDRESSES[0], ADDRESSES[1]);
        assert!(cr.is_none(), "must not contain channel after closing");

        Ok(())
    }

    #[test]
    fn test_channel_graph_update_should_not_allow_closed_channels() {
        let mut cg = ChannelGraph::new(ADDRESSES[0]);
        let changes = cg.update_channel(dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::Closed));
        assert!(changes.is_none(), "must not produce changes for a closed channel");

        let c = cg.get_channel(ADDRESSES[0], ADDRESSES[1]);
        assert!(c.is_none(), "must not allow adding closed channels");
    }

    #[test]
    fn test_channel_graph_update_should_allow_pending_to_close_channels() -> anyhow::Result<()> {
        let mut cg = ChannelGraph::new(ADDRESSES[0]);
        let ts = SystemTime::now().add(Duration::from_secs(10));
        let changes = cg.update_channel(dummy_channel(
            ADDRESSES[0],
            ADDRESSES[1],
            ChannelStatus::PendingToClose(ts),
        ));
        assert!(changes.is_none(), "must not produce changes for a closed channel");

        cg.get_channel(ADDRESSES[0], ADDRESSES[1])
            .context("must allow PendingToClose channels")?;

        Ok(())
    }

    #[async_std::test]
    async fn test_channel_graph_sync() -> anyhow::Result<()> {
        let mut last_addr = ADDRESSES[0];
        let db = hopr_db_sql::db::HoprDb::new_in_memory(ChainKeypair::random()).await?;

        for current_addr in ADDRESSES.iter().skip(1) {
            // Open channel from last node to us
            let channel = dummy_channel(last_addr, *current_addr, ChannelStatus::Open);
            db.upsert_channel(None, channel).await?;

            last_addr = *current_addr;
        }

        // Add a closed channel between 4 -> 0
        let channel = dummy_channel(ADDRESSES[4], ADDRESSES[0], ChannelStatus::Closed);
        db.upsert_channel(None, channel).await?;

        // Add an expired "pending to close" channel between 3 -> 0
        let channel = dummy_channel(
            ADDRESSES[3],
            ADDRESSES[0],
            ChannelStatus::PendingToClose(SystemTime::now().sub(Duration::from_secs(20))),
        );
        db.upsert_channel(None, channel).await?;

        // Add a not-expired "pending to close" channel between 2 -> 0
        let channel = dummy_channel(
            ADDRESSES[2],
            ADDRESSES[0],
            ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(20))),
        );
        db.upsert_channel(None, channel).await?;

        let mut cg = ChannelGraph::new(ADDRESSES[0]);
        cg.sync_channels(db.get_all_channels(None).await?)?;

        assert!(cg.has_path(ADDRESSES[0], ADDRESSES[4]), "must have path from 0 -> 4");
        assert!(
            cg.get_channel(ADDRESSES[4], ADDRESSES[0]).is_none(),
            "must not sync closed channel"
        );
        assert!(
            cg.get_channel(ADDRESSES[3], ADDRESSES[0]).is_none(),
            "must not sync expired pending to close channel"
        );
        assert!(
            cg.get_channel(ADDRESSES[2], ADDRESSES[0])
                .is_some_and(|c| c.status != ChannelStatus::Open && !c.closure_time_passed(SystemTime::now())),
            "must sync non-expired pending to close channel"
        );
        assert!(
            db.get_all_channels(None)
                .await?
                .into_iter()
                .filter(|c| !c.closure_time_passed(SystemTime::now()))
                .all(|c| cg.contains_channel(&c)),
            "must contain all non-closed channels with non-expired grace period"
        );

        Ok(())
    }

    #[test]
    fn graph_must_serialize_and_deserialize() -> anyhow::Result<()> {
        let mut cg = ChannelGraph::new(ADDRESSES[0]);

        let c = dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::Open);
        cg.update_channel(c);
        cg.update_channel_quality(ADDRESSES[0], ADDRESSES[1], 0.5);

        let c = dummy_channel(ADDRESSES[1], ADDRESSES[2], ChannelStatus::Open);
        cg.update_channel(c);
        cg.update_channel_quality(ADDRESSES[1], ADDRESSES[2], 0.1);

        let c = dummy_channel(ADDRESSES[3], ADDRESSES[4], ChannelStatus::Open);
        cg.update_channel(c);

        let serialized = bincode::serialize(&cg)?;
        let cg2 = bincode::deserialize::<ChannelGraph>(serialized.as_ref())?;

        assert_eq!(cg.me, cg2.me);
        for (src, dst, weight) in cg.graph.all_edges() {
            assert_eq!(cg2.graph.edge_weight(src, dst), Some(weight));
        }

        Ok(())
    }
}
