//! ## Auto Funding Strategy
//! This strategy listens for channel state change events to check whether a channel has dropped below
//! `min_stake_threshold` HOPR. If this happens, the strategy issues a **fund channel** transaction to re-stake the
//! channel with `funding_amount` HOPR.
//!
//! Additionally, the strategy periodically scans all outgoing open channels on each tick and funds
//! any that have fallen below the threshold. This catches channels opened with low balance and
//! channels that were underfunded when the node started.
//!
//! ### In-flight tracking
//! To prevent duplicate funding when multiple balance-decrease events arrive in quick succession,
//! the strategy maintains a set of channel IDs
//! with in-flight funding transactions. A channel is added to the set when a funding tx is
//! successfully enqueued, and removed when:
//! - A balance increase event is observed for that channel (indicating the funding confirmed), or
//! - An `on_tick` scan finds the channel's balance has risen above the threshold.
//!
//! ### Metrics
//! - `hopr_strategy_auto_funding_funding_count` — incremented when a funding tx is successfully enqueued
//! - `hopr_strategy_auto_funding_failure_count` — incremented when a funding tx fails to enqueue
//!
//! For details on default parameters see [AutoFundingStrategyConfig].
use std::{
    collections::HashSet,
    fmt::{Debug, Display, Formatter},
    sync::Mutex,
};

use async_trait::async_trait;
use futures::StreamExt;
use hopr_lib::{
    ChannelChange, ChannelDirection, ChannelEntry, ChannelId, ChannelStatus, ChannelStatusDiscriminants, HoprBalance,
    api::chain::{ChainReadChannelOperations, ChainWriteChannelOperations, ChannelSelector},
};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use tracing::{debug, info, warn};
use validator::{Validate, ValidationError};

use crate::{
    Strategy,
    errors::{StrategyError, StrategyError::CriteriaNotSatisfied},
    strategy::SingularStrategy,
};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_AUTO_FUNDINGS: hopr_metrics::SimpleCounter =
        hopr_metrics::SimpleCounter::new("hopr_strategy_auto_funding_funding_count", "Count of initiated automatic fundings").unwrap();
    static ref METRIC_COUNT_AUTO_FUNDING_FAILURES: hopr_metrics::SimpleCounter =
        hopr_metrics::SimpleCounter::new("hopr_strategy_auto_funding_failure_count", "Count of failed automatic funding attempts").unwrap();
}

/// Validates that [`AutoFundingStrategyConfig::funding_amount`] is non-zero.
fn validate_funding_amount(amount: &HoprBalance) -> std::result::Result<(), ValidationError> {
    if amount.is_zero() {
        return Err(ValidationError::new("funding_amount must be greater than zero"));
    }
    Ok(())
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

    /// Funding amount. Must be greater than zero.
    ///
    /// Defaults to 10 wxHOPR.
    #[serde_as(as = "DisplayFromStr")]
    #[default(HoprBalance::new_base(10))]
    #[validate(custom(function = "validate_funding_amount"))]
    pub funding_amount: HoprBalance,
}

/// The `AutoFundingStrategy` automatically funds a channel that
/// dropped its staked balance below the configured threshold.
///
/// Tracks channels with in-flight funding transactions to prevent duplicate
/// funding when multiple balance-decrease events arrive in quick succession.
pub struct AutoFundingStrategy<A> {
    hopr_chain_actions: A,
    cfg: AutoFundingStrategyConfig,
    /// Channels with in-flight funding transactions.
    /// Entries are added when a funding tx is enqueued and removed when a
    /// balance increase is observed for the channel or when `on_tick` finds
    /// the channel's balance is above the threshold.
    in_flight: Mutex<HashSet<ChannelId>>,
}

impl<A: ChainReadChannelOperations + ChainWriteChannelOperations> AutoFundingStrategy<A> {
    pub fn new(cfg: AutoFundingStrategyConfig, hopr_chain_actions: A) -> Self {
        if cfg.funding_amount.le(&cfg.min_stake_threshold) {
            warn!(
                funding_amount = %cfg.funding_amount,
                min_stake_threshold = %cfg.min_stake_threshold,
                "funding_amount is not greater than min_stake_threshold; \
                 successful funding may re-trigger the threshold check"
            );
        }
        Self {
            cfg,
            hopr_chain_actions,
            in_flight: Mutex::new(HashSet::new()),
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
impl<A: ChainReadChannelOperations + ChainWriteChannelOperations + Send + Sync> SingularStrategy
    for AutoFundingStrategy<A>
{
    /// Periodically scans all outgoing open channels and funds any with balance at or below
    /// the configured threshold. Skips channels that already have in-flight funding transactions.
    ///
    /// This handles two cases that event-driven funding misses:
    /// - Channels opened with balance already below threshold (only a `ChannelOpened` event
    ///   is emitted, which doesn't trigger balance-based funding)
    /// - Channels that were underfunded when the node started or restarted (no events are
    ///   replayed to the strategy at startup)
    async fn on_tick(&self) -> crate::errors::Result<()> {
        let mut channels = self
            .hopr_chain_actions
            .stream_channels(
                ChannelSelector::default()
                    .with_source(*self.hopr_chain_actions.me())
                    .with_allowed_states(&[ChannelStatusDiscriminants::Open]),
            )
            .await
            .map_err(|e| StrategyError::Other(e.into()))?;

        while let Some(channel) = channels.next().await {
            if channel.balance.le(&self.cfg.min_stake_threshold) {
                let channel_id = *channel.get_id();

                // Skip channels with in-flight funding
                {
                    let in_flight = self.in_flight.lock().unwrap_or_else(|e| e.into_inner());
                    if in_flight.contains(&channel_id) {
                        debug!(%channel, "skipping channel with in-flight funding");
                        continue;
                    }
                }

                info!(
                    %channel,
                    balance = %channel.balance,
                    threshold = %self.cfg.min_stake_threshold,
                    "on_tick: stake on channel at or below threshold"
                );

                let fund_result = self
                    .hopr_chain_actions
                    .fund_channel(&channel_id, self.cfg.funding_amount)
                    .await;
                // Drop the confirmation future (BoxFuture) before channel_id goes out of scope.
                let funded = fund_result.map(|_rx| ()).map_err(|e| e.to_string());
                match funded {
                    Ok(()) => {
                        let mut in_flight = self.in_flight.lock().unwrap_or_else(|e| e.into_inner());
                        in_flight.insert(channel_id);

                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_COUNT_AUTO_FUNDINGS.increment();

                        info!(%channel, amount = %self.cfg.funding_amount, "on_tick: issued re-staking of channel");
                    }
                    Err(e) => {
                        warn!(%channel, error = %e, "on_tick: failed to enqueue funding tx");

                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_COUNT_AUTO_FUNDING_FAILURES.increment();
                    }
                }
            } else {
                // Channel is above threshold; clear any stale in-flight entry
                let mut in_flight = self.in_flight.lock().unwrap_or_else(|e| e.into_inner());
                in_flight.remove(channel.get_id());
            }
        }

        debug!("auto-funding on_tick scan complete");
        Ok(())
    }

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

        if let ChannelChange::Balance { left: old, right: new } = change {
            // If balance increased, clear in-flight state for this channel
            if new > old {
                let mut in_flight = self.in_flight.lock().unwrap_or_else(|e| e.into_inner());
                if in_flight.remove(channel.get_id()) {
                    debug!(%channel, "cleared in-flight funding state after balance increase");
                }
            }

            if new.le(&self.cfg.min_stake_threshold) && channel.status == ChannelStatus::Open {
                let channel_id = *channel.get_id();

                // Skip channels with in-flight funding
                {
                    let in_flight = self.in_flight.lock().unwrap_or_else(|e| e.into_inner());
                    if in_flight.contains(&channel_id) {
                        debug!(%channel, "skipping channel with in-flight funding");
                        return Ok(());
                    }
                }

                info!(
                    %channel,
                    balance = %channel.balance,
                    threshold = %self.cfg.min_stake_threshold,
                    "stake on channel at or below threshold"
                );

                let fund_result = self
                    .hopr_chain_actions
                    .fund_channel(&channel_id, self.cfg.funding_amount)
                    .await;
                // Drop the confirmation future (BoxFuture) before channel_id goes out of scope.
                let funded = fund_result.map(|_rx| ()).map_err(|e| e.to_string());
                match funded {
                    Ok(()) => {
                        let mut in_flight = self.in_flight.lock().unwrap_or_else(|e| e.into_inner());
                        in_flight.insert(channel_id);

                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_COUNT_AUTO_FUNDINGS.increment();

                        info!(%channel, amount = %self.cfg.funding_amount, "issued re-staking of channel");
                    }
                    Err(e) => {
                        warn!(%channel, error = %e, "failed to enqueue funding tx");

                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_COUNT_AUTO_FUNDING_FAILURES.increment();
                    }
                }
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

    use super::*;
    use crate::{
        auto_funding::{AutoFundingStrategy, AutoFundingStrategyConfig},
        strategy::SingularStrategy,
    };
    use futures::StreamExt;
    use futures_time::future::FutureExt;
    use hex_literal::hex;
    use hopr_chain_connector::{create_trustful_hopr_blokli_connector, testing::BlokliTestStateBuilder};
    use hopr_lib::{
        Address, BytesRepresentable, ChainKeypair, Keypair, XDaiBalance,
        api::chain::{ChainEvent, ChainEvents},
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

        let c1 = ChannelEntry::new(*ALICE, *BOB, 10_u32.into(), 0_u32.into(), ChannelStatus::Open, 0);

        let c2 = ChannelEntry::new(*BOB, *CHRIS, 5_u32.into(), 0_u32.into(), ChannelStatus::Open, 0);

        let c3 = ChannelEntry::new(
            *CHRIS,
            *DAVE,
            5_u32.into(),
            0_u32.into(),
            ChannelStatus::PendingToClose(
                chrono::DateTime::<chrono::Utc>::from_str("2025-11-10T00:00:00+00:00")?.into(),
            ),
            0,
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

    #[test]
    fn test_config_validation_rejects_zero_funding_amount() {
        let cfg = AutoFundingStrategyConfig {
            min_stake_threshold: HoprBalance::new_base(1),
            funding_amount: HoprBalance::zero(),
        };
        assert!(
            cfg.validate().is_err(),
            "config with zero funding_amount should fail validation"
        );
    }

    #[test]
    fn test_config_validation_accepts_valid_config() {
        let cfg = AutoFundingStrategyConfig {
            min_stake_threshold: HoprBalance::new_base(1),
            funding_amount: HoprBalance::new_base(10),
        };
        assert!(
            cfg.validate().is_ok(),
            "config with valid funding_amount should pass validation"
        );
    }

    #[test]
    fn test_default_config_passes_validation() {
        let cfg = AutoFundingStrategyConfig::default();
        assert!(cfg.validate().is_ok(), "default config should pass validation");
    }

    #[test_log::test(tokio::test)]
    async fn test_on_tick_funds_underfunded_channels() -> anyhow::Result<()> {
        let stake_limit = HoprBalance::from(7_u32);
        let fund_amount = HoprBalance::from(5_u32);

        // BOB -> CHRIS channel with balance below threshold
        let c1 = ChannelEntry::new(
            *BOB,
            *CHRIS,
            3_u32.into(),
            0_u32.into(),
            ChannelStatus::Open,
            0_u32.into(),
        );

        // BOB -> DAVE channel with balance above threshold
        let c2 = ChannelEntry::new(
            *BOB,
            *DAVE,
            10_u32.into(),
            0_u32.into(),
            ChannelStatus::Open,
            0_u32.into(),
        );

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS, &*DAVE],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([c1, c2])
            .build_dynamic_client([1; Address::SIZE].into());

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

        // on_tick should scan channels and fund c1 (below threshold) but not c2 (above threshold)
        afs.on_tick().await?;

        // Expect a ChannelBalanceIncreased event for c1
        events
            .filter(|event| {
                futures::future::ready(
                    matches!(event, ChainEvent::ChannelBalanceIncreased(c, amount) if c.get_id() == c1.get_id() && amount == &fund_amount),
                )
            })
            .next()
            .timeout(futures_time::time::Duration::from_secs(2))
            .await?;

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_in_flight_prevents_duplicate_funding() -> anyhow::Result<()> {
        let stake_limit = HoprBalance::from(7_u32);
        let fund_amount = HoprBalance::from(5_u32);

        let c1 = ChannelEntry::new(
            *BOB,
            *CHRIS,
            3_u32.into(),
            0_u32.into(),
            ChannelStatus::Open,
            0_u32.into(),
        );

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS, &*DAVE],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([c1])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        let _events = chain_connector.subscribe()?;

        let cfg = AutoFundingStrategyConfig {
            min_stake_threshold: stake_limit,
            funding_amount: fund_amount,
        };

        let afs = AutoFundingStrategy::new(cfg, chain_connector);

        // First call should trigger funding
        afs.on_own_channel_changed(
            &c1,
            ChannelDirection::Outgoing,
            ChannelChange::Balance {
                left: HoprBalance::from(10_u32),
                right: c1.balance,
            },
        )
        .await?;

        // Verify the channel is in the in-flight set
        {
            let in_flight = afs.in_flight.lock().unwrap();
            assert!(
                in_flight.contains(c1.get_id()),
                "channel should be in the in-flight set after funding"
            );
        }

        // Second call with same balance should be skipped due to in-flight tracking.
        // This returns Ok(()) rather than triggering another funding tx.
        afs.on_own_channel_changed(
            &c1,
            ChannelDirection::Outgoing,
            ChannelChange::Balance {
                left: HoprBalance::from(10_u32),
                right: c1.balance,
            },
        )
        .await?;

        // The in-flight set should still contain exactly one entry (unchanged)
        {
            let in_flight = afs.in_flight.lock().unwrap();
            assert_eq!(in_flight.len(), 1, "in-flight set should still have exactly one entry");
            assert!(
                in_flight.contains(c1.get_id()),
                "channel should still be in the in-flight set"
            );
        }

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_balance_increase_clears_in_flight() -> anyhow::Result<()> {
        let stake_limit = HoprBalance::from(7_u32);
        let fund_amount = HoprBalance::from(5_u32);

        let c1 = ChannelEntry::new(
            *BOB,
            *CHRIS,
            3_u32.into(),
            0_u32.into(),
            ChannelStatus::Open,
            0_u32.into(),
        );

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS, &*DAVE],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([c1])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        let _events = chain_connector.subscribe()?;

        let cfg = AutoFundingStrategyConfig {
            min_stake_threshold: stake_limit,
            funding_amount: fund_amount,
        };

        let afs = AutoFundingStrategy::new(cfg, chain_connector);

        // Trigger funding (balance decrease below threshold)
        afs.on_own_channel_changed(
            &c1,
            ChannelDirection::Outgoing,
            ChannelChange::Balance {
                left: HoprBalance::from(10_u32),
                right: c1.balance,
            },
        )
        .await?;

        // Verify channel is in-flight
        {
            let in_flight = afs.in_flight.lock().unwrap();
            assert!(in_flight.contains(c1.get_id()));
        }

        // Simulate balance increase event (funding confirmed)
        let funded_channel = ChannelEntry::new(
            *BOB,
            *CHRIS,
            (3_u32 + 5_u32).into(),
            0_u32.into(),
            ChannelStatus::Open,
            0_u32.into(),
        );

        afs.on_own_channel_changed(
            &funded_channel,
            ChannelDirection::Outgoing,
            ChannelChange::Balance {
                left: HoprBalance::from(3_u32),
                right: HoprBalance::from(8_u32),
            },
        )
        .await?;

        // Verify channel is no longer in-flight
        {
            let in_flight = afs.in_flight.lock().unwrap();
            assert!(
                !in_flight.contains(c1.get_id()),
                "channel should be cleared from in-flight after balance increase"
            );
        }

        Ok(())
    }
}
