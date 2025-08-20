//! Collection of objects and functionality allowing building of p2p or stream protocols for the higher business logic
//! layers.
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
//! where `U` is the type of an aggregated ticket extractable (`ResponseChannel<Result<Ticket, String>>`) and `T`
//! represents a network negotiated identifier (`RequestId`).
//!
//! In broader context the protocol flow is as follows:
//!
//! 1. requesting ticket aggregation
//!
//!    - the peer A desires to aggregate tickets, collects the tickets into a data collection and sends a request
//!      containing the collection to aggregate `Vec<AcknowledgedTicket>` to peer B using the `Send` mechanism
//!
//! 2. responding to ticket aggregation
//!
//!    - peer B obtains the request from peer A, performs the ticket aggregation and returns a result of that operation
//!      in the form of `std::result::Result<Ticket, String>` using the `Reply` mechanism
//!
//! 3. accepting the aggregated ticket
//!    - peer A receives the aggregated ticket using the `Receive` mechanism
//!
//! Furthermore, apart from the basic positive case scenario, standard mechanics of protocol communication apply:
//!
//! - the requesting side can time out, if the responding side takes too long to provide an aggregated ticket, in which
//!   case the ticket is not considered aggregated, even if eventually an aggregated ticket is delivered
//! - the responder can fail to aggregate tickets in which case it replies with an error string describing the failure
//!   reason and it is the requester's responsibility to handle the negative case as well
//!   - in the absence of response, the requester will time out

/// Coder and decoder for the transport binary protocol layer
mod codec;

/// Configuration of the protocol components.
pub mod config;
/// Errors produced by the crate.
pub mod errors;

// protocols
/// `heartbeat` p2p protocol
pub mod heartbeat;
/// processor for the protocol
pub mod processor;

/// Stream processing utilities
pub mod stream;

pub mod timer;

/// Allows capturing dissected HOPR packets before they are processed by the transport.
///
/// Requires the `capture` feature to be enabled.
#[cfg(feature = "capture")]
mod capture;

use std::collections::HashMap;

use futures::{SinkExt, StreamExt};
use hopr_async_runtime::spawn_as_abortable;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_db_api::protocol::{HoprDbProtocolOperations, IncomingPacket};
use hopr_internal_types::{prelude::HoprPseudonym, protocol::Acknowledgement};
use hopr_network_types::prelude::ResolvedTransportRouting;
use hopr_transport_bloom::persistent::WrappedTagBloomFilter;
use hopr_transport_identity::{Multiaddr, PeerId};
use hopr_transport_packet::prelude::ApplicationData;
use rust_stream_ext_concurrent::then_concurrent::StreamThenConcurrentExt;
use tracing::{Instrument, error, trace, warn};

use crate::processor::{PacketSendFinalizer, PacketUnwrapping, PacketWrapping};
pub use crate::{processor::DEFAULT_PRICE_PER_PACKET, timer::execute_on_tick};

const HOPR_PACKET_SIZE: usize = hopr_crypto_packet::prelude::HoprPacket::SIZE;
const SLOW_OP_MS: u128 = 150;

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
    #[cfg(feature = "capture")]
    #[strum(to_string = "packet capture")]
    Capture,
}
/// Processed indexer generated events.
#[derive(Debug, Clone)]
pub enum PeerDiscovery {
    Allow(PeerId),
    Ban(PeerId),
    Announce(PeerId, Vec<Multiaddr>),
}

#[cfg(feature = "capture")]
fn inspect_ticket_data_in_packet(raw_packet: &[u8]) -> &[u8] {
    use hopr_primitive_types::traits::BytesEncodable;
    if raw_packet.len() >= hopr_internal_types::tickets::Ticket::SIZE {
        &raw_packet[raw_packet.len() - hopr_internal_types::tickets::Ticket::SIZE..]
    } else {
        &[]
    }
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
        impl futures::Sink<(HoprPseudonym, ApplicationData)> + Send + Sync + 'static,
        impl futures::Stream<Item = (ApplicationData, ResolvedTransportRouting, PacketSendFinalizer)>
        + Send
        + Sync
        + 'static,
    ),
) -> HashMap<ProtocolProcesses, hopr_async_runtime::AbortHandle>
where
    Db: HoprDbProtocolOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    let me = packet_cfg.packet_keypair.clone();

    #[cfg(feature = "capture")]
    let me_pub = *hopr_crypto_types::keypairs::Keypair::public(&me);

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

    #[cfg(feature = "capture")]
    let capture = {
        use std::str::FromStr;
        let writer: Box<dyn capture::PacketWriter + Send + 'static> =
            if let Ok(desc) = std::env::var("HOPR_CAPTURE_PACKETS") {
                if let Ok(udp_writer) = std::net::SocketAddr::from_str(&desc)
                    .map_err(std::io::Error::other)
                    .and_then(capture::UdpPacketDump::new)
                {
                    warn!("udp packet capture initialized to {desc}");
                    Box::new(udp_writer)
                } else if let Ok(pcap_writer) = std::fs::File::create(&desc).and_then(capture::PcapPacketWriter::new) {
                    warn!("pcap file packet capture initialized to {desc}");
                    Box::new(pcap_writer)
                } else {
                    error!(desc, "failed to create packet capture: invalid socket address or file");
                    Box::new(capture::NullWriter)
                }
            } else {
                warn!("no packet capture specified");
                Box::new(capture::NullWriter)
            };
        let (capture, ah) = capture::packet_capture_channel(writer);
        processes.insert(ProtocolProcesses::Capture, ah);
        capture
    };

    let tbf = if let Some(bloom_filter_persistent_path) = bloom_filter_persistent_path {
        let tbf = WrappedTagBloomFilter::new(bloom_filter_persistent_path);
        let tbf_2 = tbf.clone();
        processes.insert(
            ProtocolProcesses::BloomPersist,
            spawn_as_abortable!(Box::pin(execute_on_tick(
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
        WrappedTagBloomFilter::new("no_tbf".into())
    };

    let msg_processor_read = processor::PacketProcessor::new(db.clone(), packet_cfg);
    let msg_processor_write = msg_processor_read.clone();

    #[cfg(feature = "capture")]
    let capture_clone = capture.clone();

    let msg_to_send_tx = wire_msg.0.clone();
    processes.insert(
        ProtocolProcesses::MsgOut,
        spawn_as_abortable!(async move {
            let _neverending = api
                .1
                .then_concurrent(|(data, routing, finalizer)| {
                    let msg_processor = msg_processor_write.clone();

                    #[cfg(feature = "capture")]
                    let (mut capture_clone, data_clone, num_surbs) =
                        (capture_clone.clone(), data.clone(), routing.count_return_paths() as u8);

                    async move {
                        match PacketWrapping::send(&msg_processor, data, routing).await {
                            Ok(v) => {
                                #[cfg(all(feature = "prometheus", not(test)))]
                                {
                                    METRIC_PACKET_COUNT_PER_PEER.increment(&[&v.next_hop.to_string(), "out"]);
                                    METRIC_PACKET_COUNT.increment(&["sent"]);
                                }
                                finalizer.finalize(Ok(()));

                                #[cfg(feature = "capture")]
                                let _ = capture_clone.try_send(
                                    capture::PacketBeforeTransit::OutgoingPacket {
                                        me: me_pub,
                                        next_hop: v.next_hop,
                                        num_surbs,
                                        is_forwarded: false,
                                        data: data_clone.to_bytes().into_vec().into(),
                                        ack_challenge: v.ack_challenge.as_ref().into(),
                                        ticket: inspect_ticket_data_in_packet(&v.data).into(),
                                    }
                                    .into(),
                                );

                                Some((v.next_hop.into(), v.data))
                            }
                            Err(e) => {
                                finalizer.finalize(Err(e));
                                None
                            }
                        }
                    }
                })
                .filter_map(futures::future::ready)
                .inspect(|(peer, _)| trace!(%peer, "protocol message out"))
                .map(Ok)
                .forward(msg_to_send_tx)
                .instrument(tracing::trace_span!("msg protocol processing - outgoing"))
                .await;
        }),
    );

    let msg_to_send_tx = wire_msg.0.clone();
    let db_for_recv = db.clone();
    let me_for_recv = me.clone();

    #[cfg(feature = "capture")]
    let capture_clone = capture.clone();

    processes.insert(
        ProtocolProcesses::MsgIn,
        spawn_as_abortable!(async move {
            let _neverending = wire_msg
                .1
                .then_concurrent(move |(peer, data)| {
                    let msg_processor = msg_processor_read.clone();
                    let db = db_for_recv.clone();
                    let mut msg_to_send_tx = msg_to_send_tx.clone();
                    let me = me.clone();

                    trace!(%peer, "protocol message in");

                    #[cfg(feature = "capture")]
                    let (mut capture_clone, ticket_data_clone) = (
                        capture.clone(),
                        inspect_ticket_data_in_packet(&data).to_vec()
                    );

                    async move {
                        let now = std::time::Instant::now();
                        let res = msg_processor.recv(&peer, data).await.map_err(move |e| (peer, e));
                        let elapsed = now.elapsed();
                        if elapsed.as_millis() > SLOW_OP_MS {
                            warn!(%peer, ?elapsed, "msg_processor.recv took too long");
                        }

                        // If there was an error caused by interpretation of the packet data,
                        // we must send a random acknowledgement back.
                        if let Err((peer, error)) = &res {
                            #[cfg(all(feature = "prometheus", not(test)))]
                            if let hopr_crypto_packet::errors::PacketError::TicketValidation(_) = error {
                                METRIC_REJECTED_TICKETS_COUNT.increment();
                            }

                            error!(%peer, %error, "failed to process the received message");

                            let peer: OffchainPublicKey = match peer.try_into() {
                                Ok(p) => p,
                                Err(error) => {
                                    warn!(%peer, %error, "Dropping packet - cannot convert peer id");
                                    return None;
                                }
                            };

                            // Send random signed acknowledgement to give feedback to the sender
                            let ack = Acknowledgement::random(&me);

                            #[cfg(feature = "capture")]
                            let captured_packet: capture::CapturedPacket = capture::PacketBeforeTransit::OutgoingAck {
                                me: me_pub,
                                next_hop: peer,
                                ack,
                                is_random: true,
                            }.into();

                            match db
                                .to_send_no_ack(ack.as_ref().to_vec().into_boxed_slice(), peer)
                                .await {
                                    Ok(ack_packet) => {
                                        let now = std::time::Instant::now();
                                        if msg_to_send_tx.send((
                                                ack_packet.next_hop.into(),
                                                ack_packet.data,
                                            )).await.is_err() {
                                            error!("failed to forward an acknowledgement for a failed packet recv to the transport layer");
                                        }
                                        let elapsed = now.elapsed();
                                        if elapsed.as_millis() > SLOW_OP_MS {
                                            warn!(?elapsed," msg_to_send_tx.send took too long");
                                        }

                                        #[cfg(feature = "capture")]
                                        let _ = capture_clone.try_send(captured_packet);
                                    },
                                    Err(error) => tracing::error!(%error, "Failed to create random ack packet for a failed receive"),
                                }
                        }

                        #[cfg(feature = "capture")]
                        if let Ok(packet) = &res {
                            let _ = capture_clone.try_send(capture::PacketBeforeTransit::IncomingPacket {
                                    me: me_pub,
                                    packet,
                                    ticket: ticket_data_clone.into(),
                                }.into()
                            );
                        }

                        res.ok()
                    }
                })
                .filter_map(move |maybe_packet| {
                    let tbf = tbf.clone();

                    async move {
                    if let Some(packet) = maybe_packet {
                        match packet {
                            IncomingPacket::Acknowledgement { packet_tag, previous_hop, .. } |
                            IncomingPacket::Final { packet_tag, previous_hop,.. } |
                            IncomingPacket::Forwarded { packet_tag, previous_hop, .. } => {
                                if tbf.is_tag_replay(&packet_tag).await {
                                    warn!(%previous_hop, "replayed packet received");

                                    #[cfg(all(feature = "prometheus", not(test)))]
                                    METRIC_REPLAYED_PACKET_COUNT.increment();

                                    None
                                } else {
                                    Some(packet)
                                }
                            }
                        }
                    } else {
                        trace!("received empty packet");
                        None
                    }
                }
                })
                .then_concurrent(move |packet| {
                    let mut msg_to_send_tx = wire_msg.0.clone();
                    let db = db.clone();
                    let me = me_for_recv.clone();

                    #[cfg(feature = "capture")]
                    let mut capture_clone = capture_clone.clone();

                    async move {

                    match packet {
                        IncomingPacket::Acknowledgement {
                            previous_hop,
                            ack,
                            ..
                        } => {
                            trace!(%previous_hop, "received a valid acknowledgement");
                            let now = std::time::Instant::now();
                            match db.handle_acknowledgement(ack).await {
                                Ok(_) => trace!(%previous_hop, "successfully processed a known acknowledgement"),
                                // Eventually, we do not care here if the acknowledgement does not belong to any
                                // unacknowledged packets.
                                Err(error) => trace!(%previous_hop, %error, "valid acknowledgement is unknown or error occurred while processing it"),
                            }
                            let elapsed = now.elapsed();
                            if elapsed.as_millis() > SLOW_OP_MS {
                                warn!(%previous_hop, ?elapsed, "ack_processor.handle_acknowledgement took too long");
                            }

                            // We do not acknowledge back acknowledgements.
                            None
                        },
                        IncomingPacket::Final {
                            previous_hop,
                            sender,
                            plain_text,
                            ack_key,
                            ..
                        } => {
                            // Send acknowledgement back
                            trace!(%previous_hop, "acknowledging final packet back");
                            let ack = Acknowledgement::new(ack_key, &me);

                            #[cfg(feature = "capture")]
                            let captured_packet: capture::CapturedPacket = capture::PacketBeforeTransit::OutgoingAck {
                                me: me_pub,
                                next_hop: previous_hop,
                                ack,
                                is_random: false,
                            }.into();

                            #[cfg(all(feature = "prometheus", not(test)))]
                            {
                                METRIC_PACKET_COUNT_PER_PEER.increment(&[&previous_hop.to_string(), "in"]);
                                METRIC_PACKET_COUNT.increment(&["received"]);
                            }

                            if let Ok(ack_packet) = db
                                .to_send_no_ack(ack.as_ref().to_vec().into_boxed_slice(), previous_hop)
                                .await
                                .inspect_err(|error| error!(%error, "failed to create ack packet for a received message"))
                                {
                                    if msg_to_send_tx.send((ack_packet.next_hop.into(), ack_packet.data)).await.is_err() {
                                        error!("failed to send an acknowledgement for a received packet to the transport layer");
                                    }

                                    #[cfg(feature = "capture")]
                                    let _ = capture_clone.try_send(captured_packet);
                                }

                                Some((sender, plain_text))
                        }
                        IncomingPacket::Forwarded {
                            previous_hop,
                            next_hop,
                            data,
                            ack,
                            ..
                        } => {
                            // First, relay the packet to the next hop
                            trace!(%previous_hop, %next_hop, "forwarding packet to the next hop");
                            #[cfg(feature = "capture")]
                            let captured_packet: capture::CapturedPacket = capture::PacketBeforeTransit::OutgoingPacket {
                                me: me_pub,
                                next_hop,
                                num_surbs: 0,
                                is_forwarded: true,
                                data: data.as_ref().into(),
                                ack_challenge: Default::default(),
                                ticket: inspect_ticket_data_in_packet(data.as_ref()).into()
                            }.into();

                            msg_to_send_tx
                                .send((next_hop.into(), data))
                                .await
                                .unwrap_or_else(|_| {
                                    error!("failed to forward a packet to the transport layer");
                                });

                            #[cfg(all(feature = "prometheus", not(test)))]
                            {
                                METRIC_PACKET_COUNT.increment(&["forwarded"]);
                            }

                            #[cfg(feature = "capture")]
                            let _ = capture_clone.try_send(captured_packet);

                            // And then send acknowledgement to the previous hop
                            trace!(%previous_hop, %next_hop, "acknowledging forwarded packet back");
                            #[cfg(feature = "capture")]
                            let captured_packet: capture::CapturedPacket = capture::PacketBeforeTransit::OutgoingAck {
                                me: me_pub,
                                next_hop: previous_hop,
                                ack,
                                is_random: false,
                            }.into();

                            if let Ok(ack_packet) = db
                                .to_send_no_ack(ack.as_ref().to_vec().into_boxed_slice(), previous_hop)
                                .await
                                .inspect_err(|error| error!(%error, "failed to create ack packet for a relayed message"))
                            {
                                msg_to_send_tx
                                    .send((ack_packet.next_hop.into(), ack_packet.data))
                                    .await
                                    .unwrap_or_else(|_| {
                                        error!("failed to send an acknowledgement for a relayed packet to the transport layer");
                                    });

                                #[cfg(feature = "capture")]
                                let _ = capture_clone.try_send(captured_packet);
                            }
                            None
                        }
                    }
                }})
                .filter_map(|maybe_data| async move {
                    if let Some((sender, data)) = maybe_data {
                        ApplicationData::from_bytes(data.as_ref())
                            .inspect_err(|error| tracing::error!(%error, "failed to decode application data"))
                            .ok()
                            .map(|data| (sender, data))
                    } else {
                        None
                    }
                })
                .map(Ok)
                .forward(api.0)
                .instrument(tracing::trace_span!("msg protocol processing - incoming"))
                .await;
        }),
    );

    processes
}
