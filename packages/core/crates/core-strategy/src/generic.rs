use utils_types::channels::ChannelEntry;
use utils_types::primitives::Balance;

pub trait ChannelStrategy {
    fn name(&self) -> &str;

    fn tick<Q>(&self,
            balance: Balance,
            network_size: u32,
            outgoing_channel_peer_ids: &[&str],
            quality_of: Q,
            peer_ids: &[&str])
        -> StrategyTickResult
    where Q: Fn(&str) -> Option<f64>;
}

#[derive(Clone)]
pub struct ChannelOpenRequest {
    pub peer_id: String,
    pub stake: f64
}

pub struct StrategyTickResult {
    pub(crate) to_open: Vec<ChannelOpenRequest>,
    pub(crate) to_close: Vec<String>
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use js_sys::JsString;
    use wasm_bindgen::JsValue;
    use wasm_bindgen::prelude::wasm_bindgen;

    use crate::generic::ChannelStrategy;

    use utils_types::primitives::wasm::Balance;

    macro_rules! convert_from_jstrvec {
        ($v:expr,$r:ident) => {
            let _aux: Vec<String> = $v.iter().map(String::from).collect();
            let $r = _aux.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
        };
    }

    #[wasm_bindgen]
    pub struct ChannelOpenRequest {
        w: super::ChannelOpenRequest
    }

    #[wasm_bindgen]
    impl ChannelOpenRequest {
        pub fn peer_id(&self) -> String {
            self.w.peer_id.clone()
        }

        pub fn stake(&self) -> f64 {
            self.w.stake
        }
    }

    #[wasm_bindgen]
    pub struct StrategyTickResult {
        pub(crate) w: super::StrategyTickResult
    }

    #[wasm_bindgen]
    impl StrategyTickResult {
        pub fn to_open_count(&self) -> usize {
            self.w.to_open.len()
        }

        pub fn to_open(&self,index: usize) -> ChannelOpenRequest {
            ChannelOpenRequest {
                w: self.w.to_open[index].clone()
            }
        }

        pub fn to_close_count(&self) -> usize {
            self.w.to_close.len()
        }

        pub fn to_close_peer(&self, index: usize) -> String {
            self.w.to_close[index].clone()
        }
    }

    pub fn tick_wrap<S>(strategy: &S, balance: Balance, network_size: u32, current_channels: Vec<JsString>, quality_of: &js_sys::Function, peer_ids: Vec<JsString>) ->  StrategyTickResult
        where S: ChannelStrategy {
        convert_from_jstrvec!(current_channels, bind_ch);
        convert_from_jstrvec!(peer_ids, bind_p);

        StrategyTickResult {
            w: strategy.tick(balance.w,network_size,bind_ch.as_slice(),
                           | peer_id: &str | {
                               let this = JsValue::null();
                               let str = JsString::from(peer_id);

                               let quality = quality_of.call1(&this, &str);
                               quality.ok().map(|q| q.as_f64()).flatten()
                           },
                           bind_p.as_slice())
        }
    }
}