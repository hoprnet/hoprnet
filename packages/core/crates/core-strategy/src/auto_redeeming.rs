use crate::strategy::SingularStrategy;
use crate::Strategy;
use async_std::sync::RwLock;
use async_trait::async_trait;
use core_ethereum_actions::redeem::redeem_ticket;
use core_ethereum_actions::transaction_queue::TransactionSender;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::acknowledgement::AcknowledgedTicket;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use utils_log::info;
use validator::Validate;

/// Configuration object for the `AutoRedeemingStrategy`
#[derive(Debug, Clone, PartialEq, Eq, Validate, Serialize, Deserialize)]
pub struct AutoRedeemingStrategyConfig {
    /// If set, the strategy will redeem only aggregated tickets.
    /// Defaults to true.
    pub redeem_only_aggregated: bool,
}

impl Default for AutoRedeemingStrategyConfig {
    fn default() -> Self {
        Self {
            redeem_only_aggregated: false,
        }
    }
}

/// The `AutoRedeemingStrategy` automatically sends an acknowledged ticket
/// for redemption once encountered.
/// The strategy does not await the result of the redemption.
pub struct AutoRedeemingStrategy<Db: HoprCoreEthereumDbActions> {
    db: Arc<RwLock<Db>>,
    tx_sender: TransactionSender,
    #[allow(dead_code)]
    cfg: AutoRedeemingStrategyConfig,
}

impl<Db: HoprCoreEthereumDbActions> Display for AutoRedeemingStrategy<Db> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::AutoRedeeming(Default::default()))
    }
}

impl<Db: HoprCoreEthereumDbActions> AutoRedeemingStrategy<Db> {
    pub fn new(cfg: AutoRedeemingStrategyConfig, db: Arc<RwLock<Db>>, tx_sender: TransactionSender) -> Self {
        Self { cfg, db, tx_sender }
    }
}

#[async_trait(? Send)]
impl<Db: HoprCoreEthereumDbActions + 'static> SingularStrategy for AutoRedeemingStrategy<Db> {
    async fn on_acknowledged_ticket(&self, ack: &AcknowledgedTicket) -> crate::errors::Result<()> {
        if !self.cfg.redeem_only_aggregated || ack.ticket.is_aggregated() {
            info!("{self} strategy: auto-redeeming {ack}");
            let _ = redeem_ticket(self.db.clone(), ack.clone(), self.tx_sender.clone()).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::auto_redeeming::{AutoRedeemingStrategy, AutoRedeemingStrategyConfig};
    use crate::strategy::SingularStrategy;
    use async_std::sync::RwLock;
    use async_trait::async_trait;
    use core_crypto::keypairs::{ChainKeypair, Keypair};
    use core_crypto::types::{Challenge, CurvePoint, HalfKey, Hash};
    use core_ethereum_actions::transaction_queue::{TransactionExecutor, TransactionQueue, TransactionResult};
    use core_ethereum_db::db::CoreEthereumDb;
    use core_ethereum_db::traits::HoprCoreEthereumDbActions;
    use core_types::acknowledgement::{AcknowledgedTicket, UnacknowledgedTicket};
    use core_types::channels::Ticket;
    use hex_literal::hex;
    use mockall::mock;
    use std::sync::Arc;
    use utils_db::constants::ACKNOWLEDGED_TICKETS_PREFIX;
    use utils_db::db::DB;
    use utils_db::rusty::RustyLevelDbShim;
    use utils_types::primitives::{Address, Balance, BalanceType, Snapshot, U256};
    use utils_types::traits::BinarySerializable;

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).unwrap();
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).unwrap();
    }

    fn generate_random_ack_ticket(idx_offset: u32) -> AcknowledgedTicket {
        let counterparty = &BOB;
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();

        let cp1: CurvePoint = hk1.to_challenge().into();
        let cp2: CurvePoint = hk2.to_challenge().into();
        let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

        let price_per_packet: U256 = 10000000000000000u128.into(); // 0.01 HOPR

        let ticket = Ticket::new(
            &ALICE.public().to_address(),
            &Balance::new(
                price_per_packet.divide_f64(1.0f64).unwrap() * 5u64.into(),
                BalanceType::HOPR,
            ),
            0_u32.into(),
            idx_offset.into(),
            1.0f64,
            4u64.into(),
            Challenge::from(cp_sum).to_ethereum_challenge(),
            counterparty,
            &Hash::default(),
        )
        .unwrap();

        let unacked_ticket = UnacknowledgedTicket::new(ticket, hk1, counterparty.public().to_address());
        unacked_ticket.acknowledge(&hk2, &ALICE, &Hash::default()).unwrap()
    }

    fn to_acknowledged_ticket_key(ack: &AcknowledgedTicket) -> utils_db::db::Key {
        let mut ack_key = Vec::new();

        ack_key.extend_from_slice(&ack.ticket.channel_id.to_bytes());
        ack_key.extend_from_slice(&ack.ticket.channel_epoch.to_be_bytes());
        ack_key.extend_from_slice(&ack.ticket.index.to_be_bytes());

        utils_db::db::Key::new_bytes_with_prefix(&ack_key, ACKNOWLEDGED_TICKETS_PREFIX).unwrap()
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

    #[async_std::test]
    async fn test_auto_redeeming_strategy_redeem() {
        let ack_ticket = generate_random_ack_ticket(1);

        let mut inner_db = DB::new(RustyLevelDbShim::new_in_memory());
        inner_db
            .set(to_acknowledged_ticket_key(&ack_ticket), &ack_ticket)
            .await
            .unwrap();

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(inner_db, ALICE.public().to_address())));

        db.write()
            .await
            .set_channels_domain_separator(&Hash::default(), &Snapshot::default())
            .await
            .unwrap();

        let ack_clone = ack_ticket.clone();

        let (tx, awaiter) = futures::channel::oneshot::channel::<()>();
        let mut tx_exec = MockTxExec::new();
        tx_exec
            .expect_redeem_ticket()
            .times(1)
            .withf(move |ack| ack_clone.ticket.eq(&ack.ticket))
            .return_once(move |_| {
                tx.send(()).unwrap();
                TransactionResult::RedeemTicket {
                    tx_hash: Hash::default(),
                }
            });

        // Start the TransactionQueue with the mock TransactionExecutor
        let tx_queue = TransactionQueue::new(db.clone(), Box::new(tx_exec));
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn_local(async move {
            tx_queue.transaction_loop().await;
        });

        let cfg = AutoRedeemingStrategyConfig {
            redeem_only_aggregated: false,
        };

        let ars = AutoRedeemingStrategy::new(cfg, db.clone(), tx_sender);
        ars.on_acknowledged_ticket(&ack_ticket).await.unwrap();

        awaiter.await.unwrap();
    }

    #[async_std::test]
    async fn test_auto_redeeming_strategy_redeem_agg_only() {
        let ack_ticket_unagg = generate_random_ack_ticket(1);
        let ack_ticket_agg = generate_random_ack_ticket(3);

        let mut inner_db = DB::new(RustyLevelDbShim::new_in_memory());
        inner_db
            .set(to_acknowledged_ticket_key(&ack_ticket_unagg), &ack_ticket_unagg)
            .await
            .unwrap();
        inner_db
            .set(to_acknowledged_ticket_key(&ack_ticket_agg), &ack_ticket_agg)
            .await
            .unwrap();

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(inner_db, ALICE.public().to_address())));

        db.write()
            .await
            .set_channels_domain_separator(&Hash::default(), &Snapshot::default())
            .await
            .unwrap();

        let ack_clone_agg = ack_ticket_agg.clone();

        let (tx, awaiter) = futures::channel::oneshot::channel::<()>();
        let mut tx_exec = MockTxExec::new();
        tx_exec
            .expect_redeem_ticket()
            .times(1)
            .withf(move |ack| ack_clone_agg.ticket.eq(&ack.ticket))
            .return_once(move |_| {
                tx.send(()).unwrap();
                TransactionResult::RedeemTicket {
                    tx_hash: Hash::default(),
                }
            });

        // Start the TransactionQueue with the mock TransactionExecutor
        let tx_queue = TransactionQueue::new(db.clone(), Box::new(tx_exec));
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn_local(async move {
            tx_queue.transaction_loop().await;
        });

        let cfg = AutoRedeemingStrategyConfig {
            redeem_only_aggregated: true,
        };

        let ars = AutoRedeemingStrategy::new(cfg, db.clone(), tx_sender);
        ars.on_acknowledged_ticket(&ack_ticket_unagg).await.unwrap();
        ars.on_acknowledged_ticket(&ack_ticket_agg).await.unwrap();

        awaiter.await.unwrap();
    }
}
