use crate::errors::Result;
use chain_db::traits::HoprCoreEthereumDbActions;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::primitives::Address;
use petgraph::algo::has_path_connecting;
use petgraph::graphmap::DiGraphMap;
use petgraph::visit::{EdgeFiltered, EdgeRef};
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
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
    /// This value is currently present only for *own channels*.
    pub quality: Option<f64>,
}

/// Implements a HOPR payment channel graph (directed) cached in-memory.
///
/// This structure is useful for tracking channel state changes and
/// packet path finding.
/// The structure is updated only from the Indexer and therefore contains only
/// the channels that were *seen* on-chain. The network qualities are also
/// added to the graph on the fly.
/// Using this structure is much faster than querying the DB and therefore
/// is preferred for per-packet path-finding computations.
/// Per default, the graph does not track channels in `Closed` state, and therefore
/// cannot detect channel re-openings.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelGraph {
    me: Address,
    graph: DiGraphMap<Address, ChannelEdge>,
}

impl ChannelGraph {
    /// Maximum number of intermediate hops the automatic path finding algorithm will look for.
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
            graph: DiGraphMap::default(),
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

    /// Looks up an `Open` or `PendingToClose` channel given the source and destination.
    /// Returns `None` if no such edge exists in the graph.
    pub fn get_channel(&self, source: Address, destination: Address) -> Option<&ChannelEntry> {
        self.graph.edge_weight(source, destination).map(|w| &w.channel)
    }

    /// Gets all `Open` outgoing channels going from the given `Address`
    pub fn open_channels_from(&self, source: Address) -> impl Iterator<Item = (Address, Address, &ChannelEdge)> {
        self.graph
            .edges_directed(source, Direction::Outgoing)
            .filter(|c| c.weight().channel.status == ChannelStatus::Open)
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
            let ret = self
                .graph
                .remove_edge(channel.source, channel.destination)
                .map(|old_value| ChannelChange::diff_channels(&old_value.channel, &channel));

            info!("removed {channel}");

            return ret;
        }

        if let Some(old_value) = self.graph.edge_weight_mut(channel.source, channel.destination) {
            let old_channel = old_value.channel;
            old_value.channel = channel;

            let ret = ChannelChange::diff_channels(&old_channel, &channel);
            info!(
                "updated {channel}: {}",
                ret.iter().map(ChannelChange::to_string).collect::<Vec<_>>().join(",")
            );
            Some(ret)
        } else {
            let weighted = ChannelEdge { channel, quality: None };
            self.graph.add_edge(channel.source, channel.destination, weighted);
            info!("new {channel}");

            None
        }
    }

    /// Updates the quality value of network connection between `source` and `destination`
    /// The given quality value must be always non-negative
    pub fn update_channel_quality(&mut self, source: Address, destination: Address, quality: f64) {
        assert!(quality >= 0_f64, "quality must be non-negative");
        if let Some(channel) = self.graph.edge_weight_mut(source, destination) {
            if quality != channel.quality.unwrap_or(-1_f64) {
                channel.quality = Some(quality);
                debug!("updated quality of {} to {quality}", channel.channel);
            }
        }
    }

    /// Gets quality of the given channel. Returns `None` if no such channel exists or no
    /// quality has been set for that channel.
    pub fn get_channel_quality(&self, source: Address, destination: Address) -> Option<f64> {
        self.graph.edge_weight(source, destination).and_then(|w| w.quality)
    }

    /// Synchronizes the channel entries in this graph with the database.
    /// The synchronization is one-way from DB to the graph, not vice versa.
    pub async fn sync_channels<Db: HoprCoreEthereumDbActions>(&mut self, db: &Db) -> Result<()> {
        db.get_channels().await?.into_iter().for_each(|c| {
            self.update_channel(c);
        });
        info!("synced {} channels to the graph", self.graph.edge_count());
        Ok(())
    }

    /// Checks whether the given channel is in the graph already.
    pub fn contains_channel(&self, channel: &ChannelEntry) -> bool {
        self.graph.contains_edge(channel.source, channel.destination)
    }
}

#[cfg(test)]
mod tests {
    use crate::channel_graph::ChannelGraph;
    use chain_db::db::CoreEthereumDb;
    use chain_db::traits::HoprCoreEthereumDbActions;
    use hopr_internal_types::channels::{ChannelChange, ChannelEntry, ChannelStatus};
    use hopr_primitive_types::prelude::*;
    use lazy_static::lazy_static;
    use std::ops::Add;
    use std::str::FromStr;
    use std::time::{Duration, SystemTime};
    use utils_db::db::DB;
    use utils_db::CurrentDbShim;

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
    fn test_channel_graph_update_quality() {
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
            .expect("must have quality when set");
        assert_eq!(0.5_f64, q, "quality must be equal");
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
    fn test_channel_graph_update_changes() {
        let mut cg = ChannelGraph::new(ADDRESSES[0]);

        let mut c = dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::Open);

        let changes = cg.update_channel(c);
        assert!(changes.is_none(), "must not produce changes for a new channel");

        let cr = cg
            .get_channel(ADDRESSES[0], ADDRESSES[1])
            .expect("must contain channel");
        assert!(c.eq(cr), "channels must be equal");

        let ts = SystemTime::now().add(Duration::from_secs(10));
        c.balance = Balance::zero(BalanceType::HOPR);
        c.status = ChannelStatus::PendingToClose(ts);
        let changes = cg.update_channel(c).expect("must contain changes");
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
            .expect("must contain channel");
        assert!(c.eq(cr), "channels must be equal");
    }

    #[test]
    fn test_channel_graph_update_changes_on_close() {
        let mut cg = ChannelGraph::new(ADDRESSES[0]);

        let ts = SystemTime::now().add(Duration::from_secs(10));
        let mut c = dummy_channel(ADDRESSES[0], ADDRESSES[1], ChannelStatus::PendingToClose(ts));

        let changes = cg.update_channel(c);
        assert!(changes.is_none(), "must not produce changes for a new channel");

        let cr = cg
            .get_channel(ADDRESSES[0], ADDRESSES[1])
            .expect("must contain channel");
        assert!(c.eq(cr), "channels must be equal");

        c.balance = Balance::zero(BalanceType::HOPR);
        c.status = ChannelStatus::Closed;
        let changes = cg.update_channel(c).expect("must contain changes");
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
    fn test_channel_graph_update_should_allow_pending_to_close_channels() {
        let mut cg = ChannelGraph::new(ADDRESSES[0]);
        let ts = SystemTime::now().add(Duration::from_secs(10));
        let changes = cg.update_channel(dummy_channel(
            ADDRESSES[0],
            ADDRESSES[1],
            ChannelStatus::PendingToClose(ts),
        ));
        assert!(changes.is_none(), "must not produce changes for a closed channel");

        cg.get_channel(ADDRESSES[0], ADDRESSES[1])
            .expect("must allow PendingToClose channels");
    }

    #[async_std::test]
    async fn test_channel_graph_sync() {
        let testing_snapshot = Snapshot::default();
        let mut last_addr = ADDRESSES[0];
        let mut db = CoreEthereumDb::new(DB::new(CurrentDbShim::new_in_memory().await), last_addr);

        for current_addr in ADDRESSES.iter().skip(1) {
            // Open channel from last node to us
            let channel = dummy_channel(last_addr, *current_addr, ChannelStatus::Open);
            db.update_channel_and_snapshot(&channel.get_id(), &channel, &testing_snapshot)
                .await
                .unwrap();

            last_addr = *current_addr;
        }

        // Add a pending to close channel between 4 -> 0
        let channel = dummy_channel(ADDRESSES[4], ADDRESSES[0], ChannelStatus::Closed);
        db.update_channel_and_snapshot(&channel.get_id(), &channel, &testing_snapshot)
            .await
            .unwrap();

        let mut cg = ChannelGraph::new(ADDRESSES[0]);
        cg.sync_channels(&db).await.expect("should sync graph");

        assert!(cg.has_path(ADDRESSES[0], ADDRESSES[4]), "must have path from 0 -> 4");
        assert!(
            cg.get_channel(ADDRESSES[4], ADDRESSES[0]).is_none(),
            "must not sync closed channel"
        );
        assert!(
            db.get_channels()
                .await
                .unwrap()
                .into_iter()
                .filter(|c| c.status != ChannelStatus::Closed)
                .all(|c| cg.contains_channel(&c)),
            "must contain all non-closed channels"
        );
    }
}
