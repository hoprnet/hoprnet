use crate::{strategy::SingularStrategy, Strategy};
use async_std::sync::{Mutex, RwLock};
use async_trait::async_trait;
use core_ethereum_actions::{
    errors::CoreEthereumActionsError::ChannelDoesNotExist, redeem::redeem_tickets_in_channel,
    transaction_queue::TransactionSender,
};
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_protocol::ticket_aggregation::processor::TicketAggregationActions;
use core_types::acknowledgement::{AcknowledgedTicket, AcknowledgedTicketStatus};
use log::warn;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};
use std::{
    fmt::{Display, Formatter},
    sync::Arc,
    time::Duration,
};
use utils_log::{debug, error};
use validator::Validate;

/// Configuration object for the `AggregatingStrategy`
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Validate, Serialize, Deserialize)]
pub struct AggregatingStrategyConfig {
    /// Number of acknowledged tickets in a channel that has to be exceeded to trigger
    /// the ticket aggregation in that channel.
    /// Default is 100.
    pub aggregation_threshold: u32,

    /// Maximum time to wait for the ticket aggregation to complete.
    /// This does not affect the runtime of the strategy `on_acknowledged_ticket` event processing.
    /// Default is 60 seconds.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub aggregation_timeout: Duration,

    /// Indicates whether to also perform ticket redemption after the ticket aggregation
    /// in a channel was successful.
    /// Default is `true`.
    pub redeem_after_aggregation: bool,
}

impl Default for AggregatingStrategyConfig {
    fn default() -> Self {
        Self {
            aggregation_threshold: 100,
            aggregation_timeout: Duration::from_secs(60),
            redeem_after_aggregation: true,
        }
    }
}

/// Represents a strategy that starts aggregating tickets in a certain
/// channel, once the amount of acknowledged tickets in that channel goes
/// above the given threshold.
/// Optionally, the strategy can also redeem the aggregated ticket, if the aggregation
/// was successful.
pub struct AggregatingStrategy<Db: HoprCoreEthereumDbActions, T, U> {
    db: Arc<RwLock<Db>>,
    tx_sender: TransactionSender,
    ticket_aggregator: Arc<Mutex<TicketAggregationActions<T, U>>>,
    cfg: AggregatingStrategyConfig,
}

impl<Db: HoprCoreEthereumDbActions, T, U> Display for AggregatingStrategy<Db, T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::Aggregating(Default::default()))
    }
}

impl<Db: HoprCoreEthereumDbActions, T, U> AggregatingStrategy<Db, T, U> {
    pub fn new(
        cfg: AggregatingStrategyConfig,
        db: Arc<RwLock<Db>>,
        tx_sender: TransactionSender,
        ticket_aggregator: TicketAggregationActions<T, U>,
    ) -> Self {
        Self {
            cfg,
            db,
            tx_sender,
            ticket_aggregator: Arc::new(Mutex::new(ticket_aggregator)),
        }
    }
}

#[async_trait(? Send)]
impl<Db: HoprCoreEthereumDbActions + 'static, T, U> SingularStrategy for AggregatingStrategy<Db, T, U> {
    async fn on_acknowledged_ticket(&self, ack: &AcknowledgedTicket) -> crate::errors::Result<()> {
        let channel_id = ack.ticket.channel_id;

        let channel = match self.db.read().await.get_channel(&channel_id).await? {
            Some(channel) => channel,
            None => {
                error!("{self} strategy: encountered {ack} in a non-existing channel!");
                return Err(ChannelDoesNotExist.into());
            }
        };

        let ack_tickets_in_db = self
            .db
            .read()
            .await
            .get_acknowledged_tickets(Some(channel.clone()))
            .await?;

        let being_aggregated_count = ack_tickets_in_db
            .iter()
            .filter(|ack| match ack.status {
                AcknowledgedTicketStatus::BeingAggregated { .. } => true,
                _ => false,
            })
            .count() as u32;

        if being_aggregated_count > 0 {
            debug!(
                "{self} strategy: {channel} already has ticket aggregation in progress (size {being_aggregated_count}), not aggregating yet"
            );
            return Ok(());
        }

        let acks_in_channel = ack_tickets_in_db
            .iter()
            .filter(|ack| ack.status == AcknowledgedTicketStatus::Untouched)
            .count() as u32;

        if acks_in_channel < self.cfg.aggregation_threshold {
            debug!(
                "{self} strategy: {channel} has {acks_in_channel} < {} ack tickets, not aggregating yet",
                self.cfg.aggregation_threshold
            );
            return Ok(());
        }

        let ticket_aggregation_awaiter = match self.ticket_aggregator.lock().await.aggregate_tickets(&channel_id) {
            Ok(mut awaiter) => awaiter.consume_and_wait(self.cfg.aggregation_timeout).await,
            Err(e) => {
                warn!("{self} could not aggregate tickets due to {e}");
                return Err(crate::errors::StrategyError::Other(e.to_string()));
            }
        };

        if let Err(e) = ticket_aggregation_awaiter {
            error!("{self} aggregation protocol failed due to {}", e.to_string());
            return Err(crate::errors::StrategyError::Other("ticket aggregation failed".into()));
        }

        if self.cfg.redeem_after_aggregation {
            match redeem_tickets_in_channel(self.db.clone(), &channel, true, self.tx_sender.clone()).await {
                Ok(_) => {
                    debug!("redeeming tickets");
                    // TODO: This is not necessary
                    // for result in futures::future::join_all(tx_result).await {
                    //     if let Err(e) = result {
                    //         error!("aggregating strategy: failed to redeem aggregated ticket {e}")
                    //     }
                    // }
                }
                Err(e) => {
                    error!("{self} could not submit transaction to redeem aggregated ticket {e}");
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::strategy::SingularStrategy;
    use async_std::sync::RwLock;
    use async_trait::async_trait;
    use core_crypto::{
        keypairs::{ChainKeypair, Keypair, OffchainKeypair},
        types::{Hash, Response},
    };
    use core_ethereum_actions::transaction_queue::{TransactionExecutor, TransactionQueue, TransactionResult};
    use core_ethereum_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
    use core_protocol::ticket_aggregation::processor::{TicketAggregationInteraction, TicketAggregationProcessed};
    use core_types::{
        acknowledgement::AcknowledgedTicket,
        channels::{generate_channel_id, ChannelEntry, Ticket},
    };
    use futures::StreamExt;
    use hex_literal::hex;
    use lazy_static::lazy_static;
    use mockall::mock;
    use std::sync::Arc;
    use utils_db::{constants::ACKNOWLEDGED_TICKETS_PREFIX, db::DB, rusty::RustyLevelDbShim};
    use utils_types::{
        primitives::{Address, Balance, BalanceType, Snapshot, U256},
        traits::{BinarySerializable, PeerIdLike},
    };

    lazy_static! {
        static ref PEERS: Vec<OffchainKeypair> = vec![
            hex!("b91a28ff9840e9c93e5fafd581131f0b9f33f3e61b02bf5dd83458aa0221f572"),
            hex!("82283757872f99541ce33a47b90c2ce9f64875abf08b5119a8a434b2fa83ea98")
        ]
        .iter()
        .map(|private| OffchainKeypair::from_secret(private).unwrap())
        .collect();
        static ref PEERS_CHAIN: Vec<ChainKeypair> = vec![
            hex!("51d3003d908045a4d76d0bfc0d84f6ff946b5934b7ea6a2958faf02fead4567a"),
            hex!("e1f89073a01831d0eed9fe2c67e7d65c144b9d9945320f6d325b1cccc2d124e9"),
        ]
        .iter()
        .map(|private| ChainKeypair::from_secret(private).unwrap())
        .collect();
    }

    mock! {
        TxExec { }
        #[async_trait(? Send)]
        impl TransactionExecutor for TxExec {
            async fn redeem_ticket(&self, ticket: AcknowledgedTicket) -> TransactionResult;
            async fn open_channel(&self, destination: Address, balance: Balance) -> TransactionResult;
            async fn fund_channel(&self, channel_id: Hash, amount: Balance) -> TransactionResult;
            async fn close_channel_initialize(&self, src: Address, dst: Address) -> TransactionResult;
            async fn close_channel_finalize(&self, src: Address, dst: Address) -> TransactionResult;
            async fn withdraw(&self, recipient: Address, amount: Balance) -> TransactionResult;
        }
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
            &Balance::new(price_per_packet.divide_f64(ticket_win_prob).unwrap(), BalanceType::HOPR),
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

    fn to_acknowledged_ticket_key(ack: &AcknowledgedTicket) -> utils_db::db::Key {
        let mut ack_key = Vec::new();

        ack_key.extend_from_slice(&ack.ticket.channel_id.to_bytes());
        ack_key.extend_from_slice(&ack.ticket.channel_epoch.to_be_bytes());
        ack_key.extend_from_slice(&ack.ticket.index.to_be_bytes());

        utils_db::db::Key::new_bytes_with_prefix(&ack_key, ACKNOWLEDGED_TICKETS_PREFIX).unwrap()
    }

    async fn init_dbs(inner_dbs: Vec<DB<RustyLevelDbShim>>) -> Vec<Arc<RwLock<CoreEthereumDb<RustyLevelDbShim>>>> {
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

    #[async_std::test]
    async fn test_strategy_aggregation() {
        // db_0: Alice (channel source)
        // db_1: Bob (channel destination)
        let mut inner_dbs = (0..2)
            .map(|_| DB::new(RustyLevelDbShim::new_in_memory()))
            .collect::<Vec<_>>();

        let mut acked_tickets: Vec<AcknowledgedTicket> = vec![];
        for i in 0..5 {
            let acked_ticket = mock_acknowledged_ticket(&PEERS_CHAIN[0], &PEERS_CHAIN[1], i);
            inner_dbs[1]
                .set(to_acknowledged_ticket_key(&acked_ticket), &acked_ticket)
                .await
                .unwrap();

            acked_tickets.push(acked_ticket);
        }

        let cfg = super::AggregatingStrategyConfig {
            aggregation_threshold: 5,
            aggregation_timeout: std::time::Duration::from_secs(5),
            redeem_after_aggregation: false,
        };

        let dbs = init_dbs(inner_dbs).await;

        let channel = ChannelEntry::new(
            (&PEERS_CHAIN[0]).into(),
            (&PEERS_CHAIN[1]).into(),
            Balance::new(1u64.into(), BalanceType::HOPR),
            6u64.into(),
            core_types::channels::ChannelStatus::Open,
            1u32.into(),
            0u64.into(),
        );

        dbs[1]
            .write()
            .await
            .update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        let mut alice = TicketAggregationInteraction::<(), ()>::new(dbs[0].clone(), &PEERS_CHAIN[0]);
        let mut bob = TicketAggregationInteraction::<(), ()>::new(dbs[1].clone(), &PEERS_CHAIN[1]);

        let (_tx, _awaiter) = futures::channel::oneshot::channel::<()>();
        let tx_exec = MockTxExec::new();

        // Start the TransactionQueue with the mock TransactionExecutor
        let tx_queue = TransactionQueue::new(dbs[1].clone(), Box::new(tx_exec));
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn_local(async move {
            tx_queue.transaction_loop().await;
        });

        let aggregation_strategy = super::AggregatingStrategy::new(cfg, dbs[1].clone(), tx_sender, bob.writer());

        let threshold_ticket = acked_tickets.last().unwrap();

        let ongoing_strategy_tick = aggregation_strategy.on_acknowledged_ticket(&threshold_ticket);

        futures::future::join(
            async move {
                assert!(ongoing_strategy_tick.await.is_ok());

                assert_eq!(
                    dbs[1]
                        .read()
                        .await
                        .get_acknowledged_tickets_range(
                            &channel.get_id(),
                            channel.channel_epoch.as_u32(),
                            0u64,
                            u64::MAX
                        )
                        .await
                        .unwrap()
                        .len(),
                    1
                )
            },
            Box::pin(async move {
                let mut finalizer = None;

                match bob.next().await {
                    Some(TicketAggregationProcessed::Send(_, acked_tickets, request_finalizer)) => {
                        let _ = finalizer.insert(request_finalizer);
                        alice
                            .writer()
                            .receive_aggregation_request(PEERS[1].public().to_peerid(), acked_tickets, ())
                            .unwrap();
                    }
                    //  alice.ack_event_queue.0.start_send(super::TicketAggregationToProcess::ToProcess(destination, acked_tickets)),
                    _ => panic!("unexpected action happened"),
                };

                match alice.next().await {
                    Some(TicketAggregationProcessed::Reply(_, aggregated_ticket, ())) => bob
                        .writer()
                        .receive_ticket(PEERS[0].public().to_peerid(), aggregated_ticket, ())
                        .unwrap(),
                    _ => panic!("unexpected action happened"),
                };

                match bob.next().await {
                    Some(TicketAggregationProcessed::Receive(_destination, ())) => (),
                    _ => panic!("unexpected action happened"),
                };

                finalizer.unwrap().finalize();
            }),
        )
        .await;
    }

    #[async_std::test]
    async fn test_strategy_aggregation_and_redemption() {
        // db_0: Alice (channel source)
        // db_1: Bob (channel destination)
        let mut inner_dbs = (0..2)
            .map(|_| DB::new(RustyLevelDbShim::new_in_memory()))
            .collect::<Vec<_>>();

        let mut acked_tickets: Vec<AcknowledgedTicket> = vec![];
        for i in 0..5 {
            let acked_ticket = mock_acknowledged_ticket(&PEERS_CHAIN[0], &PEERS_CHAIN[1], i);
            inner_dbs[1]
                .set(to_acknowledged_ticket_key(&acked_ticket), &acked_ticket)
                .await
                .unwrap();

            acked_tickets.push(acked_ticket);
        }

        let cfg = super::AggregatingStrategyConfig {
            aggregation_threshold: 5,
            aggregation_timeout: std::time::Duration::from_secs(5),
            redeem_after_aggregation: true,
        };

        let dbs = init_dbs(inner_dbs).await;

        let channel = ChannelEntry::new(
            (&PEERS_CHAIN[0]).into(),
            (&PEERS_CHAIN[1]).into(),
            Balance::new(1u64.into(), BalanceType::HOPR),
            6u64.into(),
            core_types::channels::ChannelStatus::Open,
            1u32.into(),
            0u64.into(),
        );

        dbs[1]
            .write()
            .await
            .update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        let mut alice = TicketAggregationInteraction::<(), ()>::new(dbs[0].clone(), &PEERS_CHAIN[0]);
        let mut bob = TicketAggregationInteraction::<(), ()>::new(dbs[1].clone(), &PEERS_CHAIN[1]);

        let (tx, awaiter) = futures::channel::oneshot::channel::<()>();
        let mut tx_exec = MockTxExec::new();

        let first_challenge = acked_tickets.first().unwrap().ticket.challenge.clone();

        tx_exec
            .expect_redeem_ticket()
            .times(1)
            .withf(
                move |ack| {
                    ack.ticket
                        .amount
                        .eq(&Balance::new(50000000000000000u128.into(), BalanceType::HOPR))
                        && ack.ticket.win_prob() == 1.0f64
                        && ack.ticket.challenge.eq(&first_challenge)
                        && ack.ticket.channel_epoch.eq(&1u32)
                        && ack.ticket.index.eq(&0u64)
                        && ack.ticket.index_offset.eq(&4u32)
                }, // signatures will be different, so we can't use .eq()
            )
            .return_once(move |_| {
                tx.send(()).unwrap();
                TransactionResult::RedeemTicket {
                    tx_hash: Hash::default(),
                }
            });

        // Start the TransactionQueue with the mock TransactionExecutor
        let tx_queue = TransactionQueue::new(dbs[1].clone(), Box::new(tx_exec));
        let tx_sender = tx_queue.new_sender();

        async_std::task::spawn_local(async move {
            tx_queue.transaction_loop().await;
        });

        let aggregation_strategy = super::AggregatingStrategy::new(cfg, dbs[1].clone(), tx_sender, bob.writer());

        let threshold_ticket = acked_tickets.last().unwrap();

        let ongoing_strategy_tick = aggregation_strategy.on_acknowledged_ticket(&threshold_ticket);

        futures::future::join(
            async move {
                assert!(ongoing_strategy_tick.await.is_ok());

                // TODO: not checking the redeemed value
                // assert_eq!(
                //     dbs[1]
                //         .read()
                //         .await
                //         .get_acknowledged_tickets_range(
                //             &channel.get_id(),
                //             channel.channel_epoch.as_u32(),
                //             0u64,
                //             u64::MAX
                //         )
                //         .await
                //         .unwrap()
                //         .len(),
                //     0,
                //     "all tickets redeemed"
                // )
            },
            Box::pin(async move {
                let mut finalizer = None;

                match bob.next().await {
                    Some(TicketAggregationProcessed::Send(_, acked_tickets, request_finalizer)) => {
                        let _ = finalizer.insert(request_finalizer);
                        alice
                            .writer()
                            .receive_aggregation_request(PEERS[1].public().to_peerid(), acked_tickets, ())
                            .unwrap();
                    }
                    //  alice.ack_event_queue.0.start_send(super::TicketAggregationToProcess::ToProcess(destination, acked_tickets)),
                    _ => panic!("unexpected action happened"),
                };

                match alice.next().await {
                    Some(TicketAggregationProcessed::Reply(_, aggregated_ticket, ())) => bob
                        .writer()
                        .receive_ticket(PEERS[0].public().to_peerid(), aggregated_ticket, ())
                        .unwrap(),
                    _ => panic!("unexpected action happened"),
                };

                match bob.next().await {
                    Some(TicketAggregationProcessed::Receive(_destination, ())) => (),
                    _ => panic!("unexpected action happened"),
                };

                finalizer.unwrap().finalize();
            }),
        )
        .await;

        awaiter.await.unwrap();
    }
}
