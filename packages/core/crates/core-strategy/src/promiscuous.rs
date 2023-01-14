use utils_types::channels::ChannelEntry;
use utils_types::primitives::{BaseBalance, Balance};

use crate::generic::{ChannelOpenRequest, ChannelStrategy, StrategyTickResult};

pub struct PromiscuousStrategy {
    network_quality_threshold: f64,
    minimum_channel_stake: Balance,
    minimum_node_balance: Balance
}

impl Default for PromiscuousStrategy {
    fn default() -> Self {
        PromiscuousStrategy {
            network_quality_threshold: 0.5,
            minimum_channel_stake: Balance::from_str("100000000000000000").unwrap(),
            minimum_node_balance: Balance::from_str("100000000000000000").unwrap()
        }
    }
}

impl ChannelStrategy for PromiscuousStrategy {
    fn name(&self) -> &str {
        "promiscuous"
    }

    fn tick<Q>(&self, balance: Balance, peer_ids: impl Iterator<Item=String>, outgoing_channel_peer_ids: &[&str], quality_of: Q) -> StrategyTickResult
        where Q: Fn(&str) -> Option<f64> {

        let mut to_close: Vec<String> = vec![];
        let mut new_channel_candidates: Vec<(String, f64)> = vec![];
        let mut network_size: usize = 0;

        for peer_id in peer_ids {
            let quality = quality_of(peer_id.as_str()).unwrap_or(0f64);

            if quality <= self.network_quality_threshold && outgoing_channel_peer_ids.contains(&peer_id.as_str()) {
                to_close.push(peer_id.to_string());
            }

            if quality >= self.network_quality_threshold && !outgoing_channel_peer_ids.contains(&peer_id.as_str()) {
                new_channel_candidates.push((peer_id.to_string(), quality));
            }

            network_size = network_size + 1;
        }

        // We compute the upper bound for channels as a square-root of the perceived network size
        let max_channels = (network_size as f64).sqrt().ceil() as usize;


        // Sort the new channel candidates by best quality first, then truncate to the number of available slots
        new_channel_candidates.sort_unstable_by(|(_, q1), (_, q2)| q1.partial_cmp(q2).unwrap() );
        new_channel_candidates.truncate(max_channels - (outgoing_channel_peer_ids.len() - to_close.len()));

        let mut to_open: Vec<ChannelOpenRequest> = vec![];
        let mut remaining_balance = balance.clone();
        for peer_id in new_channel_candidates.into_iter().map(|(p,_)| p)  {
            // Stop if we ran out of balance
            if remaining_balance.lte(&self.minimum_node_balance) {
                break;
            }

            // If we haven't added this peer id yet, add it to the list for channel opening
            if to_open.iter().find(|&p| p.peer_id.eq(&peer_id)).is_none() {
                to_open.push(ChannelOpenRequest{
                    peer_id,
                    stake: self.minimum_channel_stake.clone()
                });
                remaining_balance = balance.sub(&self.minimum_channel_stake);
            }
        }

        StrategyTickResult {
            to_open,
            to_close
        }
    }
}


/// Unit tests of pure Rust code
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promisc_basic() {
        let strat = PromiscuousStrategy::default();


        //let results = strat.tick(Balance::from_u64(10), )

    }
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

        #[wasm_bindgen(constructor)]
        pub fn new(network_quality_threshold: f64, minimum_node_balance: Balance, minimum_channel_stake: Balance) -> Self {
            PromiscuousStrategy {
                w: super::PromiscuousStrategy {
                    network_quality_threshold,
                    minimum_node_balance: minimum_node_balance.w,
                    minimum_channel_stake: minimum_channel_stake.w
                }
            }
        }

        pub fn default() -> Self {
            PromiscuousStrategy {
                w: super::PromiscuousStrategy::default()
            }
        }

        #[wasm_bindgen(getter)]
        pub fn name(&self) -> String {
            self.w.name().into()
        }

        pub fn tick(&self, balance: Balance, peer_ids: &js_sys::Iterator, outgoing_channel_peer_ids: Vec<JsString>, quality_of: &js_sys::Function) ->  StrategyTickResult {
            crate::generic::wasm::tick_wrap(&self.w, balance, peer_ids, outgoing_channel_peer_ids, quality_of)
        }
    }
}

