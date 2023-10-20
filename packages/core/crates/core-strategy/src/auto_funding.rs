use async_trait::async_trait;
use core_ethereum_actions::channels::ChannelActions;
use core_ethereum_actions::CoreEthereumActions;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::channels::ChannelDirection::Outgoing;
use core_types::channels::{ChannelChange, ChannelDirection, ChannelEntry, ChannelStatus};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::fmt::{Debug, Display, Formatter};
use utils_log::info;
use utils_types::primitives::{Balance, BalanceType};
use validator::Validate;

use crate::strategy::SingularStrategy;
use crate::Strategy;

/// Configuration for `AutoFundingStrategy`
#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Validate, Serialize, Deserialize)]
pub struct AutoFundingStrategyConfig {
    /// Minimum stake that a channel's balance must not go below.
    /// Default is 1 HOPR
    #[serde_as(as = "DisplayFromStr")]
    pub min_stake_threshold: Balance,

    /// Funding amount.
    /// Defaults to 10 HOPR.
    #[serde_as(as = "DisplayFromStr")]
    pub funding_amount: Balance,
}

impl Default for AutoFundingStrategyConfig {
    fn default() -> Self {
        Self {
            min_stake_threshold: Balance::new_from_str("1000000000000000000", BalanceType::HOPR),
            funding_amount: Balance::new_from_str("10000000000000000000", BalanceType::HOPR),
        }
    }
}

/// The `AutoFundingStrategy` automatically funds channel that
/// dropped it's staked balance below the configured threshold.
pub struct AutoFundingStrategy<Db: HoprCoreEthereumDbActions + Clone> {
    chain_actions: CoreEthereumActions<Db>,
    cfg: AutoFundingStrategyConfig,
}

impl<Db: HoprCoreEthereumDbActions + Clone> AutoFundingStrategy<Db> {
    pub fn new(cfg: AutoFundingStrategyConfig, chain_actions: CoreEthereumActions<Db>) -> Self {
        Self { cfg, chain_actions }
    }
}

impl<Db: HoprCoreEthereumDbActions + Clone> Debug for AutoFundingStrategy<Db> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Strategy::AutoFunding(self.cfg))
    }
}

impl<Db: HoprCoreEthereumDbActions + Clone> Display for AutoFundingStrategy<Db> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::AutoFunding(self.cfg))
    }
}

#[async_trait(? Send)]
impl<Db: HoprCoreEthereumDbActions + Clone> SingularStrategy for AutoFundingStrategy<Db> {
    async fn on_own_channel_changed(
        &self,
        channel: &ChannelEntry,
        direction: ChannelDirection,
        change: ChannelChange,
    ) -> crate::errors::Result<()> {
        // Can only auto-fund outgoing channels
        if direction != Outgoing {
            return Ok(());
        }

        if let ChannelChange::CurrentBalance { right: new, .. } = change {
            if new.lt(&self.cfg.min_stake_threshold) && channel.status == ChannelStatus::Open {
                info!(
                    "stake on {channel} is below threshold {} < {}",
                    channel.balance, self.cfg.min_stake_threshold
                );

                let rx = self
                    .chain_actions
                    .fund_channel(channel.get_id(), self.cfg.funding_amount)
                    .await?;
                std::mem::drop(rx); // The Receiver is not intentionally awaited here and the oneshot Sender can fail safely
                info!("issued re-staking of {channel} with {}", self.cfg.funding_amount);
            }
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
    use core_ethereum_actions::CoreEthereumActions;
    use core_ethereum_db::db::CoreEthereumDb;
    use core_ethereum_db::traits::HoprCoreEthereumDbActions;
    use core_types::acknowledgement::AcknowledgedTicket;
    use core_types::channels::ChannelChange::CurrentBalance;
    use core_types::channels::ChannelDirection::Outgoing;
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
            async fn fund_channel(&self, destination: Address, amount: Balance) -> TransactionResult;
            async fn close_channel_initialize(&self, src: Address, dst: Address) -> TransactionResult;
            async fn close_channel_finalize(&self, src: Address, dst: Address) -> TransactionResult;
            async fn withdraw(&self, recipient: Address, amount: Balance) -> TransactionResult;
        }
    }

    #[async_std::test]
    async fn test_auto_funding_strategy() {
        let self_addr = Address::random();
        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            self_addr,
        )));

        let stake_limit = Balance::new(7_u32.into(), BalanceType::HOPR);
        let fund_amount = Balance::new(5_u32.into(), BalanceType::HOPR);

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

        let fund_amount_c = fund_amount.clone();
        let (tx, awaiter) = futures::channel::oneshot::channel::<()>();
        let mut tx_exec = MockTxExec::new();
        tx_exec
            .expect_fund_channel()
            .times(1)
            .withf(move |dst, balance| c2.destination.eq(dst) && fund_amount_c.eq(&balance))
            .return_once(move |dst, _| {
                if dst.eq(&c2.destination) {
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
            funding_amount: fund_amount,
        };

        let actions = CoreEthereumActions::new(self_addr, db.clone(), tx_sender);

        let afs = AutoFundingStrategy::new(cfg, actions);
        afs.on_own_channel_changed(
            &c1,
            Outgoing,
            CurrentBalance {
                left: Balance::zero(BalanceType::HOPR),
                right: c1.balance,
            },
        )
        .await
        .unwrap();
        afs.on_own_channel_changed(
            &c2,
            Outgoing,
            CurrentBalance {
                left: Balance::zero(BalanceType::HOPR),
                right: c2.balance,
            },
        )
        .await
        .unwrap();
        afs.on_own_channel_changed(
            &c3,
            Outgoing,
            CurrentBalance {
                left: Balance::zero(BalanceType::HOPR),
                right: c3.balance,
            },
        )
        .await
        .unwrap();

        awaiter.await.unwrap();
    }
}
