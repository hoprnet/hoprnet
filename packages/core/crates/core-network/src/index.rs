use crate::{
    heartbeat::{Heartbeat, HeartbeatConfig, HeartbeatRequest},
    network::{wasm::WasmNetworkApi, Health, Network, PeerOrigin, PeerStatus},
    ping::{Ping, PingConfig},
};
use core_crypto::random::random_integer;
use futures::Future;
use gloo_timers::future::sleep;
use js_sys::{Array, Date, Function, JsString, Map, Number, Promise};
use libp2p::PeerId;
use std::{collections::HashMap, pin::Pin, rc::Rc, str::FromStr, time::Duration};
use utils_log::{error, info, warn};
use utils_misc::{
    streaming_iterable::{JsStreamingIterable, StreamingIterable},
    utils::wasm::js_map_to_hash_map,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};

/// Extracts version from the protocol identifier
/// ```rust
/// # use core_network::index::version_from_protocol;
/// let protocol: &str = "`/hopr/mont_blanc/heartbeat/2.1.0";
///
/// assert_eq!(version_from_protocol(protocol.into()), String::from("2.1.0"));
/// ```
pub fn version_from_protocol(protocol: String) -> String {
    let mut parts = protocol.as_str().split("/");
    match parts.nth(4) {
        None => "unknown".into(),
        Some(v) => v.into(),
    }
}

// Add PeerId import to typing file
#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
import { PeerId } from '@libp2p/interface-peer-id'
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub type JsConnection;

    #[wasm_bindgen(method, getter, structural, js_name = "remotePeer")]
    pub fn remote_peer(this: &JsConnection) -> JsPeerId;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "PeerId")]
    pub type JsPeerId;

    #[wasm_bindgen(structural, method, js_name = "toString")]
    pub fn to_string(this: &JsPeerId) -> String;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub type JsRegistrar;

    #[wasm_bindgen(structural, method)]
    pub fn handle(
        this: &JsRegistrar,
        protocol: Vec<JsString>,
        handler_function: &Closure<dyn Fn(IncomingConnection) -> ()>,
    ) -> Promise;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub type JsLibp2p;

    #[wasm_bindgen(structural, method, catch, js_name = "getRegistrar")]
    pub fn get_registrar(this: &JsLibp2p) -> Result<JsRegistrar, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub type IncomingConnection;

    #[wasm_bindgen(structural, method, getter)]
    pub fn connection(this: &IncomingConnection) -> JsConnection;

    #[wasm_bindgen(structural, method, getter)]
    pub fn stream(this: &IncomingConnection) -> JsStreamingIterable;

    #[wasm_bindgen(structural, method, getter)]
    pub fn protocol(this: &IncomingConnection) -> String;
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DialResponseStatus {
    SUCCESS = "SUCCESS",
    TIMEOUT = "E_TIMEOUT",
    ABORTED = "E_ABORTED",
    DIAL_ERROR = "E_DIAL",
    DHT_ERROR = "E_DHT_QUERY",
    NO_DHT = "E_NO_DHT",
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub type DialResponse;

    #[wasm_bindgen(structural, method, getter)]
    pub fn status(this: &DialResponse) -> DialResponseStatus;

    #[wasm_bindgen(structural, method, getter)]
    pub fn resp(this: &DialResponse) -> IncomingConnection;
}

#[wasm_bindgen(module = "@libp2p/peer-id")]
extern "C" {
    fn peerIdFromString(string: String) -> JsPeerId;
}

#[wasm_bindgen]
pub struct CoreNetwork {
    components: Rc<JsLibp2p>,
    network: Rc<Network<WasmNetworkApi>>,
    pinger: Rc<Ping<StreamingIterable>>,
    heartbeat: Rc<Heartbeat>,
}

static PEER_METADATA_PROTOCOL_VERSION: &str = "protocol_version";

// pin_project_lite::pin_project! {
//     pub struct DialRequest {
//         destination: PeerId,
//         protocols: Vec<String>,
//         dial:  Function,
//         components: JsLibp2p
//     }
// }

// impl DialRequest {
//     pub fn new(destination: PeerId, protocols: Vec<String>, dial: Function, components: JsLibp2p) -> Self {
//         Self {
//             destination,
//             protocols,
//             dial,
//             components,
//         }
//     }
// }

// impl<'a> Future for DialRequest {
//     type Output = Promise;

//     fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {}
// }

#[wasm_bindgen]
impl CoreNetwork {
    #[wasm_bindgen(constructor)]
    pub fn new(
        me: JsPeerId,
        quality_threshold: Number,
        on_peer_offline_cb: Function,
        on_network_health_change_cb: Function,
        is_public_cb: Function,
        close_connection_cb: Function,
        version: String,
        environment_id: String,
        components: JsLibp2p, // used to call handle
        send_message_cb: Function,
        max_parallel_pings: Number,
        heartbeat_variance: Number,
        heartbeat_interval: Number,
        heartbeat_threshold: Number,
    ) -> Self {
        let me: PeerId = PeerId::from_str(me.to_string().as_str()).unwrap();
        let network_api = WasmNetworkApi::new(
            on_peer_offline_cb,
            on_network_health_change_cb,
            is_public_cb,
            close_connection_cb,
        );

        let ping_config = PingConfig::new(
            max_parallel_pings.value_of() as usize,
            environment_id.to_owned(),
            version.to_owned(),
            Duration::from_secs(30),
        );

        let heartbeat_config = HeartbeatConfig::new(
            max_parallel_pings.value_of() as usize,
            heartbeat_variance.value_of() as u32,
            heartbeat_interval.value_of() as u32,
            heartbeat_threshold.value_of() as u64,
            environment_id,
            version,
        );

        let network = Rc::new(Network::new(me, quality_threshold.as_f64().unwrap(), network_api));

        // let closure = ;

        let send_message_cb = Rc::new(send_message_cb);
        let components = Rc::new(components);
        let components_to_move = components.clone();

        // let wasm_ping_api = WasmPingApi::new(send_message_cb);
        // let components_to_move = components.clone().dyn_into::<JsLibp2p>().unwrap();
        // let send_message_to_move = send_message_cb.clone().dyn_into::<Function>().unwrap();

        let network_to_move = network.clone();
        let pinger = Rc::new(Ping::new(
            ping_config,
            move |peer: &PeerId, result: crate::types::Result| network_to_move.update(peer, result),
            move |destination: PeerId,
                  protocols: Vec<String>|
                  -> Pin<Box<dyn Future<Output = Result<StreamingIterable, String>>>> {
                let send_message_to_move = send_message_cb.clone();
                let components_to_move = components_to_move.clone();

                Box::pin(async move {
                    let this = JsValue::undefined();

                    let peer = peerIdFromString(destination.to_string());
                    info!("converted PeerId {:?}", peer.to_string());
                    let protocols: Array = Array::from_iter(protocols.iter().map(|x| JsString::from(x.as_str())));

                    info!("converted protocols {:?}", protocols);
                    match send_message_to_move.call3(&this, &components_to_move, &peer, &protocols) {
                        Ok(obj) => {
                            let promise = obj.unchecked_into::<Promise>();

                            let dial_response = JsFuture::from(promise).await;

                            info!("{:?}", dial_response);

                            let dial_response = dial_response.unwrap().unchecked_into::<DialResponse>();

                            if dial_response.status() != DialResponseStatus::SUCCESS {
                                Err(format!("{:?}", dial_response.status()))
                            } else {
                                Ok(StreamingIterable::from(dial_response.resp().stream()))
                            }
                        }
                        Err(e) => {
                            error!("error while dialing {:?}", e);
                            todo!()
                        }
                    }
                })
            },
        ));

        let heartbeat = Rc::new(Heartbeat::new(heartbeat_config));

        Self {
            components,
            network,
            pinger,
            heartbeat,
        }
    }

    #[wasm_bindgen]
    pub async fn start(&mut self) {
        let network_to_move = self.network.clone();

        let handle_heartbeat = Closure::<dyn Fn(IncomingConnection) -> ()>::new(move |incoming: IncomingConnection| {
            let mut peer_metadata = HashMap::<String, String>::new();

            let proto_version = version_from_protocol(incoming.protocol());

            peer_metadata.insert(PEER_METADATA_PROTOCOL_VERSION.to_owned(), proto_version);

            let remote = PeerId::from_str(incoming.connection().remote_peer().to_string().as_str()).unwrap();

            if network_to_move.has(&remote) {
                network_to_move.update_with_metadata(&remote, Ok(Date::now() as u64), Some(peer_metadata))
            } else {
                network_to_move.add_with_metadata(&remote, PeerOrigin::IncomingConnection, Some(peer_metadata));
            }

            spawn_local(async move {
                HeartbeatRequest::from(incoming.stream()).handle().await;
            });
        });

        JsFuture::from(
            self.components
                .get_registrar()
                .expect("Libp2p instance without registrar")
                .handle(
                    self.heartbeat
                        .get_protocols()
                        .iter()
                        .map(|s| JsString::from(s.as_str()))
                        .collect(),
                    &handle_heartbeat,
                ),
        )
        .await
        .expect("Could not register heartbeat handler");

        // Leave callback to JS garbage collector
        handle_heartbeat.forget();

        // FIXME: makes this thread-safe
        let network_to_move = self.network.clone();
        let heartbeat_to_move = self.heartbeat.clone();
        let ping_to_move = self.pinger.clone();

        spawn_local(async move {
            while !heartbeat_to_move.has_ended() {
                let next_interval = random_integer(
                    heartbeat_to_move.get_config().heartbeat_interval as u64,
                    Some(
                        heartbeat_to_move.get_config().heartbeat_interval as u64
                            + heartbeat_to_move.get_config().heartbeat_variance as u64,
                    ),
                )
                .expect("Failed to compute next heartbeat interval");

                sleep(Duration::from_millis(next_interval)).await;
                let threshold = Date::now() as u64 - heartbeat_to_move.get_config().heartbeat_threshold;
                info!("Checking nodes since {}", threshold);
                ping_to_move
                    .ping_peers(network_to_move.find_peers_to_ping(threshold))
                    .await;
            }
        });
    }

    #[wasm_bindgen]
    pub async fn stop(&mut self) {
        match self.heartbeat.set_ended() {
            Ok(()) => (),
            Err(e) => info!("Could not end heartbeat mechanism due to {}", e),
        }
    }

    /// Ping the peers represented as a Vec<JsString> values that are converted into usable
    /// PeerIds.
    ///
    /// # Arguments
    /// * `peers` - Vector of String representations of the PeerIds to be pinged.
    #[wasm_bindgen]
    pub async fn ping(&self, mut peers: Vec<JsString>) {
        let converted = peers
            .drain(..)
            .filter_map(|x| {
                let x: String = x.into();
                PeerId::from_str(&x).ok()
            })
            .collect::<Vec<_>>();

        self.pinger.ping_peers(converted).await;
    }

    #[wasm_bindgen]
    pub fn register(&mut self, peer: JsPeerId, origin: PeerOrigin) {
        self.register_with_metadata(peer, origin, &Map::new())
    }

    #[wasm_bindgen(js_name = "registerWithMetadata")]
    pub fn register_with_metadata(&mut self, peer: JsPeerId, origin: PeerOrigin, metadata: &Map) {
        match PeerId::from_str(&peer.to_string().to_owned()) {
            Ok(p) => self
                .network
                .add_with_metadata(&p, origin, js_map_to_hash_map(&metadata)),
            Err(err) => {
                warn!(
                    "Failed to parse peer id {}, network ignores the register attempt: {}",
                    peer.to_string(),
                    err.to_string()
                );
            }
        }
    }
    #[wasm_bindgen(js_name = "peersToPing")]
    pub fn peers_to_ping(&self, threshold: u64) -> Vec<JsString> {
        self.network
            .find_peers_to_ping(threshold)
            .iter()
            .map(|x| x.to_base58().into())
            .collect::<Vec<JsString>>()
    }

    #[wasm_bindgen]
    pub fn contains(&self, peer: JsPeerId) -> bool {
        match PeerId::from_str(&peer.to_string()) {
            Ok(p) => self.network.has(&p),
            Err(err) => {
                warn!(
                    "Failed to parse peer id {}, network assumes it is not present: {}",
                    peer.to_string(),
                    err.to_string()
                );
                false
            }
        }
    }

    #[wasm_bindgen]
    pub fn unregister(&self, peer: JsPeerId) {
        let peer: String = peer.to_string().into();
        match PeerId::from_str(&peer) {
            Ok(p) => self.network.remove(&p),
            Err(err) => {
                warn!(
                    "Failed to parse peer id {}, network ignores the unregister attempt: {}",
                    peer,
                    err.to_string()
                );
            }
        }
    }

    #[wasm_bindgen(js_name = "getPeerInfo")]
    pub fn get_peer_info(&self, peer: JsPeerId) -> Option<PeerStatus> {
        let peer: String = peer.to_string().into();
        match PeerId::from_str(&peer) {
            Ok(p) => self.network.get_peer_status(&p),
            Err(err) => {
                warn!(
                    "Failed to parse peer id {}, peer info unavailable: {}",
                    peer,
                    err.to_string()
                );
                None
            }
        }
    }
    #[wasm_bindgen(js_name = "qualityOf")]
    pub fn quality_of(&self, peer: JsPeerId) -> f64 {
        let peer: String = peer.to_string().into();
        match PeerId::from_str(&peer) {
            Ok(p) => match self.network.get_peer_status(&p) {
                Some(v) => v.quality,
                _ => 0.0f64,
            },
            Err(err) => {
                warn!(
                    "Failed to parse peer id {}, using lowest possible quality: {}",
                    peer,
                    err.to_string()
                );
                0.0f64
            }
        }
    }

    #[wasm_bindgen]
    pub fn all(&self) -> Vec<JsString> {
        self.network
            .filter(|_| true)
            .iter()
            .map(|x| x.to_base58().into())
            .collect::<Vec<JsString>>()
    }

    /// Returns the quality of the network as a network health indicator.
    #[wasm_bindgen]
    pub fn health(&self) -> Health {
        self.network.get_health()
    }

    /// Total count of the peers observed withing the network
    #[wasm_bindgen]
    pub fn length(&self) -> usize {
        self.network.get_entries_length()
    }

    #[wasm_bindgen]
    pub fn debug_output(&self) -> String {
        self.network.debug_output()
    }
}
