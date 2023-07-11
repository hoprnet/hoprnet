use utils_log::debug;
use utils_types::primitives::{Address, Balance};

use crate::generic::{ChannelStrategy, OutgoingChannelStatus, StrategyTickResult};

/// Implements passive strategy which does nothing.
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct PassiveStrategy;

impl ChannelStrategy for PassiveStrategy {
    const NAME: &'static str = "passive";

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
        debug!("using passive strategy that does nothing");
        StrategyTickResult::new(0, vec![], vec![])
    }
}

#[cfg(test)]
mod tests {
    use crate::generic::ChannelStrategy;
    use crate::passive::PassiveStrategy;

    #[test]
    fn test_passive() {
        assert_eq!("passive", PassiveStrategy::NAME);
    }
}

/// WASM bindings
#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::generic::wasm::StrategyTickResult;
    use crate::generic::ChannelStrategy;
    use crate::passive::PassiveStrategy;
    use crate::strategy_tick;
    use std::str::FromStr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::{Address, Balance};
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;

    #[wasm_bindgen]
    impl PassiveStrategy {
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
