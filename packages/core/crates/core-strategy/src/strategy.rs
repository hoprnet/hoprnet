use std::fmt::{Display, Formatter};
use async_trait::async_trait;
use core_types::acknowledgement::AcknowledgedTicket;
use core_types::channels::{ChannelEntry};
use utils_log::error;
use crate::errors::Result;

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

pub struct MultiStrategy {
    strategies: Vec<Box<dyn SingularStrategy >>
}

impl MultiStrategy {
    pub fn new(strategies: Vec<Box<dyn SingularStrategy>>) -> Self {
        Self { strategies }
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
                error!("error on_tick in strategy {strat}: {e}")
            }
        }
        Ok(())
    }

    async fn on_acknowledged_ticket(&self, ack: &AcknowledgedTicket) -> Result<()> {
        for strat in self.strategies.iter() {
            if let Err(e) = strat.on_acknowledged_ticket(ack).await {
                error!("error on_tick in strategy {strat}: {e}")
            }
        }
        Ok(())
    }

    async fn on_channel_state_changed(&self, channel: &ChannelEntry) -> Result<()> {
        for strat in self.strategies.iter() {
            if let Err(e) = strat.on_channel_state_changed(channel).await {
                error!("error on_tick in strategy {strat}: {e}")
            }
        }
        Ok(())
    }
}

