use crate::errors::{
    ProtocolError::{self, ProtocolTicketAggregation, Retry, TransportError},
    Result,
};
use async_lock::RwLock;
use core_crypto::{
    keypairs::{ChainKeypair, Keypair},
    types::OffchainPublicKey,
};
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_mixer::future_extensions::StreamThenConcurrentExt;
use core_types::{
    acknowledgement::AcknowledgedTicket,
    channels::{generate_channel_id, Ticket},
};
use futures::{
    channel::mpsc::{channel, Receiver, Sender, UnboundedSender},
    future::poll_fn,
};
use futures_lite::stream::{Stream, StreamExt};
use libp2p_identity::PeerId;
use std::{pin::Pin, sync::Arc, task::Poll};
use utils_db::errors::DbError;
use utils_log::{debug, error, info, warn};
use utils_types::{
    primitives::{Address, Balance, BalanceType, EthereumChallenge, U256},
    traits::PeerIdLike,
};

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
    ToReceive(PeerId, std::result::Result<Ticket, String>),
    ToProcess(PeerId, Vec<AcknowledgedTicket>),
    ToSend(PeerId),
}

/// Emitted by the processor background pipeline once processed
#[derive(Debug)]
pub enum TicketAggregationProcessed {
    Receive(PeerId, std::result::Result<Ticket, String>),
    Reply(PeerId, std::result::Result<Ticket, String>),
    Send(PeerId, Vec<AcknowledgedTicket>),
}

/// Implements protocol ticket aggregation logic for acknowledgements
pub struct TicketAggregationProcessor<Db: HoprCoreEthereumDbActions> {
    db: Arc<RwLock<Db>>,
    chain_key: ChainKeypair,
}

impl<Db: HoprCoreEthereumDbActions> Clone for TicketAggregationProcessor<Db> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            chain_key: self.chain_key.clone(),
        }
    }
}

impl<Db: HoprCoreEthereumDbActions> TicketAggregationProcessor<Db> {
    pub fn new(db: Arc<RwLock<Db>>, chain_key: &ChainKeypair) -> Self {
        Self { db, chain_key: chain_key.clone() }
    }

    pub async fn aggregate_tickets(
        &mut self,
        destination: PeerId,
        mut acked_tickets: Vec<AcknowledgedTicket>,
    ) -> Result<Ticket> {
        if acked_tickets.len() < 1 {
            return Err(ProtocolTicketAggregation("At least one ticket required".to_owned()));
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
            .await?
            .ok_or_else(|| {
                debug!("Missing domain separator");
                ProtocolTicketAggregation("Missing domain separator".into())
            })?;

        let destination = self
            .db
            .read()
            .await
            .get_chain_key(
                &OffchainPublicKey::from_peerid(&destination)
                    .expect("Invalid PeerId. Could not convert to OffchainPublicKey"),
            )
            .await?
            .ok_or_else(|| {
                debug!("Could not find chain key for {}", destination);
                ProtocolTicketAggregation("Could not find chain key".into())
            })?;

        let channel_id = generate_channel_id(&(&self.chain_key).into(), &destination);

        acked_tickets.sort();
        acked_tickets.dedup();

        let channel_epoch = acked_tickets[0].ticket.channel_epoch;

        let mut final_value = U256::zero();

        for (i, acked_ticket) in acked_tickets.iter().enumerate() {
            if channel_id != acked_ticket.ticket.channel_id {
                return Err(ProtocolTicketAggregation("Invalid channel".to_owned()));
            }

            if acked_ticket.ticket.channel_epoch != channel_epoch {
                return Err(ProtocolTicketAggregation("Channel epochs do not match".to_owned()));
            }

            if i + 1 < acked_tickets.len() {
                if acked_ticket.ticket.index + acked_ticket.ticket.index_offset as u64
                    >= acked_tickets[i + 1].ticket.index
                {
                    return Err(ProtocolTicketAggregation(
                        "Tickets with overlapping index intervals".to_owned(),
                    ));
                }
            }

            if acked_ticket
                .verify(&(&self.chain_key).into(), &destination, &domain_separator)
                .is_err()
            {
                return Err(ProtocolTicketAggregation("Not a valid ticket".to_owned()));
            }

            if !acked_ticket.is_winning_ticket(&domain_separator) {
                return Err(ProtocolTicketAggregation("Not a winnign ticket".to_owned()));
            }

            final_value += acked_ticket.ticket.amount.amount();
        }

        let first_acked_ticket = acked_tickets.first().unwrap();
        let last_acked_ticket = acked_tickets.last().unwrap();

        Ticket::new(
            &destination,
            &Balance::new(final_value, BalanceType::HOPR),
            first_acked_ticket.ticket.index.into(),
            (last_acked_ticket.ticket.index - first_acked_ticket.ticket.index).into(),
            1.0,
            channel_epoch.into(),
            first_acked_ticket.ticket.challenge.clone(),
            &self.chain_key,
            &domain_separator,
        ).map_err(|e| e.into())
    }

    pub async fn handle_aggregated_ticket(&self, aggregated_ticket: Ticket) -> Result<()> {
        let channel_id = aggregated_ticket.channel_id.clone();
        let stored_acked_tickets = self
            .db
            .read()
            .await
            .get_acknowledged_tickets_range(
                &aggregated_ticket.channel_id,
                aggregated_ticket.channel_epoch,
                aggregated_ticket.index,
                aggregated_ticket.index + aggregated_ticket.index_offset as u64 - 1u64,
            )
            .await?;

        if stored_acked_tickets.len() == 0 {
            debug!("Received unexpected aggregated ticket in channel {}", channel_id);
            return Err(ProtocolTicketAggregation("Unexpected ticket".into()));
        }

        let mut stored_value = Balance::new(U256::zero(), BalanceType::HOPR);

        for stored_acked_ticket in stored_acked_tickets.iter() {
            stored_value = stored_value.add(&stored_acked_ticket.ticket.amount);
        }

        // Value of received ticket can be higher (profit for us) but not lower
        if aggregated_ticket.amount.lt(&stored_value) {
            debug!(
                "Dropping aggregated ticket in channel {} because its value is lower than sum of stored tickets",
                channel_id
            );
            return Err(ProtocolTicketAggregation(
                "Value of received aggregated ticket is too low".into(),
            ));
        }

        if aggregated_ticket.win_prob() != 1.0f64 {
            debug!(
                "Received aggregated ticket in channel {} win probability less than 100%",
                channel_id
            );
            return Err(ProtocolTicketAggregation(
                "Aggregated tickets must have 100% win probability".into(),
            ));
        }

        let first_stored_ticket = stored_acked_tickets.first().unwrap();

        let domain_separator = self
            .db
            .read()
            .await
            .get_channels_domain_separator()
            .await?
            .ok_or_else(|| {
                debug!(
                    "cannot process aggregated ticket in channel {} due to missing domain separator",
                    channel_id
                );
                ProtocolTicketAggregation("Missing domain separator".into())
            })?;

        let acked_aggregated_ticket = AcknowledgedTicket::new(
            aggregated_ticket,
            first_stored_ticket.response.clone(),
            first_stored_ticket.signer.clone(),
            &self.chain_key,
            &domain_separator,
        )
        .map_err(|e| {
            ProtocolTicketAggregation(format!(
                "Cannot create acknowledged ticket from aggregated ticket {}",
                e
            ))
        })?;

        if acked_aggregated_ticket
            .verify(
                &first_stored_ticket.signer,
                &(&self.chain_key).into(),
                &domain_separator,
            )
            .is_err()
        {
            debug!(
                "Received an aggregated ticket in channel {} that is invalid. Dropping ticket.",
                channel_id
            );
            return Err(ProtocolTicketAggregation("Aggregated ticket is invalid".into()));
        }

        self.db
            .write()
            .await
            .replace_acked_tickets_by_aggregated_ticket(acked_aggregated_ticket)
            .await?;

        Ok(())
        // AcknowledgedTicket::new(ticket, Response::default(), Address::default(), self.)
    }

    pub async fn prepare_aggregatable_tickets(&self, source: &PeerId) -> Result<Vec<AcknowledgedTicket>> {
        let source_addr = self
            .db
            .read()
            .await
            .get_chain_key(&OffchainPublicKey::from_peerid(source)?)
            .await?
            .ok_or_else(|| {
                ProtocolTicketAggregation(format!(
                    "Cannot aggregate tickets because we do not know the chain address for peer {}",
                    source
                ))
            })?;

        let channel_id = generate_channel_id(&source_addr, &(&self.chain_key).into());

        let channel = self.db.read().await.get_channel(&channel_id).await?.ok_or_else(|| {
            ProtocolTicketAggregation(format!(
                "Cannot aggregate tickets in channel {} because indexer has no record for that particular channel",
                channel_id
            ))
        })?;

        let tickets_to_aggregate = self
            .db
            .read()
            .await
            .get_acknowledged_tickets_range(&channel_id, channel.channel_epoch.as_u32(), 0u64, u64::MAX)
            .await?;

        if tickets_to_aggregate.len() == 0 {
            debug!("Dropping request to aggretate tickets in channel {}", channel_id);
            return Err(ProtocolTicketAggregation("No tickets to aggregate".into()));
        }

        Ok(tickets_to_aggregate)
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
    pub fn receive_ticket(&mut self, source: PeerId, ticket: std::result::Result<Ticket, String>) -> Result<()> {
        // TODO: received ticket should be emitted somehow and component tickets removed
        self.process(TicketAggregationToProcess::ToReceive(source, ticket))
    }

    /// Process the received aggregation request
    pub fn receive_aggregation_request(&mut self, source: PeerId, tickets: Vec<AcknowledgedTicket>) -> Result<()> {
        // TODO: received tickets should be processed here and a single Ticket emitted

        self.process(TicketAggregationToProcess::ToProcess(source, tickets))
    }

    /// Pushes a new collection of tickets into the processing.
    pub fn aggregate_tickets(&mut self, destination: PeerId) -> Result<()> {
        // #[cfg(all(feature = "prometheus", not(test)))]
        // METRIC_SENT_ACKS.increment();
        // TODO: metrics here would be nice as well

        self.process(TicketAggregationToProcess::ToSend(destination))
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
    pub fn new<Db: HoprCoreEthereumDbActions + 'static>(db: Arc<RwLock<Db>>, chain_key: &ChainKeypair) -> Self {
        let (processing_in_tx, processing_in_rx) =
            channel::<TicketAggregationToProcess>(TICKET_AGGREGATION_RX_QUEUE_SIZE + TICKET_AGGREGATION_TX_QUEUE_SIZE);
        let (processing_out_tx, processing_out_rx) =
            channel::<TicketAggregationProcessed>(TICKET_AGGREGATION_RX_QUEUE_SIZE + TICKET_AGGREGATION_TX_QUEUE_SIZE);

        let processor = TicketAggregationProcessor::new(db, chain_key);

        let mut processing_stream = processing_in_rx.then_concurrent(move |event| {
            let mut processor = processor.clone();
            let mut processed_tx = processing_out_tx.clone();

            async move {
                let processed = match event {
                    TicketAggregationToProcess::ToProcess(destination, acked_tickets) => {
                        match processor.aggregate_tickets(destination, acked_tickets).await {
                            Ok(tickets) => Some(TicketAggregationProcessed::Reply(destination, Ok(tickets))),
                            Err(e) => {
                                debug!("Dropping tickets aggregation request due unexpected error {}", e);
                                None
                            }
                            Err(ProtocolTicketAggregation(e)) => {
                                // forward error to counterparty
                                Some(TicketAggregationProcessed::Reply(destination, Err(e.to_string())))
                            }
                        }
                    }
                    TicketAggregationToProcess::ToReceive(destination, aggregated_ticket) => match aggregated_ticket {
                        Ok(ticket) => {
                            processor.handle_aggregated_ticket(ticket).await;
                            None
                        }
                        Err(e) => {
                            debug!("Counterparty refused to aggregrate tickets. {}", e);
                            None
                        }
                    },
                    TicketAggregationToProcess::ToSend(source) => {
                        match processor.prepare_aggregatable_tickets(&source).await {
                            Ok(tickets) => Some(TicketAggregationProcessed::Send(source, tickets)),
                            Err(_) => None,
                        }
                    }
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

        spawn_local(async move {
            // poll the stream until it's done
            while processing_stream.next().await.is_some() {}
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

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        return Pin::new(self).ack_event_queue.1.poll_next(cx);
    }
}

#[cfg(test)]
mod tests {
    #[async_std::test]
    async fn test_ticket_aggregation_should_work() {
        assert!(false)
    }
}
