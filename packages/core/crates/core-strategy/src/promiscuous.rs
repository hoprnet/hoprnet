use utils_types::primitives::{Balance, BaseBalance};

use crate::generic::{ChannelStrategy, OutgoingChannelStatus, StrategyTickResult};

/// Implements promiscuous strategy.
/// This strategy opens channels to peers, which have quality above a given threshold.
/// At the same time, it closes channels opened to peers whose quality dropped below this threshold.
pub struct PromiscuousStrategy {
    network_quality_threshold: f64,
    new_channel_stake: Balance,
    minimum_channel_balance: Balance,
    minimum_node_balance: Balance,
}

impl Default for PromiscuousStrategy {
    /// Creates promiscuous strategy with default parameters,
    /// that is quality threshold 0.5, new channel stake 0.1 txHOPR,
    /// minimum channel balance to 0.01 txHOPR (meaning an auto-channel can be used for 10 msgs)
    /// minimum token balance on the node should not drop below 0.1 txHOPR.
    fn default() -> Self {
        PromiscuousStrategy {
            network_quality_threshold: 0.5,
            new_channel_stake: Balance::from_str("100000000000000000").unwrap(),
            minimum_channel_balance: Balance::from_str("10000000000000000").unwrap(),
            minimum_node_balance: Balance::from_str("100000000000000000").unwrap(),
        }
    }
}

impl ChannelStrategy for PromiscuousStrategy {
    const NAME: &'static str = "promiscuous";

    fn tick<Q>(
        &self,
        balance: Balance,
        peer_ids: impl Iterator<Item = String>,
        outgoing_channels: Vec<OutgoingChannelStatus>,
        quality_of_peer: Q,
    ) -> StrategyTickResult
    where
        Q: Fn(&str) -> Option<f64>,
    {
        let mut to_close: Vec<String> = vec![];
        let mut new_channel_candidates: Vec<(String, f64)> = vec![];
        let mut network_size: usize = 0;

        // Go through all the peer ids we know, get their qualities and find out which channels should be closed and
        // which peer ids should become candidates for a new channel
        // Also re-open all the channels that have dropped under minimum given balance
        for peer_id in peer_ids {
            // Skip this peer if we already processed it (iterator may have duplicates)
            if to_close.contains(&peer_id)
                || new_channel_candidates
                    .iter()
                    .find(|(p, _)| p.eq(&peer_id))
                    .is_some()
            {
                continue;
            }

            // Retrieve quality of that peer
            let quality = quality_of_peer(peer_id.as_str())
                .expect(format!("failed to retrieve quality of {}", peer_id).as_str());

            // Also get channels we have opened with it
            let channel_with_peer = outgoing_channels
                .iter()
                .find(|c| c.peer_id.eq(&peer_id.as_str()));

            if let Some(channel) = channel_with_peer {
                if quality <= self.network_quality_threshold {
                    // Need to close the channel, because quality has dropped
                    to_close.push(peer_id);
                } else if channel.stake.lt(&self.minimum_channel_balance) {
                    // Need to re-open channel, because channel stake has dropped
                    to_close.push(peer_id.clone());
                    new_channel_candidates.push((peer_id, quality));
                }
            } else if quality >= self.network_quality_threshold {
                // Try to open channel with this peer, because it is high-quality
                new_channel_candidates.push((peer_id, quality));
            }

            network_size = network_size + 1;
        }

        // We compute the upper bound for channels as a square-root of the perceived network size
        let max_auto_channels = (network_size as f64).sqrt().ceil() as usize;

        // Sort the new channel candidates by best quality first, then truncate to the number of available slots
        // This way, we'll prefer candidates with higher quality, when we don't have enough node balance
        new_channel_candidates
            .sort_unstable_by(|(_, q1), (_, q2)| q1.partial_cmp(q2).unwrap().reverse());
        new_channel_candidates
            .truncate(max_auto_channels - (outgoing_channels.len() - to_close.len()));

        // Go through the new candidates for opening channels allow them to open based on our available node balance
        let mut to_open: Vec<OutgoingChannelStatus> = vec![];
        let mut remaining_balance = balance.clone();
        for peer_id in new_channel_candidates.into_iter().map(|(p, _)| p) {
            // Stop if we ran out of balance
            if remaining_balance.lte(&self.minimum_node_balance) {
                break;
            }

            // If we haven't added this peer id yet, add it to the list for channel opening
            if to_open.iter().find(|&p| p.peer_id.eq(&peer_id)).is_none() {
                to_open.push(OutgoingChannelStatus {
                    peer_id,
                    stake: self.new_channel_stake.clone(),
                });
                remaining_balance = balance.sub(&self.new_channel_stake);
            }
        }

        StrategyTickResult::new(max_auto_channels, to_open, to_close)
    }
}

/// Unit tests of pure Rust code
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_promiscuous_basic() {
        let strat = PromiscuousStrategy::default();

        assert_eq!(strat.name(), "promiscuous");

        let peers = HashMap::from([
            ("Alice".to_string(), 0.1),
            ("Bob".to_string(), 0.7),
            ("Charlie".to_string(), 0.9),
            ("Dahlia".to_string(), 0.1),
            ("Eugene".to_string(), 0.8),
            ("Felicia".to_string(), 0.3),
            ("Gustave".to_string(), 1.0),
            ("Heather".to_string(), 0.1),
            ("Ian".to_string(), 0.2),
            ("Kate".to_string(), 0.3),
        ]);

        let balance = Balance::from_str("1000000000000000000").unwrap();
        let low_balance = Balance::from_str("1000000000000000").unwrap();

        let outgoing_channels = vec![
            OutgoingChannelStatus {
                peer_id: "Alice".to_string(),
                stake: balance.clone(),
            },
            OutgoingChannelStatus {
                peer_id: "Charlie".to_string(),
                stake: balance.clone(),
            },
            OutgoingChannelStatus {
                peer_id: "Gustave".to_string(),
                stake: low_balance,
            },
        ];

        let results = strat.tick(
            balance,
            peers.iter().map(|x| x.0.clone()),
            outgoing_channels,
            |s| peers.get(s).copied(),
        );

        assert_eq!(results.max_auto_channels(), 4);

        assert_eq!(results.to_close().len(), 2);
        assert_eq!(results.to_open().len(), 3);

        assert!(results.to_close().contains(&"Alice".to_string()));
        assert!(results.to_close().contains(&"Gustave".to_string()));

        assert_eq!(results.to_open()[0].peer_id, "Gustave".to_string());
        assert_eq!(results.to_open()[1].peer_id, "Eugene".to_string());
        assert_eq!(results.to_open()[2].peer_id, "Bob".to_string());
    }
}

/// WASM bindings
#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;

    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::wasm::Balance;

    use crate::generic::wasm::StrategyTickResult;
    use crate::generic::ChannelStrategy;

    #[wasm_bindgen]
    pub struct PromiscuousStrategy {
        w: super::PromiscuousStrategy,
    }

    #[wasm_bindgen]
    impl PromiscuousStrategy {
        #[wasm_bindgen(constructor)]
        pub fn new(
            network_quality_threshold: f64,
            minimum_node_balance: Balance,
            new_channel_stake: Balance,
            minimum_channel_balance: Balance,
        ) -> Self {
            PromiscuousStrategy {
                w: super::PromiscuousStrategy {
                    network_quality_threshold,
                    minimum_node_balance: minimum_node_balance.w,
                    new_channel_stake: new_channel_stake.w,
                    minimum_channel_balance: minimum_channel_balance.w,
                },
            }
        }

        pub fn default() -> Self {
            PromiscuousStrategy {
                w: super::PromiscuousStrategy::default(),
            }
        }

        #[wasm_bindgen(getter)]
        pub fn name(&self) -> String {
            self.w.name().into()
        }

        pub fn tick(
            &self,
            balance: Balance,
            peer_ids: &js_sys::Iterator,
            outgoing_channels: JsValue,
            quality_of: &js_sys::Function,
        ) -> JsResult<StrategyTickResult> {
            crate::generic::wasm::tick_wrap(
                &self.w,
                balance,
                peer_ids,
                outgoing_channels,
                quality_of,
            )
        }
    }
}
