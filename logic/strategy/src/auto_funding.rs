use async_trait::async_trait;
use chain_actions::channels::ChannelActions;
use hopr_internal_types::channels::ChannelDirection::Outgoing;
use hopr_internal_types::channels::{ChannelChange, ChannelDirection, ChannelEntry, ChannelStatus};
use hopr_primitive_types::primitives::{Balance, BalanceType};
use log::info;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::fmt::{Debug, Display, Formatter};
use validator::Validate;

use crate::errors::StrategyError::CriteriaNotSatisfied;
use crate::strategy::SingularStrategy;
use crate::Strategy;

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::SimpleCounter;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_AUTO_FUNDINGS: SimpleCounter =
        SimpleCounter::new("hopr_strategy_auto_funding_funding_count", "Count of initiated automatic fundings").unwrap();
}

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
pub struct AutoFundingStrategy<A: ChannelActions> {
    chain_actions: A,
    cfg: AutoFundingStrategyConfig,
}

impl<A: ChannelActions> AutoFundingStrategy<A> {
    pub fn new(cfg: AutoFundingStrategyConfig, chain_actions: A) -> Self {
        Self { cfg, chain_actions }
    }
}

impl<A: ChannelActions> Debug for AutoFundingStrategy<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Strategy::AutoFunding(self.cfg))
    }
}

impl<A: ChannelActions> Display for AutoFundingStrategy<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::AutoFunding(self.cfg))
    }
}

#[async_trait]
impl<A: ChannelActions + Send + Sync> SingularStrategy for AutoFundingStrategy<A> {
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

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_COUNT_AUTO_FUNDINGS.increment();

                let rx = self
                    .chain_actions
                    .fund_channel(channel.get_id(), self.cfg.funding_amount)
                    .await?;
                std::mem::drop(rx); // The Receiver is not intentionally awaited here and the oneshot Sender can fail safely
                info!("issued re-staking of {channel} with {}", self.cfg.funding_amount);
            }
            Ok(())
        } else {
            Err(CriteriaNotSatisfied)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::auto_funding::{AutoFundingStrategy, AutoFundingStrategyConfig};
    use crate::strategy::SingularStrategy;
    use async_trait::async_trait;
    use chain_actions::action_queue::{ActionConfirmation, PendingAction};
    use chain_actions::channels::ChannelActions;
    use chain_types::actions::Action;
    use chain_types::chain_events::ChainEventType;
    use futures::{future::ok, FutureExt};
    use hopr_crypto_random::random_bytes;
    use hopr_crypto_types::types::Hash;
    use hopr_internal_types::channels::ChannelChange::CurrentBalance;
    use hopr_internal_types::channels::ChannelDirection::Outgoing;
    use hopr_internal_types::channels::{ChannelEntry, ChannelStatus};
    use hopr_primitive_types::primitives::{Address, Balance, BalanceType};
    use hopr_primitive_types::traits::BinarySerializable;
    use mockall::mock;

    mock! {
        ChannelAct { }
        #[async_trait]
        impl ChannelActions for ChannelAct {
            async fn open_channel(&self, destination: Address, amount: Balance) -> chain_actions::errors::Result<PendingAction>;
            async fn fund_channel(&self, channel_id: Hash, amount: Balance) -> chain_actions::errors::Result<PendingAction>;
            async fn close_channel(
                &self,
                counterparty: Address,
                direction: hopr_internal_types::channels::ChannelDirection,
                redeem_before_close: bool,
            ) -> chain_actions::errors::Result<PendingAction>;
        }
    }

    fn mock_action_confirmation(channel: ChannelEntry, balance: Balance) -> ActionConfirmation {
        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());
        ActionConfirmation {
            tx_hash: random_hash,
            event: Some(ChainEventType::ChannelBalanceIncreased(channel, balance)),
            action: Action::FundChannel(channel, balance),
        }
    }

    #[async_std::test]
    async fn test_auto_funding_strategy() {
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

        let c2 = ChannelEntry::new(
            Address::random(),
            Address::random(),
            Balance::new(5_u32.into(), BalanceType::HOPR),
            0_u32.into(),
            ChannelStatus::Open,
            0_u32.into(),
            0_u32.into(),
        );

        let c3 = ChannelEntry::new(
            Address::random(),
            Address::random(),
            Balance::new(5_u32.into(), BalanceType::HOPR),
            0_u32.into(),
            ChannelStatus::PendingToClose,
            0_u32.into(),
            0_u32.into(),
        );

        let mut actions = MockChannelAct::new();
        let fund_amount_c = fund_amount.clone();
        actions
            .expect_fund_channel()
            .times(1)
            .withf(move |h, balance| c2.get_id().eq(h) && fund_amount_c.eq(&balance))
            .return_once(move |_, _| Ok(ok(mock_action_confirmation(c2, fund_amount)).boxed()));

        let cfg = AutoFundingStrategyConfig {
            min_stake_threshold: stake_limit,
            funding_amount: fund_amount,
        };

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
    }
}
