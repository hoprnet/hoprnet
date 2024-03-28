use crate::errors::{
    ProtocolError::{Retry, TransportError},
    Result,
};
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
pub use hopr_db_api::tickets::AggregationPrerequisites;
use hopr_db_api::tickets::HoprDbTicketOperations;
use hopr_internal_types::prelude::*;
use libp2p::request_response::{OutboundRequestId, ResponseChannel};
use libp2p_identity::PeerId;
use rust_stream_ext_concurrent::then_concurrent::StreamThenConcurrentExt;
use std::{pin::Pin, task::Poll};
use tracing::{error, warn};

use async_std::task::{sleep, spawn};

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::SimpleCounter;

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

/// The input to the processor background pipeline
#[allow(clippy::type_complexity)] // TODO: The type needs to be significantly refactored to easily move around
#[allow(clippy::large_enum_variant)] // TODO: refactor the large types used in the enum
#[derive(Debug)]
pub enum TicketAggregationToProcess<T, U> {
    ToReceive(PeerId, std::result::Result<Ticket, String>, U),
    ToProcess(PeerId, Vec<AcknowledgedTicket>, T),
    ToSend(Hash, AggregationPrerequisites, TicketAggregationFinalizer),
}

/// Emitted by the processor background pipeline once processed
#[allow(clippy::large_enum_variant)] // TODO: refactor the large types used in the enum
#[derive(Debug)]
pub enum TicketAggregationProcessed<T, U> {
    Receive(PeerId, AcknowledgedTicket, U),
    Reply(PeerId, std::result::Result<Ticket, String>, T),
    Send(PeerId, Vec<AcknowledgedTicket>, TicketAggregationFinalizer),
}

#[derive(Debug)]
pub struct AwaitingAggregator<T, U, Db>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
    T: Send,
    U: Send,
{
    db: Db,
    ck: ChainKeypair,
    writer: TicketAggregationActions<T, U>,
    agg_timeout: std::time::Duration,
}

impl<T, U, Db> Clone for AwaitingAggregator<T, U, Db>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
    T: Send,
    U: Send,
{
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            ck: self.ck.clone(),
            writer: self.writer.clone(),
            agg_timeout: self.agg_timeout,
        }
    }
}

impl<T, U, Db> AwaitingAggregator<T, U, Db>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
    T: Send,
    U: Send,
{
    pub fn new(
        db: Db,
        ck: ChainKeypair,
        writer: TicketAggregationActions<T, U>,
        agg_timeout: std::time::Duration,
    ) -> Self {
        Self {
            db,
            ck,
            writer,
            agg_timeout,
        }
    }

    pub async fn aggregate_tickets_in_the_channel(
        &self,
        channel: &Hash,
        prerequisites: AggregationPrerequisites,
    ) -> Result<()> {
        let mut awaiter = self.writer.clone().aggregate_tickets(channel, prerequisites)?;

        if let Err(e) = awaiter.consume_and_wait(self.agg_timeout).await {
            warn!("Error occured on ticket aggregation for '{channel}', performing a rollback: {e}");
            self.db.rollback_aggregation_in_channel(*channel).await?;
        }

        Ok(())
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
    pub fn aggregate_tickets(
        &mut self,
        channel: &Hash,
        prerequisites: AggregationPrerequisites,
    ) -> Result<TicketAggregationAwaiter> {
        let (tx, rx) = oneshot::channel::<()>();

        self.process(TicketAggregationToProcess::ToSend(
            *channel,
            prerequisites,
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
    pub fn new<Db>(db: Db, chain_key: &ChainKeypair) -> Self
    where
        Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug + 'static,
    {
        let (processing_in_tx, processing_in_rx) = channel::<TicketAggregationToProcess<T, U>>(
            TICKET_AGGREGATION_RX_QUEUE_SIZE + TICKET_AGGREGATION_TX_QUEUE_SIZE,
        );
        let (processing_out_tx, processing_out_rx) = channel::<TicketAggregationProcessed<T, U>>(
            TICKET_AGGREGATION_RX_QUEUE_SIZE + TICKET_AGGREGATION_TX_QUEUE_SIZE,
        );

        let chain_key = chain_key.clone();

        let mut processing_stream = processing_in_rx.then_concurrent(move |event| {
            let chain_key = chain_key.clone();
            let db = db.clone();
            let mut processed_tx = processing_out_tx.clone();

            async move {
                let processed = match event {
                    TicketAggregationToProcess::ToProcess(destination, acked_tickets, response) => {
                        let opk: std::result::Result<OffchainPublicKey, hopr_primitive_types::errors::GeneralError> =
                            destination.try_into();
                        match opk {
                            Ok(opk) => match db.aggregate_tickets(opk, acked_tickets, &chain_key).await {
                                Ok(tickets) => {
                                    Some(TicketAggregationProcessed::Reply(destination, Ok(tickets), response))
                                }
                                Err(hopr_db_api::errors::DbError::TicketAggregationError(e)) => {
                                    // forward error to counterparty
                                    Some(TicketAggregationProcessed::Reply(destination, Err(e), response))
                                }
                                Err(e) => {
                                    error!("Dropping tickets aggregation request due unexpected error {e}");
                                    None
                                }
                            },
                            Err(e) => {
                                error!(
                                    "Failed to deserialize the destination '{destination}' to an offchain public key: {e}"
                                );
                                None
                            }
                        }
                    }
                    TicketAggregationToProcess::ToReceive(destination, aggregated_ticket, request) => {
                            match aggregated_ticket {
                                Ok(ticket) => match db
                                    .process_received_aggregated_ticket(ticket.clone(), &chain_key)
                                    .await
                                {
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
                    TicketAggregationToProcess::ToSend(channel, prerequsites, finalizer) => {
                        match db.prepare_aggregation_in_channel(&channel, prerequsites).await {
                            Ok(Some((source, tickets))) => {
                                #[cfg(all(feature = "prometheus", not(test)))]
                                {
                                    METRIC_AGGREGATED_TICKETS.increment_by(tickets.len() as u64);
                                    METRIC_AGGREGATION_COUNT.increment();
                                }

                                Some(TicketAggregationProcessed::Send(source.into(), tickets, finalizer))
                            }
                            Ok(None) => { finalizer.finalize(); None },
                            Err(e) => {
                                error!("An error occured when preparing the channel aggregation: {e}");
                                None
                            },
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
    use async_std::prelude::FutureExt;
    use futures_lite::StreamExt;
    use hex_literal::hex;
    use hopr_crypto_types::{
        keypairs::{ChainKeypair, Keypair, OffchainKeypair},
        types::{Hash, Response},
    };
    use hopr_db_api::accounts::HoprDbAccountOperations;
    use hopr_db_api::channels::HoprDbChannelOperations;
    use hopr_db_api::db::HoprDb;
    use hopr_db_api::info::{DomainSeparator, HoprDbInfoOperations};
    use hopr_db_api::tickets::HoprDbTicketOperations;
    use hopr_db_api::HoprDbGeneralModelOperations;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use lazy_static::lazy_static;
    use std::ops::{Add, Mul};
    use std::time::Duration;

    use super::TicketAggregationProcessed;

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

    async fn init_db(db: HoprDb) {
        let db_clone = db.clone();

        let peers = PEERS.clone();
        let peers_chain = PEERS_CHAIN.clone();

        db.begin_transaction()
            .await
            .unwrap()
            .perform(move |tx| {
                Box::pin(async move {
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Hash::default())
                        .await
                        .unwrap();
                    for (offchain, chain) in peers.iter().zip(peers_chain.iter()) {
                        db_clone
                            .insert_account(
                                Some(tx),
                                AccountEntry::new(
                                    *offchain.public(),
                                    chain.public().to_address(),
                                    AccountType::NotAnnounced,
                                ),
                            )
                            .await?
                    }

                    Ok::<(), hopr_db_api::errors::DbError>(())
                })
            })
            .await
            .unwrap();
    }

    #[async_std::test]
    async fn test_ticket_aggregation() {
        let _ = env_logger::builder().is_test(true).try_init();

        let db_alice = HoprDb::new_in_memory(PEERS_CHAIN[0].clone()).await;
        let db_bob = HoprDb::new_in_memory(PEERS_CHAIN[1].clone()).await;
        init_db(db_alice.clone()).await;
        init_db(db_bob.clone()).await;

        const NUM_TICKETS: u64 = 30;

        let mut tickets = vec![];
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

            tickets.push(ack_ticket)
        }

        let alice_addr: Address = (&PEERS_CHAIN[0]).into();
        let bob_addr: Address = (&PEERS_CHAIN[1]).into();

        let alice_packet_key = PEERS[0].public().into();
        let bob_packet_key = PEERS[1].public().into();

        let channel_alice_bob = ChannelEntry::new(
            alice_addr,
            bob_addr,
            agg_balance.mul(10),
            NUM_TICKETS.into(),
            ChannelStatus::Open,
            1u32.into(),
        );

        db_alice.upsert_channel(None, channel_alice_bob).await.unwrap();
        db_bob.upsert_channel(None, channel_alice_bob).await.unwrap();

        for ticket in tickets.into_iter() {
            db_bob.upsert_ticket(None, ticket).await.unwrap();
        }

        let mut alice = super::TicketAggregationInteraction::<(), ()>::new(db_alice.clone(), &PEERS_CHAIN[0]);
        let mut bob = super::TicketAggregationInteraction::<(), ()>::new(db_bob.clone(), &PEERS_CHAIN[1]);

        let mut awaiter = bob
            .writer()
            .aggregate_tickets(&channel_alice_bob.get_id(), Default::default())
            .unwrap();

        let mut finalizer = None;
        match bob.next().timeout(Duration::from_secs(5)).await {
            Ok(Some(TicketAggregationProcessed::Send(_, acked_tickets, request_finalizer))) => {
                let _ = finalizer.insert(request_finalizer);
                alice
                    .writer()
                    .receive_aggregation_request(bob_packet_key, acked_tickets, ())
                    .unwrap();
            }
            _ => panic!("unexpected action happened while sending agg request by Bob"),
        };

        match alice.next().timeout(Duration::from_secs(5)).await {
            Ok(Some(TicketAggregationProcessed::Reply(_, aggregated_ticket, ()))) => bob
                .writer()
                .receive_ticket(alice_packet_key, aggregated_ticket, ())
                .unwrap(),
            _ => panic!("unexpected action happened while awaiting agg request at Alice"),
        };

        match bob.next().timeout(Duration::from_secs(5)).await {
            Ok(Some(TicketAggregationProcessed::Receive(_destination, _acked_tkt, ()))) => {
                finalizer.take().unwrap().finalize()
            }
            _ => panic!("unexpected action happened while awaiting agg response at Bob"),
        }

        let stored_acked_tickets = db_bob.get_tickets(None, (&channel_alice_bob).into()).await.unwrap();

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
