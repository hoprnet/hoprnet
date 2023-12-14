use crate::aggregating::AggregatingStrategy;
use crate::auto_funding::AutoFundingStrategy;
use crate::auto_redeeming::AutoRedeemingStrategy;
use crate::errors::Result;
use crate::promiscuous::PromiscuousStrategy;
use crate::Strategy;
use async_lock::RwLock;
use async_trait::async_trait;
use core_ethereum_actions::channels::ChannelActions;
use core_ethereum_actions::CoreEthereumActions;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_network::network::{Network, NetworkExternalActions};
use core_protocol::ticket_aggregation::processor::BasicTicketAggregationActions;
use core_types::acknowledgement::AcknowledgedTicket;
use core_types::channels::{ChannelChange, ChannelDirection, ChannelEntry, ChannelStatus, Ticket};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use utils_log::{debug, error, info, warn};
use validator::Validate;

use utils_misc::time::native::current_timestamp;

#[cfg(all(feature = "prometheus", not(test)))]
use {
    strum::VariantNames,
    utils_metrics::metrics::{MultiGauge, SimpleCounter},
};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_CLOSURE_FINALIZATIONS: SimpleCounter =
        SimpleCounter::new("core_counter_strategy_count_closure_finalization", "Count of channels where closure finalizing was initiated automatically").unwrap();

    static ref METRIC_ENABLED_STRATEGIES: MultiGauge =
        MultiGauge::new("core_multi_gauge_strategy_enabled_strategies", "List of enabled strategies", Strategy::VARIANTS).unwrap();
}

/// Basic single strategy.
#[cfg_attr(test, mockall::automock)]
#[async_trait(? Send)]
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

/// Internal strategy which runs per tick and finalizes `PendingToClose` channels
/// which have elapsed the grace period
struct ChannelCloseFinalizer<Db: HoprCoreEthereumDbActions + Clone> {
    db: Arc<RwLock<Db>>,
    chain_actions: CoreEthereumActions<Db>,
}

impl<Db: HoprCoreEthereumDbActions + Clone> Display for ChannelCloseFinalizer<Db> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "channel_closure_finalizer")
    }
}

#[async_trait(? Send)]
impl<Db: HoprCoreEthereumDbActions + Clone> SingularStrategy for ChannelCloseFinalizer<Db> {
    async fn on_tick(&self) -> Result<()> {
        let to_close = self
            .db
            .read()
            .await
            .get_outgoing_channels()
            .await?
            .iter()
            .filter(|channel| {
                channel.status == ChannelStatus::PendingToClose && channel.closure_time_passed(current_timestamp())
            })
            .map(|channel| async {
                let channel_cpy = *channel;
                info!("channel closure finalizer: finalizing closure of {channel_cpy}");
                match self
                    .chain_actions
                    .close_channel(channel_cpy.destination, ChannelDirection::Outgoing, false)
                    .await
                {
                    Ok(_) => {
                        // Currently, we're not interested in awaiting the Close transactions to confirmation
                        debug!("channel closure finalizer: finalizing closure of {channel_cpy}");
                    }
                    Err(e) => error!("channel closure finalizer: failed to finalize closure of {channel_cpy}: {e}"),
                }
            })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await
            .len();

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_COUNT_CLOSURE_FINALIZATIONS.increment_by(to_close as u64);

        info!("channel closure finalizer: initiated closure finalization of {to_close} channels");
        Ok(())
    }
}

/// Configuration options for the `MultiStrategy` chain.
/// If `fail_on_continue` is set, the `MultiStrategy` sequence behaves as logical AND chain,
/// otherwise it behaves like a logical OR chain.
#[derive(Debug, Clone, PartialEq, Validate, Serialize, Deserialize)]
pub struct MultiStrategyConfig {
    /// Determines if the strategy should continue executing the next strategy if the current one failed.
    /// If set to `true`, the strategy behaves like a logical AND chain of `SingularStrategies`
    /// Otherwise, it behaves like a logical OR chain of `SingularStrategies`.
    /// Default is `true`.
    pub on_fail_continue: bool,

    /// Indicate whether the `MultiStrategy` can contain another `MultiStrategy`.
    /// Default is `true`.
    pub allow_recursive: bool,

    /// Indicates if the strategy should check for `PendingToClose` channels which have
    /// elapsed the closure grace period, to issue another channel closing transaction to close them.
    /// If not set, the user has to trigger the channel closure manually once again after the grace period
    /// is over.
    /// Default: false
    pub finalize_channel_closure: bool,

    /// Configuration of individual sub-strategies.
    /// Default is empty, which makes the `MultiStrategy` behave as passive.
    pub strategies: Vec<Strategy>,
}

impl Default for MultiStrategyConfig {
    fn default() -> Self {
        Self {
            on_fail_continue: true,
            allow_recursive: true,
            finalize_channel_closure: false,
            strategies: Vec::new(),
        }
    }
}

/// Defines an execution chain of `SingularStrategies`.
/// The `MultiStrategy` itself also implements the `SingularStrategy` trait,
/// which makes it possible (along with different `on_fail_continue` policies) to construct
/// various logical strategy chains.
pub struct MultiStrategy {
    strategies: Vec<Box<dyn SingularStrategy>>,
    cfg: MultiStrategyConfig,
}

impl MultiStrategy {
    /// Constructs new `MultiStrategy`.
    /// The strategy can contain another `MultiStrategy` if `allow_recursive` is set.
    pub fn new<Db, Net>(
        cfg: MultiStrategyConfig,
        db: Arc<RwLock<Db>>,
        network: Arc<RwLock<Network<Net>>>,
        chain_actions: CoreEthereumActions<Db>,
        ticket_aggregator: BasicTicketAggregationActions<std::result::Result<Ticket, String>>,
    ) -> Self
    where
        Db: HoprCoreEthereumDbActions + Clone + 'static,
        Net: NetworkExternalActions + 'static,
    {
        let mut strategies = Vec::<Box<dyn SingularStrategy>>::new();

        if cfg.finalize_channel_closure {
            strategies.push(Box::new(ChannelCloseFinalizer {
                db: db.clone(),
                chain_actions: chain_actions.clone(),
            }));
        }

        #[cfg(all(feature = "prometheus", not(test)))]
        Strategy::VARIANTS
            .iter()
            .for_each(|s| METRIC_ENABLED_STRATEGIES.set(&[*s], 0_f64));

        for strategy in cfg.strategies.iter() {
            match strategy {
                Strategy::Promiscuous(sub_cfg) => strategies.push(Box::new(PromiscuousStrategy::new(
                    *sub_cfg,
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

#[async_trait(? Send)]
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
            finalize_channel_closure: false,
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
            finalize_channel_closure: false,
            strategies: Vec::new(),
        };

        let ms = MultiStrategy {
            strategies: vec![Box::new(s1), Box::new(s2)],
            cfg,
        };
        ms.on_tick().await.expect_err("on_tick should fail");
    }
}
