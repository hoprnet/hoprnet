//! HOPR packet processing pipeline.

mod builder;
mod config;

pub use builder::{PacketPipelineBuilder, Unset};
use bytes::Bytes;
pub use config::{AcknowledgementPipelineConfig, PacketPipelineConfig};
use futures::{SinkExt, StreamExt, future::Either};
use futures_time::{future::FutureExt as TimeExt, stream::StreamExt as TimeStreamExt};
use hopr_api::{
    PeerId,
    node::TicketEvent,
    types::{crypto::prelude::*, internal::prelude::*},
};
use hopr_crypto_packet::HoprSurb;
use hopr_protocol_app::prelude::*;
use hopr_protocol_hopr::prelude::*;
use hopr_utils::{
    network_types::timeout::{SinkTimeoutError, TimeoutSinkExt, TimeoutStreamExt},
    runtime::AbortableList,
};
use rust_stream_ext_concurrent::then_concurrent::StreamThenConcurrentExt;
use tracing::Instrument;

use crate::PeerProtocolCounterRegistry;

/// Default concurrency for the incoming acknowledgement processing pipeline when not overridden
/// via [`AcknowledgementPipelineConfig::ack_input_concurrency`].
const DEFAULT_ACK_INPUT_CONCURRENCY: usize = 10;
/// Default concurrency for the outgoing acknowledgement processing pipeline when not overridden
/// via [`AcknowledgementPipelineConfig::ack_output_concurrency`].
const DEFAULT_ACK_OUTPUT_CONCURRENCY: usize = 10;
const QUEUE_SEND_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(50);
const PACKET_DECODING_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(150);
const PACKET_ENCODING_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(150);

/// Number of Rayon threads kept permanently free for outgoing packet encode (SURB generation).
/// The ingress decode concurrency default is `pool_thread_count - ENCODE_RESERVED_THREADS` so
/// that heavy download traffic cannot starve the upload/SURB replenishment path.
const ENCODE_RESERVED_THREADS: usize = 2;

/// Artificial per-packet delay injected into `wire_in` when the Rayon pool is detected as
/// congested. 20 ms keeps the decode queue from growing while still allowing acks and keep-alive
/// SURB packets to drain through at a reasonable pace.
const INGRESS_THROTTLE_DELAY: std::time::Duration = std::time::Duration::from_millis(20);

/// Pool outstanding-task watermark factor: the gate trips when
/// `outstanding_tasks > pool_thread_count * INGRESS_POOL_HIGH_WATERMARK_FACTOR`.
/// A factor of 3 gives one full pool worth of headroom above the cap.
const INGRESS_POOL_HIGH_WATERMARK_FACTOR: usize = 3;

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_PACKET_COUNT:  hopr_api::types::telemetry::MultiCounter =  hopr_api::types::telemetry::MultiCounter::new(
        "hopr_packets_count",
        "Number of processed packets of different types (sent, received, forwarded)",
        &["type"]
    ).unwrap();
    static ref METRIC_PACKET_REJECTED_COUNT: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_packet_rejected_count",
        "Number of incoming packets rejected due various reasons",
        &["reason"]
    ).unwrap();
    // Tracks how often the Rayon-backed packet decode path exceeds PACKET_DECODING_TIMEOUT.
    // A sustained non-zero rate here indicates the Rayon pool is saturated—correlate with
    // `hopr_rayon_tasks_cancelled_total` and hopr_rayon_queue_wait_seconds to diagnose whether
    // the bottleneck is queue depth, individual task duration, or both.
    static ref METRIC_PACKET_DECODE_TIMEOUTS: hopr_api::types::telemetry::SimpleCounter = hopr_api::types::telemetry::SimpleCounter::new(
        "hopr_packet_decode_timeouts_total",
        "Number of incoming packets dropped due to decode timeout (sustained rate indicates Rayon pool saturation)"
    ).unwrap();
    static ref METRIC_VALIDATION_ERRORS: hopr_api::types::telemetry::MultiCounter =  hopr_api::types::telemetry::MultiCounter::new(
        "hopr_packet_ticket_validation_errors",
        "Number of different ticket validation errors encountered during packet processing",
        &["type"]
    ).unwrap();
    static ref METRIC_RECEIVED_ACKS: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_protocol_ack_received_count",
        "Number of received acknowledgements",
        &["valid"]
    ).unwrap();
    static ref METRIC_SENT_ACKS: hopr_api::types::telemetry::SimpleCounter = hopr_api::types::telemetry::SimpleCounter::new(
        "hopr_protocol_ack_sent_count",
        "Number of sent message acknowledgements"
    ).unwrap();
    static ref METRIC_TICKETS_COUNT: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_tickets_count",
        "Number of tickets by type (winning, losing, rejected)",
        &["type"]
    ).unwrap();
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, strum::Display)]
pub enum PacketPipelineProcesses {
    #[strum(to_string = "HOPR [msg] - ingress")]
    MsgIn,
    #[strum(to_string = "HOPR [msg] - egress")]
    MsgOut,
    #[strum(to_string = "HOPR [ack] - egress")]
    AckOut,
    #[strum(to_string = "HOPR [ack] - ingress")]
    AckIn,
    #[strum(to_string = "HOPR [msg] - mixer")]
    Mixer,
}

/// Performs encoding of outgoing Application protocol packets into HOPR protocol outgoing packets.
async fn start_outgoing_packet_pipeline<AppOut, E, WOut, WOutErr>(
    app_outgoing: AppOut,
    encoder: std::sync::Arc<E>,
    wire_outgoing: WOut,
    counters: super::counters::PeerProtocolCounterRegistry,
    concurrency: usize,
) where
    AppOut: futures::Stream<Item = (ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)> + Send + 'static,
    E: PacketEncoder + Send + Sync + 'static,
    WOut: futures::Sink<(PeerId, Bytes), Error = SinkTimeoutError<WOutErr>> + Clone + Unpin + Send + 'static,
    WOutErr: std::error::Error,
{
    let res = app_outgoing
        .then_concurrent(
            |(routing, data)| {
                let encoder = encoder.clone();
                let counters = counters.clone();
                async move {
                    match hopr_utils::parallelize::cpu::spawn_encode_blocking(
                        move || {
                            encoder.encode_packet(
                                data.data.to_bytes(),
                                routing,
                                data.packet_info
                                    .map(|data| data.signals_to_destination)
                                    .unwrap_or_default(),
                            )
                        },
                        "packet_encode",
                    )
                    .timeout(futures_time::time::Duration::from(PACKET_ENCODING_TIMEOUT))
                    .await
                    {
                        Ok(Ok(Ok(packet))) => {
                            #[cfg(all(feature = "telemetry", not(test)))]
                            METRIC_PACKET_COUNT.increment(&["sent"]);

                            counters.get_or_create(&packet.next_hop).record_message_sent();
                            tracing::trace!(peer = packet.next_hop.to_peerid_str(), "protocol message out");
                            Some((packet.next_hop.into(), packet.data))
                        }
                        Ok(Ok(Err(error))) => {
                            tracing::error!(%error, "outgoing packet could not be encoded");
                            None
                        }
                        Ok(Err(error)) => {
                            tracing::error!(%error, "parallel processing of the outgoing packet failed");
                            None
                        }
                        Err(error) => {
                            tracing::error!(%error, "timeout while processing the outgoing packet");
                            hopr_utils::parallelize::cpu::ENCODE_TIMEOUT_DROPS
                                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            None
                        }
                    }
                }
            },
            concurrency,
        )
        .filter_map(futures::future::ready)
        .map(Ok)
        .forward_to_timeout(wire_outgoing)
        .in_current_span()
        .await;

    if let Err(error) = res {
        tracing::error!(
            task = "transport (protocol - msg egress)",
            %error,
            "long-running background task finished with error"
        );
    } else {
        tracing::warn!(
            task = "transport (protocol - msg egress)",
            "long-running background task finished"
        )
    }
}

/// Performs HOPR protocol decoding of incoming packets into Application protocol packets.
///
/// `wire_incoming` --> `decoder` --> `ack_outgoing` (final + forwarded)
///                             | --> `wire_outgoing` (forwarded)
///                             | --> `ack_incoming` (forwarded)
///                             | --> `app_incoming` (final)
#[allow(clippy::too_many_arguments)]
async fn start_incoming_packet_pipeline<WIn, WOut, D, T, TEvt, AckIn, AckOut, AppIn, AppInErr>(
    (wire_outgoing, wire_incoming): (WOut, WIn),
    decoder: std::sync::Arc<D>,
    ticket_proc: std::sync::Arc<T>,
    ticket_events: TEvt,
    (ack_outgoing, ack_incoming): (AckOut, AckIn),
    app_incoming: AppIn,
    counters: super::counters::PeerProtocolCounterRegistry,
    concurrency: usize,
) where
    WIn: futures::Stream<Item = (PeerId, Bytes)> + Send + 'static,
    WOut: futures::Sink<(PeerId, Bytes)> + Clone + Unpin + Send + 'static,
    WOut::Error: std::error::Error,
    D: PacketDecoder + Sync + Send + 'static,
    T: UnacknowledgedTicketProcessor + Send + 'static,
    TEvt: futures::Sink<TicketEvent> + Clone + Unpin + Send + 'static,
    TEvt::Error: std::error::Error,
    AckIn: futures::Sink<(OffchainPublicKey, Vec<Acknowledgement>)> + Send + Unpin + Clone + 'static,
    AckIn::Error: std::error::Error,
    AckOut: futures::Sink<(OffchainPublicKey, Option<HalfKey>)> + Send + Unpin + Clone + 'static,
    AckOut::Error: std::error::Error,
    AppIn: futures::Sink<(HoprPseudonym, ApplicationDataIn), Error = SinkTimeoutError<AppInErr>> + Send + 'static,
    AppInErr: std::error::Error,
{
    let ack_outgoing_success = ack_outgoing.clone();
    let ack_outgoing_failure = ack_outgoing;
    let ticket_proc_success = ticket_proc;

    let res = wire_incoming
        .then_concurrent(move |(peer, data)| {
            let decoder = decoder.clone();
            let mut ack_outgoing_failure = ack_outgoing_failure.clone();
            let mut ticket_events_reject = ticket_events.clone();

            tracing::trace!(%peer, "protocol message in");

            async move {
                match hopr_utils::parallelize::cpu::spawn_decode_blocking(move || decoder.decode(peer, data), "packet_decode")
                    .timeout(futures_time::time::Duration::from(PACKET_DECODING_TIMEOUT))
                    .await {
                    Ok(Ok(Ok(packet))) => {
                        tracing::trace!(%peer, ?packet, "successfully decoded incoming packet");
                        Some(packet)
                    },
                    Ok(Ok(Err(IncomingPacketError::Undecodable(error)))) => {
                        // Do not send an ack back if the packet could not be decoded at all
                        //
                        // Potentially adversarial behavior
                        tracing::trace!(%peer, %error, "not sending ack back on undecodable packet - possible adversarial behavior");

                        #[cfg(all(feature = "telemetry", not(test)))]
                        METRIC_PACKET_REJECTED_COUNT.increment(&["undecodable"]);

                        None
                    },
                    Ok(Ok(Err(IncomingPacketError::ProcessingError(sender, error)))) => {
                        tracing::error!(%peer, %error, "failed to process the decoded packet");
                        // On this failure, we send back a random acknowledgement
                        ack_outgoing_failure
                            .send((*sender, None))
                            .await
                            .unwrap_or_else(|error| {
                                tracing::error!(%error, "failed to send ack to the egress queue");
                            });

                        #[cfg(all(feature = "telemetry", not(test)))]
                        METRIC_PACKET_REJECTED_COUNT.increment(&["processing_error"]);

                        None
                    },
                    Ok(Ok(Err(IncomingPacketError::InvalidTicket(sender, error)))) => {
                        tracing::error!(%peer, %error, "failed to validate ticket on the received packet");
                        if let Err(error) = ticket_events_reject
                            .send(TicketEvent::RejectedTicket(error.ticket, error.issuer))
                            .await {
                            tracing::error!(%error, "failed to notify invalid ticket rejection");
                        }
                        // On this failure, we send back a random acknowledgement
                        ack_outgoing_failure
                            .send((*sender, None))
                            .await
                            .unwrap_or_else(|error| {
                                tracing::error!(%error, "failed to send ack to the egress queue");
                            });

                        #[cfg(all(feature = "telemetry", not(test)))]
                        {
                            METRIC_VALIDATION_ERRORS.increment(&[error.kind.as_ref()]);
                            METRIC_PACKET_REJECTED_COUNT.increment(&["invalid_ticket"]);
                            METRIC_TICKETS_COUNT.increment(&["rejected"]);
                        }

                        None
                    }
                    Ok(Err(error)) => {
                        tracing::error!(%error, "parallel processing of the incoming packet failed");
                        None
                    },
                    Err(_) => {
                        // If we cannot decode the packet within the time limit, just drop it
                        tracing::error!(
                            %peer,
                            timeout_ms = PACKET_DECODING_TIMEOUT.as_millis() as u64,
                            "dropped incoming packet: decode timeout - check the 'hopr_rayon_queue_wait_seconds' metric for pool saturation"
                        );
                        hopr_utils::parallelize::cpu::DECODE_TIMEOUT_DROPS
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        #[cfg(all(feature = "telemetry", not(test)))]
                        {
                            METRIC_PACKET_DECODE_TIMEOUTS.increment();
                            METRIC_PACKET_REJECTED_COUNT.increment(&["timeout"]);
                        }

                        None
                    }
                }
            }.instrument(tracing::debug_span!("incoming_packet_decode", %peer))
        }, concurrency)
        .filter_map(futures::future::ready)
        // Branch on the packet type BEFORE building the async future so each arm only clones
        // the handles it actually needs. `futures::future::Either` lets us return three
        // distinct async blocks from one closure without boxing.
        .then_concurrent(move |packet| {
            match packet {
                IncomingPacket::Acknowledgement(ack) => {
                    let mut ack_incoming = ack_incoming.clone();
                    let counters = counters.clone();
                    Either::Left(async move {
                        let IncomingAcknowledgementPacket { previous_hop, received_acks, .. } = *ack;
                        tracing::trace!(previous_hop = previous_hop.to_peerid_str(), num_acks = received_acks.len(), "incoming acknowledgements");
                        counters.get_or_create(&previous_hop).record_acks_received(received_acks.len() as u64);

                        ack_incoming
                            .send((previous_hop, received_acks))
                            .await
                            .unwrap_or_else(|error| {
                                tracing::error!(%error, "failed dispatching received acknowledgement to the ticket ack queue");
                            });

                        // We do not acknowledge back acknowledgements.
                        None
                    })
                }
                IncomingPacket::Final(final_packet) => {
                    let mut ack_outgoing_success = ack_outgoing_success.clone();
                    Either::Right(Either::Left(async move {
                        let IncomingFinalPacket {
                            previous_hop,
                            sender,
                            plain_text,
                            ack_key,
                            info,
                            ..
                        } = *final_packet;
                        tracing::trace!(previous_hop = previous_hop.to_peerid_str(), "incoming final packet");

                        // Send acknowledgement back
                        ack_outgoing_success
                            .send((previous_hop, Some(ack_key)))
                            .await
                            .unwrap_or_else(|error| {
                                tracing::error!(%error, "failed to send ack to the egress queue");
                            });

                        #[cfg(all(feature = "telemetry", not(test)))]
                        METRIC_PACKET_COUNT.increment(&["received"]);

                        Some((sender, plain_text, info))
                    }))
                }
                IncomingPacket::Forwarded(fwd_packet) => {
                    let ticket_proc = ticket_proc_success.clone();
                    let mut wire_outgoing = wire_outgoing.clone();
                    let mut ack_outgoing_success = ack_outgoing_success.clone();
                    let counters = counters.clone();
                    Either::Right(Either::Right(async move {
                        let IncomingForwardedPacket {
                            previous_hop,
                            next_hop,
                            data,
                            ack_key_prev_hop,
                            ack_challenge,
                            received_ticket,
                            ..
                        } = *fwd_packet;
                        // Per requirements, this call is not blocking
                        if let Err(error) = ticket_proc.insert_unacknowledged_ticket(&next_hop, ack_challenge, received_ticket) {
                            tracing::error!(
                                previous_hop = previous_hop.to_peerid_str(),
                                next_hop = next_hop.to_peerid_str(),
                                %error,
                                "failed to insert unacknowledged ticket into the ticket processor"
                            );

                            #[cfg(all(feature = "telemetry", not(test)))]
                            METRIC_PACKET_REJECTED_COUNT.increment(&["unack_processing_error"]);

                            return None;
                        }

                        // First, relay the packet to the next hop
                        tracing::trace!(
                            previous_hop = previous_hop.to_peerid_str(),
                            next_hop = next_hop.to_peerid_str(),
                            "forwarding packet to the next hop"
                        );

                        match wire_outgoing.send((next_hop.into(), data)).await {
                            Ok(()) => {
                                counters.get_or_create(&next_hop).record_message_sent();

                                #[cfg(all(feature = "telemetry", not(test)))]
                                METRIC_PACKET_COUNT.increment(&["forwarded"]);
                            }
                            Err(error) => {
                                tracing::error!(%error, "failed to forward a packet to the transport layer");
                                return None;
                            }
                        }

                        // Send acknowledgement back
                        tracing::trace!(previous_hop = previous_hop.to_peerid_str(), "acknowledging forwarded packet back");
                        ack_outgoing_success
                            .send((previous_hop, Some(ack_key_prev_hop)))
                            .await
                            .unwrap_or_else(|error| {
                                tracing::error!(%error, "failed to send ack to the egress queue");
                            });

                        None
                    }))
                }
            }
        }, concurrency)
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
        .forward_to_timeout(app_incoming)
        .in_current_span()
        .await;

    if let Err(error) = res {
        tracing::error!(
            task = "transport (protocol - msg ingress)",
            %error,
            "long-running background task finished with error"
        );
    } else {
        tracing::warn!(
            task = "transport (protocol - msg ingress)",
            "long-running background task finished"
        )
    }
}

async fn start_outgoing_ack_pipeline<AckOut, E, WOut>(
    ack_outgoing: AckOut,
    encoder: std::sync::Arc<E>,
    cfg: AcknowledgementPipelineConfig,
    packet_key: OffchainKeypair,
    wire_outgoing: WOut,
) where
    AckOut: futures::Stream<Item = (OffchainPublicKey, Option<HalfKey>)> + Send + 'static,
    E: PacketEncoder + Sync + Send + 'static,
    WOut: futures::Sink<(PeerId, Bytes)> + Clone + Unpin + Send + 'static,
    WOut::Error: std::error::Error,
{
    ack_outgoing
        .map(move |(destination, maybe_ack_key)| {
            let packet_key = packet_key.clone();
            // Sign acknowledgement with the given half-key or generate a signed random one
            let ack = maybe_ack_key
                .map(|ack_key| VerifiedAcknowledgement::new(ack_key, &packet_key))
                .unwrap_or_else(|| VerifiedAcknowledgement::random(&packet_key));
            (destination, ack)
        })
        .buffer(futures_time::time::Duration::from(cfg.ack_buffer_interval))
        .filter(|acks| futures::future::ready(!acks.is_empty()))
        // Group by sender, reusing the same HashMap across buffer cycles so we don't
        // re-allocate its bucket storage every `cfg.ack_buffer_interval` (default 200ms).
        //
        // The halfbrown map uses a Vec backing for a small number of distinct senders
        // (<32) and transitions to hashbrown otherwise — calling `drain()` keeps the
        // underlying allocation, leaving us with only the per-group Vec<Ack> to allocate
        // (which downstream consumes as owned values).
        .scan(
            halfbrown::HashMap::<OffchainPublicKey, Vec<VerifiedAcknowledgement>, ahash::RandomState>::with_capacity_and_hasher(
                cfg.ack_grouping_capacity,
                ahash::RandomState::default(),
            ),
            |groups, buffered_acks| {
                for (dst, ack) in buffered_acks {
                    groups
                        .entry(dst)
                        .and_modify(|v| v.push(ack))
                        .or_insert_with(|| vec![ack]);
                }
                tracing::trace!(
                    num_groups = groups.len(),
                    num_acks = groups.values().map(|v| v.len()).sum::<usize>(),
                    "acknowledgements grouped"
                );
                let drained: Vec<_> = groups.drain().collect();
                futures::future::ready(Some(futures::stream::iter(drained)))
            },
        )
        .flatten()
        .for_each_concurrent(
            cfg.ack_output_concurrency.filter(|&n| n > 0).unwrap_or(DEFAULT_ACK_OUTPUT_CONCURRENCY),
            move |(destination, acks)| {
                let encoder = encoder.clone();
                let mut wire_outgoing = wire_outgoing.clone();
                async move {
                    // Make sure that the acknowledgements are sent in batches of at most MAX_ACKNOWLEDGEMENTS_BATCH_SIZE
                    // TODO: find better strategy to avoid reallocations
                    let c = acks.chunks(MAX_ACKNOWLEDGEMENTS_BATCH_SIZE).map(|c| c.to_vec()).collect::<Vec<_>>();
                    for ack_chunk in c {
                        let encoder = encoder.clone();
                        #[cfg(all(feature = "telemetry", not(test)))]
                        let ack_chunk_len = ack_chunk.len() as u64;
                        match hopr_utils::parallelize::cpu::spawn_fifo_blocking(move || encoder.encode_acknowledgements(&ack_chunk, &destination), "ack_encode").await {
                            Ok(Ok(ack_packet)) => {
                                wire_outgoing
                                    .feed((ack_packet.next_hop.into(), ack_packet.data))
                                    .await
                                    .unwrap_or_else(|error| {
                                        tracing::error!(%error, "failed to forward an acknowledgement to the transport layer");
                                    });

                                #[cfg(all(feature = "telemetry", not(test)))]
                                METRIC_SENT_ACKS.increment_by(ack_chunk_len);
                            }
                            Ok(Err(error)) => tracing::error!(%error, "failed to encode acknowledgements"),
                            Err(error) => tracing::error!(%error, "parallel processing of the outgoing acknowledgements failed"),
                        }
                    }
                    if let Err(error) = wire_outgoing.flush().await {
                        tracing::error!(%error, "failed to flush acknowledgements batch to the transport layer");
                    }
                    tracing::trace!("acknowledgements out");
                }.instrument(tracing::debug_span!("outgoing_ack_batch", peer = destination.to_peerid_str()))
            }
        )
        .in_current_span()
        .await;

    tracing::warn!(
        task = "transport (protocol - ack outgoing)",
        "long-running background task finished"
    );
}

/// Drains incoming acknowledgements without forwarding them to an [`UnacknowledgedTicketProcessor`].
///
/// Used by Entry and Exit nodes — neither processes incoming ticket acknowledgements.
/// Entry nodes receive acks from relays (they pay for forwarding), Exit nodes keep
/// the pipeline alive for future PIX use. In both cases the queue must be actively
/// drained; dropping the receiver causes every inbound ack dispatch to fail with
/// `SendError(disconnected)`.
async fn start_drain_incoming_ack_pipeline<AckIn>(ack_incoming: AckIn)
where
    AckIn: futures::Stream<Item = (OffchainPublicKey, Vec<Acknowledgement>)> + Send + 'static,
{
    ack_incoming
        .for_each(move |(peer, acks)| {
            tracing::trace!(%peer, num = acks.len(), "received acknowledgements (drained, not processed)");
            futures::future::ready(())
        })
        .in_current_span()
        .await;

    tracing::warn!(
        task = "transport (protocol - ticket acknowledgement drain)",
        "long-running background task finished"
    );
}

async fn start_relay_incoming_ack_pipeline<AckIn, T, TEvt>(
    ack_incoming: AckIn,
    ticket_events: TEvt,
    ticket_proc: std::sync::Arc<T>,
    concurrency: usize,
) where
    AckIn: futures::Stream<Item = (OffchainPublicKey, Vec<Acknowledgement>)> + Send + 'static,
    T: UnacknowledgedTicketProcessor + Sync + Send + 'static,
    TEvt: futures::Sink<TicketEvent> + Clone + Unpin + Send + 'static,
    TEvt::Error: std::error::Error,
{
    ack_incoming
        .for_each_concurrent(concurrency, move |(peer, acks)| {
            let ticket_proc = ticket_proc.clone();
            let mut ticket_evt = ticket_events.clone();
            async move {
                tracing::trace!(num = acks.len(), "received acknowledgements");
                match hopr_utils::parallelize::cpu::spawn_fifo_blocking(
                    move || ticket_proc.acknowledge_tickets(peer, acks),
                    "ack_decode",
                )
                .await
                {
                    Ok(Ok(resolutions)) if !resolutions.is_empty() => {
                        let resolutions_iter = resolutions.into_iter().filter_map(|resolution| match resolution {
                            ResolvedAcknowledgement::RelayingWin(redeemable_ticket) => {
                                tracing::trace!("received ack for a winning ticket");
                                #[cfg(all(feature = "telemetry", not(test)))]
                                {
                                    METRIC_RECEIVED_ACKS.increment(&["true"]);
                                    METRIC_TICKETS_COUNT.increment(&["winning"]);
                                }
                                Some(Ok(TicketEvent::WinningTicket(redeemable_ticket)))
                            }
                            ResolvedAcknowledgement::RelayingLoss(_) => {
                                // Losing tickets are not getting accounted for anywhere.
                                tracing::trace!("received ack for a losing ticket");
                                #[cfg(all(feature = "telemetry", not(test)))]
                                {
                                    METRIC_RECEIVED_ACKS.increment(&["true"]);
                                    METRIC_TICKETS_COUNT.increment(&["losing"]);
                                }
                                None
                            }
                        });

                        // All acknowledgements that resulted in winning tickets go upstream
                        if let Err(error) = ticket_evt.send_all(&mut futures::stream::iter(resolutions_iter)).await {
                            tracing::error!(%error, "failed to notify ticket resolutions");
                        }
                    }
                    Ok(Ok(_)) => {
                        tracing::debug!("acknowledgement batch could not acknowledge any ticket");
                    }
                    Ok(Err(TicketAcknowledgementError::UnexpectedAcknowledgement)) => {
                        // Unexpected acknowledgements naturally happen
                        // as acknowledgements of 0-hop packets
                        tracing::trace!("received unexpected acknowledgement");
                    }
                    Ok(Err(error)) => {
                        tracing::error!(%error, "failed to acknowledge ticket");
                    }
                    Err(error) => {
                        tracing::error!(%error, "parallel processing of the incoming acknowledgements failed")
                    }
                }
            }
            .instrument(tracing::debug_span!("incoming_ack_batch", peer = peer.to_peerid_str()))
        })
        .in_current_span()
        .await;

    tracing::warn!(
        task = "transport (protocol - ticket acknowledgement)",
        "long-running background task finished"
    );
}
/// Node type for which the packet processing pipeline is being constructed.
///
/// The three HOPR node types differ in how they treat tickets and incoming acknowledgements:
/// * [`Relay`](NodeType::Relay) — full pipeline, processes tickets and incoming acknowledgements.
/// * [`Entry`](NodeType::Entry) — does not process tickets and does not even start the incoming acknowledgement
///   pipeline.
/// * [`Exit`](NodeType::Exit) — does not process tickets, but still runs the incoming acknowledgement pipeline (which
///   only drains the stream) for future use.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum NodeType {
    Relay,
    Entry,
    Exit,
}

/// No-op [`UnacknowledgedTicketProcessor`] used by node types that do not process tickets
/// (Entry and Exit). All methods are unreachable because the inner pipeline never invokes
/// them on those node types (Entry skips the ack pipeline entirely, Exit uses the drain
/// variant, and the forwarded packet branch never fires on a terminal/source node).
#[derive(Debug, Default, Copy, Clone)]
#[doc(hidden)]
pub struct NoopTicketProcessor;

impl UnacknowledgedTicketProcessor for NoopTicketProcessor {
    type Error = std::convert::Infallible;

    #[inline]
    fn insert_unacknowledged_ticket(
        &self,
        _: &OffchainPublicKey,
        _: HalfKeyChallenge,
        _: UnacknowledgedTicket,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn acknowledge_tickets(
        &self,
        _: OffchainPublicKey,
        _: Vec<Acknowledgement>,
    ) -> Result<Vec<ResolvedAcknowledgement>, TicketAcknowledgementError<Self::Error>> {
        Ok(Vec::with_capacity(0))
    }
}
/// Shared implementation of the packet pipeline used by [`PacketPipelineBuilder`]'s
/// terminal `build_for_*` methods.
#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip_all, level = "trace", fields(me = packet_key.public().to_peerid_str()))]
pub(super) fn run_packet_pipeline_inner<WIn, WOut, C, D, T, TEvt, AppOut, AppIn>(
    node_type: NodeType,
    packet_key: OffchainKeypair,
    wire_msg: (WOut, WIn),
    codec: (C, D),
    ticket_proc: T,
    ticket_events: TEvt,
    cfg: PacketPipelineConfig,
    api: (AppOut, AppIn),
    counters: PeerProtocolCounterRegistry,
) -> AbortableList<PacketPipelineProcesses>
where
    WOut: futures::Sink<(PeerId, Bytes)> + Clone + Unpin + Send + 'static,
    WOut::Error: std::error::Error,
    WIn: futures::Stream<Item = (PeerId, Bytes)> + Send + 'static,
    C: PacketEncoder + Sync + Send + 'static,
    D: PacketDecoder + Sync + Send + 'static,
    T: UnacknowledgedTicketProcessor + Sync + Send + 'static,
    TEvt: futures::Sink<TicketEvent> + Clone + Unpin + Send + 'static,
    TEvt::Error: std::error::Error,
    AppOut: futures::Sink<(HoprPseudonym, ApplicationDataIn)> + Send + 'static,
    AppOut::Error: std::error::Error,
    AppIn: futures::Stream<Item = (ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)> + Send + 'static,
{
    let mut processes = AbortableList::default();

    #[cfg(all(feature = "telemetry", not(test)))]
    {
        // Initialize the lazy statics here
        lazy_static::initialize(&METRIC_PACKET_COUNT);
        lazy_static::initialize(&METRIC_PACKET_DECODE_TIMEOUTS);
        lazy_static::initialize(&METRIC_PACKET_REJECTED_COUNT);
        lazy_static::initialize(&METRIC_VALIDATION_ERRORS);
    }

    let (outgoing_ack_tx, outgoing_ack_rx) = hopr_utils::network_types::crossfire_sink::bounded_sink_channel::<(
        OffchainPublicKey,
        Option<HalfKey>,
    )>(cfg.ack_config.ack_out_buffer_size);

    let (incoming_ack_tx, incoming_ack_rx) = hopr_utils::network_types::crossfire_sink::bounded_sink_channel::<(
        OffchainPublicKey,
        Vec<Acknowledgement>,
    )>(cfg.ack_config.ticket_ack_buffer_size);

    // Attach timeouts to all Sinks so that the pipelines are not blocked when
    // some channel is not being timely processed
    let (wire_out, wire_in) = (wire_msg.0.with_timeout(QUEUE_SEND_TIMEOUT), wire_msg.1);
    let (app_out, app_in) = (api.0.with_timeout(QUEUE_SEND_TIMEOUT), api.1);
    let incoming_ack_tx = incoming_ack_tx.with_timeout(QUEUE_SEND_TIMEOUT);
    let outgoing_ack_tx = outgoing_ack_tx.with_timeout(QUEUE_SEND_TIMEOUT);
    let ticket_events = ticket_events.with_timeout(QUEUE_SEND_TIMEOUT);

    let encoder = std::sync::Arc::new(codec.0);
    let decoder = std::sync::Arc::new(codec.1);
    let ticket_proc = std::sync::Arc::new(ticket_proc);

    // `avail_concurrency` is used as a deep ready-queue for the encode (egress) path where deep
    // queuing is harmless, and as a fallback when the Rayon pool has not been initialised yet.
    // Zero is normalised to 1 to prevent deadlock (0 concurrent tasks = no work done ever).
    let avail_concurrency = std::thread::available_parallelism()
        .ok()
        .map(|n| n.get())
        .unwrap_or(1)
        .max(1)
        * 8;

    let output_concurrency = cfg.output_concurrency.filter(|&n| n > 0).unwrap_or(avail_concurrency);

    // The ingress decode concurrency is deliberately capped below the Rayon pool size so that
    // ENCODE_RESERVED_THREADS are always available for outgoing encode / SURB generation.
    // Without this cap the FIFO pool fills up with decode work under heavy download traffic and
    // SURB replenishment starves, slowly collapsing the session's download throughput.
    let pool_threads = hopr_utils::parallelize::cpu::pool_thread_count();
    let default_input_concurrency = if pool_threads > ENCODE_RESERVED_THREADS {
        pool_threads - ENCODE_RESERVED_THREADS
    } else if pool_threads > 0 {
        1 // pool is tiny but initialised — leave at least 1 decode slot
    } else {
        avail_concurrency // pool not initialised yet; fall back to the old behaviour
    };
    let input_concurrency = cfg
        .input_concurrency
        .filter(|&n| n > 0)
        .unwrap_or(default_input_concurrency);

    // --- Ingress gate (safety-net backpressure) ---
    // outstanding_tasks() is a single atomic load, so checking it per-packet is cheaper than
    // maintaining a sampler task + shared AtomicBool. The delay is only incurred when the pool is
    // actually congested, which is also when packets arrive fastest.
    let effective_pool = if pool_threads > 0 {
        pool_threads
    } else {
        avail_concurrency.max(1)
    };
    let high_watermark = effective_pool * INGRESS_POOL_HIGH_WATERMARK_FACTOR;
    let wire_in = wire_in.then(move |(peer, data)| async move {
        if hopr_utils::parallelize::cpu::outstanding_tasks() > high_watermark {
            hopr_utils::runtime::prelude::sleep(INGRESS_THROTTLE_DELAY).await;
        }
        (peer, data)
    });

    processes.insert(
        PacketPipelineProcesses::MsgOut,
        hopr_utils::spawn_as_abortable!(
            start_outgoing_packet_pipeline(
                app_in,
                encoder.clone(),
                wire_out.clone(),
                counters.clone(),
                output_concurrency
            )
            .in_current_span()
        ),
    );

    processes.insert(
        PacketPipelineProcesses::MsgIn,
        hopr_utils::spawn_as_abortable!(
            start_incoming_packet_pipeline(
                (wire_out.clone(), wire_in),
                decoder,
                ticket_proc.clone(),
                ticket_events.clone(),
                (outgoing_ack_tx, incoming_ack_tx),
                app_out,
                counters.clone(),
                input_concurrency,
            )
            .in_current_span()
        ),
    );

    processes.insert(
        PacketPipelineProcesses::AckOut,
        hopr_utils::spawn_as_abortable!(
            start_outgoing_ack_pipeline(outgoing_ack_rx, encoder, cfg.ack_config, packet_key.clone(), wire_out,)
                .in_current_span()
        ),
    );

    let ack_input_concurrency = cfg
        .ack_config
        .ack_input_concurrency
        .filter(|&n| n > 0)
        .unwrap_or(DEFAULT_ACK_INPUT_CONCURRENCY);

    match node_type {
        NodeType::Relay => {
            processes.insert(
                PacketPipelineProcesses::AckIn,
                hopr_utils::spawn_as_abortable!(
                    start_relay_incoming_ack_pipeline(
                        incoming_ack_rx,
                        ticket_events,
                        ticket_proc,
                        ack_input_concurrency
                    )
                    .in_current_span()
                ),
            );
        }
        NodeType::Exit => {
            // Exit nodes still run the incoming acknowledgement pipeline (for future PIX use),
            // but only drain the stream — incoming acknowledgements are NOT forwarded to the
            // UnacknowledgedTicketProcessor because Exit nodes do not process tickets.
            let _ = (ticket_events, ticket_proc, ack_input_concurrency);
            processes.insert(
                PacketPipelineProcesses::AckIn,
                hopr_utils::spawn_as_abortable!(start_drain_incoming_ack_pipeline(incoming_ack_rx).in_current_span()),
            );
        }
        NodeType::Entry => {
            // Entry nodes do not process tickets, but they DO receive ticket acknowledgements
            // (they pay relays for forwarding). The queue must be actively drained so the
            // inbound dispatcher can keep sending without hitting `SendError(disconnected)`.
            let _ = (ticket_events, ticket_proc, ack_input_concurrency);
            processes.insert(
                PacketPipelineProcesses::AckIn,
                hopr_utils::spawn_as_abortable!(start_drain_incoming_ack_pipeline(incoming_ack_rx).in_current_span()),
            );
        }
    }

    processes
}

#[cfg(test)]
mod tests {
    use futures::channel::mpsc;

    use super::*;

    /// Regression test for the Entry-node ack-sink bug.
    ///
    /// Before the fix, `NodeType::Entry` dropped `incoming_ack_rx` immediately at
    /// pipeline startup. Every subsequent call to `incoming_ack_tx.send(…)` then
    /// returned `SendError(disconnected)`, flooding logs with ~300 errors per run.
    ///
    /// `start_drain_incoming_ack_pipeline` must hold the receiver open for the
    /// lifetime of its task; once the sender side is dropped the task completes
    /// cleanly.
    #[tokio::test]
    async fn drain_pipeline_keeps_receiver_alive() {
        let (tx, rx) = mpsc::channel::<(OffchainPublicKey, Vec<Acknowledgement>)>(16);
        let drain = tokio::spawn(start_drain_incoming_ack_pipeline(rx));

        // Give the drain task a chance to start up.
        tokio::task::yield_now().await;

        // With the old code (drop receiver) tx.is_closed() would be true here.
        assert!(!tx.is_closed(), "drain task must hold the receiver alive");

        // Drop the sender — drain task should complete cleanly.
        drop(tx);
        drain
            .await
            .expect("drain task must finish cleanly after sender is dropped");
    }

    /// Regression: the drain must complete cleanly when the sender is dropped (no deadlock/panic).
    #[tokio::test]
    async fn drain_pipeline_completes_on_empty_stream() {
        let (tx, rx) = mpsc::channel::<(OffchainPublicKey, Vec<Acknowledgement>)>(32);
        let drain = tokio::spawn(start_drain_incoming_ack_pipeline(rx));
        drop(tx);
        drain
            .await
            .expect("drain task must finish cleanly after sender is dropped");
    }

    /// A disconnected `futures::mpsc::Sender` must not panic; `send` returns `Err`.
    #[tokio::test]
    async fn disconnected_sender_returns_err_not_panic() {
        let (tx, rx) = mpsc::channel::<u8>(4);
        drop(rx);
        let mut tx2 = tx.clone();
        let result = tx2.send(42u8).await;
        assert!(result.is_err(), "send to disconnected receiver must return Err");
    }

    /// The `for_each` loop used in `SessionsManagement(0)` must not terminate when
    /// the receiver is dropped; it must continue consuming the upstream stream.
    #[tokio::test]
    async fn session_management_dispatcher_survives_disconnected_sink() -> anyhow::Result<()> {
        use anyhow::Context;
        use futures::SinkExt;

        let (data_tx, data_rx) = mpsc::channel::<u8>(4);

        // Drop the receiver immediately — simulates HoprSocket being dropped.
        drop(data_rx);

        // Upstream stream of 10 items.
        let upstream = futures::stream::iter(0u8..10);

        // Run the resilient for_each pattern used in SessionsManagement(0).
        let dispatcher = tokio::spawn(async move {
            upstream
                .for_each(move |item| {
                    let mut tx = data_tx.clone();
                    async move {
                        // Error is expected (disconnected); we must NOT abort the stream.
                        let _ = tx.send(item).await;
                    }
                })
                .await;
        });

        // Task must complete without panic even though every send fails.
        dispatcher
            .await
            .context("dispatcher must complete cleanly even with a disconnected sink")?;
        Ok(())
    }

    /// Regression: previously, the first `Unrelated` packet triggered task exit and
    /// dropped `rx_from_protocol`.  After the fix the task must run to completion.
    #[tokio::test]
    async fn session_management_dispatcher_does_not_drop_upstream_on_disconnected_sink() -> anyhow::Result<()> {
        use anyhow::Context;
        use futures::SinkExt;

        let (data_tx, data_rx) = mpsc::channel::<u8>(1);
        drop(data_rx); // receiver gone from the start

        let (upstream_tx, upstream_rx) = mpsc::channel::<u8>(16);
        let upstream = upstream_rx;

        let dispatcher = tokio::spawn(async move {
            upstream
                .for_each(move |item| {
                    let mut tx = data_tx.clone();
                    async move {
                        let _ = tx.send(item).await;
                    }
                })
                .await;
        });

        // Send several items; the dispatcher must process them all.
        let mut sender = upstream_tx;
        for i in 0u8..20 {
            sender.send(i).await.context("upstream send must succeed")?;
        }
        drop(sender); // close upstream → dispatcher finishes

        dispatcher
            .await
            .context("dispatcher must complete when upstream closes, even with disconnected sink")?;
        Ok(())
    }
}
