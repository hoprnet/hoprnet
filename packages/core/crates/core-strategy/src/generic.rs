use utils_types::channels::ChannelStatus;
use utils_types::primitives::Balance;
use utils_types::primitives::BalanceType::HOPR;

/// Basic strategy trait that all strategies must implement.
/// Strategies make decisions to automatically open/close certain channels.
/// The decision is done by the `tick` method based on the current node's balance, current view
/// of the network (all peer ids, currently opened outgoing channels) and quality estimations
/// of connections to the other peers in the network.
/// Other additional parameters affecting this decision could be members of implementors
/// of this trait.
pub trait ChannelStrategy {
    /// Human readable name of the strategy
    const NAME: &'static str;

    /// Human readable name of the strategy
    fn name(&self) -> &str {
        Self::NAME
    }

    /// Performs the strategy tick - deciding which new channels should be opened and
    /// which existing channels should be closed.
    fn tick<Q>(
        &self,
        balance: Balance,
        peer_ids: impl Iterator<Item = String>,
        outgoing_channels: Vec<OutgoingChannelStatus>,
        quality_of_peer: Q,
    ) -> StrategyTickResult
    where
        Q: Fn(&str) -> Option<f64>;

    /// Indicates if according to this strategy, a commitment should be made for the given channel.
    fn should_commit_to_channel(&self, _channel: &OutgoingChannelStatus) -> bool {
        true
    }
}

/// Represents a request to open a channel with a stake.
#[derive(Clone)]
pub struct OutgoingChannelStatus {
    pub peer_id: String,
    pub stake: Balance,
    pub status: ChannelStatus,
}

#[cfg(feature = "wasm")]
impl From<&wasm::OutgoingChannelStatus> for OutgoingChannelStatus {
    fn from(x: &wasm::OutgoingChannelStatus) -> Self {
        OutgoingChannelStatus {
            peer_id: x.peer_id.clone(),
            stake: Balance::from_str(x.stake_str.as_str(), HOPR),
            status: x.status.clone()
        }
    }
}

/// A decision made by a strategy on each tick,
/// represents which channels should be closed and which should be opened.
/// Also indicates a number of maximum channels this strategy can open given the current network size.
/// Note that the number changes as the network size changes.
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct StrategyTickResult {
    pub max_auto_channels: usize,
    to_open: Vec<OutgoingChannelStatus>,
    to_close: Vec<String>,
}

impl StrategyTickResult {
    /// Constructor for the strategy tick result.
    pub fn new(
        max_auto_channels: usize,
        to_open: Vec<OutgoingChannelStatus>,
        to_close: Vec<String>,
    ) -> Self {
        StrategyTickResult {
            max_auto_channels,
            to_open,
            to_close,
        }
    }

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
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;

    use crate::generic::{ChannelStrategy, StrategyTickResult};

    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;

    use serde::{Deserialize, Serialize};
    use utils_types::channels::ChannelStatus;
    use utils_types::primitives::Balance;

    #[derive(Serialize, Deserialize)]
    pub struct OutgoingChannelStatus {
        pub peer_id: String,
        pub stake_str: String,
        pub status: ChannelStatus,
    }

    impl From<&super::OutgoingChannelStatus> for OutgoingChannelStatus {
        fn from(x: &crate::generic::OutgoingChannelStatus) -> Self {
            OutgoingChannelStatus {
                peer_id: x.peer_id.clone(),
                stake_str: x.stake.to_string(),
                status: x.status.clone()
            }
        }
    }

    #[wasm_bindgen]
    impl StrategyTickResult {
        #[wasm_bindgen(constructor)]
        pub fn create(
            max_auto_channels: u32,
            to_open: JsValue,
            to_close: Vec<JsString>,
        ) -> JsResult<StrategyTickResult> {
            Ok(StrategyTickResult::new(
                    max_auto_channels as usize,
                    serde_wasm_bindgen::from_value::<Vec<OutgoingChannelStatus>>(to_open)?
                        .into_iter()
                        .map(|x| super::OutgoingChannelStatus::from(&x))
                        .collect(),
                    to_close.iter().map(String::from).collect(),
                )
            )
        }

        #[wasm_bindgen(js_name = "to_open")]
        pub fn channels_to_open(&self) -> JsResult<JsValue> {
            let ret: Vec<OutgoingChannelStatus> = self
                .to_open()
                .iter()
                .map(|s| OutgoingChannelStatus::from(s))
                .collect();

            ok_or_jserr!(serde_wasm_bindgen::to_value(&ret))
        }

        #[wasm_bindgen(js_name = "to_close")]
        pub fn channels_to_close(&self) -> Vec<JsString> {
            self.to_close()
                .iter()
                .map(|s| JsString::from(s.clone()))
                .collect()
        }
    }

    /// Generic binding for all strategies to use in WASM wrappers
    /// Since wasm_bindgen annotation is not supported on trait impls, the WASM-wrapped strategies cannot implement a common trait.
    pub fn tick_wrap<S: ChannelStrategy>(
        strategy: &S,
        balance: Balance,
        peer_ids: &js_sys::Iterator,
        outgoing_channels: JsValue,
        quality_of: &js_sys::Function,
    ) -> JsResult<StrategyTickResult> {
        Ok(strategy.tick(
                balance,
                peer_ids
                    .into_iter()
                    .map(|v| v.unwrap().as_string().unwrap()),
                serde_wasm_bindgen::from_value::<Vec<OutgoingChannelStatus>>(outgoing_channels)?
                    .iter()
                    .map(|c| super::OutgoingChannelStatus::from(c))
                    .collect(),
                |peer_id: &str| {
                        quality_of
                            .call1(&JsValue::null(), &JsString::from(peer_id))
                            .ok()
                            .map(|q| q.as_f64())
                            .flatten()
                    },
            )
        )
    }
}
