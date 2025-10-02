//! HOPR library creating a unified [`Hopr`] object that can be used on its own,
//! as well as integrated into other systems and libraries.
//!
//! The [`Hopr`] object is standalone, meaning that once it is constructed and run,
//! it will perform its functionality autonomously. The API it offers serves as a
//! high-level integration point for other applications and utilities, but offers
//! a complete and fully featured HOPR node stripped from top level functionality
//! such as the REST API, key management...
//!
//! The intended way to use hopr_lib is for a specific tool to be built on top of it;
//! should the default `hoprd` implementation not be acceptable.
//!
//! For most of the practical use cases, the `hoprd` application should be a preferable
//! choice.

/// Helper functions.
mod helpers;

/// Configuration-related public types
pub mod config;
/// Various public constants.
pub mod constants;
/// Lists all errors thrown from this library.
pub mod errors;

/// Utility module with helper types and functionality over hopr-lib behavior.
pub mod utils;

/// Public traits for interactions with this library.
pub mod traits;

/// Functionality related to the HOPR node state.
pub mod state;

/// Re-exports of libraries necessary for API and interface operations.
pub mod exports {
    pub mod chain {
        pub use hopr_chain_types as types;
    }

    pub mod types {
        pub use hopr_primitive_types as primitive;
    }
    pub mod crypto {
        pub use hopr_crypto_keypair as keypair;
        pub use hopr_crypto_types as types;
    }

    pub mod network {
        pub use hopr_network_types as types;
    }

    pub use hopr_transport as transport;
}

/// Export of relevant types for easier integration.
pub mod prelude {
    pub use super::exports::{
        crypto::{
            keypair::key_pair::HoprKeys,
            types::prelude::{ChainKeypair, Hash, OffchainKeypair},
        },
        network::types::{
            prelude::ForeignDataMode,
            udp::{ConnectedUdpStream, UdpStreamParallelism},
        },
        transport::{OffchainPublicKey, socket::HoprSocket},
        types::primitive::prelude::Address,
    };
}

use std::{
    collections::HashMap,
    ops::Deref,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use async_lock::RwLock;
pub use async_trait::async_trait;
use errors::{HoprLibError, HoprStatusError};
use futures::{FutureExt, StreamExt, channel::mpsc::channel, future::AbortHandle};
use hopr_api::{
    chain::{
        AccountSelector, AnnouncementError, ChainEvents, ChainKeyOperations, ChainReadAccountOperations,
        ChainReadChannelOperations, ChainValues, ChainWriteAccountOperations, ChainWriteChannelOperations,
        ChainWriteTicketOperations, ChannelSelector,
    },
    db::{HoprDbPeersOperations, HoprDbTicketOperations, PeerStatus, TicketSelector},
};
use hopr_async_runtime::prelude::spawn;
pub use hopr_chain_api::config::{
    Addresses as NetworkContractAddresses, EnvironmentType, Network as ChainNetwork, ProtocolsConfig,
};
use hopr_chain_api::{HoprChain, HoprChainProcess, config::ChainNetworkConfig, errors::HoprChainError, wait_for_funds};
use hopr_chain_types::ContractAddresses;
pub use hopr_crypto_keypair::key_pair::{HoprKeys, IdentityRetrievalModes};
use hopr_crypto_types::prelude::Hash;
use hopr_db_node::{HoprNodeDb, HoprNodeDbConfig};
pub use hopr_internal_types::prelude::*;
pub use hopr_network_types::prelude::{DestinationRouting, IpProtocol, RoutingOptions};
pub use hopr_path::channel_graph::GraphExportConfig;
use hopr_path::channel_graph::{ChannelGraph, ChannelGraphConfig, NodeScoreUpdate};
#[cfg(all(feature = "prometheus", not(test)))]
use hopr_platform::time::native::current_time;
pub use hopr_primitive_types::prelude::*;
pub use hopr_strategy::Strategy;
use hopr_strategy::strategy::{MultiStrategy, SingularStrategy};
#[cfg(feature = "runtime-tokio")]
pub use hopr_transport::transfer_session;
pub use hopr_transport::{
    ApplicationData, ApplicationDataIn, ApplicationDataOut, HalfKeyChallenge, Health, HoprSession, IncomingSession,
    Keypair, Multiaddr, OffchainKeypair as HoprOffchainKeypair, PeerId, PingQueryReplier, ProbeError, SESSION_MTU,
    SURB_SIZE, ServiceId, SessionCapabilities, SessionCapability, SessionClientConfig, SessionId as HoprSessionId,
    SessionManagerError, SessionTarget, SurbBalancerConfig, Tag, TicketStatistics, TransportSessionError,
    config::{HostConfig, HostType, looks_like_domain},
    errors::{HoprTransportError, NetworkingError, ProtocolError},
};
use hopr_transport::{
    ChainKeypair, HoprTransport, HoprTransportConfig, OffchainKeypair, PeerDiscovery, execute_on_tick,
};
use tracing::{debug, error, info, trace, warn};

use crate::{
    config::SafeModule,
    constants::{MIN_NATIVE_BALANCE, SUGGESTED_NATIVE_BALANCE},
    state::HoprState,
    traits::chain::{CloseChannelResult, OpenChannelResult},
};

#[cfg(all(feature = "prometheus", not(test)))]
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

/// HOPR main object providing the entire HOPR node functionality
///
/// Instantiating this object creates all processes and objects necessary for
/// running the HOPR node. Once created, the node can be started using the
/// `run()` method.
///
/// Externally offered API should be sufficient to perform all necessary tasks
/// with the HOPR node manually, but it is advised to create such a configuration
/// that manual interaction is unnecessary.
///
/// As such, the `hopr_lib` serves mainly as an integration point into Rust programs.
pub struct Hopr {
    me: OffchainKeypair,
    cfg: config::HoprLibConfig,
    state: Arc<state::AtomicHoprState>,
    transport_api: HoprTransport<HoprNodeDb, HoprChain>,
    hopr_chain_api: HoprChain,
    node_db: HoprNodeDb,
    // objects that could be removed pending architectural cleanup ========
    chain_cfg: ChainNetworkConfig,
    channel_graph: Arc<RwLock<hopr_path::channel_graph::ChannelGraph>>,
    multistrategy: Arc<MultiStrategy>,
}

impl Hopr {
    pub fn new(
        mut cfg: config::HoprLibConfig,
        me: &OffchainKeypair,
        me_onchain: &ChainKeypair,
    ) -> crate::errors::Result<Self> {
        if hopr_crypto_random::is_rng_fixed() {
            warn!("!! FOR TESTING ONLY !! THIS BUILD IS USING AN INSECURE FIXED RNG !!")
        }

        let multiaddress: Multiaddr = (&cfg.host).try_into()?;

        let db_path: PathBuf = [&cfg.db.data, "node_db"].iter().collect();
        info!(path = ?db_path, "Initiating DB");

        if cfg.db.force_initialize {
            info!("Force cleaning up existing database");
            hopr_platform::file::native::remove_dir_all(db_path.as_path()).map_err(|e| {
                HoprLibError::GeneralError(format!(
                    "Failed to remove the existing DB directory at '{db_path:?}': {e}"
                ))
            })?;
            cfg.db.initialize = true
        }

        // create DB dir if it does not exist
        if let Some(parent_dir_path) = db_path.as_path().parent() {
            if !parent_dir_path.is_dir() {
                std::fs::create_dir_all(parent_dir_path).map_err(|e| {
                    HoprLibError::GeneralError(format!(
                        "Failed to create DB parent directory at '{parent_dir_path:?}': {e}"
                    ))
                })?
            }
        }

        let db_cfg = HoprNodeDbConfig {
            create_if_missing: cfg.db.initialize,
            force_create: cfg.db.force_initialize,
            log_slow_queries: std::time::Duration::from_millis(150),
            surb_ring_buffer_size: std::env::var("HOPR_PROTOCOL_SURB_RB_SIZE")
                .ok()
                .and_then(|s| u64::from_str(&s).map(|v| v as usize).ok())
                .unwrap_or_else(|| HoprNodeDbConfig::default().surb_ring_buffer_size),
            surb_distress_threshold: std::env::var("HOPR_PROTOCOL_SURB_RB_DISTRESS")
                .ok()
                .and_then(|s| u64::from_str(&s).map(|v| v as usize).ok())
                .unwrap_or_else(|| HoprNodeDbConfig::default().surb_distress_threshold),
        };
        let db = futures::executor::block_on(HoprNodeDb::new(db_path.as_path(), me_onchain.clone(), db_cfg))?;

        if let Some(provider) = &cfg.chain.provider {
            info!(provider, "Creating chain components using the custom provider");
        } else {
            info!("Creating chain components using the default provider");
        }
        let resolved_environment = hopr_chain_api::config::ChainNetworkConfig::new(
            &cfg.chain.network,
            crate::constants::APP_VERSION_COERCED,
            cfg.chain.provider.as_deref(),
            cfg.chain.max_rpc_requests_per_sec,
            &mut cfg.chain.protocols,
        )
        .map_err(|e| HoprLibError::GeneralError(format!("Failed to resolve blockchain environment: {e}")))?;

        let contract_addresses = ContractAddresses::from(&resolved_environment);
        info!(
            myself = me_onchain.public().to_hex(),
            contract_addresses = ?contract_addresses,
            "Resolved contract addresses",
        );

        let my_multiaddresses = vec![multiaddress];

        let channel_graph = Arc::new(RwLock::new(ChannelGraph::new(
            me_onchain.public().to_address(),
            ChannelGraphConfig::default(),
        )));

        // TODO (4.0): replace this with new implementation that follows the chain traits from the hopr-api crate
        let hopr_hopr_chain_api = hopr_chain_api::HoprChain::new(
            me_onchain.clone(),
            db.clone(),
            &cfg.db.data,
            resolved_environment.clone(),
            cfg.safe_module.module_address,
            ContractAddresses {
                announcements: resolved_environment.announcements,
                channels: resolved_environment.channels,
                token: resolved_environment.token,
                price_oracle: resolved_environment.ticket_price_oracle,
                win_prob_oracle: resolved_environment.winning_probability_oracle,
                network_registry: resolved_environment.network_registry,
                network_registry_proxy: resolved_environment.network_registry_proxy,
                stake_factory: resolved_environment.node_stake_v2_factory,
                safe_registry: resolved_environment.node_safe_registry,
                module_implementation: resolved_environment.module_implementation,
            },
            cfg.safe_module.safe_address,
            hopr_chain_api::IndexerConfig {
                start_block_number: resolved_environment.channel_contract_deploy_block as u64,
                fast_sync: cfg.chain.fast_sync,
                enable_logs_snapshot: cfg.chain.enable_logs_snapshot,
                logs_snapshot_url: cfg.chain.logs_snapshot_url.clone(),
                data_directory: cfg.db.data.clone(),
            },
        )?;

        let hopr_transport_api = HoprTransport::new(
            me,
            me_onchain,
            HoprTransportConfig {
                transport: cfg.transport.clone(),
                network: cfg.network_options.clone(),
                protocol: cfg.protocol,
                probe: cfg.probe,
                session: cfg.session,
            },
            db.clone(),
            hopr_hopr_chain_api.clone(),
            channel_graph.clone(),
            my_multiaddresses,
        );

        let multi_strategy = Arc::new(MultiStrategy::new(cfg.strategy.clone(), hopr_hopr_chain_api.clone()));
        debug!(
            strategies = tracing::field::debug(&multi_strategy),
            "Initialized strategies"
        );

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            METRIC_PROCESS_START_TIME.set(current_time().as_unix_timestamp().as_secs_f64());
            METRIC_HOPR_LIB_VERSION.set(
                &[const_format::formatcp!("{}", constants::APP_VERSION)],
                f64::from_str(const_format::formatcp!(
                    "{}.{}",
                    env!("CARGO_PKG_VERSION_MAJOR"),
                    env!("CARGO_PKG_VERSION_MINOR")
                ))
                .unwrap_or(0.0),
            );

            // Calling get_ticket_statistics will initialize the respective metrics on tickets
            if let Err(e) = futures::executor::block_on(db.get_ticket_statistics(None)) {
                error!(error = %e, "Failed to initialize ticket statistics metrics");
            }
        }

        Ok(Self {
            me: me.clone(),
            cfg,
            state: Arc::new(state::AtomicHoprState::new(state::HoprState::Uninitialized)),
            transport_api: hopr_transport_api,
            hopr_chain_api: hopr_hopr_chain_api,
            node_db: db,
            chain_cfg: resolved_environment,
            channel_graph,
            multistrategy: multi_strategy,
        })
    }

    fn error_if_not_in_state(&self, state: state::HoprState, error: String) -> errors::Result<()> {
        if self.status() == state {
            Ok(())
        } else {
            Err(HoprLibError::StatusError(HoprStatusError::NotThereYet(state, error)))
        }
    }

    pub fn status(&self) -> state::HoprState {
        self.state.load(Ordering::Relaxed)
    }

    pub fn network(&self) -> String {
        self.cfg.chain.network.clone()
    }

    pub async fn get_balance<C: Currency + Send>(&self) -> errors::Result<Balance<C>> {
        Ok(self.hopr_chain_api.node_balance().await?)
    }

    pub async fn get_safe_balance<C: Currency + Send>(&self) -> errors::Result<Balance<C>> {
        Ok(self.hopr_chain_api.safe_balance().await?)
    }

    pub fn get_safe_config(&self) -> SafeModule {
        self.cfg.safe_module.clone()
    }

    pub fn chain_config(&self) -> ChainNetworkConfig {
        self.chain_cfg.clone()
    }

    pub fn config(&self) -> &config::HoprLibConfig {
        &self.cfg
    }

    pub fn get_provider(&self) -> String {
        self.cfg
            .chain
            .provider
            .clone()
            .unwrap_or(self.chain_cfg.chain.default_provider.clone())
    }

    #[inline]
    fn is_public(&self) -> bool {
        self.cfg.chain.announce
    }

    pub async fn run<
        #[cfg(feature = "session-server")] T: traits::session::HoprSessionServer + Clone + Send + 'static,
    >(
        &self,
        #[cfg(feature = "session-server")] serve_handler: T,
    ) -> errors::Result<(
        hopr_transport::socket::HoprSocket<
            futures::channel::mpsc::Receiver<ApplicationDataIn>,
            futures::channel::mpsc::Sender<(
                hopr_transport::ApplicationDataOut,
                hopr_network_types::types::DestinationRouting,
            )>,
        >,
        HashMap<state::HoprLibProcesses, AbortHandle>,
    )> {
        self.error_if_not_in_state(
            state::HoprState::Uninitialized,
            "Cannot start the hopr node multiple times".into(),
        )?;

        info!(
            address = %self.me_onchain(), minimum_balance = %*SUGGESTED_NATIVE_BALANCE,
            "Node is not started, please fund this node",
        );

        wait_for_funds(
            *MIN_NATIVE_BALANCE,
            *SUGGESTED_NATIVE_BALANCE,
            Duration::from_secs(200),
            &self.hopr_chain_api,
        )
        .await?;

        let mut processes: HashMap<state::HoprLibProcesses, AbortHandle> = HashMap::new();

        info!("Starting the node...");

        self.state.store(state::HoprState::Initializing, Ordering::Relaxed);

        let balance: XDaiBalance = self.get_balance().await?;
        let minimum_balance = *constants::MIN_NATIVE_BALANCE;

        info!(
            address = %self.hopr_chain_api.me_onchain(),
            %balance,
            %minimum_balance,
            "Node information"
        );

        if balance.le(&minimum_balance) {
            return Err(HoprLibError::GeneralError(
                "Cannot start the node without a sufficiently funded wallet".to_string(),
            ));
        }

        // Once we are able to query the chain,
        // check if the ticket price is configured correctly.
        let network_min_ticket_price = self.hopr_chain_api.minimum_ticket_price().await?;

        let configured_ticket_price = self.cfg.protocol.outgoing_ticket_price;
        if configured_ticket_price.is_some_and(|c| c < network_min_ticket_price) {
            return Err(HoprLibError::ChainApi(HoprChainError::Api(format!(
                "configured outgoing ticket price is lower than the network minimum ticket price: \
                 {configured_ticket_price:?} < {network_min_ticket_price}"
            ))));
        }

        // Once we are able to query the chain,
        // check if the winning probability is configured correctly.
        let network_min_win_prob = self.hopr_chain_api.minimum_incoming_ticket_win_prob().await?;
        let configured_win_prob = self.cfg.protocol.outgoing_ticket_winning_prob;
        if !std::env::var("HOPR_TEST_DISABLE_CHECKS").is_ok_and(|v| v.to_lowercase() == "true")
            && configured_win_prob
                .and_then(|c| WinningProbability::try_from(c).ok())
                .is_some_and(|c| c.approx_cmp(&network_min_win_prob).is_lt())
        {
            return Err(HoprLibError::ChainApi(HoprChainError::Api(format!(
                "configured outgoing ticket winning probability is lower than the network minimum winning \
                 probability: {configured_win_prob:?} < {network_min_win_prob}"
            ))));
        }

        self.state.store(state::HoprState::Indexing, Ordering::Relaxed);

        // Calculate the minimum capacity based on accounts (each account can generate 2 messages),
        // plus 100 as an additional buffer
        let minimum_capacity = self
            .hopr_chain_api
            .count_accounts(AccountSelector {
                public_only: true,
                ..Default::default()
            })
            .await?
            .saturating_mul(2)
            .saturating_add(100);

        let chain_discovery_events_capacity = std::env::var("HOPR_INTERNAL_CHAIN_DISCOVERY_CHANNEL_CAPACITY")
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .filter(|&c| c > 0)
            .unwrap_or(2048)
            .max(minimum_capacity);

        debug!(
            capacity = chain_discovery_events_capacity,
            minimum_required = minimum_capacity,
            "Creating chain discovery events channel"
        );
        let (indexer_peer_update_tx, indexer_peer_update_rx) =
            futures::channel::mpsc::channel::<PeerDiscovery>(chain_discovery_events_capacity);

        let indexer_event_pipeline = helpers::chain_events_to_transport_events(
            self.hopr_chain_api.subscribe()?,
            self.me_onchain(),
            self.multistrategy.clone(),
            self.channel_graph.clone(),
            self.node_db.clone(),
        );

        spawn(async move {
            let result = indexer_event_pipeline
                .map(Ok)
                .forward(indexer_peer_update_tx)
                .inspect(|result| {
                    tracing::warn!(
                        ?result,
                        task = "indexer -> transport",
                        "long-running background task finished"
                    )
                })
                .await;

            result.expect("The index to transport event chain failed")
        });

        info!("Start the chain process and sync the indexer");
        for (id, proc) in self.hopr_chain_api.start().await?.into_iter() {
            let nid = match id {
                HoprChainProcess::Indexer => state::HoprLibProcesses::Indexing,
                HoprChainProcess::OutgoingOnchainActionQueue => state::HoprLibProcesses::OutgoingOnchainActionQueue,
            };
            processes.insert(nid, proc);
        }

        info!(peer_id = %self.me_peer_id(), address = %self.me_onchain(), version = constants::APP_VERSION, "Node information");

        // Check Safe-module status:
        // 1) if the node is already included into the module
        // 2) if the module is enabled in the safe
        // 3) if the safe is the owner of the module
        // if any of the conditions is not met, return error
        if !self.hopr_chain_api.check_node_safe_module_status().await? {
            return Err(HoprLibError::ChainApi(HoprChainError::Api(
                "Safe and module are not configured correctly".into(),
            )));
        }

        // Possibly register a node-safe pair to NodeSafeRegistry.
        // Following that, the connector is set to use safe tx variants.

        if self
            .hopr_chain_api
            .can_register_with_safe(&self.cfg.safe_module.safe_address)
            .await?
        {
            info!("Registering safe by node");

            if self.me_onchain() == self.cfg.safe_module.safe_address {
                return Err(HoprLibError::GeneralError("cannot self as staking safe address".into()));
            }

            if let Err(error) = self
                .hopr_chain_api
                .register_safe(&self.cfg.safe_module.safe_address)
                .await?
                .await
            {
                // Intentionally ignoring the errored state
                error!(%error, "Failed to register node with safe")
            }
        }

        if self.is_public() {
            // At this point the node is already registered with Safe, so
            // we can announce via Safe-compliant TX

            let multiaddresses_to_announce = self.transport_api.announceable_multiaddresses();

            // The announcement is intentionally not awaited until confirmation
            match self
                .hopr_chain_api
                .announce(&multiaddresses_to_announce, &self.me)
                .await
            {
                Ok(_) => info!(?multiaddresses_to_announce, "Announcing node on chain",),
                Err(AnnouncementError::AlreadyAnnounced) => {
                    info!(multiaddresses_announced = ?multiaddresses_to_announce, "Node already announced on chain")
                }
                // If the announcement fails, we keep going to prevent the node from retrying
                // after restart.
                // Functionality is limited, and users must check the logs for errors.
                Err(error) => error!(%error, "Failed to transmit node announcement"),
            }
        }

        {
            // Sync key ids from indexed Accounts

            // Sync the Channel graph
            let channel_graph = self.channel_graph.clone();
            let mut cg = channel_graph.write_arc().await;

            info!("Syncing channels from the previous runs");
            let mut channel_stream = self.hopr_chain_api.stream_channels(ChannelSelector::any()).await?;
            while let Some(channel) = channel_stream.next().await {
                cg.update_channel(channel);
            }

            // Initialize node latencies and scores in the channel graph:
            // Sync only those nodes that we know that had a good quality
            // Other nodes will be repopulated into the channel graph during heartbeat
            // rounds.
            info!("Syncing peer qualities from the previous runs");
            let min_quality_to_sync: f64 = std::env::var("HOPR_MIN_PEER_QUALITY_TO_SYNC")
                .map_err(|e| e.to_string())
                .and_then(|v| std::str::FromStr::from_str(&v).map_err(|_| "parse error".to_string()))
                .unwrap_or_else(|error| {
                    warn!(error, "invalid value for HOPR_MIN_PEER_QUALITY_TO_SYNC env variable");
                    constants::DEFAULT_MIN_QUALITY_TO_SYNC
                });

            let mut peer_stream = self
                .node_db
                .get_network_peers(Default::default(), false)
                .await?
                .filter(|status| futures::future::ready(status.quality >= min_quality_to_sync));

            while let Some(peer) = peer_stream.next().await {
                if let Some(key) = self.hopr_chain_api.packet_key_to_chain_key(&peer.id.0).await? {
                    // For nodes that had a good quality, we assign a perfect score
                    cg.update_node_score(&key, NodeScoreUpdate::Initialize(peer.last_seen_latency, 1.0));
                } else {
                    error!(peer = %peer.id.1, "Could not translate peer information");
                }
            }

            info!(
                channels = cg.count_channels(),
                nodes = cg.count_nodes(),
                "Channel graph sync complete"
            );
        }

        // notifier on acknowledged ticket reception
        let multi_strategy_ack_ticket = self.multistrategy.clone();

        let ack_ticket_channel_capacity = std::env::var("HOPR_INTERNAL_ACKED_TICKET_CHANNEL_CAPACITY")
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .filter(|&c| c > 0)
            .unwrap_or(2048);

        debug!(
            capacity = ack_ticket_channel_capacity,
            "Creating acknowledged ticket channel"
        );
        let (on_ack_tkt_tx, mut on_ack_tkt_rx) = channel::<AcknowledgedTicket>(ack_ticket_channel_capacity);
        self.node_db.start_ticket_processing(Some(on_ack_tkt_tx))?;

        processes.insert(
            state::HoprLibProcesses::OnReceivedAcknowledgement,
            hopr_async_runtime::spawn_as_abortable!(async move {
                while let Some(ack) = on_ack_tkt_rx.next().await {
                    if let Err(error) = hopr_strategy::strategy::SingularStrategy::on_acknowledged_winning_ticket(
                        &*multi_strategy_ack_ticket,
                        &ack,
                    )
                    .await
                    {
                        error!(%error, "Failed to process acknowledged winning ticket with the strategy");
                    }
                }

                tracing::warn!(
                    task = %state::HoprLibProcesses::OnReceivedAcknowledgement,
                    "long-running background task finished"
                )
            }),
        );

        let incoming_session_channel_capacity = std::env::var("HOPR_INTERNAL_SESSION_INCOMING_CAPACITY")
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .filter(|&c| c > 0)
            .unwrap_or(256);

        debug!(
            capacity = incoming_session_channel_capacity,
            "Creating incoming session channel"
        );
        let (session_tx, _session_rx) = channel::<IncomingSession>(incoming_session_channel_capacity);

        #[cfg(feature = "session-server")]
        {
            processes.insert(
                state::HoprLibProcesses::SessionServer,
                hopr_async_runtime::spawn_as_abortable!(
                    _session_rx
                        .for_each_concurrent(None, move |session| {
                            let serve_handler = serve_handler.clone();
                            async move {
                                let session_id = *session.session.id();
                                match serve_handler.process(session).await {
                                    Ok(_) => debug!(
                                        session_id = ?session_id,
                                        "Client session processed successfully"
                                    ),
                                    Err(e) => error!(
                                        session_id = ?session_id,
                                        error = %e,
                                        "Client session processing failed"
                                    ),
                                }
                            }
                        })
                        .inspect(|_| tracing::warn!(
                            task = %state::HoprLibProcesses::SessionServer,
                            "long-running background task finished"
                        ))
                ),
            );
        }

        info!("Starting transport");
        let (hopr_socket, transport_processes) = self.transport_api.run(indexer_peer_update_rx, session_tx).await?;
        for (id, proc) in transport_processes.into_iter() {
            processes.insert(id.into(), proc);
        }

        let db_clone = self.node_db.clone();
        processes.insert(
            state::HoprLibProcesses::TicketIndexFlush,
            hopr_async_runtime::spawn_as_abortable!(
                Box::pin(execute_on_tick(
                    Duration::from_secs(5),
                    move || {
                        let db_clone = db_clone.clone();
                        async move {
                            match db_clone.persist_outgoing_ticket_indices().await {
                                Ok(n) => debug!(count = n, "Successfully flushed states of outgoing ticket indices"),
                                Err(e) => error!(error = %e, "Failed to flush ticket indices"),
                            }
                        }
                    },
                    "flush the states of outgoing ticket indices".into(),
                ))
                .inspect(|_| tracing::warn!(
                    task = %state::HoprLibProcesses::TicketIndexFlush,
                    "long-running background task finished"
                ))
            ),
        );

        // NOTE: after the chain is synced, we can reset tickets which are considered
        // redeemed but on-chain state does not align with that. This implies there was a problem
        // right when the transaction was sent on-chain. In such cases, we simply let it retry and
        // handle errors appropriately.
        let mut channels = self
            .hopr_chain_api
            .stream_channels(ChannelSelector {
                direction: vec![ChannelDirection::Incoming],
                ..Default::default()
            })
            .await?;

        while let Some(channel) = channels.next().await {
            self.node_db
                .update_ticket_states_and_fetch(
                    TicketSelector::from(&channel)
                        .with_state(AcknowledgedTicketStatus::BeingRedeemed)
                        .with_index_range(channel.ticket_index.as_u64()..),
                    AcknowledgedTicketStatus::Untouched,
                )
                .await?
                .for_each(|ticket| {
                    info!(%ticket, "fixed next out-of-sync ticket");
                    futures::future::ready(())
                })
                .await;
        }

        // NOTE: strategy ticks must start after the chain is synced, otherwise
        // the strategy would react to historical data and drain through the native
        // balance on chain operations not relevant for the present network state
        let multi_strategy = self.multistrategy.clone();
        let strategy_interval = self.cfg.strategy.execution_interval;
        processes.insert(
            state::HoprLibProcesses::StrategyTick,
            hopr_async_runtime::spawn_as_abortable!(
                execute_on_tick(
                    Duration::from_secs(strategy_interval),
                    move || {
                        let multi_strategy = multi_strategy.clone();

                        async move {
                            trace!(state = "started", "strategy tick");
                            let _ = multi_strategy.on_tick().await;
                            trace!(state = "finished", "strategy tick");
                        }
                    },
                    "run strategies".into(),
                )
                .inspect(
                    |_| tracing::warn!(task = %state::HoprLibProcesses::StrategyTick, "long-running background task finished")
                )
            ),
        );

        self.state.store(state::HoprState::Running, Ordering::Relaxed);

        info!(
            id = %self.me_peer_id(),
            version = constants::APP_VERSION,
            "NODE STARTED AND RUNNING"
        );

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_HOPR_NODE_INFO.set(
            &[
                &self.me.public().to_peerid_str(),
                &self.me_onchain().to_string(),
                &self.cfg.safe_module.safe_address.to_string(),
                &self.cfg.safe_module.module_address.to_string(),
            ],
            1.0,
        );

        Ok((hopr_socket, processes))
    }

    // p2p transport =========
    /// Own PeerId used in the libp2p transport layer
    pub fn me_peer_id(&self) -> PeerId {
        (*self.me.public()).into()
    }

    /// Get the list of all announced public nodes in the network
    pub async fn get_public_nodes(&self) -> errors::Result<Vec<(PeerId, Address, Vec<Multiaddr>)>> {
        Ok(self
            .hopr_chain_api
            .stream_accounts(AccountSelector {
                public_only: true,
                ..Default::default()
            })
            .await?
            .filter_map(|entry| {
                futures::future::ready(
                    entry
                        .get_multiaddr()
                        .map(|maddr| (PeerId::from(entry.public_key), entry.chain_addr, vec![maddr])),
                )
            })
            .collect()
            .await)
    }

    /// Ping another node in the network based on the PeerId
    ///
    /// Returns the RTT (round trip time), i.e. how long it took for the ping to return.
    pub async fn ping(&self, peer: &PeerId) -> errors::Result<(std::time::Duration, PeerStatus)> {
        self.error_if_not_in_state(
            state::HoprState::Running,
            "Node is not ready for on-chain operations".into(),
        )?;

        Ok(self.transport_api.ping(peer).await?)
    }

    /// Create a client session connection returning a session object that implements
    /// [`futures::io::AsyncRead`] and [`futures::io::AsyncWrite`] and can bu used as a read/write binary session.
    #[cfg(feature = "session-client")]
    pub async fn connect_to(
        &self,
        destination: Address,
        target: SessionTarget,
        cfg: SessionClientConfig,
    ) -> errors::Result<HoprSession> {
        self.error_if_not_in_state(
            state::HoprState::Running,
            "Node is not ready for on-chain operations".into(),
        )?;

        let backoff = backon::ConstantBuilder::default()
            .with_max_times(self.cfg.session.establish_max_retries as usize)
            .with_delay(self.cfg.session.establish_retry_timeout)
            .with_jitter();

        use backon::Retryable;

        Ok((|| {
            let cfg = cfg.clone();
            let target = target.clone();
            async { self.transport_api.new_session(destination, target, cfg).await }
        })
        .retry(backoff)
        .sleep(backon::FuturesTimerSleeper)
        .await?)
    }

    /// Sends keep-alive to the given [`HoprSessionId`], making sure the session is not
    /// closed due to inactivity.
    #[cfg(feature = "session-client")]
    pub async fn keep_alive_session(&self, id: &HoprSessionId) -> errors::Result<()> {
        self.error_if_not_in_state(
            state::HoprState::Running,
            "Node is not ready for on-chain operations".into(),
        )?;
        Ok(self.transport_api.probe_session(id).await?)
    }

    #[cfg(feature = "session-client")]
    pub async fn get_session_surb_balancer_config(
        &self,
        id: &HoprSessionId,
    ) -> errors::Result<Option<SurbBalancerConfig>> {
        self.error_if_not_in_state(
            state::HoprState::Running,
            "Node is not ready for on-chain operations".into(),
        )?;
        Ok(self.transport_api.session_surb_balancing_cfg(id).await?)
    }

    #[cfg(feature = "session-client")]
    pub async fn update_session_surb_balancer_config(
        &self,
        id: &HoprSessionId,
        cfg: SurbBalancerConfig,
    ) -> errors::Result<()> {
        self.error_if_not_in_state(
            state::HoprState::Running,
            "Node is not ready for on-chain operations".into(),
        )?;
        Ok(self.transport_api.update_session_surb_balancing_cfg(id, cfg).await?)
    }

    /// List all multiaddresses announced by this node
    pub fn local_multiaddresses(&self) -> Vec<Multiaddr> {
        self.transport_api.local_multiaddresses()
    }

    /// List all multiaddresses on which the node is listening
    pub async fn listening_multiaddresses(&self) -> Vec<Multiaddr> {
        self.transport_api.listening_multiaddresses().await
    }

    /// List all multiaddresses observed for a PeerId
    pub async fn network_observed_multiaddresses(&self, peer: &PeerId) -> Vec<Multiaddr> {
        self.transport_api.network_observed_multiaddresses(peer).await
    }

    /// List all multiaddresses announced on-chain for the given node.
    pub async fn multiaddresses_announced_on_chain(&self, peer: &PeerId) -> Vec<Multiaddr> {
        let peer = *peer;
        // PeerId -> OffchainPublicKey is a CPU-intensive blocking operation
        let pubkey =
            match hopr_parallelize::cpu::spawn_blocking(move || prelude::OffchainPublicKey::from_peerid(&peer)).await {
                Ok(k) => k,
                Err(e) => {
                    error!(%peer, error = %e, "failed to convert peer id to off-chain key");
                    return vec![];
                }
            };

        match self.hopr_chain_api.find_account_by_packet_key(&pubkey).await {
            Ok(Some(entry)) => Vec::from_iter(entry.get_multiaddr()),
            Ok(None) => {
                error!(%peer, "no information");
                vec![]
            }
            Err(e) => {
                error!(%peer, error = %e, "failed to retrieve information");
                vec![]
            }
        }
    }

    // Network =========

    /// Get measured network health
    pub async fn network_health(&self) -> Health {
        self.transport_api.network_health().await
    }

    /// List all peers connected to this
    pub async fn network_connected_peers(&self) -> errors::Result<Vec<PeerId>> {
        Ok(self.transport_api.network_connected_peers().await?)
    }

    /// Get all data collected from the network relevant for a PeerId
    pub async fn network_peer_info(&self, peer: &PeerId) -> errors::Result<Option<PeerStatus>> {
        Ok(self.transport_api.network_peer_info(peer).await?)
    }

    /// Get peers connected peers with quality higher than some value
    pub async fn all_network_peers(
        &self,
        minimum_quality: f64,
    ) -> errors::Result<Vec<(Option<Address>, PeerId, PeerStatus)>> {
        Ok(
            futures::stream::iter(self.transport_api.network_connected_peers().await?)
                .filter_map(|peer| async move {
                    if let Ok(Some(info)) = self.transport_api.network_peer_info(&peer).await {
                        if info.get_average_quality() >= minimum_quality {
                            Some((peer, info))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .filter_map(|(peer_id, info)| async move {
                    let address = self.peerid_to_chain_key(&peer_id).await.ok().flatten();
                    Some((address, peer_id, info))
                })
                .collect::<Vec<_>>()
                .await,
        )
    }

    // Ticket ========
    /// Get all tickets in a channel specified by [`prelude::Hash`]
    pub async fn tickets_in_channel(&self, channel: &prelude::Hash) -> errors::Result<Option<Vec<AcknowledgedTicket>>> {
        Ok(self.transport_api.tickets_in_channel(channel).await?)
    }

    /// Get all tickets
    pub async fn all_tickets(&self) -> errors::Result<Vec<Ticket>> {
        Ok(self.transport_api.all_tickets().await?)
    }

    /// Get statistics for all tickets
    pub async fn ticket_statistics(&self) -> errors::Result<TicketStatistics> {
        Ok(self.transport_api.ticket_statistics().await?)
    }

    /// Reset the ticket metrics to zero
    pub async fn reset_ticket_statistics(&self) -> errors::Result<()> {
        Ok(self.node_db.reset_ticket_statistics().await?)
    }

    // DB ============
    pub fn peer_resolver(&self) -> &impl ChainKeyOperations {
        &self.hopr_chain_api
    }

    // Chain =========
    pub fn me_onchain(&self) -> Address {
        self.hopr_chain_api.me_onchain()
    }

    /// Get ticket price
    pub async fn get_ticket_price(&self) -> errors::Result<HoprBalance> {
        Ok(self.hopr_chain_api.minimum_ticket_price().await?)
    }

    /// Get minimum incoming ticket winning probability
    pub async fn get_minimum_incoming_ticket_win_probability(&self) -> errors::Result<WinningProbability> {
        Ok(self.hopr_chain_api.minimum_incoming_ticket_win_prob().await?)
    }

    /// List of all accounts announced on the chain
    pub async fn accounts_announced_on_chain(&self) -> errors::Result<Vec<AccountEntry>> {
        Ok(self
            .hopr_chain_api
            .stream_accounts(AccountSelector {
                public_only: true,
                ..Default::default()
            })
            .await?
            .collect()
            .await)
    }

    /// Get the channel entry from Hash.
    /// @returns the channel entry of those two nodes
    pub async fn channel_from_hash(&self, channel_id: &Hash) -> errors::Result<Option<ChannelEntry>> {
        Ok(self.hopr_chain_api.channel_by_id(channel_id).await?)
    }

    /// Get the channel entry between source and destination node.
    /// @param src Address
    /// @param dest Address
    /// @returns the channel entry of those two nodes
    pub async fn channel(&self, src: &Address, dest: &Address) -> errors::Result<Option<ChannelEntry>> {
        Ok(self.hopr_chain_api.channel_by_parties(src, dest).await?)
    }

    /// List all channels open from a specified Address
    pub async fn channels_from(&self, src: &Address) -> errors::Result<Vec<ChannelEntry>> {
        Ok(self
            .hopr_chain_api
            .stream_channels(ChannelSelector {
                counterparty: Some(*src),
                direction: vec![ChannelDirection::Incoming],
                allowed_states: vec![
                    ChannelStatusDiscriminants::Closed,
                    ChannelStatusDiscriminants::Open,
                    ChannelStatusDiscriminants::PendingToClose,
                ],
            })
            .await?
            .collect()
            .await)
    }

    /// List all channels open to a specified address
    pub async fn channels_to(&self, dest: &Address) -> errors::Result<Vec<ChannelEntry>> {
        Ok(self
            .hopr_chain_api
            .stream_channels(ChannelSelector {
                counterparty: Some(*dest),
                direction: vec![ChannelDirection::Outgoing],
                allowed_states: vec![
                    ChannelStatusDiscriminants::Closed,
                    ChannelStatusDiscriminants::Open,
                    ChannelStatusDiscriminants::PendingToClose,
                ],
            })
            .await?
            .collect()
            .await)
    }

    /// List all channels
    pub async fn all_channels(&self) -> errors::Result<Vec<ChannelEntry>> {
        Ok(self
            .hopr_chain_api
            .stream_channels(ChannelSelector {
                counterparty: None,
                direction: vec![],
                allowed_states: vec![
                    ChannelStatusDiscriminants::Closed,
                    ChannelStatusDiscriminants::Open,
                    ChannelStatusDiscriminants::PendingToClose,
                ],
            })
            .await?
            .collect()
            .await)
    }

    /// List all corrupted channels
    pub async fn corrupted_channels(&self) -> errors::Result<Vec<CorruptedChannelEntry>> {
        Ok(self.hopr_chain_api.corrupted_channels().await?)
    }

    /// Current safe allowance balance
    pub async fn safe_allowance(&self) -> errors::Result<HoprBalance> {
        Ok(self.hopr_chain_api.safe_allowance().await?)
    }

    /// Withdraw on-chain assets to a given address
    /// @param recipient the account where the assets should be transferred to
    /// @param amount how many tokens to be transferred
    pub async fn withdraw_tokens(&self, recipient: Address, amount: HoprBalance) -> errors::Result<prelude::Hash> {
        self.error_if_not_in_state(
            state::HoprState::Running,
            "Node is not ready for on-chain operations".into(),
        )?;

        let awaiter = self.hopr_chain_api.withdraw(amount, &recipient).await?;

        Ok(awaiter.await?)
    }

    /// Withdraw on-chain native assets to a given address
    /// @param recipient the account where the assets should be transferred to
    /// @param amount how many tokens to be transferred
    pub async fn withdraw_native(&self, recipient: Address, amount: XDaiBalance) -> errors::Result<prelude::Hash> {
        self.error_if_not_in_state(
            state::HoprState::Running,
            "Node is not ready for on-chain operations".into(),
        )?;

        let awaiter = self.hopr_chain_api.withdraw(amount, &recipient).await?;

        Ok(awaiter.await?)
    }

    pub async fn open_channel(&self, destination: &Address, amount: HoprBalance) -> errors::Result<OpenChannelResult> {
        self.error_if_not_in_state(
            state::HoprState::Running,
            "Node is not ready for on-chain operations".into(),
        )?;

        let (channel_id, tx_hash) = self.hopr_chain_api.open_channel(destination, amount).await?.await?;

        Ok(OpenChannelResult { tx_hash, channel_id })
    }

    pub async fn fund_channel(&self, channel_id: &prelude::Hash, amount: HoprBalance) -> errors::Result<prelude::Hash> {
        self.error_if_not_in_state(
            state::HoprState::Running,
            "Node is not ready for on-chain operations".into(),
        )?;

        let awaiter = self.hopr_chain_api.fund_channel(channel_id, amount).await?;

        Ok(awaiter.await?)
    }

    pub async fn close_channel_by_id(&self, channel_id: &ChannelId) -> errors::Result<CloseChannelResult> {
        self.error_if_not_in_state(
            state::HoprState::Running,
            "Node is not ready for on-chain operations".into(),
        )?;

        let (status, tx_hash) = self.hopr_chain_api.close_channel(channel_id).await?.await?;

        Ok(CloseChannelResult { tx_hash, status })
    }

    pub async fn get_channel_closure_notice_period(&self) -> errors::Result<Duration> {
        Ok(self.hopr_chain_api.channel_closure_notice_period().await?)
    }

    pub async fn redeem_all_tickets<B: Into<HoprBalance>>(
        &self,
        min_value: B,
        only_aggregated: bool,
    ) -> errors::Result<()> {
        self.error_if_not_in_state(
            state::HoprState::Running,
            "Node is not ready for on-chain operations".into(),
        )?;

        let min_value = min_value.into();
        let chain_api = self.hopr_chain_api.clone();
        self.hopr_chain_api
            .stream_channels(ChannelSelector {
                counterparty: None,
                direction: vec![ChannelDirection::Incoming],
                allowed_states: vec![
                    ChannelStatusDiscriminants::Open,
                    ChannelStatusDiscriminants::PendingToClose,
                ],
            })
            .await?
            .for_each_concurrent(20, |channel| {
                let chain_api = chain_api.clone();
                async move {
                    match chain_api
                        .redeem_tickets_via_selector(
                            TicketSelector::from(&channel)
                                .with_amount(min_value..)
                                .with_aggregated_only(only_aggregated)
                                .with_index_range(channel.ticket_index.as_u64()..)
                                .with_state(AcknowledgedTicketStatus::Untouched),
                        )
                        .await
                    {
                        Ok(awaiters) => info!(count = awaiters.len(), %channel, "redeemed tickets in channel"),
                        Err(error) => error!(%error, %channel, "failed to redeem tickets"),
                    }
                }
            })
            .await;

        Ok(())
    }

    pub async fn redeem_tickets_with_counterparty<B: Into<HoprBalance>>(
        &self,
        counterparty: &Address,
        min_value: B,
        only_aggregated: bool,
    ) -> errors::Result<usize> {
        self.redeem_tickets_in_channel(
            &generate_channel_id(counterparty, &self.me_onchain()),
            min_value,
            only_aggregated,
        )
        .await
    }

    pub async fn redeem_tickets_in_channel<B: Into<HoprBalance>>(
        &self,
        channel_id: &prelude::Hash,
        min_value: B,
        only_aggregated: bool,
    ) -> errors::Result<usize> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let channel = self
            .hopr_chain_api
            .channel_by_id(channel_id)
            .await?
            .ok_or(HoprLibError::GeneralError("Channel not found".into()))?;

        let out = self
            .hopr_chain_api
            .redeem_tickets_via_selector(
                TicketSelector::from(channel)
                    .with_amount(min_value.into()..)
                    .with_aggregated_only(only_aggregated)
                    .with_index_range(channel.ticket_index.as_u64()..)
                    .with_state(AcknowledgedTicketStatus::Untouched),
            )
            .await?;

        Ok(out.len())
    }

    pub async fn redeem_ticket(&self, ack_ticket: AcknowledgedTicket) -> errors::Result<()> {
        self.error_if_not_in_state(
            state::HoprState::Running,
            "Node is not ready for on-chain operations".into(),
        )?;

        // We do not await the on-chain confirmation
        #[allow(clippy::let_underscore_future)]
        let _ = self.hopr_chain_api.redeem_ticket(ack_ticket).await?;

        Ok(())
    }

    pub async fn peerid_to_chain_key(&self, peer_id: &PeerId) -> errors::Result<Option<Address>> {
        let peer_id = *peer_id;
        // PeerId -> OffchainPublicKey is a CPU-intensive blocking operation
        let pubkey = hopr_parallelize::cpu::spawn_blocking(move || prelude::OffchainPublicKey::from_peerid(&peer_id))
            .await
            .map_err(|e| HoprLibError::GeneralError(format!("failed to convert peer id to off-chain key: {}", e)))?;

        Ok(self.hopr_chain_api.packet_key_to_chain_key(&pubkey).await?)
    }

    pub async fn chain_key_to_peerid(&self, address: &Address) -> errors::Result<Option<PeerId>> {
        Ok(self
            .hopr_chain_api
            .chain_key_to_packet_key(address)
            .await
            .map(|pk| pk.map(|v| v.into()))?)
    }

    pub async fn export_channel_graph(&self, cfg: GraphExportConfig) -> String {
        self.channel_graph.read_arc().await.as_dot(cfg)
    }

    pub async fn export_raw_channel_graph(&self) -> errors::Result<String> {
        let cg = self.channel_graph.read_arc().await;
        serde_json::to_string(cg.deref()).map_err(|e| HoprLibError::GeneralError(e.to_string()))
    }

    pub async fn get_indexer_state(&self) -> errors::Result<hopr_chain_api::IndexerStateInfo> {
        Ok(self.hopr_chain_api.get_indexer_state().await?)
    }
}
