use async_std::sync::RwLock;
use futures::{channel::mpsc::unbounded, FutureExt};
use std::str::FromStr;
use std::{pin::Pin, sync::Arc, time::Duration};

use core_ethereum_api::HoprChain;
use core_ethereum_db::db::CoreEthereumDb;
use core_ethereum_types::ContractAddresses;
use core_path::{channel_graph::ChannelGraph, DbPeerAddressResolver};
use core_strategy::strategy::{MultiStrategy, SingularStrategy};
use core_transport::{
    build_heartbeat, build_index_updater, build_manual_ping, build_network, build_packet_actions,
    build_ticket_aggregation, libp2p_identity, p2p_loop, ApplicationData, ChainKeypair, HalfKeyChallenge,
    HoprTransport, Keypair, Multiaddr, OffchainKeypair, TransportOutput, UniversalTimer,
};
use core_types::protocol::TagBloomFilter;
use utils_db::rusty::RustyLevelDbShim;
use utils_log::{debug, info};
use utils_types::{primitives::Address, traits::BinarySerializable};

use crate::chain::ChainNetworkConfig;
use crate::{config::HoprLibConfig, constants};

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::spawn_local;

#[cfg(all(feature = "wasm", not(test)))]
use wasm_bindgen_futures::spawn_local;

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
pub fn build_components<FOnReceived, FOnSent, FSaveTbf>(
    cfg: HoprLibConfig,
    chain_config: ChainNetworkConfig,
    me: OffchainKeypair,
    me_onchain: ChainKeypair,
    db: Arc<RwLock<CoreEthereumDb<RustyLevelDbShim>>>,
    on_acknowledgement: FOnSent,
    on_final_packet: FOnReceived,
    tbf: TagBloomFilter,
    save_tbf: FSaveTbf,
    my_multiaddresses: Vec<Multiaddr>, // TODO: needed only because there's no STUN ATM
) -> (
    HoprTransport,
    HoprChain,
    Vec<Pin<Box<dyn futures::Future<Output = HoprLoopComponents>>>>,
)
where
    FOnReceived: Fn(ApplicationData) + 'static,
    FOnSent: Fn(HalfKeyChallenge) + 'static,
    FSaveTbf: Fn(Box<[u8]>) + 'static,
{
    use futures::StreamExt;
    use utils_log::error;
    use utils_types::traits::PeerIdLike;

    let identity: libp2p_identity::Keypair = (&me).into();

    let (network, network_events_tx, network_events_rx) =
        build_network(identity.public().to_peer_id(), cfg.network_options);

    let addr_resolver = DbPeerAddressResolver(db.clone());

    let ticket_aggregation = build_ticket_aggregation(db.clone(), &me_onchain);

    // TODO: this needs refactoring of the config structures
    let contract_addrs = ContractAddresses {
        announcements: Address::from_str(&chain_config.announcements).unwrap(),
        channels: Address::from_str(&chain_config.channels).unwrap(),
        token: Address::from_str(&chain_config.token).unwrap(),
        price_oracle: Address::from_str(&chain_config.ticket_price_oracle).unwrap(),
        network_registry: Address::from_str(&chain_config.network_registry).unwrap(),
        network_registry_proxy: Address::from_str(&chain_config.network_registry_proxy).unwrap(),
        stake_factory: Address::from_str(&chain_config.node_stake_v2_factory).unwrap(),
        safe_registry: Address::from_str(&chain_config.node_safe_registry).unwrap(),
        module_implementation: Address::from_str(&chain_config.module_implementation).unwrap(),
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
    let on_ack_tkt_tx = crate::processes::spawn_ack_winning_ticket_handling(multi_strategy.clone());

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
    crate::processes::spawn_transport_output(transport_output_rx, on_final_packet, on_acknowledgement);

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
        // Box::pin(async move { indexer_refreshing_loop.map(|_| HoprLoopComponents::Indexing).await }),
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
        // Box::pin(async move {
        //     action_queue
        //         .action_loop()
        //         .map(|_| HoprLoopComponents::OutgoingOnchainTxQueue)
        //         .await
        // }),
    ];

    (hopr_transport_api, hopr_chain_api, ready_loops)
}
