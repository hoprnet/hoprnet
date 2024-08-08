//! Collection of objects and functionality allowing building of p2p or stream protocols for the higher business logic layers.
//!
//! ## Contents
//!
//! Supported protocol configurations:
//!
//! - `msg`
//! - `ack`
//! - `heartbeat`
//! - `ticket_aggregation`
//!
//! Supported protocol processors:
//!
//! - `ticket_aggregation`
//!
//! ### `ticket_aggregation`
//!
//! Ticket aggregation processing mechanism is responsible for ingesting the ticket aggregation related requests:
//!
//! - `Receive(PeerId, U)`,
//! - `Reply(PeerId, std::result::Result<Ticket, String>, T)`,
//! - `Send(PeerId, Vec<AcknowledgedTicket>, TicketAggregationFinalizer)`,
//!
//! where `U` is the type of an aggregated ticket extractable (`ResponseChannel<Result<Ticket, String>>`) and `T` represents a network negotiated identifier (`RequestId`).
//!
//! In broader context the protocol flow is as follows:
//!
//! 1. requesting ticket aggregation
//!
//!    - the peer A desires to aggregate tickets, collects the tickets into a data collection and sends a request containing the collection to aggregate `Vec<AcknowledgedTicket>` to peer B using the `Send` mechanism
//!
//! 2. responding to ticket aggregation
//!
//!    - peer B obtains the request from peer A, performs the ticket aggregation and returns a result of that operation in the form of `std::result::Result<Ticket, String>` using the `Reply` mechanism
//!
//! 3. accepting the aggregated ticket
//!    - peer A receives the aggregated ticket using the `Receive` mechanism
//!
//! Furthermore, apart from the basic positive case scenario, standard mechanics of protocol communication apply:
//!
//! - the requesting side can time out, if the responding side takes too long to provide an aggregated ticket, in which case the ticket is not considered aggregated, even if eventually an aggregated ticket is delivered
//! - the responder can fail to aggregate tickets in which case it replies with an error string describing the failure reason and it is the requester's responsibility to handle the negative case as well
//!   - in the absence of response, the requester will time out
//!

/// Configuration of the protocol components.
pub mod config;
/// Errors produced by the crate.
pub mod errors;

/// Bloom filter for the transport layer.
pub mod bloom;
// protocols
/// `ack` p2p protocol
pub mod ack;
/// `heartbeat` p2p protocol
pub mod heartbeat;
/// `msg` p2p protocol
pub mod msg;
/// `ticket_aggregation` p2p protocol
pub mod ticket_aggregation;

pub mod timer;
use core_path::path::TransportPath;
use hopr_crypto_types::keypairs::ChainKeypair;
pub use timer::execute_on_tick;

use futures::{SinkExt, StreamExt};
pub use msg::processor::DEFAULT_PRICE_PER_PACKET;
use msg::processor::{PacketSendFinalizer, PacketUnwrapping, PacketWrapping};

use libp2p::PeerId;
use rust_stream_ext_concurrent::then_concurrent::StreamThenConcurrentExt;
use std::collections::HashMap;
use tracing::error;

use hopr_async_runtime::prelude::spawn;
use hopr_db_api::protocol::HoprDbProtocolOperations;
use hopr_internal_types::{
    protocol::{Acknowledgement, ApplicationData},
    tickets::AcknowledgedTicket,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ProtocolProcesses {
    AckIn,
    AckOut,
    MsgIn,
    MsgOut,
    BloomPersist,
}

pub async fn run_msg_ack_protocol<Db>(
    cfg: msg::processor::PacketInteractionConfig,
    db: Db,
    me_onchain: &ChainKeypair,
    bloom_filter_persistent_path: Option<String>,
    on_ack_ticket: impl futures::Sink<AcknowledgedTicket> + Send + Sync + 'static,
    wire_ack: (
        impl futures::Sink<(PeerId, Acknowledgement)> + Send + Sync + 'static,
        impl futures::Stream<Item = (PeerId, Acknowledgement)> + Send + Sync + 'static,
    ),
    wire_msg: (
        impl futures::Sink<(PeerId, Box<[u8]>)> + Clone + Unpin + Send + Sync + 'static,
        impl futures::Stream<Item = (PeerId, Box<[u8]>)> + Send + Sync + 'static,
    ),
    api: (
        impl futures::Sink<ApplicationData> + Send + Sync + 'static,
        impl futures::Stream<Item = (ApplicationData, TransportPath, PacketSendFinalizer)> + Send + Sync + 'static,
    ),
) -> HashMap<ProtocolProcesses, hopr_async_runtime::prelude::JoinHandle<()>>
where
    Db: HoprDbProtocolOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    let mut processes = HashMap::new();

    let tbf = if let Some(bloom_filter_persistent_path) = bloom_filter_persistent_path {
        let tbf = bloom::WrappedTagBloomFilter::new(bloom_filter_persistent_path);
        let tbf_2 = tbf.clone();
        processes.insert(
            ProtocolProcesses::BloomPersist,
            spawn(Box::pin(execute_on_tick(
                std::time::Duration::from_secs(90),
                move || {
                    let tbf_clone = tbf_2.clone();

                    async move { tbf_clone.save().await }
                },
            ))),
        );
        tbf
    } else {
        bloom::WrappedTagBloomFilter::new("no_tbf".into())
    };

    let ack_processor_read = ack::processor::AcknowledgementProcessor::new(db.clone(), me_onchain);
    let ack_processor_write = ack_processor_read.clone();
    let msg_processor_read = msg::processor::PacketProcessor::new(db.clone(), tbf, cfg);
    let msg_processor_write = msg_processor_read.clone();

    processes.insert(
        ProtocolProcesses::AckIn,
        spawn(async move {
            let _neverending = wire_ack
                .1
                .then_concurrent(move |(peer, ack)| {
                    let ack_processor = ack_processor_read.clone();

                    async move { ack_processor.recv(&peer, ack).await }
                })
                .filter_map(|v| async move {
                    if let Ok(ack::processor::AckResult::RelayerWinning(acknowledged_ticket)) = v {
                        Some(acknowledged_ticket)
                    } else {
                        None
                    }
                })
                .map(Ok)
                .forward(on_ack_ticket)
                .await;
        }),
    );

    let (internal_ack_send, internal_ack_rx) = futures::channel::mpsc::unbounded::<(PeerId, Acknowledgement)>();

    processes.insert(
        ProtocolProcesses::AckOut,
        spawn(async move {
            let _neverending = internal_ack_rx
                .then_concurrent(move |(peer, ack)| {
                    let ack_processor = ack_processor_write.clone();

                    async move { (peer, ack_processor.send(&peer, ack).await) }
                })
                .map(Ok)
                .forward(wire_ack.0)
                .await;
        }),
    );

    let msg_to_send_tx = wire_msg.0.clone();
    processes.insert(
        ProtocolProcesses::MsgOut,
        spawn(async move {
            let _neverending = api
                .1
                .then_concurrent(|(data, path, finalizer)| {
                    let msg_processor = msg_processor_write.clone();

                    async move {
                        let p = PacketWrapping::send(&msg_processor, data, path).await;
                        finalizer.finalize();
                        p
                    }
                })
                .filter_map(|v| async move {
                    if let Ok((peer, octets)) = v {
                        Some((peer, octets))
                    } else {
                        None
                    }
                })
                // delay purposefully isolated into a separate concurrent task
                .then_concurrent(|v| async {
                    msg::processor::Delayer::new(msg::mixer::MixerConfig::default())
                        .add_delay()
                        .await;
                    v
                })
                .map(Ok)
                .forward(wire_msg.0)
                .await;
        }),
    );

    processes.insert(
        ProtocolProcesses::MsgIn,
        spawn(async move {
            let _neverending = wire_msg
                .1
                .then_concurrent(move |(peer, data)| {
                    let msg_processor = msg_processor_read.clone();

                    async move { msg_processor.recv(&peer, data).await }
                })
                .filter_map(move |v| {
                    let mut internal_ack_send = internal_ack_send.clone();
                    let mut msg_to_send_tx = msg_to_send_tx.clone();

                    async move {
                        match v {
                            Ok(v) => match v {
                                msg::processor::RecvOperation::Receive { data, ack } => {
                                    internal_ack_send.send((ack.peer, ack.ack)).await.unwrap_or_else(|e| {
                                        error!("Failed to forward an acknowledgement to the transport layer: {e}");
                                    });
                                    Some(data)
                                }
                                msg::processor::RecvOperation::Forward { msg, ack } => {
                                    msg_to_send_tx.send((msg.peer, msg.data)).await.unwrap_or_else(|_e| {
                                        error!("Failed to forward a message to the transport layer");
                                    });
                                    internal_ack_send.send((ack.peer, ack.ack)).await.unwrap_or_else(|e| {
                                        error!("Failed to forward an acknowledgement to the transport layer: {e}");
                                    });
                                    None
                                }
                            },
                            Err(e) => {
                                error!("Failed to process received message: {e}");
                                None
                            }
                        }
                    }
                })
                .map(Ok)
                .forward(api.0)
                .await;
        }),
    );

    processes
}
