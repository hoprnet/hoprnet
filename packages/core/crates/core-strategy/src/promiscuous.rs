use utils_types::channels::ChannelEntry;
use utils_types::primitives::Balance;

use crate::generic::{ChannelOpenRequest, ChannelStrategy, StrategyTickResult};

pub struct PromiscuousStrategy ;

pub const NETWORK_QUALITY_THRESHOLD: f64 = 0.1f64;

impl ChannelStrategy for PromiscuousStrategy {
    fn name(&self) -> &str {
        "promiscuous"
    }

    fn tick<Q>(&self, balance: Balance, network_size: u32, outgoing_channel_peer_ids: &[&str], quality_of: Q, peer_ids: &[&str]) -> StrategyTickResult
        where Q: Fn(&str) -> Option<f64> {

        let min_stake = Balance::from_str("10000000000000000", "txHOPR").unwrap();

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

        // Retrieve qualities of all outgoing channels
        let mut outgoing_channel_qualities: Vec<(&str, f64)> = outgoing_channel_peer_ids
            .iter()
            .map(|&peer_id| {
                // Let's see if we already checked this peer's quality
                if let Some((_, cached_quality)) =  all_peers_qualities.iter().find(|(p,_)| (*p).eq(peer_id)) {
                    (peer_id, *cached_quality)
                }
                else {
                    // Generally this should not happen,
                    // the peers_ids list should be always a superset of the channel recipients.
                    (peer_id, quality_of(peer_id).unwrap_or(0f64))
                }
            })
            .collect();

        // Find all outgoing channels which dropped below the quality threshold
        // those we will be closing.
        let to_close: Vec<String> = outgoing_channel_qualities
            .iter()
            .filter(|(_, q)| *q < NETWORK_QUALITY_THRESHOLD)
            .copied()
            .map(|(peer_id,_)| peer_id.to_string())
            .collect();

        // Maximum number of channels we can open
        let mut max_to_open = max_channels - outgoing_channel_peer_ids.len() + to_close.len();
        let mut remaining_balance = &balance;

        for (peer_id, quality) in all_peers_qualities.into_iter().filter(|(_, q)| *q > NETWORK_QUALITY_THRESHOLD).take(max_to_open) {

            remaining_balance = &balance.sub(&min_stake);
        }

        StrategyTickResult {
            to_open: vec![],
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

        #[wasm_bindgen(getter)]
        pub fn name(&self) -> String {
            self.w.name().into()
        }

        pub fn tick(&self, balance: Balance, network_size: u32, current_channels: Vec<JsString>, quality_of: &js_sys::Function, peer_ids: Vec<JsString>) ->  StrategyTickResult {
            crate::generic::wasm::tick_wrap(&self.w, balance, network_size, current_channels, quality_of, peer_ids)
        }
    }
}

