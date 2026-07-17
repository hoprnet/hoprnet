use std::sync::Arc;

use futures::{StreamExt, pin_mut};
use hopr_api::{
    HoprBalance, Multiaddr, OffchainPublicKey, PeerId,
    chain::{ChainKeyOperations, WinningProbability},
    graph::{EdgeCapacityUpdate, MeasurableEdge, NetworkGraphUpdate},
    types::{
        chain::chain_events::ChainEvent,
        internal::prelude::ChannelStatus,
        primitive::prelude::{Address, UnitaryFloatOps},
    },
};
use hopr_transport::{NeighborTelemetry, PathTelemetry};
use parking_lot::RwLock;

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_CHANNELS_COUNT: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_channels_count",
        "Number of open channels of the node per direction",
        &["direction"]
    ).unwrap();
}

/// Processes chain events and records them as graph updates.
///
/// Drives the chain-to-graph edge of the topology pipeline: converts incoming on-chain
/// `ChainEvent`s into [`NetworkGraphUpdate`] calls so the routing graph stays current.
/// When `peer_discovery_tx` is `Some`, each [`ChainEvent::Announcement`] is also forwarded
/// to the p2p network layer so it can initiate connections to newly discovered peers.
/// Runs until the supplied `events` stream terminates.
#[allow(clippy::too_many_arguments)]
pub(super) async fn process_chain_events<C, G>(
    chain_reader: C,
    graph_updater: G,
    events: impl futures::Stream<Item = ChainEvent> + Send + 'static,
    own_chain_addr: Address,
    own_packet_key: OffchainPublicKey,
    ticket_price: Arc<RwLock<HoprBalance>>,
    win_probability: Arc<RwLock<WinningProbability>>,
    mut peer_discovery_tx: Option<futures::channel::mpsc::Sender<(PeerId, Vec<Multiaddr>)>>,
) where
    C: ChainKeyOperations + Clone + Send + Sync + 'static,
    G: NetworkGraphUpdate + Send + Sync + 'static,
{
    pin_mut!(events);

    // Tracks the node's currently-open channel IDs per direction so `hopr_channels_count`
    // can be maintained incrementally from channel events. The initial on-chain state is
    // replayed as `ChannelOpened` events by the state-sync subscription at startup, so the
    // sets are seeded correctly without an explicit query. Set operations are idempotent,
    // making this robust to duplicated events.
    #[cfg(all(feature = "telemetry", not(test)))]
    let (mut incoming_open, mut outgoing_open) = (std::collections::HashSet::new(), std::collections::HashSet::new());

    while let Some(chain_event) = events.next().await {
        tracing::debug!(event = %chain_event, "processing chain event");
        match chain_event {
            ChainEvent::Announcement(account) => {
                tracing::debug!(
                    account = %account.public_key,
                    "recording graph node for announced account"
                );
                graph_updater.record_node(account.public_key);
                if let Some(ref mut tx) = peer_discovery_tx {
                    let peer_id: PeerId = account.public_key.into();
                    let multiaddrs = account.get_multiaddrs();
                    let _span = tracing::info_span!(
                        "peer_announcement",
                        peer = %peer_id,
                        multiaddresses = ?multiaddrs,
                    )
                    .entered();
                    if let Err(e) = tx.try_send((peer_id, multiaddrs.to_vec())) {
                        tracing::error!(%e, "peer-discovery channel full or closed; announcement dropped");
                    }
                }
            }
            ChainEvent::ChannelOpened(channel)
            | ChainEvent::ChannelClosureInitiated(channel)
            | ChainEvent::ChannelClosed(channel)
            | ChainEvent::ChannelBalanceIncreased(channel, _)
            | ChainEvent::ChannelBalanceDecreased(channel, _) => {
                let src_addr = channel.source;
                let dst_addr = channel.destination;

                #[cfg(all(feature = "telemetry", not(test)))]
                {
                    let channel_id = *channel.get_id();
                    let is_open = matches!(channel.status, ChannelStatus::Open);
                    if src_addr == own_chain_addr {
                        if is_open {
                            outgoing_open.insert(channel_id);
                        } else {
                            outgoing_open.remove(&channel_id);
                        }
                        METRIC_CHANNELS_COUNT.set(&["outgoing"], outgoing_open.len() as f64);
                    } else if dst_addr == own_chain_addr {
                        if is_open {
                            incoming_open.insert(channel_id);
                        } else {
                            incoming_open.remove(&channel_id);
                        }
                        METRIC_CHANNELS_COUNT.set(&["incoming"], incoming_open.len() as f64);
                    }
                }

                let reader = chain_reader.clone();
                let keys = hopr_utils::runtime::prelude::spawn_blocking(move || {
                    let resolve = |addr: Address| {
                        if addr == own_chain_addr {
                            return Ok(Some(own_packet_key));
                        }
                        reader.chain_key_to_packet_key(&addr).map_err(anyhow::Error::from)
                    };
                    resolve(src_addr).and_then(|src| resolve(dst_addr).map(|dst| src.zip(dst)))
                })
                .await
                .map_err(anyhow::Error::from)
                .flatten();

                match keys {
                    Ok(Some((from, to))) => {
                        let capacity =
                            match channel.status {
                                ChannelStatus::Closed | ChannelStatus::PendingToClose(_) => None,
                                _ => ticket_price.read().div_f64(win_probability.read().as_f64()).ok().map(
                                    |ticket_value| {
                                        channel
                                            .balance
                                            .amount()
                                            .checked_div(ticket_value.amount())
                                            .map(|v| v.low_u128())
                                            .unwrap_or(u128::MAX)
                                    },
                                ),
                            };

                        tracing::debug!(
                            %channel, ?capacity,
                            "recording graph edge for channel capacity"
                        );
                        graph_updater.record_edge(MeasurableEdge::<NeighborTelemetry, PathTelemetry>::Capacity(
                            Box::new(EdgeCapacityUpdate {
                                capacity,
                                src: from,
                                dest: to,
                            }),
                        ));
                    }
                    Ok(None) => {
                        tracing::error!(
                            %channel,
                            "could not find packet keys for channel endpoints"
                        );
                    }
                    Err(error) => {
                        tracing::error!(
                            %error, %channel,
                            "failed to convert chain keys to packet keys"
                        );
                    }
                }
            }
            ChainEvent::WinningProbabilityIncreased(prob) | ChainEvent::WinningProbabilityDecreased(prob) => {
                tracing::debug!(%prob, "recording winning probability change");
                *win_probability.write() = prob;
            }
            ChainEvent::TicketPriceChanged(price) => {
                tracing::debug!(%price, "recording ticket price change");
                *ticket_price.write() = price;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
        time::SystemTime,
    };

    use anyhow::Context as _;
    use hopr_api::{
        HoprBalance, OffchainPublicKey,
        chain::{ChainKeyOperations, HoprKeyIdent, KeyIdMapping, WinningProbability},
        graph::{EdgeCapacityUpdate, MeasurableEdge, MeasurablePath, MeasurablePeer, NetworkGraphUpdate},
        types::{
            chain::chain_events::ChainEvent,
            crypto::prelude::{ChainKeypair, Keypair, OffchainKeypair},
            internal::prelude::{AccountEntry, AccountType, ChannelEntry, ChannelStatus},
            primitive::prelude::Address,
        },
    };
    use parking_lot::RwLock;

    use super::process_chain_events;

    // ---------------------------------------------------------------------------
    // Stubs
    // ---------------------------------------------------------------------------

    #[derive(Debug, Clone, thiserror::Error)]
    #[error("stub: {0}")]
    struct StubError(String);

    #[derive(Debug, Clone)]
    struct NoopMapper;

    impl KeyIdMapping<HoprKeyIdent, OffchainPublicKey> for NoopMapper {
        fn map_key_to_id(&self, _key: &OffchainPublicKey) -> Option<HoprKeyIdent> {
            None
        }

        fn map_id_to_public(&self, _id: &HoprKeyIdent) -> Option<OffchainPublicKey> {
            None
        }
    }

    #[derive(Debug, Clone)]
    struct StubChainKeys {
        keys: HashMap<Address, OffchainPublicKey>,
        mapper: NoopMapper,
    }

    impl StubChainKeys {
        fn new(pairs: impl IntoIterator<Item = (Address, OffchainPublicKey)>) -> Self {
            Self {
                keys: pairs.into_iter().collect(),
                mapper: NoopMapper,
            }
        }
    }

    impl ChainKeyOperations for StubChainKeys {
        type Error = StubError;
        type Mapper = NoopMapper;

        fn chain_key_to_packet_key(&self, chain: &Address) -> Result<Option<OffchainPublicKey>, Self::Error> {
            Ok(self.keys.get(chain).copied())
        }

        fn packet_key_to_chain_key(&self, packet: &OffchainPublicKey) -> Result<Option<Address>, Self::Error> {
            Ok(self.keys.iter().find_map(|(a, k)| (k == packet).then_some(*a)))
        }

        fn key_id_mapper_ref(&self) -> &Self::Mapper {
            &self.mapper
        }
    }

    #[derive(Debug, Clone)]
    enum GraphCall {
        Node(OffchainPublicKey),
        Edge(Box<EdgeCapacityUpdate>),
    }

    #[derive(Debug, Clone, Default)]
    struct RecordingGraph {
        calls: Arc<Mutex<Vec<GraphCall>>>,
    }

    impl RecordingGraph {
        fn recorded(&self) -> Vec<GraphCall> {
            self.calls.lock().unwrap().clone()
        }

        fn edges(&self) -> Vec<EdgeCapacityUpdate> {
            self.recorded()
                .into_iter()
                .filter_map(|c| if let GraphCall::Edge(e) = c { Some(*e) } else { None })
                .collect()
        }

        fn nodes(&self) -> Vec<OffchainPublicKey> {
            self.recorded()
                .into_iter()
                .filter_map(|c| if let GraphCall::Node(n) = c { Some(n) } else { None })
                .collect()
        }
    }

    impl NetworkGraphUpdate for RecordingGraph {
        fn record_edge<N, P>(&self, update: MeasurableEdge<N, P>)
        where
            N: MeasurablePeer + Clone + Send + Sync + 'static,
            P: MeasurablePath + Clone + Send + Sync + 'static,
        {
            if let MeasurableEdge::Capacity(cap) = update {
                self.calls.lock().unwrap().push(GraphCall::Edge(cap));
            }
        }

        fn record_node<N>(&self, update: N)
        where
            N: hopr_api::graph::MeasurableNode + Clone + Send + Sync + 'static,
        {
            self.calls.lock().unwrap().push(GraphCall::Node(update.into()));
        }
    }

    // ---------------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------------

    fn make_keypairs() -> (OffchainKeypair, ChainKeypair) {
        (OffchainKeypair::random(), ChainKeypair::random())
    }

    fn channel(src: Address, dst: Address, balance: u128, status: ChannelStatus) -> ChannelEntry {
        ChannelEntry::builder()
            .source(src)
            .destination(dst)
            .amount(balance)
            .status(status)
            .build()
            .expect("valid channel")
    }

    fn account(key: OffchainPublicKey, addr: Address) -> AccountEntry {
        use hopr_api::types::primitive::prelude::KeyIdent;
        AccountEntry {
            public_key: key,
            chain_addr: addr,
            entry_type: AccountType::NotAnnounced,
            safe_address: None,
            key_id: KeyIdent::default(),
        }
    }

    async fn run(
        events: Vec<ChainEvent>,
        chain: StubChainKeys,
        graph: RecordingGraph,
        own_chain_addr: Address,
        own_packet_key: OffchainPublicKey,
        ticket_price: HoprBalance,
        win_probability: WinningProbability,
    ) {
        let _ = run_with_peer_discovery(
            events,
            chain,
            graph,
            own_chain_addr,
            own_packet_key,
            ticket_price,
            win_probability,
        )
        .await;
    }

    async fn run_with_peer_discovery(
        events: Vec<ChainEvent>,
        chain: StubChainKeys,
        graph: RecordingGraph,
        own_chain_addr: Address,
        own_packet_key: OffchainPublicKey,
        ticket_price: HoprBalance,
        win_probability: WinningProbability,
    ) -> Vec<(hopr_api::PeerId, Vec<hopr_api::Multiaddr>)> {
        use futures::StreamExt;
        let (tx, rx) = futures::channel::mpsc::channel(64);
        process_chain_events(
            chain,
            graph,
            futures::stream::iter(events),
            own_chain_addr,
            own_packet_key,
            Arc::new(RwLock::new(ticket_price)),
            Arc::new(RwLock::new(win_probability)),
            Some(tx),
        )
        .await;
        rx.collect().await
    }

    // ---------------------------------------------------------------------------
    // Tests
    // ---------------------------------------------------------------------------

    #[tokio::test]
    async fn announcement_records_node() {
        let (offchain, chain) = make_keypairs();
        let addr = chain.public().to_address();
        let graph = RecordingGraph::default();

        run(
            vec![ChainEvent::Announcement(account(*offchain.public(), addr))],
            StubChainKeys::new([]),
            graph.clone(),
            addr,
            *offchain.public(),
            HoprBalance::from(10u64),
            WinningProbability::ALWAYS,
        )
        .await;

        assert_eq!(graph.nodes(), vec![*offchain.public()]);
        assert!(graph.edges().is_empty());
    }

    #[tokio::test]
    async fn announcement_should_forward_to_peer_discovery_when_tx_is_set() -> anyhow::Result<()> {
        use std::str::FromStr;

        use hopr_api::types::internal::prelude::AccountType;

        let (offchain, chain) = make_keypairs();
        let addr = chain.public().to_address();
        let multiaddr = hopr_api::Multiaddr::from_str("/ip4/1.2.3.4/tcp/9000").context("parse multiaddr")?;
        let entry = AccountEntry {
            entry_type: AccountType::Announced(vec![multiaddr.clone()]),
            ..account(*offchain.public(), addr)
        };
        let graph = RecordingGraph::default();

        let received = run_with_peer_discovery(
            vec![ChainEvent::Announcement(entry)],
            StubChainKeys::new([]),
            graph.clone(),
            addr,
            *offchain.public(),
            HoprBalance::from(10u64),
            WinningProbability::ALWAYS,
        )
        .await;

        assert_eq!(received.len(), 1, "expected exactly one peer-discovery event");
        let (peer_id, addrs) = &received[0];
        assert_eq!(
            *peer_id,
            hopr_api::PeerId::from(*offchain.public()),
            "peer id must match the announced account's public key"
        );
        assert_eq!(addrs, &vec![multiaddr], "multiaddrs must be forwarded unchanged");
        assert_eq!(
            graph.nodes(),
            vec![*offchain.public()],
            "graph must also record the node"
        );
        Ok(())
    }

    #[tokio::test]
    async fn channel_opened_records_capacity() {
        let (src_offchain, src_chain) = make_keypairs();
        let (dst_offchain, dst_chain) = make_keypairs();
        let src_addr = src_chain.public().to_address();
        let dst_addr = dst_chain.public().to_address();

        let graph = RecordingGraph::default();
        let stub = StubChainKeys::new([(src_addr, *src_offchain.public()), (dst_addr, *dst_offchain.public())]);

        // price=10, win_prob=1.0, balance=100 → capacity = 100/(10/1.0) = 10
        run(
            vec![ChainEvent::ChannelOpened(channel(
                src_addr,
                dst_addr,
                100,
                ChannelStatus::Open,
            ))],
            stub,
            graph.clone(),
            src_addr,
            *src_offchain.public(),
            HoprBalance::from(10u64),
            WinningProbability::ALWAYS,
        )
        .await;

        let edges = graph.edges();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].capacity, Some(10));
        assert_eq!(edges[0].src, *src_offchain.public());
        assert_eq!(edges[0].dest, *dst_offchain.public());
    }

    #[tokio::test]
    async fn channel_balance_decreased_records_updated_capacity() {
        let (src_offchain, src_chain) = make_keypairs();
        let (dst_offchain, dst_chain) = make_keypairs();
        let src_addr = src_chain.public().to_address();
        let dst_addr = dst_chain.public().to_address();

        let graph = RecordingGraph::default();
        let stub = StubChainKeys::new([(src_addr, *src_offchain.public()), (dst_addr, *dst_offchain.public())]);

        // price=10, win_prob=1.0, balance=50 after decrease → capacity = 50/10 = 5
        run(
            vec![ChainEvent::ChannelBalanceDecreased(
                channel(src_addr, dst_addr, 50, ChannelStatus::Open),
                HoprBalance::from(50u64),
            )],
            stub,
            graph.clone(),
            src_addr,
            *src_offchain.public(),
            HoprBalance::from(10u64),
            WinningProbability::ALWAYS,
        )
        .await;

        let edges = graph.edges();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].capacity, Some(5));
    }

    #[tokio::test]
    async fn channel_closed_records_capacity_none() {
        let (src_offchain, src_chain) = make_keypairs();
        let (dst_offchain, dst_chain) = make_keypairs();
        let src_addr = src_chain.public().to_address();
        let dst_addr = dst_chain.public().to_address();

        let graph = RecordingGraph::default();
        let stub = StubChainKeys::new([(src_addr, *src_offchain.public()), (dst_addr, *dst_offchain.public())]);

        run(
            vec![ChainEvent::ChannelClosed(channel(
                src_addr,
                dst_addr,
                0,
                ChannelStatus::Closed,
            ))],
            stub,
            graph.clone(),
            src_addr,
            *src_offchain.public(),
            HoprBalance::from(10u64),
            WinningProbability::ALWAYS,
        )
        .await;

        let edges = graph.edges();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].capacity, None);
    }

    /// Regression test: before the fix, ChannelClosureInitiated was a no-op and the
    /// graph kept the prior `Some(N)` capacity for the channel lifetime of the close
    /// timeout window, allowing routing to keep picking the dying edge.
    #[tokio::test]
    async fn channel_closure_initiated_records_capacity_none() {
        let (src_offchain, src_chain) = make_keypairs();
        let (dst_offchain, dst_chain) = make_keypairs();
        let src_addr = src_chain.public().to_address();
        let dst_addr = dst_chain.public().to_address();

        let graph = RecordingGraph::default();
        let stub = StubChainKeys::new([(src_addr, *src_offchain.public()), (dst_addr, *dst_offchain.public())]);

        run(
            vec![ChainEvent::ChannelClosureInitiated(channel(
                src_addr,
                dst_addr,
                100,
                ChannelStatus::PendingToClose(SystemTime::now()),
            ))],
            stub,
            graph.clone(),
            src_addr,
            *src_offchain.public(),
            HoprBalance::from(10u64),
            WinningProbability::ALWAYS,
        )
        .await;

        let edges = graph.edges();
        assert_eq!(edges.len(), 1, "closure-initiated must emit a graph update");
        assert_eq!(
            edges[0].capacity, None,
            "closure-initiated must zero out the capacity so routing stops using this edge"
        );
    }

    #[tokio::test]
    async fn ticket_price_change_affects_subsequent_capacity() {
        let (src_offchain, src_chain) = make_keypairs();
        let (dst_offchain, dst_chain) = make_keypairs();
        let src_addr = src_chain.public().to_address();
        let dst_addr = dst_chain.public().to_address();

        let graph = RecordingGraph::default();
        let stub = StubChainKeys::new([(src_addr, *src_offchain.public()), (dst_addr, *dst_offchain.public())]);

        // initial price=10; after price change to 20, balance=200 → 200/(20/1.0) = 10
        run(
            vec![
                ChainEvent::TicketPriceChanged(HoprBalance::from(20u64)),
                ChainEvent::ChannelOpened(channel(src_addr, dst_addr, 200, ChannelStatus::Open)),
            ],
            stub,
            graph.clone(),
            src_addr,
            *src_offchain.public(),
            HoprBalance::from(10u64),
            WinningProbability::ALWAYS,
        )
        .await;

        let edges = graph.edges();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].capacity, Some(10));
    }

    #[tokio::test]
    async fn win_probability_change_affects_subsequent_capacity() -> anyhow::Result<()> {
        let (src_offchain, src_chain) = make_keypairs();
        let (dst_offchain, dst_chain) = make_keypairs();
        let src_addr = src_chain.public().to_address();
        let dst_addr = dst_chain.public().to_address();

        let graph = RecordingGraph::default();
        let stub = StubChainKeys::new([(src_addr, *src_offchain.public()), (dst_addr, *dst_offchain.public())]);

        // initial win_prob=1.0; after decrease to 0.5, balance=100, price=10 → 100/(10/0.5) = 5
        let new_prob = WinningProbability::try_from_f64(0.5).context("0.5 is a valid winning probability")?;
        run(
            vec![
                ChainEvent::WinningProbabilityDecreased(new_prob),
                ChainEvent::ChannelOpened(channel(src_addr, dst_addr, 100, ChannelStatus::Open)),
            ],
            stub,
            graph.clone(),
            src_addr,
            *src_offchain.public(),
            HoprBalance::from(10u64),
            WinningProbability::ALWAYS,
        )
        .await;

        let edges = graph.edges();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].capacity, Some(5));
        Ok(())
    }

    #[tokio::test]
    async fn unknown_chain_key_produces_no_graph_update() {
        let (src_offchain, src_chain) = make_keypairs();
        let (_, dst_chain) = make_keypairs();
        let src_addr = src_chain.public().to_address();
        let dst_addr = dst_chain.public().to_address();

        let graph = RecordingGraph::default();
        // dst is NOT in the stub map → chain_key_to_packet_key returns None for dst
        let stub = StubChainKeys::new([(src_addr, *src_offchain.public())]);

        run(
            vec![ChainEvent::ChannelOpened(channel(
                src_addr,
                dst_addr,
                100,
                ChannelStatus::Open,
            ))],
            stub,
            graph.clone(),
            src_addr,
            *src_offchain.public(),
            HoprBalance::from(10u64),
            WinningProbability::ALWAYS,
        )
        .await;

        assert!(graph.edges().is_empty(), "unknown key must produce no graph update");
    }

    #[tokio::test]
    async fn self_address_resolved_via_own_packet_key() {
        let (own_offchain, own_chain) = make_keypairs();
        let (dst_offchain, dst_chain) = make_keypairs();
        let own_chain_addr = own_chain.public().to_address();
        let dst_addr = dst_chain.public().to_address();

        let graph = RecordingGraph::default();
        // own_chain_addr not in stub — must be resolved via own_packet_key
        let stub = StubChainKeys::new([(dst_addr, *dst_offchain.public())]);

        run(
            vec![ChainEvent::ChannelOpened(channel(
                own_chain_addr,
                dst_addr,
                100,
                ChannelStatus::Open,
            ))],
            stub,
            graph.clone(),
            own_chain_addr,
            *own_offchain.public(),
            HoprBalance::from(10u64),
            WinningProbability::ALWAYS,
        )
        .await;

        let edges = graph.edges();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].src, *own_offchain.public());
        assert_eq!(edges[0].dest, *dst_offchain.public());
    }
}
