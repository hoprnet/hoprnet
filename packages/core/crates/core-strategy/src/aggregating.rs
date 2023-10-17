use crate::{strategy::SingularStrategy, Strategy};
use async_std::sync::{Mutex, RwLock};
use async_trait::async_trait;
use core_ethereum_actions::errors::CoreEthereumActionsError::ChannelDoesNotExist;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_protocol::ticket_aggregation::processor::TicketAggregationActions;
use core_types::acknowledgement::{AcknowledgedTicket, AcknowledgedTicketStatus};
use core_types::channels::ChannelDirection::Incoming;
use core_types::channels::{ChannelChange, ChannelDirection, ChannelEntry, ChannelStatus};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};
use std::fmt::Debug;
use std::{
    fmt::{Display, Formatter},
    sync::Arc,
    time::Duration,
};
use utils_log::{debug, error, info, warn};
use utils_types::primitives::{Balance, BalanceType};
use validator::Validate;

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::spawn_local;

use core_ethereum_actions::redeem::TicketRedeemActions;
use core_ethereum_actions::CoreEthereumActions;
#[cfg(all(feature = "wasm", not(test)))]
use wasm_bindgen_futures::spawn_local;

/// Configuration object for the `AggregatingStrategy`
#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Validate, Serialize, Deserialize)]
pub struct AggregatingStrategyConfig {
    /// Number of acknowledged winning tickets in a channel that triggers the ticket aggregation
    /// in that channel when exceeded.
    /// This condition is independent of `unrealized_balance_ratio`.
    /// Default is 100.
    #[validate(range(min = 2))]
    pub aggregation_threshold: Option<u32>,

    /// Percentage of unrealized balance in a channel (relative to , that triggers the ticket aggregation
    /// in that channel when exceeded.
    /// This condition is independent of `aggregation_threshold`.
    /// Default is 0.9
    #[validate(range(min = 0_f32, max = 1.0_f32))]
    pub unrealized_balance_ratio: Option<f32>,

    /// Maximum time to wait for the ticket aggregation to complete.
    /// This does not affect the runtime of the strategy `on_acknowledged_ticket` event processing.
    /// Default is 60 seconds.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub aggregation_timeout: Duration,

    /// If set, the strategy will automatically aggregate tickets in channel that has transitioned
    /// to the `PendingToClose` state. This happens regardless if `aggregation_threshold`
    /// or `unrealized_balance_ratio` thresholds are met on that channel.
    /// If the aggregation on-close fails, the tickets are automatically sent for redeeming instead.
    /// Default is true.
    pub aggregate_on_channel_close: bool,
}

impl Default for AggregatingStrategyConfig {
    fn default() -> Self {
        Self {
            aggregation_threshold: Some(100),
            unrealized_balance_ratio: Some(0.9),
            aggregation_timeout: Duration::from_secs(60),
            aggregate_on_channel_close: true,
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
    chain_actions: CoreEthereumActions<Db>,
    ticket_aggregator: Arc<Mutex<TicketAggregationActions<T, U>>>,
    cfg: AggregatingStrategyConfig,
}

impl<Db, T, U> Debug for AggregatingStrategy<Db, T, U>
where
    Db: HoprCoreEthereumDbActions,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Strategy::Aggregating(self.cfg))
    }
}

impl<Db: HoprCoreEthereumDbActions, T, U> Display for AggregatingStrategy<Db, T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::Aggregating(self.cfg))
    }
}

impl<Db: HoprCoreEthereumDbActions, T, U> AggregatingStrategy<Db, T, U> {
    pub fn new(
        cfg: AggregatingStrategyConfig,
        db: Arc<RwLock<Db>>,
        chain_actions: CoreEthereumActions<Db>,
        ticket_aggregator: TicketAggregationActions<T, U>,
    ) -> Self {
        Self {
            cfg,
            db,
            chain_actions,
            ticket_aggregator: Arc::new(Mutex::new(ticket_aggregator)),
        }
    }
}

impl<Db: HoprCoreEthereumDbActions + 'static, T, U> AggregatingStrategy<Db, T, U> {
    async fn start_aggregation(&self, channel: ChannelEntry, redeem_if_failed: bool) -> crate::errors::Result<()> {
        debug!("{self} strategy: starting aggregation in {channel}");
        match self.ticket_aggregator.lock().await.aggregate_tickets(&channel) {
            Ok(mut awaiter) => {
                // Spawn waiting for the aggregation as a separate task
                let agg_timeout = self.cfg.aggregation_timeout;
                let actions_clone = self.chain_actions.clone();
                let strategy_id = self.to_string();

                spawn_local(async move {
                    match awaiter.consume_and_wait(agg_timeout).await {
                        Ok(_) => {
                            // The TicketAggregationActions will raise the on_acknowledged_ticket event,
                            // so the AutoRedeem strategy can take care of redeeming if needed
                            info!("{strategy_id} strategy: has completed ticket aggregation");
                        }
                        Err(e) => {
                            warn!("{strategy_id} strategy: could not aggregate tickets: {e}");
                            if redeem_if_failed {
                                info!("{strategy_id} strategy: initiating redemption of all tickets in {channel} after aggregation failure");

                                if let Err(e) = actions_clone.redeem_tickets_in_channel(&channel, false).await {
                                    error!("{strategy_id} strategy: failed to issue redeeming of all tickets in {channel}: {e}");
                                }

                                // We do not need to await the redemption completion of all the tickets
                            }
                        }
                    }
                });

                Ok(())
            }
            Err(e) => {
                warn!("{self} strategy: could not initiate aggregate tickets due to {e}");
                Err(crate::errors::StrategyError::Other("ticket aggregation failed".into()))
            }
        }
    }
}

#[async_trait(? Send)]
impl<Db: HoprCoreEthereumDbActions + 'static, T, U> SingularStrategy for AggregatingStrategy<Db, T, U> {
    async fn on_acknowledged_winning_ticket(&self, ack: &AcknowledgedTicket) -> crate::errors::Result<()> {
        let channel_id = ack.ticket.channel_id;

        let channel = match self.db.read().await.get_channel(&channel_id).await? {
            Some(channel) => channel,
            None => {
                error!("{self} strategy: encountered {ack} in a non-existing channel!");
                return Err(ChannelDoesNotExist.into());
            }
        };

        let ack_tickets_in_db = self.db.read().await.get_acknowledged_tickets(Some(channel)).await?;

        let mut aggregatable_tickets = 0;
        let mut unredeemed_value = Balance::zero(BalanceType::HOPR);

        for ticket in ack_tickets_in_db.iter() {
            match ticket.status {
                AcknowledgedTicketStatus::Untouched => {
                    aggregatable_tickets += 1;
                    unredeemed_value = unredeemed_value.add(&ticket.ticket.amount);
                }
                AcknowledgedTicketStatus::BeingAggregated { .. } => {
                    debug!(
                        "{self} strategy: {channel} already has ticket aggregation in progress, not aggregating yet"
                    );
                    return Ok(());
                }
                AcknowledgedTicketStatus::BeingRedeemed { .. } => {}
            }
        }

        let mut can_aggregate = false;

        // Check the aggregation threshold
        if let Some(agg_threshold) = self.cfg.aggregation_threshold {
            if aggregatable_tickets >= agg_threshold {
                info!("{self} strategy: {channel} has {aggregatable_tickets} >= {agg_threshold} ack tickets");
                can_aggregate = true;
            } else {
                debug!("{self} strategy: {channel} has {aggregatable_tickets} < {agg_threshold} ack tickets, not aggregating yet");
            }
        }
        if let Some(unrealized_threshold) = self.cfg.unrealized_balance_ratio {
            let diminished_balance = channel.balance.mul_f64(unrealized_threshold as f64);

            // Trigger aggregation if unrealized balance greater or equal to X percent of the current balance
            // and there are at least two tickets
            if unredeemed_value.gte(&diminished_balance) {
                if aggregatable_tickets > 1 {
                    info!("{self} strategy: {channel} has unrealized balance {unredeemed_value} >= {diminished_balance} in {aggregatable_tickets} tickets");
                    can_aggregate = true;
                } else {
                    debug!("{self} strategy: {channel} has unrealized balance {unredeemed_value} >= {diminished_balance} but in just {aggregatable_tickets} tickets, not aggregating yet");
                }
            } else {
                debug!(
                    "{self} strategy: {channel} has unrealized balance {unredeemed_value} < {diminished_balance} in {aggregatable_tickets} tickets, not aggregating yet"
                );
            }
        }

        // Proceed with aggregation
        if can_aggregate {
            self.start_aggregation(channel, false).await
        } else {
            debug!("{self} strategy: channel {channel_id} has not met the criteria for aggregation");
            Ok(())
        }
    }

    async fn on_own_channel_changed(
        &self,
        channel: &ChannelEntry,
        direction: ChannelDirection,
        change: ChannelChange,
    ) -> crate::errors::Result<()> {
        if !self.cfg.aggregate_on_channel_close || direction != Incoming {
            return Ok(());
        }

        if let ChannelChange::Status { left: old, right: new } = change {
            if old != ChannelStatus::Open || new != ChannelStatus::PendingToClose {
                debug!("{self} strategy: ignoring channel {channel} state change that's not in PendingToClose");
                return Ok(());
            }

            let ack_tickets_in_db = self.db.read().await.get_acknowledged_tickets(Some(*channel)).await?;

            let mut aggregatable_tickets = 0;

            for ticket in ack_tickets_in_db.iter() {
                match ticket.status {
                    AcknowledgedTicketStatus::Untouched => {
                        aggregatable_tickets += 1;
                    }
                    AcknowledgedTicketStatus::BeingAggregated { .. } => {
                        debug!(
                        "{self} strategy: {channel} already has ticket aggregation in progress, not aggregating yet"
                    );
                        return Ok(());
                    }
                    AcknowledgedTicketStatus::BeingRedeemed { .. } => {}
                }
            }

            if aggregatable_tickets > 1 {
                debug!("{self} strategy: {channel} has {aggregatable_tickets} aggregatable tickets");
                self.start_aggregation(*channel, true).await
            } else {
                debug!("{self} strategy: closing {channel} does not have more than 1 tickets to aggregate");
                Ok(())
            }
        } else {
            Ok(())
        }
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
    use core_ethereum_actions::transaction_queue::{
        TransactionExecutor, TransactionQueue, TransactionResult, TransactionSender,
    };
    use core_ethereum_actions::CoreEthereumActions;
    use core_ethereum_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
    use core_protocol::ticket_aggregation::processor::{
        TicketAggregationActions, TicketAggregationInteraction, TicketAggregationProcessed,
    };
    use core_types::channels::ChannelDirection::Incoming;
    use core_types::channels::{ChannelChange, ChannelStatus};
    use core_types::{
        acknowledgement::AcknowledgedTicket,
        channels::{generate_channel_id, ChannelEntry, Ticket},
    };
    use futures::channel::oneshot::Receiver;
    use futures::{FutureExt, StreamExt};
    use hex_literal::hex;
    use lazy_static::lazy_static;
    use mockall::mock;
    use std::pin::pin;
    use std::sync::Arc;
    use std::time::Duration;
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
            async fn fund_channel(&self, destination: Address, amount: Balance) -> TransactionResult;
            async fn close_channel_initialize(&self, src: Address, dst: Address) -> TransactionResult;
            async fn close_channel_finalize(&self, src: Address, dst: Address) -> TransactionResult;
            async fn withdraw(&self, recipient: Address, amount: Balance) -> TransactionResult;
        }
    }

    fn mock_acknowledged_ticket(signer: &ChainKeypair, destination: &ChainKeypair, index: u64) -> AcknowledgedTicket {
        let price_per_packet: U256 = 20_u32.into();
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

    async fn populate_db_with_ack_tickets(db: &mut DB<RustyLevelDbShim>, amount: usize) -> Vec<AcknowledgedTicket> {
        let mut acked_tickets = Vec::new();
        for i in 0..amount {
            let acked_ticket = mock_acknowledged_ticket(&PEERS_CHAIN[0], &PEERS_CHAIN[1], i as u64);
            db.set(to_acknowledged_ticket_key(&acked_ticket), &acked_ticket)
                .await
                .unwrap();

            acked_tickets.push(acked_ticket);
        }
        acked_tickets
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

    fn spawn_aggregation_interaction<Db: HoprCoreEthereumDbActions + 'static>(
        db_alice: Arc<RwLock<Db>>,
        db_bob: Arc<RwLock<Db>>,
        key_alice: &ChainKeypair,
        key_bob: &ChainKeypair,
    ) -> (TicketAggregationActions<(), ()>, Receiver<()>) {
        let mut alice = TicketAggregationInteraction::<(), ()>::new(db_alice, key_alice);
        let mut bob = TicketAggregationInteraction::<(), ()>::new(db_bob, key_bob);

        let (tx, awaiter) = futures::channel::oneshot::channel::<()>();
        let bob_aggregator = bob.writer();

        async_std::task::spawn_local(async move {
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
                Some(TicketAggregationProcessed::Receive(_destination, _ticket, ())) => (),
                _ => panic!("unexpected action happened"),
            };

            finalizer.unwrap().finalize();
            let _ = tx.send(());
        });

        return (bob_aggregator, awaiter);
    }

    fn spawn_tx_queue<Db: HoprCoreEthereumDbActions + 'static>(
        db: Arc<RwLock<Db>>,
        tx_exec: MockTxExec,
    ) -> TransactionSender {
        let tx_queue = TransactionQueue::new(db, Box::new(tx_exec));
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn_local(async move {
            tx_queue.transaction_loop().await;
        });

        tx_sender
    }

    #[async_std::test]
    async fn test_strategy_aggregation_on_ack() {
        let _ = env_logger::builder().is_test(true).try_init();

        // db_0: Alice (channel source)
        // db_1: Bob (channel destination)
        let mut inner_dbs = (0..2)
            .map(|_| DB::new(RustyLevelDbShim::new_in_memory()))
            .collect::<Vec<_>>();

        let acked_tickets = populate_db_with_ack_tickets(&mut inner_dbs[1], 5).await;

        let dbs = init_dbs(inner_dbs).await;

        let channel = ChannelEntry::new(
            (&PEERS_CHAIN[0]).into(),
            (&PEERS_CHAIN[1]).into(),
            Balance::new(1u64.into(), BalanceType::HOPR),
            6u64.into(),
            ChannelStatus::Open,
            1u32.into(),
            0u64.into(),
        );

        dbs[1]
            .write()
            .await
            .update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        let tx_sender = spawn_tx_queue(dbs[1].clone(), MockTxExec::new());
        let (bob_aggregator, awaiter) =
            spawn_aggregation_interaction(dbs[0].clone(), dbs[1].clone(), &PEERS_CHAIN[0], &PEERS_CHAIN[1]);

        let cfg = super::AggregatingStrategyConfig {
            aggregation_threshold: Some(5),
            unrealized_balance_ratio: None,
            aggregation_timeout: std::time::Duration::from_secs(5),
            aggregate_on_channel_close: false,
        };

        let actions = CoreEthereumActions::new(PEERS_CHAIN[1].public().to_address(), dbs[1].clone(), tx_sender.clone());

        let aggregation_strategy = super::AggregatingStrategy::new(cfg, dbs[1].clone(), actions, bob_aggregator);

        let threshold_ticket = acked_tickets.last().unwrap();
        aggregation_strategy
            .on_acknowledged_winning_ticket(&threshold_ticket)
            .await
            .expect("strategy call should succeed");

        // Wait until aggregation has finished
        let f1 = pin!(awaiter);
        let f2 = pin!(async_std::task::sleep(Duration::from_secs(5)).fuse());
        let _ = futures::future::select(f1, f2).await;

        assert_eq!(
            dbs[1]
                .read()
                .await
                .get_acknowledged_tickets_range(&channel.get_id(), channel.channel_epoch.as_u32(), 0u64, u64::MAX)
                .await
                .unwrap()
                .len(),
            1,
            "there should be a single aggregated ticket"
        );
    }

    #[async_std::test]
    async fn test_strategy_aggregation_on_ack_when_unrealized_balance_exceeded() {
        let _ = env_logger::builder().is_test(true).try_init();

        // db_0: Alice (channel source)
        // db_1: Bob (channel destination)
        let mut inner_dbs = (0..2)
            .map(|_| DB::new(RustyLevelDbShim::new_in_memory()))
            .collect::<Vec<_>>();

        let acked_tickets = populate_db_with_ack_tickets(&mut inner_dbs[1], 4).await;

        let dbs = init_dbs(inner_dbs).await;

        let channel = ChannelEntry::new(
            (&PEERS_CHAIN[0]).into(),
            (&PEERS_CHAIN[1]).into(),
            Balance::new(100u64.into(), BalanceType::HOPR),
            6u64.into(),
            ChannelStatus::Open,
            1u32.into(),
            0u64.into(),
        );

        dbs[1]
            .write()
            .await
            .update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        let tx_sender = spawn_tx_queue(dbs[1].clone(), MockTxExec::new());
        let (bob_aggregator, awaiter) =
            spawn_aggregation_interaction(dbs[0].clone(), dbs[1].clone(), &PEERS_CHAIN[0], &PEERS_CHAIN[1]);

        let cfg = super::AggregatingStrategyConfig {
            aggregation_threshold: None,
            unrealized_balance_ratio: Some(0.75),
            aggregation_timeout: Duration::from_secs(5),
            aggregate_on_channel_close: false,
        };

        let actions = CoreEthereumActions::new(PEERS_CHAIN[1].public().to_address(), dbs[1].clone(), tx_sender.clone());

        let aggregation_strategy = super::AggregatingStrategy::new(cfg, dbs[1].clone(), actions, bob_aggregator);

        let threshold_ticket = acked_tickets.last().unwrap();

        aggregation_strategy
            .on_acknowledged_winning_ticket(&threshold_ticket)
            .await
            .expect("strategy call should succeed");

        // Wait until aggregation has finished
        let f1 = pin!(awaiter);
        let f2 = pin!(async_std::task::sleep(Duration::from_secs(5)));
        let _ = futures::future::select(f1, f2).await;

        assert_eq!(
            dbs[1]
                .read()
                .await
                .get_acknowledged_tickets_range(&channel.get_id(), channel.channel_epoch.as_u32(), 0u64, u64::MAX)
                .await
                .unwrap()
                .len(),
            1,
            "there should be a single aggregated ticket"
        );
    }

    #[async_std::test]
    async fn test_strategy_aggregation_on_channel_close() {
        let _ = env_logger::builder().is_test(true).try_init();

        // db_0: Alice (channel source)
        // db_1: Bob (channel destination)
        let mut inner_dbs = (0..2)
            .map(|_| DB::new(RustyLevelDbShim::new_in_memory()))
            .collect::<Vec<_>>();

        populate_db_with_ack_tickets(&mut inner_dbs[1], 5).await;

        let cfg = super::AggregatingStrategyConfig {
            aggregation_threshold: None,
            unrealized_balance_ratio: None,
            aggregation_timeout: Duration::from_secs(5),
            aggregate_on_channel_close: true,
        };

        let dbs = init_dbs(inner_dbs).await;

        let mut channel = ChannelEntry::new(
            (&PEERS_CHAIN[0]).into(),
            (&PEERS_CHAIN[1]).into(),
            Balance::new(1u64.into(), BalanceType::HOPR),
            6u64.into(),
            ChannelStatus::Open,
            1u32.into(),
            0u64.into(),
        );

        dbs[1]
            .write()
            .await
            .update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        let tx_sender = spawn_tx_queue(dbs[1].clone(), MockTxExec::new());
        let (bob_aggregator, awaiter) =
            spawn_aggregation_interaction(dbs[0].clone(), dbs[1].clone(), &PEERS_CHAIN[0], &PEERS_CHAIN[1]);

        let actions = CoreEthereumActions::new(PEERS_CHAIN[1].public().to_address(), dbs[1].clone(), tx_sender.clone());

        let aggregation_strategy = super::AggregatingStrategy::new(cfg, dbs[1].clone(), actions, bob_aggregator);

        channel.status = ChannelStatus::PendingToClose;
        dbs[1]
            .write()
            .await
            .update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        aggregation_strategy
            .on_own_channel_changed(
                &channel,
                Incoming,
                ChannelChange::Status {
                    left: ChannelStatus::Open,
                    right: ChannelStatus::PendingToClose,
                },
            )
            .await
            .expect("strategy call should not fail");

        // Wait until aggregation has finished
        let f1 = pin!(awaiter);
        let f2 = pin!(async_std::task::sleep(Duration::from_secs(5)));
        let _ = futures::future::select(f1, f2).await;

        assert_eq!(
            dbs[1]
                .read()
                .await
                .get_acknowledged_tickets_range(&channel.get_id(), channel.channel_epoch.as_u32(), 0u64, u64::MAX)
                .await
                .unwrap()
                .len(),
            1,
            "there should be a single aggregated ticket"
        );
    }
}
