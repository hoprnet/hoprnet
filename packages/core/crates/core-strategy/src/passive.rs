use utils_types::primitives::Balance;

use crate::generic::{ChannelStrategy, OutgoingChannelStatus, StrategyTickResult};

/// Implements passive strategy which does nothing.
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct PassiveStrategy;

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
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl PassiveStrategy {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new() -> Self {
        PassiveStrategy {}
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
    use crate::passive::PassiveStrategy;

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
            crate::generic::wasm::tick_wrap(
                self,
                balance,
                peer_ids,
                outgoing_channels,
                quality_of,
            )
        }
    }
}
