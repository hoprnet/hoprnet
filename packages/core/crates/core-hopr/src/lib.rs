pub mod adaptors;
pub mod constants;
pub mod errors;
mod helpers;
mod p2p;
mod timer;

use crate::{adaptors::indexer::IndexerProcessed, p2p::api, timer::UniversalTimer};
use async_lock::RwLock;
use core_crypto::keypairs::ChainKeypair;
use core_ethereum_db::db::CoreEthereumDb;
use core_network::{
    heartbeat::Heartbeat,
    messaging::ControlMessage,
    network::{Network, NetworkEvent},
    ping::Ping,
};
use core_network::{heartbeat::HeartbeatConfig, ping::PingConfig, PeerId};
use core_p2p::libp2p_identity;
use core_protocol::{
    ack::{config::AckProtocolConfig, processor::AcknowledgementInteraction},
    heartbeat::config::HeartbeatProtocolConfig,
    msg::{
        config::MsgProtocolConfig,
        processor::{PacketActions, PacketInteraction, PacketInteractionConfig},
    },
    ticket_aggregation::{
        config::TicketAggregationProtocolConfig,
        processor::{TicketAggregationActions, TicketAggregationInteraction},
    },
};
use core_types::{channels::Ticket, protocol::TagBloomFilter};
use futures::channel::mpsc::{unbounded, UnboundedSender};
use futures::future::poll_fn;
use futures::{channel::mpsc::Sender, FutureExt, SinkExt, Stream, StreamExt};
use libp2p::request_response::{RequestId, ResponseChannel};
use multiaddr::Multiaddr;
use std::pin::Pin;
use std::{sync::Arc, time::Duration};
use utils_log::{error, info};
use utils_types::traits::BinarySerializable;

use core_ethereum_actions::transaction_queue::TransactionQueue;

use core_strategy::{
    strategy::{MultiStrategy, SingularStrategy},
    StrategyConfig,
};
use core_types::acknowledgement::AcknowledgedTicket;
use core_types::channels::ChannelEntry;

const MAXIMUM_NETWORK_UPDATE_EVENT_QUEUE_SIZE: usize = 2000;

/// This is used by the indexer to emit events when a change on channel entry is detected.
#[derive(Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct ChannelEventEmitter {
    tx: UnboundedSender<ChannelEntry>,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl ChannelEventEmitter {
    pub async fn send_event(&self, channel: &ChannelEntry) {
        let mut sender = self.tx.clone();
        let _ = sender.send(channel.clone()).await;
    }
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
    OutgoingOnchainTxQueue,
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
            HoprLoopComponents::OutgoingOnchainTxQueue => {
                write!(f, "on-chain transaction queue component for outgoing transactions")
            }
        }
    }
}

#[cfg(feature = "wasm")]
pub mod wasm_impls {
    use std::str::FromStr;

    use super::*;
    use core_crypto::keypairs::Keypair;
    use core_crypto::{keypairs::OffchainKeypair, types::HalfKeyChallenge};
    use core_ethereum_actions::transaction_queue::wasm::WasmTxExecutor;
    use core_ethereum_actions::CoreEthereumActions;
    use core_ethereum_db::db::wasm::Database;
    use core_ethereum_db::traits::HoprCoreEthereumDbActions;
    use core_network::network::NetworkConfig;
    use core_path::channel_graph::ChannelGraph;
    use core_path::path::TransportPath;
    use core_path::DbPeerAddressResolver;
    use core_strategy::strategy::MultiStrategyConfig;
    use core_types::channels::{ChannelChange, ChannelStatus};
    use core_types::protocol::ApplicationData;
    use utils_db::rusty::RustyLevelDbShim;
    use utils_misc::ok_or_jserr;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    #[derive(Clone)]
    pub struct HoprTools {
        ping: adaptors::ping::wasm::WasmPing,
        network: adaptors::network::wasm::WasmNetwork,
        indexer: adaptors::indexer::WasmIndexerInteractions,
        pkt_sender: PacketActions,
        chain_actions: core_ethereum_actions::wasm::WasmCoreEthereumActions,
        ticket_aggregate_actions: TicketAggregationActions<ResponseChannel<Result<Ticket, String>>, RequestId>,
        channel_events: ChannelEventEmitter,
        channel_graph: core_path::channel_graph::wasm::ChannelGraph,
    }

    impl HoprTools {
        pub fn new(
            ping: Ping<adaptors::ping::PingExternalInteractions<DbPeerAddressResolver>>,
            peers: Arc<RwLock<Network<adaptors::network::ExternalNetworkInteractions>>>,
            change_notifier: Sender<NetworkEvent>,
            indexer: adaptors::indexer::WasmIndexerInteractions,
            pkt_sender: PacketActions,
            chain_actions: core_ethereum_actions::wasm::WasmCoreEthereumActions,
            ticket_aggregate_actions: TicketAggregationActions<ResponseChannel<Result<Ticket, String>>, RequestId>,
            channel_events: ChannelEventEmitter,
            channel_graph: core_path::channel_graph::wasm::ChannelGraph,
        ) -> Self {
            Self {
                ping: adaptors::ping::wasm::WasmPing::new(Arc::new(RwLock::new(ping))),
                network: adaptors::network::wasm::WasmNetwork::new(peers, change_notifier),
                indexer,
                pkt_sender,
                ticket_aggregate_actions,
                chain_actions,
                channel_events,
                channel_graph,
            }
        }
    }

    #[wasm_bindgen]
    impl HoprTools {
        pub fn ping(&self) -> adaptors::ping::wasm::WasmPing {
            self.ping.clone()
        }

        pub fn network(&self) -> adaptors::network::wasm::WasmNetwork {
            self.network.clone()
        }

        pub fn index_updater(&self) -> adaptors::indexer::WasmIndexerInteractions {
            self.indexer.clone()
        }

        pub fn channel_events(&self) -> ChannelEventEmitter {
            self.channel_events.clone()
        }

        pub fn channel_graph(&self) -> core_path::channel_graph::wasm::ChannelGraph {
            self.channel_graph.clone()
        }

        pub async fn send_message(
            &self,
            msg: ApplicationData,
            path: TransportPath,
            timeout_in_millis: u64,
        ) -> Result<HalfKeyChallenge, JsValue> {
            match self.pkt_sender.clone().send_packet(msg, path) {
                Ok(mut awaiter) => {
                    utils_log::debug!("Awaiting the HalfKeyChallenge");
                    ok_or_jserr!(
                        awaiter
                            .consume_and_wait(std::time::Duration::from_millis(timeout_in_millis))
                            .await
                    )
                }
                Err(e) => Err(wasm_bindgen::JsValue::from(e.to_string())),
            }
        }

        pub async fn aggregate_tickets(
            &mut self,
            channel: &ChannelEntry,
            timeout_in_millis: u64,
        ) -> Result<(), JsValue> {
            ok_or_jserr!(
                ok_or_jserr!(self.ticket_aggregate_actions.aggregate_tickets(channel))?
                    .consume_and_wait(std::time::Duration::from_millis(timeout_in_millis))
                    .await
            )
        }

        pub fn chain_actions(&self) -> core_ethereum_actions::wasm::WasmCoreEthereumActions {
            self.chain_actions.clone()
        }
    }

    /// The main core function building all core components
    ///
    /// This method creates a group of utilities that can be used to generate triggers for the core application
    /// business logic, as well as the main loop that can be triggered to start event processing.
    ///
    /// The loop containing all the individual core components is running indefinitely, it will not stop or return
    /// until the first unrecoverable error/panic is encountered.
    pub fn build_components(
        me: libp2p_identity::Keypair,
        me_onchain: ChainKeypair,
        db: Arc<RwLock<CoreEthereumDb<RustyLevelDbShim>>>,
        network_cfg: NetworkConfig,
        hb_cfg: HeartbeatConfig,
        ping_cfg: PingConfig,
        on_acknowledgement: js_sys::Function,
        packet_cfg: PacketInteractionConfig,
        on_final_packet: js_sys::Function,
        tbf: TagBloomFilter,
        save_tbf: js_sys::Function,
        tx_executor: WasmTxExecutor,
        my_multiaddresses: Vec<Multiaddr>, // TODO: needed only because there's no STUN ATM
        ack_proto_cfg: AckProtocolConfig,
        heartbeat_proto_cfg: HeartbeatProtocolConfig,
        msg_proto_cfg: MsgProtocolConfig,
        ticket_aggregation_proto_cfg: TicketAggregationProtocolConfig,
        strategy_cfg: StrategyConfig,
    ) -> (HoprTools, impl std::future::Future<Output = ()>) {
        let identity = me;

        let (network_events_tx, network_events_rx) =
            futures::channel::mpsc::channel::<NetworkEvent>(MAXIMUM_NETWORK_UPDATE_EVENT_QUEUE_SIZE);

        let network = Arc::new(RwLock::new(Network::new(
            identity.public().to_peer_id(),
            network_cfg,
            adaptors::network::ExternalNetworkInteractions::new(network_events_tx.clone()),
        )));

        let channel_graph = Arc::new(RwLock::new(ChannelGraph::new(me_onchain.public().to_address())));

        let addr_resolver = DbPeerAddressResolver(db.clone());

        let ticket_aggregation = TicketAggregationInteraction::new(db.clone(), &me_onchain.clone());

        let tx_queue = TransactionQueue::new(db.clone(), Box::new(tx_executor));

        let chain_actions =
            CoreEthereumActions::new(me_onchain.public().to_address(), db.clone(), tx_queue.new_sender());

        let multi_strategy = Arc::new(MultiStrategy::new(
            strategy_cfg,
            db.clone(),
            network.clone(),
            chain_actions.clone(),
            ticket_aggregation.writer(),
        ));
        info!("initialized strategies: {multi_strategy:?}");

        let on_ack_tx = adaptors::interactions::wasm::spawn_ack_receiver_loop(on_acknowledgement);

        // on acknowledged ticket notifier
        let (on_ack_tkt_tx, mut rx) = unbounded::<AcknowledgedTicket>();
        let ms_clone = multi_strategy.clone();
        wasm_bindgen_futures::spawn_local(async move {
            while let Some(ack) = poll_fn(|cx| Pin::new(&mut rx).poll_next(cx)).await {
                let _ = ms_clone.on_acknowledged_winning_ticket(&ack).await;
            }
        });

        // on channel state change notifier
        let (on_channel_event_tx, mut rx) = unbounded::<ChannelEntry>();
        let ms_clone = multi_strategy.clone();
        let cg_clone = channel_graph.clone();
        let db_clone = db.clone();
        let my_addr = me_onchain.public().to_address();
        wasm_bindgen_futures::spawn_local(async move {
            while let Some(channel) = poll_fn(|cx| Pin::new(&mut rx).poll_next(cx)).await {
                let maybe_direction = channel.direction(&my_addr);
                let change = cg_clone.write().await.update_channel(channel);

                // Check if this is our own channel
                if let Some(own_channel_direction) = maybe_direction {
                    if let Some(change_set) = change {
                        for channel_change in change_set {
                            let _ = ms_clone
                                .on_own_channel_changed(&channel, own_channel_direction, channel_change)
                                .await;

                            // Cleanup invalid tickets from the DB if epoch has changed
                            // TODO: this should be moved somewhere else once event broadcasts are implemented
                            if let ChannelChange::Epoch { .. } = channel_change {
                                let _ = db_clone.write().await.cleanup_invalid_channel_tickets(&channel).await;
                            }
                        }
                    } else if channel.status == ChannelStatus::Open {
                        // Emit Opening event if the channel did not exist before in the graph
                        let _ = ms_clone
                            .on_own_channel_changed(
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
        });

        let ack_actions = AcknowledgementInteraction::new(db.clone(), &packet_cfg.chain_keypair);

        let on_final_packet_tx = adaptors::interactions::wasm::spawn_on_final_packet_loop(on_final_packet);

        let tbf = Arc::new(RwLock::new(tbf));

        let packet_actions = PacketInteraction::new(db.clone(), tbf.clone(), packet_cfg);

        let (ping_tx, ping_rx) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
        let (pong_tx, pong_rx) =
            futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<(ControlMessage, String), ()>)>();

        // manual ping explicitly called by the API
        let ping = Ping::new(
            ping_cfg.clone(),
            ping_tx,
            pong_rx,
            adaptors::ping::PingExternalInteractions::new(
                network.clone(),
                addr_resolver.clone(),
                channel_graph.clone(),
            ),
        );

        let (indexer_update_tx, indexer_update_rx) =
            futures::channel::mpsc::channel::<IndexerProcessed>(adaptors::indexer::INDEXER_UPDATE_QUEUE_SIZE);
        let indexer_updater =
            adaptors::indexer::WasmIndexerInteractions::new(db.clone(), network.clone(), indexer_update_tx);

        let hopr_tools = HoprTools::new(
            ping,
            network.clone(),
            network_events_tx,
            indexer_updater,
            packet_actions.writer(),
            core_ethereum_actions::wasm::WasmCoreEthereumActions::new_from_actions(chain_actions),
            ticket_aggregation.writer(),
            ChannelEventEmitter {
                tx: on_channel_event_tx,
            },
            core_path::channel_graph::wasm::ChannelGraph::new(channel_graph.clone()),
        );

        let (hb_ping_tx, hb_ping_rx) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
        let (hb_pong_tx, hb_pong_rx) =
            futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<(ControlMessage, String), ()>)>();

        let heartbeat_network_clone = network.clone();
        let ping_network_clone = network.clone();
        let swarm_network_clone = network.clone();
        let tbf_clone = tbf.clone();
        let multistrategy_clone = multi_strategy.clone();
        let cg_clone = channel_graph.clone();
        let resolver_clone = addr_resolver.clone();

        let ready_loops: Vec<Pin<Box<dyn futures::Future<Output = HoprLoopComponents>>>> = vec![
            Box::pin(async move {
                let hb_pinger = Ping::new(
                    ping_cfg,
                    hb_ping_tx,
                    hb_pong_rx,
                    adaptors::ping::PingExternalInteractions::new(ping_network_clone, resolver_clone, cg_clone),
                );
                Heartbeat::new(
                    hb_cfg,
                    hb_pinger,
                    adaptors::heartbeat::HeartbeatExternalInteractions::new(heartbeat_network_clone),
                )
                .heartbeat_loop()
                .map(|_| HoprLoopComponents::Heartbeat)
                .await
            }),
            Box::pin(
                p2p::p2p_loop(
                    identity,
                    swarm_network_clone,
                    network_events_rx,
                    indexer_update_rx,
                    ack_actions,
                    packet_actions,
                    ticket_aggregation,
                    api::HeartbeatRequester::new(hb_ping_rx),
                    api::HeartbeatResponder::new(hb_pong_tx),
                    api::ManualPingRequester::new(ping_rx),
                    api::HeartbeatResponder::new(pong_tx),
                    my_multiaddresses,
                    ack_proto_cfg,
                    heartbeat_proto_cfg,
                    msg_proto_cfg,
                    ticket_aggregation_proto_cfg,
                    on_final_packet_tx,
                    on_ack_tx,
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
                        if let Err(_) = save_tbf.call1(
                            &wasm_bindgen::JsValue::null(),
                            js_sys::Uint8Array::from(bloom.to_bytes().as_ref()).as_ref(),
                        ) {
                            error!("failed to call save tbf closure");
                        }
                        info!("tag bloom filter saved");
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
        let mut futs = helpers::to_futures_unordered(ready_loops);

        let cg_clone = channel_graph.clone();
        let db_clone = db.clone();
        let main_loop = async move {
            // TODO: move this to a specialized one-shot startup initialization function?
            {
                let db = db_clone.read().await;
                if let Err(e) = cg_clone.write().await.sync_channels(&*db).await {
                    error!("failed to initialize channel graph from the DB: {e}");
                }
            }

            while let Some(process) = futs.next().await {
                error!("CRITICAL: the core system loop unexpectedly stopped: '{}'", process);
                unreachable!("Futures inside the main loop should never terminate, but run in the background");
            }
        };

        (hopr_tools, main_loop)
    }

    #[wasm_bindgen]
    pub struct CoreApp {
        tools: Option<HoprTools>,
        main_loop: Option<js_sys::Promise>,
    }

    #[wasm_bindgen]
    impl CoreApp {
        /// Constructor for the CoreApp
        #[wasm_bindgen(constructor)]
        pub fn new(
            me: &OffchainKeypair,
            me_onchain: &ChainKeypair,
            db: Database,
            network_cfg: NetworkConfig,
            hb_cfg: HeartbeatConfig,
            ping_cfg: PingConfig,
            on_acknowledgement: js_sys::Function,
            packet_cfg: PacketInteractionConfig,
            on_final_packet: js_sys::Function,
            tbf: TagBloomFilter,
            save_tbf: js_sys::Function,
            tx_executor: WasmTxExecutor,
            my_multiaddresses: Vec<js_sys::JsString>,
            ack_proto_cfg: AckProtocolConfig,
            heartbeat_proto_cfg: HeartbeatProtocolConfig,
            msg_proto_cfg: MsgProtocolConfig,
            ticket_aggregation_proto_cfg: TicketAggregationProtocolConfig,
            multi_strategy_cfg: MultiStrategyConfig,
        ) -> Self {
            let me: libp2p_identity::Keypair = me.into();
            let (tools, run_loop) = build_components(
                me,
                me_onchain.clone(),
                db.as_ref_counted(),
                network_cfg,
                hb_cfg,
                ping_cfg,
                on_acknowledgement,
                packet_cfg,
                on_final_packet,
                tbf,
                save_tbf,
                tx_executor,
                my_multiaddresses
                    .into_iter()
                    .map(|ma| {
                        let ma: String = ma.into();
                        multiaddr::Multiaddr::from_str(ma.as_str()).expect("Should be a valid multiaddress string")
                    })
                    .collect::<Vec<_>>(),
                ack_proto_cfg,
                heartbeat_proto_cfg,
                msg_proto_cfg,
                ticket_aggregation_proto_cfg,
                multi_strategy_cfg,
            );

            Self {
                tools: Some(tools),
                main_loop: Some(wasm_bindgen_futures::future_to_promise(
                    run_loop.map(|_| -> Result<JsValue, JsValue> { Ok(JsValue::UNDEFINED) }),
                )),
            }
        }

        #[wasm_bindgen]
        pub fn tools(&mut self) -> HoprTools {
            self.tools.take().expect("HOPR tools should only be extracted once")
        }

        #[wasm_bindgen]
        pub fn main_loop(&mut self) -> js_sys::Promise {
            self.main_loop
                .take()
                .expect("HOPR main loop should only be extracted once")
        }
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use utils_log::logger::wasm::JsLogger;
    use utils_misc::utils::wasm::JsResult;
    use wasm_bindgen::prelude::*;

    pub use crate::constants::wasm::*;

    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
    #[cfg(feature = "wee_alloc")]
    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

    static LOGGER: JsLogger = JsLogger {};

    #[allow(dead_code)]
    #[wasm_bindgen]
    pub fn core_hopr_initialize_crate() {
        let _ = JsLogger::install(&LOGGER, None);

        // When the `console_error_panic_hook` feature is enabled, we can call the
        // `set_panic_hook` function at least once during initialization, and then
        // we will get better error messages if our code ever panics.
        //
        // For more details see
        // https://github.com/rustwasm/console_error_panic_hook#readme
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
    }

    #[wasm_bindgen]
    pub fn core_hopr_gather_metrics() -> JsResult<String> {
        utils_metrics::metrics::wasm::gather_all_metrics()
    }
}
