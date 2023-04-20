// use crate::heartbeat::Heartbeat;
use crate::{
    heartbeat::{Heartbeat, HeartbeatConfig, HeartbeatRequest},
    network::{wasm::WasmNetworkApi, Health, Network, PeerOrigin, PeerStatus},
    ping::{wasm::WasmPingApi, Ping, PingConfig},
};
use gloo_timers::future::sleep;
use js_sys::{Array, Date, Function, JsString, Map, Number, Promise, Reflect, Symbol, Uint8Array};
use libp2p::PeerId;
use std::{collections::HashMap, pin::Pin, str::FromStr, time::Duration};
use utils_log::{error, info, warn};
use utils_misc::{streaming_iterable::JsStreamingIterable, utils::wasm::js_map_to_hash_map};
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
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(is_type_of = AsyncIterable::looks_like_async_iterable, typescript_type = "AsyncIterable<Uint8Array>")]
    pub type AsyncIterable;
}

impl AsyncIterable {
    fn looks_like_async_iterable(it: &JsValue) -> bool {
        if !it.is_object() {
            return false;
        }

        let async_sym = Symbol::async_iterator();
        let async_iter_fn = match Reflect::get(it, async_sym.as_ref()) {
            Ok(f) => f,
            Err(_) => return false,
        };

        async_iter_fn.is_function()
    }
}

/// Special-purpose version of js_sys::IteratorNext for Uint8Arrays
#[wasm_bindgen]
extern "C" {
    #[derive(Clone, Debug)]
    #[wasm_bindgen]
    pub type Uint8ArrayIteratorNext;

    #[wasm_bindgen(method, getter, structural)]
    pub fn done(this: &Uint8ArrayIteratorNext) -> bool;

    #[wasm_bindgen(method, getter, structural)]
    pub fn value(this: &Uint8ArrayIteratorNext) -> Box<[u8]>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub type JsConnection;

    #[wasm_bindgen(method, getter, structural)]
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
    pub fn handle(this: &JsRegistrar, handler_function: &Closure<dyn FnMut(IncomingConnection) -> ()>) -> Promise;
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
pub struct CoreNetwork {
    components: JsLibp2p,
    network: Network<WasmNetworkApi>,
    pinger: Ping<WasmPingApi>,
    heartbeat: Heartbeat,
    send_message_cb: Function,
}

static PEER_METADATA_PROTOCOL_VERSION: &str = "protocol_version";

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
        on_finished_ping_cb: Function,
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

        let ping_api = WasmPingApi::new(environment_id.to_owned(), version.to_owned(), on_finished_ping_cb);

        let ping_config = PingConfig::new(
            max_parallel_pings.value_of() as usize,
            environment_id,
            version,
            Duration::from_secs(30),
        );

        let heartbeat_config = HeartbeatConfig::new(
            max_parallel_pings.value_of() as usize,
            heartbeat_variance.value_of() as u32,
            heartbeat_interval.value_of() as u32,
            heartbeat_threshold.value_of() as u64,
        );

        let pinger = Ping::new(ping_config, ping_api);
        let network = Network::new(me, quality_threshold.as_f64().unwrap(), network_api);
        let heartbeat = Heartbeat::new(heartbeat_config);

        Self {
            components,
            network,
            pinger,
            heartbeat,
            send_message_cb,
        }
    }

    #[wasm_bindgen]
    pub async fn start(&mut self) {
        // Cast to 'static to use in Closure
        let this = unsafe { std::mem::transmute::<&mut Self, &'static mut Self>(self) };
        let cb = Closure::<dyn FnMut(IncomingConnection) -> ()>::new(move |incoming: IncomingConnection| {
            let mut peer_metadata = HashMap::<String, String>::new();

            let proto_version = version_from_protocol(incoming.protocol());

            peer_metadata.insert(PEER_METADATA_PROTOCOL_VERSION.to_owned(), proto_version);

            let remote = PeerId::from_str(incoming.connection().remote_peer().to_string().as_str()).unwrap();

            if this.network.has(&remote) {
                this.network
                    .update_with_metadata(&remote, Ok(Date::now() as u64), Some(peer_metadata))
            } else {
                this.network
                    .add_with_metadata(&remote, PeerOrigin::IncomingConnection, Some(peer_metadata));
            }

            spawn_local(async move {
                match HeartbeatRequest::from(incoming.stream()).await {
                    Err(e) => error!("Failed processing heartbeat request {}", e),
                    Ok(()) => (),
                }
            });
        });

        JsFuture::from(
            self.components
                .get_registrar()
                .expect("Libp2p instance without registrar")
                .handle(&cb),
        )
        .await
        .expect("Could not register heartbeat handler");

        let this_ref = unsafe { std::mem::transmute::<&mut Self, &'static Self>(self) };

        let message_transport =
            move |msg: Box<[u8]>, peer: String| -> Pin<Box<dyn futures::Future<Output = Result<Box<[u8]>, String>>>> {
                Box::pin(async move {
                    let js_this = JsValue::null();
                    let data: JsValue = js_sys::Uint8Array::from(msg.as_ref()).into();
                    let peer: JsValue = JsString::from(peer.as_str()).into();

                    // call a send_msg_cb producing a JS promise that is further converted to a Future
                    // holding the reply of the pinged peer for the ping message.
                    match this_ref.send_message_cb.call2(&js_this, &data, &peer) {
                        Ok(r) => {
                            let promise = Promise::from(r);
                            let data = JsFuture::from(promise)
                                .await
                                .map(|x| Array::from(x.as_ref()).get(0))
                                .map(|x| Uint8Array::new(&x).to_vec().into_boxed_slice())
                                .map_err(|x| {
                                    x.dyn_ref::<JsString>()
                                        .map_or("Failed to send ping message".to_owned(), |x| -> String { x.into() })
                                });

                            data
                        }
                        Err(e) => {
                            error!(
                                "The message transport could not be established: {}",
                                e.as_string()
                                    .unwrap_or_else(|| { "The message transport failed with unknown error".to_owned() })
                                    .as_str()
                            );
                            Err(format!("Failed to extract transport error as string: {:?}", e))
                        }
                    }
                })
            };

        let this = unsafe { std::mem::transmute::<&mut Self, &'static mut Self>(self) };

        spawn_local(async move {
            while !this.heartbeat.ended {
                sleep(Duration::from_millis(this.heartbeat.config.heartbeat_interval as u64)).await;
                let threshold = Date::now() as u64 - this.heartbeat.config.heartbeat_threshold;
                info!("Checking nodes since {}", threshold);
                this.pinger
                    .ping_peers(this.network.find_peers_to_ping(threshold), &message_transport);
            }
        });

        // Leave callback to JS garbage collector
        cb.forget();
    }

    #[wasm_bindgen]
    pub async fn stop(&mut self) {
        self.heartbeat.ended = true;
    }

    #[wasm_bindgen(js_name = "pingNode")]
    pub async fn ping_node() {}

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

        let message_transport =
            |msg: Box<[u8]>, peer: String| -> Pin<Box<dyn futures::Future<Output = Result<Box<[u8]>, String>>>> {
                Box::pin(async move {
                    let this = JsValue::null();
                    let data: JsValue = js_sys::Uint8Array::from(msg.as_ref()).into();
                    let peer: JsValue = JsString::from(peer.as_str()).into();

                    // call a send_msg_cb producing a JS promise that is further converted to a Future
                    // holding the reply of the pinged peer for the ping message.
                    match self.send_message_cb.call2(&this, &data, &peer) {
                        Ok(r) => {
                            let promise = js_sys::Promise::from(r);
                            let data = JsFuture::from(promise)
                                .await
                                .map(|x| js_sys::Array::from(x.as_ref()).get(0))
                                .map(|x| js_sys::Uint8Array::new(&x).to_vec().into_boxed_slice())
                                .map_err(|x| {
                                    x.dyn_ref::<JsString>()
                                        .map_or("Failed to send ping message".to_owned(), |x| -> String { x.into() })
                                });

                            data
                        }
                        Err(e) => {
                            error!(
                                "The message transport could not be established: {}",
                                e.as_string()
                                    .unwrap_or_else(|| { "The message transport failed with unknown error".to_owned() })
                                    .as_str()
                            );
                            Err(format!("Failed to extract transport error as string: {:?}", e))
                        }
                    }
                })
            };

        self.pinger.ping_peers(converted, &message_transport).await;
    }

    #[wasm_bindgen]
    pub fn register(&mut self, peer: JsPeerId, origin: PeerOrigin) {
        self.register_with_metadata(peer, origin, &Map::new())
    }

    #[wasm_bindgen(js_name = "registerWithMetadata")]
    pub fn register_with_metadata(&mut self, peer: JsPeerId, origin: PeerOrigin, metadata: &js_sys::Map) {
        match PeerId::from_str(&peer.to_string().to_owned()) {
            Ok(p) => self
                .network
                .add_with_metadata(&p, origin, js_map_to_hash_map(&Map::new())),
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
    pub fn contains(&self, peer: JsString) -> bool {
        let peer: String = peer.into();
        match PeerId::from_str(&peer) {
            Ok(p) => self.network.has(&p),
            Err(err) => {
                warn!(
                    "Failed to parse peer id {}, network assumes it is not present: {}",
                    peer,
                    err.to_string()
                );
                false
            }
        }
    }

    #[wasm_bindgen]
    pub fn unregister(&mut self, peer: JsPeerId) {
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
        self.network.last_health
    }

    /// Total count of the peers observed withing the network
    #[wasm_bindgen]
    pub fn length(&self) -> usize {
        self.network.entries.len()
    }

    #[wasm_bindgen]
    pub fn debug_output(&self) -> String {
        self.network.debug_output()
    }
}
