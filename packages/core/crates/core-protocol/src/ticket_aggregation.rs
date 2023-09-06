use crate::errors::ProtocolError::{ProtocolTicketAggregation, Retry, TransportError};
use crate::errors::Result;
use async_lock::RwLock;
use core_crypto::keypairs::{ChainKeypair, Keypair};
use core_mixer::future_extensions::StreamThenConcurrentExt;
use core_types::{
    acknowledgement::AcknowledgedTicket,
    channels::{generate_channel_id, Ticket},
};
use futures::{
    channel::mpsc::{channel, Receiver, Sender, UnboundedSender},
    future::poll_fn,
    {stream::Stream, StreamExt},
};
use libp2p_identity::PeerId;
use std::pin::Pin;
use std::sync::Arc;

use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use utils_log::{debug, error, info, warn};
use utils_types::primitives::{Address, Balance, BalanceType, EthereumChallenge, U256};

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::spawn_local;

#[cfg(all(feature = "wasm", not(test)))]
use wasm_bindgen_futures::spawn_local;

// #[cfg(all(feature = "prometheus", not(test)))]
// use utils_metrics::metrics::SimpleCounter;
// #[cfg(all(feature = "prometheus", not(test)))]
// lazy_static::lazy_static! {
//     static ref METRIC_RECEIVED_SUCCESSFUL_ACKS: SimpleCounter = SimpleCounter::new(
//         "core_counter_received_successful_acks",
//         "Number of received successful acknowledgements"
//     )
//     .unwrap();
// }

// lazy_static::lazy_static! {
//     /// Fixed price per packet to 0.01 HOPR
//     static ref DEFAULT_PRICE_PER_PACKET: U256 = 10000000000000000u128.into();
// }

// Default sizes of the acknowledgement queues
pub const TICKET_AGGREGATION_TX_QUEUE_SIZE: usize = 2048;
pub const TICKET_AGGREGATION_RX_QUEUE_SIZE: usize = 2048;

/// The input to the processor background pipeline
#[derive(Debug)]
pub enum TicketAggregationToProcess {
    ToReceive(PeerId, std::result::Result<AcknowledgedTicket, String>),
    ToProcess(PeerId, Vec<AcknowledgedTicket>),
    ToSend(PeerId, Vec<AcknowledgedTicket>),
}

/// Emitted by the processor background pipeline once processed
#[derive(Debug)]
pub enum TicketAggregationProcessed {
    Receive(PeerId, std::result::Result<AcknowledgedTicket, String>),
    Reply(PeerId, std::result::Result<AcknowledgedTicket, String>),
    Send(PeerId, Vec<AcknowledgedTicket>),
}

/// Implements protocol ticket aggregation logic for acknowledgements
pub struct TicketAggregationProcessor<Db: HoprCoreEthereumDbActions> {
    db: Arc<RwLock<Db>>,
    chain_key: ChainKeypair, // pub on_received: Option<UnboundedSender<HalfKeyChallenge>>,
                             // pub on_acknowledged_ticket: Option<UnboundedSender<AcknowledgedTicket>>,
}

impl<Db: HoprCoreEthereumDbActions> Clone for TicketAggregationProcessor<Db> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            chain_key: self.chain_key.clone(),
            // on_acknowledgement: self.on_acknowledgement.clone(),
            // on_acknowledged_ticket: self.on_acknowledged_ticket.clone(),
        }
    }
}

impl<Db: HoprCoreEthereumDbActions> TicketAggregationProcessor<Db> {
    pub fn new(db: Arc<RwLock<Db>>, chain_key: ChainKeypair) -> Self {
        Self {
            db,
            chain_key, // on_acknowledgement,
                       // on_acknowledged_ticket,
        }
    }

    pub async fn aggregate_tickets(
        &mut self,
        destination: Address,
        mut acked_tickets: Vec<AcknowledgedTicket>,
    ) -> std::result::Result<Ticket, String> {
        if acked_tickets.len() < 1 {
            return Err("At least one ticket required".to_owned());
        }

        if acked_tickets.len() == 1 {
            return Ok(acked_tickets[0].ticket.clone());
        }

        // @TODO who's receiving this error?
        let domain_separator = self
            .db
            .read()
            .await
            .get_channels_domain_separator()
            .await
            .map_err(|e| e.to_string())?
            .expect("missing domain separator");

        let channel_id = generate_channel_id(&(&self.chain_key).into(), &destination);

        acked_tickets.sort();
        acked_tickets.dedup();

        let channel_epoch = acked_tickets[0].ticket.channel_epoch;

        let mut final_value = U256::zero();

        for (i, acked_ticket) in acked_tickets.iter().enumerate() {
            if channel_id != acked_ticket.ticket.channel_id {
                return Err("Invalid channel".to_owned());
            }

            if acked_ticket.ticket.channel_epoch != channel_epoch {
                return Err("Channel epochs do not match".to_owned());
            }

            if i + 1 < acked_tickets.len() {
                if acked_ticket.ticket.index + acked_ticket.ticket.index_offset as u64
                    >= acked_tickets[i + 1].ticket.index
                {
                    return Err("Tickets with overlapping index intervals".to_owned());
                }
            }

            if acked_ticket
                .verify(&(&self.chain_key).into(), &destination, &domain_separator)
                .is_err()
            {
                return Err("Not a valid ticket".to_owned());
            }

            if !acked_ticket.is_winning_ticket(&domain_separator) {
                return Err("Not a winnign ticket".to_owned());
            }

            final_value += acked_ticket.ticket.amount.amount();
        }

        let last_acked_ticket = acked_tickets.last().unwrap();

        Ticket::new(
            &destination,
            &Balance::new(final_value, BalanceType::HOPR),
            (last_acked_ticket.ticket.index + 1).into(),
            1u64.into(),
            1.0,
            channel_epoch.into(),
            last_acked_ticket.ticket.challenge.clone(),
            &self.chain_key,
            &domain_separator,
        )
        .map_err(|e| e.to_string())
    }
}

/// External API for feeding Ticket Aggregation actions into the Ticket Aggregation
/// processor processing the elements independently in the background.
#[derive(Debug, Clone)]
pub struct TicketAggregationActions {
    pub queue: Sender<TicketAggregationToProcess>,
}

impl TicketAggregationActions {
    /// Pushes the aggregated ticket received from the transport layer into processing.
    pub fn receive_ticket(
        &mut self,
        source: PeerId,
        ticket: std::result::Result<AcknowledgedTicket, String>,
    ) -> Result<()> {
        // TODO: received ticket should be emitted somehow and component tickets removed
        Err(crate::errors::ProtocolError::ProtocolTicketAggregation(
            "Failed to process received ticket".to_owned(),
        ))
    }

    /// Process the received aggregation request
    pub fn receive_aggregation_request(&mut self, source: PeerId, tickets: Vec<AcknowledgedTicket>) -> Result<()> {
        // TODO: received tickets should be processed here and a single Ticket emitted

        for ticket in tickets {}
        Err(crate::errors::ProtocolError::ProtocolTicketAggregation(
            "Failed to process received ticket".to_owned(),
        ))
    }

    /// Pushes a new collection of tickets into the processing.
    pub fn send_aggregation_request(&mut self, destination: PeerId, tickets: Vec<AcknowledgedTicket>) -> Result<()> {
        // #[cfg(all(feature = "prometheus", not(test)))]
        // METRIC_SENT_ACKS.increment();
        // TODO: metrics here would be nice as well

        self.process(TicketAggregationToProcess::ToSend(destination, tickets))
    }

    fn process(&mut self, event: TicketAggregationToProcess) -> Result<()> {
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

/// Sets up processing of ticket aggregation interactions and returns relevant read and write mechanism.
///
/// <ADD SPECIFIC DETAILS HERE>
pub struct TicketAggregationInteraction {
    ack_event_queue: (Sender<TicketAggregationToProcess>, Receiver<TicketAggregationProcessed>),
}

impl TicketAggregationInteraction {
    /// Creates a new instance given the DB to process the ticket aggregation requests.
    pub fn new<Db: HoprCoreEthereumDbActions + 'static>(db: Arc<RwLock<Db>>, chain_key: ChainKeypair) -> Self {
        let (processing_in_tx, processing_in_rx) =
            channel::<TicketAggregationToProcess>(TICKET_AGGREGATION_RX_QUEUE_SIZE + TICKET_AGGREGATION_TX_QUEUE_SIZE);
        let (processing_out_tx, processing_out_rx) =
            channel::<TicketAggregationProcessed>(TICKET_AGGREGATION_RX_QUEUE_SIZE + TICKET_AGGREGATION_TX_QUEUE_SIZE);

        let processor = TicketAggregationProcessor::new(db, chain_key);

        let processing_stream = processing_in_rx.then_concurrent(move |event| {
            let mut processor = processor.clone();
            let mut processed_tx = processing_out_tx.clone();

            async move {
                let processed = None;

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

        spawn_local(async move {
            processing_stream
                .map(|x| Ok(x))
                .forward(futures::sink::drain())
                .await
                .unwrap();
        });

        Self {
            ack_event_queue: (processing_in_tx, processing_out_rx),
        }
    }

    pub fn writer(&self) -> TicketAggregationActions {
        TicketAggregationActions {
            queue: self.ack_event_queue.0.clone(),
        }
    }
}

impl Stream for TicketAggregationInteraction {
    type Item = TicketAggregationProcessed;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use futures_lite::stream::StreamExt;
        return std::pin::Pin::new(self).ack_event_queue.1.poll_next(cx);
    }
}

#[cfg(test)]
mod tests {
    #[async_std::test]
    async fn test_ticket_aggregation_should_work() {
        assert!(false)
    }
}
