use futures::channel::mpsc::{channel, Sender, unbounded, UnboundedSender};
use futures::future::poll_fn;

use core_crypto::types::HalfKeyChallenge;
use core_types::acknowledgement::AcknowledgedTicket;
use utils_log::error;
use utils_types::traits::BinarySerializable;


#[cfg(feature = "wasm")]
pub mod wasm {
    use std::pin::Pin;

    use super::*;

    use futures::Stream;
    use utils_log::debug;
    use wasm_bindgen::prelude::*;

    pub fn spawn_ack_receiver_loop(on_ack: Option<js_sys::Function>) -> Option<UnboundedSender<HalfKeyChallenge>> {
        match on_ack {
            Some(on_ack_fn) => {
                let (tx, mut rx) = unbounded::<HalfKeyChallenge>();

                wasm_bindgen_futures::spawn_local(async move {
                    while let Some(ack) = poll_fn(|cx| Pin::new(&mut rx).poll_next(cx)).await {
                        let param: JsValue = js_sys::Uint8Array::from(ack.to_bytes().as_ref()).into();
                        if let Err(e) = on_ack_fn.call1(&JsValue::null(), &param) {
                            error!("failed to call on_ack closure: {:?}", e.as_string());
                        }
                    }
                });

                Some(tx)
            },
            None => None,
        }
    }

    pub fn spawn_ack_tkt_receiver_loop(on_ack_tkt: Option<js_sys::Function>) -> Option<UnboundedSender<AcknowledgedTicket>>  {
        match on_ack_tkt {
            Some(on_ack_tkt_fn) => {
                let (tx, mut rx) = unbounded::<AcknowledgedTicket>();

                wasm_bindgen_futures::spawn_local(async move {
                    while let Some(ack) = poll_fn(|cx| Pin::new(&mut rx).poll_next(cx)).await {
                        let param: JsValue = js_sys::Uint8Array::from(ack.to_bytes().as_ref()).into();
                        if let Err(e) = on_ack_tkt_fn.call1(&JsValue::null(), &param) {
                            error!("failed to call on_ack_ticket closure: {:?}", e.as_string());
                        }
                    }
                });

                Some(tx)
            },
            None => None,
        }
    }

    const ON_PACKET_QUEUE_SIZE: usize = 4096;

    pub fn spawn_on_final_packet_loop(on_final_packet: Option<js_sys::Function>) -> Option<Sender<Box<[u8]>>>  {
        match on_final_packet {
            Some(on_msg_rcv) => {
                let (tx, mut rx) = channel::<Box<[u8]>>(ON_PACKET_QUEUE_SIZE);

                wasm_bindgen_futures::spawn_local(async move {
                    while let Some(packet) = poll_fn(|cx| Pin::new(&mut rx).poll_next(cx)).await {
                        debug!("wasm packet interaction loop received a new packet");

                        let param: JsValue = js_sys::Uint8Array::from(packet.as_ref()).into();
                        if let Err(e) = on_msg_rcv.call1(&JsValue::null(), &param) {
                            error!("failed to call on_ack_ticket closure: {:?}", e.as_string());
                        }
                    }
                });

                Some(tx)
            },
            None => None,
        }
    }
}


