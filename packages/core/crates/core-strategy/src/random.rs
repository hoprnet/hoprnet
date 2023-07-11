use utils_types::primitives::{Address, Balance};

use crate::generic::{ChannelStrategy, OutgoingChannelStatus, StrategyTickResult};

/// Implements random strategy (cover traffic)
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct RandomStrategy;

impl ChannelStrategy for RandomStrategy {
    const NAME: &'static str = "random";

    fn tick<Q>(
        &mut self,
        _balance: Balance,
        _addresses: impl Iterator<Item = Address>,
        _outgoing_channels: Vec<OutgoingChannelStatus>,
        _quality_of: Q,
    ) -> StrategyTickResult
    where
        Q: Fn(&str) -> Option<f64>,
    {
        unimplemented!("Cover Traffic Strategy (Random strategy) not yet implemented!");
    }
}

#[cfg(test)]
mod tests {
    use crate::generic::ChannelStrategy;
    use crate::random::RandomStrategy;

    #[test]
    fn test_random() {
        assert_eq!("random", RandomStrategy::NAME);
    }
}

/// WASM bindings
#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::generic::wasm::StrategyTickResult;
    use crate::generic::ChannelStrategy;
    use crate::random::RandomStrategy;
    use crate::strategy_tick;
    use std::str::FromStr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::{Address, Balance};
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;

    #[wasm_bindgen]
    impl RandomStrategy {
        #[wasm_bindgen(constructor)]
        pub fn _new() -> Self {
            Self {}
        }

        #[wasm_bindgen(getter)]
        pub fn name(&self) -> String {
            Self::NAME.into()
        }

        #[wasm_bindgen(js_name = "tick")]
        pub fn _tick(
            &mut self,
            balance: Balance,
            peer_ids: &js_sys::Iterator,
            outgoing_channels: JsValue,
            quality_of: &js_sys::Function,
        ) -> JsResult<StrategyTickResult> {
            strategy_tick!(self, balance, peer_ids, outgoing_channels, quality_of)
        }
    }
}
