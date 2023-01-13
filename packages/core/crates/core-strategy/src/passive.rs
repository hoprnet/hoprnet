use utils_types::channels::ChannelEntry;
use utils_types::primitives::Balance;

use crate::generic::{ChannelStrategy, StrategyTickResult};

pub struct PassiveStrategy;

impl ChannelStrategy for PassiveStrategy {
    fn name(&self) -> &str {
        "passive"
    }

    fn tick<Q>(&self, _balance: Balance, _network_size: u32, _current_channels: &[&str], _quality_of: Q, _peer_ids: impl Iterator<Item=String>) -> StrategyTickResult
        where Q: Fn(&str) -> Option<f64> {

        StrategyTickResult{
            to_open: vec![],
            to_close: vec![]
        }
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use js_sys::JsString;
    use wasm_bindgen::prelude::wasm_bindgen;
    use crate::generic::ChannelStrategy;

    use utils_types::primitives::wasm::Balance;

    use crate::generic::wasm::StrategyTickResult;

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

        pub fn tick(&self, balance: Balance, network_size: u32, current_channels: Vec<JsString>, quality_of: &js_sys::Function, peer_ids: Vec<JsString>) ->  StrategyTickResult {
            crate::generic::wasm::tick_wrap(&self.w, balance, network_size, current_channels, quality_of, peer_ids)
        }
    }
}