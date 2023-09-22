use crate::errors::Result;
use async_trait::async_trait;
use core_types::acknowledgement::AcknowledgedTicket;
use core_types::channels::ChannelEntry;
use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use validator::Validate;
use utils_log::error;

/// Basic single strategy.
#[async_trait(? Send)]
pub trait SingularStrategy: Display {
    async fn on_tick(&self) -> Result<()> {
        Ok(())
    }
    async fn on_acknowledged_ticket(&self, _ack: &AcknowledgedTicket) -> Result<()> {
        Ok(())
    }
    async fn on_channel_state_changed(&self, _channel: &ChannelEntry) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq, Eq)]
pub struct MultiStrategyConfig {
    /// Determines if should continue executing the next strategy if the current one failed.
    pub on_fail_continue: bool
}

impl Default for MultiStrategyConfig {
    fn default() -> Self {
        Self {
            on_fail_continue: true
        }
    }
}

/// Defines an execution chain of `SingularStrategies`
/// The `MultiStrategy` itself also implements the `SingularStrategy` trait,
/// which makes it possible (along with different `on_fail_continue` policies) to construct
/// various conditional strategy chains.
pub struct MultiStrategy {
    strategies: Vec<Box<dyn SingularStrategy>>,
    cfg: MultiStrategyConfig
}

impl MultiStrategy {
    pub fn new(strategies: Vec<Box<dyn SingularStrategy>>, cfg: MultiStrategyConfig) -> Self {
        Self { strategies, cfg }
    }
}

impl Display for MultiStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MultiStrategy for {} strategies", self.strategies.len())
    }
}

#[async_trait(? Send)]
impl SingularStrategy for MultiStrategy {
    async fn on_tick(&self) -> Result<()> {
        for strat in self.strategies.iter() {
            if let Err(e) = strat.on_tick().await {
                error!("error on_tick in strategy {strat}: {e}");
                if !self.cfg.on_fail_continue {
                    break
                }
            }
        }
        Ok(())
    }

    async fn on_acknowledged_ticket(&self, ack: &AcknowledgedTicket) -> Result<()> {
        for strat in self.strategies.iter() {
            if let Err(e) = strat.on_acknowledged_ticket(ack).await {
                error!("error on_acknowledged_ticket in strategy {strat}: {e}");
                if !self.cfg.on_fail_continue {
                    break
                }
            }
        }
        Ok(())
    }

    async fn on_channel_state_changed(&self, channel: &ChannelEntry) -> Result<()> {
        for strat in self.strategies.iter() {
            if let Err(e) = strat.on_channel_state_changed(channel).await {
                error!("error on_channel_state_changed in strategy {strat}: {e}");
                if !self.cfg.on_fail_continue {
                    break
                }
            }
        }
        Ok(())
    }
}
