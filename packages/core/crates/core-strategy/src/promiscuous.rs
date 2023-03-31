use libp2p_identity::PeerId;
use rand::rngs::OsRng;
use rand::seq::SliceRandom;
use simple_moving_average::{SumTreeSMA, SMA};
use utils_log::{debug, info, warn};

use core_types::channels::ChannelStatus::{Open, PendingToClose};
use utils_types::primitives::{Balance, BalanceType};

use crate::generic::{ChannelStrategy, OutgoingChannelStatus, StrategyTickResult};

/// Size of the simple moving average window used to smoothen the number of registered peers.
pub const SMA_WINDOW_SIZE: usize = 3;

type SimpleMovingAvg = SumTreeSMA<usize, usize, SMA_WINDOW_SIZE>;

/// Implements promiscuous strategy.
/// This strategy opens outgoing channels to peers, which have quality above a given threshold.
/// At the same time, it closes outgoing channels opened to peers whose quality dropped below this threshold.
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct PromiscuousStrategy {
    /// A quality threshold between 0 and 1 used to determine whether the strategy should open channel with the peer.
    /// Defaults to 0.5
    pub network_quality_threshold: f64,

    /// A stake of tokens that should be allocated to a channel opened by the strategy.
    /// Defaults to 0.1 HOPR
    pub new_channel_stake: Balance,

    /// A minimum channel token stake. If reached, the channel will be closed and re-opened with `new_channel_stake`.
    /// Defaults to 0.01 HOPR
    pub minimum_channel_balance: Balance,

    /// Minimum token balance of the node. When reached, the strategy will not open any new channels.
    /// Defaults to 0.01 HOPR
    pub minimum_node_balance: Balance,

    /// Maximum number of opened channels the strategy should maintain.
    /// Defaults to square-root of the sampled network size.
    pub max_channels: Option<usize>,

    /// Determines if the strategy should automatically redeem tickets.
    /// Defaults to false
    pub auto_redeem_tickets: bool,

    /// If set, the strategy will aggressively close channels (even with peers above the `network_quality_threshold`)
    /// if the number of opened outgoing channels (regardless if opened by the strategy or manually) exceeds the
    /// `max_channels` limit.
    /// Defaults to true
    pub enforce_max_channels: bool,

    sma: SimpleMovingAvg,
}

impl Default for PromiscuousStrategy {
    fn default() -> Self {
        PromiscuousStrategy {
            network_quality_threshold: 0.5,
            new_channel_stake: Balance::from_str("100000000000000000", BalanceType::HOPR),
            minimum_channel_balance: Balance::from_str("10000000000000000", BalanceType::HOPR),
            minimum_node_balance: Balance::from_str("100000000000000000", BalanceType::HOPR),
            max_channels: None,
            auto_redeem_tickets: false,
            enforce_max_channels: true,
            sma: SimpleMovingAvg::new(),
        }
    }
}

impl ChannelStrategy for PromiscuousStrategy {
    const NAME: &'static str = "promiscuous";

    fn tick<Q>(
        &mut self,
        balance: Balance,
        peer_ids: impl Iterator<Item = PeerId>,
        outgoing_channels: Vec<OutgoingChannelStatus>,
        quality_of_peer: Q,
    ) -> StrategyTickResult
    where
        Q: Fn(&str) -> Option<f64>,
    {
        let mut to_open: Vec<OutgoingChannelStatus> = vec![];
        let mut to_close: Vec<PeerId> = vec![];
        let mut new_channel_candidates: Vec<(PeerId, f64)> = vec![];
        let mut network_size: usize = 0;

        // Go through all the peer ids we know, get their qualities and find out which channels should be closed and
        // which peer ids should become candidates for a new channel
        // Also re-open all the channels that have dropped under minimum given balance
        for peer_id in peer_ids {
            if to_close.contains(&peer_id) || new_channel_candidates.iter().find(|(p, _)| p.eq(&peer_id)).is_some() {
                // Skip this peer if we already processed it (iterator may have duplicates)
                debug!("encountered duplicate peer {}", peer_id);
                continue;
            }

            // Retrieve quality of that peer
            let quality = quality_of_peer(&peer_id.to_base58())
                .expect(format!("failed to retrieve quality of {}", peer_id).as_str());

            // Also get channels we have opened with it
            let channel_with_peer = outgoing_channels
                .iter()
                .filter(|c| c.status == Open)
                .find(|c| c.peer_id == peer_id);

            if let Some(channel) = channel_with_peer {
                if quality <= self.network_quality_threshold {
                    // Need to close the channel, because quality has dropped
                    debug!("new channel closure candidate with {} (quality {})", peer_id, quality);
                    to_close.push(peer_id);
                } else if channel.stake.lt(&self.minimum_channel_balance) {
                    // Need to re-open channel, because channel stake has dropped
                    debug!("new channel closure & re-stake candidate with {}", peer_id);
                    to_close.push(peer_id.clone());
                    new_channel_candidates.push((peer_id, quality));
                }
            } else if quality >= self.network_quality_threshold {
                // Try to open channel with this peer, because it is high-quality
                debug!("new channel opening candidate {} with quality {}", peer_id, quality);
                new_channel_candidates.push((peer_id, quality));
            }

            network_size += 1;
        }
        self.sma.add_sample(network_size);
        info!("evaluated qualities of {} peers seen in the network", network_size);

        if self.sma.get_num_samples() < self.sma.get_sample_window_size() {
            info!(
                "not yet enough samples ({} out of {}) of network size to perform a strategy tick, skipping.",
                self.sma.get_num_samples(),
                self.sma.get_sample_window_size()
            );
            return StrategyTickResult::new(0, vec![], vec![]);
        }

        // Also mark for closing all channels which are in PendingToClose state
        let before_pending = outgoing_channels.len();
        outgoing_channels
            .iter()
            .filter(|c| c.status == PendingToClose)
            .for_each(|c| to_close.push(c.peer_id.clone()));
        debug!(
            "{} channels are in PendingToClose, so strategy will mark them for closing too",
            outgoing_channels.len() - before_pending
        );

        // We compute the upper bound for channels as a square-root of the perceived network size
        let max_auto_channels = self
            .max_channels
            .unwrap_or((self.sma.get_average() as f64).sqrt().ceil() as usize);
        debug!(
            "current upper bound for maximum number of auto-channels if {}",
            max_auto_channels
        );

        // Count all the opened channels
        let count_opened = outgoing_channels.iter().filter(|c| c.status == Open).count();
        let occupied = count_opened - to_close.len();

        // If there is still more channels opened than we allow, close some
        // lowest-quality ones which passed the threshold
        if occupied > max_auto_channels && self.enforce_max_channels {
            warn!("there are {} opened channels, but the strategy allows only {}", occupied, max_auto_channels);

            let mut sorted_channels: Vec<OutgoingChannelStatus> = outgoing_channels
                .iter()
                .filter(|c| !to_close.contains(&c.peer_id))
                .cloned()
                .collect();

            // Sort by quality, lowest-quality first
            sorted_channels.sort_unstable_by(|p1, p2| {
                    quality_of_peer(&p1.peer_id.to_base58()).zip(quality_of_peer(&p2.peer_id.to_base58()))
                        .and_then(|(q1, q2)| q1.partial_cmp(&q2))
                        .expect(format!("failed to retrieve quality of {} or {}", p1.peer_id, p2.peer_id).as_str())
            });

            // Close the lowest-quality channels (those we did not mark for closing yet)
            sorted_channels
                .into_iter()
                .take(occupied - max_auto_channels)
                .for_each(|c| to_close.push(c.peer_id));
        }

        if max_auto_channels > occupied {
            // Sort the new channel candidates by best quality first, then truncate to the number of available slots
            // This way, we'll prefer candidates with higher quality, when we don't have enough node balance
            // Shuffle first, so the equal candidates are randomized and then use unstable sorting for that purpose.
            new_channel_candidates.shuffle(&mut OsRng);
            new_channel_candidates.sort_unstable_by(|(_, q1), (_, q2)| q1.partial_cmp(q2).unwrap().reverse());
            new_channel_candidates.truncate(max_auto_channels - occupied);
            debug!("got {} new channel candidates", new_channel_candidates.len());

            // Go through the new candidates for opening channels allow them to open based on our available node balance
            let mut remaining_balance = balance.clone();
            for peer_id in new_channel_candidates.into_iter().map(|(p, _)| p) {
                // Stop if we ran out of balance
                if remaining_balance.lte(&self.minimum_node_balance) {
                    warn!("strategy ran out of allowed node balance - balance is {}", remaining_balance.to_string());
                    break;
                }

                // If we haven't added this peer id yet, add it to the list for channel opening
                if to_open.iter().find(|&p| p.peer_id.eq(&peer_id)).is_none() {
                    debug!("promoting peer {} for channel opening", peer_id);
                    to_open.push(OutgoingChannelStatus {
                        peer_id,
                        stake: self.new_channel_stake.clone(),
                        status: Open,
                    });
                    remaining_balance = balance.sub(&self.new_channel_stake);
                }
            }
        }

        info!(
            "strategy tick #{} result: {} peers for channel opening, {} peer for channel closure",
            self.sma.get_num_samples(),
            to_open.len(),
            to_close.len()
        );
        StrategyTickResult::new(max_auto_channels, to_open, to_close)
    }
}

/// Unit tests of pure Rust code
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::str::FromStr;
    use vector_assertions::assert_vec_eq;

    #[test]
    fn test_promiscuous_strategy_basic() {
        let mut strat = PromiscuousStrategy::default();

        assert_eq!(strat.name(), "promiscuous");

        let alice = PeerId::random();
        let bob = PeerId::random();
        let charlie = PeerId::random();
        let eugene = PeerId::random();
        let gustave = PeerId::random();

        let peers = HashMap::from([
            (alice.clone(), 0.1),
            (bob.clone(), 0.7),
            (charlie.clone(), 0.9),
            (PeerId::random(), 0.1),
            (eugene.clone(), 0.8),
            (PeerId::random(), 0.3),
            (gustave.clone(), 1.0),
            (PeerId::random(), 0.1),
            (PeerId::random(), 0.2),
            (PeerId::random(), 0.3),
        ]);

        let balance = Balance::from_str("1000000000000000000", BalanceType::HOPR);
        let low_balance = Balance::from_str("1000000000000000", BalanceType::HOPR);

        let outgoing_channels = vec![
            OutgoingChannelStatus {
                peer_id: alice.clone(),
                stake: balance.clone(),
                status: Open,
            },
            OutgoingChannelStatus {
                peer_id: charlie.clone(),
                stake: balance.clone(),
                status: Open,
            },
            OutgoingChannelStatus {
                peer_id: gustave.clone(),
                stake: low_balance,
                status: Open,
            },
        ];

        // Add fake samples to allow the test to run
        strat.sma.add_sample(peers.len());
        strat.sma.add_sample(peers.len());

        let results = strat.tick(balance, peers.iter().map(|(x,_)| x.clone()), outgoing_channels, |s| {
            peers.get(&PeerId::from_str(s).unwrap()).copied()
        });

        assert_eq!(results.max_auto_channels(), 4);

        assert_eq!(results.to_close().len(), 2);
        assert_eq!(results.to_open().len(), 3);

        assert_vec_eq!(results.to_close(), vec![ alice, gustave ]);
        assert_vec_eq!(results.to_open().into_iter().map(|r| r.peer_id).collect::<Vec<PeerId>>(), vec![gustave, eugene, bob]);
    }

    #[test]
    fn test_promiscuous_strategy_more_channels_than_allowed() {
        let mut strat = PromiscuousStrategy::default();
        let mut peers = HashMap::new();
        let mut outgoing_channels = Vec::new();
        for i in 0..100 {
            let peerid = PeerId::random();
            peers.insert(peerid.clone(), 0.9 - i as f64 * 0.02);
            if outgoing_channels.len() < 20 {
                outgoing_channels.push(OutgoingChannelStatus {
                    peer_id: peerid,
                    stake: Balance::from_str("100000000000000000", BalanceType::HOPR),
                    status: Open
                });
            }
        }

        // Add fake samples to allow the test to run
        strat.sma.add_sample(peers.len());
        strat.sma.add_sample(peers.len());

        let results = strat.tick(Balance::from_str("1000000000000000000", BalanceType::HOPR),
                                 peers.iter().map(|(&x, _)| x.clone()),
                                 outgoing_channels.clone(), |s| {
                                    peers.get(&PeerId::from_str(s).unwrap()).copied()
            });

        assert_eq!(results.max_auto_channels(), 10);
        assert_eq!(results.to_open().len(), 0);
        assert_eq!(results.to_close().len(), 10);

        // Only the last 10 lowest quality channels get closed
        assert_vec_eq!(results.to_close(), outgoing_channels.into_iter()
            .rev()
            .map(|s| s.peer_id)
            .take(10)
            .collect::<Vec<PeerId>>());
    }
}

/// WASM bindings
#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;

    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::Balance;

    use crate::generic::wasm::StrategyTickResult;
    use crate::generic::ChannelStrategy;
    use crate::promiscuous::PromiscuousStrategy;
    use crate::strategy_tick;

    #[wasm_bindgen]
    impl PromiscuousStrategy {
        #[wasm_bindgen(constructor)]
        pub fn _new() -> Self {
            Self::default()
        }

        #[wasm_bindgen(getter, js_name = "name")]
        pub fn _name(&self) -> String {
            self.name().into()
        }

        #[wasm_bindgen(js_name = "tick")]
        pub fn _tick(
            &mut self,
            balance: Balance,
            peer_ids: &js_sys::Iterator,
            outgoing_channels: JsValue,
            quality_of: &js_sys::Function,
        ) -> JsResult<StrategyTickResult> {
            strategy_tick!(self, balance, peer_ids, outgoing_channels, quality_of)
        }
    }
}
