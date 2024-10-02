//! HOPR library creating a unified [`Hopr`] object that can be used on its own,
//! as well as integrated into other systems and libraries.
//!
//! The [`Hopr`] object is standalone, meaning that once it is constructed and run,
//! it will perform its functionality autonomously. The API it offers serves as a
//! high level integration point for other applications and utilities, but offers
//! a complete and fully featured HOPR node stripped from top level functionality
//! such as the REST API, key management...
//!
//! The intended way to use hopr_lib is for a specific tool to be built on top of it,
//! should the default `hoprd` implementation not be acceptable.
//!
//! For most of the practical use cases, the `hoprd` application should be a preferable
//! choice.

/// Configuration related public types
pub mod config;
/// Various public constants.
pub mod constants;
/// Enumerates all errors thrown from this library.
pub mod errors;

use async_lock::RwLock;
use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    Stream, StreamExt,
};
use futures_concurrency::stream::StreamExt as _;
use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Arc},
    time::Duration,
};
use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};
use tracing::{debug, error, info, warn};

use chain_actions::{
    action_state::{ActionState, IndexerActionTracker},
    channels::ChannelActions,
    node::NodeActions,
    redeem::TicketRedeemActions,
};
use chain_api::{
    can_register_with_safe, config::ChainNetworkConfig, wait_for_funds, HoprChain, HoprChainProcess,
    SignificantChainEvent,
};
use chain_types::chain_events::ChainEventType;
use chain_types::ContractAddresses;
use core_path::channel_graph::ChannelGraph;
use errors::HoprStatusError;
use hopr_async_runtime::prelude::{sleep, spawn, JoinHandle};
use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_db_api::logs::HoprDbLogOperations;
use hopr_db_sql::{
    accounts::HoprDbAccountOperations,
    api::{info::SafeInfo, resolver::HoprDbResolverOperations, tickets::HoprDbTicketOperations},
    channels::HoprDbChannelOperations,
    db::{HoprDb, HoprDbConfig},
    info::HoprDbInfoOperations,
    prelude::{ChainOrPacketKey::ChainKey, DbSqlError, HoprDbPeersOperations},
    HoprDbAllOperations, HoprDbGeneralModelOperations,
};
use hopr_platform::file::native::{join, remove_dir_all};
use hopr_strategy::strategy::{MultiStrategy, SingularStrategy};
use hopr_transport::{
    execute_on_tick, ChainKeypair, Hash, HoprTransport, HoprTransportConfig, HoprTransportProcess, IncomingSession,
    IndexerTransportEvent, Network, OffchainKeypair, PeerDiscovery, PeerEligibility, PeerOrigin, PeerStatus,
};
pub use {
    chain_actions::errors::ChainActionsError,
    chain_api::config::{
        Addresses as NetworkContractAddresses, EnvironmentType, Network as ChainNetwork, ProtocolsConfig,
    },
    hopr_internal_types::prelude::*,
    hopr_network_types::prelude::{IpProtocol, RoutingOptions},
    hopr_primitive_types::prelude::*,
    hopr_primitive_types::rlp,
    hopr_strategy::Strategy,
    hopr_transport::{
        config::{looks_like_domain, HostConfig, HostType},
        constants::RESERVED_TAG_UPPER_LIMIT,
        errors::{HoprTransportError, NetworkingError, ProtocolError},
        libp2p::identity::PeerId,
        ApplicationData, HalfKeyChallenge, Health, IncomingSession as HoprIncomingSession, Keypair, Multiaddr,
        OffchainKeypair as HoprOffchainKeypair, SendMsg, Session as HoprSession, SessionCapability,
        SessionClientConfig, SessionId as HoprSessionId, SessionTarget, TicketStatistics, SESSION_USABLE_MTU_SIZE,
    },
};

#[cfg(feature = "runtime-tokio")]
pub use hopr_transport::transfer_session;

use crate::constants::{MIN_NATIVE_BALANCE, ONBOARDING_INFORMATION_INTERVAL, SUGGESTED_NATIVE_BALANCE};
use crate::{config::SafeModule, errors::HoprLibError};

#[cfg(all(feature = "prometheus", not(test)))]
use {
    hopr_metrics::metrics::{MultiGauge, SimpleGauge},
    hopr_platform::time::native::current_time,
    std::str::FromStr,
};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_PROCESS_START_TIME: SimpleGauge = SimpleGauge::new(
        "hopr_up",
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
        write!(f, "{:?}", self)
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
    #[strum(to_string = "libp2p component responsible for the handling of the p2p communication")]
    Swarm,
    #[strum(to_string = "HOPR protocol processing: ack ingress")]
    ProtocolAckIn,
    #[strum(to_string = "HOPR protocol processing: ack egress")]
    ProtocolAckOut,
    #[strum(to_string = "HOPR protocol processing: msg ingress")]
    ProtocolMsgIn,
    #[strum(to_string = "HOPR protocol processing: msg egress")]
    ProtocolMsgOut,
    #[strum(to_string = "session router pairing the session streams based on the PeerId and ApplicationTag")]
    SessionsRouter,
    #[strum(to_string = "graceful session on stream close")]
    SessionsTerminator,
    #[strum(to_string = "gracefully session terminator upon expiration")]
    SessionsExpiration,
    #[cfg(feature = "session-server")]
    #[strum(to_string = "session server providing the exit node session stream functionality")]
    SessionServer,
    #[strum(to_string = "heartbeat component responsible for maintaining the network quality measurements")]
    Heartbeat,
    #[strum(to_string = "tick wake up the strategies to perform an action")]
    StrategyTick,
    #[strum(to_string = "save operation for the bloom filter")]
    BloomFilterSave,
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
        match value {
            hopr_transport::HoprTransportProcess::Swarm => HoprLibProcesses::Swarm,
            hopr_transport::HoprTransportProcess::ProtocolAckIn => HoprLibProcesses::ProtocolAckIn,
            hopr_transport::HoprTransportProcess::ProtocolAckOut => HoprLibProcesses::ProtocolAckOut,
            hopr_transport::HoprTransportProcess::ProtocolMsgIn => HoprLibProcesses::ProtocolMsgIn,
            hopr_transport::HoprTransportProcess::ProtocolMsgOut => HoprLibProcesses::ProtocolMsgOut,
            hopr_transport::HoprTransportProcess::Heartbeat => HoprLibProcesses::Heartbeat,
            hopr_transport::HoprTransportProcess::SessionsManagement => HoprLibProcesses::SessionsRouter,
            hopr_transport::HoprTransportProcess::SessionsTerminator => HoprLibProcesses::SessionsTerminator,
            hopr_transport::HoprTransportProcess::SessionsExpiration => HoprLibProcesses::SessionsExpiration,
            hopr_transport::HoprTransportProcess::BloomFilterSave => HoprLibProcesses::BloomFilterSave,
        }
    }
}

/// Creates a pipeline that chains the indexer-generated data, processes them into
/// the individual components and creates a filtered output stream that is fed into
/// the transport layer swarm.
///
/// * `event_stream` - represents the events generated by the indexer.
///   If the Indexer is not synced, it will not generate any events.
/// * `preloading_event_stream` - a stream used by the components to preload the data from the objects (db, channel graph...)
#[allow(clippy::too_many_arguments)] // TODO: refactor this function into a reasonable group of components once fully rearchitected
pub async fn chain_events_to_transport_events<StreamIn, StreamInPreloading, Db>(
    event_stream: StreamIn,
    preloading_event_stream: StreamInPreloading,
    me: PeerId,
    me_onchain: Address,
    db: Db,
    multi_strategy: Arc<MultiStrategy>,
    channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
    indexer_action_tracker: Arc<IndexerActionTracker>,
    network: Arc<Network<Db>>,
) -> impl Stream<Item = PeerDiscovery> + Send + 'static
where
    Db: HoprDbAllOperations + Clone + Send + Sync + std::fmt::Debug + 'static,
    StreamIn: Stream<Item = SignificantChainEvent> + Send + 'static,
    StreamInPreloading: Stream<Item = IndexerTransportEvent> + Send + 'static,
{
    let network_clone = network.clone();

    Box::pin(event_stream.filter_map(move |event| {
        let db = db.clone();
        let multi_strategy = multi_strategy.clone();
        let channel_graph = channel_graph.clone();
        let indexer_action_tracker = indexer_action_tracker.clone();
        let network = network.clone();

        async move {

        let resolved = indexer_action_tracker.match_and_resolve(&event).await;
        debug!("resolved {} indexer expectations in {event}", resolved.len());

        match event.event_type {
                ChainEventType::Announcement{peer, address, multiaddresses} => {
                    if peer != me {
                        // decapsulate the `p2p/<peer_id>` to remove duplicities
                        let mas = multiaddresses
                            .into_iter()
                            .map(|ma| hopr_transport::strip_p2p_protocol(&ma))
                            .filter(|v| !v.is_empty())
                            .collect::<Vec<_>>();

                        if ! mas.is_empty() {
                            if let Err(e) = network.add(&peer, PeerOrigin::NetworkRegistry, mas.clone()).await
                            {
                                error!("failed to record '{peer}' from the NetworkRegistry: {e}");
                            }

                            if db
                                .is_allowed_in_network_registry(None, address)
                                .await
                                .unwrap_or(false)
                            {
                                Some(vec![IndexerTransportEvent::Announce(peer, mas), IndexerTransportEvent::EligibilityUpdate(peer, PeerEligibility::Eligible)])
                            } else {
                                Some(vec![IndexerTransportEvent::Announce(peer, mas)])
                            }
                        } else { None }
                    } else {
                        debug!("Skipping announcements for myself ({peer})");
                        None
                    }
                }
                ChainEventType::ChannelOpened(channel) |
                ChainEventType::ChannelClosureInitiated(channel) |
                ChainEventType::ChannelClosed(channel) |
                ChainEventType::ChannelBalanceIncreased(channel, _) | // needed ?
                ChainEventType::ChannelBalanceDecreased(channel, _) | // needed ?
                ChainEventType::TicketRedeemed(channel, _) => {   // needed ?
                    let maybe_direction = channel.direction(&me_onchain);

                    let change = channel_graph
                        .write()
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
                                let offchain_key: OffchainPublicKey = pk.try_into().expect("must be an offchain key at this point");
                                let peer_id = offchain_key.into();

                                match allowed {
                                    chain_types::chain_events::NetworkRegistryStatus::Allowed => {
                                        if let Err(e) = network.add(&peer_id, PeerOrigin::NetworkRegistry, vec![]).await {
                                            error!(peer = %peer_id, "failed to allow locally (already allowed on-chain): {e}")
                                        }
                                    },
                                    chain_types::chain_events::NetworkRegistryStatus::Denied => {
                                        if let Err(e) = network.remove(&peer_id).await {
                                            error!(peer = %peer_id, "failed to ban locally (already banned on-chain): {e}")
                                        }
                                    },
                                };

                            Some(vec![IndexerTransportEvent::EligibilityUpdate(
                                peer_id,
                                allowed.clone().into()
                            )])
                            } else { None }

                        }
                        Err(e) => {
                            error!("on_network_registry_node_allowed failed with: {e}");
                            None
                        },
                    }
                }
                ChainEventType::NodeSafeRegistered(safe_address) =>  {
                    info!("node safe registered {safe_address}");
                    None
                }
            }
    }})
    .flat_map(futures::stream::iter))
    // merge the indexer source with the init source
    .merge(preloading_event_stream)
    .filter_map(move |event| {
        let network = network_clone.clone();

        async move {
            match event {
                IndexerTransportEvent::EligibilityUpdate(peer, eligibility) => match eligibility {
                    PeerEligibility::Eligible => Some(vec![PeerDiscovery::Allow(peer)]),
                    PeerEligibility::Ineligible => {
                        if let Err(e) = network.remove(&peer).await {
                            error!("failed to remove '{peer}' from the local registry: {e}")
                        }
                        Some(vec![PeerDiscovery::Ban(peer)])
                    }
                },
                IndexerTransportEvent::Announce(peer, multiaddress) => {
                    Some(vec![PeerDiscovery::Announce(peer, multiaddress)])
                }
            }
        }
    })
    .flat_map(futures::stream::iter)
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
    // onchain keypair is necessary, because the ack interaction needs it to be constructable in runtime
    me_onchain: ChainKeypair,
    cfg: config::HoprLibConfig,
    state: Arc<AtomicHoprState>,
    transport_api: HoprTransport<HoprDb>,
    chain_api: HoprChain<HoprDb>,
    // objects that could be removed pending architectural cleanup ========
    db: HoprDb,
    chain_cfg: ChainNetworkConfig,
    channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
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
        info!("Initiating the DB at '{db_path:?}'");

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
        };
        let db = futures::executor::block_on(HoprDb::new(db_path.as_path(), me_onchain.clone(), db_cfg))?;

        if let Some(provider) = &cfg.chain.provider {
            info!("Creating chain components using the custom provider: {provider}");
        } else {
            info!("Creating chain components using the default provider");
        }
        let resolved_environment = chain_api::config::ChainNetworkConfig::new(
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
            contract_addresses = tracing::field::debug(contract_addresses),
            "Resolved contract addresses",
        );

        let my_multiaddresses = vec![multiaddress];

        let (tx_indexer_events, rx_indexer_events) = async_channel::unbounded::<SignificantChainEvent>();

        let channel_graph = Arc::new(RwLock::new(ChannelGraph::new(me_onchain.public().to_address())));

        let hopr_transport_api = HoprTransport::new(
            me,
            me_onchain,
            HoprTransportConfig {
                transport: cfg.transport.clone(),
                network: cfg.network_options.clone(),
                protocol: cfg.protocol,
                heartbeat: cfg.heartbeat,
            },
            db.clone(),
            channel_graph.clone(),
            my_multiaddresses,
        );

        let hopr_chain_api = chain_api::HoprChain::new(
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
            chain_indexer::IndexerConfig {
                start_block_number: resolved_environment.channel_contract_deploy_block as u64,
                fast_sync: cfg.chain.fast_sync,
            },
            tx_indexer_events,
        );

        let multi_strategy = Arc::new(MultiStrategy::new(
            cfg.strategy.clone(),
            db.clone(),
            hopr_chain_api.actions_ref().clone(),
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
                error!("failed to initialize ticket statistics metrics: {e}");
            }
        }

        Ok(Self {
            me: me.clone(),
            me_onchain: me_onchain.clone(),
            cfg,
            state: Arc::new(AtomicHoprState::new(HoprState::Uninitialized)),
            transport_api: hopr_transport_api,
            chain_api: hopr_chain_api,
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
            Err(errors::HoprLibError::StatusError(HoprStatusError::NotThereYet(
                state, error,
            )))
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

    pub async fn get_balance(&self, balance_type: BalanceType) -> errors::Result<Balance> {
        Ok(self.chain_api.get_balance(balance_type).await?)
    }

    pub async fn get_eligibility_status(&self) -> errors::Result<bool> {
        Ok(self.chain_api.get_eligibility_status().await?)
    }

    pub async fn get_safe_balance(&self, balance_type: BalanceType) -> errors::Result<Balance> {
        let safe_balance = self
            .chain_api
            .get_safe_balance(self.cfg.safe_module.safe_address, balance_type)
            .await?;

        if balance_type == BalanceType::HOPR {
            let my_db = self.db.clone();
            self.db
                .begin_transaction()
                .await?
                .perform(|tx| {
                    Box::pin(async move {
                        let db_safe_balance = my_db.get_safe_hopr_balance(Some(tx)).await?;
                        if safe_balance != db_safe_balance {
                            warn!(
                                "Safe balance in the DB {db_safe_balance} mismatches on chain balance: {safe_balance}"
                            );
                            my_db.set_safe_hopr_balance(Some(tx), safe_balance).await?;
                        }
                        Ok::<_, DbSqlError>(())
                    })
                })
                .await?;
        }
        Ok(safe_balance)
    }

    pub fn get_safe_config(&self) -> SafeModule {
        self.cfg.safe_module.clone()
    }

    pub fn chain_config(&self) -> ChainNetworkConfig {
        self.chain_cfg.clone()
    }

    #[inline]
    fn is_public(&self) -> bool {
        self.cfg.chain.announce
    }

    pub async fn run<#[cfg(feature = "session-server")] T: HoprSessionReactor + Clone + Send + 'static>(
        &self,
        #[cfg(feature = "session-server")] serve_handler: T,
    ) -> errors::Result<(HoprSocket, HashMap<HoprLibProcesses, JoinHandle<()>>)> {
        self.error_if_not_in_state(
            HoprState::Uninitialized,
            "Cannot start the hopr node multiple times".into(),
        )?;

        info!(
            "Node is not started, please fund this node {} with at least {}",
            self.me_onchain(),
            Balance::new_from_str(SUGGESTED_NATIVE_BALANCE, BalanceType::Native).to_formatted_string()
        );

        let mut processes: HashMap<HoprLibProcesses, JoinHandle<()>> = HashMap::new();

        wait_for_funds(
            self.me_onchain(),
            Balance::new_from_str(MIN_NATIVE_BALANCE, BalanceType::Native),
            Duration::from_secs(200),
            self.chain_api.rpc(),
        )
        .await?;

        info!("Starting hopr node...");

        self.state.store(HoprState::Initializing, Ordering::Relaxed);

        let balance = self.get_balance(BalanceType::Native).await?;

        let minimum_balance = Balance::new_from_str(constants::MIN_NATIVE_BALANCE, BalanceType::Native);

        info!(
            "Ethereum account {} has {}. Minimum balance is {}",
            self.chain_api.me_onchain(),
            balance.to_formatted_string(),
            minimum_balance.to_formatted_string()
        );

        if balance.le(&minimum_balance) {
            return Err(errors::HoprLibError::GeneralError(
                "Cannot start the node without a sufficiently funded wallet".to_string(),
            ));
        }

        info!("Linking chain and packet keys");
        self.db
            .insert_account(
                None,
                AccountEntry {
                    public_key: *self.me.public(),
                    chain_addr: self.chain_api.me_onchain(),
                    // Will be set once we announce ourselves and Indexer processes the announcement
                    entry_type: AccountType::NotAnnounced,
                },
            )
            .await?;

        self.state.store(HoprState::Indexing, Ordering::Relaxed);

        let (to_process_tx, to_process_rx) = async_channel::unbounded::<IndexerTransportEvent>();

        let (indexer_peer_update_tx, indexer_peer_update_rx) = futures::channel::mpsc::unbounded::<PeerDiscovery>();

        let indexer_event_pipeline = chain_events_to_transport_events(
            self.rx_indexer_significant_events.clone(),
            to_process_rx,
            self.me_peer_id(),
            self.me_onchain(),
            self.db.clone(),
            self.multistrategy.clone(),
            self.channel_graph.clone(),
            self.chain_api.action_state(),
            self.transport_api.network(),
        )
        .await;

        spawn(async move {
            indexer_event_pipeline
                .map(Ok)
                .forward(indexer_peer_update_tx)
                .await
                .expect("The index to transport event chain failed");
        });

        info!("Start the chain process and sync the indexer");
        for (id, proc) in self.chain_api.start().await?.into_iter() {
            let nid = match id {
                HoprChainProcess::Indexer => HoprLibProcesses::Indexing,
                HoprChainProcess::OutgoingOnchainActionQueue => HoprLibProcesses::OutgoingOnchainActionQueue,
            };
            processes.insert(nid, proc);
        }

        {
            // Show onboarding information
            let my_ethereum_address = self.me_onchain().to_hex();
            let my_peer_id = self.me_peer_id();
            let my_version = crate::constants::APP_VERSION;

            while !self.is_allowed_to_access_network(&my_peer_id).await.unwrap_or(false) {
                info!("Once you become eligible to join the HOPR network, you can continue your onboarding by using the following URL: https://hub.hoprnet.org/staking/onboarding?HOPRdNodeAddressForOnboarding={my_ethereum_address}, or by manually entering the node address of your node on https://hub.hoprnet.org/.");

                sleep(ONBOARDING_INFORMATION_INTERVAL).await;

                info!("Node information: peerID => {my_peer_id}, Ethereum address => {my_ethereum_address}, version => {my_version}");
                info!("Node Ethereum address: {my_ethereum_address} <- put this into staking hub");
            }
        }
        // TODO: wait here for the confirmation that the node is allowed in the registry

        info!("Loading initial peers from the storage");
        for (peer, _address, multiaddresses) in self.transport_api.get_public_nodes().await? {
            if self.is_allowed_to_access_network(&peer).await? {
                debug!("Using initial public node '{peer}' with '{:?}'", multiaddresses);
                if let Err(e) = to_process_tx
                    .send(IndexerTransportEvent::EligibilityUpdate(
                        peer,
                        PeerEligibility::Eligible,
                    ))
                    .await
                {
                    error!("Failed to send index update event to transport: {e}");
                }

                if let Err(e) = to_process_tx
                    .send(IndexerTransportEvent::Announce(peer, multiaddresses.clone()))
                    .await
                {
                    error!("Failed to send index update event to transport: {e}");
                }

                // Self-reference is not needed in the network storage
                if &peer != self.transport_api.me() {
                    if let Err(e) = self
                        .transport_api
                        .network()
                        .add(&peer, PeerOrigin::Initialization, multiaddresses)
                        .await
                    {
                        error!("Failed to store the peer observation: {e}");
                    }
                }
            }
        }

        // Possibly register node-safe pair to NodeSafeRegistry. Following that the
        // connector is set to use safe tx variants.
        if can_register_with_safe(
            self.me_onchain(),
            self.cfg.safe_module.safe_address,
            self.chain_api.rpc(),
        )
        .await?
        {
            info!("Registering safe by node");

            if self.me_onchain() == self.cfg.safe_module.safe_address {
                return Err(errors::HoprLibError::GeneralError(
                    "cannot self as staking safe address".into(),
                ));
            }

            if let Err(e) = self
                .chain_api
                .actions_ref()
                .register_safe_by_node(self.cfg.safe_module.safe_address)
                .await?
                .await
            {
                // Intentionally ignoring the errored state
                error!("Failed to register node with safe: {e}")
            }
        }

        self.db
            .set_safe_info(
                None,
                SafeInfo {
                    safe_address: self.cfg.safe_module.safe_address,
                    module_address: self.cfg.safe_module.module_address,
                },
            )
            .await?;

        if self.is_public() {
            // At this point the node is already registered with Safe, so
            // we can announce via Safe-compliant TX

            let multiaddresses_to_announce = self.transport_api.announceable_multiaddresses();

            // The announcement is intentionally not awaited until confirmation
            match self
                .chain_api
                .actions_ref()
                .announce(&multiaddresses_to_announce, &self.me)
                .await
            {
                Ok(_) => info!("Announcing node on chain: {:?}", multiaddresses_to_announce),
                Err(ChainActionsError::AlreadyAnnounced) => {
                    info!("Node already announced on chain as {:?}", multiaddresses_to_announce)
                }
                // If the announcement fails, we keep going to prevent the node from retrying
                // after restart.
                // Functionality is limited, and users must check the logs for errors.
                Err(e) => error!("Failed to transmit node announcement: {e}"),
            }
        }

        {
            let channel_graph = self.channel_graph.clone();
            let mut cg = channel_graph.write().await;

            info!("Syncing channels from the previous runs");
            let channels = self.db.get_all_channels(None).await?;

            cg.sync_channels(channels).map_err(|e| {
                HoprLibError::GeneralError(format!("failed to initialize channel graph from the DB: {e}"))
            })?;

            // Sync all the qualities there too
            let mut peer_stream = self
                .db
                .get_network_peers(Default::default(), false)
                .await
                .map_err(hopr_db_sql::api::errors::DbError::from)?;
            while let Some(peer) = peer_stream.next().await {
                if let Some(ChainKey(key)) = self.db.translate_key(None, peer.id.0).await? {
                    cg.update_channel_quality(self.me_onchain(), key, peer.get_quality());
                } else {
                    error!("could not translate peer info: {}", peer.id.1);
                }
            }
        }

        let socket = HoprSocket::new();
        let transport_output_tx = socket.writer();

        // notifier on acknowledged ticket reception
        let multi_strategy_ack_ticket = self.multistrategy.clone();
        let (on_ack_tkt_tx, mut on_ack_tkt_rx) = unbounded::<AcknowledgedTicket>();
        processes.insert(
            HoprLibProcesses::OnReceivedAcknowledgement,
            spawn(async move {
                while let Some(ack) = on_ack_tkt_rx.next().await {
                    let _ = hopr_strategy::strategy::SingularStrategy::on_acknowledged_winning_ticket(
                        &*multi_strategy_ack_ticket,
                        &ack,
                    )
                    .await;
                }
            }),
        );

        let (session_tx, _session_rx) = unbounded::<IncomingSession>();

        #[cfg(feature = "session-server")]
        {
            processes.insert(
                HoprLibProcesses::SessionServer,
                spawn(_session_rx.for_each_concurrent(None, move |session| {
                    let serve_handler = serve_handler.clone();
                    async move {
                        let session_id = *session.session.id();
                        match serve_handler.process(session).await {
                            Ok(_) => debug!(
                                session_id = tracing::field::debug(session_id),
                                "client session processed successfully"
                            ),
                            Err(e) => error!(
                                session_id = tracing::field::debug(session_id),
                                "client session {session_id} processing failed: {e}"
                            ),
                        }
                    }
                })),
            );
        }

        for (id, proc) in self
            .transport_api
            .run(
                &self.me,
                &self.me_onchain,
                String::from(constants::APP_VERSION),
                self.transport_api.network(),
                join(&[&self.cfg.db.data, "tbf"]).map_err(|e| {
                    errors::HoprLibError::GeneralError(format!("Failed to construct the bloom filter: {e}"))
                })?,
                transport_output_tx,
                on_ack_tkt_tx,
                indexer_peer_update_rx,
                session_tx,
            )
            .await
            .into_iter()
        {
            processes.insert(id.into(), proc);
        }

        let db_clone = self.db.clone();
        processes.insert(
            HoprLibProcesses::TicketIndexFlush,
            spawn(Box::pin(execute_on_tick(Duration::from_secs(5), move || {
                let db_clone = db_clone.clone();
                async move {
                    match db_clone.persist_outgoing_ticket_indices().await {
                        Ok(n) => debug!("successfully flushed states of {} outgoing ticket indices", n),
                        Err(e) => error!("failed to flush ticket indices: {e}"),
                    }
                }
            }))),
        );

        // NOTE: strategy ticks must start after the chain is synced, otherwise
        // the strategy would react to historical data and drain through the native
        // balance on chain operations not relevant for the present network state
        let multi_strategy = self.multistrategy.clone();
        let strategy_interval = self.cfg.strategy.execution_interval;
        processes.insert(
            HoprLibProcesses::StrategyTick,
            spawn(async move {
                execute_on_tick(Duration::from_secs(strategy_interval), move || {
                    let multi_strategy = multi_strategy.clone();

                    async move {
                        debug!(state = "started", "strategy tick");
                        let _ = multi_strategy.on_tick().await;
                        debug!(state = "finished", "strategy tick");
                    }
                })
                .await;
            }),
        );

        self.state.store(HoprState::Running, Ordering::Relaxed);

        info!(
            id = self.transport_api.me().to_string(),
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

    /// Gets the current indexer state: last indexed block ID and checksum
    pub async fn get_indexer_state(&self) -> errors::Result<Option<SerializableLog>> {
        Ok(self.db.get_last_checksummed_log().await?)
    }

    /// Test whether the peer with PeerId is allowed to access the network
    pub async fn is_allowed_to_access_network(&self, peer: &PeerId) -> errors::Result<bool> {
        Ok(self.transport_api.is_allowed_to_access_network(peer).await?)
    }

    /// Ping another node in the network based on the PeerId
    pub async fn ping(&self, peer: &PeerId) -> errors::Result<(std::time::Duration, PeerStatus)> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        Ok(self.transport_api.ping(peer).await?)
    }

    #[cfg(feature = "session-client")]
    pub async fn connect_to(&self, cfg: SessionClientConfig) -> errors::Result<HoprSession> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        Ok(self.transport_api.new_session(cfg).await?)
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
        destination: PeerId,
        options: RoutingOptions,
        application_tag: Option<u16>,
    ) -> errors::Result<()> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        self.transport_api
            .send_message(msg, destination, options, application_tag)
            .await?;

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
                error!("failed to convert peer id {peer} to off-chain key: {e}");
                return vec![];
            }
        };

        match self.db.get_account(None, key).await {
            Ok(Some(entry)) => Vec::from_iter(entry.get_multiaddr()),
            Ok(None) => {
                error!("no information about {peer}");
                vec![]
            }
            Err(e) => {
                error!("failed to retrieve information about {peer}: {e}");
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

    // Chain =========
    pub fn me_onchain(&self) -> Address {
        self.chain_api.me_onchain()
    }

    /// Get ticket price
    pub async fn get_ticket_price(&self) -> errors::Result<Option<U256>> {
        Ok(self.chain_api.ticket_price().await?)
    }

    /// Get ticket winning probability
    pub async fn get_ticket_probability(&self) -> errors::Result<f64> {
        Ok(hopr_internal_types::tickets::WINNING_PROB)
    }

    /// List of all accounts announced on the chain
    pub async fn accounts_announced_on_chain(&self) -> errors::Result<Vec<AccountEntry>> {
        Ok(self.db.get_accounts(None, false).await?)
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
        Ok(self.chain_api.channel(src, dest).await?)
    }

    /// List all channels open from a specified Address
    pub async fn channels_from(&self, src: &Address) -> errors::Result<Vec<ChannelEntry>> {
        Ok(self.chain_api.channels_from(src).await?)
    }

    /// List all channels open to a specified address
    pub async fn channels_to(&self, dest: &Address) -> errors::Result<Vec<ChannelEntry>> {
        Ok(self.chain_api.channels_to(dest).await?)
    }

    /// List all channels
    pub async fn all_channels(&self) -> errors::Result<Vec<ChannelEntry>> {
        Ok(self.chain_api.all_channels().await?)
    }

    /// Current safe allowance balance
    pub async fn safe_allowance(&self) -> errors::Result<Balance> {
        Ok(self.chain_api.safe_allowance().await?)
    }

    /// Withdraw on-chain assets to a given address
    /// @param recipient the account where the assets should be transferred to
    /// @param amount how many tokens to be transferred
    pub async fn withdraw(&self, recipient: Address, amount: Balance) -> errors::Result<Hash> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        Ok(self
            .chain_api
            .actions_ref()
            .withdraw(recipient, amount)
            .await?
            .await?
            .tx_hash)
    }

    pub async fn open_channel(&self, destination: &Address, amount: &Balance) -> errors::Result<OpenChannelResult> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let awaiter = self.chain_api.actions_ref().open_channel(*destination, *amount).await?;

        let channel_id = generate_channel_id(&self.chain_api.me_onchain(), destination);
        Ok(awaiter.await.map(|confirm| OpenChannelResult {
            tx_hash: confirm.tx_hash,
            channel_id,
        })?)
    }

    pub async fn fund_channel(&self, channel_id: &Hash, amount: &Balance) -> errors::Result<Hash> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        Ok(self
            .chain_api
            .actions_ref()
            .fund_channel(*channel_id, *amount)
            .await?
            .await
            .map(|confirm| confirm.tx_hash)?)
    }

    pub async fn close_channel(
        &self,
        counterparty: &Address,
        direction: ChannelDirection,
        redeem_before_close: bool,
    ) -> errors::Result<CloseChannelResult> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let confirmation = self
            .chain_api
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
            _ => Err(errors::HoprLibError::GeneralError(
                "close channel transaction failed".into(),
            )),
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
                None => Err(errors::HoprLibError::ChainError(ChainActionsError::InvalidArguments(
                    "cannot close channel that is not own".into(),
                ))),
            },
            None => Err(errors::HoprLibError::ChainError(ChainActionsError::ChannelDoesNotExist)),
        }
    }

    pub async fn get_channel_closure_notice_period(&self) -> errors::Result<Duration> {
        Ok(self.chain_api.get_channel_closure_notice_period().await?)
    }

    pub async fn redeem_all_tickets(&self, only_aggregated: bool) -> errors::Result<()> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        // We do not await the on-chain confirmation
        self.chain_api.actions_ref().redeem_all_tickets(only_aggregated).await?;

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
            .chain_api
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
            if channel.destination == self.chain_api.me_onchain() {
                // We do not await the on-chain confirmation
                redeem_count = self
                    .chain_api
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
        let _ = self.chain_api.actions_ref().redeem_ticket(ack_ticket).await?;

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
}
