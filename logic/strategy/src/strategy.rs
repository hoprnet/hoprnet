//! ## Multi Strategy
//!
//! This strategy can stack multiple above strategies (called sub-strategies in this context) into one.
//! Once a strategy event is triggered, it is executed sequentially on the sub-strategies one by one.
//! The strategy can be configured to not call the next sub-strategy event if the sub-strategy currently being executed
//! failed, which is done by setting the `on_fail_continue` flag.
//!
//! Hence, the sub-strategy chain then can behave as a logical AND (`on_fail_continue` = `false`) execution chain
//! or logical OR (`on_fail_continue` = `true`) execution chain.
//!
//! A Multi Strategy can also contain another Multi Strategy as a sub-strategy if `allow_recursive` flag is set.
//! However, this recursion is always allowed up to 2 levels only.
//! Along with the `on_fail_continue` value, the recursive feature allows constructing more complex logical strategy
//! chains.
//!
//! The MultiStrategy can also observe channels being `PendingToClose` and running out of closure grace period,
//! and if this happens, it will issue automatically the final close transaction, which transitions the state to
//! `Closed`. This can be controlled by the `finalize_channel_closure` parameter.
//!
//! For details on default parameters see [MultiStrategyConfig].
use std::fmt::{Debug, Display, Formatter};

use async_trait::async_trait;
use hopr_api::{
    chain::{ChainReadChannelOperations, ChainValues, ChainWriteChannelOperations, ChainWriteTicketOperations},
    db::HoprDbTicketOperations,
};
use hopr_internal_types::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
#[cfg(all(feature = "prometheus", not(test)))]
use strum::VariantNames;
use tracing::{error, warn};
use validator::{Validate, ValidationError};

use crate::{
    Strategy, auto_funding::AutoFundingStrategy, auto_redeeming::AutoRedeemingStrategy,
    channel_finalizer::ClosureFinalizerStrategy, errors::Result,
};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_ENABLED_STRATEGIES: hopr_metrics::MultiGauge =
        hopr_metrics::MultiGauge::new("hopr_strategy_enabled_strategies", "List of enabled strategies", &["strategy"]).unwrap();
}

/// Basic single strategy.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SingularStrategy: Display {
    /// Strategy event raised at period intervals (typically each 1 minute).
    async fn on_tick(&self) -> Result<()> {
        Ok(())
    }

    /// Strategy event raised when a new **winning** acknowledged ticket is received in a channel
    async fn on_acknowledged_winning_ticket(&self, _ack: &AcknowledgedTicket) -> Result<()> {
        Ok(())
    }

    /// Strategy event raised whenever the Indexer registers a change on node's own channel.
    async fn on_own_channel_changed(
        &self,
        _channel: &ChannelEntry,
        _direction: ChannelDirection,
        _change: ChannelChange,
    ) -> Result<()> {
        Ok(())
    }
}

#[inline]
fn just_true() -> bool {
    true
}

#[inline]
fn sixty_seconds() -> std::time::Duration {
    std::time::Duration::from_secs(60)
}

#[inline]
fn empty_vector() -> Vec<Strategy> {
    vec![]
}

fn validate_execution_interval(interval: &std::time::Duration) -> std::result::Result<(), ValidationError> {
    if interval < &std::time::Duration::from_secs(10) {
        Err(ValidationError::new(
            "strategy execution interval must be at least 1 second",
        ))
    } else {
        Ok(())
    }
}

/// Configuration options for the `MultiStrategy` chain.
/// If `fail_on_continue` is set, the `MultiStrategy` sequence behaves as logical AND chain,
/// otherwise it behaves like a logical OR chain.
#[serde_as]
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MultiStrategyConfig {
    /// Determines if the strategy should continue executing the next strategy if the current one failed.
    /// If set to `true`, the strategy behaves like a logical AND chain of `SingularStrategies`
    /// Otherwise, it behaves like a logical OR chain of `SingularStrategies`.
    ///
    /// Default is true.
    #[default = true]
    #[serde(default = "just_true")]
    pub on_fail_continue: bool,

    /// Indicate whether the `MultiStrategy` can contain another `MultiStrategy`.
    ///
    /// Default is true.
    #[default = true]
    #[serde(default = "just_true")]
    pub allow_recursive: bool,

    /// Execution interval of the configured strategies in seconds.
    ///
    /// Default is 60 seconds, minimum is 10 seconds.
    #[default(sixty_seconds())]
    #[serde(default = "sixty_seconds")]
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    #[validate(custom(function = "validate_execution_interval"))]
    pub execution_interval: std::time::Duration,

    /// Configuration of individual sub-strategies.
    ///
    /// Default is empty, which makes the `MultiStrategy` behave as passive.
    #[default(_code = "vec![]")]
    #[serde(default = "empty_vector")]
    pub strategies: Vec<Strategy>,
}

/// Defines an execution chain of `SingularStrategies`.
/// The `MultiStrategy` itself also implements the `SingularStrategy` trait,
/// which makes it possible (along with different `on_fail_continue` policies) to construct
/// various logical strategy chains.
pub struct MultiStrategy {
    strategies: Vec<Box<dyn SingularStrategy + Send + Sync>>,
    cfg: MultiStrategyConfig,
}

impl MultiStrategy {
    /// Constructs new `MultiStrategy`.
    /// The strategy can contain another `MultiStrategy` if `allow_recursive` is set.
    pub fn new<A, Db>(cfg: MultiStrategyConfig, hopr_chain_actions: A, node_db: Db) -> Self
    where
        A: ChainReadChannelOperations
            + ChainWriteChannelOperations
            + ChainWriteTicketOperations
            + ChainValues
            + Clone
            + Send
            + Sync
            + 'static,
        Db: HoprDbTicketOperations + Clone + Send + Sync + 'static,
    {
        let mut strategies = Vec::<Box<dyn SingularStrategy + Send + Sync>>::new();

        #[cfg(all(feature = "prometheus", not(test)))]
        Strategy::VARIANTS
            .iter()
            .for_each(|s| METRIC_ENABLED_STRATEGIES.set(&[*s], 0_f64));

        for strategy in cfg.strategies.iter() {
            match strategy {
                Strategy::AutoRedeeming(sub_cfg) => strategies.push(Box::new(AutoRedeemingStrategy::new(
                    *sub_cfg,
                    hopr_chain_actions.clone(),
                    node_db.clone(),
                ))),
                Strategy::AutoFunding(sub_cfg) => {
                    strategies.push(Box::new(AutoFundingStrategy::new(*sub_cfg, hopr_chain_actions.clone())))
                }
                Strategy::ClosureFinalizer(sub_cfg) => strategies.push(Box::new(ClosureFinalizerStrategy::new(
                    *sub_cfg,
                    hopr_chain_actions.clone(),
                ))),
                Strategy::Multi(sub_cfg) => {
                    if cfg.allow_recursive {
                        let mut cfg_clone = sub_cfg.clone();
                        cfg_clone.allow_recursive = false; // Do not allow more levels of recursion

                        strategies.push(Box::new(Self::new(
                            cfg_clone,
                            hopr_chain_actions.clone(),
                            node_db.clone(),
                        )))
                    } else {
                        error!("recursive multi-strategy not allowed and skipped")
                    }
                }

                // Passive strategy = empty MultiStrategy
                Strategy::Passive => strategies.push(Box::new(Self {
                    cfg: Default::default(),
                    strategies: Vec::new(),
                })),
            }

            #[cfg(all(feature = "prometheus", not(test)))]
            METRIC_ENABLED_STRATEGIES.set(&[&strategy.to_string()], 1_f64);
        }

        Self { strategies, cfg }
    }
}

impl Debug for MultiStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Strategy::Multi(self.cfg.clone()))
    }
}

impl Display for MultiStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::Multi(self.cfg.clone()))
    }
}

#[async_trait]
impl SingularStrategy for MultiStrategy {
    async fn on_tick(&self) -> Result<()> {
        for strategy in self.strategies.iter() {
            if let Err(e) = strategy.on_tick().await {
                if !self.cfg.on_fail_continue {
                    warn!(%self, %strategy, "on_tick chain stopped at strategy");
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    async fn on_acknowledged_winning_ticket(&self, ack: &AcknowledgedTicket) -> Result<()> {
        for strategy in self.strategies.iter() {
            if let Err(e) = strategy.on_acknowledged_winning_ticket(ack).await {
                if !self.cfg.on_fail_continue {
                    warn!(%self, %strategy, "on_acknowledged_ticket chain stopped at strategy");
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    async fn on_own_channel_changed(
        &self,
        channel: &ChannelEntry,
        direction: ChannelDirection,
        change: ChannelChange,
    ) -> Result<()> {
        for strategy in self.strategies.iter() {
            if let Err(e) = strategy.on_own_channel_changed(channel, direction, change).await {
                if !self.cfg.on_fail_continue {
                    warn!(%self, "on_channel_state_changed chain stopped at strategy");
                    return Err(e);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
impl Display for MockSingularStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "mock")
    }
}

#[cfg(test)]
mod tests {
    use mockall::Sequence;

    use crate::{
        errors::StrategyError::Other,
        strategy::{MockSingularStrategy, MultiStrategy, MultiStrategyConfig, SingularStrategy},
    };

    #[tokio::test]
    async fn test_multi_strategy_logical_or_flow() -> anyhow::Result<()> {
        let mut seq = Sequence::new();

        let mut s1 = MockSingularStrategy::new();
        s1.expect_on_tick()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|| Err(Other("error".into())));

        let mut s2 = MockSingularStrategy::new();
        s2.expect_on_tick().times(1).in_sequence(&mut seq).returning(|| Ok(()));

        let cfg = MultiStrategyConfig {
            on_fail_continue: true,
            allow_recursive: true,
            execution_interval: std::time::Duration::from_secs(1),
            strategies: Vec::new(),
        };

        let ms = MultiStrategy {
            strategies: vec![Box::new(s1), Box::new(s2)],
            cfg,
        };
        ms.on_tick().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_multi_strategy_logical_and_flow() {
        let mut seq = Sequence::new();

        let mut s1 = MockSingularStrategy::new();
        s1.expect_on_tick()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|| Err(Other("error".into())));

        let mut s2 = MockSingularStrategy::new();
        s2.expect_on_tick().never().in_sequence(&mut seq).returning(|| Ok(()));

        let cfg = MultiStrategyConfig {
            on_fail_continue: false,
            allow_recursive: true,
            execution_interval: std::time::Duration::from_secs(1),
            strategies: Vec::new(),
        };

        let ms = MultiStrategy {
            strategies: vec![Box::new(s1), Box::new(s2)],
            cfg,
        };
        ms.on_tick().await.expect_err("on_tick should fail");
    }
}
