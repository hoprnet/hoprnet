use utils_log::debug;
use utils_types::primitives::{Address, Balance};

use crate::generic::{ChannelStrategy, OutgoingChannelStatus, StrategyTickResult};

/// Implements passive strategy which does nothing.
#[derive(Debug, Clone)]
pub struct PassiveStrategy;

impl ChannelStrategy for PassiveStrategy {
    const NAME: &'static str = "passive";

    fn tick(
        &mut self,
        _balance: Balance,
        _addresses: impl Iterator<Item = (Address, f64)>,
        _outgoing_channels: Vec<OutgoingChannelStatus>,
    ) -> StrategyTickResult {
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
    use std::sync::Mutex;
    use crate::generic::wasm::StrategyTickResult;
    use crate::generic::{ChannelStrategy, PeerQuality};
    use crate::strategy_tick;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::Balance;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;
    use utils_log::error;

    #[wasm_bindgen]
    pub struct PassiveStrategy {
        w: Mutex<super::PassiveStrategy>
    }

    #[wasm_bindgen]
    impl PassiveStrategy {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self {
                w: Mutex::new(super::PassiveStrategy)
            }
        }

        #[wasm_bindgen(getter)]
        pub fn name(&self) -> String {
            super::PassiveStrategy::NAME.into()
        }

        pub fn tick(
            &self,
            balance: Balance,
            peers: PeerQuality,
            outgoing_channels: JsValue,
        ) -> JsResult<StrategyTickResult> {
            if let Ok(mut s) = self.w.lock() {
                strategy_tick!(s, balance, peers, outgoing_channels)
            } else {
                error!("could not lock for strategy tick");
                Err("strategy lock failed".into())
            }
        }
    }
}
