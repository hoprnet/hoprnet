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
mod helpers;

pub use {
    chain_actions::errors::ChainActionsError,
    chain_api::config::{
        Addresses as NetworkContractAddresses, EnvironmentType, Network as ChainNetwork, ProtocolsConfig,
    },
    core_strategy::Strategy,
    core_transport::{
        config::{looks_like_domain, HostConfig, HostType},
        errors::{HoprTransportError, ProtocolError},
        ApplicationData, HalfKeyChallenge, Health, Keypair, Multiaddr, TicketStatistics, TransportOutput,
    },
    hopr_internal_types::prelude::*,
    hopr_primitive_types::prelude::*,
    hopr_primitive_types::rlp,
};

use std::{
    collections::HashMap,
    future::poll_fn,
    pin::Pin,
    str::FromStr,
    sync::{atomic::Ordering, Arc},
    time::Duration,
};

use async_lock::RwLock;
use async_std::task::spawn;
use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver},
    pin_mut, Future, FutureExt, Stream, StreamExt,
};

use chain_actions::{
    action_state::{ActionState, IndexerActionTracker},
    channels::ChannelActions,
    node::NodeActions,
    redeem::TicketRedeemActions,
};
use chain_api::config::ChainNetworkConfig;
use chain_api::HoprChain;
use chain_api::{can_register_with_safe, wait_for_funds, SignificantChainEvent};
use chain_types::chain_events::ChainEventType;
use chain_types::ContractAddresses;
use core_path::channel_graph::ChannelGraph;
use core_strategy::{
    aggregating::AwaitingAggregator,
    strategy::{MultiStrategy, SingularStrategy},
};
use core_transport::libp2p::identity::PeerId;
use core_transport::{
    build_index_updater, build_network, build_packet_actions, build_ticket_aggregation, build_transport_components,
    execute_on_tick, p2p_loop,
};
use core_transport::{ChainKeypair, Hash, HoprTransport, OffchainKeypair};
use core_transport::{IndexerToProcess, Network, PeerEligibility, PeerOrigin};
use hopr_platform::file::native::{join, read_file, remove_dir_all, write};
use tracing::{debug, error, info, warn};

use crate::config::HoprLibConfig;
use crate::config::SafeModule;
use crate::constants::{MIN_NATIVE_BALANCE, SUGGESTED_NATIVE_BALANCE};

use hopr_db_api::{
    accounts::HoprDbAccountOperations,
    db::{HoprDb, HoprDbConfig},
    info::{HoprDbInfoOperations, SafeInfo},
    resolver::HoprDbResolverOperations,
    HoprDbGeneralModelOperations,
};
use hopr_db_api::{channels::HoprDbChannelOperations, HoprDbAllOperations};

use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_db_api::prelude::ChainOrPacketKey::ChainKey;
use hopr_db_api::prelude::{DbError, HoprDbPeersOperations};
#[cfg(all(feature = "prometheus", not(test)))]
use {
    hopr_metrics::metrics::{MultiGauge, SimpleCounter, SimpleGauge},
    hopr_platform::time::native::current_time,
};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_SEND_MESSAGE_FAIL_COUNT: SimpleCounter = SimpleCounter::new(
        "hopr_failed_send_message_count",
        "Number of sent messages failures"
    ).unwrap();
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
pub enum HoprLoopComponents {
    #[strum(to_string = "libp2p component responsible for the handling of the p2p communication")]
    Swarm,
    #[strum(to_string = "heartbeat component responsible for maintaining the network quality measurements")]
    Heartbeat,
    #[strum(to_string = "tick wake up the strategies to perform an action")]
    StrategyTick,
    #[strum(to_string = "save operation for the bloom filter")]
    BloomFilterSave,
    #[strum(to_string = "initial indexing operation into the DB")]
    Indexing,
    #[strum(to_string = "on-chain transaction queue component for outgoing transactions")]
    OutgoingOnchainActionQueue,
    #[strum(to_string = "flush operation of outgoing ticket indices to the DB")]
    TicketIndexFlush,
}

impl HoprLoopComponents {
    /// Identifies whether a loop is allowed to finish or should
    /// run indefinitely.
    pub fn can_finish(&self) -> bool {
        matches!(self, HoprLoopComponents::Indexing)
    }
}

#[allow(clippy::too_many_arguments)] // TODO: refactor this function into a reasonable group of components once fully rearchitected
pub fn to_chain_events_refresh_process<Db, S, T>(
    me: PeerId,
    me_onchain: Address,
    db: Db,
    multi_strategy: Arc<MultiStrategy>,
    event_stream: S,
    channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
    transport_indexer_actions: core_transport::IndexerActions,
    indexer_action_tracker: Arc<IndexerActionTracker>,
    network: Arc<Network<T>>,
) -> Pin<Box<dyn futures::Future<Output = ()> + Send>>
where
    Db: HoprDbAllOperations + Send + Sync + 'static,
    S: Stream<Item = SignificantChainEvent> + Send + 'static,
    T: HoprDbAllOperations + Sync + Send + std::fmt::Debug + 'static,
{
    Box::pin(async move {
        pin_mut!(event_stream);
        while let Some(event) = event_stream.next().await {
            let resolved = indexer_action_tracker.match_and_resolve(&event).await;
            debug!("resolved {} indexer expectations in {}", resolved.len(), event);

            match event.event_type {
                ChainEventType::Announcement{peer, address, multiaddresses} => {
                    if peer != me {
                        // decapsulate the `p2p/<peer_id>` to remove duplicities
                        let mas = multiaddresses
                            .into_iter()
                            .map(|ma| core_transport::decapsulate_p2p_protocol(&ma))
                            .filter(|v| !v.is_empty())
                            .collect::<Vec<_>>();

                        if ! mas.is_empty() {
                            if let Err(e) = network.add(&peer, PeerOrigin::NetworkRegistry, mas.clone()).await
                            {
                                error!("failed to record '{peer}' from the NetworkRegistry: {e}");
                            }

                            transport_indexer_actions
                                .emit_indexer_update(IndexerToProcess::Announce(peer, mas))
                                .await;

                            if db
                                .is_allowed_in_network_registry(None, address)
                                .await
                                .unwrap_or(false)
                            {
                                transport_indexer_actions
                                    .emit_indexer_update(IndexerToProcess::EligibilityUpdate(peer, PeerEligibility::Eligible))
                                    .await;
                            }
                        }
                    } else {
                        debug!("Skipping announcements for myself ({peer})");
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
                                let _ = core_strategy::strategy::SingularStrategy::on_own_channel_changed(
                                    &*multi_strategy,
                                    &channel,
                                    own_channel_direction,
                                    channel_change,
                                )
                                .await;
                            }
                        } else if channel.status == ChannelStatus::Open {
                            // Emit Opening event if the channel did not exist before in the graph
                            let _ = core_strategy::strategy::SingularStrategy::on_own_channel_changed(
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
                }
                ChainEventType::NetworkRegistryUpdate(address, allowed) => {
                    let packet_key = db.translate_key(None, address).await;
                    match packet_key {
                        Ok(pk) => {
                            if let Some(pk) = pk {
                                let offchain_key: OffchainPublicKey = pk.try_into().expect("must be an offchain key at this point");
                                let peer_id = offchain_key.into();

                                transport_indexer_actions
                                    .emit_indexer_update(IndexerToProcess::EligibilityUpdate(
                                        peer_id,
                                        allowed.clone().into()
                                    ))
                                    .await;

                                match allowed {
                                    chain_types::chain_events::NetworkRegistryStatus::Allowed => {
                                        if let Err(e) = network.add(&peer_id, PeerOrigin::NetworkRegistry, vec![]).await {
                                            error!("failed to allow '{peer_id}' locally, although it is allowed on-chain: {e}")
                                        }
                                    },
                                    chain_types::chain_events::NetworkRegistryStatus::Denied => {
                                        if let Err(e) = network.remove(&peer_id).await {
                                            error!("failed to allow '{peer_id}' locally, although it is allowed on-chain: {e}")
                                        }
                                    },
                                };
                            }
                        }
                        Err(e) => error!("on_network_registry_node_allowed failed with: {}", e),
                    }
                }
                ChainEventType::NodeSafeRegistered(safe_address) => info!("node safe registered {safe_address}"),
            }
        }

        error!("The chain update process of HOPR objects should never stop")
    })
}

/// Main builder of the hopr lib components
#[allow(clippy::type_complexity)] // TODO: refactor this function into a reasonable group of components once fully rearchitected
#[allow(clippy::too_many_arguments)] // TODO: refactor this function into a reasonable group of components once fully rearchitected
pub fn build_components<FSaveTbf, T>(
    cfg: HoprLibConfig,
    chain_config: ChainNetworkConfig,
    me: OffchainKeypair,
    me_onchain: ChainKeypair,
    db: T,
    tbf: TagBloomFilter,
    save_tbf: FSaveTbf,
    my_multiaddresses: Vec<Multiaddr>, // TODO: needed only because there's no STUN ATM
) -> (
    HoprTransport<T>,
    HoprChain<T>,
    HashMap<HoprLoopComponents, Pin<Box<dyn futures::Future<Output = ()> + Send + Sync>>>,
    UnboundedReceiver<TransportOutput>,
    Arc<MultiStrategy>,
)
where
    FSaveTbf: Fn(Box<[u8]>) + Clone + Send + Sync + 'static,
    T: HoprDbAllOperations + Sync + Send + std::fmt::Debug + Clone + 'static,
{
    let identity: core_transport::libp2p::identity::Keypair = (&me).into();

    info!(
        "Creating local network registry and registering own external multiaddresses: {:?}",
        my_multiaddresses
    );

    let network = build_network(
        identity.public().to_peer_id(),
        my_multiaddresses.clone(),
        cfg.network_options,
        db.clone(),
    );

    let ticket_aggregation = build_ticket_aggregation(db.clone(), &me_onchain);

    let contract_addrs = ContractAddresses {
        announcements: chain_config.announcements,
        channels: chain_config.channels,
        token: chain_config.token,
        price_oracle: chain_config.ticket_price_oracle,
        network_registry: chain_config.network_registry,
        network_registry_proxy: chain_config.network_registry_proxy,
        stake_factory: chain_config.node_stake_v2_factory,
        safe_registry: chain_config.node_safe_registry,
        module_implementation: chain_config.module_implementation,
    };

    let (tx_indexer_events, rx_indexer_events) = futures::channel::mpsc::unbounded();

    let (action_queue, chain_actions, rpc_operations) = chain_api::build_chain_components(
        &me_onchain,
        chain_config.clone(),
        contract_addrs,
        cfg.safe_module.module_address,
        db.clone(),
    );

    let multi_strategy = Arc::new(MultiStrategy::new(
        cfg.strategy,
        db.clone(),
        chain_actions.clone(),
        AwaitingAggregator::new(
            db.clone(),
            me_onchain.clone(),
            ticket_aggregation.writer(),
            cfg.protocol.ticket_aggregation.timeout,
        ),
    ));
    debug!("initialized strategies: {multi_strategy:?}");

    let channel_graph = Arc::new(RwLock::new(ChannelGraph::new(me_onchain.public().to_address())));

    let (indexer_updater, indexer_update_rx) = build_index_updater(db.clone(), network.clone());

    let indexer_refreshing_loop = to_chain_events_refresh_process(
        (*me.public()).into(),
        core_transport::Keypair::public(&me_onchain).to_address(),
        db.clone(),
        multi_strategy.clone(),
        rx_indexer_events,
        channel_graph.clone(),
        indexer_updater.clone(),
        action_queue.action_state(),
        network.clone(),
    );

    let hopr_chain_api = chain_api::HoprChain::new(
        me_onchain.clone(),
        db.clone(),
        contract_addrs,
        cfg.safe_module.safe_address,
        chain_indexer::IndexerConfig {
            start_block_number: chain_config.channel_contract_deploy_block as u64,
        },
        tx_indexer_events,
        chain_actions.clone(),
        rpc_operations.clone(),
        channel_graph.clone(),
    );

    // on acknowledged ticket notifier
    let multi_strategy_ack_ticket = multi_strategy.clone();
    let (on_ack_tkt_tx, mut on_ack_tkt_rx) = unbounded::<AcknowledgedTicket>();
    spawn(async move {
        while let Some(ack) = poll_fn(|cx| Pin::new(&mut on_ack_tkt_rx).poll_next(cx)).await {
            let _ = core_strategy::strategy::SingularStrategy::on_acknowledged_winning_ticket(
                &*multi_strategy_ack_ticket,
                &ack,
            )
            .await;
        }
    });

    let tbf = Arc::new(RwLock::new(tbf));

    let (packet_actions, ack_actions) = build_packet_actions(&me, &me_onchain, db.clone(), tbf.clone());

    let ((ping, ping_rx, pong_tx), (mut heartbeat, hb_ping_rx, hb_pong_tx), network_events_rx) =
        build_transport_components(
            cfg.protocol,
            cfg.heartbeat,
            network.clone(),
            db.clone(),
            channel_graph.clone(),
        );

    let hopr_transport_api = HoprTransport::new(
        identity.clone(),
        me_onchain.clone(),
        cfg.transport,
        db.clone(),
        ping,
        network.clone(),
        indexer_updater,
        packet_actions.writer(),
        ticket_aggregation.writer(),
        channel_graph.clone(),
        my_multiaddresses.clone(),
    );

    let (transport_output_tx, transport_output_rx) = unbounded::<TransportOutput>();

    let swarm_network_clone = network.clone();
    let tbf_clone = tbf.clone();
    let multistrategy_clone = multi_strategy.clone();

    spawn(async move {
        let chain_events: Vec<Pin<Box<dyn futures::Future<Output = HoprLoopComponents> + Send>>> = vec![
            Box::pin(indexer_refreshing_loop.map(|_| HoprLoopComponents::Indexing)),
            Box::pin(async move {
                action_queue
                    .action_loop()
                    .map(|_| HoprLoopComponents::OutgoingOnchainActionQueue)
                    .await
            }),
        ];

        let mut futs = crate::helpers::to_futures_unordered(chain_events);

        while let Some(process) = futs.next().await {
            if process.can_finish() {
                continue;
            } else {
                error!("CRITICAL: the core chain loop unexpectedly stopped: '{}'", process);
                panic!("CRITICAL: the core chain loop unexpectedly stopped: '{}'", process);
            }
        }
    });

    let mut processes: HashMap<HoprLoopComponents, Pin<Box<dyn futures::Future<Output = ()> + Send>>> = HashMap::new();
    processes.insert(
        HoprLoopComponents::Heartbeat,
        Box::pin(async move { heartbeat.heartbeat_loop().await }),
    );
    processes.insert(
        HoprLoopComponents::Swarm,
        Box::pin(p2p_loop(
            String::from(constants::APP_VERSION),
            identity,
            swarm_network_clone,
            network_events_rx,
            indexer_update_rx,
            ack_actions,
            packet_actions,
            ticket_aggregation,
            core_transport::api::HeartbeatRequester::new(hb_ping_rx),
            core_transport::api::HeartbeatResponder::new(hb_pong_tx),
            core_transport::api::ManualPingRequester::new(ping_rx),
            core_transport::api::HeartbeatResponder::new(pong_tx),
            my_multiaddresses,
            cfg.protocol,
            transport_output_tx,
            on_ack_tkt_tx,
        )),
    );
    processes.insert(
        HoprLoopComponents::BloomFilterSave,
        Box::pin(execute_on_tick(Duration::from_secs(90), move || {
            let tbf_clone = tbf_clone.clone();
            let save_tbf = save_tbf.clone();

            async move {
                let bloom = tbf_clone.read().await.clone(); // Clone to immediately release the lock
                (save_tbf)(bloom.to_bytes());
            }
        })),
    );

    processes.insert(
        HoprLoopComponents::TicketIndexFlush,
        Box::pin(execute_on_tick(Duration::from_secs(5), move || {
            let db_clone = db.clone();
            async move {
                match db_clone.persist_outgoing_ticket_indices().await {
                    Ok(n) => debug!("successfully flushed states of {} outgoing ticket indices", n),
                    Err(e) => error!("failed to flush ticket indices: {e}"),
                }
            }
        })),
    );

    let processes = processes
        .into_iter()
        .map(|(tag, process)| process.map(|_| tag))
        .collect::<Vec<_>>();

    spawn(async move {
        let mut futs = crate::helpers::to_futures_unordered(processes);

        while let Some(process) = futs.next().await {
            if process.can_finish() {
                continue;
            } else {
                error!("CRITICAL: the core chain loop unexpectedly stopped: '{}'", process);
                panic!("CRITICAL: the core chain loop unexpectedly stopped: '{}'", process);
            }
        }
    });

    // TODO: return join handles for all background running tasks
    (
        hopr_transport_api,
        hopr_chain_api,
        HashMap::new(),
        transport_output_rx,
        multistrategy_clone,
    )
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
    is_public: bool,
    ingress_rx: Option<UnboundedReceiver<TransportOutput>>,
    state: Arc<AtomicHoprState>,
    network: String,
    transport_api: HoprTransport<HoprDb>,
    chain_api: HoprChain<HoprDb>,
    db: HoprDb,
    chain_cfg: ChainNetworkConfig,
    safe_module_cfg: SafeModule,
    multistrategy: Arc<MultiStrategy>,
}

impl Hopr {
    pub fn new(mut cfg: config::HoprLibConfig, me: &OffchainKeypair, me_onchain: &ChainKeypair) -> Self {
        let multiaddress = match &cfg.host.address {
            core_transport::config::HostType::IPv4(ip) => {
                Multiaddr::from_str(format!("/ip4/{}/tcp/{}", ip.as_str(), cfg.host.port).as_str()).unwrap()
            }
            core_transport::config::HostType::Domain(domain) => {
                Multiaddr::from_str(format!("/dns4/{}/tcp/{}", domain.as_str(), cfg.host.port).as_str()).unwrap()
            }
        };

        let db_path: String = join(&[&cfg.db.data, "db"]).expect("Could not create a db storage path");
        info!("Initiating the DB at: {db_path}");

        if cfg.db.force_initialize {
            info!("Force cleaning up existing database");
            remove_dir_all(&db_path).expect("Failed to remove the preexisting DB directory");
            cfg.db.initialize = true
        }

        // create DB dir if it does not exist
        if let Some(parent_dir_path) = std::path::Path::new(&db_path).parent() {
            if !parent_dir_path.is_dir() {
                std::fs::create_dir_all(parent_dir_path).expect("Failed to create a DB directory")
            }
        }

        let db_cfg = HoprDbConfig {
            create_if_missing: cfg.db.initialize,
            force_create: cfg.db.force_initialize,
            log_slow_queries: std::time::Duration::from_millis(150),
        };
        let db = async_std::task::block_on(HoprDb::new(db_path.clone(), me_onchain.clone(), db_cfg));

        info!("Creating chain components using provider URL: {:?}", cfg.chain.provider);
        let resolved_environment = chain_api::config::ChainNetworkConfig::new(
            &cfg.chain.network,
            crate::constants::APP_VERSION_COERCED,
            cfg.chain.provider.as_deref(),
            &mut cfg.chain.protocols,
        )
        .expect("Failed to resolve blockchain environment");
        let contract_addresses = ContractAddresses::from(&resolved_environment);
        info!(
            "Resolved contract addresses for myself as '{}': {:?}",
            me_onchain.public().to_hex(),
            contract_addresses
        );

        // let mut packetCfg = PacketInteractionConfig::new(packetKeypair, chainKeypair)
        // packetCfg.check_unrealized_balance = cfg.chain.check_unrealized_balance

        let is_public = cfg.chain.announce;

        let tbf_path = join(&[&cfg.db.data, "tbf"]).expect("Could not create a tbf storage path");
        info!("Creating the Bloom filter storage at: {}", tbf_path);

        let tbf = read_file(&tbf_path)
            .and_then(|data| {
                TagBloomFilter::from_bytes(&data)
                    .map_err(|e| hopr_platform::error::PlatformError::GeneralError(e.to_string()))
            })
            .unwrap_or_else(|_| {
                debug!("No tag Bloom filter found, using empty");
                TagBloomFilter::default()
            });

        let save_tbf = move |data: Box<[u8]>| {
            if let Err(e) = write(&tbf_path, data) {
                error!("Tag Bloom filter save failed: {e}")
            } else {
                info!("Tag Bloom filter saved successfully")
            };
        };

        let (transport_api, chain_api, _processes, transport_ingress, multistrategy) = build_components(
            cfg.clone(),
            resolved_environment.clone(),
            me.clone(),
            me_onchain.clone(),
            db.clone(),
            tbf,
            save_tbf,
            vec![multiaddress],
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
        }

        Self {
            state: Arc::new(AtomicHoprState::new(HoprState::Uninitialized)),
            network: cfg.chain.network,
            is_public,
            ingress_rx: Some(transport_ingress),
            me: me.clone(),
            db,
            transport_api,
            chain_api,
            chain_cfg: resolved_environment,
            safe_module_cfg: cfg.safe_module,
            multistrategy,
        }
    }

    fn error_if_not_in_state(&self, state: HoprState, error: String) -> errors::Result<()> {
        if self.status() == state {
            Ok(())
        } else {
            Err(errors::HoprLibError::StatusError(error))
        }
    }

    /// Get the ingress object for messages arriving to this node
    #[must_use]
    pub fn ingress(&mut self) -> UnboundedReceiver<TransportOutput> {
        self.ingress_rx
            .take()
            .expect("The ingress received can only be taken out once")
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
        self.network.clone()
    }

    pub async fn get_balance(&self, balance_type: BalanceType) -> errors::Result<Balance> {
        Ok(self.chain_api.get_balance(balance_type).await?)
    }

    pub async fn get_safe_balance(&self, balance_type: BalanceType) -> errors::Result<Balance> {
        let safe_balance = self
            .chain_api
            .get_safe_balance(self.safe_module_cfg.safe_address, balance_type)
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
                        Ok::<_, DbError>(())
                    })
                })
                .await?;
        }
        Ok(safe_balance)
    }

    pub fn get_safe_config(&self) -> SafeModule {
        self.safe_module_cfg.clone()
    }

    pub fn chain_config(&self) -> ChainNetworkConfig {
        self.chain_cfg.clone()
    }

    pub async fn run(&self) -> errors::Result<impl Future<Output = ()>> {
        self.error_if_not_in_state(
            HoprState::Uninitialized,
            "Cannot start the hopr node multiple times".into(),
        )?;

        info!(
            "Node is not started, please fund this node {} with at least {}",
            self.me_onchain(),
            Balance::new_from_str(SUGGESTED_NATIVE_BALANCE, BalanceType::HOPR).to_formatted_string()
        );

        wait_for_funds(
            self.me_onchain(),
            Balance::new_from_str(MIN_NATIVE_BALANCE, BalanceType::Native),
            Duration::from_secs(200),
            self.chain_api.rpc(),
        )
        .await
        .expect("failed to wait for funds");

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

        // wait for the indexer sync
        info!("Start the indexer and sync the chain");
        self.chain_api.sync_chain().await?;

        info!("Loading initial peers from the storage");
        self.transport_api.init_from_db().await?;

        // Possibly register node-safe pair to NodeSafeRegistry. Following that the
        // connector is set to use safe tx variants.
        if can_register_with_safe(
            self.me_onchain(),
            self.safe_module_cfg.safe_address,
            self.chain_api.rpc(),
        )
        .await?
        {
            info!("Registering safe by node");

            if self.me_onchain() == self.safe_module_cfg.safe_address {
                return Err(errors::HoprLibError::GeneralError(
                    "cannot self as staking safe address".into(),
                ));
            }

            if let Err(e) = self
                .chain_api
                .actions_ref()
                .register_safe_by_node(self.safe_module_cfg.safe_address)
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
                    safe_address: self.safe_module_cfg.safe_address,
                    module_address: self.safe_module_cfg.module_address,
                },
            )
            .await?;

        if self.is_public {
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
                // If the announcement fails we keep going to prevent the node from retrying
                // after restart. Functionality is limited and users must check the logs for
                // errors.
                Err(e) => error!("Failed to transmit node announcement: {e}"),
            }
        }

        self.state.store(HoprState::Running, Ordering::Relaxed);
        {
            let channel_graph = self.chain_api.channel_graph().clone();
            let mut cg = channel_graph.write().await;

            info!("Syncing channels from the previous runs");
            let channels = self.db.get_all_channels(None).await?;

            if let Err(e) = cg.sync_channels(channels) {
                error!("failed to initialize channel graph from the DB: {e}");
            }

            // Sync all the qualities there too
            let mut peer_stream = self.db.get_network_peers(Default::default(), false).await?;
            while let Some(peer) = peer_stream.next().await {
                if let Some(ChainKey(key)) = self.db.translate_key(None, peer.id.0).await? {
                    cg.update_channel_quality(self.me_onchain(), key, peer.get_quality());
                } else {
                    error!("could not translate peer info: {}", peer.id.1);
                }
            }
        }

        info!("# STARTED NODE");
        info!("ID {}", self.transport_api.me());
        info!("Protocol version {}", constants::APP_VERSION);

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_HOPR_NODE_INFO.set(
            &[
                &self.me.public().to_peerid_str(),
                &self.me_onchain().to_string(),
                &self.safe_module_cfg.safe_address.to_string(),
                &self.safe_module_cfg.module_address.to_string(),
            ],
            1.0,
        );

        // NOTE: strategy ticks must start after the chain is synced, otherwise
        // the strategy would react to historical data and drain through the native
        // balance on chain operations not relevant for the present network state
        let multi_strategy = self.multistrategy.clone();
        spawn(async move {
            execute_on_tick(Duration::from_secs(60), move || {
                let multi_strategy = multi_strategy.clone();

                async move {
                    info!("doing strategy tick");
                    let _ = multi_strategy.on_tick().await;
                    info!("strategy tick done");
                }
            })
            .await;

            error!(
                "CRITICAL: the core chain loop unexpectedly stopped: '{}'",
                HoprLoopComponents::StrategyTick
            );
            panic!(
                "CRITICAL: the core chain loop unexpectedly stopped: '{}'",
                HoprLoopComponents::StrategyTick
            );
        });

        // TODO: return JoinHandles here
        Ok(futures::future::pending())
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

    /// Test whether the peer with PeerId is allowed to access the network
    pub async fn is_allowed_to_access_network(&self, peer: &PeerId) -> errors::Result<bool> {
        Ok(self.transport_api.is_allowed_to_access_network(peer).await?)
    }

    /// Ping another node in the network based on the PeerId
    pub async fn ping(&self, peer: &PeerId) -> errors::Result<Option<std::time::Duration>> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        Ok(self.transport_api.ping(peer).await?)
    }

    /// Send a message to another peer in the network
    ///
    /// @param msg message to send
    /// @param destination PeerId of the destination
    /// @param intermediatePath optional set path manually
    /// @param hops optional number of required intermediate nodes
    /// @param applicationTag optional tag identifying the sending application
    /// @returns ack challenge
    #[tracing::instrument(level = "debug", skip(self, msg))]
    pub async fn send_message(
        &self,
        msg: Box<[u8]>,
        destination: PeerId,
        intermediate_path: Option<Vec<PeerId>>,
        hops: Option<u16>,
        application_tag: Option<u16>,
    ) -> errors::Result<HalfKeyChallenge> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let result = self
            .transport_api
            .send_message(msg, destination, intermediate_path, hops, application_tag)
            .await;

        #[cfg(all(feature = "prometheus", not(test)))]
        if result.is_err() {
            SimpleCounter::increment(&METRIC_SEND_MESSAGE_FAIL_COUNT);
        }

        Ok(result?)
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

    /// List all multiaddresses for this node announced to DHT
    pub async fn multiaddresses_announced_to_dht(&self, peer: &PeerId) -> Vec<Multiaddr> {
        self.transport_api.multiaddresses_announced_to_dht(peer).await
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
    pub async fn network_peer_info(&self, peer: &PeerId) -> errors::Result<Option<core_transport::PeerStatus>> {
        Ok(self.transport_api.network_peer_info(peer).await?)
    }

    /// Get peers connected peers with quality higher than some value
    pub async fn all_network_peers(
        &self,
        minimum_quality: f64,
    ) -> errors::Result<Vec<(Option<Address>, PeerId, core_transport::PeerStatus)>> {
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
        let pk = core_transport::OffchainPublicKey::try_from(peer_id)?;
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
