use crate::errors::{
    ProtocolError::{ProtocolTicketAggregation, Retry, TransportError},
    Result,
};
use async_lock::RwLock;
use chain_db::traits::HoprCoreEthereumDbActions;
use futures::{
    channel::{
        mpsc::{channel, Receiver, Sender},
        oneshot,
    },
    future::{poll_fn, Either},
    pin_mut,
};
use futures_lite::stream::{Stream, StreamExt};
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use libp2p::request_response::{OutboundRequestId, ResponseChannel};
use libp2p_identity::PeerId;
use rust_stream_ext_concurrent::then_concurrent::StreamThenConcurrentExt;
use std::ops::Add;
use std::{pin::Pin, sync::Arc, task::Poll};
use tracing::{debug, error, info, trace, warn, Instrument};

use futures::stream::FuturesUnordered;
use hopr_internal_types::acknowledgement::AcknowledgedTicketStatus;

use async_std::task::{sleep, spawn};

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::SimpleCounter;

use hopr_primitive_types::primitives::U256;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_AGGREGATED_TICKETS: SimpleCounter = SimpleCounter::new(
        "hopr_aggregated_tickets_count",
        "Number of aggregated tickets"
    )
    .unwrap();
    static ref METRIC_AGGREGATION_COUNT: SimpleCounter = SimpleCounter::new(
        "hopr_aggregations_count",
        "Number of performed ticket aggregations"
    )
    .unwrap();
}

// Default sizes of the acknowledgement queues
pub const TICKET_AGGREGATION_TX_QUEUE_SIZE: usize = 2048;
pub const TICKET_AGGREGATION_RX_QUEUE_SIZE: usize = 2048;

/// Variants of lists of acknowledged tickets for aggregation
#[derive(Clone, Debug)]
pub enum AggregationList {
    /// Aggregate all acknowledged tickets in the given channel
    WholeChannel(ChannelEntry),

    /// Aggregate the given range of acknowledged tickets in a channel
    ChannelRange {
        /// ID of the channel
        channel_id: Hash,
        /// Channel epoch
        epoch: u32,
        /// Starting ticket index
        index_start: u64,
        /// The last ticket index (inclusive)
        index_end: u64,
    },

    /// Aggregate the given list of acknowledged tickets.
    /// The tickets must belong to the same channel and already be marked as `BeingAggregated`
    TicketList(Vec<AcknowledgedTicket>),
}

impl AggregationList {
    pub async fn rollback<Db: HoprCoreEthereumDbActions + std::fmt::Debug>(self, db: Arc<RwLock<Db>>) -> Result<()> {
        let tickets = match self {
            AggregationList::WholeChannel(channel) => {
                db.read()
                    .await
                    .get_acknowledged_tickets_range(&channel.get_id(), channel.channel_epoch.as_u32(), 0u64, u64::MAX)
                    .await?
            }
            AggregationList::ChannelRange {
                channel_id,
                epoch,
                index_start,
                index_end,
            } => {
                db.read()
                    .await
                    .get_acknowledged_tickets_range(&channel_id, epoch, index_start, index_end)
                    .await?
            }
            AggregationList::TicketList(list) => list,
        };

        let reverted = tickets
            .iter()
            .map(|t| async {
                let mut ticket = t.clone();
                ticket.status = AcknowledgedTicketStatus::Untouched;
                if let Err(e) = db
                    .write()
                    // .instrument(tracing::debug_span!("db: rollback (update acknowledged ticket)"))
                    .await
                    .update_acknowledged_ticket(&ticket)
                    .await
                {
                    error!("failed to revert {ticket} : {e}");
                    false
                } else {
                    true
                }
            })
            .collect::<FuturesUnordered<_>>()
            .filter(|r| *r)
            .count()
            .await;

        warn!("reverted {reverted} ack tickets to untouched state");
        Ok(())
    }

    #[tracing::instrument(level = "debug")]
    async fn into_vec<Db: HoprCoreEthereumDbActions + std::fmt::Debug>(
        self,
        db: Arc<RwLock<Db>>,
    ) -> Result<Vec<AcknowledgedTicket>> {
        let list = match self {
            AggregationList::WholeChannel(channel) => {
                db.write()
                    .await
                    .prepare_aggregatable_tickets(&channel.get_id(), channel.channel_epoch.as_u32(), 0u64, u64::MAX)
                    .await?
            }
            AggregationList::ChannelRange {
                channel_id,
                epoch,
                index_start,
                index_end,
            } => {
                db.write()
                    .await
                    .prepare_aggregatable_tickets(&channel_id, epoch, index_start, index_end)
                    .await?
            }
            AggregationList::TicketList(list) => list,
        };

        if list.is_empty() {
            debug!("got empty list of tickets to aggregate");
            return Err(ProtocolTicketAggregation("no tickets to aggregate".into()));
        }

        let signer = list[0].signer;
        let channel_id = list[0].ticket.channel_id;

        let channel_balance = db
            .read()
            .await
            .get_channel(&channel_id)
            .await?
            .ok_or(ProtocolTicketAggregation(format!(
                "channel {channel_id} does not exist"
            )))?
            .balance;

        let mut total_amount = Balance::zero(BalanceType::HOPR);

        for tkt in list.iter() {
            if tkt.signer != signer
                || tkt.ticket.channel_id != channel_id
                || tkt.status != AcknowledgedTicketStatus::BeingAggregated
            {
                error!("{tkt} does not belong to the aggregation list");
                return Err(ProtocolTicketAggregation(
                    "invalid list of tickets to aggregate given".into(),
                ));
            }

            total_amount = total_amount.add(&tkt.ticket.amount);
            if total_amount.gt(&channel_balance) {
                return Err(ProtocolTicketAggregation(format!("aggregation list has total value of {total_amount} which is greater than balance {channel_balance} in channel {channel_id}")));
            }
        }

        Ok(list)
    }
}

/// The input to the processor background pipeline
#[allow(clippy::type_complexity)] // TODO: The type needs to be significantly refactored to easily move around
#[allow(clippy::large_enum_variant)] // TODO: refactor the large types used in the enum
#[derive(Debug)]
pub enum TicketAggregationToProcess<T, U> {
    ToReceive(PeerId, std::result::Result<Ticket, String>, U),
    ToProcess(PeerId, Vec<AcknowledgedTicket>, T),
    ToSend(AggregationList, TicketAggregationFinalizer),
}

/// Emitted by the processor background pipeline once processed
#[allow(clippy::large_enum_variant)] // TODO: refactor the large types used in the enum
#[derive(Debug)]
pub enum TicketAggregationProcessed<T, U> {
    Receive(PeerId, AcknowledgedTicket, U),
    Reply(PeerId, std::result::Result<Ticket, String>, T),
    Send(PeerId, Vec<AcknowledgedTicket>, TicketAggregationFinalizer),
}

/// Implements protocol ticket aggregation logic for acknowledgements
pub struct TicketAggregationProcessor<Db: HoprCoreEthereumDbActions + std::fmt::Debug> {
    db: Arc<RwLock<Db>>,
    chain_key: ChainKeypair,
}

impl<Db: HoprCoreEthereumDbActions + std::fmt::Debug> Clone for TicketAggregationProcessor<Db> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            chain_key: self.chain_key.clone(),
        }
    }
}

impl<Db: HoprCoreEthereumDbActions + std::fmt::Debug> TicketAggregationProcessor<Db> {
    pub fn new(db: Arc<RwLock<Db>>, chain_key: &ChainKeypair) -> Self {
        Self {
            db,
            chain_key: chain_key.clone(),
        }
    }

    pub async fn aggregate_tickets(
        &mut self,
        destination: PeerId,
        mut acked_tickets: Vec<AcknowledgedTicket>,
    ) -> Result<Ticket> {
        if acked_tickets.is_empty() {
            return Err(ProtocolTicketAggregation("At least one ticket required".to_owned()));
        }

        if acked_tickets.len() == 1 {
            return Ok(acked_tickets[0].ticket.clone());
        }

        let domain_separator = self
            .db
            .read()
            .await
            .get_channels_domain_separator()
            .await?
            .ok_or_else(|| {
                warn!("Missing domain separator");
                ProtocolTicketAggregation("Missing domain separator".into())
            })?;

        let destination = self
            .db
            .read()
            .await
            .get_chain_key(
                &OffchainPublicKey::try_from(destination)
                    .expect("Invalid PeerId. Could not convert to OffchainPublicKey"),
            )
            .await?
            .ok_or_else(|| {
                warn!("Could not find chain key for {}", destination);
                ProtocolTicketAggregation("Could not find chain key".into())
            })?;

        let channel_id = generate_channel_id(&(&self.chain_key).into(), &destination);
        let channel_entry = self
            .db
            .read()
            .await
            .get_channel(&channel_id)
            .await?
            .ok_or(ProtocolTicketAggregation(format!(
                "channel {channel_id} does not exist"
            )))?;
        let channel_balance = channel_entry.balance;

        acked_tickets.sort();
        acked_tickets.dedup();

        let channel_epoch = channel_entry.channel_epoch;

        let mut final_value = Balance::zero(BalanceType::HOPR);

        for (i, acked_ticket) in acked_tickets.iter().enumerate() {
            if channel_id != acked_ticket.ticket.channel_id {
                return Err(ProtocolTicketAggregation(format!(
                    "aggregated ticket has an invalid channel id {}",
                    acked_ticket.ticket.channel_id
                )));
            }

            if U256::from(acked_ticket.ticket.channel_epoch) != channel_epoch {
                return Err(ProtocolTicketAggregation("Channel epochs do not match".to_owned()));
            }

            if i + 1 < acked_tickets.len()
                && acked_ticket.ticket.index + acked_ticket.ticket.index_offset as u64
                    > acked_tickets[i + 1].ticket.index
            {
                return Err(ProtocolTicketAggregation(
                    "Tickets with overlapping index intervals".to_owned(),
                ));
            }

            if acked_ticket
                .verify(&(&self.chain_key).into(), &destination, &domain_separator)
                .is_err()
            {
                return Err(ProtocolTicketAggregation("Not a valid ticket".to_owned()));
            }

            if !acked_ticket.is_winning_ticket(&domain_separator) {
                return Err(ProtocolTicketAggregation("Not a winning ticket".to_owned()));
            }

            final_value = final_value.add(&acked_ticket.ticket.amount);
            if final_value.gt(&channel_balance) {
                return Err(ProtocolTicketAggregation(format!("ticket amount to aggregate {final_value} is greater than the balance {channel_balance} of channel {channel_id}")));
            }

            #[cfg(all(feature = "prometheus", not(test)))]
            METRIC_AGGREGATED_TICKETS.increment();
        }

        info!(
            "aggregated {} tickets in channel {channel_id} with total value {final_value}",
            acked_tickets.len()
        );

        let first_acked_ticket = acked_tickets.first().unwrap();
        let last_acked_ticket = acked_tickets.last().unwrap();

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_AGGREGATION_COUNT.increment();

        trace!("after ticket aggregation, ensure the current ticket index is larger than the last index and the on-chain index");
        // calculate the minimum current ticket index as the larger value from the acked ticket index and on-chain ticket_index from channel_entry
        let current_ticket_index_from_acked_tickets = U256::from(last_acked_ticket.ticket.index).add(1);
        let current_ticket_index_gte = current_ticket_index_from_acked_tickets.max(channel_entry.ticket_index);
        {
            self.db
                .write()
                // .instrument(tracing::debug_span!(
                //     "db: aggregate tickets (ensure current ticket index gte)"
                // ))
                .await
                .ensure_current_ticket_index_gte(&channel_id, current_ticket_index_gte)
                .await?;
        }

        Ticket::new(
            &destination,
            &final_value,
            first_acked_ticket.ticket.index.into(),
            (last_acked_ticket.ticket.index - first_acked_ticket.ticket.index + 1).into(),
            1.0, // Aggregated tickets have always 100% winning probability
            channel_epoch,
            first_acked_ticket.ticket.challenge.clone(),
            &self.chain_key,
            &domain_separator,
        )
        .map_err(|e| e.into())
    }

    pub async fn handle_aggregated_ticket(&self, aggregated_ticket: Ticket) -> Result<AcknowledgedTicket> {
        let channel_id = aggregated_ticket.channel_id;
        debug!("received aggregated {aggregated_ticket}");

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

        if stored_acked_tickets.is_empty() {
            debug!("Received unexpected aggregated ticket in channel {}", channel_id);
            return Err(ProtocolTicketAggregation("Unexpected ticket".into()));
        }

        let mut stored_value = Balance::zero(BalanceType::HOPR);

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

        // calculate the new current ticket index
        let current_ticket_index_from_aggregated_ticket =
            U256::from(aggregated_ticket.index).add(aggregated_ticket.index_offset);

        let acked_aggregated_ticket = AcknowledgedTicket::new(
            aggregated_ticket,
            first_stored_ticket.response.clone(),
            first_stored_ticket.signer,
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
            // .instrument(tracing::debug_span!(
            //     "storing received aggregated ticket",
            //     ticket = acked_aggregated_ticket.to_string()
            // ))
            .await
            .replace_acked_tickets_by_aggregated_ticket(acked_aggregated_ticket.clone())
            .await?;

        self.db
            .write()
            .instrument(tracing::info_span!(
                "ensure the current ticket index is not smaller than the the aggregated ticket",
                ticket_index = current_ticket_index_from_aggregated_ticket.to_string()
            ))
            .await
            .ensure_current_ticket_index_gte(&channel_id, current_ticket_index_from_aggregated_ticket)
            .await?;

        Ok(acked_aggregated_ticket)
    }

    async fn create_aggregation_request(
        &self,
        ticket_list: AggregationList,
    ) -> Result<(PeerId, Vec<AcknowledgedTicket>)> {
        let tickets_to_aggregate = ticket_list.into_vec(self.db.clone()).await?;

        let signer = tickets_to_aggregate[0].signer;

        let source_peer_id = self.db.read().await.get_packet_key(&signer).await?.ok_or_else(|| {
            ProtocolTicketAggregation(format!(
                "cannot aggregate tickets because we do not know the peer id for {signer}",
            ))
        })?;

        Ok((source_peer_id.into(), tickets_to_aggregate))
    }
}

#[derive(Debug)]
pub struct TicketAggregationAwaiter {
    rx: Option<oneshot::Receiver<()>>,
}

impl From<oneshot::Receiver<()>> for TicketAggregationAwaiter {
    fn from(value: oneshot::Receiver<()>) -> Self {
        Self { rx: Some(value) }
    }
}

impl TicketAggregationAwaiter {
    pub async fn consume_and_wait(&mut self, until_timeout: std::time::Duration) -> Result<()> {
        match self.rx.take() {
            Some(resolve) => {
                let timeout = sleep(until_timeout);
                pin_mut!(resolve, timeout);
                match futures::future::select(resolve, timeout).await {
                    Either::Left((result, _)) => result.map_err(|_| TransportError("Canceled".to_owned())),
                    Either::Right(_) => Err(TransportError("Timed out on sending a packet".to_owned())),
                }
            }
            None => Err(TransportError(
                "Packet send process observation already consumed".to_owned(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct TicketAggregationFinalizer {
    tx: Option<oneshot::Sender<()>>,
}

impl TicketAggregationFinalizer {
    pub fn new(tx: oneshot::Sender<()>) -> Self {
        Self { tx: Some(tx) }
    }

    pub fn finalize(mut self) {
        if let Some(sender) = self.tx.take() {
            if sender.send(()).is_err() {
                error!("Failed to notify the awaiter about the successful ticket aggregation")
            }
        } else {
            error!("Sender for packet send signalization is already spent")
        }
    }
}

/// External API for feeding Ticket Aggregation actions into the Ticket Aggregation
/// processor processing the elements independently in the background.
#[derive(Debug)]
pub struct TicketAggregationActions<T, U> {
    pub queue: Sender<TicketAggregationToProcess<T, U>>,
}

pub type BasicTicketAggregationActions<T> = TicketAggregationActions<ResponseChannel<T>, OutboundRequestId>;

impl<T, U> Clone for TicketAggregationActions<T, U> {
    /// Generic type requires handwritten clone function
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
        }
    }
}

impl<T, U> TicketAggregationActions<T, U> {
    /// Pushes the aggregated ticket received from the transport layer into processing.
    pub fn receive_ticket(
        &mut self,
        source: PeerId,
        ticket: std::result::Result<Ticket, String>,
        request: U,
    ) -> Result<()> {
        self.process(TicketAggregationToProcess::ToReceive(source, ticket, request))
    }

    /// Process the received aggregation request
    pub fn receive_aggregation_request(
        &mut self,
        source: PeerId,
        tickets: Vec<AcknowledgedTicket>,
        request: T,
    ) -> Result<()> {
        self.process(TicketAggregationToProcess::ToProcess(source, tickets, request))
    }

    /// Pushes a new collection of tickets into the processing.
    pub fn aggregate_tickets(&mut self, ack_tickets: AggregationList) -> Result<TicketAggregationAwaiter> {
        let (tx, rx) = oneshot::channel::<()>();

        self.process(TicketAggregationToProcess::ToSend(
            ack_tickets,
            TicketAggregationFinalizer::new(tx),
        ))?;

        Ok(rx.into())
    }

    fn process(&mut self, event: TicketAggregationToProcess<T, U>) -> Result<()> {
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

type AckEventQueue<T, U> = (
    Sender<TicketAggregationToProcess<T, U>>,
    Receiver<TicketAggregationProcessed<T, U>>,
);

/// Sets up processing of ticket aggregation interactions and returns relevant read and write mechanism.
pub struct TicketAggregationInteraction<T, U>
where
    T: Send,
    U: Send,
{
    ack_event_queue: AckEventQueue<T, U>,
}

impl<T: 'static, U: 'static> TicketAggregationInteraction<T, U>
where
    T: Send,
    U: Send,
{
    /// Creates a new instance given the DB to process the ticket aggregation requests.
    pub fn new<Db: HoprCoreEthereumDbActions + Send + Sync + std::fmt::Debug + 'static>(
        db: Arc<RwLock<Db>>,
        chain_key: &ChainKeypair,
    ) -> Self {
        let (processing_in_tx, processing_in_rx) = channel::<TicketAggregationToProcess<T, U>>(
            TICKET_AGGREGATION_RX_QUEUE_SIZE + TICKET_AGGREGATION_TX_QUEUE_SIZE,
        );
        let (processing_out_tx, processing_out_rx) = channel::<TicketAggregationProcessed<T, U>>(
            TICKET_AGGREGATION_RX_QUEUE_SIZE + TICKET_AGGREGATION_TX_QUEUE_SIZE,
        );

        let processor = TicketAggregationProcessor::new(db, chain_key);

        let mut processing_stream = processing_in_rx.then_concurrent(move |event| {
            let mut processor = processor.clone();
            let mut processed_tx = processing_out_tx.clone();

            async move {
                let processed = match event {
                    TicketAggregationToProcess::ToProcess(destination, acked_tickets, response) => {
                        match processor.aggregate_tickets(destination, acked_tickets).await {
                            Ok(tickets) => Some(TicketAggregationProcessed::Reply(destination, Ok(tickets), response)),
                            Err(ProtocolTicketAggregation(e)) => {
                                // forward error to counterparty
                                Some(TicketAggregationProcessed::Reply(destination, Err(e), response))
                            }
                            Err(e) => {
                                error!("Dropping tickets aggregation request due unexpected error {e}");
                                None
                            }
                        }
                    }
                    TicketAggregationToProcess::ToReceive(destination, aggregated_ticket, request) => {
                        match aggregated_ticket {
                            Ok(ticket) => match processor.handle_aggregated_ticket(ticket.clone()).await {
                                Ok(acked_ticket) => {
                                    Some(TicketAggregationProcessed::Receive(destination, acked_ticket, request))
                                }
                                Err(e) => {
                                    error!("Error while handling aggregated ticket: {e}");
                                    None
                                }
                            },
                            Err(e) => {
                                warn!("Counterparty refused to aggregate tickets: {e}");
                                None
                            }
                        }
                    }
                    TicketAggregationToProcess::ToSend(tickets_to_agg, finalizer) => {
                        match processor.create_aggregation_request(tickets_to_agg).await {
                            Ok((source, tickets)) => Some(TicketAggregationProcessed::Send(source, tickets, finalizer)),
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

        spawn(async move {
            // poll the stream until it's done
            while processing_stream.next().await.is_some() {}
        });

        Self {
            ack_event_queue: (processing_in_tx, processing_out_rx),
        }
    }

    pub fn writer(&self) -> TicketAggregationActions<T, U> {
        TicketAggregationActions {
            queue: self.ack_event_queue.0.clone(),
        }
    }
}

impl<T, U> Stream for TicketAggregationInteraction<T, U>
where
    T: Send,
    U: Send,
{
    type Item = TicketAggregationProcessed<T, U>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(self).ack_event_queue.1.poll_next(cx)
    }
}

#[cfg(test)]
mod tests {
    use async_lock::RwLock;
    use chain_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
    use futures_lite::StreamExt;
    use hex_literal::hex;
    use hopr_crypto_types::{
        keypairs::{ChainKeypair, Keypair, OffchainKeypair},
        types::{Hash, Response},
    };
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use lazy_static::lazy_static;
    use std::ops::{Add, Mul};
    use std::{sync::Arc, time::Duration};
    use utils_db::constants::ACKNOWLEDGED_TICKETS_PREFIX;
    use utils_db::{db::DB, CurrentDbShim};

    use super::{AggregationList, TicketAggregationProcessed};

    lazy_static! {
        static ref PEERS: Vec<OffchainKeypair> = [
            hex!("b91a28ff9840e9c93e5fafd581131f0b9f33f3e61b02bf5dd83458aa0221f572"),
            hex!("82283757872f99541ce33a47b90c2ce9f64875abf08b5119a8a434b2fa83ea98")
        ]
        .iter()
        .map(|private| OffchainKeypair::from_secret(private).unwrap())
        .collect();
        static ref PEERS_CHAIN: Vec<ChainKeypair> = [
            hex!("51d3003d908045a4d76d0bfc0d84f6ff946b5934b7ea6a2958faf02fead4567a"),
            hex!("e1f89073a01831d0eed9fe2c67e7d65c144b9d9945320f6d325b1cccc2d124e9")
        ]
        .iter()
        .map(|private| ChainKeypair::from_secret(private).unwrap())
        .collect();
    }

    fn mock_acknowledged_ticket(signer: &ChainKeypair, destination: &ChainKeypair, index: u64) -> AcknowledgedTicket {
        let price_per_packet: U256 = 10000000000000000u128.into();
        let ticket_win_prob = 1.0f64;

        let channel_id = generate_channel_id(&signer.into(), &destination.into());

        let channel_epoch = 1u64;
        let domain_separator = Hash::default();

        let response = Response::new(
            &Hash::create(&[
                &channel_id.to_bytes(),
                &channel_epoch.to_be_bytes(),
                &index.to_be_bytes(),
            ])
            .to_bytes(),
        );

        let ticket = Ticket::new(
            &destination.into(),
            &Balance::new(price_per_packet.div_f64(ticket_win_prob).unwrap(), BalanceType::HOPR),
            index.into(),
            1u64.into(),
            ticket_win_prob,
            1u64.into(),
            response.to_challenge().into(),
            signer,
            &domain_separator,
        )
        .unwrap();

        AcknowledgedTicket::new(ticket, response, signer.into(), destination, &domain_separator).unwrap()
    }

    async fn init_dbs(inner_dbs: Vec<DB<CurrentDbShim>>) -> Vec<Arc<RwLock<CoreEthereumDb<CurrentDbShim>>>> {
        let mut dbs = Vec::new();
        for (i, inner_db) in inner_dbs.into_iter().enumerate() {
            let db = Arc::new(RwLock::new(CoreEthereumDb::new(inner_db, (&PEERS_CHAIN[i]).into())));

            db.write()
                .await
                .set_channels_domain_separator(&Hash::default(), &Snapshot::default())
                .await
                .unwrap();

            for i in 0..PEERS.len() {
                db.write()
                    .await
                    .link_chain_and_packet_keys(&(&PEERS_CHAIN[i]).into(), PEERS[i].public(), &Snapshot::default())
                    .await
                    .unwrap();
            }

            dbs.push(db);
        }
        dbs
    }

    fn to_acknowledged_ticket_key(ack: &AcknowledgedTicket) -> utils_db::db::Key {
        let mut ack_key = Vec::new();

        ack_key.extend_from_slice(&ack.ticket.channel_id.to_bytes());
        ack_key.extend_from_slice(&ack.ticket.channel_epoch.to_be_bytes());
        ack_key.extend_from_slice(&ack.ticket.index.to_be_bytes());

        utils_db::db::Key::new_bytes_with_prefix(&ack_key, ACKNOWLEDGED_TICKETS_PREFIX).unwrap()
    }

    #[async_std::test]
    async fn test_ticket_aggregation() {
        let _ = env_logger::builder().is_test(true).try_init();

        let mut inner_dbs =
            futures::future::join_all((0..2).map(|_| async { DB::new(CurrentDbShim::new_in_memory().await) })).await;

        const NUM_TICKETS: u64 = 30;

        let mut agg_balance = Balance::zero(BalanceType::HOPR);
        // Generate acknowledged tickets
        for i in 1..=NUM_TICKETS {
            let mut ack_ticket = mock_acknowledged_ticket(&PEERS_CHAIN[0], &PEERS_CHAIN[1], i);

            // Mark the first ticket as redeemed, so it does not enter the aggregation
            if i == 1 {
                ack_ticket.status = AcknowledgedTicketStatus::BeingRedeemed;
            } else {
                agg_balance = agg_balance.add(&ack_ticket.ticket.amount);
            }

            inner_dbs[1]
                .set(to_acknowledged_ticket_key(&ack_ticket), &ack_ticket)
                .await
                .unwrap();
        }

        let dbs = init_dbs(inner_dbs).await;

        let alice_addr: Address = (&PEERS_CHAIN[0]).into();
        let bob_addr: Address = (&PEERS_CHAIN[1]).into();

        let alice_packet_key = PEERS[0].public().into();
        let bob_packet_key = PEERS[1].public().into();

        let channel_id_alice_bob = generate_channel_id(&(&PEERS_CHAIN[0]).into(), &(&PEERS_CHAIN[1]).into());

        let channel_alice_bob = ChannelEntry::new(
            alice_addr,
            bob_addr,
            agg_balance.mul(10),
            NUM_TICKETS.into(),
            ChannelStatus::Open,
            1u32.into(),
        );

        dbs[1]
            .write()
            .await
            .update_channel_and_snapshot(&channel_id_alice_bob, &channel_alice_bob, &Snapshot::default())
            .await
            .unwrap();

        dbs[0]
            .write()
            .await
            .update_channel_and_snapshot(&channel_id_alice_bob, &channel_alice_bob, &Snapshot::default())
            .await
            .unwrap();

        let mut alice = super::TicketAggregationInteraction::<(), ()>::new(dbs[0].clone(), &PEERS_CHAIN[0]);
        let mut bob = super::TicketAggregationInteraction::<(), ()>::new(dbs[1].clone(), &PEERS_CHAIN[1]);

        let mut awaiter = bob
            .writer()
            .aggregate_tickets(AggregationList::WholeChannel(channel_alice_bob))
            .unwrap();
        let mut finalizer = None;
        match bob.next().await {
            Some(TicketAggregationProcessed::Send(_, acked_tickets, request_finalizer)) => {
                let _ = finalizer.insert(request_finalizer);
                alice
                    .writer()
                    .receive_aggregation_request(bob_packet_key, acked_tickets, ())
                    .unwrap();
            }
            _ => panic!("unexpected action happened"),
        };

        match alice.next().await {
            Some(TicketAggregationProcessed::Reply(_, aggregated_ticket, ())) => bob
                .writer()
                .receive_ticket(alice_packet_key, aggregated_ticket, ())
                .unwrap(),
            _ => panic!("unexpected action happened"),
        };

        match bob.next().await {
            Some(TicketAggregationProcessed::Receive(_destination, _acked_tkt, ())) => {
                finalizer.take().unwrap().finalize()
            }
            _ => panic!("unexpected action happened"),
        }

        let stored_acked_tickets = dbs[1]
            .read()
            .await
            .get_acknowledged_tickets_range(&channel_id_alice_bob, 1u32, 0, u64::MAX)
            .await
            .unwrap();

        assert_eq!(
            stored_acked_tickets.len(),
            2,
            "there should be 1 aggregated ticket and 1 ticket being redeemed"
        );

        assert_eq!(
            AcknowledgedTicketStatus::BeingRedeemed,
            stored_acked_tickets[0].status,
            "first ticket must being redeemed"
        );
        assert!(
            stored_acked_tickets[1].ticket.is_aggregated(),
            "aggregated balance invalid"
        );
        assert_eq!(
            AcknowledgedTicketStatus::Untouched,
            stored_acked_tickets[1].status,
            "second ticket must be untouched"
        );
        assert_eq!(
            agg_balance, stored_acked_tickets[1].ticket.amount,
            "aggregated balance invalid"
        );

        awaiter.consume_and_wait(Duration::from_millis(2000)).await.unwrap();
    }
}
