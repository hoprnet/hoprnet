use async_trait::async_trait;
use core_types::acknowledgement::wasm::AcknowledgedTicket;
use core_types::channels::ChannelEntry;
use utils_log::error;
use crate::errors::Result;

#[async_trait]
pub trait SingularStrategy {
    fn name(&self) -> String;

    async fn on_tick(&self) -> Result<()> {
        Ok(())
    }
    async fn on_acknowledged_ticket(&self, ack: &AcknowledgedTicket) -> Result<()> {
        Ok(())
    }
    async fn on_channel_close(&self, channel: &ChannelEntry) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct MultiStrategy {
    strategies: Vec<Box<dyn SingularStrategy>>
}

impl MultiStrategy {
    pub fn new(strategies: Vec<Box<dyn SingularStrategy>>) -> Self {
        Self { strategies }
    }
}

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

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::wasm_bindgen;
    use core_types::acknowledgement::wasm::AcknowledgedTicket;
    use core_types::channels::ChannelEntry;
    use utils_misc::utils::wasm::JsResult;
    use crate::strategy::{MultiStrategy, SingularStrategy};

    #[wasm_bindgen]
    impl MultiStrategy {
        #[wasm_bindgen(js_name = "on_acknowledged_ticket")]
        pub async fn _on_acknowledged_ticket(&self, ack: &AcknowledgedTicket) -> JsResult<()> {
            Ok(self.on_acknowledged_ticket(ack.clone().into()).await?)
        }

        #[wasm_bindgen(js_name = "on_channel_close")]
        pub async fn _on_channel_close(&self, channel: &ChannelEntry) -> JsResult<()> {
            Ok(self.on_channel_close(channel.clone().into()).await?)
        }
    }
}
