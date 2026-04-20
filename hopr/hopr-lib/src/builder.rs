use std::{convert::identity, sync::Arc, time::Duration};

use futures::{FutureExt, StreamExt, channel::mpsc::channel};
// Re-exports for downstream convenience
pub use hopr_api::types::crypto::keypairs::Keypair;
pub use hopr_api::types::crypto::prelude::{ChainKeypair, OffchainKeypair};
use hopr_api::{
    chain::{AnnouncementError, HoprChainApi, SafeRegistrationError, StateSyncOptions},
    ct::{CoverTrafficGeneration, ProbingTrafficGeneration},
    graph::{EdgeCapacityUpdate, HoprGraphApi},
    network::{NetworkBuilder, NetworkStreamControl},
    node::{AtomicHoprState, HoprState, NodeOnchainIdentity, TicketEvent},
    tickets::{TicketFactory, TicketManagement},
    types::{
        chain::chain_events::ChainEvent,
        primitive::prelude::{Address, UnitaryFloatOps},
    },
};
use hopr_async_runtime::AbortableList;
use hopr_network_types::addr::is_public_address;
use hopr_transport::HoprTransport;
use tokio::spawn;
use validator::Validate;

use crate::{
    Hopr, HoprLibError, HoprLibProcess, IncomingSession, MIN_NATIVE_BALANCE, NODE_READY_TIMEOUT,
    SUGGESTED_NATIVE_BALANCE, config::HoprLibConfig, constants,
};

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_PROCESS_START_TIME:  hopr_metrics::SimpleGauge =  hopr_metrics::SimpleGauge::new(
        "hopr_start_time",
        "The unix timestamp in seconds at which the process was started"
    ).unwrap();
    static ref METRIC_HOPR_LIB_VERSION:  hopr_metrics::MultiGauge =  hopr_metrics::MultiGauge::new(
        "hopr_lib_version",
        "Executed version of hopr-lib",
        &["version"]
    ).unwrap();
    static ref METRIC_HOPR_NODE_INFO:  hopr_metrics::MultiGauge =  hopr_metrics::MultiGauge::new(
        "hopr_node_addresses",
        "Node on-chain and off-chain addresses",
        &["peerid", "address", "safe_address", "module_address"]
    ).unwrap();
}

/// Intermediate state produced by the shared initialization sequence.
///
/// Holds all resources needed by the divergent `build_edge` / `build_full` finalization steps.
struct PreHopr<Chain, Graph, Net, NB, Ct> {
    chain_id: ChainKeypair,
    transport_id: OffchainKeypair,
    cfg: HoprLibConfig,
    state: Arc<AtomicHoprState>,
    transport_api: HoprTransport<Chain, Graph, Net>,
    chain_api: Chain,
    ticket_event_subscribers: (
        async_broadcast::Sender<TicketEvent>,
        async_broadcast::InactiveReceiver<TicketEvent>,
    ),
    processes: AbortableList<HoprLibProcess>,
    session_tx: futures::channel::mpsc::Sender<IncomingSession>,
    session_rx: futures::channel::mpsc::Receiver<IncomingSession>,
    // Moved out for transport.run()
    cover_traffic: Ct,
    network_builder: NB,
}

/// Abstract builder for the [`Hopr`] node object.
///
/// Collects components common to all node types (edge and relay/full) and provides
/// two distinct build paths:
///
/// - [`build_edge`](HoprBuilder::build_edge) — standalone ticket factory, no ticket management
/// - [`build_full`](HoprBuilder::build_full) — coupled ticket factory + ticket manager
///
/// # Type Parameters
///
/// - `Chain` — blockchain API ([`HoprChainApi`])
/// - `Graph` — network graph ([`HoprGraphApi`])
/// - `NB` — network builder factory ([`NetworkBuilder`])
/// - `Ct` — cover traffic / probing ([`ProbingTrafficGeneration`] + [`CoverTrafficGeneration`])
///
/// Ticket management, ticket factory, and session server are **not** builder generics —
/// they are parameters to the build methods or handled by the caller after building.
pub struct HoprBuilder<Chain = (), Graph = (), NB = (), Ct = ()> {
    chain: Option<Chain>,
    graph: Option<Graph>,
    network_builder: Option<NB>,
    cover_traffic: Option<Ct>,
    identity: Option<(ChainKeypair, OffchainKeypair)>,
    safe_and_module: Option<(Address, Address)>,
    cfg: HoprLibConfig,
}

// Manual Default — no trait bounds on generics
impl<Chain, Graph, NB, Ct> Default for HoprBuilder<Chain, Graph, NB, Ct> {
    fn default() -> Self {
        Self {
            chain: None,
            graph: None,
            network_builder: None,
            cover_traffic: None,
            identity: None,
            safe_and_module: None,
            cfg: Default::default(),
        }
    }
}

// === Configuration methods (no trait bounds needed) ===

impl<Chain, Graph, NB, Ct> HoprBuilder<Chain, Graph, NB, Ct> {
    /// Sets the chain API implementation.
    pub fn with_chain_api(mut self, chain: Chain) -> Self {
        self.chain = Some(chain);
        self
    }

    /// Sets the network graph.
    pub fn with_graph(mut self, graph: Graph) -> Self {
        self.graph = Some(graph);
        self
    }

    /// Sets the network builder (factory for P2P network).
    pub fn with_network_builder(mut self, builder: NB) -> Self {
        self.network_builder = Some(builder);
        self
    }

    /// Sets the cover traffic and probing provider.
    pub fn with_cover_traffic(mut self, ct: Ct) -> Self {
        self.cover_traffic = Some(ct);
        self
    }

    /// Sets the node's on-chain and off-chain identity.
    pub fn with_identity(mut self, chain_key: &ChainKeypair, offchain_key: &OffchainKeypair) -> Self {
        self.identity = Some((chain_key.clone(), offchain_key.clone()));
        self
    }

    /// Sets the node Safe and module addresses.
    pub fn with_safe_module(mut self, safe: &Address, module: &Address) -> Self {
        self.safe_and_module = Some((*safe, *module));
        self
    }

    /// Sets the [`HoprLibConfig`].
    pub fn with_config(mut self, cfg: HoprLibConfig) -> Self {
        self.cfg = cfg;
        self
    }
}

/// Result of a successful build, containing the node and a receiver for incoming sessions.
///
/// The caller is responsible for attaching a session server to the `session_rx`
/// if the node should process incoming sessions.
pub struct BuiltNode<Chain, Graph, Net, TMgr> {
    /// The built HOPR node.
    pub node: Hopr<Chain, Graph, Net, TMgr>,
    /// Receiver for incoming sessions from the transport layer.
    ///
    /// Attach a [`HoprSessionServer`](hopr_api::node::HoprSessionServer) to this
    /// receiver to process incoming sessions, or drop it to discard them.
    pub session_rx: futures::channel::mpsc::Receiver<IncomingSession>,
}

// === Build methods ===

impl<Chain, Graph, NB, Ct> HoprBuilder<Chain, Graph, NB, Ct>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
    Graph: HoprGraphApi<HoprNodeId = hopr_api::OffchainPublicKey> + Clone + Send + Sync + 'static,
    <Graph as hopr_api::graph::NetworkGraphTraverse>::Observed:
        hopr_api::graph::traits::EdgeObservableRead + Send + 'static,
    <Graph as hopr_api::graph::NetworkGraphWrite>::Observed: hopr_api::graph::traits::EdgeObservableWrite + Send,
    NB: NetworkBuilder + Send + Sync + 'static,
    <NB as NetworkBuilder>::Network:
        hopr_api::network::NetworkView + NetworkStreamControl + Send + Sync + Clone + 'static,
    Ct: ProbingTrafficGeneration + CoverTrafficGeneration + Send + Sync + 'static,
{
    /// Builds an edge (entry/exit) [`Hopr`] node.
    ///
    /// Edge nodes use a standalone ticket factory for outgoing tickets but do **not**
    /// process incoming winning tickets — the incoming ticket sink is drained.
    ///
    /// The caller must sync the `ticket_factory` with on-chain state before calling
    /// this method (e.g. via `sync_from_outgoing_channels`).
    pub async fn build_edge<TFact>(
        self,
        ticket_factory: TFact,
    ) -> Result<BuiltNode<Chain, Graph, <NB as NetworkBuilder>::Network, ()>, HoprLibError>
    where
        TFact: TicketFactory + Clone + Send + Sync + 'static,
    {
        let pre = self.pre_build().await?;
        let session_rx = pre.session_rx;

        // Edge nodes drain incoming tickets — no ticket processing.
        tracing::info!("starting transport for edge node");
        let (_, transport_processes) = pre
            .transport_api
            .run(
                pre.cover_traffic,
                pre.network_builder,
                futures::sink::drain(),
                ticket_factory,
                pre.session_tx,
            )
            .await?;

        let mut processes = pre.processes;
        processes.flat_map_extend_from(transport_processes, HoprLibProcess::Transport);

        let node = Hopr {
            chain_id: NodeOnchainIdentity {
                node_address: pre.chain_id.public().to_address(),
                safe_address: pre.cfg.safe_module.safe_address,
                module_address: pre.cfg.safe_module.module_address,
            },
            cfg: pre.cfg,
            state: pre.state.clone(),
            ticket_event_subscribers: pre.ticket_event_subscribers,
            transport_id: pre.transport_id,
            transport_api: pre.transport_api,
            chain_api: pre.chain_api,
            processes,
            ticket_manager: (),
        };

        node.state
            .store(HoprState::Running, std::sync::atomic::Ordering::Relaxed);
        tracing::info!(
            id = %node.transport_id.public().to_peerid_str(),
            version = constants::APP_VERSION,
            "EDGE NODE STARTED AND RUNNING"
        );

        Ok(BuiltNode { node, session_rx })
    }

    /// Builds a full (relay) [`Hopr`] node.
    ///
    /// Full nodes process incoming winning tickets via the `ticket_manager` and use a
    /// coupled ticket factory that accounts for unrealized balance.
    ///
    /// The caller must:
    /// 1. Create the ticket manager and factory together (e.g. via `HoprTicketManager::new_with_factory`)
    /// 2. Sync both with on-chain state before calling this method
    /// 3. Manage periodic outgoing index persistence externally
    pub async fn build_full<TMgr, TFact>(
        self,
        ticket_manager: TMgr,
        ticket_factory: TFact,
    ) -> Result<BuiltNode<Chain, Graph, <NB as NetworkBuilder>::Network, TMgr>, HoprLibError>
    where
        TMgr: TicketManagement + Clone + Send + Sync + 'static,
        TFact: TicketFactory + Clone + Send + Sync + 'static,
    {
        let pre = self.pre_build().await?;
        let session_rx = pre.session_rx;

        let mut processes = pre.processes;

        // === Ticket event processing ===
        // Incoming winning tickets are forwarded to the ticket manager.
        tracing::info!("starting ticket events processor");
        let (tickets_tx, tickets_rx) = channel(8192);
        let (tickets_rx_stream, tickets_handle) = futures::stream::abortable(tickets_rx);
        processes.insert(HoprLibProcess::TicketEvents, tickets_handle);
        let new_ticket_tx = pre.ticket_event_subscribers.0.clone();
        let tmgr_clone = ticket_manager.clone();
        spawn(
            tickets_rx_stream
                .for_each(move |event| {
                    if let TicketEvent::WinningTicket(ticket) = &event
                        && let Err(error) = tmgr_clone.insert_incoming_ticket(**ticket)
                    {
                        tracing::error!(%error, "failed to insert incoming ticket");
                    }
                    if let Err(error) = new_ticket_tx.try_broadcast(event) {
                        tracing::error!(%error, "failed to broadcast ticket event");
                    }
                    futures::future::ready(())
                })
                .inspect(|_| {
                    tracing::warn!(
                        task = %HoprLibProcess::TicketEvents,
                        "long-running background task finished"
                    )
                }),
        );

        // === Channel closure → ticket neglect ===
        // When an incoming channel is closed, neglect any remaining tickets.
        {
            let chain_for_neglect = pre.chain_api.clone();
            let tmgr_for_neglect = ticket_manager.clone();
            let events = pre.chain_api.subscribe().map_err(HoprLibError::chain)?;
            let (neglect_handle, neglect_reg) = hopr_async_runtime::AbortHandle::new_pair();
            spawn(
                futures::stream::Abortable::new(
                    events.filter_map(move |event| {
                        futures::future::ready(match event {
                            ChainEvent::ChannelClosed(ch) => Some(ch),
                            _ => None,
                        })
                    }),
                    neglect_reg,
                )
                .for_each(move |closed_channel| {
                    let chain = chain_for_neglect.clone();
                    let tmgr = tmgr_for_neglect.clone();
                    async move {
                        use hopr_api::types::internal::prelude::ChannelDirection;
                        match closed_channel.direction(chain.me()) {
                            Some(ChannelDirection::Incoming) => {
                                match tmgr.neglect_tickets(closed_channel.get_id(), None) {
                                    Ok(neglected) if !neglected.is_empty() => {
                                        tracing::warn!(
                                            num_neglected = neglected.len(),
                                            %closed_channel,
                                            "tickets on incoming closed channel were neglected"
                                        );
                                    }
                                    Ok(_) => {}
                                    Err(error) => {
                                        tracing::error!(
                                            %error, %closed_channel,
                                            "failed to neglect tickets on closed channel"
                                        );
                                    }
                                }
                            }
                            Some(ChannelDirection::Outgoing) => {
                                // Outgoing ticket index cleanup happens automatically on new epoch
                            }
                            _ => {} // Event for a channel that is not ours
                        }
                    }
                })
                .inspect(|_| {
                    tracing::warn!(
                        task = %HoprLibProcess::ChannelEvents,
                        "channel closure ticket neglect task finished"
                    )
                }),
            );
            processes.insert(HoprLibProcess::OutIndexSync, neglect_handle);
        }

        // === Start transport ===
        tracing::info!("starting transport for full node");
        let (_, transport_processes) = pre
            .transport_api
            .run(
                pre.cover_traffic,
                pre.network_builder,
                tickets_tx,
                ticket_factory,
                pre.session_tx,
            )
            .await?;
        processes.flat_map_extend_from(transport_processes, HoprLibProcess::Transport);

        let node = Hopr {
            chain_id: NodeOnchainIdentity {
                node_address: pre.chain_id.public().to_address(),
                safe_address: pre.cfg.safe_module.safe_address,
                module_address: pre.cfg.safe_module.module_address,
            },
            cfg: pre.cfg,
            state: pre.state.clone(),
            ticket_event_subscribers: pre.ticket_event_subscribers,
            transport_id: pre.transport_id,
            transport_api: pre.transport_api,
            chain_api: pre.chain_api,
            processes,
            ticket_manager,
        };

        node.state
            .store(HoprState::Running, std::sync::atomic::Ordering::Relaxed);

        tracing::info!(
            id = %node.transport_id.public().to_peerid_str(),
            version = constants::APP_VERSION,
            "FULL NODE STARTED AND RUNNING"
        );

        #[cfg(all(feature = "telemetry", not(test)))]
        METRIC_HOPR_NODE_INFO.set(
            &[
                &node.transport_id.public().to_peerid_str(),
                &node.chain_id.node_address.to_string(),
                &node.chain_id.safe_address.to_string(),
                &node.chain_id.module_address.to_string(),
            ],
            1.0,
        );

        Ok(BuiltNode { node, session_rx })
    }

    /// Shared initialization sequence for both edge and full node builds.
    async fn pre_build(
        mut self,
    ) -> Result<PreHopr<Chain, Graph, <NB as NetworkBuilder>::Network, NB, Ct>, HoprLibError> {
        self.cfg.validate()?;

        let chain_api = self
            .chain
            .clone()
            .ok_or(HoprLibError::BuilderError("missing chain API"))?;
        let graph = self.graph.clone().ok_or(HoprLibError::BuilderError("missing graph"))?;
        let (chain_id, transport_id) = self
            .identity
            .clone()
            .ok_or(HoprLibError::BuilderError("missing identity"))?;
        let cover_traffic = self
            .cover_traffic
            .take()
            .ok_or(HoprLibError::BuilderError("missing cover traffic"))?;
        let network_builder = self
            .network_builder
            .take()
            .ok_or(HoprLibError::BuilderError("missing network builder"))?;

        // Create transport
        let transport_api = HoprTransport::new(
            (&chain_id, &transport_id),
            chain_api.clone(),
            graph.clone(),
            vec![(&self.cfg.host).try_into().map_err(HoprLibError::TransportError)?],
            self.cfg.protocol.clone(),
        )
        .map_err(HoprLibError::TransportError)?;

        // Telemetry
        #[cfg(all(feature = "telemetry", not(test)))]
        {
            use hopr_api::types::primitive::traits::AsUnixTimestamp;
            METRIC_PROCESS_START_TIME.set(hopr_platform::time::current_time().as_unix_timestamp().as_secs_f64());
            METRIC_HOPR_LIB_VERSION.set(
                &[const_format::formatcp!("{}", constants::APP_VERSION)],
                const_format::formatcp!(
                    "{}.{}",
                    env!("CARGO_PKG_VERSION_MAJOR"),
                    env!("CARGO_PKG_VERSION_MINOR")
                )
                .parse()
                .unwrap_or(0.0),
            );
        }

        // Ticket event broadcast
        let (mut new_tickets_tx, new_tickets_rx) = async_broadcast::broadcast(2048);
        new_tickets_tx.set_await_active(false);
        new_tickets_tx.set_overflow(true);

        // === Fund check ===
        let me_onchain = chain_id.public().to_address();

        #[cfg(feature = "testing")]
        tracing::warn!("!! FOR TESTING ONLY !! Node is running with some safety checks disabled!");

        tracing::info!(
            address = %me_onchain,
            minimum_balance = %*SUGGESTED_NATIVE_BALANCE,
            "node is not started, please fund this node",
        );

        crate::helpers::wait_for_funds(
            *MIN_NATIVE_BALANCE,
            *SUGGESTED_NATIVE_BALANCE,
            Duration::from_secs(200),
            me_onchain,
            &chain_api,
        )
        .await?;

        tracing::info!("starting HOPR node...");
        let balance: hopr_api::types::primitive::prelude::XDaiBalance =
            chain_api.balance(me_onchain).await.map_err(HoprLibError::chain)?;
        let minimum_balance = *constants::MIN_NATIVE_BALANCE;

        tracing::info!(address = %me_onchain, %balance, %minimum_balance, "node information");

        if balance.le(&minimum_balance) {
            return Err(HoprLibError::GeneralError(
                "cannot start the node without a sufficiently funded wallet".into(),
            ));
        }

        // === Ticket price / win prob validation ===
        let network_min_ticket_price = chain_api.minimum_ticket_price().await.map_err(HoprLibError::chain)?;
        let configured_ticket_price = self.cfg.protocol.packet.codec.outgoing_ticket_price;
        if configured_ticket_price.is_some_and(|c| c < network_min_ticket_price) {
            return Err(HoprLibError::GeneralError(format!(
                "configured outgoing ticket price < network minimum: {configured_ticket_price:?} < \
                 {network_min_ticket_price}"
            )));
        }

        let network_min_win_prob = chain_api
            .minimum_incoming_ticket_win_prob()
            .await
            .map_err(HoprLibError::chain)?;
        let configured_win_prob = self.cfg.protocol.packet.codec.outgoing_win_prob;
        if !std::env::var("HOPR_TEST_DISABLE_CHECKS").is_ok_and(|v| v.to_lowercase() == "true")
            && configured_win_prob.is_some_and(|c| c.approx_cmp(&network_min_win_prob).is_lt())
        {
            return Err(HoprLibError::GeneralError(format!(
                "configured outgoing win probability < network minimum: {configured_win_prob:?} < \
                 {network_min_win_prob}"
            )));
        }

        tracing::info!(
            peer_id = %transport_id.public().to_peerid_str(),
            address = %me_onchain,
            version = constants::APP_VERSION,
            "Node information"
        );

        // === Safe registration ===
        let safe_addr = self.cfg.safe_module.safe_address;
        if me_onchain == safe_addr {
            return Err(HoprLibError::GeneralError(
                "cannot use self as staking safe address".into(),
            ));
        }

        tracing::info!(%safe_addr, "registering safe with this node");
        match chain_api.register_safe(&safe_addr).await {
            Ok(awaiter) => {
                awaiter.await.map_err(|error| {
                    tracing::error!(%safe_addr, %error, "safe registration failed");
                    HoprLibError::chain(error)
                })?;
                tracing::info!(%safe_addr, "safe successfully registered with this node");
            }
            Err(SafeRegistrationError::AlreadyRegistered(registered_safe)) if registered_safe == safe_addr => {
                tracing::info!(%safe_addr, "this safe is already registered with this node");
            }
            Err(SafeRegistrationError::AlreadyRegistered(registered_safe)) if registered_safe != safe_addr => {
                tracing::error!(%safe_addr, %registered_safe, "node registered with different safe");
                return Err(HoprLibError::GeneralError("node registered with different safe".into()));
            }
            Err(error) => {
                tracing::error!(%safe_addr, %error, "safe registration failed");
                return Err(HoprLibError::chain(error));
            }
        }

        // === Announce ===
        let multiaddresses_to_announce = if self.cfg.publish {
            transport_api.announceable_multiaddresses()
        } else {
            Vec::with_capacity(0)
        };

        multiaddresses_to_announce
            .iter()
            .filter(|a| !is_public_address(a))
            .for_each(|multi_addr| tracing::warn!(?multi_addr, "announcing private multiaddress"));

        let chain_api_clone = chain_api.clone();
        let me_offchain = *transport_id.public();
        let node_ready = spawn(async move {
            chain_api_clone
                .await_key_binding(&me_offchain, NODE_READY_TIMEOUT)
                .await
        });

        tracing::info!(?multiaddresses_to_announce, "announcing node on chain");
        match chain_api.announce(&multiaddresses_to_announce, &transport_id).await {
            Ok(awaiter) => {
                awaiter.await.map_err(|error| {
                    tracing::error!(?multiaddresses_to_announce, %error, "node announcement failed");
                    HoprLibError::chain(error)
                })?;
                tracing::info!(?multiaddresses_to_announce, "node announced successfully");
            }
            Err(AnnouncementError::AlreadyAnnounced) => {
                tracing::info!("node already announced on chain");
            }
            Err(error) => {
                tracing::error!(%error, "failed to transmit node announcement");
                return Err(HoprLibError::chain(error));
            }
        }

        // === Key binding ===
        let this_node_account = node_ready
            .await
            .map_err(HoprLibError::other)?
            .map_err(HoprLibError::chain)?;
        if this_node_account.chain_addr != me_onchain || this_node_account.safe_address.is_none_or(|a| a != safe_addr) {
            tracing::error!(%this_node_account, "account key-binding mismatch");
            return Err(HoprLibError::GeneralError("account key-binding mismatch".into()));
        }

        tracing::info!(%this_node_account, "node account is ready");

        // === Session channel ===
        let incoming_session_capacity = std::env::var("HOPR_INTERNAL_SESSION_INCOMING_CAPACITY")
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .filter(|&c| c > 0)
            .unwrap_or(256);

        let processes = AbortableList::<HoprLibProcess>::default();
        let (session_tx, session_rx) = channel::<IncomingSession>(incoming_session_capacity);

        // === Chain → Graph event wiring ===
        // Subscribe to chain events and update the graph accordingly.
        // This runs as a background task for the lifetime of the node.
        let processes = {
            let chain_events = chain_api
                .subscribe_with_state_sync([StateSyncOptions::PublicAccounts, StateSyncOptions::OpenedChannels])
                .map_err(HoprLibError::chain)?;

            let graph_updater = graph.clone();
            let chain_reader = chain_api.clone();

            let ticket_price = Arc::new(parking_lot::RwLock::new(
                chain_reader.minimum_ticket_price().await.unwrap_or_default(),
            ));
            let win_probability = Arc::new(parking_lot::RwLock::new(
                chain_reader
                    .minimum_incoming_ticket_win_prob()
                    .await
                    .unwrap_or_default(),
            ));

            let proc = async move {
                chain_events
                    .for_each(|chain_event| {
                        let chain_reader = chain_reader.clone();
                        let graph_updater = graph_updater.clone();
                        let ticket_price = ticket_price.clone();
                        let win_probability = win_probability.clone();

                        async move {
                            match chain_event {
                                ChainEvent::Announcement(account) => {
                                    tracing::debug!(
                                        account = %account.public_key,
                                        "recording graph node for announced account"
                                    );
                                    graph_updater.record_node(account.public_key);
                                }
                                ChainEvent::ChannelOpened(channel)
                                | ChainEvent::ChannelClosed(channel)
                                | ChainEvent::ChannelBalanceIncreased(channel, _)
                                | ChainEvent::ChannelBalanceDecreased(channel, _) => {
                                    let keys = hopr_async_runtime::prelude::spawn_blocking(move || {
                                        chain_reader
                                            .chain_key_to_packet_key(&channel.source)
                                            .and_then(|src| {
                                                Ok(src.zip(chain_reader.chain_key_to_packet_key(&channel.destination)?))
                                            })
                                            .map_err(anyhow::Error::from)
                                    })
                                    .await
                                    .map_err(anyhow::Error::from)
                                    .and_then(identity);

                                    match keys {
                                        Ok(Some((from, to))) => {
                                            let capacity = if matches!(
                                                channel.status,
                                                hopr_api::types::internal::prelude::ChannelStatus::Closed
                                            ) {
                                                None
                                            } else if let Ok(ticket_value) =
                                                ticket_price.read().div_f64(win_probability.read().as_f64())
                                            {
                                                Some(
                                                    channel
                                                        .balance
                                                        .amount()
                                                        .checked_div(ticket_value.amount())
                                                        .map(|v| v.low_u128())
                                                        .unwrap_or(u128::MAX),
                                                )
                                            } else {
                                                None
                                            };

                                            tracing::debug!(
                                                %channel, ?capacity,
                                                "recording graph edge for channel capacity"
                                            );
                                            graph_updater.record_edge(hopr_api::graph::MeasurableEdge::<
                                                hopr_transport::NeighborTelemetry,
                                                hopr_transport::PathTelemetry,
                                            >::Capacity(
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
                                ChainEvent::ChannelClosureInitiated(_) => {}
                                ChainEvent::WinningProbabilityIncreased(prob)
                                | ChainEvent::WinningProbabilityDecreased(prob) => {
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
                    })
                    .await;
            }
            .inspect(|_| {
                tracing::warn!(
                    task = "chain-to-graph event wiring",
                    "long-running background task finished"
                )
            });
            let mut processes = processes;
            processes.insert(
                HoprLibProcess::ChannelEvents,
                hopr_async_runtime::spawn_as_abortable!(proc),
            );
            processes
        };

        Ok(PreHopr {
            chain_id,
            transport_id,
            cfg: self.cfg,
            state: Arc::new(AtomicHoprState::new(HoprState::Uninitialized)),
            transport_api,
            chain_api,
            ticket_event_subscribers: (new_tickets_tx, new_tickets_rx.deactivate()),
            processes,
            session_tx,
            session_rx,
            cover_traffic,
            network_builder,
        })
    }
}
