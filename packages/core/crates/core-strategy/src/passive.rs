use utils_log::debug;
use utils_types::primitives::Balance;

use crate::generic::{ChannelStrategy, OutgoingChannelStatus, StrategyTickResult};

/// Implements passive strategy which does nothing.
pub struct PassiveStrategy;

impl ChannelStrategy for PassiveStrategy {
    const NAME: &'static str = "passive";

    fn tick<Q>(
        &mut self,
        _balance: Balance,
        _peer_ids: impl Iterator<Item = String>,
        _outgoing_channel_peer_ids: Vec<OutgoingChannelStatus>,
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
        let strat = PassiveStrategy {};
        assert_eq!("passive", strat.name());
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

    use crate::generic::wasm::StrategyTickResult;

    #[wasm_bindgen]
    pub struct PassiveStrategy {
        w: super::PassiveStrategy,
    }

    #[wasm_bindgen]
    impl PassiveStrategy {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            PassiveStrategy {
                w: super::PassiveStrategy {},
            }
        }

        pub fn configure(&mut self, _settings: JsValue) -> JsResult<()> {
            Ok(())
        }

        #[wasm_bindgen(getter)]
        pub fn name(&self) -> String {
            self.w.name().into()
        }

        pub fn tick(
            &mut self,
            balance: Balance,
            peer_ids: &js_sys::Iterator,
            outgoing_channels: JsValue,
            quality_of: &js_sys::Function,
        ) -> JsResult<StrategyTickResult> {
            crate::generic::wasm::tick_wrap(&mut self.w, balance, peer_ids, outgoing_channels, quality_of)
        }
    }
}
