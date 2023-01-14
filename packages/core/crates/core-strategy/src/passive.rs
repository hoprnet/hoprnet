use utils_types::channels::ChannelEntry;
use utils_types::primitives::Balance;

use crate::generic::{ChannelStrategy, StrategyTickResult};

pub struct PassiveStrategy;

impl ChannelStrategy for PassiveStrategy {
    const NAME: &'static str = "passive";

    fn tick<Q>(&self, _balance: Balance, _peer_ids: impl Iterator<Item=String>, _outgoing_channel_peer_ids: &[&str], _quality_of: Q) -> StrategyTickResult
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

        pub fn tick(&self, balance: Balance, peer_ids: &js_sys::Iterator, outgoing_channel_peer_ids: Vec<JsString>, quality_of: &js_sys::Function) ->  StrategyTickResult {
            crate::generic::wasm::tick_wrap(&self.w, balance, peer_ids, outgoing_channel_peer_ids, quality_of)
        }
    }
}