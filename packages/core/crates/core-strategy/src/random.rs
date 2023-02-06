use async_trait::async_trait;
use utils_types::channels::{AcknowledgedTicket, ChannelEntry};
use utils_types::primitives::Balance;

use crate::generic::{ChannelStrategy, OutgoingChannelStatus, StrategyTickResult};

/// Implements random strategy (cover traffic)
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct RandomStrategy;

#[async_trait]
impl ChannelStrategy for RandomStrategy {
    const NAME: &'static str = "random";

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
        unimplemented!("Cover Traffic Strategy (Random strategy) not yet implemented!");
    }

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
impl RandomStrategy {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new() -> Self {
        RandomStrategy {}
    }

    pub async fn on_winning_ticket(&self, _ack_ticket: &AcknowledgedTicket) {
        unimplemented!()
    }

    pub async fn on_channel_closing(&self, _channel: &ChannelEntry) {
        unimplemented!()
    }

    pub fn should_commit_to_channel(&self, _channel: &ChannelEntry) -> bool {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use crate::generic::ChannelStrategy;
    use crate::random::RandomStrategy;

    #[test]
    fn test_random() {
        let strat = RandomStrategy {};
        assert_eq!("random", strat.name());
    }
}

/// WASM bindings
#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::generic::ChannelStrategy;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;

    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::Balance;

    use crate::generic::StrategyTickResult;
    use crate::generic::wasm::WasmChannelStrategy;
    use crate::random::RandomStrategy;

    impl WasmChannelStrategy for RandomStrategy {}

    #[wasm_bindgen]
    impl RandomStrategy {
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
            self.wrapped_tick(
                balance,
                peer_ids,
                outgoing_channels,
                quality_of,
            )
        }
    }
}
