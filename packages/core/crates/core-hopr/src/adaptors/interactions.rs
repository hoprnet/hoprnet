use futures::channel::mpsc::{channel, unbounded, Sender, UnboundedSender};
use futures::future::poll_fn;

use core_crypto::types::HalfKeyChallenge;
use core_types::acknowledgement::AcknowledgedTicket;
use utils_log::error;

#[cfg(feature = "wasm")]
pub mod wasm {
    use std::pin::Pin;

    use super::*;

    use core_packet::interaction::ApplicationData;
    use futures::Stream;
    use utils_log::debug;
    use wasm_bindgen::prelude::*;

    /// Helper loop ensuring conversion and enqueueing of events on acknowledgement
    pub fn spawn_ack_receiver_loop(on_ack: Option<js_sys::Function>) -> Option<UnboundedSender<HalfKeyChallenge>> {
        match on_ack {
            Some(on_ack_fn) => {
                let (tx, mut rx) = unbounded::<HalfKeyChallenge>();

                wasm_bindgen_futures::spawn_local(async move {
                    while let Some(ack) = poll_fn(|cx| Pin::new(&mut rx).poll_next(cx)).await {
                        if let Err(e) = on_ack_fn.call1(&JsValue::null(), &ack.into()) {
                            error!("failed to call on_ack closure: {:?}", e.as_string());
                        }
                    }
                });

                Some(tx)
            }
            None => None,
        }
    }

    /// Helper loop ensuring conversion and enqueueing of events on acknowledgement ticket
    pub fn spawn_ack_tkt_receiver_loop(
        on_ack_tkt: Option<js_sys::Function>,
    ) -> Option<UnboundedSender<AcknowledgedTicket>> {
        match on_ack_tkt {
            Some(on_ack_tkt_fn) => {
                let (tx, mut rx) = unbounded::<AcknowledgedTicket>();

                wasm_bindgen_futures::spawn_local(async move {
                    while let Some(ack) = poll_fn(|cx| Pin::new(&mut rx).poll_next(cx)).await {
                        if let Err(e) = on_ack_tkt_fn.call1(
                            &JsValue::null(),
                            &core_types::acknowledgement::wasm::AcknowledgedTicket::from(ack).into(),
                        ) {
                            error!("failed to call on_ack_ticket closure: {:?}", e.as_string());
                        }
                    }
                });

                Some(tx)
            }
            None => None,
        }
    }

    const ON_PACKET_QUEUE_SIZE: usize = 4096;

    /// Helper loop ensuring conversion and enqueueing of events on receiving the final packet
    pub fn spawn_on_final_packet_loop(on_final_packet: Option<js_sys::Function>) -> Option<Sender<ApplicationData>> {
        match on_final_packet {
            Some(on_msg_rcv) => {
                let (tx, mut rx) = channel::<ApplicationData>(ON_PACKET_QUEUE_SIZE);

                wasm_bindgen_futures::spawn_local(async move {
                    while let Some(packet) = poll_fn(|cx| Pin::new(&mut rx).poll_next(cx)).await {
                        debug!("wasm packet interaction loop received a new packet");

                        if let Ok(param) = serde_wasm_bindgen::to_value(&packet) {
                            if let Err(e) = on_msg_rcv.call1(&JsValue::null(), &param) {
                                error!("failed to call on_ack_ticket closure: {:?}", e.as_string());
                            }
                        } else {
                            error!("failed to serialize application data to JsValue");
                        }
                    }
                });

                Some(tx)
            }
            None => None,
        }
    }
}
