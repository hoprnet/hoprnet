mod chain;
pub mod config;
pub mod constants;
pub mod errors;
mod helpers;
mod processes;


pub use {
    core_transport::{TransportOutput, ApplicationData, HalfKeyChallenge},
    chain::{Network, ProtocolsConfig},
    utils_types::primitives::{Address, Balance, BalanceType},
};

use std::{
    pin::Pin,
    sync::Arc,
    str::FromStr,
    time::Duration
};

use async_lock::RwLock;
use async_std::task::spawn_local;
use futures::{
    Future, channel::mpsc::{unbounded, UnboundedReceiver}, FutureExt, StreamExt
};

use core_ethereum_actions::{channels::ChannelActions, node::NodeActions, redeem::TicketRedeemActions, errors::CoreEthereumActionsError};
use core_ethereum_api::{can_register_with_safe, wait_for_funds, ChannelEntry};
use core_ethereum_types::chain_events::ChainEventType;
use core_transport::PeerEligibility;
use core_transport::TicketStatistics;
use core_types::protocol::TagBloomFilter;
use core_types::{
    account::AccountEntry,
    acknowledgement::AcknowledgedTicket,
    channels::{generate_channel_id, ChannelStatus, Ticket},
};

use utils_db::db::DB;
use utils_log::debug;
use utils_types::traits::{BinarySerializable, PeerIdLike, ToHex as _};

use core_ethereum_api::HoprChain;
use core_ethereum_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
use core_ethereum_types::ContractAddresses;
use core_path::{channel_graph::ChannelGraph, DbPeerAddressResolver};
use core_strategy::strategy::{MultiStrategy, SingularStrategy};
use core_transport::{
    build_heartbeat, build_index_updater, build_manual_ping, build_network, build_packet_actions,
    build_ticket_aggregation, libp2p_identity, p2p_loop, UniversalTimer,
};
use core_transport::libp2p_identity::PeerId;
use core_transport::{
    ChainKeypair, Hash, Health, HoprTransport, Keypair, Multiaddr, OffchainKeypair,
};
use platform::file::native::{join, read_file, remove_dir_all, write};
use utils_db::rusty::RustyLevelDbShim;
use utils_log::{error, info};
use utils_types::primitives::{Snapshot, U256};

use crate::chain::SmartContractConfig;
use crate::config::SafeModule;
use crate::constants::{MIN_NATIVE_BALANCE, SUGGESTED_NATIVE_BALANCE};
use crate::chain::ChainNetworkConfig;
use crate::config::HoprLibConfig;


#[cfg(all(feature = "prometheus", not(test)))]
use platform::time::native::current_timestamp;

#[cfg(all(feature = "prometheus", not(test)))]
use utils_metrics::metrics::{MultiGauge, SimpleCounter, SimpleGauge};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_MESSAGE_FAIL_COUNT: SimpleCounter = SimpleCounter::new(
        "core_counter_failed_send_messages",
        "Number of sent messages failures"
    ).unwrap();
    static ref METRIC_PROCESS_START_TIME: SimpleGauge = SimpleGauge::new(
        "hoprd_gauge_startup_unix_time_seconds",
        "The unix timestamp at which the process was started"
    ).unwrap();
    static ref METRIC_HOPR_LIB_VERSION: MultiGauge = MultiGauge::new(
        "hoprd_mgauge_version",
        "Executed version of HOPRd",
        &["version"]
    ).unwrap();
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum State {
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
    pub status: core_types::channels::ChannelStatus,
}


/// Enum differentiator for loop component futures.
///
/// Used to differentiate the type of the future that exits the loop premateruly
/// by tagging it as an enum.
#[derive(Debug, Clone)]
pub enum HoprLoopComponents {
    Swarm,
    Heartbeat,
    Timer,
    Indexing,
    OutgoingOnchainTxQueue,
}

impl HoprLoopComponents {
    pub fn can_finish(&self) -> bool {
        matches!(self, HoprLoopComponents::Indexing)
    }
}

impl std::fmt::Display for HoprLoopComponents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HoprLoopComponents::Swarm => write!(
                f,
                "libp2p component responsible for the handling of the p2p communication"
            ),
            HoprLoopComponents::Heartbeat => write!(
                f,
                "heartbeat component responsible for maintaining the network quality measurements"
            ),
            HoprLoopComponents::Timer => write!(f, "universal timer component for executing timed actions"),
            HoprLoopComponents::Indexing => write!(f, "initial indexing operation into the DB"),
            HoprLoopComponents::OutgoingOnchainTxQueue => {
                write!(f, "on-chain transaction queue component for outgoing transactions")
            }
        }
    }
}

/// Main builder of the hopr lib components
pub fn build_components<FSaveTbf>(
    cfg: HoprLibConfig,
    chain_config: ChainNetworkConfig,
    me: OffchainKeypair,
    me_onchain: ChainKeypair,
    db: Arc<RwLock<CoreEthereumDb<RustyLevelDbShim>>>,
    tbf: TagBloomFilter,
    save_tbf: FSaveTbf,
    my_multiaddresses: Vec<Multiaddr>, // TODO: needed only because there's no STUN ATM
) -> (
    HoprTransport,
    HoprChain,
    Vec<Pin<Box<dyn futures::Future<Output = HoprLoopComponents>>>>,
    UnboundedReceiver<TransportOutput>,
)
where
    FSaveTbf: Fn(Box<[u8]>) + 'static,
{
    let identity: libp2p_identity::Keypair = (&me).into();

    let (network, network_events_tx, network_events_rx) =
        build_network(identity.public().to_peer_id(), cfg.network_options);

    let addr_resolver = DbPeerAddressResolver(db.clone());

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

    let (action_queue, chain_actions, rpc_operations) = crate::chain::build_chain_components(
        &me_onchain,
        chain_config.clone(),
        contract_addrs,
        cfg.safe_module.module_address,
        db.clone(),
    );

    let multi_strategy = Arc::new(MultiStrategy::new(
        cfg.strategy,
        db.clone(),
        network.clone(),
        chain_actions.clone(),
        ticket_aggregation.writer(),
    ));
    debug!("initialized strategies: {multi_strategy:?}");

    let channel_graph = Arc::new(RwLock::new(ChannelGraph::new(me_onchain.public().to_address())));

    let (indexer_updater, indexer_update_rx) = build_index_updater(db.clone(), network.clone());

    let indexer_refreshing_loop = crate::processes::spawn_refresh_process_for_chain_events(
        me.public().to_peerid(),
        core_transport::Keypair::public(&me_onchain).to_address(),
        db.clone(),
        multi_strategy.clone(),
        rx_indexer_events,
        channel_graph.clone(),
        indexer_updater.clone(),
        action_queue.action_state(),
    );

    let hopr_chain_api: HoprChain = crate::chain::build_chain_api(
        me_onchain.clone(),
        db.clone(),
        contract_addrs,
        cfg.safe_module.safe_address,
        chain_config.channel_contract_deploy_block as u64,
        tx_indexer_events,
        chain_actions.clone(),
        rpc_operations.clone(),
        channel_graph.clone(),
    );

    // on acknowledged ticket notifier
    let (winning_ticket_process, on_ack_tkt_tx) = crate::processes::spawn_ack_winning_ticket_handling(multi_strategy.clone());

    let tbf = Arc::new(RwLock::new(tbf));

    let (packet_actions, ack_actions) = build_packet_actions(&me, &me_onchain, db.clone(), tbf.clone());

    let (ping, ping_rx, pong_tx) = build_manual_ping(
        cfg.protocol,
        network.clone(),
        addr_resolver.clone(),
        channel_graph.clone(),
    );

    let (mut heartbeat, hb_ping_rx, hb_pong_tx) = build_heartbeat(
        cfg.protocol,
        cfg.heartbeat,
        network.clone(),
        addr_resolver.clone(),
        channel_graph.clone(),
    );

    let hopr_transport_api = HoprTransport::new(
        identity.clone(),
        me_onchain.clone(),
        cfg.transport,
        db.clone(),
        ping,
        network.clone(),
        network_events_tx,
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

    // NOTE: This would normally be passed as ready loops and triggered in the
    // Hopr object's run, but with TS not fully migrated, these processes have to be
    // spawned to make sure that announce and registrations pass
    spawn_local(async move {
        let chain_events: Vec<Pin<Box<dyn futures::Future<Output = HoprLoopComponents>>>> = vec![
            Box::pin(async move { indexer_refreshing_loop.map(|_| HoprLoopComponents::Indexing).await }),
            Box::pin(async move {
                action_queue
                    .action_loop()
                    .map(|_| HoprLoopComponents::OutgoingOnchainTxQueue)
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

    let ready_loops: Vec<Pin<Box<dyn futures::Future<Output = HoprLoopComponents>>>> = vec![
        Box::pin(async move { heartbeat.heartbeat_loop().map(|_| HoprLoopComponents::Heartbeat).await }),
        Box::pin(
            p2p_loop(
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
            )
            .map(|_| HoprLoopComponents::Swarm),
        ),
        Box::pin(async move {
            UniversalTimer::new(Duration::from_secs(60))
                .timer_loop(|| async {
                    info!("doing strategy tick");
                    let _ = multistrategy_clone.on_tick().await;
                    info!("strategy tick done");
                })
                .map(|_| HoprLoopComponents::Timer)
                .await
        }),
        Box::pin(async move {
            UniversalTimer::new(Duration::from_secs(90))
                .timer_loop(|| async {
                    let bloom = tbf_clone.read().await.clone(); // Clone to immediately release the lock
                    (save_tbf)(bloom.to_bytes());
                })
                .map(|_| HoprLoopComponents::Timer)
                .await
        }),
    ];

    async_std::task::spawn_local(Box::pin(async move {
            let mut futs = crate::helpers::to_futures_unordered(ready_loops);

            while let Some(process) = futs.next().await {
                if process.can_finish() {
                    continue;
                } else {
                    error!("CRITICAL: the core chain loop unexpectedly stopped: '{}'", process);
                    panic!("CRITICAL: the core chain loop unexpectedly stopped: '{}'", process);
                }
            }
        })
    );

    // TODO: return join handles for all background running tasks
    (hopr_transport_api, hopr_chain_api, vec![], transport_output_rx)
}

// #[derive(Clone)]
pub struct Hopr {
    me: OffchainKeypair,
    is_public: bool,
    ingress_rx: Option<UnboundedReceiver<TransportOutput>>,
    state: State,
    transport_api: HoprTransport,
    chain_api: HoprChain,
    chain_cfg: ChainNetworkConfig,
    safe_module_cfg: SafeModule,
}

impl Hopr {
    pub fn new(mut cfg: config::HoprLibConfig, me: &OffchainKeypair, me_onchain: &ChainKeypair) -> Self
    {
        // pre-flight checks
        // Announced limitation for the `providence` release
        if !cfg.chain.announce {
            panic!("Announce option should be turned ON in Providence, only public nodes are supported");
        }

        let multiaddress = match &cfg.host.address {
            core_transport::config::HostType::IPv4(ip) => {
                Multiaddr::from_str(format!("/ip4/{}/tcp/{}", ip.as_str(), cfg.host.port).as_str()).unwrap()
            }
            core_transport::config::HostType::Domain(domain) => {
                Multiaddr::from_str(format!("/dns4/{}/tcp/{}", domain.as_str(), cfg.host.port).as_str()).unwrap()
            }
        };

        let db_path: String = join(&[&cfg.db.data, "db", crate::constants::DB_VERSION_TAG])
            .expect("Could not create a db storage path");
        info!("Initiating the DB at: {db_path}");

        if cfg.db.force_initialize {
            info!("Force cleaning up existing database");
            remove_dir_all(&db_path).expect("Failed to remove the preexisting DB directory");
            cfg.db.initialize = true
        }

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(utils_db::rusty::RustyLevelDbShim::new(&db_path, cfg.db.initialize)),
            me_onchain.public().to_address(),
        )));

        info!("Creating chain components using provider URL: {:?}", cfg.chain.provider);
        let resolved_environment =
            crate::chain::ChainNetworkConfig::new(&cfg.chain.network, cfg.chain.provider.as_deref(), &mut cfg.chain.protocols)
                .expect("Failed to resolve blockchain environment");
        let contract_addresses = SmartContractConfig::from(&resolved_environment);
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
                    .map_err(|e| platform::error::PlatformError::GeneralError(e.to_string()))
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

        let (transport_api, chain_api, _processes, transport_ingress) = build_components(
            cfg.clone(),
            resolved_environment.clone(),
            me.clone(),
            me_onchain.clone(),
            db,
            tbf,
            save_tbf,
            vec![multiaddress],
        );

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            METRIC_PROCESS_START_TIME.set(current_timestamp().as_secs() as f64);
            METRIC_HOPR_LIB_VERSION.set(
                &["version"],
                f64::from_str(const_format::formatcp!(
                    "{}.{}",
                    env!("CARGO_PKG_VERSION_MAJOR"),
                    env!("CARGO_PKG_VERSION_MINOR")
                ))
                .unwrap_or(0.0),
            );
        }

        Self {
            state: State::Uninitialized,
            is_public,
            ingress_rx: Some(transport_ingress),
            me: me.clone(),
            transport_api,
            chain_api,
            chain_cfg: resolved_environment,
            safe_module_cfg: cfg.safe_module,
        }
    }

    /// Get the ingress object for messages arriving to this node
    #[must_use]
    pub fn ingress(&mut self) -> UnboundedReceiver<TransportOutput> {
        self.ingress_rx.take().expect("The ingress received can only be taken out once")
    }

    pub fn status(&self) -> State {
        self.state
    }

    pub fn version_coerced(&self) -> String {
        String::from(constants::APP_VERSION_COERCED)
    }

    pub fn version(&self) -> String {
        String::from(constants::APP_VERSION)
    }

    pub async fn get_balance(&self, balance_type: BalanceType) -> errors::Result<Balance> {
        Ok(self.chain_api.get_balance(balance_type).await?)
    }

    pub async fn get_safe_balance(&self, balance_type: BalanceType) -> errors::Result<Balance> {
        Ok(self
            .chain_api
            .get_safe_balance(self.safe_module_cfg.safe_address, balance_type)
            .await?)
    }

    pub fn get_safe_config(&self) -> SafeModule {
        self.safe_module_cfg.clone()
    }

    pub fn chain_config(&self) -> ChainNetworkConfig {
        self.chain_cfg.clone()
    }

    pub async fn run(
        &mut self,
    ) -> errors::Result<impl Future<Output = ()>> {
        if self.state != State::Uninitialized {
            return Err(errors::HoprLibError::GeneralError(
                "Cannot start the hopr node multiple times".to_owned(),
            ));
        }

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

        self.state = State::Initializing;

        let balance = self.get_balance(BalanceType::Native).await?;

        let minimum_balance = Balance::new(U256::new(constants::MIN_NATIVE_BALANCE), BalanceType::Native);

        info!(
            "Ethereum account {} has {}. Minimum balance is {}",
            self.chain_api.me_onchain(),
            balance.to_formatted_string(),
            minimum_balance.to_formatted_string()
        );

        if balance.lte(&minimum_balance) {
            return Err(errors::HoprLibError::GeneralError(
                "Cannot start the node without a sufficiently funded wallet".to_string(),
            ));
        }

        info!("Linking chain and packet keys");
        self.chain_api
            .db()
            .write()
            .await
            .link_chain_and_packet_keys(&self.chain_api.me_onchain(), self.me.public(), &Snapshot::default())
            .await
            .map_err(core_transport::errors::HoprTransportError::from)?;

        info!("Loading initial peers");
        let index_updater = self.transport_api.index_updater();
        for (peer_id, _address, multiaddresses) in self.transport_api.get_public_nodes().await?.into_iter() {
            if self.transport_api.is_allowed_to_access_network(&peer_id).await {
                debug!("Using initial public node '{peer_id}'");
                index_updater
                    .emit_indexer_update(core_transport::IndexerToProcess::EligibilityUpdate(
                        peer_id,
                        PeerEligibility::Eligible,
                    ))
                    .await;
                index_updater
                    .emit_indexer_update(core_transport::IndexerToProcess::Announce(peer_id, multiaddresses))
                    .await;
            }
        }

        self.state = State::Indexing;

        // wait for the indexer sync
        info!("Starting chain interaction, which will trigger the indexer");
        self.chain_api.sync_chain().await?;

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

            if self
                .chain_api
                .actions_ref()
                .register_safe_by_node(self.safe_module_cfg.safe_address)
                .await?
                .await
                .is_ok()
            {
                let db = self.chain_api.db().clone();
                let mut db = db.write().await;
                db.set_staking_safe_address(&self.safe_module_cfg.safe_address).await?;
                db.set_staking_module_address(&self.safe_module_cfg.module_address)
                    .await?;
            } else {
                // Intentionally ignoring the errored state
                error!("Failed to register node with safe")
            }
        }

        if self.is_public {
            // At this point the node is already registered with Safe, so
            // we can announce via Safe-compliant TX

            // TODO: allow announcing all addresses once that option is supported
            let multiaddresses_to_announce = self.transport_api.announceable_multiaddresses();
            info!("Announcing node on chain: {:?}", &multiaddresses_to_announce[0]);
            if self
                .chain_api
                .actions_ref()
                .announce(&multiaddresses_to_announce[0], &self.me)
                .await
                .is_err()
            {
                // If the announcement fails we keep going to prevent the node from retrying
                // after restart. Functionality is limited and users must check the logs for
                // errors.
                error!("Failed to announce a node")
            }
        }

        self.state = State::Running;

        {
            info!("Syncing channels from the previous runs");
            let locked_db = self.chain_api.db();
            let db = locked_db.read().await;
            if let Err(e) = self.chain_api.channel_graph().write().await.sync_channels(&*db).await {
                error!("failed to initialize channel graph from the DB: {e}");
            }
        }

        info!("# STARTED NODE");
        info!("ID {}", self.transport_api.me());
        info!("Protocol version {}", constants::APP_VERSION);

        // let processes = self.processes.take().expect("processes should be present in the node");

        // Ok(Box::pin(async move {
        //     let mut futs = crate::helpers::to_futures_unordered(processes);

        //     while let Some(process) = futs.next().await {
        //         if process.can_finish() {
        //             continue;
        //         } else {
        //             error!("CRITICAL: the core chain loop unexpectedly stopped: '{}'", process);
        //             panic!("CRITICAL: the core chain loop unexpectedly stopped: '{}'", process);
        //         }
        //     }
        // }))
        Ok(futures::future::pending())
    }

    // p2p transport =========
    /// Own PeerId used in the libp2p transport layer
    pub fn me_peer_id(&self) -> PeerId {
        self.me.public().to_peerid()
    }

    /// Get the list of all announced public nodes in the network
    pub async fn get_public_nodes(&self) -> errors::Result<Vec<(PeerId, Address, Vec<Multiaddr>)>> {
        Ok(self.transport_api.get_public_nodes().await?)
    }

    /// Test whether the peer with PeerId is allowed to access the network
    pub async fn is_allowed_to_access_network(&self, peer: &PeerId) -> bool {
        self.transport_api.is_allowed_to_access_network(peer).await
    }

    /// Ping another node in the network based on the PeerId
    pub async fn ping(&self, peer: &PeerId) -> errors::Result<Option<std::time::Duration>> {
        if self.status() != State::Running {
            return Err(crate::errors::HoprLibError::GeneralError(
                "Node is not ready for on-chain operations".into(),
            ));
        }

        Ok(self.transport_api.ping(peer).await)
    }

    /// Send a message to another peer in the network
    ///
    /// @param msg message to send
    /// @param destination PeerId of the destination
    /// @param intermediatePath optional set path manually
    /// @param hops optional number of required intermediate nodes
    /// @param applicationTag optional tag identifying the sending application
    /// @returns ack challenge
    pub async fn send_message(
        &self,
        msg: Box<[u8]>,
        destination: PeerId,
        intermediate_path: Option<Vec<PeerId>>,
        hops: Option<u16>,
        application_tag: Option<u16>,
    ) -> errors::Result<HalfKeyChallenge> {
        if self.status() != State::Running {
            return Err(crate::errors::HoprLibError::GeneralError(
                "Node is not ready for on-chain operations".into(),
            ));
        }

        let result = self
            .transport_api
            .send_message(msg, destination, intermediate_path, hops, application_tag)
            .await;

        #[cfg(all(feature = "prometheus", not(test)))]
        if result.is_err() {
            SimpleCounter::increment(&METRIC_MESSAGE_FAIL_COUNT);
        }

        Ok(result?)
    }

    /// Attempts to aggregate all tickets in the given channel
    pub async fn aggregate_tickets(&self, channel: &Hash) -> errors::Result<()> {
        Ok(self.transport_api.aggregate_tickets(channel).await?)
    }

    /// List all multiaddresses announced
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

    /// Unregister a peer from the network
    pub async fn unregister(&self, peer: &PeerId) {
        self.transport_api.network_unregister(peer).await
    }

    /// List all peers connected to this
    pub async fn network_connected_peers(&self) -> Vec<PeerId> {
        self.transport_api.network_connected_peers().await
    }

    /// Get all data collected from the network relevant for a PeerId
    pub async fn network_peer_info(&self, peer: &PeerId) -> Option<core_transport::PeerStatus> {
        self.transport_api.network_peer_info(peer).await
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
        Ok(self.chain_api.db().read().await.get_accounts().await?)
    }

    /// Get the channel entry from Hash.
    /// @returns the channel entry of those two nodes
    pub async fn channel_from_hash(&self, channel: &Hash) -> errors::Result<Option<ChannelEntry>> {
        Ok(self.chain_api.db().read().await.get_channel(channel).await?)
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
        if self.status() != State::Running {
            return Err(crate::errors::HoprLibError::GeneralError(
                "Node is not ready for on-chain operations".into(),
            ));
        }

        Ok(self
            .chain_api
            .actions_ref()
            .withdraw(recipient, amount)
            .await?
            .await?
            .tx_hash)
    }

    pub async fn open_channel(&self, destination: &Address, amount: &Balance) -> errors::Result<OpenChannelResult> {
        if self.status() != State::Running {
            return Err(crate::errors::HoprLibError::GeneralError(
                "Node is not ready for on-chain operations".into(),
            ));
        }

        let awaiter = self.chain_api.actions_ref().open_channel(*destination, *amount).await?;

        let channel_id = generate_channel_id(&self.chain_api.me_onchain(), destination);
        Ok(awaiter.await.map(|confirm| OpenChannelResult {
            tx_hash: confirm.tx_hash,
            channel_id,
        })?)
    }

    pub async fn fund_channel(&self, channel_id: &Hash, amount: &Balance) -> errors::Result<Hash> {
        if self.status() != State::Running {
            return Err(crate::errors::HoprLibError::GeneralError(
                "Node is not ready for on-chain operations".into(),
            ));
        }

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
        direction: core_types::channels::ChannelDirection,
        redeem_before_close: bool,
    ) -> errors::Result<CloseChannelResult> {
        if self.status() != State::Running {
            return Err(crate::errors::HoprLibError::GeneralError(
                "Node is not ready for on-chain operations".into(),
            ));
        }

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
            ChainEventType::ChannelClosureInitiated(_) => Ok(CloseChannelResult {
                tx_hash: confirmation.tx_hash,
                status: ChannelStatus::PendingToClose,
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

    pub async fn close_channel_by_id(&self, channel_id: Hash, redeem_before_close: bool) -> errors::Result<CloseChannelResult> {
        match self.channel_from_hash(&channel_id).await? {
            Some(channel) => {
                match channel.orientation(&self.me_onchain()) {
                    Some((direction, counterparty)) => self.close_channel(&counterparty, direction, redeem_before_close).await,
                    None => Err(errors::HoprLibError::ChainError(CoreEthereumActionsError::InvalidArguments("cannot close channel that is not own".into()))),
                }
            }
            None => Err(errors::HoprLibError::ChainError(CoreEthereumActionsError::ChannelDoesNotExist))
        }
    }

    pub async fn get_channel_closure_notice_period(&self) -> errors::Result<Duration> {
        Ok(self.chain_api.get_channel_closure_notice_period().await?)
    }

    pub async fn redeem_all_tickets(&self, only_aggregated: bool) -> errors::Result<()> {
        if self.status() != State::Running {
            return Err(crate::errors::HoprLibError::GeneralError(
                "Node is not ready for on-chain operations".into(),
            ));
        }

        // We do not await the on-chain confirmation
        self.chain_api.actions_ref().redeem_all_tickets(only_aggregated).await?;

        Ok(())
    }

    pub async fn redeem_tickets_with_counterparty(
        &self,
        counterparty: &Address,
        only_aggregated: bool,
    ) -> errors::Result<()> {
        if self.status() != State::Running {
            return Err(crate::errors::HoprLibError::GeneralError(
                "Node is not ready for on-chain operations".into(),
            ));
        }

        // We do not await the on-chain confirmation
        let _ = self
            .chain_api
            .actions_ref()
            .redeem_tickets_with_counterparty(counterparty, only_aggregated)
            .await?;

        Ok(())
    }

    pub async fn redeem_tickets_in_channel(&self, channel: &Hash, only_aggregated: bool) -> errors::Result<usize> {
        if self.status() != State::Running {
            return Err(crate::errors::HoprLibError::GeneralError(
                "Node is not ready for on-chain operations".into(),
            ));
        }

        let channel = self.chain_api.db().read().await.get_channel(channel).await?;
        let mut redeem_count = 0;

        if let Some(channel) = channel {
            if channel.destination == self.chain_api.me_onchain() {
                // We do not await the on-chain confirmation
                redeem_count = self.chain_api
                    .actions_ref()
                    .redeem_tickets_in_channel(&channel, only_aggregated)
                    .await?
                    .len();
            }
        }

        Ok(redeem_count)
    }

    pub async fn redeem_ticket(&self, ack_ticket: AcknowledgedTicket) -> errors::Result<()> {
        if self.status() != State::Running {
            return Err(crate::errors::HoprLibError::GeneralError(
                "Node is not ready for on-chain operations".into(),
            ));
        }

        // We do not await the on-chain confirmation
        #[allow(clippy::let_underscore_future)]
        let _ = self.chain_api.actions_ref().redeem_ticket(ack_ticket).await?;

        Ok(())
    }

    pub async fn peerid_to_chain_key(&self, peer_id: &PeerId) -> errors::Result<Option<Address>> {
        let pk = core_transport::OffchainPublicKey::from_peerid(peer_id)?;
        Ok(self.chain_api.db().read().await.get_chain_key(&pk).await?)
    }

    pub async fn chain_key_to_peerid(&self, address: &Address) -> errors::Result<Option<PeerId>> {
        Ok(self
            .chain_api
            .db()
            .read()
            .await
            .get_packet_key(address)
            .await
            .map(|pk| pk.map(|v| v.to_peerid()))?)
    }
}