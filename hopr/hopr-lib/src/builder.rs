use std::{sync::Arc, time::Duration};

use futures::{FutureExt, SinkExt, StreamExt, TryFutureExt, channel::mpsc::channel};
use hopr_api::{
    chain::{AnnouncementError, HoprChainApi, SafeRegistrationError},
    ct::{CoverTrafficGeneration, ProbingTrafficGeneration},
    graph::HoprGraphApi,
    network::{NetworkBuilder, NetworkStreamControl},
    node::{AtomicHoprState, HoprState, NodeOnchainIdentity, TicketEvent},
    tickets::{TicketFactory, TicketManagement},
    types::primitive::prelude::Address,
};
use hopr_async_runtime::AbortableList;
use hopr_network_types::addr::is_public_address;
use hopr_transport::HoprTransport;
use tokio::spawn;
use validator::Validate;

use crate::{
    Hopr, HoprLibError, HoprLibProcess, IncomingSession, MIN_NATIVE_BALANCE, NODE_READY_TIMEOUT,
    SUGGESTED_NATIVE_BALANCE, config::HoprLibConfig, constants, traits::HoprSessionServer,
};

// Re-exports for downstream convenience
pub use hopr_api::types::crypto::keypairs::Keypair;
pub use hopr_api::types::crypto::prelude::{ChainKeypair, OffchainKeypair};

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

/// Abstract builder for the [`Hopr`] node object.
///
/// Uses the builder pattern with `with_*` methods to accept trait-bounded components.
///
/// # Type Parameters
///
/// - `Chain` — blockchain API ([`HoprChainApi`])
/// - `Graph` — network graph ([`HoprGraphApi`])
/// - `NB` — network builder factory ([`NetworkBuilder`])
/// - `TMgr` — ticket management ([`TicketManagement`]), `()` for edge nodes
/// - `Srv` — session server handler ([`HoprSessionServer`])
/// - `Ct` — cover traffic / probing ([`ProbingTrafficGeneration`] + [`CoverTrafficGeneration`])
/// - `TFact` — ticket factory ([`TicketFactory`])
pub struct HoprBuilder<Chain = (), Graph = (), NB = (), TMgr = (), Srv = (), Ct = (), TFact = ()> {
    chain: Option<Chain>,
    graph: Option<Graph>,
    network_builder: Option<NB>,
    cover_traffic: Option<Ct>,
    ticket_factory: Option<TFact>,
    ticket_manager: Option<TMgr>,
    session_server: Option<Srv>,
    identity: Option<(ChainKeypair, OffchainKeypair)>,
    safe_and_module: Option<(Address, Address)>,
    cfg: HoprLibConfig,
}

// Manual Default — no trait bounds on generics
impl<Chain, Graph, NB, TMgr, Srv, Ct, TFact> Default
    for HoprBuilder<Chain, Graph, NB, TMgr, Srv, Ct, TFact>
{
    fn default() -> Self {
        Self {
            chain: None,
            graph: None,
            network_builder: None,
            cover_traffic: None,
            ticket_factory: None,
            ticket_manager: None,
            session_server: None,
            identity: None,
            safe_and_module: None,
            cfg: Default::default(),
        }
    }
}

// === Configuration methods (no trait bounds needed) ===

impl<Chain, Graph, NB, TMgr, Srv, Ct, TFact>
    HoprBuilder<Chain, Graph, NB, TMgr, Srv, Ct, TFact>
{
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

    /// Sets the ticket factory for outgoing ticket creation.
    pub fn with_ticket_factory(mut self, factory: TFact) -> Self {
        self.ticket_factory = Some(factory);
        self
    }

    /// Sets the ticket management for incoming ticket processing (relay nodes).
    ///
    /// If not set, the node operates in edge mode (no ticket processing).
    pub fn with_ticket_management(mut self, mgr: TMgr) -> Self {
        self.ticket_manager = Some(mgr);
        self
    }

    /// Sets the session server handler.
    pub fn with_session_server(mut self, srv: Srv) -> Self {
        self.session_server = Some(srv);
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

// === Build method ===

impl<Chain, Graph, NB, TMgr, Srv, Ct, TFact>
    HoprBuilder<Chain, Graph, NB, TMgr, Srv, Ct, TFact>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
    Graph: HoprGraphApi<HoprNodeId = hopr_api::OffchainPublicKey> + Clone + Send + Sync + 'static,
    <Graph as hopr_api::graph::NetworkGraphTraverse>::Observed:
        hopr_api::graph::traits::EdgeObservableRead + Send + 'static,
    <Graph as hopr_api::graph::NetworkGraphWrite>::Observed:
        hopr_api::graph::traits::EdgeObservableWrite + Send,
    NB: NetworkBuilder + Send + Sync + 'static,
    <NB as NetworkBuilder>::Network:
        hopr_api::network::NetworkView + NetworkStreamControl + Send + Sync + Clone + 'static,
    TMgr: TicketManagement + Default + Clone + Send + Sync + 'static,
    Srv: HoprSessionServer + Clone + Send + 'static,
    Ct: ProbingTrafficGeneration + CoverTrafficGeneration + Send + Sync + 'static,
    TFact: TicketFactory + Clone + Send + Sync + 'static,
{
    /// Builds and starts the [`Hopr`] node.
    ///
    /// Performs the full initialization sequence:
    /// 1. Validate configuration
    /// 2. Create transport layer
    /// 3. Wait for initial funding
    /// 4. Validate ticket parameters
    /// 5. Register Safe on-chain
    /// 6. Announce multiaddresses
    /// 7. Await key binding confirmation
    /// 8. Start transport and background processes
    pub async fn build(
        mut self,
    ) -> Result<Hopr<Chain, Graph, <NB as NetworkBuilder>::Network, TMgr>, HoprLibError> {
        self.cfg.validate()?;

        let chain_api = self
            .chain
            .clone()
            .ok_or(HoprLibError::BuilderError("missing chain API"))?;
        let graph = self
            .graph
            .clone()
            .ok_or(HoprLibError::BuilderError("missing graph"))?;
        let (chain_id, transport_id) = self
            .identity
            .clone()
            .ok_or(HoprLibError::BuilderError("missing identity"))?;
        let cover_traffic = self
            .cover_traffic
            .take()
            .ok_or(HoprLibError::BuilderError("missing cover traffic"))?;
        let ticket_factory = self
            .ticket_factory
            .take()
            .ok_or(HoprLibError::BuilderError("missing ticket factory"))?;
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
            use hopr_api::types::internal::prelude::AsUnixTimestamp;
            METRIC_PROCESS_START_TIME
                .set(hopr_platform::time::current_time().as_unix_timestamp().as_secs_f64());
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
        let network_min_ticket_price =
            chain_api.minimum_ticket_price().await.map_err(HoprLibError::chain)?;
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
            && configured_win_prob.is_some_and(|c| {
                use hopr_api::types::primitive::prelude::UnitaryFloatOps;
                c.approx_cmp(&network_min_win_prob).is_lt()
            })
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
            Err(SafeRegistrationError::AlreadyRegistered(registered_safe))
                if registered_safe == safe_addr =>
            {
                tracing::info!(%safe_addr, "this safe is already registered with this node");
            }
            Err(SafeRegistrationError::AlreadyRegistered(registered_safe))
                if registered_safe != safe_addr =>
            {
                tracing::error!(%safe_addr, %registered_safe, "node registered with different safe");
                return Err(HoprLibError::GeneralError(
                    "node registered with different safe".into(),
                ));
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
        match chain_api
            .announce(&multiaddresses_to_announce, &transport_id)
            .await
        {
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
        if this_node_account.chain_addr != me_onchain
            || this_node_account
                .safe_address
                .is_none_or(|a| a != safe_addr)
        {
            tracing::error!(%this_node_account, "account key-binding mismatch");
            return Err(HoprLibError::GeneralError(
                "account key-binding mismatch".into(),
            ));
        }

        tracing::info!(%this_node_account, "node account is ready");

        // === Session infrastructure ===
        tracing::info!("initializing session infrastructure");
        let incoming_session_capacity = std::env::var("HOPR_INTERNAL_SESSION_INCOMING_CAPACITY")
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .filter(|&c| c > 0)
            .unwrap_or(256);

        #[allow(unused_mut)]
        let mut processes = AbortableList::<HoprLibProcess>::default();

        let (session_tx, _session_rx) = channel::<IncomingSession>(incoming_session_capacity);

        if let Some(serve_handler) = self.session_server.take() {
            tracing::debug!(capacity = incoming_session_capacity, "creating session server");
            processes.insert(
                HoprLibProcess::SessionServer,
                hopr_async_runtime::spawn_as_abortable!(
                    _session_rx
                        .for_each_concurrent(None, move |session| {
                            let serve_handler = serve_handler.clone();
                            async move {
                                let session_id = *session.session.id();
                                match serve_handler.process(session).await {
                                    Ok(_) => {
                                        tracing::debug!(?session_id, "session processed successfully")
                                    }
                                    Err(error) => {
                                        tracing::error!(?session_id, %error, "session processing failed")
                                    }
                                }
                            }
                        })
                        .inspect(|_| tracing::warn!(
                            task = %HoprLibProcess::SessionServer,
                            "long-running background task finished"
                        ))
                ),
            );
        }

        // === Ticket event handling ===
        let ticket_manager = self.ticket_manager.take().unwrap_or_default();

        let (tickets_tx, tickets_rx) = channel(8192);
        let (tickets_rx_stream, tickets_handle) = futures::stream::abortable(tickets_rx);
        processes.insert(HoprLibProcess::TicketEvents, tickets_handle);
        let new_ticket_tx = new_tickets_tx.clone();
        spawn(
            tickets_rx_stream
                .for_each(move |event| {
                    // TODO: Ticket insertion needs `insert_incoming_ticket` on TicketManagement trait.
                    // For now, winning tickets are broadcast to subscribers only.
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

        // === Start transport ===
        tracing::info!("starting transport");
        let (_, transport_processes) = transport_api
            .run(cover_traffic, network_builder, tickets_tx, ticket_factory, session_tx)
            .await?;
        processes.flat_map_extend_from(transport_processes, HoprLibProcess::Transport);

        // === Assemble Hopr object ===
        let mut hopr = Hopr {
            chain_id: NodeOnchainIdentity {
                node_address: chain_id.public().to_address(),
                safe_address: self.cfg.safe_module.safe_address,
                module_address: self.cfg.safe_module.module_address,
            },
            cfg: self.cfg.clone(),
            state: Arc::new(AtomicHoprState::new(HoprState::Uninitialized)),
            ticket_event_subscribers: (new_tickets_tx, new_tickets_rx.deactivate()),
            transport_id,
            transport_api,
            chain_api,
            processes,
            ticket_manager,
        };

        hopr.state
            .store(HoprState::Running, std::sync::atomic::Ordering::Relaxed);

        tracing::info!(
            id = %hopr.transport_id.public().to_peerid_str(),
            version = constants::APP_VERSION,
            "NODE STARTED AND RUNNING"
        );

        #[cfg(all(feature = "telemetry", not(test)))]
        METRIC_HOPR_NODE_INFO.set(
            &[
                &hopr.transport_id.public().to_peerid_str(),
                &hopr.chain_id.node_address.to_string(),
                &hopr.chain_id.safe_address.to_string(),
                &hopr.chain_id.module_address.to_string(),
            ],
            1.0,
        );

        Ok(hopr)
    }
}
