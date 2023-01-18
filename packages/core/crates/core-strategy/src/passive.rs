use utils_types::primitives::Balance;

use crate::generic::{ChannelStrategy, OutgoingChannelStatus, StrategyTickResult};

/// Implements passive strategy which does nothing.
pub struct PassiveStrategy;

impl ChannelStrategy for PassiveStrategy {
    const NAME: &'static str = "passive";

    fn tick<Q>(&self, _balance: Balance, _peer_ids: impl Iterator<Item=String>, _outgoing_channel_peer_ids: Vec<OutgoingChannelStatus>, _quality_of: Q) -> StrategyTickResult
        where Q: Fn(&str) -> Option<f64> {

        StrategyTickResult::new(0, vec![], vec![])
    }
}

/// WASM bindings
#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::JsValue;
    use wasm_bindgen::prelude::wasm_bindgen;
    use crate::generic::ChannelStrategy;

    use utils_types::primitives::wasm::Balance;

    use crate::generic::wasm::{JsResult, StrategyTickResult};

    #[wasm_bindgen]
    pub struct PassiveStrategy {
        w: super::PassiveStrategy
    }

    #[wasm_bindgen]
    impl PassiveStrategy {

        #[wasm_bindgen(getter)]
        pub fn name(&self) -> String {
            self.w.name().into()
        }

        pub fn tick(&self, balance: Balance, peer_ids: &js_sys::Iterator, outgoing_channels: JsValue, quality_of: &js_sys::Function) ->  JsResult<StrategyTickResult> {
            crate::generic::wasm::tick_wrap(&self.w, balance, peer_ids, outgoing_channels, quality_of)
        }
    }
}