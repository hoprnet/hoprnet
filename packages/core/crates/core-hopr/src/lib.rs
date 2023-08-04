mod adaptors;
mod helpers;
mod p2p;

use std::sync::Arc;

use async_lock::RwLock;
use futures::{StreamExt, FutureExt};

use core_network::{
    PeerId,
    network::{Network, NetworkConfig},
    heartbeat::{Heartbeat, HeartbeatConfig},
    messaging::ControlMessage,
    ping::{Ping, PingConfig}
};
use core_p2p::libp2p_identity;
use utils_log::error;

use crate::p2p::api;


#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct HoprTools {
    ping: Ping<adaptors::ping::PingExternalInteractions>,
    peers: adaptors::network::wasm::WasmNetwork
}

impl HoprTools {
    // pub async fn ping()
    pub fn new(ping: Ping<adaptors::ping::PingExternalInteractions>,
        peers: Arc<RwLock<Network<adaptors::network::ExternalNetworkInteractions>>>) -> Self
    {
        Self {
            ping,
            peers: adaptors::network::wasm::WasmNetwork::new(peers) }
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
            HoprLoopComponents::Swarm => write!(f, "libp2p component responsible for the handling of the p2p communication"),
            HoprLoopComponents::Heartbeat => write!(f, "heartbeat component responsible for maintaining the network quality measurements"),
        }
    }
}

/// The main core loop containing all of the individual core components running indefinitely
/// or until the first error/panic.
/// 
/// # Arguments
/// me: Placeholder for an object that can transform to the libp2p_identity::Keypair
// #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub async fn build_components(me: String,
    network_cfg: NetworkConfig, network_quality_threshold: f64, hb_cfg: HeartbeatConfig, ping_cfg: PingConfig) -> (HoprTools, futures::future::BoxFuture<'static, ()>) {
    // TODO: this needs to be passed from above -> packet key
    let identity = libp2p_identity::Keypair::generate_ed25519();

    let network = Arc::new(RwLock::new(Network::new(
        PeerId::from(identity.public()),
        network_quality_threshold,
        adaptors::network::ExternalNetworkInteractions{}
    )));

    let (ping_tx, ping_rx) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
    let (pong_tx, pong_rx) = futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<ControlMessage, ()>)>();
    
    // manual ping explicitly called by the API
    let ping = Ping::new(
        ping_cfg.clone(),
        ping_tx,
        pong_rx, 
        adaptors::ping::PingExternalInteractions::new(network.clone())
    );

    let network_for_heartbeat = network.clone();
    let main_loop = async move {
        let (hb_ping_tx, hb_ping_rx) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
        let (hb_pong_tx, hb_pong_rx) = futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<ControlMessage, ()>)>();
    
        let ready_loops: Vec<std::pin::Pin<Box<dyn futures::Future<Output = HoprLoopComponents>>>> = vec![
            // TODO: network mechanism to process connections (register, unregister, close...)
            // heartbeat mechanism
            Box::pin(async move {
                let hb_pinger = Ping::new(ping_cfg, hb_ping_tx, hb_pong_rx, adaptors::ping::PingExternalInteractions::new(network_for_heartbeat.clone()));
                Heartbeat::new(hb_cfg, hb_pinger, adaptors::heartbeat::HeartbeatExternalInteractions::new(network_for_heartbeat))
                    .heartbeat_loop()
                    .map(|_| HoprLoopComponents::Heartbeat).await
                }
            ),
            Box::pin(p2p::p2p_loop(identity,
                api::HeartbeatRequester::new(hb_ping_rx), api::HeartbeatResponder::new(hb_pong_tx),
                api::ManualPingRequester::new(ping_rx), api::HeartbeatResponder::new(pong_tx)
            ).map(|_| HoprLoopComponents::Swarm))
        ];

        let mut futs = helpers::to_futures_unordered(ready_loops);
        while let Some(process) = futs.next().await {
            error!("CRITICAL: the core system loop unexpectadly stopped: '{}'", process);
            unreachable!("Futures inside the main loop should never terminate, but run in the background");
        };
    };

    // TODO: once main_loop is Send, it can be used
    // (HoprTools::new(ping, network), Box::pin(main_loop))
    (HoprTools::new(ping, network), Box::pin(async {}))
}

#[cfg(feature = "wasm")]
pub mod wasm_impl {
    use super::*;
    use std::str::FromStr;
    use wasm_bindgen::prelude::*;
    
    use core_network::ping::Pinging;

    #[wasm_bindgen]
    impl HoprTools {
        /// Ping the peers represented as a Vec<JsString> values that are converted into usable
        /// PeerIds.
        ///
        /// # Arguments
        /// * `peers` - Vector of String representations of the PeerIds to be pinged.
        #[wasm_bindgen]
        pub async fn ping(&mut self, mut peers: Vec<js_sys::JsString>) {
            let converted = peers
                .drain(..)
                .filter_map(|x| {
                    let x: String = x.into();
                    core_network::PeerId::from_str(&x).ok()
                })
                .collect::<Vec<_>>();

            self.ping.ping(converted).await;
        }
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use utils_log::logger::JsLogger;
    use utils_misc::utils::wasm::JsResult;
    use wasm_bindgen::prelude::*;

    // Temporarily re-export core-packet
    #[allow(unused_imports)]
    use core_packet::interaction::wasm::*;

    // Temporarily re-export core-ethereum-misc commitments
    #[allow(unused_imports)]
    use core_ethereum_misc::commitment::wasm::*;

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
