use std::pin::Pin;

use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::future::poll_fn;
use futures::{stream::Stream, StreamExt};
use libp2p_identity::PeerId;
use rust_stream_ext_concurrent::then_concurrent::StreamThenConcurrentExt;
use tracing::{error, trace, warn};

use hopr_crypto_packet::errors::PacketError::{Retry, TransportError};
use hopr_crypto_packet::errors::Result;
use hopr_crypto_types::prelude::*;
use hopr_db_api::prelude::HoprDbProtocolOperations;
pub use hopr_db_api::protocol::AckResult;
use hopr_internal_types::prelude::*;

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::{MultiCounter, SimpleCounter};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
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
}

// Default sizes of the acknowledgement queues
pub const ACK_TX_QUEUE_SIZE: usize = 2048;
pub const ACK_RX_QUEUE_SIZE: usize = 2048;

#[derive(Debug)]
pub enum AckToProcess {
    ToReceive(PeerId, Acknowledgement),
    ToSend(PeerId, Acknowledgement),
}

#[allow(clippy::large_enum_variant)] // TODO: Uses too large objects
#[derive(Debug)]
pub enum AckProcessed {
    Receive(PeerId, Result<AckResult>),
    Send(PeerId, Acknowledgement),
}

/// Implements protocol acknowledgement logic for acknowledgements
#[derive(Clone)]
pub struct AcknowledgementProcessor<Db: HoprDbProtocolOperations> {
    db: Db,
    chain_key: ChainKeypair,
}

impl<Db: HoprDbProtocolOperations> AcknowledgementProcessor<Db> {
    pub fn new(db: Db, chain_key: &ChainKeypair) -> Self {
        Self {
            db,
            chain_key: chain_key.clone(),
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn handle_acknowledgement(
        &self,
        ack: Acknowledgement,
    ) -> std::result::Result<AckResult, hopr_db_api::errors::DbError> {
        self.db.handle_acknowledgement(ack, self.chain_key.clone()).await
    }
}

/// External API for feeding Acknowledgement actions into the Acknowledgement processor
/// processing the elements independently in the background.
#[derive(Debug, Clone)]
pub struct AcknowledgementActions {
    pub queue: Sender<AckToProcess>,
}

impl AcknowledgementActions {
    /// Pushes the acknowledgement received from the transport layer into processing.
    pub fn receive_acknowledgement(&mut self, source: PeerId, acknowledgement: Acknowledgement) -> Result<()> {
        self.process(AckToProcess::ToReceive(source, acknowledgement))
    }

    /// Pushes a new outgoing acknowledgement into the processing.
    pub fn send_acknowledgement(&mut self, destination: PeerId, acknowledgement: Acknowledgement) -> Result<()> {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_SENT_ACKS.increment();

        self.process(AckToProcess::ToSend(destination, acknowledgement))
    }

    fn process(&mut self, event: AckToProcess) -> Result<()> {
        self.queue.try_send(event).map_err(|e| {
            if e.is_full() {
                Retry
            } else if e.is_disconnected() {
                TransportError("queue is closed".to_string())
            } else {
                TransportError(format!("Unknown error: {}", e))
            }
        })
    }
}

/// Sets up processing of acknowledgement interactions and returns relevant read and write mechanism.
///
/// When a new acknowledgement is delivered from the transport, the `receive_acknowledgement`
/// method is used to push it into the processing queue of incoming acknowledgements.
///
/// Acknowledgements issued by this node are generated using the `send_acknowledgement` method.
///
/// The result of processing the acknowledgements can be extracted as a stream.
pub struct AcknowledgementInteraction {
    ack_event_queue: (Sender<AckToProcess>, Receiver<AckProcessed>),
}

impl AcknowledgementInteraction {
    /// Creates a new instance given the DB and our public key used to verify the acknowledgements.
    pub fn new<Db: HoprDbProtocolOperations + Send + Sync + Clone + 'static>(db: Db, chain_key: &ChainKeypair) -> Self {
        let (processing_in_tx, processing_in_rx) = channel::<AckToProcess>(ACK_RX_QUEUE_SIZE + ACK_TX_QUEUE_SIZE);
        let (processing_out_tx, processing_out_rx) = channel::<AckProcessed>(ACK_RX_QUEUE_SIZE + ACK_TX_QUEUE_SIZE);

        let processor = AcknowledgementProcessor::new(db, chain_key);

        let processing_stream = processing_in_rx.then_concurrent(move |event| {
            let processor = processor.clone();
            let mut processed_tx = processing_out_tx.clone();

            async move {
                let processed: Option<AckProcessed> = match event {
                    AckToProcess::ToReceive(peer, mut ack) => {
                        if let Ok(remote_pk) = OffchainPublicKey::try_from(peer) {
                            trace!("validating incoming acknowledgement from {}", peer);
                            if ack.validate(&remote_pk) {
                                match processor.handle_acknowledgement(ack).await {
                                    Ok(reply) => {
                                        match &reply {
                                            AckResult::Sender(_) => {
                                                #[cfg(all(feature = "prometheus", not(test)))]
                                                METRIC_RECEIVED_ACKS.increment(&["true"]);
                                            }
                                            AckResult::RelayerWinning(_) => {
                                                #[cfg(all(feature = "prometheus", not(test)))]
                                                {
                                                    METRIC_RECEIVED_ACKS.increment(&["true"]);
                                                    METRIC_TICKETS_COUNT.increment(&["winning"]);
                                                }
                                            }
                                            AckResult::RelayerLosing => {
                                                #[cfg(all(feature = "prometheus", not(test)))]
                                                {
                                                    METRIC_RECEIVED_ACKS.increment(&["true"]);
                                                    METRIC_TICKETS_COUNT.increment(&["losing"]);
                                                }
                                            }
                                        }

                                        Some(AckProcessed::Receive(peer, Ok(reply)))
                                    }
                                    Err(e) => {
                                        error!(
                                            "Encountered error while handling acknowledgement from peer '{}': {}",
                                            &peer, e
                                        );

                                        #[cfg(all(feature = "prometheus", not(test)))]
                                        METRIC_RECEIVED_ACKS.increment(&["false"]);

                                        None
                                    }
                                }
                            } else {
                                error!("failed to verify signature on acknowledgement from peer {}", peer);
                                None
                            }
                        } else {
                            error!("invalid remote peer id {}", peer);
                            None
                        }
                    }
                    AckToProcess::ToSend(peer, ack) => Some(AckProcessed::Send(peer, ack)),
                };

                if let Some(event) = processed {
                    match poll_fn(|cx| Pin::new(&mut processed_tx).poll_ready(cx)).await {
                        Ok(_) => match processed_tx.start_send(event) {
                            Ok(_) => {}
                            Err(e) => error!("Failed to pass a processed ack message: {}", e),
                        },
                        Err(e) => {
                            warn!("The receiver for processed ack no longer exists: {}", e);
                        }
                    };
                }
            }
        });

        // NOTE: This spawned task does not need to be explicitly canceled, since it will
        // be automatically dropped when the event sender object is dropped.
        async_std::task::spawn(async move {
            processing_stream.map(Ok).forward(futures::sink::drain()).await.unwrap();
        });

        Self {
            ack_event_queue: (processing_in_tx, processing_out_rx),
        }
    }

    pub fn writer(&self) -> AcknowledgementActions {
        AcknowledgementActions {
            queue: self.ack_event_queue.0.clone(),
        }
    }
}

impl Stream for AcknowledgementInteraction {
    type Item = AckProcessed;

    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        use futures_lite::stream::StreamExt;
        Pin::new(self).ack_event_queue.1.poll_next(cx)
    }
}
