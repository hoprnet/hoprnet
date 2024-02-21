//! ## Multi Strategy
//!
//! This strategy can stack multiple above strategies (called sub-strategies in this context) into one.
//! Once a strategy event is triggered, it is executed sequentially on the sub-strategies one by one.
//! The strategy can be configured to not call the next sub-strategy event if the sub-strategy currently being executed failed,
//! which is done by setting the `on_fail_continue` flag.
//!
//! Hence, the sub-strategy chain then can behave as a logical AND (`on_fail_continue` = `false`) execution chain
//! or logical OR (`on_fail_continue` = `true`) execution chain.
//!
//! A Multi Strategy can also contain another Multi Strategy as a sub-strategy if `allow_recursive` flag is set.
//! However, this recursion is always allowed up to 2 levels only.
//! Along with the `on_fail_continue` value, the recursive feature allows constructing more complex logical strategy chains.
//!
//! The MultiStrategy can also observe channels being `PendingToClose` and running out of closure grace period,
//! and if this happens, it will issue automatically the final close transaction, which transitions the state to `Closed`.
//! This can be controlled by the `finalize_channel_closure` parameter.
//!
//! For details on default parameters see [MultiStrategyConfig].
use async_lock::RwLock;
use async_trait::async_trait;
use chain_actions::ChainActions;
use chain_db::traits::HoprCoreEthereumDbActions;
use core_network::network::{Network, NetworkExternalActions};
use core_protocol::ticket_aggregation::processor::BasicTicketAggregationActions;
use hopr_internal_types::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use tracing::{error, warn};
use validator::Validate;

use crate::aggregating::AggregatingStrategy;
use crate::auto_funding::AutoFundingStrategy;
use crate::auto_redeeming::AutoRedeemingStrategy;
use crate::errors::Result;
use crate::promiscuous::PromiscuousStrategy;
use crate::Strategy;

use crate::channel_finalizer::ClosureFinalizerStrategy;
#[cfg(all(feature = "prometheus", not(test)))]
use {hopr_metrics::metrics::MultiGauge, strum::VariantNames};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_ENABLED_STRATEGIES: MultiGauge =
        MultiGauge::new("hopr_strategy_enabled_strategies", "List of enabled strategies", &["strategy"]).unwrap();
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

/// Configuration options for the `MultiStrategy` chain.
/// If `fail_on_continue` is set, the `MultiStrategy` sequence behaves as logical AND chain,
/// otherwise it behaves like a logical OR chain.
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MultiStrategyConfig {
    /// Determines if the strategy should continue executing the next strategy if the current one failed.
    /// If set to `true`, the strategy behaves like a logical AND chain of `SingularStrategies`
    /// Otherwise, it behaves like a logical OR chain of `SingularStrategies`.
    ///
    /// Default is true.
    #[default = true]
    pub on_fail_continue: bool,

    /// Indicate whether the `MultiStrategy` can contain another `MultiStrategy`.
    ///
    /// Default is true.
    #[default = true]
    pub allow_recursive: bool,

    /// Configuration of individual sub-strategies.
    ///
    /// Default is empty, which makes the `MultiStrategy` behave as passive.
    #[default(_code = "vec![]")]
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
    pub fn new<Db, Net>(
        cfg: MultiStrategyConfig,
        db: Arc<RwLock<Db>>,
        network: Arc<RwLock<Network<Net>>>,
        chain_actions: ChainActions<Db>,
        ticket_aggregator: BasicTicketAggregationActions<std::result::Result<Ticket, String>>,
    ) -> Self
    where
        Db: HoprCoreEthereumDbActions + Clone + Send + Sync + 'static,
        Net: NetworkExternalActions + Send + Sync + 'static,
    {
        let mut strategies = Vec::<Box<dyn SingularStrategy + Send + Sync>>::new();

        #[cfg(all(feature = "prometheus", not(test)))]
        Strategy::VARIANTS
            .iter()
            .for_each(|s| METRIC_ENABLED_STRATEGIES.set(&[*s], 0_f64));

        for strategy in cfg.strategies.iter() {
            match strategy {
                Strategy::Promiscuous(sub_cfg) => strategies.push(Box::new(PromiscuousStrategy::new(
                    sub_cfg.clone(),
                    db.clone(),
                    network.clone(),
                    chain_actions.clone(),
                ))),
                Strategy::Aggregating(sub_cfg) => strategies.push(Box::new(AggregatingStrategy::new(
                    *sub_cfg,
                    db.clone(),
                    chain_actions.clone(),
                    ticket_aggregator.clone(),
                ))),
                Strategy::AutoRedeeming(sub_cfg) => {
                    strategies.push(Box::new(AutoRedeemingStrategy::new(*sub_cfg, chain_actions.clone())))
                }
                Strategy::AutoFunding(sub_cfg) => {
                    strategies.push(Box::new(AutoFundingStrategy::new(*sub_cfg, chain_actions.clone())))
                }
                Strategy::ClosureFinalizer(sub_cfg) => strategies.push(Box::new(ClosureFinalizerStrategy::new(
                    *sub_cfg,
                    db.clone(),
                    chain_actions.clone(),
                ))),
                Strategy::Multi(sub_cfg) => {
                    if cfg.allow_recursive {
                        let mut cfg_clone = sub_cfg.clone();
                        cfg_clone.allow_recursive = false; // Do not allow more levels of recursion

                        strategies.push(Box::new(Self::new(
                            cfg_clone,
                            db.clone(),
                            network.clone(),
                            chain_actions.clone(),
                            ticket_aggregator.clone(),
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
                    warn!("{self} on_tick chain stopped at {strategy}");
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
                    warn!("{self} on_acknowledged_ticket chain stopped at {strategy}");
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
                    warn!("{self} on_channel_state_changed chain stopped at {strategy}");
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
    use crate::errors::StrategyError::Other;
    use crate::strategy::{MockSingularStrategy, MultiStrategy, MultiStrategyConfig, SingularStrategy};
    use mockall::Sequence;

    #[async_std::test]
    async fn test_multi_strategy_logical_or_flow() {
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
            strategies: Vec::new(),
        };

        let ms = MultiStrategy {
            strategies: vec![Box::new(s1), Box::new(s2)],
            cfg,
        };
        ms.on_tick().await.expect("on_tick should not fail");
    }

    #[async_std::test]
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
            strategies: Vec::new(),
        };

        let ms = MultiStrategy {
            strategies: vec![Box::new(s1), Box::new(s2)],
            cfg,
        };
        ms.on_tick().await.expect_err("on_tick should fail");
    }
}
