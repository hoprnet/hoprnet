use futures::channel::mpsc::{channel, unbounded, Sender, UnboundedSender};
use futures::future::poll_fn;

use core_crypto::types::HalfKeyChallenge;
use utils_log::error;

#[cfg(feature = "wasm")]
pub mod wasm {
    use std::pin::Pin;

    use super::*;

    use core_types::protocol::ApplicationData;
    use futures::Stream;
    use utils_log::debug;
    use wasm_bindgen::prelude::*;
    use utils_types::traits::BinarySerializable;

    /// Helper loop ensuring conversion and enqueueing of events on acknowledgement
    pub fn spawn_ack_receiver_loop(on_ack: js_sys::Function) -> UnboundedSender<HalfKeyChallenge> {
        let (tx, mut rx) = unbounded::<HalfKeyChallenge>();

        wasm_bindgen_futures::spawn_local(async move {
            while let Some(ack) = poll_fn(|cx| Pin::new(&mut rx).poll_next(cx)).await {
                if let Err(e) = on_ack.call1(&JsValue::null(), &ack.into()) {
                    error!("failed to call on_ack closure: {:?}", e.as_string());
                }
            }
        });

        tx
    }

    const ON_PACKET_QUEUE_SIZE: usize = 4096;

    /// Helper loop ensuring conversion and enqueueing of events on receiving the final packet
    pub fn spawn_on_final_packet_loop(on_final_packet: js_sys::Function) -> Sender<ApplicationData> {
        let (tx, mut rx) = channel::<ApplicationData>(ON_PACKET_QUEUE_SIZE);

        wasm_bindgen_futures::spawn_local(async move {
            while let Some(packet) = poll_fn(|cx| Pin::new(&mut rx).poll_next(cx)).await {
                debug!("wasm packet interaction loop received a new packet");
                if let Err(e) = on_final_packet.call1(&JsValue::null(), &packet.into()) {
                    error!("failed to call on_ack_ticket closure: {:?}", e.as_string());
                }
            }
        });

        tx
    }
}
