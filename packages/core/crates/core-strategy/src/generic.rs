use utils_types::channels::ChannelEntry;
use utils_types::primitives::Balance;

/// Basic strategy trait that all strategies must implement.
pub trait ChannelStrategy {

    /// Human readable name of the strategy
    fn name(&self) -> &str;

    /// Performs the strategy tick
    fn tick<Q>(&self,
            balance: Balance,
            peer_ids: impl Iterator<Item=String>,
            outgoing_channel_peer_ids: &[&str],
            quality_of: Q)
        -> StrategyTickResult
    where Q: Fn(&str) -> Option<f64>;

    /// Indicates if according to this strategy, a commitment should be made for the given channel.
    fn should_commit_to_channel(&self, _channel: &ChannelEntry) -> bool {
        true
    }
}

/// Represents a request to open a channel with a stake.
#[derive(Clone)]
pub struct ChannelOpenRequest {
    pub peer_id: String,
    pub stake: Balance
}

/// A decision made by the strategy on each tick,
/// represents which channels should be closed and which should be opened.
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

        pub fn stake(&self) -> Balance {
            self.w.stake.clone().into()
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

        pub fn to_open(&self, index: usize) -> ChannelOpenRequest {
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


    /// Generic binding for all strategies to use in WASM wrappers
    /// Since wasm_bindgen annotation is not supported on trait impls, the WASM-wrapped strategies cannot implement a common trait.
    pub fn tick_wrap<S: ChannelStrategy>(strategy: &S, balance: Balance, peer_ids: &js_sys::Iterator, outgoing_channel_peer_ids: Vec<JsString>, quality_of: &js_sys::Function) ->  StrategyTickResult {
        convert_from_jstrvec!(outgoing_channel_peer_ids, bind_ch);

        StrategyTickResult {
            w: strategy.tick(balance.w,
                             peer_ids.into_iter().map(|v| v.unwrap().as_string().unwrap()),
                             bind_ch.as_slice(),
                             | peer_id: &str | {
                               let this = JsValue::null();
                               let str = JsString::from(peer_id);

                               let quality = quality_of.call1(&this, &str);
                               quality.ok().map(|q| q.as_f64()).flatten()
                           })
        }
    }
}