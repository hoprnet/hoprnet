pub mod adaptors;
pub mod errors;
mod helpers;
mod p2p;

use std::sync::Arc;

use async_lock::RwLock;
use futures::{channel::mpsc::Sender, FutureExt, StreamExt};
use multiaddr::Multiaddr;

use core_network::{heartbeat::HeartbeatConfig, ping::PingConfig, PeerId};

use core_ethereum_db::db::CoreEthereumDb;
use core_network::{
    heartbeat::Heartbeat,
    messaging::ControlMessage,
    network::{Network, NetworkEvent},
    ping::Ping,
};
use core_p2p::libp2p_identity;
use core_packet::interaction::{AcknowledgementInteraction, PacketActions, PacketInteraction, PacketInteractionConfig};
use utils_log::error;

use crate::adaptors::indexer::IndexerProcessed;
use crate::p2p::api;

#[cfg(feature = "wasm")]
use {core_ethereum_db::db::wasm::Database, utils_db::leveldb::wasm::LevelDbShim, wasm_bindgen::prelude::wasm_bindgen};

const MAXIMUM_NETWORK_UPDATE_EVENT_QUEUE_SIZE: usize = 2000;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
#[derive(Clone)]
pub struct HoprTools {
    ping: adaptors::ping::wasm::WasmPing,
    network: adaptors::network::wasm::WasmNetwork,
    indexer: adaptors::indexer::WasmIndexerInteractions,
    pkt_sender: PacketActions,
}

#[cfg(feature = "wasm")]
impl HoprTools {
    pub fn new(
        ping: Ping<adaptors::ping::PingExternalInteractions>,
        peers: Arc<RwLock<Network<adaptors::network::ExternalNetworkInteractions>>>,
        change_notifier: Sender<NetworkEvent>,
        indexer: adaptors::indexer::WasmIndexerInteractions,
        packet_sender: PacketActions,
    ) -> Self {
        Self {
            ping: adaptors::ping::wasm::WasmPing::new(Arc::new(RwLock::new(ping))),
            network: adaptors::network::wasm::WasmNetwork::new(peers, change_notifier),
            indexer,
            pkt_sender: packet_sender,
        }
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl HoprTools {
    #[wasm_bindgen]
    pub fn ping(&self) -> adaptors::ping::wasm::WasmPing {
        self.ping.clone()
    }

    #[wasm_bindgen]
    pub fn network(&self) -> adaptors::network::wasm::WasmNetwork {
        self.network.clone()
    }

    #[wasm_bindgen]
    pub fn index_updater(&self) -> adaptors::indexer::WasmIndexerInteractions {
        self.indexer.clone()
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
        }
    }
}

/// The main core function building all core components
///
/// This method creates a group of utilities that can be used to generate triggers for the core application
/// business logic, as well as the main loop that can be triggered to start event processing.
///
/// The loop containing all of the individual core components is running indefinitely, it will not stop or return
/// until the first unrecoverable error/panic is encountered.
#[cfg(feature = "wasm")]
pub fn build_components(
    me: libp2p_identity::Keypair,
    db: Arc<RwLock<CoreEthereumDb<LevelDbShim>>>,
    network_quality_threshold: f64,
    hb_cfg: HeartbeatConfig,
    ping_cfg: PingConfig,
    on_acknowledgement: Option<js_sys::Function>,
    on_acknowledged_ticket: Option<js_sys::Function>,
    packet_cfg: PacketInteractionConfig,
    on_final_packet: Option<js_sys::Function>,
    my_multiaddresses: Vec<Multiaddr>, // TODO: needed only because there's no STUN ATM
) -> (HoprTools, impl std::future::Future<Output = ()>) {
    use core_mixer::mixer::{Mixer, MixerConfig};

    let identity = me;

    let on_ack_tx = adaptors::interactions::wasm::spawn_ack_receiver_loop(on_acknowledgement);
    let on_ack_tkt_tx = adaptors::interactions::wasm::spawn_ack_tkt_receiver_loop(on_acknowledged_ticket);

    let (network_events_tx, network_events_rx) =
        futures::channel::mpsc::channel::<NetworkEvent>(MAXIMUM_NETWORK_UPDATE_EVENT_QUEUE_SIZE);

    let network = Arc::new(RwLock::new(Network::new(
        identity.public().to_peer_id(),
        network_quality_threshold,
        adaptors::network::ExternalNetworkInteractions::new(network_events_tx.clone()),
    )));

    let ack_actions = AcknowledgementInteraction::new(db.clone(), &packet_cfg.chain_keypair, on_ack_tx, on_ack_tkt_tx);

    let on_final_packet_tx = adaptors::interactions::wasm::spawn_on_final_packet_loop(on_final_packet);

    let packet_actions = PacketInteraction::new(
        db.clone(),
        Mixer::new_with_gloo_timers(MixerConfig::default()),
        ack_actions.writer(),
        on_final_packet_tx,
        packet_cfg,
    );

    let (ping_tx, ping_rx) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
    let (pong_tx, pong_rx) = futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<ControlMessage, ()>)>();

    // manual ping explicitly called by the API
    let ping = Ping::new(
        ping_cfg.clone(),
        ping_tx,
        pong_rx,
        adaptors::ping::PingExternalInteractions::new(network.clone()),
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
    );

    let (hb_ping_tx, hb_ping_rx) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
    let (hb_pong_tx, hb_pong_rx) =
        futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<ControlMessage, ()>)>();

    let heartbeat_network_clone = network.clone();
    let ping_network_clone = network.clone();
    let swarm_network_clone = network.clone();
    let ready_loops: Vec<std::pin::Pin<Box<dyn futures::Future<Output = HoprLoopComponents>>>> = vec![
        Box::pin(async move {
            let hb_pinger = Ping::new(
                ping_cfg,
                hb_ping_tx,
                hb_pong_rx,
                adaptors::ping::PingExternalInteractions::new(ping_network_clone),
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
                api::HeartbeatRequester::new(hb_ping_rx),
                api::HeartbeatResponder::new(hb_pong_tx),
                api::ManualPingRequester::new(ping_rx),
                api::HeartbeatResponder::new(pong_tx),
                my_multiaddresses,
            )
            .map(|_| HoprLoopComponents::Swarm),
        ),
    ];
    let mut futs = helpers::to_futures_unordered(ready_loops);

    let main_loop = async move {
        while let Some(process) = futs.next().await {
            error!("CRITICAL: the core system loop unexpectadly stopped: '{}'", process);
            unreachable!("Futures inside the main loop should never terminate, but run in the background");
        }
    };

    (hopr_tools, main_loop)
}

#[cfg(feature = "wasm")]
pub mod wasm_impl {
    use std::str::FromStr;

    use super::*;
    use core_crypto::{keypairs::OffchainKeypair, types::HalfKeyChallenge};
    use core_packet::interaction::ApplicationData;
    use core_path::path::Path;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    impl HoprTools {
        #[wasm_bindgen]
        pub async fn send_message(
            &self,
            msg: &ApplicationData,
            path: &Path,
            timeout_in_millis: u64,
        ) -> Result<HalfKeyChallenge, JsValue> {
            match self.pkt_sender.clone().send_packet(msg.clone(), path.clone()) {
                Ok(mut awaiter) => {
                    utils_log::debug!("Awaiting the HalfKeyChallenge");
                    awaiter
                        .consume_and_wait(std::time::Duration::from_millis(timeout_in_millis))
                        .await
                        .map_err(|e| wasm_bindgen::JsValue::from(e.to_string()))
                }
                Err(e) => Err(wasm_bindgen::JsValue::from(e.to_string())),
            }
        }
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
            db: Database, // TODO: replace the string with the KeyPair
            network_quality_threshold: f64,
            hb_cfg: HeartbeatConfig,
            ping_cfg: PingConfig,
            on_acknowledgement: Option<js_sys::Function>,
            on_acknowledged_ticket: Option<js_sys::Function>,
            packet_cfg: PacketInteractionConfig,
            on_final_packet: Option<js_sys::Function>,
            my_multiaddresses: Vec<js_sys::JsString>,
        ) -> Self {
            let me: libp2p_identity::Keypair = me.into();
            let (tools, run_loop) = build_components(
                me,
                db.as_ref_counted(),
                network_quality_threshold,
                hb_cfg,
                ping_cfg,
                on_acknowledgement,
                on_acknowledged_ticket,
                packet_cfg,
                on_final_packet,
                my_multiaddresses
                    .into_iter()
                    .map(|ma| {
                        let ma: String = ma.into();
                        multiaddr::Multiaddr::from_str(ma.as_str()).expect("Should be a valid multiaddress string")
                    })
                    .collect::<Vec<_>>(),
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
