use utils_types::channels::ChannelEntry;
use utils_types::primitives::Balance;

use crate::generic::{ChannelStrategy, StrategyTickResult};

pub struct PromiscuousStrategy ;


impl ChannelStrategy for PromiscuousStrategy {
    fn name(&self) -> &str {
        "promiscuous"
    }

    fn tick<Q>(&self, balance: Balance, network_size: u32, outgoing_channel_peer_ids: &[&str], quality_of: Q, peer_ids: &[&str]) -> StrategyTickResult
        where Q: Fn(&str) -> Option<f64> {
        todo!()
    }
}


/// Unit tests of pure Rust code
#[cfg(test)]
mod tests {
    use super::*;

}

/// Module for WASM wrappers of Rust code
#[cfg(feature = "wasm")]
pub mod wasm {
    use js_sys::JsString;
    use wasm_bindgen::prelude::*;

    use utils_types::primitives::wasm::Balance;

    use crate::generic::ChannelStrategy;
    use crate::generic::wasm::StrategyTickResult;

    #[wasm_bindgen]
    pub struct PromiscuousStrategy {
        w: super::PromiscuousStrategy
    }

    #[wasm_bindgen]
    impl PromiscuousStrategy {

        #[wasm_bindgen(getter)]
        pub fn name(&self) -> String {
            self.w.name().into()
        }

        pub fn tick(&self, balance: Balance, network_size: u32, current_channels: Vec<JsString>, quality_of: &js_sys::Function, peer_ids: Vec<JsString>) ->  StrategyTickResult {
            crate::generic::wasm::tick_wrap(&self.w, balance, network_size, current_channels, quality_of, peer_ids)
        }
    }
}

