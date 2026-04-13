use std::{convert::identity, path::PathBuf, sync::Arc, time::Duration};

use futures::{FutureExt, SinkExt, StreamExt, TryFutureExt, channel::mpsc::channel};
use futures_concurrency::stream::StreamExt as ConcurrencyStreamExt;
use hopr_api::{
    chain::{
        AccountSelector, AnnouncementError, ChannelSelector, HoprChainApi, SafeRegistrationError, StateSyncOptions,
    },
    graph::{EdgeCapacityUpdate, NetworkGraphUpdate},
    network::NetworkEvent,
    node::{AtomicHoprState, HoprState, NodeOnchainIdentity, TicketEvent},
    tickets::TicketManagement,
};
use hopr_async_runtime::AbortableList;
use hopr_ct_full_network::ProberConfig;
use hopr_network_graph::{ChannelGraph, SharedChannelGraph};
use hopr_network_types::addr::is_public_address;
use hopr_ticket_manager::{HoprTicketFactory, HoprTicketManager, RedbStore, RedbTicketQueue};
use hopr_transport::HoprTransport;
use hopr_transport_p2p::{HoprLibp2pNetworkBuilder, HoprNetwork, PeerDiscovery};
use tokio::spawn;
use validator::Validate;

use crate::{
    Address, ChainKeypair, ChannelDirection, ChannelStatus, Hopr, HoprLibError, HoprLibProcess, IncomingSession,
    Keypair, MIN_NATIVE_BALANCE, NODE_READY_TIMEOUT, NeighborTelemetry, OffchainKeypair, PathTelemetry, PeerId,
    SUGGESTED_NATIVE_BALANCE, UnitaryFloatOps, XDaiBalance, config::HoprLibConfig, constants,
    exports::types::chain::chain_events::ChainEvent, helpers, traits::HoprSessionServer,
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

pub type SharedTicketManager = Arc<HoprTicketManager<RedbStore, RedbTicketQueue>>;

pub struct HoprBuilder<Chain, Srv> {
    chain: Option<Chain>,
    identity: Option<(ChainKeypair, OffchainKeypair)>,
    safe_and_module: Option<(Address, Address)>,
    _session_server: Option<Srv>,
    cfg: HoprLibConfig,
    prober_cfg: ProberConfig,
    ticket_index_db_path: Option<PathBuf>,
    // Set during build, not by the user
    network_builder: Option<HoprLibp2pNetworkBuilder>,
    graph: Option<SharedChannelGraph>,
    ct: Option<hopr_ct_full_network::FullNetworkDiscovery<SharedChannelGraph>>,
    session_tx: Option<futures::channel::mpsc::Sender<IncomingSession>>,
}

// Needs to be manually implemented so we do not need to impose Default requirements
// on the generic parameters.
impl<Chain, Srv> Default for HoprBuilder<Chain, Srv> {
    fn default() -> Self {
        Self {
            chain: None,
            identity: None,
            safe_and_module: None,
            _session_server: None,
            cfg: Default::default(),
            prober_cfg: Default::default(),
            ticket_index_db_path: None,
            network_builder: None,
            graph: None,
            ct: None,
            session_tx: None,
        }
    }
}

impl<Chain, Srv> HoprBuilder<Chain, Srv> {
    /// Set the `Chain` object.
    ///
    /// This parameter is required.
    pub fn chain(mut self, chain: Chain) -> Self {
        self.chain = Some(chain);
        self
    }

    /// Set the node's on-chain and off-chain identity.
    ///
    /// This parameter is required.
    pub fn identity<I: for<'a> Into<(&'a ChainKeypair, &'a OffchainKeypair)>>(mut self, identity: I) -> Self {
        let (ckp, okp) = identity.into();
        self.identity = Some((ckp.clone(), okp.clone()));
        self
    }

    /// Sets the node Safe and module addresses.
    ///
    /// This parameter is required.
    pub fn safe_and_module(mut self, safe: &Address, module: &Address) -> Self {
        self.safe_and_module = Some((*safe, *module));
        self
    }

    /// Path where to store the outgoing ticket indices.
    ///
    /// This parameter is optional. If not set, a temporary file is used and discarded
    /// after the node is stopped.
    pub fn with_index_db_path(mut self, path_buf: PathBuf) -> Self {
        // This is currently same as ticket db location
        self.ticket_index_db_path = Some(path_buf);
        self
    }

    /// Path where to store the incoming winning tickets.
    ///
    /// This parameter is optional. If not set, a temporary file is used and discarded
    /// after the node is stopped.
    pub fn with_ticket_db_path(mut self, path_buf: PathBuf) -> Self {
        // This is currently same as ticket db location
        self.ticket_index_db_path = Some(path_buf);
        self
    }

    /// Sets the [`HoprLibConfig`].
    ///
    /// This parameter is optional, uses default if not set.
    pub fn with_config(mut self, cfg: HoprLibConfig) -> Self {
        self.cfg = cfg;
        self
    }

    /// Sets the [`ProberConfig`].
    ///
    /// This parameter is optional, uses default if not set.
    pub fn with_ct_prober_config(mut self, cfg: ProberConfig) -> Self {
        self.prober_cfg = cfg;
        self
    }

    /// Sets the Session Server handler.
    ///
    /// This parameter is required when built with the `session-server` feature.
    #[cfg(feature = "session-server")]
    pub fn session_server(mut self, session_server: Srv) -> Self {
        self._session_server = Some(session_server);
        self
    }
}

impl<Chain, Srv> HoprBuilder<Chain, Srv>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
    Srv: HoprSessionServer + Clone + Send + 'static,
{
    async fn build_graph_and_network(&mut self) -> Result<(), HoprLibError> {
        let chain_connector = self.chain.as_ref().ok_or(HoprLibError::BuilderError("missing chain"))?;
        let packet_key = self
            .identity
            .as_ref()
            .map(|(_, p)| p.clone())
            .ok_or(HoprLibError::BuilderError("missing identity"))?;

        // Calculate the minimum capacity based on accounts (each account can generate 2 messages),
        // plus 100 as an additional buffer
        let minimum_capacity = chain_connector
            .count_accounts(AccountSelector {
                public_only: true,
                ..Default::default()
            })
            .await
            .map_err(crate::HoprLibError::chain)?
            .saturating_mul(2)
            .saturating_add(100);

        let chain_discovery_events_capacity = std::env::var("HOPR_INTERNAL_CHAIN_DISCOVERY_CHANNEL_CAPACITY")
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .filter(|&c| c > 0)
            .unwrap_or(2048)
            .max(minimum_capacity);

        tracing::debug!(
            capacity = chain_discovery_events_capacity,
            minimum_required = minimum_capacity,
            "creating chain discovery events channel"
        );
        let (indexer_peer_update_tx, indexer_peer_update_rx) =
            futures::channel::mpsc::channel::<PeerDiscovery>(chain_discovery_events_capacity);

        // create network
        let network_builder = HoprLibp2pNetworkBuilder::new(indexer_peer_update_rx);
        // create graph
        let graph = std::sync::Arc::new(ChannelGraph::new(*packet_key.public()));

        // END = implementation definitions

        // START = process chain and network events into graph updates
        let chain_events = chain_connector
            .subscribe_with_state_sync([StateSyncOptions::PublicAccounts, StateSyncOptions::OpenedChannels])
            .map_err(HoprLibError::chain)?;
        let network_events = network_builder.subscribe_network_events();
        let graph_updater = graph.clone();
        let chain_reader = chain_connector.clone();
        let indexer_peer_update_tx = indexer_peer_update_tx.clone();

        let proc =
            async move {
                enum Event {
                    Chain(ChainEvent),
                    Network(NetworkEvent),
                }

                let ticket_price = Arc::new(parking_lot::RwLock::new(chain_reader.minimum_ticket_price().await.unwrap_or_default()));
                let win_probability = Arc::new(parking_lot::RwLock::new(chain_reader.minimum_incoming_ticket_win_prob().await.unwrap_or_default()));

                network_events
                    .map(Event::Network)
                    .merge(chain_events.map(Event::Chain))
                    .for_each(|event| {
                        let mut indexer_peer_update_tx = indexer_peer_update_tx.clone();
                        let chain_reader = chain_reader.clone();
                        let graph_updater = graph_updater.clone();
                        let ticket_price = ticket_price.clone();
                        let win_probability = win_probability.clone();

                        async move {
                            match event {
                                Event::Chain(chain_event) => {
                                    match chain_event {
                                        ChainEvent::Announcement(account) =>{
                                            tracing::debug!(account = %account.public_key, "recording graph update for announced account");
                                            graph_updater.record_node(account.public_key);
                                            let peer_id: PeerId = account.public_key.into();
                                            if let Err(error) = indexer_peer_update_tx.send(PeerDiscovery::Announce(peer_id, account.get_multiaddrs().to_vec())).await {
                                                tracing::error!(peer = %peer_id, %error, reason = "failed to propagate the record", "ignoring announced peer")
                                            }
                                        },
                                        ChainEvent::ChannelOpened(channel) |
                                        ChainEvent::ChannelClosed(channel) |
                                        ChainEvent::ChannelBalanceIncreased(channel, _) |
                                        ChainEvent::ChannelBalanceDecreased(channel, _) => {
                                            let keys = hopr_async_runtime::prelude::spawn_blocking(move || {
                                                chain_reader
                                                    .chain_key_to_packet_key(&channel.source)
                                                    .and_then(|src| Ok(src.zip(chain_reader.chain_key_to_packet_key(&channel.destination)?)))
                                                    .map_err(anyhow::Error::from)
                                            }).await
                                                .map_err(anyhow::Error::from)
                                                .and_then(identity);


                                            match keys {
                                                Ok(Some((from, to))) => {
                                                    let capacity =  if matches!(channel.status, ChannelStatus::Closed) {
                                                        None
                                                    } else if let Ok(ticket_value) = ticket_price.read().div_f64(win_probability.read().as_f64()) {
                                                        Some(
                                                            channel.balance
                                                                .amount()
                                                                .checked_div(ticket_value.amount())
                                                                .map(|v| v.low_u128())
                                                                .unwrap_or(u128::MAX)
                                                        )
                                                    } else {
                                                        None
                                                    };

                                                    tracing::debug!(%channel, ?capacity, "recording graph update for channel capacity change");
                                                    graph_updater.record_edge(crate::api::graph::MeasurableEdge::<NeighborTelemetry, PathTelemetry>::Capacity(Box::new(EdgeCapacityUpdate{
                                                        capacity,
                                                        src: from,
                                                        dest: to
                                                    })));
                                                },
                                                Ok(None) => {
                                                    tracing::error!(%channel, "could not find packet keys for the channel endpoints");
                                                },
                                                Err(error) => {
                                                    tracing::error!(%error, %channel, "failed to convert chain keys to packet keys for graph update");
                                                }
                                            }
                                        },
                                        ChainEvent::ChannelClosureInitiated(_channel) => {},
                                        ChainEvent::WinningProbabilityIncreased(probability) |
                                        ChainEvent::WinningProbabilityDecreased(probability) => {
                                            tracing::debug!(%probability, "recording winning probability change");
                                            *win_probability.write() = probability;
                                        }
                                        ChainEvent::TicketPriceChanged(price) => {
                                            tracing::debug!(%price, "recording ticket price change");
                                            *ticket_price.write() = price;
                                        },
                                        _ => {}
                                    }
                                }
                                Event::Network(network_event) => {
                                    match network_event {
                                        NetworkEvent::PeerConnected(peer_id) =>
                                            if let Ok(opk) = crate::peer_id_to_public_key(&peer_id) {
                                                graph_updater.record_edge(crate::api::graph::MeasurableEdge::<NeighborTelemetry, PathTelemetry>::ConnectionStatus {
                                                    peer: opk,
                                                    connected: true
                                                });
                                            } else {
                                                tracing::error!(%peer_id, "failed to convert peer ID to public key for graph update");
                                            },
                                        NetworkEvent::PeerDisconnected(peer_id) =>
                                            if let Ok(opk) = crate::peer_id_to_public_key(&peer_id) {
                                                graph_updater.record_edge(crate::api::graph::MeasurableEdge::<NeighborTelemetry, PathTelemetry>::ConnectionStatus {
                                                    peer: opk,
                                                    connected: false
                                                });
                                            } else {
                                                tracing::error!(%peer_id, "failed to convert peer ID to public key for graph update");
                                            },
                                    };
                                }
                            }
                        }
                    })
                    .await;
            }
                .inspect(|_| tracing::warn!(task = "Interconnecting chain, graph and network", "long-running background task finished"));
        let _jh = tokio::spawn(proc);
        // END = process chain and network events into graph updates

        self.graph = Some(graph.clone());
        self.network_builder = Some(network_builder);
        self.ct = Some(hopr_ct_full_network::FullNetworkDiscovery::new(
            *packet_key.public(),
            self.prober_cfg,
            graph,
        ));

        Ok(())
    }

    async fn pre_build<TMgr>(
        &mut self,
        ticket_manager: TMgr,
    ) -> Result<Hopr<Chain, SharedChannelGraph, HoprNetwork, TMgr>, HoprLibError> {
        self.cfg.validate()?;

        let hopr_chain_api = self
            .chain
            .clone()
            .ok_or(HoprLibError::BuilderError("missing chain object"))?;
        let (chain_id, transport_id) = self
            .identity
            .clone()
            .ok_or(HoprLibError::BuilderError("missing identity"))?;

        self.build_graph_and_network().await?;

        let hopr_transport_api = HoprTransport::new(
            (&chain_id, &transport_id),
            hopr_chain_api.clone(),
            self.graph.clone().ok_or(HoprLibError::BuilderError("missing graph"))?,
            vec![(&self.cfg.host).try_into().map_err(HoprLibError::TransportError)?],
            self.cfg.protocol.clone(),
        )
        .map_err(HoprLibError::TransportError)?;

        #[cfg(all(feature = "telemetry", not(test)))]
        {
            use crate::AsUnixTimestamp;
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

        let (mut new_tickets_tx, new_tickets_rx) = async_broadcast::broadcast(2048);
        new_tickets_tx.set_await_active(false);
        new_tickets_tx.set_overflow(true);

        let me_onchain = chain_id.public().to_address();

        #[cfg(feature = "testing")]
        tracing::warn!("!! FOR TESTING ONLY !! Node is running with some safety checks disabled!");

        tracing::info!(
            address = %me_onchain, minimum_balance = %*SUGGESTED_NATIVE_BALANCE,
            "node is not started, please fund this node",
        );

        helpers::wait_for_funds(
            *MIN_NATIVE_BALANCE,
            *SUGGESTED_NATIVE_BALANCE,
            Duration::from_secs(200),
            me_onchain,
            &hopr_chain_api,
        )
        .await?;

        #[allow(unused_mut)]
        let mut processes = AbortableList::<HoprLibProcess>::default();

        tracing::info!("starting HOPR node...");

        let balance: XDaiBalance = hopr_chain_api.balance(me_onchain).await.map_err(HoprLibError::chain)?;
        let minimum_balance = *constants::MIN_NATIVE_BALANCE;

        tracing::info!(
            address = %me_onchain,
            %balance,
            %minimum_balance,
            "node information"
        );

        if balance.le(&minimum_balance) {
            return Err(HoprLibError::GeneralError(
                "cannot start the node without a sufficiently funded wallet".into(),
            ));
        }

        // Once we are able to query the chain,
        // check if the ticket price is configured correctly.
        let network_min_ticket_price = hopr_chain_api
            .minimum_ticket_price()
            .await
            .map_err(HoprLibError::chain)?;
        let configured_ticket_price = self.cfg.protocol.packet.codec.outgoing_ticket_price;
        if configured_ticket_price.is_some_and(|c| c < network_min_ticket_price) {
            return Err(HoprLibError::GeneralError(format!(
                "configured outgoing ticket price is lower than the network minimum ticket price: \
                 {configured_ticket_price:?} < {network_min_ticket_price}"
            )));
        }
        // Once we are able to query the chain,
        // check if the winning probability is configured correctly.
        let network_min_win_prob = hopr_chain_api
            .minimum_incoming_ticket_win_prob()
            .await
            .map_err(HoprLibError::chain)?;
        let configured_win_prob = self.cfg.protocol.packet.codec.outgoing_win_prob;
        if !std::env::var("HOPR_TEST_DISABLE_CHECKS").is_ok_and(|v| v.to_lowercase() == "true")
            && configured_win_prob.is_some_and(|c| c.approx_cmp(&network_min_win_prob).is_lt())
        {
            return Err(HoprLibError::GeneralError(format!(
                "configured outgoing ticket winning probability is lower than the network minimum winning \
                 probability: {configured_win_prob:?} < {network_min_win_prob}"
            )));
        }

        tracing::info!(peer_id = %transport_id.public().to_peerid_str(), address = %me_onchain, version = constants::APP_VERSION, "Node information");

        let safe_addr = self.cfg.safe_module.safe_address;
        if me_onchain == safe_addr {
            return Err(HoprLibError::GeneralError(
                "cannot use self as staking safe address".into(),
            ));
        }

        tracing::info!(%safe_addr, "registering safe with this node");
        match hopr_chain_api.register_safe(&safe_addr).await {
            Ok(awaiter) => {
                // Wait until the registration is confirmed on-chain, otherwise we cannot proceed.
                awaiter.await.map_err(|error| {
                    tracing::error!(%safe_addr, %error, "safe registration failed with error");
                    HoprLibError::chain(error)
                })?;
                tracing::info!(%safe_addr, "safe successfully registered with this node");
            }
            Err(SafeRegistrationError::AlreadyRegistered(registered_safe)) if registered_safe == safe_addr => {
                tracing::info!(%safe_addr, "this safe is already registered with this node");
            }
            Err(SafeRegistrationError::AlreadyRegistered(registered_safe)) if registered_safe != safe_addr => {
                // TODO: support safe deregistration flow
                tracing::error!(%safe_addr, %registered_safe, "this node is currently registered with different safe");
                return Err(HoprLibError::GeneralError("node registered with different safe".into()));
            }
            Err(error) => {
                tracing::error!(%safe_addr, %error, "safe registration failed");
                return Err(HoprLibError::chain(error));
            }
        }

        // Only public nodes announce multiaddresses
        let multiaddresses_to_announce = if self.cfg.publish {
            // The multiaddresses are filtered for the non-private ones,
            // unless `announce_local_addresses` is set to `true`.
            hopr_transport_api.announceable_multiaddresses()
        } else {
            Vec::with_capacity(0)
        };

        // Warn when announcing a private multiaddress, which is acceptable in certain scenarios
        multiaddresses_to_announce
            .iter()
            .filter(|a| !is_public_address(a))
            .for_each(|multi_addr| tracing::warn!(?multi_addr, "announcing private multiaddress"));

        let chain_api = hopr_chain_api.clone();
        let me_offchain = *transport_id.public();
        let node_ready = spawn(async move { chain_api.await_key_binding(&me_offchain, NODE_READY_TIMEOUT).await });

        // At this point the node is already registered with Safe, so
        // we can announce via Safe-compliant TX
        tracing::info!(?multiaddresses_to_announce, "announcing node on chain");
        match hopr_chain_api
            .announce(&multiaddresses_to_announce, &transport_id)
            .await
        {
            Ok(awaiter) => {
                // Wait until the announcement is confirmed on-chain, otherwise we cannot proceed.
                awaiter.await.map_err(|error| {
                    tracing::error!(?multiaddresses_to_announce, %error, "node announcement failed");
                    HoprLibError::chain(error)
                })?;
                tracing::info!(?multiaddresses_to_announce, "node has been successfully announced");
            }
            Err(AnnouncementError::AlreadyAnnounced) => {
                tracing::info!(multiaddresses_announced = ?multiaddresses_to_announce, "node already announced on chain")
            }
            Err(error) => {
                tracing::error!(%error, ?multiaddresses_to_announce, "failed to transmit node announcement");
                return Err(HoprLibError::chain(error));
            }
        }

        // Wait for the node key-binding readiness to return
        let this_node_account = node_ready
            .await
            .map_err(HoprLibError::other)?
            .map_err(HoprLibError::chain)?;
        if this_node_account.chain_addr != me_onchain || this_node_account.safe_address.is_none_or(|a| a != safe_addr) {
            tracing::error!(%this_node_account, "account bound to offchain key does not match this node");
            return Err(HoprLibError::GeneralError("account key-binding mismatch".into()));
        }

        tracing::info!(%this_node_account, "node account is ready");

        tracing::info!("initializing session infrastructure");
        let incoming_session_channel_capacity = std::env::var("HOPR_INTERNAL_SESSION_INCOMING_CAPACITY")
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .filter(|&c| c > 0)
            .unwrap_or(256);

        let (session_tx, _session_rx) = channel::<IncomingSession>(incoming_session_channel_capacity);
        #[cfg(feature = "session-server")]
        {
            let serve_handler = self
                ._session_server
                .clone()
                .ok_or(HoprLibError::BuilderError("missing session server"))?;
            tracing::debug!(capacity = incoming_session_channel_capacity, "creating session server");
            processes.insert(
                HoprLibProcess::SessionServer,
                hopr_async_runtime::spawn_as_abortable!(
                    _session_rx
                        .for_each_concurrent(None, move |session| {
                            let serve_handler = serve_handler.clone();
                            async move {
                                let session_id = *session.session.id();
                                match serve_handler.process(session).await {
                                    Ok(_) => tracing::debug!(?session_id, "client session processed successfully"),
                                    Err(error) => tracing::error!(
                                        ?session_id,
                                        %error,
                                        "client session processing failed"
                                    ),
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
        self.session_tx = Some(session_tx);

        Ok(Hopr {
            transport_id,
            chain_id: NodeOnchainIdentity {
                node_address: chain_id.public().to_address(),
                safe_address: self.cfg.safe_module.safe_address,
                module_address: self.cfg.safe_module.module_address,
            },
            cfg: self.cfg.clone(),
            state: Arc::new(AtomicHoprState::new(HoprState::Uninitialized)),
            transport_api: hopr_transport_api,
            chain_api: hopr_chain_api,
            processes,
            ticket_event_subscribers: (new_tickets_tx, new_tickets_rx.deactivate()),
            ticket_manager,
        })
    }

    /// Builds an instance of an edge (Entry or Exit) [`Hopr`] node that is already started and running.
    ///
    /// This cannot process winning tickets nor relay packets.
    pub async fn build_edge(mut self) -> Result<Hopr<Chain, SharedChannelGraph, HoprNetwork, ()>, HoprLibError> {
        // No ticket manager needed here
        let mut hopr = self.pre_build(()).await?;

        let backend = self
            .ticket_index_db_path
            .map(RedbStore::new)
            .unwrap_or_else(RedbStore::new_temp)
            .map_err(hopr_ticket_manager::TicketManagerError::store)?;

        // Ticket factory can be constructed alone - without TicketManager
        let ticket_factory = Arc::new(HoprTicketFactory::new(backend));

        // Synchronize the ticket factory with the chain before starting the packet pipeline
        ticket_factory.sync_from_outgoing_channels(
            &hopr
                .chain_api
                .stream_channels(ChannelSelector::default().with_source(hopr.chain_id.node_address))
                .map_err(HoprLibError::chain)?
                .collect::<Vec<_>>()
                .await,
        )?;

        tracing::info!("starting transport for edge node");

        // TODO: use HoprSocket?
        let (_, transport_processes) = hopr
            .transport_api
            .run(
                self.ct.ok_or(HoprLibError::BuilderError("missing ct"))?,
                self.network_builder
                    .ok_or(HoprLibError::BuilderError("missing net builder"))?,
                futures::sink::drain(), // No tickets produced by edge node
                ticket_factory,
                self.session_tx
                    .ok_or(HoprLibError::BuilderError("missing session tx"))?,
            )
            .await?;
        hopr.processes
            .flat_map_extend_from(transport_processes, HoprLibProcess::Transport);

        hopr.state
            .store(HoprState::Running, std::sync::atomic::Ordering::Relaxed);
        tracing::info!(
            id = %hopr.transport_id.public().to_peerid_str(),
            version = constants::APP_VERSION,
            "NODE STARTED AND RUNNING"
        );

        Ok(hopr)
    }

    /// Builds an instance of a full [`Hopr`] node that is already started and running.
    ///
    /// This can process winning tickets and relay packets.
    pub async fn build_relay(
        mut self,
    ) -> Result<Hopr<Chain, SharedChannelGraph, HoprNetwork, SharedTicketManager>, HoprLibError> {
        tracing::info!("starting ticket manager & factory");

        let backend = self
            .ticket_index_db_path
            .clone()
            .map(RedbStore::new)
            .unwrap_or_else(RedbStore::new_temp)
            .map_err(hopr_ticket_manager::TicketManagerError::store)?;

        // Ticket factor must be constructed along with TicketManager
        let (ticket_manager, ticket_factory) = HoprTicketManager::new_with_factory(backend);
        let ticket_manager = Arc::new(ticket_manager);
        let ticket_factory = Arc::new(ticket_factory);

        // Construct with TicketManager
        let mut hopr = self.pre_build(ticket_manager.clone()).await?;

        // Synchronize the ticket manager with the chain before starting the packet pipeline
        ticket_manager.sync_from_incoming_channels(
            &hopr
                .chain_api
                .stream_channels(ChannelSelector::default().with_destination(hopr.chain_id.node_address))
                .map_err(HoprLibError::chain)?
                .collect::<Vec<_>>()
                .await,
        )?;

        // Synchronize the ticket factory with the chain before starting the packet pipeline
        ticket_factory.sync_from_outgoing_channels(
            &hopr
                .chain_api
                .stream_channels(ChannelSelector::default().with_source(hopr.chain_id.node_address))
                .map_err(HoprLibError::chain)?
                .collect::<Vec<_>>()
                .await,
        )?;

        // Make sure outgoing ticket indices in the ticket factory are periodically persisted to disk
        let (index_sync_handle, index_sync_reg) = futures::future::AbortHandle::new_pair();
        let tfact = ticket_factory.clone();
        let tfact2 = ticket_factory.clone();
        spawn(
            futures::stream::Abortable::new(
                futures_time::stream::interval(self.cfg.out_index_sync_period.into()),
                index_sync_reg,
            )
            .for_each(move |_| {
                let tfact = tfact.clone();
                async move {
                    if let Err(error) =
                        hopr_async_runtime::prelude::spawn_blocking(move || tfact.save_outgoing_indices())
                            .map_err(hopr_ticket_manager::TicketManagerError::store)
                            .and_then(futures::future::ready)
                            .await
                    {
                        tracing::error!(%error, "failed to sync ticket indices to persistent storage:");
                    } else {
                        tracing::trace!("successfully synced ticket indices");
                    }
                }
            })
            .inspect(move |_| {
                // Do an extra save here on graceful shutdown
                if let Err(error) = tfact2.save_outgoing_indices() {
                    tracing::error!(%error, "failed to sync ticket indices to persistent storage on shutdown");
                }
                tracing::warn!(
                    task = %HoprLibProcess::OutIndexSync,
                    "long-running background task finished"
                )
            }),
        );
        hopr.processes.insert(HoprLibProcess::OutIndexSync, index_sync_handle);

        tracing::info!("starting ticket events processor");
        let (tickets_tx, tickets_rx) = channel(8192);
        let (tickets_rx, tickets_handle) = futures::stream::abortable(tickets_rx);
        hopr.processes.insert(HoprLibProcess::TicketEvents, tickets_handle);
        let new_ticket_tx = hopr.ticket_event_subscribers.0.clone();
        let tmgr = ticket_manager.clone();
        spawn(
            tickets_rx
                .for_each(move |event| {
                    // Ticket manager processes new winning tickets
                    if let TicketEvent::WinningTicket(ticket) = &event
                        && let Err(error) = tmgr.insert_incoming_ticket(**ticket)
                    {
                        tracing::error!(%error, "failed to insert incoming ticket into manager");
                    }
                    if let Err(error) = new_ticket_tx.try_broadcast(event) {
                        tracing::error!(%error, "failed to broadcast new ticket event to subscribers");
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

        tracing::info!("starting transport for relay node");

        // TODO: use HoprSocket?
        let (_, transport_processes) = hopr
            .transport_api
            .run(
                self.ct.ok_or(HoprLibError::BuilderError("missing ct"))?,
                self.network_builder
                    .ok_or(HoprLibError::BuilderError("missing net builder"))?,
                tickets_tx,
                ticket_factory,
                self.session_tx
                    .ok_or(HoprLibError::BuilderError("missing session tx"))?,
            )
            .await?;
        hopr.processes
            .flat_map_extend_from(transport_processes, HoprLibProcess::Transport);

        // We need to subscribe to channel events to know when to neglect tickets
        tracing::info!("subscribing to channel events");
        let (chain_events_sub_handle, chain_events_sub_reg) = hopr_async_runtime::AbortHandle::new_pair();
        hopr.processes
            .insert(HoprLibProcess::ChannelEvents, chain_events_sub_handle);
        let chain = hopr.chain_api.clone();
        let events = chain.subscribe().map_err(HoprLibError::chain)?;
        let tmgr = ticket_manager.clone();

        spawn(
            futures::stream::Abortable::new(
                events
                    .filter_map(move |event|
                        futures::future::ready(event.try_as_channel_closed())
                    ),
                chain_events_sub_reg
            )
                .for_each(move |closed_channel| {
                    let chain = chain.clone();
                    let tmgr = tmgr.clone();
                    async move {
                        match closed_channel.direction(chain.me()) {
                            Some(ChannelDirection::Incoming) => {
                                // Ticket manager neglects tickets on incoming channel closure
                                match tmgr.neglect_tickets(closed_channel.get_id(), None) {
                                    Ok(neglected) if !neglected.is_empty() => {
                                        tracing::warn!(num_neglected = neglected.len(), %closed_channel, "tickets on incoming closed channel were neglected");
                                    },
                                    Ok(_) => {
                                        tracing::debug!(%closed_channel, "no neglected tickets on incoming closed channel");
                                    },
                                    Err(error) => {
                                        tracing::error!(%error, %closed_channel, "failed to mark tickets on incoming closed channel as neglected");
                                    }
                                }
                            },
                            Some(ChannelDirection::Outgoing) => {
                                // Removal of outgoing ticket index is done automatically be the ticket manager
                                // when new epoch is encountered
                            }
                            _ => {} // Event for a channel that is not our own
                        }
                    }
                })
                .inspect(|_| tracing::warn!(task = %HoprLibProcess::ChannelEvents, "long-running background task finished"))
        );

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
