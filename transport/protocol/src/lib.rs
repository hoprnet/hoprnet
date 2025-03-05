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
pub use timer::execute_on_tick;

pub use msg::processor::DEFAULT_PRICE_PER_PACKET;

use core_path::path::TransportPath;
use futures::{SinkExt, StreamExt};
use libp2p::PeerId;
use msg::processor::{PacketSendFinalizer, PacketUnwrapping, PacketWrapping};
use rust_stream_ext_concurrent::then_concurrent::StreamThenConcurrentExt;
use std::collections::HashMap;
use tracing::{error, trace};

use hopr_async_runtime::prelude::{sleep, spawn};
use hopr_crypto_types::prelude::*;
use hopr_db_api::protocol::HoprDbProtocolOperations;
use hopr_internal_types::protocol::{Acknowledgement, ApplicationData};

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::{MultiCounter, SimpleCounter, SimpleGauge};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    // acknowledgement
    static ref METRIC_RECEIVED_ACKS: MultiCounter = MultiCounter::new(
        "hopr_received_ack_count",
        "Number of received acknowledgements",
        &["valid"]
    )
    .unwrap();
    static ref METRIC_SENT_ACKS: SimpleCounter =
        SimpleCounter::new("hopr_sent_acks_count", "Number of sent message acknowledgements").unwrap();
    static ref METRIC_TICKETS_COUNT: MultiCounter =
        MultiCounter::new("hopr_tickets_count", "Number of winning tickets", &["type"]).unwrap();
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
    // mixer
    static ref METRIC_QUEUE_SIZE: SimpleGauge =
        SimpleGauge::new("hopr_mixer_queue_size", "Current mixer queue size").unwrap();
    static ref METRIC_MIXER_AVERAGE_DELAY: SimpleGauge = SimpleGauge::new(
        "hopr_mixer_average_packet_delay",
        "Average mixer packet delay averaged over a packet window"
    )
    .unwrap();
}

const ENV_MIXER_PACKET_MAX_DELAY_MILLIS: &str = "HOPR_INTERNAL_MIXER_PACKET_MAX_DELAY_MILLIS";

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ProtocolProcesses {
    AckIn,
    AckOut,
    MsgIn,
    MsgOut,
    BloomPersist,
}

#[allow(clippy::too_many_arguments)]
pub async fn run_msg_ack_protocol<Db>(
    cfg: msg::processor::PacketInteractionConfig,
    db: Db,
    me: &OffchainKeypair,
    me_onchain: &ChainKeypair,
    bloom_filter_persistent_path: Option<String>,
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

    #[cfg(all(feature = "prometheus", not(test)))]
    {
        // Initialize the lazy statics here
        lazy_static::initialize(&METRIC_RECEIVED_ACKS);
        lazy_static::initialize(&METRIC_SENT_ACKS);
        lazy_static::initialize(&METRIC_TICKETS_COUNT);
        lazy_static::initialize(&METRIC_PACKET_COUNT);
        lazy_static::initialize(&METRIC_PACKET_COUNT_PER_PEER);
        lazy_static::initialize(&METRIC_REPLAYED_PACKET_COUNT);
        lazy_static::initialize(&METRIC_REJECTED_TICKETS_COUNT);
        lazy_static::initialize(&METRIC_QUEUE_SIZE);
        lazy_static::initialize(&METRIC_MIXER_AVERAGE_DELAY);
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

    let ack_processor_read = ack::processor::AcknowledgementProcessor::new(db.clone(), me_onchain);
    let ack_processor_write = ack_processor_read.clone();
    let msg_processor_read = msg::processor::PacketProcessor::new(db.clone(), tbf, cfg);
    let msg_processor_write = msg_processor_read.clone();

    processes.insert(
        ProtocolProcesses::AckIn,
        spawn(async move {
            let _neverending = wire_ack
                .1
                .for_each_concurrent(None, move |(peer, ack)| {
                    let ack_processor = ack_processor_read.clone();

                    async move {
                        let _ack_result = ack_processor.recv(&peer, ack).await;
                        #[cfg(all(feature = "prometheus", not(test)))]
                        match &_ack_result {
                            Ok(hopr_db_api::prelude::AckResult::Sender(_)) => {
                                METRIC_RECEIVED_ACKS.increment(&["true"]);
                            }
                            Ok(hopr_db_api::prelude::AckResult::RelayerWinning(_)) => {
                                METRIC_RECEIVED_ACKS.increment(&["true"]);
                                METRIC_TICKETS_COUNT.increment(&["winning"]);
                            }
                            Ok(hopr_db_api::prelude::AckResult::RelayerLosing) => {
                                METRIC_RECEIVED_ACKS.increment(&["true"]);
                                METRIC_TICKETS_COUNT.increment(&["losing"]);
                            }
                            Err(_) => {
                                METRIC_RECEIVED_ACKS.increment(&["false"]);
                            }
                        }
                    }
                })
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

                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_SENT_ACKS.increment();

                    async move { (peer, ack_processor.send(&peer, ack).await) }
                })
                .map(Ok)
                .forward(wire_ack.0)
                .await;
        }),
    );

    let msg_to_send_tx = wire_msg.0.clone();

    let mixer_cfg = msg::mixer::MixerConfig::new(
        0,
        std::env::var(ENV_MIXER_PACKET_MAX_DELAY_MILLIS)
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or_else(|| msg::mixer::MixerConfig::default().delay_range().1),
    );

    processes.insert(
        ProtocolProcesses::MsgOut,
        spawn(async move {
            let _neverending = api
                .1
                .then_concurrent(|(data, path, finalizer)| {
                    let msg_processor = msg_processor_write.clone();

                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_QUEUE_SIZE.increment(1.0f64);

                    async move {
                        match PacketWrapping::send(&msg_processor, data, path).await {
                            Ok(v) => {
                                #[cfg(all(feature = "prometheus", not(test)))]
                                {
                                    METRIC_PACKET_COUNT_PER_PEER.increment(&["out", &v.0.to_string()]);
                                    METRIC_PACKET_COUNT.increment(&["sent"]);
                                }
                                finalizer.finalize(Ok(()));
                                Some(v)
                            }
                            Err(e) => {
                                #[cfg(all(feature = "prometheus", not(test)))]
                                METRIC_QUEUE_SIZE.decrement(1.0f64);

                                finalizer.finalize(Err(e));
                                None
                            }
                        }
                    }
                })
                .filter_map(|v| async move { v })
                // delay purposefully isolated into a separate concurrent task
                .then_concurrent(|v| async move {
                    let random_delay = mixer_cfg.random_delay();
                    trace!(delay_in_ms = random_delay.as_millis(), "Created random mixer delay",);

                    sleep(random_delay).await;

                    #[cfg(all(feature = "prometheus", not(test)))]
                    {
                        METRIC_QUEUE_SIZE.decrement(1.0f64);

                        let weight = 1.0f64 / mixer_cfg.metric_delay_window as f64;
                        METRIC_MIXER_AVERAGE_DELAY.set(
                            (weight * random_delay.as_millis() as f64)
                                + ((1.0f64 - weight) * METRIC_MIXER_AVERAGE_DELAY.get()),
                        );
                    }

                    v
                })
                .map(Ok)
                .forward(wire_msg.0)
                .await;
        }),
    );

    let me = me.clone();
    processes.insert(
        ProtocolProcesses::MsgIn,
        spawn(async move {
            let _neverending = wire_msg
                .1
                .then_concurrent(move |(peer, data)| {
                    let msg_processor = msg_processor_read.clone();

                    async move {
                        let now = std::time::Instant::now();
                        let r = msg_processor.recv(&peer, data).await.map_err(|e| (peer, e));
                        let elapsed = now.elapsed();
                        if elapsed.as_millis() > 150 {
                            tracing::warn!("msg_processor.recv took {}ms", elapsed.as_millis());
                        }
                        r
                    }
                })
                .filter_map(move |v| {
                    let mut internal_ack_send = internal_ack_send.clone();
                    let mut msg_to_send_tx = msg_to_send_tx.clone();
                    let me = me.clone();

                    async move {
                        match v {
                            Ok(v) => match v {
                                msg::processor::RecvOperation::Receive { data, ack } => {
                                    #[cfg(all(feature = "prometheus", not(test)))]
                                    {
                                        METRIC_PACKET_COUNT_PER_PEER.increment(&["in", &ack.peer.to_string()]);
                                        METRIC_PACKET_COUNT.increment(&["received"]);
                                    }

                                    let now = std::time::Instant::now();
                                    internal_ack_send.send((ack.peer, ack.ack)).await.unwrap_or_else(|e| {
                                        error!(error = %e, "Failed to forward an acknowledgement to the transport layer");
                                    });
                                    let elapsed = now.elapsed();
                                    if elapsed.as_millis() > 150 {
                                        tracing::warn!("internal_ack.send took {}ms", elapsed.as_millis());
                                    }

                                    Some(data)
                                }
                                msg::processor::RecvOperation::Forward { msg, ack } => {
                                    #[cfg(all(feature = "prometheus", not(test)))]
                                    {
                                        METRIC_PACKET_COUNT_PER_PEER.increment(&["in", &ack.peer.to_string()]);
                                        METRIC_PACKET_COUNT_PER_PEER.increment(&["out", &msg.peer.to_string()]);
                                        METRIC_PACKET_COUNT.increment(&["forwarded"]);
                                    }

                                    msg_to_send_tx.send((msg.peer, msg.data)).await.unwrap_or_else(|_e| {
                                        error!("Failed to forward a message to the transport layer");
                                    });
                                    internal_ack_send.send((ack.peer, ack.ack)).await.unwrap_or_else(|e| {
                                        error!(error = %e, "Failed to forward an acknowledgement to the transport layer");
                                    });
                                    None
                                }
                            },
                            Err((peer, e)) => {
                                #[cfg(all(feature = "prometheus", not(test)))]
                                match e {
                                    hopr_crypto_packet::errors::PacketError::TagReplay => {
                                        METRIC_REPLAYED_PACKET_COUNT.increment();
                                    },
                                    hopr_crypto_packet::errors::PacketError::TicketValidation(_) => {
                                        METRIC_REJECTED_TICKETS_COUNT.increment();
                                    },
                                    _ => {}
                                }

                                error!(peer = %peer, error = %e, "Failed to process the received message");
                                // send random signed acknowledgement to give feedback to the sender
                                internal_ack_send
                                    .send((
                                        peer,
                                        Acknowledgement::random(&me),
                                    ))
                                    .await
                                    .unwrap_or_else(|e| {
                                        error!(error = %e, "Failed to forward an acknowledgement for a failed packet recv to the transport layer");
                                    });

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
