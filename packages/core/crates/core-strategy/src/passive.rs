use async_trait::async_trait;
use utils_types::channels::{AcknowledgedTicket, ChannelEntry};
use utils_types::primitives::Balance;

use crate::generic::{ChannelStrategy, OutgoingChannelStatus, StrategyTickResult};

/// Implements passive strategy which does nothing.
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct PassiveStrategy;

#[async_trait]
impl ChannelStrategy for PassiveStrategy {
    const NAME: &'static str = "passive";

    fn tick<Q>(
        &self,
        _balance: Balance,
        _peer_ids: impl Iterator<Item = String>,
        _outgoing_channel_peer_ids: Vec<OutgoingChannelStatus>,
        _quality_of: Q,
    ) -> StrategyTickResult
    where
        Q: Fn(&str) -> Option<f64>,
    {
        StrategyTickResult::new(0, vec![], vec![])
    }

    // Re-implementations to satisfy the trait, because
    // we cannot put #[wasm_bindgen] on trait impl blocks

    async fn on_winning_ticket(&self, ack_ticket: &AcknowledgedTicket) {
        self.on_winning_ticket(ack_ticket).await
    }

    async fn on_channel_closing(&self, channel: &ChannelEntry) {
        self.on_channel_closing(channel).await
    }

    fn should_commit_to_channel(&self, channel: &ChannelEntry) -> bool {
        self.should_commit_to_channel(channel)
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl PassiveStrategy {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new() -> Self {
        PassiveStrategy {}
    }

    pub async fn on_winning_ticket(&self, _ack_ticket: &AcknowledgedTicket) {
        // Passive strategy does nothing
    }

    pub async fn on_channel_closing(&self, _channel: &ChannelEntry) {
        // Passive strategy does nothing
    }

    pub fn should_commit_to_channel(&self, _channel: &ChannelEntry) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::generic::ChannelStrategy;
    use crate::passive::PassiveStrategy;

    #[test]
    fn test_passive() {
        let strat = PassiveStrategy {};
        assert_eq!("passive", strat.name());
    }
}

/// WASM bindings
#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::generic::{ChannelStrategy, StrategyTickResult};
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;

    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::Balance;
    use crate::generic::wasm::WasmChannelStrategy;
    use crate::passive::PassiveStrategy;

    impl WasmChannelStrategy for PassiveStrategy { }

    #[wasm_bindgen]
    impl PassiveStrategy {
        #[wasm_bindgen(getter, js_name="name")]
        pub fn strategy_name(&self) -> String {
            self.name().into()
        }

        #[wasm_bindgen(js_name="tick")]
        pub fn strategy_tick(
            &self,
            balance: Balance,
            peer_ids: &js_sys::Iterator,
            outgoing_channels: JsValue,
            quality_of: &js_sys::Function,
        ) -> JsResult<StrategyTickResult> {
            self.wrapped_tick(balance, peer_ids, outgoing_channels, quality_of)
        }
    }
}
