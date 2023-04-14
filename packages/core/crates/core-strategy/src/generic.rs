use core_types::channels::ChannelStatus;
use libp2p_identity::PeerId;
use std::str::FromStr;
use utils_types::primitives::{Balance, BalanceType};

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

    /// Performs the strategy tick - deciding which new channels should be opened and
    /// which existing channels should be closed.
    fn tick<Q>(
        &mut self,
        balance: Balance,
        peer_ids: impl Iterator<Item = PeerId>,
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
#[derive(Clone, Debug)]
pub struct OutgoingChannelStatus {
    pub peer_id: PeerId,
    pub stake: Balance,
    pub status: ChannelStatus,
}

#[cfg(feature = "wasm")]
impl From<&wasm::OutgoingChannelStatus> for OutgoingChannelStatus {
    fn from(x: &wasm::OutgoingChannelStatus) -> Self {
        OutgoingChannelStatus {
            peer_id: PeerId::from_str(&x.peer_id).expect("invalid peer id given"),
            stake: Balance::from_str(x.stake_str.as_str(), BalanceType::HOPR),
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
    to_close: Vec<PeerId>,
}

impl StrategyTickResult {
    /// Constructor for the strategy tick result.
    pub fn new(max_auto_channels: usize, to_open: Vec<OutgoingChannelStatus>, to_close: Vec<PeerId>) -> Self {
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
    pub fn to_close(&self) -> &Vec<PeerId> {
        &self.to_close
    }
}

/// WASM bindings for the generic strategy-related classes
#[cfg(feature = "wasm")]
pub mod wasm {
    use js_sys::JsString;
    use libp2p_identity::PeerId;
    use std::str::FromStr;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;

    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;

    use core_types::channels::ChannelStatus;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct OutgoingChannelStatus {
        pub peer_id: String,
        pub stake_str: String,
        pub status: ChannelStatus,
    }

    impl From<&super::OutgoingChannelStatus> for OutgoingChannelStatus {
        fn from(x: &crate::generic::OutgoingChannelStatus) -> Self {
            OutgoingChannelStatus {
                peer_id: x.peer_id.to_base58(),
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
        pub fn new(max_auto_channels: u32, to_open: JsValue, to_close: Vec<JsString>) -> JsResult<StrategyTickResult> {
            Ok(StrategyTickResult {
                w: super::StrategyTickResult::new(
                    max_auto_channels as usize,
                    serde_wasm_bindgen::from_value::<Vec<OutgoingChannelStatus>>(to_open)?
                        .into_iter()
                        .map(|x| super::OutgoingChannelStatus::from(&x))
                        .collect(),
                    to_close
                        .iter()
                        .map(|s| {
                            PeerId::from_str(String::from(s).as_str())
                                .expect(format!("invalid peer id given: {0}", s).as_str())
                        })
                        .collect(),
                ),
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
                .map(|s| JsString::from(s.to_base58()))
                .collect()
        }
    }

    /// Generic binding for all strategies to use in WASM wrappers
    /// Since wasm_bindgen annotation is not supported on trait impls, the WASM-wrapped strategies cannot implement a common trait.
    #[macro_export]
    macro_rules! strategy_tick {
        ($strategy:ident, $balance:ident, $peer_ids:ident, $outgoing_channels:ident, $quality_of:ident) => {
            Ok(StrategyTickResult {
                w: $strategy.tick(
                    $balance,
                    $peer_ids.into_iter().map(|v| {
                        v.and_then(|v| {
                            v.as_string()
                                .ok_or(wasm_bindgen::JsValue::from("not a string"))
                                .and_then(|s| {
                                    utils_misc::ok_or_jserr!(<libp2p_identity::PeerId as std::str::FromStr>::from_str(
                                        &s
                                    ))
                                })
                        })
                        .unwrap()
                    }),
                    serde_wasm_bindgen::from_value::<Vec<crate::generic::wasm::OutgoingChannelStatus>>(
                        $outgoing_channels,
                    )?
                    .iter()
                    .map(|c| crate::generic::OutgoingChannelStatus::from(c))
                    .collect(),
                    |peer_id: &str| {
                        $quality_of
                            .call1(&wasm_bindgen::JsValue::null(), &js_sys::JsString::from(peer_id))
                            .ok()
                            .map(|q| q.as_f64())
                            .flatten()
                    },
                ),
            })
        };
    }
}
