use async_lock::RwLock;
use rust_stream_ext_concurrent::then_concurrent::StreamThenConcurrentExt;

use core_crypto::keypairs::ChainKeypair;
use core_crypto::types::{HalfKeyChallenge, OffchainPublicKey};
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_packet::errors::PacketError::{AcknowledgementValidation, MissingDomainSeparator, Retry, TransportError};
use core_packet::errors::Result;
use core_types::acknowledgement::{AcknowledgedTicket, Acknowledgement, PendingAcknowledgement};
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::future::poll_fn;
use futures::{stream::Stream, StreamExt};
use libp2p_identity::PeerId;
use log::{debug, error, warn};
use std::pin::Pin;
use std::sync::Arc;
use utils_types::traits::{PeerIdLike, ToHex};

use async_std::task::spawn;

#[cfg(all(feature = "prometheus", not(test)))]
use utils_metrics::metrics::{MultiCounter, SimpleCounter};

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
    static ref METRIC_WINNING_TICKETS_COUNT: SimpleCounter =
        SimpleCounter::new("hopr_winning_tickets_count", "Number of winning tickets").unwrap();
    static ref METRIC_LOSING_TICKETS_COUNT: SimpleCounter =
        SimpleCounter::new("hopr_losing_tickets_count", "Number of losing tickets").unwrap();
}

// Default sizes of the acknowledgement queues
pub const ACK_TX_QUEUE_SIZE: usize = 2048;
pub const ACK_RX_QUEUE_SIZE: usize = 2048;

#[derive(Debug)]
pub enum Reply {
    Sender(HalfKeyChallenge),
    RelayerWinning(AcknowledgedTicket),
    RelayerLosing,
}

#[derive(Debug)]
pub enum AckToProcess {
    ToReceive(PeerId, Acknowledgement),
    ToSend(PeerId, Acknowledgement),
}

#[derive(Debug)]
pub enum AckProcessed {
    Receive(PeerId, Result<Reply>),
    Send(PeerId, Acknowledgement),
}

/// Implements protocol acknowledgement logic for acknowledgements
pub struct AcknowledgementProcessor<Db: HoprCoreEthereumDbActions> {
    db: Arc<RwLock<Db>>,
    chain_key: ChainKeypair,
}

impl<Db: HoprCoreEthereumDbActions> Clone for AcknowledgementProcessor<Db> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            chain_key: self.chain_key.clone(),
        }
    }
}

impl<Db: HoprCoreEthereumDbActions> AcknowledgementProcessor<Db> {
    pub fn new(db: Arc<RwLock<Db>>, chain_key: &ChainKeypair) -> Self {
        Self {
            db,
            chain_key: chain_key.clone(),
        }
    }

    pub async fn handle_acknowledgement(&mut self, ack: Acknowledgement) -> Result<Reply> {
        /*
            There are three cases:
            1. There is an unacknowledged ticket and we are
                awaiting a half key.
            2. We were the creator of the packet, hence we
                do not wait for any half key
            3. The acknowledgement is unexpected and stems from
                a protocol bug or an attacker
        */

        let pending = self
            .db
            .read()
            .await
            .get_pending_acknowledgement(&ack.ack_challenge())
            .await?
            .ok_or_else(|| {
                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_RECEIVED_ACKS.increment(&["false"]);

                AcknowledgementValidation(format!(
                    "received unexpected acknowledgement for half key challenge {} - half key {}",
                    ack.ack_challenge().to_hex(),
                    ack.ack_key_share.to_hex()
                ))
            })?;

        match pending {
            PendingAcknowledgement::WaitingAsSender => {
                // No pending ticket, nothing to do.
                debug!("received acknowledgement as sender: first relayer has processed the packet.");

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_RECEIVED_ACKS.increment(&["false"]);

                Ok(Reply::Sender(ack.ack_challenge()))
            }

            PendingAcknowledgement::WaitingAsRelayer(unacknowledged) => {
                // Try to unlock our incentive
                unacknowledged.verify_challenge(&ack.ack_key_share).map_err(|e| {
                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_RECEIVED_ACKS.increment(&["false"]);

                    AcknowledgementValidation(format!(
                        "the acknowledgement is not sufficient to solve the embedded challenge, {e}"
                    ))
                })?;

                let from_channel = self.db.read().await.get_channel_from(&unacknowledged.signer).await?;

                // Check that the channel with the ticket signer exists and the epoch on the ticket is correct
                if from_channel.is_none()
                    || from_channel.is_some_and(|c| c.channel_epoch.as_u32() != unacknowledged.ticket.channel_epoch)
                {
                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_RECEIVED_ACKS.increment(&["false"]);

                    return Err(AcknowledgementValidation(
                        "acknowledgement received for channel that does not exist or has a newer epoch".into(),
                    ));
                }

                let domain_separator = self
                    .db
                    .read()
                    .await
                    .get_channels_domain_separator()
                    .await
                    .unwrap()
                    .ok_or(MissingDomainSeparator)?;

                let ack_ticket = unacknowledged.acknowledge(&ack.ack_key_share, &self.chain_key, &domain_separator)?;

                // replace the un-acked ticket with acked ticket.
                self.db
                    .write()
                    .await
                    .replace_unack_with_ack(&ack.ack_challenge(), ack_ticket.clone())
                    .await?;

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_RECEIVED_ACKS.increment(&["true"]);

                // Check if ticket is a win
                if ack_ticket.is_winning_ticket(&domain_separator) {
                    debug!("{ack_ticket} is a win");

                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_WINNING_TICKETS_COUNT.increment();

                    Ok(Reply::RelayerWinning(ack_ticket))
                } else {
                    warn!("encountered losing {ack_ticket}");
                    self.db.write().await.mark_losing_acked_ticket(&ack_ticket).await?;

                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_LOSING_TICKETS_COUNT.increment();

                    Ok(Reply::RelayerLosing)
                }
            }
        }
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
/// When a new acknowledgement is delivered from the transport the `receive_acknowledgement`
/// method is used to push it into the processing queue of incoming acknowledgements.
///
/// Acknowledgments issued by this node are generated using the `send_acknowledgement` method.
///
/// The result of processing the acknowledgements can be extracted as a stream.
pub struct AcknowledgementInteraction {
    ack_event_queue: (Sender<AckToProcess>, Receiver<AckProcessed>),
}

impl AcknowledgementInteraction {
    /// Creates a new instance given the DB and our public key used to verify the acknowledgements.
    pub fn new<Db: HoprCoreEthereumDbActions + Send + Sync + 'static>(
        db: Arc<RwLock<Db>>,
        chain_key: &ChainKeypair,
    ) -> Self {
        let (processing_in_tx, processing_in_rx) = channel::<AckToProcess>(ACK_RX_QUEUE_SIZE + ACK_TX_QUEUE_SIZE);
        let (processing_out_tx, processing_out_rx) = channel::<AckProcessed>(ACK_RX_QUEUE_SIZE + ACK_TX_QUEUE_SIZE);

        let processor = AcknowledgementProcessor::new(db, chain_key);

        let processing_stream = processing_in_rx.then_concurrent(move |event| {
            let mut processor = processor.clone();
            let mut processed_tx = processing_out_tx.clone();

            async move {
                let processed: Option<AckProcessed> = match event {
                    AckToProcess::ToReceive(peer, mut ack) => {
                        if let Ok(remote_pk) = OffchainPublicKey::from_peerid(&peer) {
                            debug!("validating incoming acknowledgement from {}", peer);
                            if ack.validate(&remote_pk) {
                                match processor.handle_acknowledgement(ack).await {
                                    Ok(reply) => Some(AckProcessed::Receive(peer, Ok(reply))),
                                    Err(e) => {
                                        error!(
                                            "Encountered error while handling acknowledgement from peer '{}': {}",
                                            &peer, e
                                        );
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

        spawn(async move {
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
