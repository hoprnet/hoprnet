//! ## Auto Funding Strategy
//! This strategy listens for channel state change events to check whether a channel has dropped below
//! `min_stake_threshold` HOPR. If this happens, the strategy issues a **fund channel** transaction to re-stake the
//! channel with `funding_amount` HOPR.
//!
//! For details on default parameters see [AutoFundingStrategyConfig].
use std::fmt::{Debug, Display, Formatter};

use async_trait::async_trait;
use hopr_api::chain::ChainWriteChannelOperations;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use tracing::info;
use validator::Validate;

use crate::{
    Strategy,
    errors::{StrategyError, StrategyError::CriteriaNotSatisfied},
    strategy::SingularStrategy,
};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_AUTO_FUNDINGS: hopr_metrics::SimpleCounter =
        hopr_metrics::SimpleCounter::new("hopr_strategy_auto_funding_funding_count", "Count of initiated automatic fundings").unwrap();
}

/// Configuration for `AutoFundingStrategy`
#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct AutoFundingStrategyConfig {
    /// Minimum stake that a channel's balance must not go below.
    ///
    /// Default is 1 wxHOPR
    #[serde_as(as = "DisplayFromStr")]
    #[default(HoprBalance::new_base(1))]
    pub min_stake_threshold: HoprBalance,

    /// Funding amount.
    ///
    /// Defaults to 10 wxHOPR.
    #[serde_as(as = "DisplayFromStr")]
    #[default(HoprBalance::new_base(10))]
    pub funding_amount: HoprBalance,
}

/// The `AutoFundingStrategy` automatically funds a channel that
/// dropped it's staked balance below the configured threshold.
pub struct AutoFundingStrategy<A> {
    hopr_chain_actions: A,
    cfg: AutoFundingStrategyConfig,
}

impl<A: ChainWriteChannelOperations> AutoFundingStrategy<A> {
    pub fn new(cfg: AutoFundingStrategyConfig, hopr_chain_actions: A) -> Self {
        Self {
            cfg,
            hopr_chain_actions,
        }
    }
}

impl<A> Debug for AutoFundingStrategy<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Strategy::AutoFunding(self.cfg))
    }
}

impl<A> Display for AutoFundingStrategy<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::AutoFunding(self.cfg))
    }
}

#[async_trait]
impl<A: ChainWriteChannelOperations + Send + Sync> SingularStrategy for AutoFundingStrategy<A> {
    async fn on_own_channel_changed(
        &self,
        channel: &ChannelEntry,
        direction: ChannelDirection,
        change: ChannelChange,
    ) -> crate::errors::Result<()> {
        // Can only auto-fund outgoing channels
        if direction != ChannelDirection::Outgoing {
            return Ok(());
        }

        if let ChannelChange::CurrentBalance { right: new, .. } = change {
            if new.lt(&self.cfg.min_stake_threshold) && channel.status == ChannelStatus::Open {
                info!(%channel, balance = %channel.balance, threshold = %self.cfg.min_stake_threshold,
                    "stake on channel is below threshold",
                );

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_COUNT_AUTO_FUNDINGS.increment();

                let channel_id = channel.get_id();
                let rx = self
                    .hopr_chain_actions
                    .fund_channel(channel_id, self.cfg.funding_amount)
                    .await
                    .map_err(|e| StrategyError::Other(e.into()))?;

                std::mem::drop(rx); // The Receiver is not intentionally awaited here and the oneshot Sender can fail safely
                info!(%channel, amount = %self.cfg.funding_amount, "issued re-staking of channel", );
            }
            Ok(())
        } else {
            Err(CriteriaNotSatisfied)
        }
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use hopr_api::chain::ChainReceipt;

    use super::*;
    use crate::{
        auto_funding::{AutoFundingStrategy, AutoFundingStrategyConfig},
        strategy::SingularStrategy,
        tests::{MockChainActions, MockTestActions},
    };

    lazy_static::lazy_static! {
        static ref ALICE: Address = hex!("18f8ae833c85c51fbeba29cef9fbfb53b3bad950").into();
        static ref BOB: Address = hex!("44f23fa14130ca540b37251309700b6c281d972e").into();
        static ref CHRIS: Address = hex!("b6021e0860dd9d96c9ff0a73e2e5ba3a466ba234").into();
        static ref DAVE: Address = hex!("68499f50ff68d523385dc60686069935d17d762a").into();
    }

    #[tokio::test]
    async fn test_auto_funding_strategy() -> anyhow::Result<()> {
        let stake_limit = HoprBalance::from(7_u32);
        let fund_amount = HoprBalance::from(5_u32);

        let c1 = ChannelBuilder::new(*ALICE, *BOB)
            .with_stake(10)
            .with_ticket_index(0)
            .with_status(ChannelStatus::Open)
            .with_epoch(1)
            .build();

        let c2 = ChannelBuilder::new(*BOB, *CHRIS)
            .with_stake(5)
            .with_ticket_index(0)
            .with_status(ChannelStatus::Open)
            .with_epoch(1)
            .build();

        let c3 = ChannelBuilder::new(*CHRIS, *DAVE)
            .with_stake(5)
            .with_ticket_index(0)
            .with_status(ChannelStatus::PendingToClose(std::time::SystemTime::now()))
            .with_epoch(1)
            .build();

        let cfg = AutoFundingStrategyConfig {
            min_stake_threshold: stake_limit,
            funding_amount: fund_amount,
        };

        let mut mock = MockTestActions::new();
        mock.expect_fund_channel()
            .once()
            .with(
                mockall::predicate::eq(*c2.get_id()),
                mockall::predicate::eq(fund_amount),
            )
            .return_once(|_, _| Ok(ChainReceipt::default()));

        let afs = AutoFundingStrategy::new(cfg, MockChainActions(mock.into()));
        afs.on_own_channel_changed(
            &c1,
            ChannelDirection::Outgoing,
            ChannelChange::CurrentBalance {
                left: HoprBalance::zero(),
                right: c1.balance,
            },
        )
        .await?;

        afs.on_own_channel_changed(
            &c2,
            ChannelDirection::Outgoing,
            ChannelChange::CurrentBalance {
                left: HoprBalance::zero(),
                right: c2.balance,
            },
        )
        .await?;

        afs.on_own_channel_changed(
            &c3,
            ChannelDirection::Outgoing,
            ChannelChange::CurrentBalance {
                left: HoprBalance::zero(),
                right: c3.balance,
            },
        )
        .await?;

        Ok(())
    }
}
