//! Collection of objects and functionality allowing building of p2p or stream protocols for the higher business logic layers.
//!
//! ## Contents
//!
//! Supported protocol configurations:
//!
//! - `mix`
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

/// Coder and decoder for the transport binary protocol layer
mod codec;

/// Configuration of the protocol components.
pub mod config;
/// Errors produced by the crate.
pub mod errors;

/// Bloom filter for the transport layer.
pub mod bloom;
// protocols
/// `heartbeat` p2p protocol
pub mod heartbeat;
/// processor for the protocol
pub mod processor;

/// Stream processing utilities
pub mod stream;

pub mod timer;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_transport_identity::Multiaddr;
pub use timer::execute_on_tick;

use futures::{SinkExt, StreamExt};
use rust_stream_ext_concurrent::then_concurrent::StreamThenConcurrentExt;
use std::collections::HashMap;
use tracing::error;

use hopr_async_runtime::prelude::spawn;
use hopr_db_api::protocol::{HoprDbProtocolOperations, IncomingPacket};
use hopr_internal_types::protocol::{Acknowledgement, ApplicationData};
use hopr_network_types::prelude::ResolvedTransportRouting;
use hopr_transport_identity::PeerId;

pub use processor::DEFAULT_PRICE_PER_PACKET;
use processor::{PacketSendFinalizer, PacketUnwrapping, PacketWrapping};

const HOPR_PACKET_SIZE: usize = hopr_crypto_packet::prelude::HoprPacket::SIZE;

pub type HoprBinaryCodec = crate::codec::FixedLengthCodec<HOPR_PACKET_SIZE>;
pub const CURRENT_HOPR_MSG_PROTOCOL: &str = "/hopr/mix/1.0.0";

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::{MultiCounter, SimpleCounter};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    // packet
    static ref METRIC_PACKET_COUNT: MultiCounter = MultiCounter::new(
        "hopr_packets_count",
        "Number of processed packets of different types (sent, received, forwarded)",
        &["type"]
    ).unwrap();
    static ref METRIC_PACKET_COUNT_PER_PEER: MultiCounter = MultiCounter::new(
        "hopr_packets_per_peer_count",
        "Number of processed packets to/from distinct peers",
        &["peer", "direction"]
    ).unwrap();
    static ref METRIC_REPLAYED_PACKET_COUNT: SimpleCounter = SimpleCounter::new(
        "hopr_replayed_packet_count",
        "The total count of replayed packets during the packet processing pipeline run",
    ).unwrap();
    static ref METRIC_REJECTED_TICKETS_COUNT: SimpleCounter =
        SimpleCounter::new("hopr_rejected_tickets_count", "Number of rejected tickets").unwrap();
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, strum::Display)]
pub enum ProtocolProcesses {
    #[strum(to_string = "HOPR [msg] - ingress")]
    MsgIn,
    #[strum(to_string = "HOPR [msg] - egress")]
    MsgOut,
    #[strum(to_string = "HOPR [msg] - mixer")]
    Mixer,
    #[strum(to_string = "bloom filter persistence (periodic)")]
    BloomPersist,
}
/// Processed indexer generated events.
#[derive(Debug, Clone)]
pub enum PeerDiscovery {
    Allow(PeerId),
    Ban(PeerId),
    Announce(PeerId, Vec<Multiaddr>),
}

/// Run all processes responsible for handling the msg and acknowledgment protocols.
///
/// The pipeline does not handle the mixing itself, that needs to be injected as a separate process
/// overlayed on top of the `wire_msg` Stream or Sink.
#[allow(clippy::too_many_arguments)]
pub async fn run_msg_ack_protocol<Db>(
    packet_cfg: processor::PacketInteractionConfig,
    db: Db,
    bloom_filter_persistent_path: Option<String>,
    wire_msg: (
        impl futures::Sink<(PeerId, Box<[u8]>)> + Clone + Unpin + Send + Sync + 'static,
        impl futures::Stream<Item = (PeerId, Box<[u8]>)> + Send + Sync + 'static,
    ),
    api: (
        impl futures::Sink<ApplicationData> + Send + Sync + 'static,
        impl futures::Stream<Item = (ApplicationData, ResolvedTransportRouting, PacketSendFinalizer)>
            + Send
            + Sync
            + 'static,
    ),
) -> HashMap<ProtocolProcesses, hopr_async_runtime::prelude::JoinHandle<()>>
where
    Db: HoprDbProtocolOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    let me = packet_cfg.packet_keypair.clone();

    let mut processes = HashMap::new();

    #[cfg(all(feature = "prometheus", not(test)))]
    {
        // Initialize the lazy statics here
        // lazy_static::initialize(&METRIC_RECEIVED_ACKS);
        // lazy_static::initialize(&METRIC_SENT_ACKS);
        // lazy_static::initialize(&METRIC_TICKETS_COUNT);
        lazy_static::initialize(&METRIC_PACKET_COUNT);
        lazy_static::initialize(&METRIC_PACKET_COUNT_PER_PEER);
        lazy_static::initialize(&METRIC_REPLAYED_PACKET_COUNT);
        lazy_static::initialize(&METRIC_REJECTED_TICKETS_COUNT);
    }

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
                "persisting the bloom filter to disk".into(),
            ))),
        );
        tbf
    } else {
        bloom::WrappedTagBloomFilter::new("no_tbf".into())
    };

    let msg_processor_read = processor::PacketProcessor::new(db.clone(), packet_cfg);
    let msg_processor_write = msg_processor_read.clone();

    let msg_to_send_tx = wire_msg.0.clone();
    processes.insert(
        ProtocolProcesses::MsgOut,
        spawn(async move {
            let _neverending = api
                .1
                .then_concurrent(|(data, routing, finalizer)| {
                    let msg_processor = msg_processor_write.clone();

                    async move {
                        match PacketWrapping::send(&msg_processor, data, routing).await {
                            Ok(v) => {
                                let v: (PeerId, Box<[u8]>) = (v.next_hop.into(), v.data);
                                #[cfg(all(feature = "prometheus", not(test)))]
                                {
                                    METRIC_PACKET_COUNT_PER_PEER.increment(&["out", &v.0.to_string()]);
                                    METRIC_PACKET_COUNT.increment(&["sent"]);
                                }
                                finalizer.finalize(Ok(()));
                                Some(v)
                            }
                            Err(e) => {
                                finalizer.finalize(Err(e));
                                None
                            }
                        }
                    }
                })
                .filter_map(|v| async move { v })
                .map(Ok)
                .forward(msg_to_send_tx)
                .await;
        }),
    );

    let msg_to_send_tx = wire_msg.0.clone();
    let db_for_recv = db.clone();
    let me_for_recv = me.clone();
    processes.insert(
        ProtocolProcesses::MsgIn,
        spawn(async move {
            let _neverending = wire_msg
                .1
                .then_concurrent(move |(peer, data)| {
                    let msg_processor = msg_processor_read.clone();
                    let db = db_for_recv.clone();
                    let mut msg_to_send_tx = msg_to_send_tx.clone();
                    let me = me.clone();

                    async move {
                        let res = msg_processor.recv(&peer, data).await.map_err(move |e| (peer, e));
                        if let Err((peer, e)) = &res {
                            #[cfg(all(feature = "prometheus", not(test)))]
                            if let hopr_crypto_packet::errors::PacketError::TicketValidation(_) = e {
                                METRIC_REJECTED_TICKETS_COUNT.increment();
                            }

                            error!(peer = %peer, error = %e, "Failed to process the received message");

                            let peer: OffchainPublicKey = match peer.try_into() {
                                Ok(p) => p,
                                Err(error) => {
                                    tracing::warn!(%peer, %error, "Dropping packet – cannot convert peer id");
                                    return None;
                                }
                            };

                            // send random signed acknowledgement to give feedback to the sender
                            let ack = Acknowledgement::random(&me);

                            match db
                                .to_send_no_ack(ack.as_ref().to_vec().into_boxed_slice(), peer)
                                .await {
                                    Ok(ack_packet) => {
                                        msg_to_send_tx
                                            .send((
                                                ack_packet.next_hop.into(),
                                                ack_packet.data,
                                            ))
                                            .await
                                            .unwrap_or_else(|_e| {
                                                error!("Failed to forward an acknowledgement for a failed packet recv to the transport layer");
                                            });
                                    },
                                    Err(error) => tracing::error!(%error, "Failed to create random ack packet for a failed receive"),
                                }
                        }

                        res.ok().flatten()
                    }
                })
                .filter_map(move |maybe_packet| {
                    let tbf = tbf.clone();

                    async move {
                    if let Some(packet) = maybe_packet {
                        match packet {
                            IncomingPacket::Final { packet_tag, .. }
                            | IncomingPacket::Forwarded { packet_tag, .. } => {
                                if tbf.is_tag_replay(&packet_tag).await {
                                    #[cfg(all(feature = "prometheus", not(test)))]
                                    METRIC_REPLAYED_PACKET_COUNT.increment();

                                    None
                                } else {
                                    Some(packet)
                                }
                            }
                        }
                    } else {
                        None
                    }
                }
                })
                .then_concurrent(move |packet| {
                    let mut msg_to_send_tx = wire_msg.0.clone();
                    let db = db.clone();
                    let me = me_for_recv.clone();

                    async move {

                    match packet {
                        IncomingPacket::Final {
                            previous_hop,
                            plain_text,
                            ack_key,
                            ..
                        } => {
                                let ack = Acknowledgement::new(ack_key, &me);
                                if let Ok(ack_packet) = db
                                    .to_send_no_ack(ack.as_ref().to_vec().into_boxed_slice(), previous_hop)
                                    .await
                                    .inspect_err(|error| tracing::error!(error = %error, "Failed to create ack packet for a received message"))
                                    {
                                        msg_to_send_tx
                                            .send((
                                                ack_packet.next_hop.into(),
                                                ack_packet.data,
                                            ))
                                            .await
                                            .unwrap_or_else(|_e| {
                                                error!("Failed to send an acknowledgement for a received packet to the transport layer");
                                            });
                                    }

                                    Some(plain_text)
                                }
                                IncomingPacket::Forwarded {
                                    previous_hop,
                                    next_hop,
                                    data,
                                    ack,
                                    ..
                                } => {
                                    msg_to_send_tx
                                        .send((
                                            next_hop.into(),
                                            data,
                                        ))
                                        .await
                                        .unwrap_or_else(|_e| {
                                            error!("Failed to forward a packet to the transport layer");
                                        });

                                    if let Ok(ack_packet) = db
                                        .to_send_no_ack(ack.as_ref().to_vec().into_boxed_slice(), previous_hop)
                                        .await
                                        .inspect_err(|error| tracing::error!(error = %error, "Failed to create ack packet for a relayed message"))
                                    {
                                        msg_to_send_tx
                                            .send((
                                                ack_packet.next_hop.into(),
                                                ack_packet.data,
                                            ))
                                            .await
                                            .unwrap_or_else(|_e| {
                                                error!("Failed to send an acknowledgement for a relayed packet to the transport layer");
                                            });
                                    }
                            None
                        }
                    }
                }})
                .filter_map(|maybe_data| async move {
                    if let Some(data) = maybe_data {
                        ApplicationData::from_bytes(data.as_ref()).inspect_err(|error| tracing::error!(error = %error, "Failed to decode application data")).ok()
                    } else {
                        None
                    }
                })
                .map(Ok)
                .forward(api.0)
                .await;
        }),
    );

    processes
}
