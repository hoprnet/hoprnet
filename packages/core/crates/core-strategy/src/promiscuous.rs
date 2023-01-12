use utils_types::channels::ChannelEntry;
use utils_types::primitives::Balance;

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
            minimum_channel_stake: Balance::from_str("100000000000000000", "txHOPR").unwrap(),
            minimum_node_balance: Balance::from_str("100000000000000000", "txHOPR").unwrap()
        }
    }
}

impl ChannelStrategy for PromiscuousStrategy {
    fn name(&self) -> &str {
        "promiscuous"
    }

    fn tick<Q>(&self, balance: Balance, network_size: u32, outgoing_channel_peer_ids: &[&str], quality_of: Q, peer_ids: &[&str]) -> StrategyTickResult
        where Q: Fn(&str) -> Option<f64> {
        // We compute the upper bound for channels as a square-root of the perceived network size
        let max_channels = (network_size as f64).sqrt().ceil() as usize;

        // First get qualities of all peers we see
        let mut all_peers_qualities: Vec<(&str, f64)> = peer_ids
            .iter()
            .map(|&peer_id| (peer_id, quality_of(peer_id).unwrap_or(0f64)))
            .collect();

        // Sort by best qualities first, unstable sort is sufficient because we don't care about
        // the order of those who have the same quality.
        all_peers_qualities.sort_unstable_by(|&(_, q1), &(_, q2)| q1.partial_cmp(&q2).unwrap());

        // Find all outgoing channels which dropped below the quality threshold - we're going to close those.
        let to_close: Vec<String> = all_peers_qualities
            .iter()
            .filter(|(p, q)| *q <= self.network_quality_threshold && outgoing_channel_peer_ids.contains(p))
            .copied()
            .map(|(peer_id,_)| peer_id.to_string())
            .collect();

        // Maximum number of channels we can open
        let max_to_open = max_channels - outgoing_channel_peer_ids.len() + to_close.len();

        // Find potential new candidates for opening channels to them
        let new_channel_candidates = all_peers_qualities
            .into_iter()
            .filter(|(p, q)| *q > self.network_quality_threshold && !outgoing_channel_peer_ids.contains(p))
            .map(|(peer_id,_)| peer_id)
            .take(max_to_open);

        let mut to_open: Vec<ChannelOpenRequest> = vec![];
        let mut remaining_balance = balance.clone();

        for peer_id in new_channel_candidates  {
            // Stop if we ran out of balance
            if remaining_balance.lte(&self.minimum_node_balance) {
                break;
            }

            // If we haven't added this peer id yet, add it to the list for channel opening
            if to_open.iter().find(|&p| p.peer_id.eq(peer_id.into())).is_none() {
                to_open.push(ChannelOpenRequest{
                    peer_id: peer_id.to_string(),
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

        pub fn tick(&self, balance: Balance, network_size: u32, current_channels: Vec<JsString>, quality_of: &js_sys::Function, peer_ids: Vec<JsString>) ->  StrategyTickResult {
            crate::generic::wasm::tick_wrap(&self.w, balance, network_size, current_channels, quality_of, peer_ids)
        }
    }
}

