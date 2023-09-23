use crate::strategy::SingularStrategy;
use crate::Strategy;
use async_std::sync::RwLock;
use async_trait::async_trait;
use core_ethereum_actions::channels::fund_channel;
use core_ethereum_actions::transaction_queue::TransactionSender;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::channels::{ChannelEntry, ChannelStatus};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use utils_log::info;
use utils_types::primitives::{Balance, BalanceType};
use validator::Validate;

/// Configuration for `AutoFundingStrategy`
#[derive(Debug, Clone, PartialEq, Eq, Validate, Serialize, Deserialize)]
pub struct AutoFundingStrategyConfig {
    /// Minimum stake that a channel's balance must not go below.
    /// Default is 0 HOPR, which never triggers funding.
    pub min_stake_threshold: Balance,
}

impl Default for AutoFundingStrategyConfig {
    fn default() -> Self {
        Self {
            min_stake_threshold: Balance::zero(BalanceType::HOPR),
        }
    }
}

/// The `AutoFundingStrategys` automatically funds channel that
/// dropped it's staked balance below the configured threshold.
pub struct AutoFundingStrategy<Db: HoprCoreEthereumDbActions> {
    tx_sender: TransactionSender,
    db: Arc<RwLock<Db>>,
    cfg: AutoFundingStrategyConfig,
}

impl<Db: HoprCoreEthereumDbActions> AutoFundingStrategy<Db> {
    pub fn new(cfg: AutoFundingStrategyConfig, db: Arc<RwLock<Db>>, tx_sender: TransactionSender) -> Self {
        Self { cfg, tx_sender, db }
    }
}

impl<Db: HoprCoreEthereumDbActions> Display for AutoFundingStrategy<Db> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::AutoFunding(Default::default()))
    }
}

#[async_trait(? Send)]
impl<Db: HoprCoreEthereumDbActions> SingularStrategy for AutoFundingStrategy<Db> {
    async fn on_channel_state_changed(&self, channel: &ChannelEntry) -> crate::errors::Result<()> {
        if channel.balance.lt(&self.cfg.min_stake_threshold) && channel.status == ChannelStatus::Open {
            info!(
                "{self} strategy: stake on {channel} is below treshhold {} < {}",
                channel.balance, self.cfg.min_stake_threshold
            );
            let to_stake = self.cfg.min_stake_threshold.sub(&channel.balance);
            let _ = fund_channel(
                self.db.clone(),
                self.tx_sender.clone(),
                channel.get_id(),
                to_stake.clone(),
            )
            .await?;
            info!("{self} strategy: issued re-staking of {channel} with {to_stake}");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::auto_funding::{AutoFundingStrategy, AutoFundingStrategyConfig};
    use crate::strategy::SingularStrategy;
    use async_std::sync::RwLock;
    use async_trait::async_trait;
    use core_crypto::types::Hash;
    use core_ethereum_actions::transaction_queue::{TransactionExecutor, TransactionQueue, TransactionResult};
    use core_ethereum_db::db::CoreEthereumDb;
    use core_ethereum_db::traits::HoprCoreEthereumDbActions;
    use core_types::acknowledgement::AcknowledgedTicket;
    use core_types::channels::{ChannelEntry, ChannelStatus};
    use mockall::mock;
    use std::sync::Arc;
    use utils_db::db::DB;
    use utils_db::rusty::RustyLevelDbShim;
    use utils_types::primitives::{Address, Balance, BalanceType, Snapshot};

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

    #[async_std::test]
    async fn test_auto_funding_strategy() {
        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            Address::random(),
        )));

        let stake_limit = Balance::new(7_u32.into(), BalanceType::HOPR);

        let c1 = ChannelEntry::new(
            Address::random(),
            Address::random(),
            Balance::new(10_u32.into(), BalanceType::HOPR),
            0_u32.into(),
            ChannelStatus::Open,
            0_u32.into(),
            0_u32.into(),
        );
        db.write()
            .await
            .update_channel_and_snapshot(&c1.get_id(), &c1, &Snapshot::default())
            .await
            .unwrap();

        let c2 = ChannelEntry::new(
            Address::random(),
            Address::random(),
            Balance::new(5_u32.into(), BalanceType::HOPR),
            0_u32.into(),
            ChannelStatus::Open,
            0_u32.into(),
            0_u32.into(),
        );
        db.write()
            .await
            .update_channel_and_snapshot(&c2.get_id(), &c2, &Snapshot::default())
            .await
            .unwrap();

        let c3 = ChannelEntry::new(
            Address::random(),
            Address::random(),
            Balance::new(5_u32.into(), BalanceType::HOPR),
            0_u32.into(),
            ChannelStatus::PendingToClose,
            0_u32.into(),
            0_u32.into(),
        );
        db.write()
            .await
            .update_channel_and_snapshot(&c3.get_id(), &c3, &Snapshot::default())
            .await
            .unwrap();

        db.write()
            .await
            .set_staking_safe_allowance(
                &Balance::new(10_000_000u64.into(), BalanceType::HOPR),
                &Snapshot::default(),
            )
            .await
            .unwrap();

        db.write()
            .await
            .set_channels_domain_separator(&Hash::default(), &Snapshot::default())
            .await
            .unwrap();

        let stake_limit_c = stake_limit.clone();
        let (tx, awaiter) = futures::channel::oneshot::channel::<()>();
        let mut tx_exec = MockTxExec::new();
        tx_exec
            .expect_fund_channel()
            .times(1)
            .withf(move |id, balance| c2.get_id().eq(id) && stake_limit_c.sub(&c2.balance).eq(&balance))
            .return_once(move |id, _| {
                if id.eq(&c2.get_id()) {
                    tx.send(()).unwrap();
                }
                TransactionResult::FundChannel {
                    tx_hash: Hash::default(),
                }
            });

        // Start the TransactionQueue with the mock TransactionExecutor
        let tx_queue = TransactionQueue::new(db.clone(), Box::new(tx_exec));
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn_local(async move {
            tx_queue.transaction_loop().await;
        });

        let cfg = AutoFundingStrategyConfig {
            min_stake_threshold: stake_limit,
        };

        let ars = AutoFundingStrategy::new(cfg, db.clone(), tx_sender);
        ars.on_channel_state_changed(&c1).await.unwrap();
        ars.on_channel_state_changed(&c2).await.unwrap();
        ars.on_channel_state_changed(&c3).await.unwrap();

        awaiter.await.unwrap();
    }
}
