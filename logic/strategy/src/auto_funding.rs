//! ## Auto Funding Strategy
//! This strategy listens for channel state change events to check whether a channel has dropped below
//! `min_stake_threshold` HOPR. If this happens, the strategy issues a **fund channel** transaction to re-stake the
//! channel with `funding_amount` HOPR.
//!
//! For details on default parameters see [AutoFundingStrategyConfig].
use std::fmt::{Debug, Display, Formatter};

use async_trait::async_trait;
use hopr_lib::{
    ChannelChange, ChannelDirection, ChannelEntry, ChannelStatus, HoprBalance, api::chain::ChainWriteChannelOperations,
};
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

        if let ChannelChange::Balance { right: new, .. } = change {
            if new.le(&self.cfg.min_stake_threshold) && channel.status == ChannelStatus::Open {
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
    use std::str::FromStr;

    use futures::StreamExt;
    use futures_time::future::FutureExt;
    use hex_literal::hex;
    use hopr_chain_connector::{create_trustful_hopr_blokli_connector, testing::BlokliTestStateBuilder};
    use hopr_lib::{
        Address, BytesRepresentable, ChainKeypair, Keypair, XDaiBalance,
        api::chain::{ChainEvent, ChainEvents},
    };

    use super::*;
    use crate::{
        auto_funding::{AutoFundingStrategy, AutoFundingStrategyConfig},
        strategy::SingularStrategy,
    };

    lazy_static::lazy_static! {
        static ref BOB_KP: ChainKeypair = ChainKeypair::from_secret(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))
        .expect("lazy static keypair should be valid");

        static ref ALICE: Address = hex!("18f8ae833c85c51fbeba29cef9fbfb53b3bad950").into();
        static ref BOB: Address = BOB_KP.public().to_address();
        static ref CHRIS: Address = hex!("b6021e0860dd9d96c9ff0a73e2e5ba3a466ba234").into();
        static ref DAVE: Address = hex!("68499f50ff68d523385dc60686069935d17d762a").into();
    }

    #[test_log::test(tokio::test)]
    async fn test_auto_funding_strategy() -> anyhow::Result<()> {
        let stake_limit = HoprBalance::from(7_u32);
        let fund_amount = HoprBalance::from(5_u32);

        let c1 = ChannelEntry::new(
            *ALICE,
            *BOB,
            10_u32.into(),
            0_u32.into(),
            ChannelStatus::Open,
            0_u32.into(),
        );

        let c2 = ChannelEntry::new(
            *BOB,
            *CHRIS,
            5_u32.into(),
            0_u32.into(),
            ChannelStatus::Open,
            0_u32.into(),
        );

        let c3 = ChannelEntry::new(
            *CHRIS,
            *DAVE,
            5_u32.into(),
            0_u32.into(),
            ChannelStatus::PendingToClose(
                chrono::DateTime::<chrono::Utc>::from_str("2025-11-10T00:00:00+00:00")?.into(),
            ),
            0_u32.into(),
        );

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS, &*DAVE],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([c1, c2, c3])
            .build_dynamic_client([1; Address::SIZE].into());

        let snapshot = blokli_sim.snapshot();

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        let events = chain_connector.subscribe()?;

        let cfg = AutoFundingStrategyConfig {
            min_stake_threshold: stake_limit,
            funding_amount: fund_amount,
        };

        let afs = AutoFundingStrategy::new(cfg, chain_connector);
        afs.on_own_channel_changed(
            &c1,
            ChannelDirection::Outgoing,
            ChannelChange::Balance {
                left: HoprBalance::zero(),
                right: c1.balance,
            },
        )
        .await?;

        afs.on_own_channel_changed(
            &c2,
            ChannelDirection::Outgoing,
            ChannelChange::Balance {
                left: HoprBalance::zero(),
                right: c2.balance,
            },
        )
        .await?;

        afs.on_own_channel_changed(
            &c3,
            ChannelDirection::Outgoing,
            ChannelChange::Balance {
                left: HoprBalance::zero(),
                right: c3.balance,
            },
        )
        .await?;

        events
            .filter(|event| futures::future::ready(matches!(event, ChainEvent::ChannelBalanceIncreased(c, amount) if c.get_id() == c2.get_id() && amount == &fund_amount)))
            .next()
            .timeout(futures_time::time::Duration::from_secs(2))
            .await?;

        insta::assert_yaml_snapshot!(*snapshot.refresh());

        Ok(())
    }
}
