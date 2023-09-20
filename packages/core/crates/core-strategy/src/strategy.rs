use async_trait::async_trait;
use core_types::acknowledgement::wasm::AcknowledgedTicket;
use core_types::channels::ChannelEntry;
use utils_log::error;
use crate::errors::Result;

#[async_trait(? Send)]
pub trait SingularStrategy {
    fn name(&self) -> String;
    async fn on_tick(&self) -> Result<()> {
        Ok(())
    }
    async fn on_acknowledged_ticket(&self, _ack: &AcknowledgedTicket) -> Result<()> {
        Ok(())
    }
    async fn on_channel_close(&self, _channel: &ChannelEntry) -> Result<()> {
        Ok(())
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct MultiStrategy {
    strategies: Vec<Box<dyn SingularStrategy>>
}

impl MultiStrategy {
    pub fn new(strategies: Vec<Box<dyn SingularStrategy>>) -> Self {
        Self { strategies }
    }
}

#[async_trait(? Send)]
impl SingularStrategy for MultiStrategy {
    fn name(&self) -> String {
        format!("MultiStrategy for {} strategies", self.strategies.len())
    }

    async fn on_tick(&self) -> Result<()> {
        for strat in self.strategies.iter() {
            if let Err(e) = strat.on_tick().await {
                error!("error on_tick in strategy {}: {e}", strat.name())
            }
        }
        Ok(())
    }

    async fn on_acknowledged_ticket(&self, ack: &AcknowledgedTicket) -> Result<()> {
        for strat in self.strategies.iter() {
            if let Err(e) = strat.on_acknowledged_ticket(ack).await {
                error!("error on_tick in strategy {}: {e}", strat.name())
            }
        }
        Ok(())
    }

    async fn on_channel_close(&self, channel: &ChannelEntry) -> Result<()> {
        for strat in self.strategies.iter() {
            if let Err(e) = strat.on_channel_close(channel).await {
                error!("error on_tick in strategy {}: {e}", strat.name())
            }
        }
        Ok(())
    }
}

