use utils_types::primitives::Balance;

/// Basic strategy trait that all strategies must implement.
pub trait ChannelStrategy {

    /// Human readable name of the strategy
    const NAME: &'static str;

    /// Human readable name of the strategy
    fn name(&self) -> &str { Self::NAME }

    /// Performs the strategy tick - deciding which new channels should be opened and
    /// which existing channels should be closed.
    fn tick<Q>(&self,
            balance: Balance,
            peer_ids: impl Iterator<Item=String>,
            outgoing_channels: Vec<OutgoingChannelStatus>,
            quality_of: Q)
        -> StrategyTickResult
    where Q: Fn(&str) -> Option<f64>;

    /// Indicates if according to this strategy, a commitment should be made for the given channel.
    fn should_commit_to_channel(&self, _channel: &OutgoingChannelStatus) -> bool {
        true
    }
}

/// Represents a request to open a channel with a stake.
pub struct OutgoingChannelStatus {
    pub peer_id: String,
    pub stake: Balance
}

#[cfg(feature = "wasm")]
impl From<&wasm::OutgoingChannelStatus> for OutgoingChannelStatus {
    fn from(x: &wasm::OutgoingChannelStatus) -> Self {
        OutgoingChannelStatus {
            peer_id: x.peer_id.clone(),
            stake: Balance::from_str(x.stake_str.as_str()).unwrap()
        }
    }
}

/// A decision made by a strategy on each tick,
/// represents which channels should be closed and which should be opened.
/// Also indicates a number of maximum channels this strategy can open given the current network size.
/// Note that the number changes as the network size changes.
pub struct StrategyTickResult {
    max_auto_channels: usize,
    to_open: Vec<OutgoingChannelStatus>,
    to_close: Vec<String>
}

impl StrategyTickResult {

    /// Constructor for the strategy tick result.
    pub fn new(max_auto_channels: usize, to_open: Vec<OutgoingChannelStatus>, to_close: Vec<String>) -> Self {
        StrategyTickResult {
            max_auto_channels,
            to_open,
            to_close
        }
    }

    /// Maximum number of channels this strategy can open.
    pub fn max_auto_channels(&self) -> usize { self.max_auto_channels }

    /// Channels that this strategy wishes to open.
    pub fn to_open(&self) -> &Vec<OutgoingChannelStatus> {
        &self.to_open
    }

    /// Peer IDs to which any open channels should be closed according to this strategy.
    pub fn to_close(&self) -> &Vec<String> {
        &self.to_close
    }
}

/// WASM bindings for the generic strategy-related classes
#[cfg(feature = "wasm")]
pub mod wasm {
    use js_sys::JsString;
    use wasm_bindgen::JsValue;
    use wasm_bindgen::prelude::wasm_bindgen;

    use crate::generic::ChannelStrategy;

    use utils_types::primitives::wasm::Balance;
    use utils_misc::utils::wasm::JsResult;
    use utils_misc::ok_or_jserr;

    use serde::{Serialize, Deserialize};

    #[wasm_bindgen(getter_with_clone)]
    #[derive(Serialize, Deserialize)]
    pub struct OutgoingChannelStatus {
        pub peer_id: String,
        pub stake_str: String
    }

    #[wasm_bindgen]
    impl OutgoingChannelStatus {

        #[wasm_bindgen(constructor)]
        pub fn new(peer_id: &str, stake_str: &str) -> Self {
            OutgoingChannelStatus {
                peer_id: peer_id.to_string(),
                stake_str: stake_str.to_string()
            }
        }
    }

    impl From<&super::OutgoingChannelStatus> for OutgoingChannelStatus {
        fn from(x: &crate::generic::OutgoingChannelStatus) -> Self {
            OutgoingChannelStatus {
                peer_id: x.peer_id.clone(),
                stake_str: x.stake.to_string()
            }
        }
    }

    #[wasm_bindgen]
    pub struct StrategyTickResult {
        pub(crate) w: super::StrategyTickResult
    }

    #[wasm_bindgen]
    impl StrategyTickResult {
        #[wasm_bindgen(constructor)]
        pub fn new(max_auto_channels: u32, to_open: JsValue, to_close: Vec<JsString>) -> JsResult<StrategyTickResult> {
            let open: Vec<OutgoingChannelStatus> = ok_or_jserr!(serde_wasm_bindgen::from_value(to_open))?;
            Ok(StrategyTickResult {
                w: super::StrategyTickResult::new(max_auto_channels as usize,
                                                  open.into_iter().map(|x| super::OutgoingChannelStatus::from(&x)).collect(),
                                                  to_close.iter().map(String::from).collect())
            })
        }

        #[wasm_bindgen(getter)]
        pub fn max_auto_channels(&self) -> usize { self.w.max_auto_channels }

        pub fn to_open(&self) -> JsResult<JsValue> {
            let ret: Vec<OutgoingChannelStatus> = self.w
                .to_open()
                .iter()
                .map(|s| OutgoingChannelStatus::from(s)).collect();

            ok_or_jserr!(serde_wasm_bindgen::to_value(&ret))
        }

        pub fn to_close(&self) -> Vec<JsString> {
            self.w
                .to_close()
                .iter()
                .map(|s| JsString::from(s.clone()))
                .collect()
        }
    }


    /// Generic binding for all strategies to use in WASM wrappers
    /// Since wasm_bindgen annotation is not supported on trait impls, the WASM-wrapped strategies cannot implement a common trait.
    pub fn tick_wrap<S: ChannelStrategy>(strategy: &S, balance: Balance, peer_ids: &js_sys::Iterator, outgoing_channels: JsValue, quality_of: &js_sys::Function) ->  JsResult<StrategyTickResult> {

        let out_channels: Vec<OutgoingChannelStatus> = serde_wasm_bindgen::from_value(outgoing_channels)?;

        Ok(StrategyTickResult {
            w: strategy.tick(balance.w,
                             peer_ids.into_iter().map(|v| v.unwrap().as_string().unwrap()),
                             out_channels.iter().map(|c| super::OutgoingChannelStatus::from(c)).collect(),
                             | peer_id: &str | {
                               let this = JsValue::null();
                               let str = JsString::from(peer_id);

                               let quality = quality_of.call1(&this, &str);
                               quality.ok().map(|q| q.as_f64()).flatten()
                           })
        })
    }
}