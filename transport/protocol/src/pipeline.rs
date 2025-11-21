use futures::{SinkExt, StreamExt};
use futures_time::future::FutureExt as TimeExt;
use hopr_async_runtime::spawn_as_abortable;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_network_types::{prelude::*, timeout::TimeoutSinkExt};
use hopr_protocol_app::prelude::*;
use hopr_protocol_hopr::prelude::*;
use rust_stream_ext_concurrent::then_concurrent::StreamThenConcurrentExt;
use tracing::Instrument;

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
#[derive(Debug, Clone)]
pub enum TicketEvent {
    /// A winning ticket was received.
    WinningTicket(Box<RedeemableTicket>),
    /// A ticket has been rejected.
    RejectedTicket(Box<Ticket>),
}

/// Performs encoding of outgoing Application protocol packets into HOPR protocol outgoing packets.
async fn start_outgoing_packet_pipeline<AppOut, E, WOut>(
    app_outgoing: AppOut,
    encoder: std::sync::Arc<E>,
    wire_outgoing: WOut,
) where
    AppOut: futures::Stream<Item = (ResolvedTransportRouting, ApplicationDataOut)> + Send + 'static,
    E: PacketEncoder + Send + 'static,
    WOut: futures::Sink<(PeerId, Box<[u8]>)> + Clone + Unpin + Send + 'static,
    WOut::Error: std::fmt::Display,
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

                        tracing::trace!(peer = %packet.next_hop, "protocol message out");
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
        .forward(wire_outgoing)
        .instrument(tracing::trace_span!("msg protocol processing - egress"))
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
#[allow(clippy::too_many_arguments)] // Allowed on internal functions
async fn start_incoming_packet_pipeline<WIn, WOut, D, T, TEvt, AckIn, AckOut, AppIn>(
    wire_incoming: WIn,
    decoder: std::sync::Arc<D>,
    ticket_proc: std::sync::Arc<T>,
    ticket_events: TEvt,
    ack_outgoing: AckOut,
    wire_outgoing: WOut,
    ack_incoming: AckIn,
    app_incoming: AppIn,
) where
    WIn: futures::Stream<Item = (PeerId, Box<[u8]>)> + Send + 'static,
    WOut: futures::Sink<(PeerId, Box<[u8]>)> + Clone + Unpin + Send + 'static,
    WOut::Error: std::fmt::Display,
    D: PacketDecoder + Send + 'static,
    T: UnacknowledgedTicketProcessor + Send + 'static,
    TEvt: futures::Sink<TicketEvent> + Clone + Unpin + Send + 'static,
    TEvt::Error: std::fmt::Display,
    AckIn: futures::Sink<(OffchainPublicKey, Acknowledgement)> + Send + Unpin + Clone + 'static,
    AckIn::Error: std::fmt::Display,
    AckOut: futures::Sink<(OffchainPublicKey, Option<HalfKey>)> + Send + Unpin + Clone + 'static,
    AckOut::Error: std::fmt::Display,
    AppIn: futures::Sink<(HoprPseudonym, ApplicationDataIn)> + Send + 'static,
    AppIn::Error: std::fmt::Display,
{
    // Create a cache for a CPU-intensive conversion PeerId -> OffchainPublicKey

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
                        tracing::trace!(%peer, "successfully decoded incoming packet");
                        Some(packet)
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
                            .send(TicketEvent::RejectedTicket(error.ticket))
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
                        tracing::error!(%peer, "dropped incoming packet: failed to decode packet within {:?}", PACKET_DECODING_TIMEOUT);
                        None
                    }
                }
            }
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
                        let IncomingAcknowledgementPacket { previous_hop, received_ack: ack, .. } = *ack;
                        tracing::trace!(%previous_hop , "acknowledging ticket using received ack");
                        ack_incoming
                            .send((previous_hop, ack))
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

                        if let Err(error) = ticket_proc.insert_unacknowledged_ticket(ack_challenge, received_ticket).await {
                            tracing::error!(%previous_hop, %next_hop, %error, "failed to insert unacknowledged ticket into the ticket processor");
                            return None;
                        }

                        // First, relay the packet to the next hop
                        tracing::trace!(%previous_hop, %next_hop, "forwarding packet to the next hop");

                        wire_outgoing
                            .send((next_hop.into(), data))
                            .await
                            .unwrap_or_else(|error| {
                                tracing::error!(%error, "failed to forward a packet to the transport layer");
                            });

                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_PACKET_COUNT.increment(&["forwarded"]);

                        // Send acknowledgement back
                        tracing::trace!(%previous_hop, "acknowledging forwarded packet back");
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
        .forward(app_incoming)
        .instrument(tracing::trace_span!("msg protocol processing - ingress"))
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
    packet_key: OffchainKeypair,
    wire_outgoing: WOut,
) where
    AckOut: futures::Stream<Item = (OffchainPublicKey, Option<HalfKey>)> + Send + 'static,
    E: PacketEncoder + Send + 'static,
    WOut: futures::Sink<(PeerId, Box<[u8]>)> + Clone + Unpin + Send + 'static,
    WOut::Error: std::fmt::Display,
{
    ack_outgoing
        .for_each_concurrent(
            NUM_CONCURRENT_ACK_OUT_PROCESSING,
            move |(destination, maybe_ack_key)| {
                let packet_key = packet_key.clone();
                let encoder = encoder.clone();
                let mut wire_outgoing = wire_outgoing.clone();

                async move {
                    // Sign acknowledgement with the given half-key or generate a signed random one
                    let ack = hopr_parallelize::cpu::spawn_blocking(move || {
                        maybe_ack_key
                            .map(|ack_key| VerifiedAcknowledgement::new(ack_key, &packet_key))
                            .unwrap_or_else(|| VerifiedAcknowledgement::random(&packet_key))
                    })
                        .await;

                    match encoder.encode_acknowledgement(ack, &destination).await {
                        Ok(ack_packet) => {
                            wire_outgoing
                                .send((ack_packet.next_hop.into(), ack_packet.data))
                                .await
                                .unwrap_or_else(|error| {
                                    tracing::error!(%error, "failed to forward an acknowledgement to the transport layer");
                                });
                        }
                        Err(error) => tracing::error!(%error, "failed to create ack packet"),
                    }
                }
            }
        ).await;

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
    AckIn: futures::Stream<Item = (OffchainPublicKey, Acknowledgement)> + Send + 'static,
    T: UnacknowledgedTicketProcessor + Sync + Send + 'static,
    TEvt: futures::Sink<TicketEvent> + Clone + Unpin + Send + 'static,
    TEvt::Error: std::fmt::Display,
{
    ack_incoming
        .for_each_concurrent(NUM_CONCURRENT_TICKET_ACK_PROCESSING, move |(peer, ack)| {
            let ticket_proc = ticket_proc.clone();
            let mut ticket_evt = ticket_events.clone();
            async move {
                match ticket_proc.acknowledge_ticket(peer, ack).await {
                    Ok(ResolvedAcknowledgement::RelayingWin(redeemable_ticket)) => {
                        ticket_evt
                            .send(TicketEvent::WinningTicket(redeemable_ticket))
                            .await
                            .unwrap_or_else(|error| {
                                tracing::error!(%error, "failed to notify winning ticket");
                            });
                    }
                    Ok(ResolvedAcknowledgement::RelayingLoss(_)) => {
                        // Losing tickets are not getting accounted for anywhere.
                        // Possibly in some metric?
                    }
                    Err(error) => {
                        tracing::error!(%error, "failed to acknowledge ticket");
                    }
                }
            }
        })
        .await;

    tracing::warn!(
        task = "transport (protocol - ticket acknowledgement)",
        "long-running background task finished"
    );
}

/// Run all processes responsible for handling the msg and acknowledgment protocols.
///
/// The pipeline does not handle the mixing itself, that needs to be injected as a separate process
/// overlay on top of the `wire_msg` Stream or Sink.
pub fn run_packet_pipeline<WIn, WOut, C, D, T, TEvt, AppOut, AppIn>(
    packet_key: OffchainKeypair,
    wire_msg: (WOut, WIn),
    codec: (C, D),
    ticket_proc: T,
    ticket_events: TEvt,
    api: (AppOut, AppIn),
) -> std::collections::HashMap<PacketPipelineProcesses, hopr_async_runtime::AbortHandle>
where
    WOut: futures::Sink<(PeerId, Box<[u8]>)> + Clone + Unpin + Send + 'static,
    WOut::Error: std::fmt::Display,
    WIn: futures::Stream<Item = (PeerId, Box<[u8]>)> + Send + 'static,
    C: PacketEncoder + Sync + Send + 'static,
    D: PacketDecoder + Sync + Send + 'static,
    T: UnacknowledgedTicketProcessor + Sync + Send + 'static,
    TEvt: futures::Sink<TicketEvent> + Clone + Unpin + Send + 'static,
    TEvt::Error: std::fmt::Display,
    AppOut: futures::Sink<(HoprPseudonym, ApplicationDataIn)> + Send + 'static,
    AppOut::Error: std::fmt::Display,
    AppIn: futures::Stream<Item = (ResolvedTransportRouting, ApplicationDataOut)> + Send + 'static,
{
    let mut processes = std::collections::HashMap::new();

    #[cfg(all(feature = "prometheus", not(test)))]
    {
        // Initialize the lazy statics here
        lazy_static::initialize(&METRIC_PACKET_COUNT);
    }

    let (outgoing_ack_tx, outgoing_ack_rx) =
        futures::channel::mpsc::channel::<(OffchainPublicKey, Option<HalfKey>)>(ACK_OUT_BUFFER_SIZE);

    let (incoming_ack_tx, incoming_ack_rx) =
        futures::channel::mpsc::channel::<(OffchainPublicKey, Acknowledgement)>(TICKET_ACK_BUFFER_SIZE);

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
        spawn_as_abortable!(start_outgoing_packet_pipeline(
            app_in,
            encoder.clone(),
            wire_out.clone(),
        )),
    );

    processes.insert(
        PacketPipelineProcesses::MsgIn,
        spawn_as_abortable!(start_incoming_packet_pipeline(
            wire_in,
            decoder,
            ticket_proc.clone(),
            ticket_events.clone(),
            outgoing_ack_tx,
            wire_out.clone(),
            incoming_ack_tx,
            app_out,
        )),
    );

    processes.insert(
        PacketPipelineProcesses::AckOut,
        spawn_as_abortable!(start_outgoing_ack_pipeline(
            outgoing_ack_rx,
            encoder,
            packet_key.clone(),
            wire_out,
        )),
    );

    processes.insert(
        PacketPipelineProcesses::AckIn,
        spawn_as_abortable!(start_incoming_ack_pipeline(incoming_ack_rx, ticket_events, ticket_proc)),
    );

    processes
}
