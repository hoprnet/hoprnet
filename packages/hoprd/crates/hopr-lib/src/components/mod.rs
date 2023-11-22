use std::{pin::Pin, sync::Arc, time::Duration};

use async_std::sync::RwLock;
use futures::{channel::mpsc::unbounded, FutureExt};

use core_ethereum_api::HoprChain;
use core_ethereum_db::db::CoreEthereumDb;
use core_path::{channel_graph::ChannelGraph, DbPeerAddressResolver};
use core_strategy::strategy::{MultiStrategy, SingularStrategy};
use core_transport::{
    build_heartbeat, build_index_updater, build_manual_ping, build_network, build_packet_actions,
    build_ticket_aggregation, libp2p_identity, p2p_loop, ApplicationData, ChainKeypair, HalfKeyChallenge, Keypair,
    Multiaddr, OffchainKeypair, TransportOutput, UniversalTimer,
};
use core_types::protocol::TagBloomFilter;
use utils_db::rusty::RustyLevelDbShim;
use utils_log::{debug, info};
use utils_types::traits::BinarySerializable;

use crate::{config::HoprLibConfig, constants};

use core_ethereum_actions::transaction_queue::TransactionExecutor;
#[cfg(feature = "wasm")]
use core_transport::wasm_impls::HoprTransport;

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
        if let HoprLoopComponents::Indexing = self {
            true
        } else {
            false
        }
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
#[cfg(feature = "wasm")]
pub fn build_components<FOnReceived, FOnSent, FSaveTbf, TxExec>(
    cfg: HoprLibConfig,
    me: OffchainKeypair,
    me_onchain: ChainKeypair,
    db: Arc<RwLock<CoreEthereumDb<RustyLevelDbShim>>>,
    on_acknowledgement: FOnSent,
    on_final_packet: FOnReceived,
    tbf: TagBloomFilter,
    save_tbf: FSaveTbf,
    tx_executor: TxExec,
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
    TxExec: TransactionExecutor + 'static,
{
    use core_ethereum_api::SignificantChainEvent;
    use utils_types::traits::PeerIdLike;

    let identity: libp2p_identity::Keypair = (&me).into();

    let (network, network_events_tx, network_events_rx) =
        build_network(identity.public().to_peer_id(), cfg.network_options);

    let addr_resolver = DbPeerAddressResolver(db.clone());

    let ticket_aggregation = build_ticket_aggregation(db.clone(), &me_onchain);

    let (tx_queue, chain_actions) =
        crate::chain::build_chain_components(me_onchain.public().to_address(), db.clone(), tx_executor);

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
    let (tx_indexer_events, rx_indexer_events) = futures::channel::mpsc::unbounded::<SignificantChainEvent>();
    let indexer_refreshing_loop =
        crate::processes::spawn_refresh_process_for_chain_events(
            me.public().to_peerid(),
            core_transport::Keypair::public(&me_onchain).to_address(),
            db.clone(),
            multi_strategy.clone(),
            rx_indexer_events,
            channel_graph.clone(),
            indexer_updater.clone()
        );

    let hopr_chain_api: HoprChain = crate::chain::build_chain_api(
        me_onchain.clone(),
        db.clone(),
        chain_actions.clone(),
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
        core_path::channel_graph::wasm::ChannelGraph::new(channel_graph.clone()),
        my_multiaddresses.clone(),
    );

    let (transport_output_tx, transport_output_rx) = unbounded::<TransportOutput>();
    crate::processes::spawn_transport_output(transport_output_rx, on_final_packet, on_acknowledgement);

    let swarm_network_clone = network.clone();
    let tbf_clone = tbf.clone();
    let multistrategy_clone = multi_strategy.clone();

    let ready_loops: Vec<Pin<Box<dyn futures::Future<Output = HoprLoopComponents>>>> = vec![
        Box::pin(async move { indexer_refreshing_loop.map(|_| HoprLoopComponents::Indexing).await }),
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
        Box::pin(async move {
            tx_queue
                .transaction_loop()
                .map(|_| HoprLoopComponents::OutgoingOnchainTxQueue)
                .await
        }),
    ];

    (hopr_transport_api, hopr_chain_api, ready_loops)
}
