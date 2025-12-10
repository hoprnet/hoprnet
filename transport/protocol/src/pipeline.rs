use futures::{SinkExt, StreamExt};
use futures_time::{future::FutureExt as TimeExt, stream::StreamExt as TimeStreamExt};
use hopr_async_runtime::{AbortableList, spawn_as_abortable};
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_network_types::{
    prelude::*,
    timeout::{SinkTimeoutError, TimeoutSinkExt, TimeoutStreamExt},
};
use hopr_primitive_types::prelude::Address;
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

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_PACKET_COUNT:  hopr_metrics::MultiCounter =  hopr_metrics::MultiCounter::new(
        "hopr_packets_count",
        "Number of processed packets of different types (sent, received, forwarded)",
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
) where
    AppOut: futures::Stream<Item = (ResolvedTransportRouting, ApplicationDataOut)> + Send + 'static,
    E: PacketEncoder + Send + 'static,
    WOut: futures::Sink<(PeerId, Box<[u8]>), Error = SinkTimeoutError<WOutErr>> + Clone + Unpin + Send + 'static,
    WOutErr: std::error::Error,
{
    let res = app_outgoing
        .then_concurrent(|(routing, data)| {
            let encoder = encoder.clone();
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
                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_PACKET_COUNT.increment(&["sent"]);

                        tracing::trace!(peer = packet.next_hop.to_peerid_str(), "protocol message out");
                        Some((packet.next_hop.into(), packet.data))
                    }
                    Err(error) => {
                        tracing::error!(%error, "packet could not be wrapped for sending");
                        None
                    }
                }
            }
        })
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
async fn start_incoming_packet_pipeline<WIn, WOut, D, T, TEvt, AckIn, AckOut, AppIn, AppInErr>(
    (wire_outgoing, wire_incoming): (WOut, WIn),
    decoder: std::sync::Arc<D>,
    ticket_proc: std::sync::Arc<T>,
    ticket_events: TEvt,
    (ack_outgoing, ack_incoming): (AckOut, AckIn),
    app_incoming: AppIn,
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

            tracing::trace!("protocol message in");

            async move {
                match decoder.decode(peer, data)
                    .timeout(futures_time::time::Duration::from(PACKET_DECODING_TIMEOUT))
                    .await {
                    Ok(Ok(packet)) => {
                        tracing::trace!(?packet, "successfully decoded incoming packet");
                        Some(packet)
                    },
                    Ok(Err(IncomingPacketError::Undecodable(error))) => {
                        // Do not send an ack back if the packet could not be decoded at all
                        //
                        // Potentially adversarial behavior
                        tracing::trace!(%error, "not sending ack back on undecodable packet - possible adversarial behavior");
                        None
                    },
                    Ok(Err(IncomingPacketError::ProcessingError(sender, error))) => {
                        tracing::error!(%error, "failed to process the decoded packet");
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
                        None
                    }
                    Err(_) => {
                        // If we cannot decode the packet within the time limit, just drop it
                        tracing::error!("dropped incoming packet: failed to decode packet within {:?}", PACKET_DECODING_TIMEOUT);
                        None
                    }
                }
            }.instrument(tracing::debug_span!("incoming_packet_decode", %peer))
        })
        .filter_map(futures::future::ready)
        .then_concurrent(move |packet| {
            let ticket_proc = ticket_proc_success.clone();
            let mut wire_outgoing = wire_outgoing.clone();
            let mut ack_incoming = ack_incoming.clone();
            let mut ack_outgoing_success = ack_outgoing_success.clone();
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

                        #[cfg(all(feature = "prometheus", not(test)))]
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

                        wire_outgoing
                            .send((next_hop.into(), data))
                            .await
                            .unwrap_or_else(|error| {
                                tracing::error!(%error, "failed to forward a packet to the transport layer");
                            });

                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_PACKET_COUNT.increment(&["forwarded"]);

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
        .then(move |(destination, maybe_ack_key)|{
            let packet_key = packet_key.clone();
            async move {
                 // Sign acknowledgement with the given half-key or generate a signed random one
                 let ack = hopr_parallelize::cpu::spawn_blocking(move || {
                     maybe_ack_key
                         .map(|ack_key| VerifiedAcknowledgement::new(ack_key, &packet_key))
                         .unwrap_or_else(|| VerifiedAcknowledgement::random(&packet_key))
                 })
                 .await;
                (destination, ack)
            }
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
            async move {
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

/// Run all processes responsible for handling the msg and acknowledgment protocols.
///
/// The pipeline does not handle the mixing itself, that needs to be injected as a separate process
/// overlay on top of the `wire_msg` Stream or Sink.
#[tracing::instrument(skip_all, level = "trace", fields(me = packet_key.public().to_peerid_str()))]
pub fn run_packet_pipeline<WIn, WOut, C, D, T, TEvt, AppOut, AppIn>(
    packet_key: OffchainKeypair,
    wire_msg: (WOut, WIn),
    codec: (C, D),
    ticket_proc: T,
    ticket_events: TEvt,
    ack_config: AcknowledgementPipelineConfig,
    api: (AppOut, AppIn),
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
    AppIn: futures::Stream<Item = (ResolvedTransportRouting, ApplicationDataOut)> + Send + 'static,
{
    let mut processes = AbortableList::default();

    #[cfg(all(feature = "prometheus", not(test)))]
    {
        // Initialize the lazy statics here
        lazy_static::initialize(&METRIC_PACKET_COUNT);
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

    processes.insert(
        PacketPipelineProcesses::MsgOut,
        spawn_as_abortable!(
            start_outgoing_packet_pipeline(app_in, encoder.clone(), wire_out.clone(),).in_current_span()
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
            )
            .in_current_span()
        ),
    );

    processes.insert(
        PacketPipelineProcesses::AckOut,
        spawn_as_abortable!(
            start_outgoing_ack_pipeline(outgoing_ack_rx, encoder, ack_config, packet_key.clone(), wire_out,)
                .in_current_span()
        ),
    );

    processes.insert(
        PacketPipelineProcesses::AckIn,
        spawn_as_abortable!(start_incoming_ack_pipeline(incoming_ack_rx, ticket_events, ticket_proc).in_current_span()),
    );

    processes
}
