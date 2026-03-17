use futures::{SinkExt, StreamExt};
use futures_time::{future::FutureExt as TimeExt, stream::StreamExt as TimeStreamExt};
use hopr_api::types::{crypto::prelude::*, internal::prelude::*, primitive::prelude::Address};
use hopr_async_runtime::{AbortableList, spawn_as_abortable};
use hopr_crypto_packet::HoprSurb;
use hopr_network_types::timeout::{SinkTimeoutError, TimeoutSinkExt, TimeoutStreamExt};
use hopr_protocol_app::prelude::*;
use hopr_protocol_hopr::prelude::*;
use rust_stream_ext_concurrent::then_concurrent::StreamThenConcurrentExt;
use tracing::Instrument;
use validator::{Validate, ValidationError, ValidationErrors};

use crate::PeerId;

const TICKET_ACK_BUFFER_SIZE: usize = 1_000_000;
const NUM_CONCURRENT_TICKET_ACK_PROCESSING: usize = 10;
const ACK_OUT_BUFFER_SIZE: usize = 1_000_000;
const NUM_CONCURRENT_ACK_OUT_PROCESSING: usize = 10;
const QUEUE_SEND_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(50);
const PACKET_DECODING_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(150);

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_PACKET_COUNT:  hopr_metrics::MultiCounter =  hopr_metrics::MultiCounter::new(
        "hopr_packets_count",
        "Number of processed packets of different types (sent, received, forwarded)",
        &["type"]
    ).unwrap();
    // Tracks how often the Rayon-backed packet decode path exceeds PACKET_DECODING_TIMEOUT.
    // A sustained non-zero rate here indicates the Rayon pool is saturated—correlate with
    // hopr_rayon_tasks_cancelled_total and hopr_rayon_queue_wait_seconds to diagnose whether
    // the bottleneck is queue depth, individual task duration, or both.
    static ref METRIC_PACKET_DECODE_TIMEOUTS: hopr_metrics::SimpleCounter = hopr_metrics::SimpleCounter::new(
        "hopr_packet_decode_timeouts_total",
        "Number of incoming packets dropped due to decode timeout (sustained rate indicates Rayon pool saturation)"
    ).unwrap();
    static ref METRIC_VALIDATION_ERRORS: hopr_metrics::MultiCounter =  hopr_metrics::MultiCounter::new(
        "hopr_packet_ticket_validation_errors",
        "Number of different ticket validation errors encountered during packet processing",
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

/// Ticket events emitted from the packet processing pipeline.
#[derive(Debug, Clone, strum::EnumIs, strum::EnumTryAs)]
pub enum TicketEvent {
    /// A winning ticket was received.
    WinningTicket(Box<RedeemableTicket>),
    /// A ticket has been rejected.
    RejectedTicket(Box<Ticket>, Option<Address>),
}

/// Performs encoding of outgoing Application protocol packets into HOPR protocol outgoing packets.
async fn start_outgoing_packet_pipeline<AppOut, E, WOut, WOutErr>(
    app_outgoing: AppOut,
    encoder: std::sync::Arc<E>,
    wire_outgoing: WOut,
    counters: crate::counters::PeerProtocolCounterRegistry,
    concurrency: usize,
) where
    AppOut: futures::Stream<Item = (ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)> + Send + 'static,
    E: PacketEncoder + Send + 'static,
    WOut: futures::Sink<(PeerId, Box<[u8]>), Error = SinkTimeoutError<WOutErr>> + Clone + Unpin + Send + 'static,
    WOutErr: std::error::Error,
{
    let res = app_outgoing
        .then_concurrent(
            |(routing, data)| {
                let encoder = encoder.clone();
                let counters = counters.clone();
                async move {
                    match encoder
                        .encode_packet(
                            data.data.to_bytes(),
                            routing,
                            data.packet_info
                                .map(|data| data.signals_to_destination)
                                .unwrap_or_default(),
                        )
                        .await
                    {
                        Ok(packet) => {
                            #[cfg(all(feature = "telemetry", not(test)))]
                            METRIC_PACKET_COUNT.increment(&["sent"]);

                            counters.get_or_create(&packet.next_hop).record_message_sent();
                            tracing::trace!(peer = packet.next_hop.to_peerid_str(), "protocol message out");
                            Some((packet.next_hop.into(), packet.data))
                        }
                        Err(error) => {
                            tracing::error!(%error, "packet could not be wrapped for sending");
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
    counters: crate::counters::PeerProtocolCounterRegistry,
    concurrency: usize,
) where
    WIn: futures::Stream<Item = (PeerId, Box<[u8]>)> + Send + 'static,
    WOut: futures::Sink<(PeerId, Box<[u8]>)> + Clone + Unpin + Send + 'static,
    WOut::Error: std::error::Error,
    D: PacketDecoder + Send + 'static,
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
                match decoder.decode(peer, data)
                    .timeout(futures_time::time::Duration::from(PACKET_DECODING_TIMEOUT))
                    .await {
                    Ok(Ok(packet)) => {
                        tracing::trace!(%peer, ?packet, "successfully decoded incoming packet");
                        Some(packet)
                    },
                    Ok(Err(IncomingPacketError::Overloaded(error))) => {
                        tracing::warn!(%peer, %error, "dropping packet due to local CPU overload");
                        None
                    },
                    Ok(Err(IncomingPacketError::Undecodable(error))) => {
                        // Do not send an ack back if the packet could not be decoded at all
                        //
                        // Potentially adversarial behavior
                        tracing::trace!(%peer, %error, "not sending ack back on undecodable packet - possible adversarial behavior");
                        None
                    },
                    Ok(Err(IncomingPacketError::ProcessingError(sender, error))) => {
                        tracing::error!(%peer, %error, "failed to process the decoded packet");
                        // On this failure, we send back a random acknowledgement
                        ack_outgoing_failure
                            .send((sender, None))
                            .await
                            .unwrap_or_else(|error| {
                                tracing::error!(%error, "failed to send ack to the egress queue");
                            });
                        None
                    },
                    Ok(Err(IncomingPacketError::InvalidTicket(sender, error))) => {
                        tracing::error!(%peer, %error, "failed to validate ticket on the received packet");
                        if let Err(error) = ticket_events_reject
                            .send(TicketEvent::RejectedTicket(error.ticket, error.issuer))
                            .await {
                            tracing::error!(%error, "failed to notify invalid ticket rejection");
                        }
                        // On this failure, we send back a random acknowledgement
                        ack_outgoing_failure
                            .send((sender, None))
                            .await
                            .unwrap_or_else(|error| {
                                tracing::error!(%error, "failed to send ack to the egress queue");
                            });

                        #[cfg(all(feature = "telemetry", not(test)))]
                        METRIC_VALIDATION_ERRORS.increment(&[error.kind.as_ref()]);

                        None
                    }
                    Err(_) => {
                        // If we cannot decode the packet within the time limit, just drop it
                        tracing::error!(
                            %peer,
                            timeout_ms = PACKET_DECODING_TIMEOUT.as_millis() as u64,
                            "dropped incoming packet: decode timeout - check the 'hopr_rayon_queue_wait_seconds' metric for pool saturation"
                        );
                        #[cfg(all(feature = "telemetry", not(test)))]
                        METRIC_PACKET_DECODE_TIMEOUTS.increment();

                        None
                    }
                }
            }.instrument(tracing::debug_span!("incoming_packet_decode", %peer))
        }, concurrency)
        .filter_map(futures::future::ready)
        .then_concurrent(move |packet| {
            let ticket_proc = ticket_proc_success.clone();
            let mut wire_outgoing = wire_outgoing.clone();
            let mut ack_incoming = ack_incoming.clone();
            let mut ack_outgoing_success = ack_outgoing_success.clone();
            let counters = counters.clone();
            async move {
                match packet {
                    IncomingPacket::Acknowledgement(ack) => {
                        let IncomingAcknowledgementPacket { previous_hop, received_acks, .. } = *ack;
                        tracing::trace!(previous_hop = previous_hop.to_peerid_str(), num_acks = received_acks.len(), "incoming acknowledgements");
                        ack_incoming
                            .send((previous_hop, received_acks))
                            .await
                            .unwrap_or_else(|error| {
                                tracing::error!(%error, "failed dispatching received acknowledgement to the ticket ack queue");
                            });

                        // We do not acknowledge back acknowledgements.
                        None
                    },
                    IncomingPacket::Final(final_packet) => {
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
                    }
                    IncomingPacket::Forwarded(fwd_packet) => {
                        let IncomingForwardedPacket {
                            previous_hop,
                            next_hop,
                            data,
                            ack_key_prev_hop,
                            ack_challenge,
                            received_ticket,
                            ..
                        } = *fwd_packet;
                        if let Err(error) = ticket_proc.insert_unacknowledged_ticket(&next_hop, ack_challenge, received_ticket).await {
                            tracing::error!(
                                previous_hop = previous_hop.to_peerid_str(),
                                next_hop = next_hop.to_peerid_str(),
                                %error,
                                "failed to insert unacknowledged ticket into the ticket processor"
                            );
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
                    }
                }
            }}, concurrency)
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
    E: PacketEncoder + Send + 'static,
    WOut: futures::Sink<(PeerId, Box<[u8]>)> + Clone + Unpin + Send + 'static,
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
        .flat_map(|buffered_acks| {
            // Split the acknowledgements into groups based on the sender
            // The halfbrown hash map will use Vec for a lower number of distinct senders, and possibly transition to
            // the hashbrown hash map when the number of distinct senders exceeds 32.
            let mut groups = halfbrown::HashMap::<OffchainPublicKey, Vec<VerifiedAcknowledgement>, ahash::RandomState>::with_capacity_and_hasher(
                cfg.ack_grouping_capacity,
                ahash::RandomState::default()
            );
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
            futures::stream::iter(groups)
        })
        .for_each_concurrent(
            NUM_CONCURRENT_ACK_OUT_PROCESSING,
            move |(destination, acks)| {
                let encoder = encoder.clone();
                let mut wire_outgoing = wire_outgoing.clone();
                async move {
                    // Make sure that the acknowledgements are sent in batches of at most MAX_ACKNOWLEDGEMENTS_BATCH_SIZE
                    for ack_chunk in acks.chunks(MAX_ACKNOWLEDGEMENTS_BATCH_SIZE) {
                        match encoder.encode_acknowledgements(ack_chunk, &destination).await {
                            Ok(ack_packet) => {
                                wire_outgoing
                                    .feed((ack_packet.next_hop.into(), ack_packet.data))
                                    .await
                                    .unwrap_or_else(|error| {
                                        tracing::error!(%error, "failed to forward an acknowledgement to the transport layer");
                                    });
                            }
                            Err(error) => tracing::error!(%error, "failed to create ack packet"),
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

async fn start_incoming_ack_pipeline<AckIn, T, TEvt>(
    ack_incoming: AckIn,
    ticket_events: TEvt,
    ticket_proc: std::sync::Arc<T>,
    counters: crate::counters::PeerProtocolCounterRegistry,
) where
    AckIn: futures::Stream<Item = (OffchainPublicKey, Vec<Acknowledgement>)> + Send + 'static,
    T: UnacknowledgedTicketProcessor + Sync + Send + 'static,
    TEvt: futures::Sink<TicketEvent> + Clone + Unpin + Send + 'static,
    TEvt::Error: std::error::Error,
{
    ack_incoming
        .for_each_concurrent(NUM_CONCURRENT_TICKET_ACK_PROCESSING, move |(peer, acks)| {
            let ticket_proc = ticket_proc.clone();
            let mut ticket_evt = ticket_events.clone();
            let counters = counters.clone();
            async move {
                counters.get_or_create(&peer).record_acks_received(acks.len() as u64);
                tracing::trace!(num = acks.len(), "received acknowledgements");
                match ticket_proc.acknowledge_tickets(peer, acks).await {
                    Ok(resolutions) if !resolutions.is_empty() => {
                        let resolutions_iter = resolutions.into_iter().filter_map(|resolution| match resolution {
                            ResolvedAcknowledgement::RelayingWin(redeemable_ticket) => {
                                tracing::trace!("received ack for a winning ticket");
                                Some(Ok(TicketEvent::WinningTicket(redeemable_ticket)))
                            }
                            ResolvedAcknowledgement::RelayingLoss(_) => {
                                // Losing tickets are not getting accounted for anywhere.
                                tracing::trace!("received ack for a losing ticket");
                                None
                            }
                        });

                        // All acknowledgements that resulted in winning tickets go upstream
                        if let Err(error) = ticket_evt.send_all(&mut futures::stream::iter(resolutions_iter)).await {
                            tracing::error!(%error, "failed to notify ticket resolutions");
                        }
                    }
                    Ok(_) => {
                        tracing::debug!("acknowledgement batch could not acknowledge any ticket");
                    }
                    Err(TicketAcknowledgementError::UnexpectedAcknowledgement) => {
                        // Unexpected acknowledgements naturally happen
                        // as acknowledgements of 0-hop packets
                        tracing::trace!("received unexpected acknowledgement");
                    }
                    Err(error) => {
                        tracing::error!(%error, "failed to acknowledge ticket");
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

fn default_ack_buffer_interval() -> std::time::Duration {
    std::time::Duration::from_millis(200)
}

fn default_ack_grouping_capacity() -> usize {
    5
}

/// Configuration for the acknowledgement processing pipeline.
#[derive(Debug, Copy, Clone, smart_default::SmartDefault, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AcknowledgementPipelineConfig {
    /// Interval for which to wait to buffer acknowledgements before sending them out.
    ///
    /// Default is 200 ms.
    #[default(default_ack_buffer_interval())]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_ack_buffer_interval", with = "humantime_serde")
    )]
    pub ack_buffer_interval: std::time::Duration,
    /// Initial capacity when grouping outgoing acknowledgements.
    ///
    /// If set too low, it causes additional reallocations in the outgoing acknowledgement processing pipeline.
    /// The value should grow if `ack_buffer_interval` grows.
    ///
    /// Default is 5.
    #[default(default_ack_grouping_capacity())]
    #[cfg_attr(feature = "serde", serde(default = "default_ack_grouping_capacity"))]
    pub ack_grouping_capacity: usize,
}

// Requires manual implementation due to https://github.com/Keats/validator/issues/285
impl Validate for AcknowledgementPipelineConfig {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        if self.ack_grouping_capacity == 0 {
            errors.add("ack_grouping_capacity", ValidationError::new("must be greater than 0"));
        }
        if self.ack_buffer_interval < std::time::Duration::from_millis(10) {
            errors.add("ack_buffer_interval", ValidationError::new("must be at least 10 ms"));
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// Overall configuration of the input/output packet processing pipeline.
#[derive(Clone, Copy, Debug, Default, PartialEq, Validate)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct PacketPipelineConfig {
    /// Maximum concurrency when processing outgoing packets.
    ///
    /// `None` or `Some(0)` both fall back to the default (available parallelism * 8).
    pub output_concurrency: Option<usize>,
    /// Maximum concurrency when processing incoming packets.
    ///
    /// `None` or `Some(0)` both fall back to the default (available parallelism * 8).
    pub input_concurrency: Option<usize>,
    /// Configuration of the packet acknowledgement processing
    #[validate(nested)]
    pub ack_config: AcknowledgementPipelineConfig,
}

/// Run all processes responsible for handling the msg and acknowledgment protocols.
///
/// The pipeline does not handle the mixing itself, that needs to be injected as a separate process
/// overlay on top of the `wire_msg` Stream or Sink.
#[tracing::instrument(skip_all, level = "trace", fields(me = packet_key.public().to_peerid_str()))]
#[allow(clippy::too_many_arguments)]
pub fn run_packet_pipeline<WIn, WOut, C, D, T, TEvt, AppOut, AppIn>(
    packet_key: OffchainKeypair,
    wire_msg: (WOut, WIn),
    codec: (C, D),
    ticket_proc: T,
    ticket_events: TEvt,
    cfg: PacketPipelineConfig,
    api: (AppOut, AppIn),
    counters: crate::counters::PeerProtocolCounterRegistry,
) -> AbortableList<PacketPipelineProcesses>
where
    WOut: futures::Sink<(PeerId, Box<[u8]>)> + Clone + Unpin + Send + 'static,
    WOut::Error: std::error::Error,
    WIn: futures::Stream<Item = (PeerId, Box<[u8]>)> + Send + 'static,
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
        lazy_static::initialize(&METRIC_VALIDATION_ERRORS);
    }

    let (outgoing_ack_tx, outgoing_ack_rx) =
        futures::channel::mpsc::channel::<(OffchainPublicKey, Option<HalfKey>)>(ACK_OUT_BUFFER_SIZE);

    let (incoming_ack_tx, incoming_ack_rx) =
        futures::channel::mpsc::channel::<(OffchainPublicKey, Vec<Acknowledgement>)>(TICKET_ACK_BUFFER_SIZE);

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

    // Default maximum concurrency (if not set or zero) is 8 times the number of available cores.
    // Zero is normalized to the default to prevent deadlock (0 concurrent tasks = no work).
    let avail_concurrency = std::thread::available_parallelism()
        .ok()
        .map(|n| n.get())
        .unwrap_or(1)
        .max(1)
        * 8;

    let output_concurrency = cfg.output_concurrency.filter(|&n| n > 0).unwrap_or(avail_concurrency);
    let input_concurrency = cfg.input_concurrency.filter(|&n| n > 0).unwrap_or(avail_concurrency);

    processes.insert(
        PacketPipelineProcesses::MsgOut,
        spawn_as_abortable!(
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
        spawn_as_abortable!(
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
        spawn_as_abortable!(
            start_outgoing_ack_pipeline(outgoing_ack_rx, encoder, cfg.ack_config, packet_key.clone(), wire_out,)
                .in_current_span()
        ),
    );

    processes.insert(
        PacketPipelineProcesses::AckIn,
        spawn_as_abortable!(
            start_incoming_ack_pipeline(incoming_ack_rx, ticket_events, ticket_proc, counters).in_current_span()
        ),
    );

    processes
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::Context;
    use futures::{SinkExt, StreamExt, channel::mpsc};
    use hex_literal::hex;
    use hopr_api::types::{
        crypto::{
            prelude::{Keypair, OffchainKeypair, OffchainPublicKey, SimplePseudonym},
            types::HalfKey,
        },
        crypto_random::Randomizable,
        internal::prelude::*,
    };
    use hopr_crypto_packet::{HoprSurb, prelude::PacketSignals};
    use hopr_protocol_app::prelude::*;
    use hopr_protocol_hopr::prelude::*;
    use validator::Validate;

    use super::*;

    lazy_static::lazy_static! {
        static ref PEERS: Vec<OffchainKeypair> = [
            hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"),
            hex!("5bf21ea8cccd69aa784346b07bf79c84dac606e00eecaa68bf8c31aff397b1ca"),
            hex!("3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa"),
        ]
        .iter()
        .map(|private| OffchainKeypair::from_secret(private).expect("lazy static keypair should be valid"))
        .collect();
    }

    // --- Concrete error type for test doubles (std::error::Error required by trait bounds) ---

    #[derive(Debug, thiserror::Error)]
    #[error("{0}")]
    struct TestError(String);

    // --- Config validation tests ---

    #[test]
    fn ack_pipeline_config_default_is_valid() {
        let cfg = AcknowledgementPipelineConfig::default();
        assert!(cfg.validate().is_ok());
        insta::assert_yaml_snapshot!(format!("{cfg:?}"));
    }

    #[test]
    fn ack_pipeline_config_zero_grouping_capacity_is_rejected() -> anyhow::Result<()> {
        let cfg = AcknowledgementPipelineConfig {
            ack_grouping_capacity: 0,
            ..Default::default()
        };
        let err = cfg.validate().err().context("expected validation error")?;
        insta::assert_yaml_snapshot!(format!("{err}"));
        Ok(())
    }

    #[test]
    fn ack_pipeline_config_too_short_buffer_interval_is_rejected() -> anyhow::Result<()> {
        let cfg = AcknowledgementPipelineConfig {
            ack_buffer_interval: Duration::from_millis(5),
            ..Default::default()
        };
        let err = cfg.validate().err().context("expected validation error")?;
        insta::assert_yaml_snapshot!(format!("{err}"));
        Ok(())
    }

    #[test]
    fn ack_pipeline_config_both_fields_invalid() {
        let cfg = AcknowledgementPipelineConfig {
            ack_grouping_capacity: 0,
            ack_buffer_interval: Duration::from_millis(1),
        };
        let err = cfg.validate().unwrap_err();
        let err_str = format!("{err}");
        assert!(
            err_str.contains("ack_grouping_capacity") && err_str.contains("ack_buffer_interval"),
            "expected both fields in error, got: {err_str}"
        );
    }

    #[test]
    fn ack_pipeline_config_boundary_10ms_is_accepted() {
        let cfg = AcknowledgementPipelineConfig {
            ack_buffer_interval: Duration::from_millis(10),
            ..Default::default()
        };
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn packet_pipeline_config_default_is_valid() {
        let cfg = PacketPipelineConfig::default();
        assert!(cfg.validate().is_ok());
        insta::assert_yaml_snapshot!(format!("{cfg:?}"));
    }

    #[test]
    fn packet_pipeline_config_with_invalid_nested_ack_config() {
        let cfg = PacketPipelineConfig {
            ack_config: AcknowledgementPipelineConfig {
                ack_grouping_capacity: 0,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(cfg.validate().is_err());
    }

    // --- PacketPipelineProcesses display tests ---

    #[test]
    fn pipeline_process_names_are_stable() {
        let names: Vec<String> = [
            PacketPipelineProcesses::MsgIn,
            PacketPipelineProcesses::MsgOut,
            PacketPipelineProcesses::AckOut,
            PacketPipelineProcesses::AckIn,
            PacketPipelineProcesses::Mixer,
        ]
        .iter()
        .map(|p| p.to_string())
        .collect();
        insta::assert_yaml_snapshot!(names);
    }

    // --- Test doubles for pipeline behavior tests ---

    /// A test encoder that returns a fixed outgoing packet.
    struct TestEncoder {
        next_hop: OffchainPublicKey,
    }

    #[async_trait::async_trait]
    impl PacketEncoder for TestEncoder {
        type Error = TestError;

        async fn encode_packet<T: AsRef<[u8]> + Send + 'static, S: Into<PacketSignals> + Send + 'static>(
            &self,
            data: T,
            _routing: ResolvedTransportRouting<HoprSurb>,
            _signals: S,
        ) -> Result<OutgoingPacket, Self::Error> {
            Ok(OutgoingPacket {
                next_hop: self.next_hop,
                ack_challenge: HalfKeyChallenge::default(),
                data: data.as_ref().into(),
            })
        }

        async fn encode_acknowledgements(
            &self,
            _acks: &[VerifiedAcknowledgement],
            destination: &OffchainPublicKey,
        ) -> Result<OutgoingPacket, Self::Error> {
            Ok(OutgoingPacket {
                next_hop: *destination,
                ack_challenge: HalfKeyChallenge::default(),
                data: vec![0xAC, 0x4B].into_boxed_slice(),
            })
        }
    }

    /// A test decoder that returns a configurable result.
    struct TestDecoder {
        result: std::sync::Mutex<Option<Result<IncomingPacket, IncomingPacketError<TestError>>>>,
    }

    impl TestDecoder {
        fn returning_final(previous_hop: OffchainPublicKey) -> Self {
            let app_data = ApplicationData::new(42u64, b"hello world").expect("valid app data");
            Self {
                result: std::sync::Mutex::new(Some(Ok(IncomingPacket::Final(Box::new(IncomingFinalPacket {
                    packet_tag: [0u8; 16],
                    previous_hop,
                    sender: SimplePseudonym::random(),
                    plain_text: app_data.to_bytes(),
                    ack_key: HalfKey::random(),
                    info: Default::default(),
                }))))),
            }
        }

        fn returning_undecodable() -> Self {
            Self {
                result: std::sync::Mutex::new(Some(Err(IncomingPacketError::Undecodable(TestError(
                    "cannot decode".into(),
                ))))),
            }
        }

        fn returning_overloaded() -> Self {
            Self {
                result: std::sync::Mutex::new(Some(Err(IncomingPacketError::Overloaded(TestError(
                    "cpu overload".into(),
                ))))),
            }
        }

        fn returning_processing_error(sender: OffchainPublicKey) -> Self {
            Self {
                result: std::sync::Mutex::new(Some(Err(IncomingPacketError::ProcessingError(
                    sender,
                    TestError("processing failed".into()),
                )))),
            }
        }
    }

    #[async_trait::async_trait]
    impl PacketDecoder for TestDecoder {
        type Error = TestError;

        async fn decode(
            &self,
            _sender: PeerId,
            _data: Box<[u8]>,
        ) -> Result<IncomingPacket, IncomingPacketError<Self::Error>> {
            self.result
                .lock()
                .unwrap()
                .take()
                .expect("TestDecoder can only be called once")
        }
    }

    /// A no-op ticket processor that always succeeds.
    struct NoOpTicketProcessor;

    #[async_trait::async_trait]
    impl UnacknowledgedTicketProcessor for NoOpTicketProcessor {
        type Error = TestError;

        async fn insert_unacknowledged_ticket(
            &self,
            _next_hop: &OffchainPublicKey,
            _challenge: HalfKeyChallenge,
            _ticket: UnacknowledgedTicket,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn acknowledge_tickets(
            &self,
            _peer: OffchainPublicKey,
            _acks: Vec<Acknowledgement>,
        ) -> Result<Vec<ResolvedAcknowledgement>, TicketAcknowledgementError<Self::Error>> {
            Err(TicketAcknowledgementError::UnexpectedAcknowledgement)
        }
    }

    // --- Pipeline assembly tests ---

    #[tokio::test]
    async fn pipeline_creates_four_processes() -> anyhow::Result<()> {
        let packet_key = PEERS[0].clone();
        let next_hop = *PEERS[1].public();

        let (wire_out_tx, _wire_out_rx) = mpsc::unbounded::<(PeerId, Box<[u8]>)>();
        let (_wire_in_tx, wire_in_rx) = mpsc::unbounded::<(PeerId, Box<[u8]>)>();
        let (ticket_evt_tx, _ticket_evt_rx) = mpsc::unbounded::<TicketEvent>();
        let (_app_in_tx, app_in_rx) = mpsc::unbounded::<(ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>();
        let (app_out_tx, _app_out_rx) = mpsc::unbounded::<(HoprPseudonym, ApplicationDataIn)>();

        let encoder = TestEncoder { next_hop };
        let decoder = TestDecoder::returning_final(next_hop);

        let processes = run_packet_pipeline(
            packet_key,
            (wire_out_tx, wire_in_rx),
            (encoder, decoder),
            NoOpTicketProcessor,
            ticket_evt_tx,
            PacketPipelineConfig::default(),
            (app_out_tx, app_in_rx),
            Default::default(),
        );

        // The pipeline should create exactly 4 processes: MsgOut, MsgIn, AckOut, AckIn
        assert_eq!(processes.size(), 4);

        processes.abort_all();
        Ok(())
    }

    #[tokio::test]
    async fn incoming_final_packet_reaches_app_output() -> anyhow::Result<()> {
        let packet_key = PEERS[0].clone();
        let previous_hop = *PEERS[1].public();

        let (wire_out_tx, _wire_out_rx) = mpsc::unbounded::<(PeerId, Box<[u8]>)>();
        let (mut wire_in_tx, wire_in_rx) = mpsc::unbounded::<(PeerId, Box<[u8]>)>();
        let (ticket_evt_tx, _ticket_evt_rx) = mpsc::unbounded::<TicketEvent>();
        let (_app_in_tx, app_in_rx) = mpsc::unbounded::<(ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>();
        let (app_out_tx, mut app_out_rx) = mpsc::unbounded::<(HoprPseudonym, ApplicationDataIn)>();

        let encoder = TestEncoder { next_hop: previous_hop };
        let decoder = TestDecoder::returning_final(previous_hop);

        let processes = run_packet_pipeline(
            packet_key,
            (wire_out_tx, wire_in_rx),
            (encoder, decoder),
            NoOpTicketProcessor,
            ticket_evt_tx,
            PacketPipelineConfig {
                output_concurrency: Some(1),
                input_concurrency: Some(1),
                ..Default::default()
            },
            (app_out_tx, app_in_rx),
            Default::default(),
        );

        let sender_peer = PeerId::from(previous_hop);
        wire_in_tx
            .send((sender_peer, vec![0u8; 100].into_boxed_slice()))
            .await
            .context("sending wire packet")?;
        drop(wire_in_tx);

        let (_pseudonym, app_data_in) = tokio::time::timeout(Duration::from_secs(2), app_out_rx.next())
            .await
            .context("should receive app data within timeout")?
            .context("app output should have data")?;
        assert_eq!(app_data_in.data.application_tag, 42u64.into());
        assert_eq!(&*app_data_in.data.plain_text, b"hello world");

        processes.abort_all();
        Ok(())
    }

    #[tokio::test]
    async fn incoming_undecodable_packet_is_dropped_silently() -> anyhow::Result<()> {
        let packet_key = PEERS[0].clone();
        let previous_hop = *PEERS[1].public();

        let (wire_out_tx, _wire_out_rx) = mpsc::unbounded::<(PeerId, Box<[u8]>)>();
        let (mut wire_in_tx, wire_in_rx) = mpsc::unbounded::<(PeerId, Box<[u8]>)>();
        let (ticket_evt_tx, _ticket_evt_rx) = mpsc::unbounded::<TicketEvent>();
        let (_app_in_tx, app_in_rx) = mpsc::unbounded::<(ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>();
        let (app_out_tx, mut app_out_rx) = mpsc::unbounded::<(HoprPseudonym, ApplicationDataIn)>();

        let encoder = TestEncoder { next_hop: previous_hop };
        let decoder = TestDecoder::returning_undecodable();

        let processes = run_packet_pipeline(
            packet_key,
            (wire_out_tx, wire_in_rx),
            (encoder, decoder),
            NoOpTicketProcessor,
            ticket_evt_tx,
            PacketPipelineConfig {
                output_concurrency: Some(1),
                input_concurrency: Some(1),
                ..Default::default()
            },
            (app_out_tx, app_in_rx),
            Default::default(),
        );

        let sender_peer = PeerId::from(previous_hop);
        wire_in_tx
            .send((sender_peer, vec![0u8; 100].into_boxed_slice()))
            .await?;
        drop(wire_in_tx);

        let result = tokio::time::timeout(Duration::from_millis(500), app_out_rx.next()).await;
        match result {
            Err(_) => {}   // timeout — no data, as expected
            Ok(None) => {} // stream closed
            Ok(Some(_)) => panic!("undecodable packet should not produce app output"),
        }

        processes.abort_all();
        Ok(())
    }

    #[tokio::test]
    async fn incoming_overloaded_packet_is_dropped() -> anyhow::Result<()> {
        let packet_key = PEERS[0].clone();
        let previous_hop = *PEERS[1].public();

        let (wire_out_tx, _wire_out_rx) = mpsc::unbounded::<(PeerId, Box<[u8]>)>();
        let (mut wire_in_tx, wire_in_rx) = mpsc::unbounded::<(PeerId, Box<[u8]>)>();
        let (ticket_evt_tx, _ticket_evt_rx) = mpsc::unbounded::<TicketEvent>();
        let (_app_in_tx, app_in_rx) = mpsc::unbounded::<(ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>();
        let (app_out_tx, mut app_out_rx) = mpsc::unbounded::<(HoprPseudonym, ApplicationDataIn)>();

        let encoder = TestEncoder { next_hop: previous_hop };
        let decoder = TestDecoder::returning_overloaded();

        let processes = run_packet_pipeline(
            packet_key,
            (wire_out_tx, wire_in_rx),
            (encoder, decoder),
            NoOpTicketProcessor,
            ticket_evt_tx,
            PacketPipelineConfig {
                output_concurrency: Some(1),
                input_concurrency: Some(1),
                ..Default::default()
            },
            (app_out_tx, app_in_rx),
            Default::default(),
        );

        let sender_peer = PeerId::from(previous_hop);
        wire_in_tx
            .send((sender_peer, vec![0u8; 100].into_boxed_slice()))
            .await?;
        drop(wire_in_tx);

        let result = tokio::time::timeout(Duration::from_millis(500), app_out_rx.next()).await;
        match result {
            Err(_) | Ok(None) => {} // expected — packet dropped
            Ok(Some(_)) => panic!("overloaded packet should not produce app output"),
        }

        processes.abort_all();
        Ok(())
    }

    #[tokio::test]
    async fn incoming_processing_error_sends_random_ack() -> anyhow::Result<()> {
        let packet_key = PEERS[0].clone();
        let sender_key = *PEERS[1].public();

        let (wire_out_tx, mut wire_out_rx) = mpsc::unbounded::<(PeerId, Box<[u8]>)>();
        let (mut wire_in_tx, wire_in_rx) = mpsc::unbounded::<(PeerId, Box<[u8]>)>();
        let (ticket_evt_tx, _ticket_evt_rx) = mpsc::unbounded::<TicketEvent>();
        let (_app_in_tx, app_in_rx) = mpsc::unbounded::<(ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>();
        let (app_out_tx, _app_out_rx) = mpsc::unbounded::<(HoprPseudonym, ApplicationDataIn)>();

        let encoder = TestEncoder { next_hop: sender_key };
        let decoder = TestDecoder::returning_processing_error(sender_key);

        let processes = run_packet_pipeline(
            packet_key,
            (wire_out_tx, wire_in_rx),
            (encoder, decoder),
            NoOpTicketProcessor,
            ticket_evt_tx,
            PacketPipelineConfig {
                output_concurrency: Some(1),
                input_concurrency: Some(1),
                ack_config: AcknowledgementPipelineConfig {
                    ack_buffer_interval: Duration::from_millis(50),
                    ack_grouping_capacity: 5,
                },
            },
            (app_out_tx, app_in_rx),
            Default::default(),
        );

        let sender_peer = PeerId::from(sender_key);
        wire_in_tx
            .send((sender_peer, vec![0u8; 100].into_boxed_slice()))
            .await?;
        drop(wire_in_tx);

        // The processing error should trigger a random ack to be sent on the wire.
        let result = tokio::time::timeout(Duration::from_secs(2), wire_out_rx.next()).await;
        assert!(
            result.is_ok(),
            "should receive an ack packet on the wire after processing error"
        );
        let (peer_id, data) = result.unwrap().context("wire output should have an ack")?;
        assert_eq!(peer_id, sender_peer);
        // The TestEncoder returns [0xAC, 0x4B] for ack encoding
        assert_eq!(&*data, &[0xAC, 0x4B]);

        processes.abort_all();
        Ok(())
    }

    #[test]
    fn concurrency_none_falls_back_to_default() {
        let avail = std::thread::available_parallelism()
            .ok()
            .map(|n| n.get())
            .unwrap_or(1)
            .max(1)
            * 8;

        let result: usize = None::<usize>.filter(|&n| n > 0).unwrap_or(avail);
        assert_eq!(result, avail);
    }

    #[test]
    fn concurrency_zero_falls_back_to_default() {
        let avail = std::thread::available_parallelism()
            .ok()
            .map(|n| n.get())
            .unwrap_or(1)
            .max(1)
            * 8;

        let result: usize = Some(0usize).filter(|&n| n > 0).unwrap_or(avail);
        assert_eq!(result, avail);
    }

    #[test]
    fn concurrency_explicit_value_is_used() {
        let avail = std::thread::available_parallelism()
            .ok()
            .map(|n| n.get())
            .unwrap_or(1)
            .max(1)
            * 8;

        let result: usize = Some(42usize).filter(|&n| n > 0).unwrap_or(avail);
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn outgoing_pipeline_should_encode_and_forward_packet_to_wire() -> anyhow::Result<()> {
        let packet_key = PEERS[0].clone();
        let next_hop = *PEERS[1].public();

        let (wire_out_tx, mut wire_out_rx) = mpsc::unbounded::<(PeerId, Box<[u8]>)>();
        let (_wire_in_tx, wire_in_rx) = mpsc::unbounded::<(PeerId, Box<[u8]>)>();
        let (ticket_evt_tx, _ticket_evt_rx) = mpsc::unbounded::<TicketEvent>();
        let (mut app_in_tx, app_in_rx) = mpsc::unbounded::<(ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>();
        let (app_out_tx, _app_out_rx) = mpsc::unbounded::<(HoprPseudonym, ApplicationDataIn)>();

        let encoder = TestEncoder { next_hop };
        let decoder = TestDecoder::returning_final(next_hop);

        let processes = run_packet_pipeline(
            packet_key,
            (wire_out_tx, wire_in_rx),
            (encoder, decoder),
            NoOpTicketProcessor,
            ticket_evt_tx,
            PacketPipelineConfig {
                output_concurrency: Some(1),
                input_concurrency: Some(1),
                ..Default::default()
            },
            (app_out_tx, app_in_rx),
            Default::default(),
        );

        // Send an application packet through the outgoing pipeline
        let app_data = ApplicationData::new(42u64, b"outgoing test payload").expect("valid app data");
        let routing = ResolvedTransportRouting::Forward {
            pseudonym: SimplePseudonym([0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, 0x11, 0x22]),
            forward_path: hopr_api::types::internal::prelude::ValidatedPath::direct(
                next_hop,
                hopr_api::types::primitive::prelude::Address::default(),
            ),
            return_paths: vec![],
        };
        app_in_tx
            .send((routing, ApplicationDataOut::with_no_packet_info(app_data)))
            .await
            .context("sending app data to outgoing pipeline")?;
        drop(app_in_tx);

        // Verify the encoded packet arrives on the wire
        let result = tokio::time::timeout(Duration::from_secs(2), wire_out_rx.next()).await;
        let (peer_id, _data) = result
            .context("should receive wire packet within timeout")?
            .context("wire output should have data")?;
        assert_eq!(peer_id, PeerId::from(next_hop));

        processes.abort_all();
        Ok(())
    }
}
