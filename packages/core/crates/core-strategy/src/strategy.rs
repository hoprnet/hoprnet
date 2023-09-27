use crate::aggregating::AggregatingStrategy;
use crate::auto_funding::AutoFundingStrategy;
use crate::auto_redeeming::AutoRedeemingStrategy;
use crate::errors::Result;
use crate::promiscuous::PromiscuousStrategy;
use crate::Strategy;
use async_std::sync::RwLock;
use async_trait::async_trait;
use core_ethereum_actions::transaction_queue::TransactionSender;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_network::network::{Network, NetworkExternalActions};
use core_protocol::ticket_aggregation::processor::BasicTicketAggregationActions;
use core_types::acknowledgement::AcknowledgedTicket;
use core_types::channels::{ChannelEntry, Ticket};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use utils_log::{error, warn};
use validator::Validate;

/// Basic single strategy.
#[cfg_attr(test, mockall::automock)]
#[async_trait(? Send)]
pub trait SingularStrategy: Display {
    /// Strategy event raised at period intervals (typically each 1 minute).
    async fn on_tick(&self) -> Result<()> {
        Ok(())
    }

    /// Strategy event raised when a new acknowledged ticket is received in a channel
    async fn on_acknowledged_ticket(&self, _ack: &AcknowledgedTicket) -> Result<()> {
        Ok(())
    }

    /// Strategy event raised whenever the Indexer registers a change in the channel status
    async fn on_channel_state_changed(&self, _channel: &ChannelEntry) -> Result<()> {
        Ok(())
    }
}

/// Configuration options for the `MultiStrategy` chain.
/// If `fail_on_continue` is set, the `MultiStrategy` sequence behaves as logical AND chain,
/// otherwise it behaves like a logical OR chain.
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
#[derive(Clone, PartialEq, Validate, Serialize, Deserialize)]
pub struct MultiStrategyConfig {
    /// Determines if the strategy should continue executing the next strategy if the current one failed.
    /// If set to `true`, the strategy behaves like a logical AND chain of `SingularStrategies`
    /// Otherwise, it behaves like a logical OR chain of `SingularStrategies`.
    /// Default is `true`.
    pub on_fail_continue: bool,

    /// Indicate whether the `MultiStrategy` can contain another `MultiStrategy`.
    /// Default is `true`.
    pub allow_recursive: bool,

    /// Configuration of individual sub-strategies.
    /// Default is empty, which makes the `MultiStrategy` behave as passive.
    pub(crate) strategies: Vec<Strategy>, // non-pub due to wasm
}

impl MultiStrategyConfig {
    pub fn get_strategies(&mut self) -> &mut Vec<Strategy> {
        &mut self.strategies
    }
}

impl Debug for MultiStrategyConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.strategies.iter().filter(|f| match f {
                Strategy::Multi(_) => false, // avoid infinite recursion when debugging
                _ => true,
            }))
            .finish()
    }
}

impl Default for MultiStrategyConfig {
    fn default() -> Self {
        Self {
            on_fail_continue: true,
            allow_recursive: true,
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
        tx_sender: TransactionSender,
        ticket_aggregator: BasicTicketAggregationActions<std::result::Result<Ticket, String>>,
    ) -> Self
    where
        Db: HoprCoreEthereumDbActions + 'static,
        Net: NetworkExternalActions + 'static,
    {
        let mut strategies = Vec::<Box<dyn SingularStrategy>>::new();

        for strategy in cfg.strategies.iter() {
            match strategy {
                Strategy::Promiscuous(sub_cfg) => strategies.push(Box::new(PromiscuousStrategy::new(
                    sub_cfg.clone(),
                    db.clone(),
                    network.clone(),
                    tx_sender.clone(),
                ))),
                Strategy::Aggregating(sub_cfg) => strategies.push(Box::new(AggregatingStrategy::new(
                    sub_cfg.clone(),
                    db.clone(),
                    tx_sender.clone(),
                    ticket_aggregator.clone(),
                ))),
                Strategy::AutoRedeeming(sub_cfg) => strategies.push(Box::new(AutoRedeemingStrategy::new(
                    sub_cfg.clone(),
                    db.clone(),
                    tx_sender.clone(),
                ))),
                Strategy::AutoFunding(sub_cfg) => strategies.push(Box::new(AutoFundingStrategy::new(
                    sub_cfg.clone(),
                    db.clone(),
                    tx_sender.clone(),
                ))),
                Strategy::Multi(sub_cfg) => {
                    if cfg.allow_recursive {
                        let mut cfg_clone = sub_cfg.clone();
                        cfg_clone.allow_recursive = false; // Do not allow more levels of recursion

                        strategies.push(Box::new(Self::new(
                            cfg_clone,
                            db.clone(),
                            network.clone(),
                            tx_sender.clone(),
                            ticket_aggregator.clone(),
                        )))
                    } else {
                        error!("recursive multi-strategy not allowed and skipped")
                    }
                }
                // Passive strategy = empty Multistrategy
                Strategy::Passive => strategies.push(Box::new(Self {
                    cfg: Default::default(),
                    strategies: Vec::new(),
                })),
            }
        }

        Self { strategies, cfg }
    }
}

impl Display for MultiStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "multi_strategy[{}]",
            self.strategies
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

#[async_trait(? Send)]
impl SingularStrategy for MultiStrategy {
    async fn on_tick(&self) -> Result<()> {
        for strategy in self.strategies.iter() {
            if let Err(e) = strategy.on_tick().await {
                error!("error on_tick in strategy {strategy}: {e}");

                if !self.cfg.on_fail_continue {
                    warn!("{self} on_tick chain stopped at {strategy}");
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    async fn on_acknowledged_ticket(&self, ack: &AcknowledgedTicket) -> Result<()> {
        for strategy in self.strategies.iter() {
            if let Err(e) = strategy.on_acknowledged_ticket(ack).await {
                error!("error on_acknowledged_ticket in strategy {strategy}: {e}");

                if !self.cfg.on_fail_continue {
                    warn!("{self} on_acknowledged_ticket chain stopped at {strategy}");
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    async fn on_channel_state_changed(&self, channel: &ChannelEntry) -> Result<()> {
        for strategy in self.strategies.iter() {
            if let Err(e) = strategy.on_channel_state_changed(channel).await {
                error!("error on_channel_state_changed in strategy {strategy}: {e}");

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
    async fn test_multi_strategy_name() {
        let ms = MultiStrategy {
            strategies: vec![
                Box::new(MockSingularStrategy::new()),
                Box::new(MockSingularStrategy::new()),
            ],
            cfg: Default::default(),
        };
        assert_eq!("multi_strategy[mock,mock]", &ms.to_string());
    }

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
