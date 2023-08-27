use core_types::channels::ChannelStatus;
use utils_types::primitives::{Address, Balance};

#[cfg(feature = "wasm")]
use std::str::FromStr;

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

    fn name(&self) -> String {
        Self::NAME.into()
    }

    /// Performs the strategy tick - deciding which new channels should be opened and
    /// which existing channels should be closed.
    fn tick(
        &mut self,
        balance: Balance,
        addresses: impl Iterator<Item = (Address, f64)>,
        outgoing_channels: Vec<OutgoingChannelStatus>,
    ) -> StrategyTickResult;

    /// Indicates if according to this strategy, a commitment should be made for the given channel.
    fn should_commit_to_channel(&self, _channel: &OutgoingChannelStatus) -> bool {
        true
    }
}

/// Represents a request to open a channel with a stake.
#[derive(Clone, Debug)]
pub struct OutgoingChannelStatus {
    pub address: Address,
    pub stake: Balance,
    pub status: ChannelStatus,
}

#[cfg(feature = "wasm")]
impl From<&wasm::OutgoingChannelStatus> for OutgoingChannelStatus {
    fn from(x: &wasm::OutgoingChannelStatus) -> Self {
        OutgoingChannelStatus {
            address: Address::from_str(&x.address).expect("invalid peer id given"),
            stake: Balance::from_str(x.stake_str.as_str(), utils_types::primitives::BalanceType::HOPR),
            status: x.status.clone(),
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
    to_close: Vec<Address>,
}

impl StrategyTickResult {
    /// Constructor for the strategy tick result.
    pub fn new(max_auto_channels: usize, to_open: Vec<OutgoingChannelStatus>, to_close: Vec<Address>) -> Self {
        StrategyTickResult {
            max_auto_channels,
            to_open,
            to_close,
        }
    }

    /// Maximum number of channels this strategy can open.
    /// This number changes based on the network size.
    pub fn max_auto_channels(&self) -> usize {
        self.max_auto_channels
    }

    /// Channels that this strategy wishes to open.
    pub fn to_open(&self) -> &Vec<OutgoingChannelStatus> {
        &self.to_open
    }

    /// Peer IDs to which any open channels should be closed according to this strategy.
    pub fn to_close(&self) -> &Vec<Address> {
        &self.to_close
    }
}

/// Object needed only to simplify the iteration over the address and quality pair until
/// the strategy is migrated into Rust
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct PeerQuality {
    peers_with_quality: Vec<(Address, f64)>,
}

impl PeerQuality {
    pub fn new(peers: Vec<(Address, f64)>) -> Self {
        Self {
            peers_with_quality: peers,
        }
    }

    pub fn take(&mut self) -> Vec<(Address, f64)> {
        self.peers_with_quality.clone()
    }
}

/// WASM bindings for the generic strategy-related classes
#[cfg(feature = "wasm")]
pub mod wasm {
    use core_types::channels::ChannelStatus;
    use js_sys::JsString;
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::Address;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;

    #[derive(Serialize, Deserialize)]
    pub struct OutgoingChannelStatus {
        pub address: String,
        pub stake_str: String,
        pub status: ChannelStatus,
    }

    impl From<&super::OutgoingChannelStatus> for OutgoingChannelStatus {
        fn from(x: &crate::generic::OutgoingChannelStatus) -> Self {
            OutgoingChannelStatus {
                address: x.address.to_string(),
                stake_str: x.stake.to_string(),
                status: x.status.clone(),
            }
        }
    }

    #[wasm_bindgen]
    pub struct StrategyTickResult {
        pub(crate) w: super::StrategyTickResult,
    }

    #[wasm_bindgen]
    impl StrategyTickResult {
        #[wasm_bindgen(constructor)]
        pub fn new(
            max_auto_channels: u32,
            to_open_js: JsValue,
            to_close_js: Vec<JsString>,
        ) -> JsResult<StrategyTickResult> {
            let to_open = serde_wasm_bindgen::from_value::<Vec<OutgoingChannelStatus>>(to_open_js)?
                .into_iter()
                .map(|x| super::OutgoingChannelStatus::from(&x))
                .collect();

            let to_close = to_close_js
                .iter()
                .map(|s| Address::from_str(s.as_string().unwrap().as_str()).unwrap())
                .collect();

            Ok(StrategyTickResult {
                w: super::StrategyTickResult::new(max_auto_channels as usize, to_open, to_close),
            })
        }

        #[wasm_bindgen(getter)]
        pub fn max_auto_channels(&self) -> usize {
            self.w.max_auto_channels
        }

        pub fn to_open(&self) -> JsResult<JsValue> {
            let ret: Vec<OutgoingChannelStatus> = self
                .w
                .to_open()
                .iter()
                .map(|s| OutgoingChannelStatus::from(s))
                .collect();

            ok_or_jserr!(serde_wasm_bindgen::to_value(&ret))
        }

        pub fn to_close(&self) -> Vec<JsString> {
            self.w
                .to_close()
                .iter()
                .map(|s| JsString::from(s.to_string()))
                .collect()
        }
    }

    /// Generic binding for all strategies to use in WASM wrappers
    /// Since wasm_bindgen annotation is not supported on trait impls, the WASM-wrapped strategies cannot implement a common trait.
    #[macro_export]
    macro_rules! strategy_tick {
        ($strategy:ident, $balance:ident, $peers:ident, $outgoing_channels:ident) => {
            Ok(StrategyTickResult {
                w: $strategy.tick(
                    $balance,
                    $peers.take().into_iter(),
                    serde_wasm_bindgen::from_value::<Vec<crate::generic::wasm::OutgoingChannelStatus>>(
                        $outgoing_channels,
                    )?
                    .iter()
                    .map(|c| crate::generic::OutgoingChannelStatus::from(c))
                    .collect(),
                ),
            })
        };
    }
}
