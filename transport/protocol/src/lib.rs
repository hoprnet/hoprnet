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

use std::{collections::HashMap, time::Duration};

use futures::{FutureExt, SinkExt, StreamExt};
use hopr_api::{
    chain::{ChainKeyOperations, ChainReadChannelOperations, ChainValues},
    db::{HoprDbProtocolOperations, IncomingPacket},
};
use hopr_async_runtime::spawn_as_abortable;
use hopr_crypto_types::types::{HalfKey, OffchainPublicKey};
use hopr_internal_types::{
    prelude::{Acknowledgement, HoprPseudonym},
    protocol::VerifiedAcknowledgement,
};
use hopr_network_types::prelude::ResolvedTransportRouting;
use hopr_protocol_app::prelude::{ApplicationData, ApplicationDataIn, ApplicationDataOut, IncomingPacketInfo};
use hopr_transport_bloom::TagBloomFilter;
use hopr_transport_identity::{Multiaddr, PeerId};
use rust_stream_ext_concurrent::then_concurrent::StreamThenConcurrentExt;
use tracing::Instrument;

use crate::processor::{PacketUnwrapping, PacketWrapping};
pub use crate::timer::execute_on_tick;

const HOPR_PACKET_SIZE: usize = hopr_crypto_packet::prelude::HoprPacket::SIZE;
const SLOW_OP_MS: u128 = 150;

pub type HoprBinaryCodec = crate::codec::FixedLengthCodec<HOPR_PACKET_SIZE>;
pub const CURRENT_HOPR_MSG_PROTOCOL: &str = "/hopr/mix/1.0.0";

pub const TICKET_ACK_BUFFER_SIZE: usize = 1_000_000;
pub const NUM_CONCURRENT_TICKET_ACK_PROCESSING: usize = 10;

pub const ACK_OUT_BUFFER_SIZE: usize = 1_000_000;
pub const NUM_CONCURRENT_ACK_OUT_PROCESSING: usize = 10;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    // packet
    static ref METRIC_PACKET_COUNT:  hopr_metrics::MultiCounter =  hopr_metrics::MultiCounter::new(
        "hopr_packets_count",
        "Number of processed packets of different types (sent, received, forwarded)",
        &["type"]
    ).unwrap();
    static ref METRIC_REPLAYED_PACKET_COUNT: hopr_metrics::SimpleCounter = hopr_metrics::SimpleCounter::new(
        "hopr_replayed_packet_count",
        "The total count of replayed packets during the packet processing pipeline run",
    ).unwrap();
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, strum::Display)]
pub enum ProtocolProcesses {
    #[strum(to_string = "HOPR [msg] - ingress")]
    MsgIn,
    #[strum(to_string = "HOPR [msg] - egress")]
    MsgOut,
    #[strum(to_string = "HOPR [ack] - egress")]
    AckOut,
    #[strum(to_string = "HOPR [ack] - ingress - ticket acknowledgement")]
    TicketAck,
    #[strum(to_string = "HOPR [msg] - mixer")]
    Mixer,
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
/// overlay on top of the `wire_msg` Stream or Sink.
#[allow(clippy::too_many_arguments)]
pub async fn run_msg_ack_protocol<Db, R>(
    packet_cfg: processor::PacketInteractionConfig,
    db: Db,
    resolver: R,
    wire_msg: (
        impl futures::Sink<(PeerId, Box<[u8]>)> + Clone + Unpin + Send + Sync + 'static,
        impl futures::Stream<Item = (PeerId, Box<[u8]>)> + Send + Sync + 'static,
    ),
    api: (
        impl futures::Sink<(HoprPseudonym, ApplicationDataIn)> + Send + Sync + 'static,
        impl futures::Stream<Item = (ApplicationDataOut, ResolvedTransportRouting)> + Send + Sync + 'static,
    ),
) -> HashMap<ProtocolProcesses, hopr_async_runtime::AbortHandle>
where
    Db: HoprDbProtocolOperations + Clone + Send + Sync + 'static,
    R: ChainReadChannelOperations + ChainKeyOperations + ChainValues + Clone + Send + Sync + 'static,
{
    let me = packet_cfg.packet_keypair.clone();

    #[cfg(feature = "capture")]
    let me_pub = *hopr_crypto_types::keypairs::Keypair::public(&me);

    let mut processes = HashMap::new();

    #[cfg(all(feature = "prometheus", not(test)))]
    {
        // Initialize the lazy statics here
        lazy_static::initialize(&METRIC_PACKET_COUNT);
        lazy_static::initialize(&METRIC_REPLAYED_PACKET_COUNT);
    }

    #[cfg(feature = "capture")]
    let capture = {
        use std::str::FromStr;
        let writer: Box<dyn capture::PacketWriter + Send + 'static> = if let Ok(desc) =
            std::env::var("HOPR_CAPTURE_PACKETS")
        {
            if let Ok(udp_writer) = std::net::SocketAddr::from_str(&desc)
                .map_err(std::io::Error::other)
                .and_then(capture::UdpPacketDump::new)
            {
                tracing::warn!("udp packet capture initialized to {desc}");
                Box::new(udp_writer)
            } else if let Ok(pcap_writer) = std::fs::File::create(&desc).and_then(capture::PcapPacketWriter::new) {
                match std::env::var("HOPR_CAPTURE_PATH_TRIGGER") {
                    Ok(ref v) => tracing::info!(%v, "To start capturing packets, create a by 'touch {v}'"),
                    Err(ref e) => {
                        tracing::warn!(%e, "The env var 'HOPR_CAPTURE_PATH_TRIGGER' is not set, packet capture won't start")
                    }
                }
                tracing::info!("pcap file packet capture initialized to {desc}");
                Box::new(pcap_writer)
            } else {
                tracing::error!(desc, "failed to create packet capture: invalid socket address or file");
                Box::new(capture::NullWriter)
            }
        } else {
            tracing::warn!("no packet capture specified");
            Box::new(capture::NullWriter)
        };
        let (capture, ah) = capture::packet_capture_channel(writer);
        processes.insert(ProtocolProcesses::Capture, ah);
        capture
    };

    let tbf = std::sync::Arc::new(parking_lot::Mutex::new(TagBloomFilter::default()));

    let (ticket_ack_tx, ticket_ack_rx) =
        futures::channel::mpsc::channel::<(Acknowledgement, OffchainPublicKey)>(TICKET_ACK_BUFFER_SIZE);

    let db_clone = db.clone();
    let resolver_clone = resolver.clone();
    processes.insert(
        ProtocolProcesses::TicketAck,
        spawn_as_abortable!(ticket_ack_rx
            .for_each_concurrent(NUM_CONCURRENT_TICKET_ACK_PROCESSING, move |(ack, sender)| {
                let db = db_clone.clone();
                let resolver = resolver_clone.clone();
                async move {
                    if let Ok(verified) = hopr_parallelize::cpu::spawn_blocking(move || ack.verify(&sender)).await {
                        tracing::trace!(%sender, "received a valid acknowledgement");
                            match db.handle_acknowledgement(verified, &resolver).await {
                                Ok(_) => tracing::trace!(%sender, "successfully processed a known acknowledgement"),
                                // Eventually, we do not care here if the acknowledgement does not belong to any
                                // unacknowledged packets.
                                Err(error) => tracing::trace!(%sender, %error, "valid acknowledgement is unknown or error occurred while processing it"),
                            }
                    } else {
                        tracing::error!(%sender, "failed to verify signature on acknowledgement");
                    }
                }
            })
            .inspect(|_| tracing::warn!(task = "transport (protocol - ticket acknowledgement)", "long-running background task finished")))
    );

    let (ack_out_tx, ack_out_rx) =
        futures::channel::mpsc::channel::<(Option<HalfKey>, OffchainPublicKey)>(ACK_OUT_BUFFER_SIZE);

    #[cfg(feature = "capture")]
    let capture_clone = capture.clone();

    let db_clone = db.clone();
    let resolver_clone = resolver.clone();
    let me_clone = me.clone();
    let msg_to_send_tx = wire_msg.0.clone();
    processes.insert(
        ProtocolProcesses::AckOut,
        spawn_as_abortable!(
            ack_out_rx
                .for_each_concurrent(
                    NUM_CONCURRENT_ACK_OUT_PROCESSING,
                    move |(maybe_ack_key, destination)| {
                        let db = db_clone.clone();
                        let resolver = resolver_clone.clone();
                        let me = me_clone.clone();
                        let mut msg_to_send_tx_clone = msg_to_send_tx.clone();

                        #[cfg(feature = "capture")]
                        let mut capture = capture_clone.clone();
                        async move {
                            #[cfg(feature = "capture")]
                            let (is_random, me_pub) = (
                                maybe_ack_key.is_none(),
                                *hopr_crypto_types::keypairs::Keypair::public(&me),
                            );

                            // Sign acknowledgement with the given half-key or generate a signed random one
                            let ack = hopr_parallelize::cpu::spawn_blocking(move || {
                                maybe_ack_key
                                    .map(|ack_key| VerifiedAcknowledgement::new(ack_key, &me))
                                    .unwrap_or_else(|| VerifiedAcknowledgement::random(&me))
                            })
                            .await;

                            #[cfg(feature = "capture")]
                            let captured_packet: capture::CapturedPacket = capture::PacketBeforeTransit::OutgoingAck {
                                me: me_pub,
                                ack,
                                is_random,
                                next_hop: destination,
                            }
                            .into();

                            match db
                                .to_send_no_ack(ack.leak().as_ref().into(), destination, &resolver)
                                .await
                            {
                                Ok(ack_packet) => {
                                    let now = std::time::Instant::now();
                                    if msg_to_send_tx_clone
                                        .send((ack_packet.next_hop.into(), ack_packet.data))
                                        .await
                                        .is_err()
                                    {
                                        tracing::error!("failed to forward an acknowledgement to the transport layer");
                                    }
                                    let elapsed = now.elapsed();
                                    if elapsed.as_millis() > SLOW_OP_MS {
                                        tracing::warn!(?elapsed, " msg_to_send_tx.send on ack took too long");
                                    }

                                    #[cfg(feature = "capture")]
                                    let _ = capture.try_send(captured_packet);
                                }
                                Err(error) => tracing::error!(%error, "failed to create ack packet"),
                            }
                        }
                    }
                )
                .inspect(|_| tracing::warn!(
                    task = "transport (protocol - ack outgoing)",
                    "long-running background task finished"
                ))
        ),
    );

    let msg_processor_read = processor::PacketProcessor::new(db.clone(), resolver.clone(), packet_cfg);
    let msg_processor_write = msg_processor_read.clone();

    #[cfg(feature = "capture")]
    let capture_clone = capture.clone();

    let msg_to_send_tx = wire_msg.0.clone();
    processes.insert(
        ProtocolProcesses::MsgOut,
        spawn_as_abortable!(async move {
            let _neverending = api
                .1
                .then_concurrent(|(data, routing)| {
                    let msg_processor = msg_processor_write.clone();

                    #[cfg(feature = "capture")]
                    let (mut capture_clone, data_clone, num_surbs) =
                        (capture_clone.clone(), data.clone(), routing.count_return_paths() as u8);

                    async move {
                        match PacketWrapping::send(&msg_processor, data, routing).await {
                            Ok(v) => {
                                #[cfg(all(feature = "prometheus", not(test)))]
                                {
                                    METRIC_PACKET_COUNT.increment(&["sent"]);
                                }

                                #[cfg(feature = "capture")]
                                let _ = capture_clone.try_send(
                                    capture::PacketBeforeTransit::OutgoingPacket {
                                        me: me_pub,
                                        next_hop: v.next_hop,
                                        num_surbs,
                                        is_forwarded: false,
                                        data: data_clone.data.to_bytes().into_vec().into(),
                                        ack_challenge: v.ack_challenge.as_ref().into(),
                                        signals: data_clone.packet_info.unwrap_or_default().signals_to_destination,
                                        ticket: inspect_ticket_data_in_packet(&v.data).into(),
                                    }
                                    .into(),
                                );

                                Some((v.next_hop.into(), v.data))
                            }
                            Err(error) => {
                                tracing::error!(%error, "packet could not be wrapped for sending");
                                None
                            }
                        }
                    }
                })
                .filter_map(futures::future::ready)
                .inspect(|(peer, _)| tracing::trace!(%peer, "protocol message out"))
                .map(Ok)
                .forward(msg_to_send_tx)
                .instrument(tracing::trace_span!("msg protocol processing - egress"))
                .inspect(|_| {
                    tracing::warn!(
                        task = "transport (protocol - msg egress)",
                        "long-running background task finished"
                    )
                })
                .await;
        }),
    );

    let ack_out_tx_clone_1 = ack_out_tx.clone();
    let ack_out_tx_clone_2 = ack_out_tx.clone();

    #[cfg(feature = "capture")]
    let capture_clone = capture.clone();

    // Create a cache for a CPU-intensive conversion PeerId -> OffchainPublicKey
    let peer_id_cache: moka::future::Cache<PeerId, OffchainPublicKey> = moka::future::Cache::builder()
        .time_to_idle(Duration::from_secs(600))
        .max_capacity(100_000)
        .build();

    processes.insert(
        ProtocolProcesses::MsgIn,
        spawn_as_abortable!(async move {
            let _neverending = wire_msg
                .1
                .then_concurrent(move |(peer, data)| {
                    let msg_processor = msg_processor_read.clone();
                    let mut ack_out_tx = ack_out_tx_clone_1.clone();
                    let peer_id_key_cache = peer_id_cache.clone();

                    tracing::trace!(%peer, "protocol message in");

                    #[cfg(feature = "capture")]
                    let (mut capture_clone, ticket_data_clone) = (
                        capture.clone(),
                        inspect_ticket_data_in_packet(&data).to_vec()
                    );

                    async move {
                        // Try to retrieve the peer's public key from the cache or compute it if it does not exist yet
                        let peer_key = match peer_id_key_cache
                                .try_get_with_by_ref(&peer, hopr_parallelize::cpu::spawn_fifo_blocking(move || OffchainPublicKey::from_peerid(&peer)))
                                .await {
                            Ok(peer) => peer,
                            Err(error) => {
                                // There absolutely nothing we can do when the peer id is unparseable (e.g., non-ed25519 based)
                                tracing::error!(%peer, %error, "dropping packet - cannot convert peer id");
                                return None;
                            }
                        };

                        let now = std::time::Instant::now();
                        let res = msg_processor.recv(peer_key, data).await;
                        let elapsed = now.elapsed();
                        if elapsed.as_millis() > SLOW_OP_MS {
                            tracing::warn!(%peer, ?elapsed, "msg_processor.recv took too long");
                        }

                        // If there was an error caused by interpretation of the packet data,
                        // we must send a random acknowledgement back.
                        if let Err(error) = &res {
                            tracing::error!(%peer, %error, "failed to process the received packet");

                            // Send random signed acknowledgement to give feedback to the sender
                            if error.is_undecodable() {
                                // Do not send an ack back if the packet could not be decoded at all
                                // 
                                // Potentially adversarial behavior
                                tracing::trace!(%peer, "not sending ack back on undecodable packet - possible adversarial behavior");
                            } else {
                                let now = std::time::Instant::now();

                                if let Err(error) = ack_out_tx.send((None, peer_key)).await {
                                    tracing::error!(%error, "failed to send ack to the egress queue");
                                }
                                let elapsed = now.elapsed();
                                if elapsed.as_millis() > SLOW_OP_MS {
                                    tracing::warn!(%peer, ?elapsed, "ack_out.send on failed packet took too long");
                                }
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

                    futures::future::ready(
                        if let Some(packet) = maybe_packet {
                            match packet {
                                IncomingPacket::Acknowledgement { packet_tag, previous_hop, .. } |
                                IncomingPacket::Final { packet_tag, previous_hop,.. } |
                                IncomingPacket::Forwarded { packet_tag, previous_hop, .. } => {
                                    // This operation has run-time of ~10 nanoseconds,
                                    // and therefore does not need to be invoked via spawn_blocking
                                    if tbf.lock().check_and_set(&packet_tag) {
                                        tracing::warn!(%previous_hop, "replayed packet received");

                                        #[cfg(all(feature = "prometheus", not(test)))]
                                        METRIC_REPLAYED_PACKET_COUNT.increment();

                                        None
                                    } else {
                                        Some(packet)
                                    }
                                }
                            }
                        } else {
                            tracing::trace!("received empty packet");
                            None
                        }
                    )
                })
                .then_concurrent(move |packet| {
                    let mut msg_to_send_tx = wire_msg.0.clone();

                    #[cfg(feature = "capture")]
                    let mut capture_clone = capture_clone.clone();

                    let mut ticket_ack_tx_clone = ticket_ack_tx.clone();
                    let mut ack_out_tx = ack_out_tx_clone_2.clone();
                    async move {

                    match packet {
                        IncomingPacket::Acknowledgement {
                            previous_hop,
                            ack,
                            ..
                        } => {
                            tracing::trace!(%previous_hop, "acknowledging ticket using received ack");
                            let now = std::time::Instant::now();
                            if let Err(error) = ticket_ack_tx_clone.send((ack, previous_hop)).await {
                                tracing::error!(%error, "failed dispatching received acknowledgement to the ticket ack queue");
                            }
                            let elapsed = now.elapsed();
                            if elapsed.as_millis() > SLOW_OP_MS {
                                tracing::warn!(?elapsed," ack_tx.send took too long");
                            }
                            let elapsed = now.elapsed();
                            if elapsed.as_millis() > SLOW_OP_MS {
                                tracing::warn!(%previous_hop, ?elapsed, "ack_processor.handle_acknowledgement took too long");
                            }

                            // We do not acknowledge back acknowledgements.
                            None
                        },
                        IncomingPacket::Final {
                            previous_hop,
                            sender,
                            plain_text,
                            ack_key,
                            info,
                            ..
                        } => {
                            // Send acknowledgement back
                            tracing::trace!(%previous_hop, "acknowledging final packet back");
                            let now = std::time::Instant::now();
                            if let Err(error) = ack_out_tx.send((Some(ack_key), previous_hop)).await {
                                tracing::error!(%error, "failed to send ack to the egress queue");
                            }
                            let elapsed = now.elapsed();
                            if elapsed.as_millis() > SLOW_OP_MS {
                                tracing::warn!(%previous_hop, ?elapsed, "ack_out.send on final packet took too long");
                            }

                            #[cfg(all(feature = "prometheus", not(test)))]
                            {
                                METRIC_PACKET_COUNT.increment(&["received"]);
                            }

                            Some((sender, plain_text, info))
                        }
                        IncomingPacket::Forwarded {
                            previous_hop,
                            next_hop,
                            data,
                            ack_key,
                            ..
                        } => {
                            // First, relay the packet to the next hop
                            tracing::trace!(%previous_hop, %next_hop, "forwarding packet to the next hop");

                            #[cfg(feature = "capture")]
                            let captured_packet: capture::CapturedPacket = capture::PacketBeforeTransit::OutgoingPacket {
                                me: me_pub,
                                next_hop,
                                num_surbs: 0,
                                is_forwarded: true,
                                data: data.as_ref().into(),
                                ack_challenge: Default::default(),
                                signals: None.into(),
                                ticket: inspect_ticket_data_in_packet(data.as_ref()).into()
                            }.into();

                            msg_to_send_tx
                                .send((next_hop.into(), data))
                                .await
                                .unwrap_or_else(|_| {
                                    tracing::error!("failed to forward a packet to the transport layer");
                                });

                            #[cfg(all(feature = "prometheus", not(test)))]
                            {
                                METRIC_PACKET_COUNT.increment(&["forwarded"]);
                            }

                            #[cfg(feature = "capture")]
                            let _ = capture_clone.try_send(captured_packet);

                             // Send acknowledgement back
                            tracing::trace!(%previous_hop, "acknowledging forwarded packet back");
                            let now = std::time::Instant::now();
                            if let Err(error) = ack_out_tx.send((Some(ack_key), previous_hop)).await {
                                tracing::error!(%error, "failed to send ack to the egress queue");
                            }
                            let elapsed = now.elapsed();
                            if elapsed.as_millis() > SLOW_OP_MS {
                                tracing::warn!(%previous_hop, ?elapsed, "ack_out.send on forwarded packet took too long");
                            }

                            None
                        }
                    }
                }})
                .filter_map(|maybe_data| futures::future::ready(
                    // Create the ApplicationDataIn data structure for incoming data
                    maybe_data
                        .and_then(|(sender, data, aux_info)| ApplicationData::try_from(data.as_ref())
                            .inspect_err(|error| tracing::error!(%sender, %error, "failed to decode application data"))
                            .ok()
                            .map(|data| (sender, ApplicationDataIn {
                                data,
                                packet_info: IncomingPacketInfo {
                                    signals_from_sender: aux_info.packet_signals,
                                    num_saved_surbs: aux_info.num_surbs,
                                }
                            })))
                ))
                .map(Ok)
                .forward(api.0)
                .instrument(tracing::trace_span!("msg protocol processing - ingress"))
                .inspect(|_| tracing::warn!(task = "transport (protocol - msg ingress)", "long-running background task finished"))
                .await;
        }),
    );

    processes
}
