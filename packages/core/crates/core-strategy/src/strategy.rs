use crate::errors::Result;
use async_trait::async_trait;
use core_types::acknowledgement::AcknowledgedTicket;
use core_types::channels::ChannelEntry;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
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
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq, Eq)]
pub struct MultiStrategyConfig {
    /// Determines if the strategy should continue executing the next strategy if the current one failed.
    /// If set to `true`, the strategy behaves like a logical AND chain of `SingularStrategies`
    /// Otherwise, it behaves like a logical OR chain of `SingularStrategies`.
    /// Default is `true`.
    pub on_fail_continue: bool,
}

impl Default for MultiStrategyConfig {
    fn default() -> Self {
        Self { on_fail_continue: true }
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
    /// Creates new `MultiStrategy` from the list of `SingularStrategies`
    pub fn new(strategies: Vec<Box<dyn SingularStrategy>>, cfg: MultiStrategyConfig) -> Self {
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
        let ms = MultiStrategy::new(
            vec![
                Box::new(MockSingularStrategy::new()),
                Box::new(MockSingularStrategy::new()),
            ],
            Default::default(),
        );
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

        let cfg = MultiStrategyConfig { on_fail_continue: true };

        let ms = MultiStrategy::new(vec![Box::new(s1), Box::new(s2)], cfg);
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
        };

        let ms = MultiStrategy::new(vec![Box::new(s1), Box::new(s2)], cfg);
        ms.on_tick().await.expect_err("on_tick should fail");
    }
}
