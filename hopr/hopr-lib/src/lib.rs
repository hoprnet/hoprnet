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

/// Configuration-related public types
pub mod config;
/// Various public constants.
pub mod constants;
/// Lists all errors thrown from this library.
pub mod errors;

use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    ops::Deref,
    path::PathBuf,
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use async_lock::RwLock;
use errors::{HoprLibError, HoprStatusError};
use futures::{
    SinkExt, Stream, StreamExt,
    channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded},
    future::AbortHandle,
    stream::{self},
};
use hopr_async_runtime::prelude::{sleep, spawn};
pub use hopr_chain_actions::errors::ChainActionsError;
use hopr_chain_actions::{
    action_state::{ActionState, IndexerActionTracker},
    channels::ChannelActions,
    node::NodeActions,
    redeem::TicketRedeemActions,
};
pub use hopr_chain_api::config::{
    Addresses as NetworkContractAddresses, EnvironmentType, Network as ChainNetwork, ProtocolsConfig,
};
use hopr_chain_api::{
    HoprChain, HoprChainProcess, SignificantChainEvent, can_register_with_safe, config::ChainNetworkConfig,
    errors::HoprChainError, wait_for_funds,
};
use hopr_chain_rpc::HoprRpcOperations;
use hopr_chain_types::{ContractAddresses, chain_events::ChainEventType};
use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_db_api::logs::HoprDbLogOperations;
use hopr_db_sql::{
    HoprDbAllOperations,
    accounts::HoprDbAccountOperations,
    api::{info::SafeInfo, resolver::HoprDbResolverOperations, tickets::HoprDbTicketOperations},
    channels::HoprDbChannelOperations,
    db::{HoprDb, HoprDbConfig},
    info::{HoprDbInfoOperations, IndexerStateInfo},
    prelude::{ChainOrPacketKey::ChainKey, HoprDbPeersOperations},
    registry::HoprDbRegistryOperations,
};
pub use hopr_internal_types::prelude::*;
pub use hopr_network_types::prelude::{DestinationRouting, IpProtocol, RoutingOptions};
pub use hopr_path::channel_graph::GraphExportConfig;
use hopr_path::channel_graph::{ChannelGraph, ChannelGraphConfig, NodeScoreUpdate};
use hopr_platform::file::native::{join, remove_dir_all};
pub use hopr_primitive_types::prelude::*;
pub use hopr_strategy::Strategy;
use hopr_strategy::strategy::{MultiStrategy, SingularStrategy};
#[cfg(feature = "runtime-tokio")]
pub use hopr_transport::transfer_session;
pub use hopr_transport::{
    ApplicationData, HalfKeyChallenge, Health, IncomingSession as HoprIncomingSession, Keypair, Multiaddr,
    OffchainKeypair as HoprOffchainKeypair, PeerId, PingQueryReplier, ProbeError, SESSION_MTU, SURB_SIZE, ServiceId,
    Session as HoprSession, SessionCapabilities, SessionCapability, SessionClientConfig, SessionId as HoprSessionId,
    SessionManagerError, SessionTarget, SurbBalancerConfig, Tag, TicketStatistics, TransportSessionError,
    config::{HostConfig, HostType, looks_like_domain},
    errors::{HoprTransportError, NetworkingError, ProtocolError},
};
use hopr_transport::{
    ChainKeypair, Hash, HoprTransport, HoprTransportConfig, HoprTransportProcess, IncomingSession, OffchainKeypair,
    PeerDiscovery, PeerStatus, execute_on_tick,
};
use tracing::{debug, error, info, trace, warn};
#[cfg(all(feature = "prometheus", not(test)))]
use {
    hopr_metrics::metrics::{MultiGauge, SimpleGauge},
    hopr_platform::time::native::current_time,
    std::str::FromStr,
};

use crate::{
    config::SafeModule,
    constants::{MIN_NATIVE_BALANCE, ONBOARDING_INFORMATION_INTERVAL, SUGGESTED_NATIVE_BALANCE},
};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_PROCESS_START_TIME: SimpleGauge = SimpleGauge::new(
        "hopr_start_time",
        "The unix timestamp in seconds at which the process was started"
    ).unwrap();
    static ref METRIC_HOPR_LIB_VERSION: MultiGauge = MultiGauge::new(
        "hopr_lib_version",
        "Executed version of hopr-lib",
        &["version"]
    ).unwrap();
    static ref METRIC_HOPR_NODE_INFO: MultiGauge = MultiGauge::new(
        "hopr_node_addresses",
        "Node on-chain and off-chain addresses",
        &["peerid", "address", "safe_address", "module_address"]
    ).unwrap();
}

pub use async_trait::async_trait;

/// Interface representing the HOPR server behavior for each incoming session instance
/// supplied as an argument.
#[cfg(feature = "session-server")]
#[async_trait::async_trait]
pub trait HoprSessionReactor {
    /// Fully process a single HOPR session
    async fn process(&self, session: HoprIncomingSession) -> errors::Result<()>;
}

/// An enum representing the current state of the HOPR node
#[atomic_enum::atomic_enum]
#[derive(PartialEq, Eq)]
pub enum HoprState {
    Uninitialized = 0,
    Initializing = 1,
    Indexing = 2,
    Starting = 3,
    Running = 4,
}

impl Display for HoprState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub struct OpenChannelResult {
    pub tx_hash: Hash,
    pub channel_id: Hash,
}

pub struct CloseChannelResult {
    pub tx_hash: Hash,
    pub status: ChannelStatus,
}

/// Enum differentiator for loop component futures.
///
/// Used to differentiate the type of the future that exits the loop premateruly
/// by tagging it as an enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::Display)]
pub enum HoprLibProcesses {
    #[strum(to_string = "transport: {0}")]
    Transport(HoprTransportProcess),
    #[cfg(feature = "session-server")]
    #[strum(to_string = "session server providing the exit node session stream functionality")]
    SessionServer,
    #[strum(to_string = "tick wake up the strategies to perform an action")]
    StrategyTick,
    #[strum(to_string = "initial indexing operation into the DB")]
    Indexing,
    #[strum(to_string = "processing of indexed operations in internal components")]
    IndexReflection,
    #[strum(to_string = "on-chain transaction queue component for outgoing transactions")]
    OutgoingOnchainActionQueue,
    #[strum(to_string = "flush operation of outgoing ticket indices to the DB")]
    TicketIndexFlush,
    #[strum(to_string = "on received ack ticket trigger")]
    OnReceivedAcknowledgement,
}

impl HoprLibProcesses {
    /// Identifies whether a loop is allowed to finish or should
    /// run indefinitely.
    pub fn can_finish(&self) -> bool {
        matches!(self, HoprLibProcesses::Indexing)
    }
}

impl From<HoprTransportProcess> for HoprLibProcesses {
    fn from(value: HoprTransportProcess) -> Self {
        HoprLibProcesses::Transport(value)
    }
}

/// Creates a pipeline that chains the indexer-generated data, processes them into
/// the individual components, and creates a filtered output stream that is fed into
/// the transport layer swarm.
///
/// * `event_stream` - represents the events generated by the indexer. If the Indexer is not synced, it will not
///   generate any events.
/// * `preloading_event_stream` - a stream used by the components to preload the data from the objects (db, channel
///   graph...)
#[allow(clippy::too_many_arguments)]
pub async fn chain_events_to_transport_events<StreamIn, Db>(
    event_stream: StreamIn,
    me_onchain: Address,
    db: Db,
    multi_strategy: Arc<MultiStrategy>,
    channel_graph: Arc<RwLock<hopr_path::channel_graph::ChannelGraph>>,
    indexer_action_tracker: Arc<IndexerActionTracker>,
) -> impl Stream<Item = PeerDiscovery> + Send + 'static
where
    Db: HoprDbAllOperations + Clone + Send + Sync + std::fmt::Debug + 'static,
    StreamIn: Stream<Item = SignificantChainEvent> + Send + 'static,
{
    Box::pin(event_stream.filter_map(move |event| {
        let db = db.clone();
        let multi_strategy = multi_strategy.clone();
        let channel_graph = channel_graph.clone();
        let indexer_action_tracker = indexer_action_tracker.clone();

        async move {
            let resolved = indexer_action_tracker.match_and_resolve(&event).await;
            if resolved.is_empty() {
                trace!(%event, "No indexer expectations resolved for the event");
            } else {
                debug!(count = resolved.len(), %event, "resolved indexer expectations");
            }

            match event.event_type {
                ChainEventType::Announcement{peer, address, multiaddresses} => {
                    let allowed = db
                        .is_allowed_in_network_registry(None, &address)
                        .await
                        .unwrap_or(false);

                    Some(vec![PeerDiscovery::Announce(peer, multiaddresses), if allowed {
                        PeerDiscovery::Allow(peer)
                    } else {
                        PeerDiscovery::Ban(peer)
                    }])
                }
                ChainEventType::ChannelOpened(channel) |
                ChainEventType::ChannelClosureInitiated(channel) |
                ChainEventType::ChannelClosed(channel) |
                ChainEventType::ChannelBalanceIncreased(channel, _) | // needed ?
                ChainEventType::ChannelBalanceDecreased(channel, _) | // needed ?
                ChainEventType::TicketRedeemed(channel, _) => {   // needed ?
                    let maybe_direction = channel.direction(&me_onchain);

                    let change = channel_graph
                        .write_arc()
                        .await
                        .update_channel(channel);

                    // Check if this is our own channel
                    if let Some(own_channel_direction) = maybe_direction {
                        if let Some(change_set) = change {
                            for channel_change in change_set {
                                let _ = hopr_strategy::strategy::SingularStrategy::on_own_channel_changed(
                                    &*multi_strategy,
                                    &channel,
                                    own_channel_direction,
                                    channel_change,
                                )
                                .await;
                            }
                        } else if channel.status == ChannelStatus::Open {
                            // Emit Opening event if the channel did not exist before in the graph
                            let _ = hopr_strategy::strategy::SingularStrategy::on_own_channel_changed(
                                &*multi_strategy,
                                &channel,
                                own_channel_direction,
                                ChannelChange::Status {
                                    left: ChannelStatus::Closed,
                                    right: ChannelStatus::Open,
                                },
                            )
                            .await;
                        }
                    }

                    None
                }
                ChainEventType::NetworkRegistryUpdate(address, allowed) => {
                    let packet_key = db.translate_key(None, address).await;
                    match packet_key {
                        Ok(pk) => {
                            if let Some(pk) = pk {
                                let offchain_key: Result<OffchainPublicKey, _> = pk.try_into();

                                if let Ok(offchain_key) = offchain_key {
                                    let peer_id = offchain_key.into();

                                    let res = match allowed {
                                        hopr_chain_types::chain_events::NetworkRegistryStatus::Allowed => PeerDiscovery::Allow(peer_id),
                                        hopr_chain_types::chain_events::NetworkRegistryStatus::Denied => PeerDiscovery::Ban(peer_id),
                                    };

                                    Some(vec![res])
                                } else {
                                    error!("Failed to unwrap as offchain key at this point");
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        Err(error) => {
                            error!(%error, "on_network_registry_node_allowed failed");
                            None
                        },
                    }
                }
                ChainEventType::NodeSafeRegistered(safe_address) =>  {
                    info!(%safe_address, "Node safe registered");
                    None
                }
            }
        }
    })
    .flat_map(stream::iter)
)
}

/// Represents the socket behavior of the hopr-lib spawned [`Hopr`] object.
///
/// Provides a read and write stream for Hopr socket recognized data formats.
pub struct HoprSocket {
    rx: UnboundedReceiver<ApplicationData>,
    tx: UnboundedSender<ApplicationData>,
}

impl Default for HoprSocket {
    fn default() -> Self {
        let (tx, rx) = unbounded::<ApplicationData>();
        Self { rx, tx }
    }
}

impl HoprSocket {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reader(self) -> UnboundedReceiver<ApplicationData> {
        self.rx
    }

    pub fn writer(&self) -> UnboundedSender<ApplicationData> {
        self.tx.clone()
    }
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
    me_chain: ChainKeypair,
    cfg: config::HoprLibConfig,
    state: Arc<AtomicHoprState>,
    transport_api: HoprTransport<HoprDb>,
    hopr_chain_api: HoprChain<HoprDb>,
    // objects that could be removed pending architectural cleanup ========
    db: HoprDb,
    chain_cfg: ChainNetworkConfig,
    channel_graph: Arc<RwLock<hopr_path::channel_graph::ChannelGraph>>,
    multistrategy: Arc<MultiStrategy>,
    rx_indexer_significant_events: async_channel::Receiver<SignificantChainEvent>,
}

impl Hopr {
    pub fn new(
        mut cfg: config::HoprLibConfig,
        me: &OffchainKeypair,
        me_onchain: &ChainKeypair,
    ) -> crate::errors::Result<Self> {
        let multiaddress: Multiaddr = (&cfg.host).try_into()?;

        let db_path: PathBuf = [&cfg.db.data, "db"].iter().collect();
        info!(path = ?db_path, "Initiating DB");

        if cfg.db.force_initialize {
            info!("Force cleaning up existing database");
            remove_dir_all(db_path.as_path()).map_err(|e| {
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

        let db_cfg = HoprDbConfig {
            create_if_missing: cfg.db.initialize,
            force_create: cfg.db.force_initialize,
            log_slow_queries: std::time::Duration::from_millis(150),
            surb_ring_buffer_size: std::env::var("HOPR_SURB_RB_SIZE")
                .ok()
                .and_then(|s| usize::from_str(&s).ok())
                .unwrap_or_else(|| HoprDbConfig::default().surb_ring_buffer_size),
        };
        let db = futures::executor::block_on(HoprDb::new(db_path.as_path(), me_onchain.clone(), db_cfg))?;

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

        let (tx_indexer_events, rx_indexer_events) = async_channel::unbounded::<SignificantChainEvent>();

        let channel_graph = Arc::new(RwLock::new(ChannelGraph::new(
            me_onchain.public().to_address(),
            ChannelGraphConfig::default(),
        )));

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
            channel_graph.clone(),
            my_multiaddresses,
        );

        let hopr_hopr_chain_api = hopr_chain_api::HoprChain::new(
            me_onchain.clone(),
            db.clone(),
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
            hopr_chain_indexer::IndexerConfig {
                start_block_number: resolved_environment.channel_contract_deploy_block as u64,
                fast_sync: cfg.chain.fast_sync,
                enable_logs_snapshot: cfg.chain.enable_logs_snapshot,
                logs_snapshot_url: cfg.chain.logs_snapshot_url.clone(),
                data_directory: cfg.db.data.clone(),
            },
            tx_indexer_events,
        )?;

        let multi_strategy = Arc::new(MultiStrategy::new(
            cfg.strategy.clone(),
            db.clone(),
            hopr_hopr_chain_api.actions_ref().clone(),
            hopr_transport_api.ticket_aggregator(),
        ));
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
            me_chain: me_onchain.clone(),
            cfg,
            state: Arc::new(AtomicHoprState::new(HoprState::Uninitialized)),
            transport_api: hopr_transport_api,
            hopr_chain_api: hopr_hopr_chain_api,
            db,
            chain_cfg: resolved_environment,
            channel_graph,
            multistrategy: multi_strategy,
            rx_indexer_significant_events: rx_indexer_events,
        })
    }

    fn error_if_not_in_state(&self, state: HoprState, error: String) -> errors::Result<()> {
        if self.status() == state {
            Ok(())
        } else {
            Err(HoprLibError::StatusError(HoprStatusError::NotThereYet(state, error)))
        }
    }

    pub fn status(&self) -> HoprState {
        self.state.load(Ordering::Relaxed)
    }

    pub fn version_coerced(&self) -> String {
        String::from(constants::APP_VERSION_COERCED)
    }

    pub fn version(&self) -> String {
        String::from(constants::APP_VERSION)
    }

    pub fn network(&self) -> String {
        self.cfg.chain.network.clone()
    }

    pub async fn get_balance<C: Currency + Send>(&self) -> errors::Result<Balance<C>> {
        Ok(self.hopr_chain_api.get_balance().await?)
    }

    pub async fn get_eligibility_status(&self) -> errors::Result<bool> {
        Ok(self.hopr_chain_api.get_eligibility_status().await?)
    }

    pub async fn get_safe_balance<C: Currency + Send>(&self) -> errors::Result<Balance<C>> {
        let safe_balance = self
            .hopr_chain_api
            .get_safe_balance(self.cfg.safe_module.safe_address)
            .await?;
        Ok(safe_balance)
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

    pub async fn run<#[cfg(feature = "session-server")] T: HoprSessionReactor + Clone + Send + 'static>(
        &self,
        #[cfg(feature = "session-server")] serve_handler: T,
    ) -> errors::Result<(HoprSocket, HashMap<HoprLibProcesses, AbortHandle>)> {
        self.error_if_not_in_state(
            HoprState::Uninitialized,
            "Cannot start the hopr node multiple times".into(),
        )?;

        info!(
            address = %self.me_onchain(), minimum_balance = %*SUGGESTED_NATIVE_BALANCE,
            "Node is not started, please fund this node",
        );

        let mut processes: HashMap<HoprLibProcesses, AbortHandle> = HashMap::new();

        wait_for_funds(
            self.me_onchain(),
            *MIN_NATIVE_BALANCE,
            Duration::from_secs(200),
            self.hopr_chain_api.rpc(),
        )
        .await?;

        info!("Starting the node...");

        self.state.store(HoprState::Initializing, Ordering::Relaxed);

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
        let network_min_ticket_price = self.hopr_chain_api.get_minimum_ticket_price().await?;

        let configured_ticket_price = self.cfg.protocol.outgoing_ticket_price;
        if configured_ticket_price.is_some_and(|c| c < network_min_ticket_price) {
            return Err(HoprLibError::ChainApi(HoprChainError::Api(format!(
                "configured outgoing ticket price is lower than the network minimum ticket price: \
                 {configured_ticket_price:?} < {network_min_ticket_price}"
            ))));
        }

        // Once we are able to query the chain,
        // check if the winning probability is configured correctly.
        let network_min_win_prob = self.hopr_chain_api.get_minimum_winning_probability().await?;
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

        // set safe and module addresses in the DB
        self.db
            .set_safe_info(
                None,
                SafeInfo {
                    safe_address: self.cfg.safe_module.safe_address,
                    module_address: self.cfg.safe_module.module_address,
                },
            )
            .await?;

        self.state.store(HoprState::Indexing, Ordering::Relaxed);

        let (mut indexer_peer_update_tx, indexer_peer_update_rx) = futures::channel::mpsc::unbounded::<PeerDiscovery>();

        let indexer_event_pipeline = chain_events_to_transport_events(
            self.rx_indexer_significant_events.clone(),
            self.me_onchain(),
            self.db.clone(),
            self.multistrategy.clone(),
            self.channel_graph.clone(),
            self.hopr_chain_api.action_state(),
        )
        .await;

        {
            // This has to happen before the indexing process starts in order to make sure that the pre-existing data is
            // properly populated into the transport mechanism before the synced data in the follow up process.
            info!("Syncing peer announcements and network registry updates from previous runs");
            let accounts = self.db.get_accounts(None, true).await?;
            for account in accounts.into_iter() {
                match account.entry_type {
                    AccountType::NotAnnounced => {}
                    AccountType::Announced { multiaddr, .. } => {
                        indexer_peer_update_tx
                            .send(PeerDiscovery::Announce(account.public_key.into(), vec![multiaddr]))
                            .await
                            .map_err(|e| {
                                HoprLibError::GeneralError(format!("Failed to send peer discovery announcement: {e}"))
                            })?;

                        let allow_status = if self
                            .db
                            .is_allowed_in_network_registry(None, &account.chain_addr)
                            .await?
                        {
                            PeerDiscovery::Allow(account.public_key.into())
                        } else {
                            PeerDiscovery::Ban(account.public_key.into())
                        };

                        indexer_peer_update_tx.send(allow_status).await.map_err(|e| {
                            HoprLibError::GeneralError(format!(
                                "Failed to send peer discovery network registry event: {e}"
                            ))
                        })?;
                    }
                }
            }
        }

        spawn(async move {
            indexer_event_pipeline
                .map(Ok)
                .forward(indexer_peer_update_tx)
                .await
                .expect("The index to transport event chain failed");
        });

        info!("Start the chain process and sync the indexer");
        for (id, proc) in self.hopr_chain_api.start().await?.into_iter() {
            let nid = match id {
                HoprChainProcess::Indexer => HoprLibProcesses::Indexing,
                HoprChainProcess::OutgoingOnchainActionQueue => HoprLibProcesses::OutgoingOnchainActionQueue,
            };
            processes.insert(nid, proc);
        }

        {
            // Show onboarding information
            let my_ethereum_address = self.me_onchain();
            let my_peer_id = self.me_peer_id();
            let my_version = crate::constants::APP_VERSION;

            while !self
                .db
                .clone()
                .is_allowed_in_network_registry(None, &my_ethereum_address)
                .await
                .unwrap_or(false)
            {
                info!(
                    "Once you become eligible to join the HOPR network, you can continue your onboarding by using the following URL: https://hub.hoprnet.org/staking/onboarding?HOPRdNodeAddressForOnboarding={}, or by manually entering the node address of your node on https://hub.hoprnet.org/.",
                    my_ethereum_address.to_hex()
                );

                sleep(ONBOARDING_INFORMATION_INTERVAL).await;

                info!(peer_id = %my_peer_id, address = %my_ethereum_address.to_hex(), version = &my_version, "Node information");
                info!("Node Ethereum address: {my_ethereum_address} <- put this into staking hub");
            }
        }

        // Check Safe-module status:
        // 1) if the node is already included into the module
        // 2) if the module is enabled in the safe
        // 3) if the safe is the owner of the module
        // if any of the conditions is not met, return error
        let safe_module_configuration = self
            .hopr_chain_api
            .rpc()
            .check_node_safe_module_status(self.me_onchain())
            .await
            .map_err(HoprChainError::Rpc)?;

        if !safe_module_configuration.should_pass() {
            error!(
                ?safe_module_configuration,
                "Something is wrong with the safe module configuration",
            );
            return Err(HoprLibError::ChainApi(HoprChainError::Api(format!(
                "Safe and module are not configured correctly {safe_module_configuration:?}",
            ))));
        }

        // Possibly register a node-safe pair to NodeSafeRegistry.
        // Following that, the connector is set to use safe tx variants.
        if can_register_with_safe(
            self.me_onchain(),
            self.cfg.safe_module.safe_address,
            self.hopr_chain_api.rpc(),
        )
        .await?
        {
            info!("Registering safe by node");

            if self.me_onchain() == self.cfg.safe_module.safe_address {
                return Err(HoprLibError::GeneralError("cannot self as staking safe address".into()));
            }

            if let Err(e) = self
                .hopr_chain_api
                .actions_ref()
                .register_safe_by_node(self.cfg.safe_module.safe_address)
                .await?
                .await
            {
                // Intentionally ignoring the errored state
                error!(error = %e, "Failed to register node with safe")
            }
        }

        if self.is_public() {
            // At this point the node is already registered with Safe, so
            // we can announce via Safe-compliant TX

            let multiaddresses_to_announce = self.transport_api.announceable_multiaddresses();

            // The announcement is intentionally not awaited until confirmation
            match self
                .hopr_chain_api
                .actions_ref()
                .announce(&multiaddresses_to_announce, &self.me)
                .await
            {
                Ok(_) => info!(?multiaddresses_to_announce, "Announcing node on chain",),
                Err(ChainActionsError::AlreadyAnnounced) => {
                    info!(multiaddresses_announced = ?multiaddresses_to_announce, "Node already announced on chain")
                }
                // If the announcement fails, we keep going to prevent the node from retrying
                // after restart.
                // Functionality is limited, and users must check the logs for errors.
                Err(e) => error!(error = %e, "Failed to transmit node announcement"),
            }
        }

        {
            // Sync key ids from indexed Accounts

            // Sync the Channel graph
            let channel_graph = self.channel_graph.clone();
            let mut cg = channel_graph.write_arc().await;

            info!("Syncing channels from the previous runs");
            let mut channel_stream = self
                .db
                .stream_active_channels()
                .await
                .map_err(hopr_db_sql::api::errors::DbError::from)?;

            while let Some(maybe_channel) = channel_stream.next().await {
                match maybe_channel {
                    Ok(channel) => {
                        cg.update_channel(channel);
                    }
                    Err(error) => error!(%error, "Failed to sync channel into the graph"),
                }
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
                .db
                .get_network_peers(Default::default(), false)
                .await?
                .filter(|status| futures::future::ready(status.quality >= min_quality_to_sync));

            while let Some(peer) = peer_stream.next().await {
                if let Some(ChainKey(key)) = self.db.translate_key(None, peer.id.0).await? {
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

        let socket = HoprSocket::new();
        let transport_output_tx = socket.writer();

        // notifier on acknowledged ticket reception
        let multi_strategy_ack_ticket = self.multistrategy.clone();
        let (on_ack_tkt_tx, mut on_ack_tkt_rx) = unbounded::<AcknowledgedTicket>();
        self.db.start_ticket_processing(Some(on_ack_tkt_tx))?;

        processes.insert(
            HoprLibProcesses::OnReceivedAcknowledgement,
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
            }),
        );

        let (session_tx, _session_rx) = unbounded::<IncomingSession>();

        #[cfg(feature = "session-server")]
        {
            processes.insert(
                HoprLibProcesses::SessionServer,
                hopr_async_runtime::spawn_as_abortable!(_session_rx.for_each_concurrent(None, move |session| {
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
                })),
            );
        }

        for (id, proc) in self
            .transport_api
            .run(
                &self.me_chain,
                join(&[&self.cfg.db.data, "tbf"])
                    .map_err(|e| HoprLibError::GeneralError(format!("Failed to construct the bloom filter: {e}")))?,
                transport_output_tx,
                indexer_peer_update_rx,
                session_tx,
            )
            .await?
            .into_iter()
        {
            processes.insert(id.into(), proc);
        }

        let db_clone = self.db.clone();
        processes.insert(
            HoprLibProcesses::TicketIndexFlush,
            hopr_async_runtime::spawn_as_abortable!(Box::pin(execute_on_tick(
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
            ))),
        );

        // NOTE: after the chain is synced, we can reset tickets which are considered
        // redeemed but on-chain state does not align with that. This implies there was a problem
        // right when the transaction was sent on-chain. In such cases, we simply let it retry and
        // handle errors appropriately.
        if let Err(e) = self.db.fix_channels_next_ticket_state().await {
            error!(error = %e, "failed to fix channels ticket states");
        }

        // NOTE: strategy ticks must start after the chain is synced, otherwise
        // the strategy would react to historical data and drain through the native
        // balance on chain operations not relevant for the present network state
        let multi_strategy = self.multistrategy.clone();
        let strategy_interval = self.cfg.strategy.execution_interval;
        processes.insert(
            HoprLibProcesses::StrategyTick,
            hopr_async_runtime::spawn_as_abortable!(async move {
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
                .await;
            }),
        );

        self.state.store(HoprState::Running, Ordering::Relaxed);

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

        Ok((socket, processes))
    }

    // p2p transport =========
    /// Own PeerId used in the libp2p transport layer
    pub fn me_peer_id(&self) -> PeerId {
        (*self.me.public()).into()
    }

    /// Get the list of all announced public nodes in the network
    pub async fn get_public_nodes(&self) -> errors::Result<Vec<(PeerId, Address, Vec<Multiaddr>)>> {
        Ok(self.transport_api.get_public_nodes().await?)
    }

    /// Returns the most recently indexed log, if any.
    pub async fn get_indexer_state(&self) -> errors::Result<IndexerStateInfo> {
        let indexer_state_info = self.db.get_indexer_state_info(None).await?;

        match self.db.get_last_checksummed_log().await? {
            Some(log) => {
                let checksum = match log.checksum {
                    Some(checksum) => Hash::from_hex(checksum.as_str())?,
                    None => Hash::default(),
                };
                Ok(IndexerStateInfo {
                    latest_log_block_number: log.block_number as u32,
                    latest_log_checksum: checksum,
                    ..indexer_state_info
                })
            }
            None => Ok(indexer_state_info),
        }
    }

    /// Test whether the peer with PeerId is allowed to access the network
    pub async fn is_allowed_to_access_network(
        &self,
        address_like: either::Either<&PeerId, Address>,
    ) -> errors::Result<bool> {
        Ok(self.transport_api.is_allowed_to_access_network(address_like).await?)
    }

    /// Ping another node in the network based on the PeerId
    ///
    /// Returns the RTT (round trip time), i.e. how long it took for the ping to return.
    pub async fn ping(&self, peer: &PeerId) -> errors::Result<(std::time::Duration, PeerStatus)> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        Ok(self.transport_api.ping(peer).await?)
    }

    /// Create a client session connection returning a session object that implements
    /// [`AsyncRead`] and [`AsyncWrite`] and can bu used as a read/write binary session.
    #[cfg(feature = "session-client")]
    pub async fn connect_to(
        &self,
        destination: Address,
        target: SessionTarget,
        cfg: SessionClientConfig,
    ) -> errors::Result<HoprSession> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

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
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;
        Ok(self.transport_api.probe_session(id).await?)
    }

    #[cfg(feature = "session-client")]
    pub async fn get_session_surb_balancer_config(
        &self,
        id: &HoprSessionId,
    ) -> errors::Result<Option<SurbBalancerConfig>> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;
        Ok(self.transport_api.session_surb_balancing_cfg(id).await?)
    }

    #[cfg(feature = "session-client")]
    pub async fn update_session_surb_balancer_config(
        &self,
        id: &HoprSessionId,
        cfg: SurbBalancerConfig,
    ) -> errors::Result<()> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;
        Ok(self.transport_api.update_session_surb_balancing_cfg(id, cfg).await?)
    }

    /// Send a message to another peer in the network
    ///
    /// @param msg message to send
    /// @param destination PeerId of the destination
    /// @param options optional configuration of the message path using hops and intermediatePath
    /// @param applicationTag optional tag identifying the sending application
    /// @returns ack challenge
    #[tracing::instrument(level = "debug", skip(self, msg))]
    pub async fn send_message(
        &self,
        msg: Box<[u8]>,
        routing: DestinationRouting,
        application_tag: Tag,
    ) -> errors::Result<()> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        self.transport_api.send_message(msg, routing, application_tag).await?;

        Ok(())
    }

    /// Attempts to aggregate all tickets in the given channel
    pub async fn aggregate_tickets(&self, channel: &Hash) -> errors::Result<()> {
        Ok(self.transport_api.aggregate_tickets(channel).await?)
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
        let key = match OffchainPublicKey::try_from(peer) {
            Ok(k) => k,
            Err(e) => {
                error!(%peer, error = %e, "failed to convert peer id to off-chain key");
                return vec![];
            }
        };

        match self.db.get_account(None, key).await {
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
    pub async fn network_peer_info(&self, peer: &PeerId) -> errors::Result<Option<hopr_transport::PeerStatus>> {
        Ok(self.transport_api.network_peer_info(peer).await?)
    }

    /// Get peers connected peers with quality higher than some value
    pub async fn all_network_peers(
        &self,
        minimum_quality: f64,
    ) -> errors::Result<Vec<(Option<Address>, PeerId, hopr_transport::PeerStatus)>> {
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
    /// Get all tickets in a channel specified by Hash
    pub async fn tickets_in_channel(&self, channel: &Hash) -> errors::Result<Option<Vec<AcknowledgedTicket>>> {
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
        Ok(self.db.reset_ticket_statistics().await?)
    }

    // DB ============
    pub fn peer_resolver(&self) -> &impl HoprDbResolverOperations {
        &self.db
    }

    // Chain =========
    pub fn me_onchain(&self) -> Address {
        self.hopr_chain_api.me_onchain()
    }

    /// Get ticket price
    pub async fn get_ticket_price(&self) -> errors::Result<Option<HoprBalance>> {
        Ok(self.hopr_chain_api.ticket_price().await?)
    }

    /// Get minimum incoming ticket winning probability
    pub async fn get_minimum_incoming_ticket_win_probability(&self) -> errors::Result<WinningProbability> {
        Ok(self
            .db
            .get_indexer_data(None)
            .await?
            .minimum_incoming_ticket_winning_prob)
    }

    /// List of all accounts announced on the chain
    pub async fn accounts_announced_on_chain(&self) -> errors::Result<Vec<AccountEntry>> {
        Ok(self.hopr_chain_api.accounts_announced_on_chain().await?)
    }

    /// Get the channel entry from Hash.
    /// @returns the channel entry of those two nodes
    pub async fn channel_from_hash(&self, channel_id: &Hash) -> errors::Result<Option<ChannelEntry>> {
        Ok(self.db.get_channel_by_id(None, channel_id).await?)
    }

    /// Get the channel entry between source and destination node.
    /// @param src Address
    /// @param dest Address
    /// @returns the channel entry of those two nodes
    pub async fn channel(&self, src: &Address, dest: &Address) -> errors::Result<ChannelEntry> {
        Ok(self.hopr_chain_api.channel(src, dest).await?)
    }

    /// List all channels open from a specified Address
    pub async fn channels_from(&self, src: &Address) -> errors::Result<Vec<ChannelEntry>> {
        Ok(self.hopr_chain_api.channels_from(src).await?)
    }

    /// List all channels open to a specified address
    pub async fn channels_to(&self, dest: &Address) -> errors::Result<Vec<ChannelEntry>> {
        Ok(self.hopr_chain_api.channels_to(dest).await?)
    }

    /// List all channels
    pub async fn all_channels(&self) -> errors::Result<Vec<ChannelEntry>> {
        Ok(self.hopr_chain_api.all_channels().await?)
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
    pub async fn withdraw_tokens(&self, recipient: Address, amount: HoprBalance) -> errors::Result<Hash> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let awaiter = self.hopr_chain_api.actions_ref().withdraw(recipient, amount).await?;

        Ok(awaiter.await?.tx_hash)
    }

    /// Withdraw on-chain native assets to a given address
    /// @param recipient the account where the assets should be transferred to
    /// @param amount how many tokens to be transferred
    pub async fn withdraw_native(&self, recipient: Address, amount: XDaiBalance) -> errors::Result<Hash> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let awaiter = self
            .hopr_chain_api
            .actions_ref()
            .withdraw_native(recipient, amount)
            .await?;

        Ok(awaiter.await?.tx_hash)
    }

    pub async fn open_channel(&self, destination: &Address, amount: HoprBalance) -> errors::Result<OpenChannelResult> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let awaiter = self
            .hopr_chain_api
            .actions_ref()
            .open_channel(*destination, amount)
            .await?;

        let channel_id = generate_channel_id(&self.hopr_chain_api.me_onchain(), destination);
        Ok(awaiter.await.map(|confirm| OpenChannelResult {
            tx_hash: confirm.tx_hash,
            channel_id,
        })?)
    }

    pub async fn fund_channel(&self, channel_id: &Hash, amount: HoprBalance) -> errors::Result<Hash> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let awaiter = self
            .hopr_chain_api
            .actions_ref()
            .fund_channel(*channel_id, amount)
            .await?;

        Ok(awaiter.await?.tx_hash)
    }

    pub async fn close_channel(
        &self,
        counterparty: &Address,
        direction: ChannelDirection,
        redeem_before_close: bool,
    ) -> errors::Result<CloseChannelResult> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let confirmation = self
            .hopr_chain_api
            .actions_ref()
            .close_channel(*counterparty, direction, redeem_before_close)
            .await?
            .await?;

        match confirmation
            .event
            .expect("channel close action confirmation must have associated chain event")
        {
            ChainEventType::ChannelClosureInitiated(c) => Ok(CloseChannelResult {
                tx_hash: confirmation.tx_hash,
                status: c.status, // copy the information about closure time
            }),
            ChainEventType::ChannelClosed(_) => Ok(CloseChannelResult {
                tx_hash: confirmation.tx_hash,
                status: ChannelStatus::Closed,
            }),
            _ => Err(HoprLibError::GeneralError("close channel transaction failed".into())),
        }
    }

    pub async fn close_channel_by_id(
        &self,
        channel_id: Hash,
        redeem_before_close: bool,
    ) -> errors::Result<CloseChannelResult> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        match self.channel_from_hash(&channel_id).await? {
            Some(channel) => match channel.orientation(&self.me_onchain()) {
                Some((direction, counterparty)) => {
                    self.close_channel(&counterparty, direction, redeem_before_close).await
                }
                None => Err(HoprLibError::ChainError(ChainActionsError::InvalidArguments(
                    "cannot close channel that is not own".into(),
                ))),
            },
            None => Err(HoprLibError::ChainError(ChainActionsError::ChannelDoesNotExist)),
        }
    }

    pub async fn get_channel_closure_notice_period(&self) -> errors::Result<Duration> {
        Ok(self.hopr_chain_api.get_channel_closure_notice_period().await?)
    }

    pub async fn redeem_all_tickets(&self, only_aggregated: bool) -> errors::Result<()> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        // We do not await the on-chain confirmation
        self.hopr_chain_api
            .actions_ref()
            .redeem_all_tickets(only_aggregated)
            .await?;

        Ok(())
    }

    pub async fn redeem_tickets_with_counterparty(
        &self,
        counterparty: &Address,
        only_aggregated: bool,
    ) -> errors::Result<()> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        // We do not await the on-chain confirmation
        let _ = self
            .hopr_chain_api
            .actions_ref()
            .redeem_tickets_with_counterparty(counterparty, only_aggregated)
            .await?;

        Ok(())
    }

    pub async fn redeem_tickets_in_channel(&self, channel_id: &Hash, only_aggregated: bool) -> errors::Result<usize> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let channel = self.db.get_channel_by_id(None, channel_id).await?;
        let mut redeem_count = 0;

        if let Some(channel) = channel {
            if channel.destination == self.hopr_chain_api.me_onchain() {
                // We do not await the on-chain confirmation
                redeem_count = self
                    .hopr_chain_api
                    .actions_ref()
                    .redeem_tickets_in_channel(&channel, only_aggregated)
                    .await?
                    .len();
            }
        }

        Ok(redeem_count)
    }

    pub async fn redeem_ticket(&self, ack_ticket: AcknowledgedTicket) -> errors::Result<()> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        // We do not await the on-chain confirmation
        #[allow(clippy::let_underscore_future)]
        let _ = self.hopr_chain_api.actions_ref().redeem_ticket(ack_ticket).await?;

        Ok(())
    }

    pub async fn peerid_to_chain_key(&self, peer_id: &PeerId) -> errors::Result<Option<Address>> {
        let pk = hopr_transport::OffchainPublicKey::try_from(peer_id)?;
        Ok(self.db.resolve_chain_key(&pk).await?)
    }

    pub async fn chain_key_to_peerid(&self, address: &Address) -> errors::Result<Option<PeerId>> {
        Ok(self
            .db
            .resolve_packet_key(address)
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
}
