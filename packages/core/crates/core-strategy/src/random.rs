use utils_types::primitives::Balance;

use crate::generic::{ChannelStrategy, OutgoingChannelStatus, StrategyTickResult};

/// Implements random strategy (cover traffic)
pub struct RandomStrategy;

impl ChannelStrategy for RandomStrategy {
    const NAME: &'static str = "random";

    fn tick<Q>(&self, _balance: Balance, _peer_ids: impl Iterator<Item=String>, _outgoing_channel_peer_ids: Vec<OutgoingChannelStatus>, _quality_of: Q) -> StrategyTickResult
        where Q: Fn(&str) -> Option<f64> {
        unimplemented!("Cover Traffic Strategy (Random strategy) not yet implemented!");
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
    use utils_types::primitives::wasm::Balance;

    use crate::generic::wasm::StrategyTickResult;

    #[wasm_bindgen]
    pub struct RandomStrategy {
        w: super::RandomStrategy,
    }

    #[wasm_bindgen]
    impl RandomStrategy {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            RandomStrategy {
                w: super::RandomStrategy {},
            }
        }

        #[wasm_bindgen(getter)]
        pub fn name(&self) -> String {
            self.w.name().into()
        }

        pub fn tick(&self, balance: Balance, peer_ids: &js_sys::Iterator, outgoing_channels: JsValue, quality_of: &js_sys::Function) ->  JsResult<StrategyTickResult> {
            crate::generic::wasm::tick_wrap(&self.w, balance, peer_ids, outgoing_channels, quality_of)
        }
    }
}
