use std::{pin::Pin, sync::Arc};

use async_std::sync::RwLock;
use core_path::channel_graph::ChannelGraph;
use core_strategy::strategy::MultiStrategy;
use core_types::{acknowledgement::AcknowledgedTicket, channels::ChannelEntry};
use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver},
    future::poll_fn,
    Stream,
};

use core_transport::{ApplicationData, HalfKeyChallenge, TransportOutput};

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::spawn_local;

use utils_types::primitives::Address;
#[cfg(all(feature = "wasm", not(test)))]
use wasm_bindgen_futures::spawn_local;

/// Helper loop ensuring processing of winning acknowledge tickets
pub fn spawn_channel_update_handling<Db>(
    me: Address,
    db: Arc<RwLock<Db>>,
    multi_strategy: Arc<MultiStrategy>,
    channel_graph: Arc<RwLock<ChannelGraph>>,
) -> futures::channel::mpsc::UnboundedSender<ChannelEntry>
where
    Db: core_ethereum_db::traits::HoprCoreEthereumDbActions + 'static,
{
    let (on_channel_event_tx, mut rx) = unbounded::<ChannelEntry>();

    spawn_local(async move {
        while let Some(channel) = poll_fn(|cx| Pin::new(&mut rx).poll_next(cx)).await {
            let maybe_direction = channel.direction(&me);
            let change = channel_graph.write().await.update_channel(channel);

            // Check if this is our own channel
            if let Some(own_channel_direction) = maybe_direction {
                if let Some(change_set) = change {
                    for channel_change in change_set {
                        let _ = core_strategy::strategy::SingularStrategy::on_own_channel_changed(
                            &*multi_strategy,
                            &channel,
                            own_channel_direction,
                            channel_change,
                        )
                        .await;

                        // Cleanup invalid tickets from the DB if epoch has changed
                        // TODO: this should be moved somewhere else once event broadcasts are implemented
                        if let core_types::channels::ChannelChange::Epoch { .. } = channel_change {
                            let _ = db.write().await.cleanup_invalid_channel_tickets(&channel).await;
                        }
                    }
                } else if channel.status == core_types::channels::ChannelStatus::Open {
                    // Emit Opening event if the channel did not exist before in the graph
                    let _ = core_strategy::strategy::SingularStrategy::on_own_channel_changed(
                        &*multi_strategy,
                        &channel,
                        own_channel_direction,
                        core_types::channels::ChannelChange::Status {
                            left: core_types::channels::ChannelStatus::Closed,
                            right: core_types::channels::ChannelStatus::Open,
                        },
                    )
                    .await;
                }
            }
        }
    });

    on_channel_event_tx
}

/// Helper loop ensuring processing of winning acknowledge tickets
pub fn spawn_ack_winning_ticket_handling(
    multi_strategy: Arc<MultiStrategy>,
) -> futures::channel::mpsc::UnboundedSender<AcknowledgedTicket> {
    let (on_ack_tkt_tx, mut rx) = unbounded::<AcknowledgedTicket>();
    spawn_local(async move {
        while let Some(ack) = poll_fn(|cx| Pin::new(&mut rx).poll_next(cx)).await {
            let _ =
                core_strategy::strategy::SingularStrategy::on_acknowledged_winning_ticket(&*multi_strategy, &ack).await;
        }
    });

    on_ack_tkt_tx
}

/// Helper loop ensuring enqueueing of transport events going out of the module
pub fn spawn_transport_output<F1, F2>(mut rx: UnboundedReceiver<TransportOutput>, on_final_packet: F1, on_ack: F2)
where
    F1: Fn(ApplicationData) + 'static,
    F2: Fn(HalfKeyChallenge) + 'static,
{
    spawn_local(async move {
        while let Some(output) = poll_fn(|cx| Pin::new(&mut rx).poll_next(cx)).await {
            match output {
                TransportOutput::Received(msg) => (on_final_packet)(msg),
                TransportOutput::Sent(ack_challenge) => (on_ack)(ack_challenge),
            }
        }
    });
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        /// EventEmitter object used to delegate `on` calls in WSS
        pub type WasmHoprMessageEmitter;

        #[wasm_bindgen(method)]
        pub fn delegate_on(this: &WasmHoprMessageEmitter, event: js_sys::JsString, callback: js_sys::Function);
    }
}
