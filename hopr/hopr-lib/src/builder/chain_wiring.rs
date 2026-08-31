use std::sync::Arc;

use futures::{SinkExt, StreamExt, pin_mut};
use hopr_api::{
    HoprBalance, Multiaddr, OffchainPublicKey, PeerId,
    chain::{ChainKeyOperations, WinningProbability},
    graph::{EdgeBalanceUpdate, MeasurableEdge, NetworkGraphUpdate},
    types::{
        chain::chain_events::ChainEvent,
        internal::prelude::ChannelStatus,
        primitive::{
            prelude::{Address, UnitaryFloatOps},
            traits::KeyIdMapping,
        },
    },
};
use hopr_transport::{NeighborTelemetry, PathTelemetry, SurbStore};
use parking_lot::RwLock;
use tracing::Instrument;

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
///
/// Status changes on *our own outgoing* channels are also reported to `surb_store`, so SURBs whose
/// return path starts at a relayer we can no longer pay are shed rather than replied with.
///
/// Runs until the supplied `events` stream terminates.
#[allow(clippy::too_many_arguments)]
pub(super) async fn process_chain_events<C, G, S>(
    chain_reader: C,
    graph_updater: G,
    surb_store: S,
    events: impl futures::Stream<Item = ChainEvent> + Send + 'static,
    own_chain_addr: Address,
    own_packet_key: OffchainPublicKey,
    ticket_price: Arc<RwLock<HoprBalance>>,
    win_probability: Arc<RwLock<WinningProbability>>,
    mut peer_discovery_tx: Option<hopr_utils::network_types::crossfire_sink::CrossfireSink<(PeerId, Vec<Multiaddr>)>>,
) where
    C: ChainKeyOperations + Clone + Send + Sync + 'static,
    G: NetworkGraphUpdate + Send + Sync + 'static,
    S: SurbStore + Send + Sync + 'static,
{
    pin_mut!(events);

    // Seed the face value before the first event. Startup replays on-chain state as channel events,
    // and a pricing event only follows if the price actually changes — so without this the graph
    // would record balances while `ticket_face_value()` was still `None`, and selection would price
    // every path off the fallback until the next price change happened to arrive.
    push_ticket_face_value(&graph_updater, &ticket_price, &win_probability);

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
                    let span = tracing::info_span!(
                        "peer_announcement",
                        peer = %peer_id,
                        multiaddresses = ?multiaddrs,
                    );
                    if let Err(e) = tx.send((peer_id, multiaddrs.to_vec())).instrument(span.clone()).await {
                        tracing::error!(parent: &span, %e, "peer-discovery channel closed; announcement dropped");
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
                        // Emit the raw balance, not a ticket count. Dividing by the ticket face
                        // value here would bake a live ticket price and winning probability into
                        // every edge, so each price change would stale the whole graph at once.
                        // Consumers apply the face value when they evaluate a path instead.
                        let balance = match channel.status {
                            ChannelStatus::Closed | ChannelStatus::PendingToClose(_) => None,
                            _ => Some(channel.balance.amount()),
                        };

                        tracing::debug!(
                            %channel, ?balance,
                            "recording graph edge for channel balance"
                        );
                        graph_updater.record_edge(MeasurableEdge::<NeighborTelemetry, PathTelemetry>::Balance(
                            Box::new(EdgeBalanceUpdate {
                                balance,
                                src: from,
                                dest: to,
                            }),
                        ));

                        // Only our own outgoing channels matter for SURBs: `to` is then the peer
                        // we would have to pay as the first relayer of a stored SURB's return path.
                        if src_addr == own_chain_addr {
                            match chain_reader.key_id_mapper_ref().map_key_to_id(&to) {
                                // Matched exhaustively on purpose: a wildcard would silently
                                // revalidate any future non-payable status, handing out SURBs that
                                // cannot be paid for, with no compiler warning.
                                Some(relayer) => match channel.status {
                                    ChannelStatus::Closed | ChannelStatus::PendingToClose(_) => {
                                        surb_store.invalidate_relayer(&relayer)
                                    }
                                    ChannelStatus::Open => surb_store.revalidate_relayer(&relayer),
                                },
                                None => tracing::warn!(
                                    %channel,
                                    "no key id for own channel counterparty; SURB validity not updated"
                                ),
                            }
                        }
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
                push_ticket_face_value(&graph_updater, &ticket_price, &win_probability);
            }
            ChainEvent::TicketPriceChanged(price) => {
                tracing::debug!(%price, "recording ticket price change");
                *ticket_price.write() = price;
                push_ticket_face_value(&graph_updater, &ticket_price, &win_probability);
            }
            // Redemption moves balance inside a channel that is already in the graph, and the
            // `ChannelBalance*` events that accompany it carry the capacity change. This arm was
            // previously covered by a catch-all, which is why it reads as new: making the match
            // exhaustive surfaced it rather than changed it.
            ChainEvent::TicketRedeemed(..) => {}
            // The service registry describes what a node offers, not how packets reach it, so
            // none of these affect the routing graph or the capacity inputs. They are listed
            // rather than swept up by a catch-all so that the next `ChainEvent` variant is a
            // compile-time decision here instead of a silent no-op.
            ChainEvent::ServiceRegistered(_)
            | ChainEvent::ServiceUpdated(_)
            | ChainEvent::ServiceDeregistered(..)
            | ChainEvent::ServiceTypeRegistered(..)
            | ChainEvent::ServiceTypeOwnerChanged(..)
            | ChainEvent::ServiceTypeRequirementChanged(..)
            | ChainEvent::ServiceTypeRegistrationBurnChanged(..)
            | ChainEvent::ServiceTypeUpdateBurnChanged(..)
            | ChainEvent::ServiceTypeRegistrationFeeChanged(_)
            | ChainEvent::ServiceRegistryPointerChanged(_) => {}
        }
    }
}

/// Recomputes the single-hop ticket face value and pushes it into the graph.
///
/// `price / win_probability` is the amount that makes the expected payout per packet equal the
/// ticket price, i.e. one ticket's face value. Edges store only a balance, so this is the one place
/// pricing enters path selection — and a change costs a single write rather than an edge sweep.
fn push_ticket_face_value<G>(
    graph: &G,
    ticket_price: &Arc<RwLock<HoprBalance>>,
    win_probability: &Arc<RwLock<WinningProbability>>,
) where
    G: NetworkGraphUpdate,
{
    // Both guards are released before the division: holding one while taking the other is a lock
    // order this code would then be obliged to honour everywhere else.
    let price = *ticket_price.read();
    let probability = win_probability.read().as_f64();

    // The `f64` carries the probability, it does not convert it. `as_f64` packs the 56-bit encoded
    // value into the mantissa of a number in [1, 2), and `div_f64` strips it straight back out and
    // divides in `U256` — the balance never becomes a float, and a whole probability short-circuits
    // to the identity. A hand-rolled integer division here would gain nothing but the guards.
    match price.div_f64(probability) {
        Ok(face_value) => {
            let face_value = face_value.amount();
            tracing::debug!(%face_value, "recording ticket face value change");
            graph.set_ticket_face_value(face_value);
        }
        Err(error) => tracing::error!(%error, "failed to derive the ticket face value; leaving the previous one"),
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
        graph::{EdgeBalanceUpdate, MeasurableEdge, MeasurablePath, MeasurablePeer, NetworkGraphUpdate},
        types::{
            chain::chain_events::ChainEvent,
            crypto::prelude::{ChainKeypair, Keypair, OffchainKeypair},
            internal::prelude::{AccountEntry, AccountType, ChannelEntry, ChannelStatus},
            primitive::prelude::Address,
        },
    };
    use hopr_transport::MemorySurbStore;
    use parking_lot::RwLock;

    use super::process_chain_events;

    // ---------------------------------------------------------------------------
    // Stubs
    // ---------------------------------------------------------------------------

    #[derive(Debug, Clone, thiserror::Error)]
    #[error("stub: {0}")]
    struct StubError(String);

    /// Maps only the keys it was given; empty by default, in which case it maps nothing.
    #[derive(Debug, Clone, Default)]
    struct StubMapper(HashMap<OffchainPublicKey, HoprKeyIdent>);

    impl KeyIdMapping<HoprKeyIdent, OffchainPublicKey> for StubMapper {
        fn map_key_to_id(&self, key: &OffchainPublicKey) -> Option<HoprKeyIdent> {
            self.0.get(key).copied()
        }

        fn map_id_to_public(&self, id: &HoprKeyIdent) -> Option<OffchainPublicKey> {
            self.0.iter().find_map(|(k, v)| (v == id).then_some(*k))
        }
    }

    #[derive(Debug, Clone)]
    struct StubChainKeys {
        keys: HashMap<Address, OffchainPublicKey>,
        mapper: StubMapper,
    }

    impl StubChainKeys {
        fn new(pairs: impl IntoIterator<Item = (Address, OffchainPublicKey)>) -> Self {
            Self {
                keys: pairs.into_iter().collect(),
                mapper: StubMapper::default(),
            }
        }

        fn with_key_ids(mut self, ids: impl IntoIterator<Item = (OffchainPublicKey, HoprKeyIdent)>) -> Self {
            self.mapper = StubMapper(ids.into_iter().collect());
            self
        }
    }

    impl ChainKeyOperations for StubChainKeys {
        type Error = StubError;
        type Mapper = StubMapper;

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
        Edge(Box<EdgeBalanceUpdate>),
        FaceValue(hopr_api::graph::traits::Balance),
    }

    #[derive(Debug, Clone, Default)]
    struct RecordingGraph {
        calls: Arc<Mutex<Vec<GraphCall>>>,
    }

    impl RecordingGraph {
        fn recorded(&self) -> Vec<GraphCall> {
            self.calls.lock().unwrap().clone()
        }

        fn edges(&self) -> Vec<EdgeBalanceUpdate> {
            self.recorded()
                .into_iter()
                .filter_map(|c| if let GraphCall::Edge(e) = c { Some(*e) } else { None })
                .collect()
        }

        fn face_values(&self) -> Vec<hopr_api::graph::traits::Balance> {
            self.recorded()
                .into_iter()
                .filter_map(|c| if let GraphCall::FaceValue(v) = c { Some(v) } else { None })
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
        fn set_ticket_face_value(&self, ticket_face_value: hopr_api::graph::traits::Balance) {
            self.calls.lock().unwrap().push(GraphCall::FaceValue(ticket_face_value));
        }

        fn record_edge<N, P>(&self, update: MeasurableEdge<N, P>)
        where
            N: MeasurablePeer + Clone + Send + Sync + 'static,
            P: MeasurablePath + Clone + Send + Sync + 'static,
        {
            if let MeasurableEdge::Balance(balance) = update {
                self.calls.lock().unwrap().push(GraphCall::Edge(balance));
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

    /// One event of every `ChainEvent::Service*` variant, in declaration order.
    fn service_events(node: Address, owner: Address) -> anyhow::Result<Vec<ChainEvent>> {
        use hopr_api::types::internal::prelude::{ServiceEntry, ServiceMetadata, ServiceType};

        let now = SystemTime::now();
        let entry = ServiceEntry::new(
            ServiceType::GVPN_EXIT,
            node,
            owner,
            ServiceMetadata::try_from(b"exit-node".to_vec())?,
            now,
            now,
        )?;
        let burn = HoprBalance::from(7u64);

        Ok(vec![
            ChainEvent::ServiceRegistered(entry.clone()),
            ChainEvent::ServiceUpdated(entry),
            ChainEvent::ServiceDeregistered(ServiceType::GVPN_EXIT, node),
            ChainEvent::ServiceTypeRegistered(ServiceType::GVPN_EXIT, owner),
            ChainEvent::ServiceTypeOwnerChanged(ServiceType::GVPN_EXIT, Some(owner)),
            ChainEvent::ServiceTypeRequirementChanged(ServiceType::GVPN_EXIT, Some(owner)),
            ChainEvent::ServiceTypeRegistrationBurnChanged(ServiceType::GVPN_EXIT, burn),
            ChainEvent::ServiceTypeUpdateBurnChanged(ServiceType::GVPN_EXIT, burn),
            ChainEvent::ServiceTypeRegistrationFeeChanged(burn),
            ChainEvent::ServiceRegistryPointerChanged(owner),
        ])
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
        let (tx, rx) = hopr_utils::network_types::crossfire_sink::bounded_sink_channel(64);
        process_chain_events(
            chain,
            graph,
            MemorySurbStore::default(),
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

    /// Runs the event loop against a caller-supplied SURB store, so the test can inspect it after.
    async fn run_with_surb_store(
        events: Vec<ChainEvent>,
        chain: StubChainKeys,
        surb_store: MemorySurbStore,
        own_chain_addr: Address,
        own_packet_key: OffchainPublicKey,
    ) {
        process_chain_events(
            chain,
            RecordingGraph::default(),
            surb_store,
            futures::stream::iter(events),
            own_chain_addr,
            own_packet_key,
            Arc::new(RwLock::new(HoprBalance::from(1u32))),
            Arc::new(RwLock::new(WinningProbability::ALWAYS)),
            None,
        )
        .await;
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

        // The balance is emitted as-is; pricing no longer enters the per-edge value.
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
        assert_eq!(edges[0].balance, Some(hopr_api::graph::traits::Balance::from(100u64)));
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

        // The decreased balance is emitted as-is.
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
        assert_eq!(edges[0].balance, Some(hopr_api::graph::traits::Balance::from(50u64)));
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
        assert_eq!(edges[0].balance, None);
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
            edges[0].balance, None,
            "closure-initiated must clear the balance so routing stops using this edge"
        );
    }

    #[tokio::test]
    async fn ticket_price_change_pushes_a_new_face_value() {
        let (src_offchain, src_chain) = make_keypairs();
        let (dst_offchain, dst_chain) = make_keypairs();
        let src_addr = src_chain.public().to_address();
        let dst_addr = dst_chain.public().to_address();

        let graph = RecordingGraph::default();
        let stub = StubChainKeys::new([(src_addr, *src_offchain.public()), (dst_addr, *dst_offchain.public())]);

        // A price change recomputes the face value: 20 / 1.0 = 20. The balance is untouched.
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

        assert_eq!(
            graph.face_values(),
            vec![
                hopr_api::graph::traits::Balance::from(10u64),
                hopr_api::graph::traits::Balance::from(20u64),
            ],
            "the seeded face value, then the one recomputed from the price change"
        );

        let edges = graph.edges();
        assert_eq!(edges.len(), 1);
        assert_eq!(
            edges[0].balance,
            Some(hopr_api::graph::traits::Balance::from(200u64)),
            "the emitted balance must not depend on the price"
        );
    }

    #[tokio::test]
    async fn win_probability_change_pushes_a_new_face_value() -> anyhow::Result<()> {
        let (src_offchain, src_chain) = make_keypairs();
        let (dst_offchain, dst_chain) = make_keypairs();
        let src_addr = src_chain.public().to_address();
        let dst_addr = dst_chain.public().to_address();

        let graph = RecordingGraph::default();
        let stub = StubChainKeys::new([(src_addr, *src_offchain.public()), (dst_addr, *dst_offchain.public())]);

        // A winning-probability change recomputes the face value: 10 / 0.5 = 20.
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

        assert_eq!(
            graph.face_values(),
            vec![
                hopr_api::graph::traits::Balance::from(10u64),
                hopr_api::graph::traits::Balance::from(20u64),
            ],
            "the seeded face value, then the one recomputed from the probability change"
        );

        let edges = graph.edges();
        assert_eq!(edges.len(), 1);
        assert_eq!(
            edges[0].balance,
            Some(hopr_api::graph::traits::Balance::from(100u64)),
            "the emitted balance must not depend on the winning probability"
        );
        Ok(())
    }

    /// Startup replays on-chain state as channel events, and a pricing event follows only if the
    /// price actually changed. Without a seed the graph would then hold balances while
    /// `ticket_face_value()` was still `None`, and selection would price every path off the
    /// fallback for as long as the price happened to stay put.
    #[tokio::test]
    async fn a_face_value_is_seeded_before_any_pricing_event() {
        let (src_offchain, src_chain) = make_keypairs();
        let (dst_offchain, dst_chain) = make_keypairs();
        let src_addr = src_chain.public().to_address();
        let dst_addr = dst_chain.public().to_address();

        let graph = RecordingGraph::default();
        let stub = StubChainKeys::new([(src_addr, *src_offchain.public()), (dst_addr, *dst_offchain.public())]);

        // Only a channel event: exactly the startup replay, with no price or probability change.
        run(
            vec![ChainEvent::ChannelOpened(channel(
                src_addr,
                dst_addr,
                200,
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

        assert_eq!(
            graph.face_values(),
            vec![hopr_api::graph::traits::Balance::from(10u64)],
            "the graph must know the price before it records a balance it will be compared against"
        );
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

    #[tokio::test]
    async fn announcement_should_handle_disconnected_peer_discovery_tx_gracefully() {
        let (offchain, chain) = make_keypairs();
        let addr = chain.public().to_address();
        let (tx, rx) = hopr_utils::network_types::crossfire_sink::bounded_sink_channel(1);
        drop(rx); // receiver dropped — send will return Err(Disconnected)

        process_chain_events(
            StubChainKeys::new([]),
            RecordingGraph::default(),
            MemorySurbStore::default(),
            futures::stream::iter(vec![ChainEvent::Announcement(account(*offchain.public(), addr))]),
            addr,
            *offchain.public(),
            Arc::new(RwLock::new(HoprBalance::from(10u64))),
            Arc::new(RwLock::new(WinningProbability::ALWAYS)),
            Some(tx),
        )
        .await;
    }

    /// The service registry describes what a node offers, not how packets reach it, so no
    /// `ChainEvent::Service*` variant may touch the routing graph or the capacity inputs.
    ///
    /// The trailing channel event is the regression guard: a mis-wired arm that panicked or
    /// short-circuited the loop would swallow every event behind it, which no assertion on the
    /// service events alone would notice.
    #[tokio::test]
    async fn service_events_are_ignored_without_stopping_processing() -> anyhow::Result<()> {
        let (src_offchain, src_chain) = make_keypairs();
        let (dst_offchain, dst_chain) = make_keypairs();
        let src_addr = src_chain.public().to_address();
        let dst_addr = dst_chain.public().to_address();
        let stub = StubChainKeys::new([(src_addr, *src_offchain.public()), (dst_addr, *dst_offchain.public())]);

        let events = service_events(dst_addr, src_addr)?;
        assert_eq!(events.len(), 10, "every service variant must be exercised");

        // Held here rather than inside `run`, so the values after the run can be inspected.
        let ticket_price = Arc::new(RwLock::new(HoprBalance::from(10u64)));
        let win_probability = Arc::new(RwLock::new(WinningProbability::ALWAYS));
        let graph = RecordingGraph::default();

        process_chain_events(
            stub.clone(),
            graph.clone(),
            MemorySurbStore::default(),
            futures::stream::iter(events),
            src_addr,
            *src_offchain.public(),
            ticket_price.clone(),
            win_probability.clone(),
            None,
        )
        .await;

        assert!(graph.edges().is_empty(), "service events must not record graph edges");
        assert!(graph.nodes().is_empty(), "service events must not record graph nodes");
        assert_eq!(
            graph.face_values(),
            vec![hopr_api::graph::traits::Balance::from(10u64)],
            "only the unconditional startup face-value seed should be recorded"
        );
        assert_eq!(
            *ticket_price.read(),
            HoprBalance::from(10u64),
            "service events must not change the ticket price"
        );
        assert_eq!(
            win_probability.read().as_f64(),
            WinningProbability::ALWAYS.as_f64(),
            "service events must not change the winning probability"
        );

        // Same events again, this time followed by a routed one.
        let mut events = service_events(dst_addr, src_addr)?;
        events.push(ChainEvent::ChannelOpened(channel(
            src_addr,
            dst_addr,
            100,
            ChannelStatus::Open,
        )));
        let graph = RecordingGraph::default();

        run(
            events,
            stub,
            graph.clone(),
            src_addr,
            *src_offchain.public(),
            HoprBalance::from(10u64),
            WinningProbability::ALWAYS,
        )
        .await;

        let edges = graph.edges();
        assert_eq!(
            edges.len(),
            1,
            "the event following the service ones must still be routed"
        );
        assert_eq!(
            edges.len(),
            1,
            "the event following the service ones must still be routed"
        );
        assert_eq!(
            edges[0].balance,
            Some(hopr_api::graph::traits::Balance::from(100u64)),
            "the balance inputs must have survived the service events"
        );
        assert_eq!(edges[0].src, *src_offchain.public());
        assert_eq!(edges[0].dest, *dst_offchain.public());
        assert!(graph.nodes().is_empty(), "no service event may record a graph node");

        Ok(())
    }

    // ---------------------------------------------------------------------------
    // SURB store invalidation
    // ---------------------------------------------------------------------------

    /// Sets up `me -> peer` with `peer` mapped to key id 1, and runs the given channel events.
    async fn run_own_channel_events(
        statuses: impl IntoIterator<Item = ChannelStatus>,
    ) -> (MemorySurbStore, HoprKeyIdent) {
        let (me_offchain, me_chain) = make_keypairs();
        let (peer_offchain, peer_chain) = make_keypairs();
        let (me_addr, peer_addr) = (me_chain.public().to_address(), peer_chain.public().to_address());
        let peer_id = HoprKeyIdent::from(1u32);

        let stub = StubChainKeys::new([(me_addr, *me_offchain.public()), (peer_addr, *peer_offchain.public())])
            .with_key_ids([(*peer_offchain.public(), peer_id)]);

        let surb_store = MemorySurbStore::default();
        let events = statuses
            .into_iter()
            .map(|status| ChainEvent::ChannelOpened(channel(me_addr, peer_addr, 100, status)))
            .collect();

        run_with_surb_store(events, stub, surb_store.clone(), me_addr, *me_offchain.public()).await;

        (surb_store, peer_id)
    }

    #[tokio::test]
    async fn closing_an_own_outgoing_channel_should_invalidate_that_relayer() {
        let (store, peer_id) =
            run_own_channel_events([ChannelStatus::PendingToClose(std::time::SystemTime::now())]).await;
        assert!(store.is_relayer_invalidated(&peer_id), "PendingToClose must invalidate");

        let (store, peer_id) = run_own_channel_events([ChannelStatus::Closed]).await;
        assert!(store.is_relayer_invalidated(&peer_id), "Closed must invalidate");
    }

    #[tokio::test]
    async fn reopening_an_own_outgoing_channel_should_revalidate_that_relayer() {
        let (store, peer_id) = run_own_channel_events([ChannelStatus::Closed, ChannelStatus::Open]).await;
        assert!(!store.is_relayer_invalidated(&peer_id), "re-opening must revalidate");
    }

    #[tokio::test]
    async fn a_channel_that_is_not_ours_should_not_invalidate_anything() {
        // Same topology, but the event is for a channel between two other parties.
        let (me_offchain, me_chain) = make_keypairs();
        let (a_offchain, a_chain) = make_keypairs();
        let (b_offchain, b_chain) = make_keypairs();
        let (me_addr, a_addr, b_addr) = (
            me_chain.public().to_address(),
            a_chain.public().to_address(),
            b_chain.public().to_address(),
        );
        let b_id = HoprKeyIdent::from(1u32);

        let stub = StubChainKeys::new([
            (me_addr, *me_offchain.public()),
            (a_addr, *a_offchain.public()),
            (b_addr, *b_offchain.public()),
        ])
        .with_key_ids([(*b_offchain.public(), b_id)]);

        let store = MemorySurbStore::default();
        run_with_surb_store(
            vec![ChainEvent::ChannelClosed(channel(
                a_addr,
                b_addr,
                100,
                ChannelStatus::Closed,
            ))],
            stub,
            store.clone(),
            me_addr,
            *me_offchain.public(),
        )
        .await;

        assert!(
            !store.is_relayer_invalidated(&b_id),
            "someone else's channel must not affect our SURBs"
        );
    }
}
