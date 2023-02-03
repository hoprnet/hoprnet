use rand::rngs::OsRng;
use rand::seq::SliceRandom;
use utils_types::channels::{AcknowledgedTicket, ChannelEntry};

use utils_types::channels::ChannelStatus::{Open, PendingToClose};
use utils_types::primitives::{Balance, BalanceType::HOPR};

use crate::generic::{ChannelStrategy, OutgoingChannelStatus, StrategyTickResult};

/// Implements promiscuous strategy.
/// This strategy opens channels to peers, which have quality above a given threshold.
/// At the same time, it closes channels opened to peers whose quality dropped below this threshold.
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct PromiscuousStrategy {
    pub network_quality_threshold: f64,
    pub new_channel_stake: Balance,
    pub minimum_channel_balance: Balance,
    pub minimum_node_balance: Balance,
    pub max_channels: Option<usize>,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl PromiscuousStrategy {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new() -> Self {
        PromiscuousStrategy {
            network_quality_threshold: 0.5,
            new_channel_stake: Balance::from_str("100000000000000000", HOPR),
            minimum_channel_balance: Balance::from_str("10000000000000000", HOPR),
            minimum_node_balance: Balance::from_str("100000000000000000", HOPR),
            max_channels: None
        }
    }

    // Re-implementations to satisfy the trait, because
    // we cannot put #[wasm_bindgen] on trait impl blocks

    /*
    TODO: Right now there's only WASM-specific implementation until HoprCoreEthereum is migrated
    pub fn on_winning_ticket(&self, ack_ticket: &AcknowledgedTicket) {
        todo!()
    }

    pub fn on_channel_closing(&self, channel: &ChannelEntry) {
        todo!()
    }
    */

    pub fn should_commit_to_channel(&self, _channel: &ChannelEntry) -> bool {
        true
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
        let mut to_open: Vec<OutgoingChannelStatus> = vec![];
        let mut to_close: Vec<String> = vec![];

        let mut new_channel_candidates: Vec<(String, f64)> = vec![];
        let mut network_size: usize = 0;

        // Go through all the peer ids we know, get their qualities and find out which channels
        // should be closed and which peer ids should become candidates for a new channel
        // Also re-open all the channels that have dropped under minimum given balance
        for peer_id in peer_ids {
            if to_close.contains(&peer_id)
                || new_channel_candidates
                    .iter()
                    .find(|(p, _)| p.eq(&peer_id))
                    .is_some()
            {
                // Skip this peer if we already processed it (iterator may have duplicates)
                continue;
            }

            // Retrieve quality of that peer
            let quality = quality_of_peer(peer_id.as_str())
                .expect(format!("failed to retrieve quality of {}", peer_id).as_str());

            // Also get channels we have opened with it
            let channel_with_peer = outgoing_channels
                .iter()
                .filter(|c| c.status == Open)
                .find(|c| c.peer_id.eq(&peer_id.as_str()));

            if let Some(channel) = channel_with_peer {
                if quality <= self.network_quality_threshold {
                    // Need to mark the channel for closing, because quality has dropped
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

        // Also mark for closing all channels which are in PendingToClose state
        outgoing_channels
            .iter()
            .filter(|c| c.status == PendingToClose)
            .for_each(|c| to_close.push(c.peer_id.clone()));

        // We compute the upper bound for channels as a square-root of the perceived network size
        let max_auto_channels = self.max_channels.unwrap_or((network_size as f64).sqrt().ceil() as usize);
        let count_opened = outgoing_channels.iter().filter(|c| c.status == Open).count();
        let occupied = count_opened - to_close.len();

        // If there is still more channels opened than we allow, close some
        // lowest-quality ones which passed the threshold
        if occupied > max_auto_channels  {
            let mut sorted_channels: Vec<OutgoingChannelStatus> = outgoing_channels
                .iter()
                .filter(|c| !to_close.contains(&c.peer_id))
                .cloned()
                .collect();
            // Sort by quality, lowest-quality first
            sorted_channels.
                sort_unstable_by(|p1, p2| {
                    let q1 = quality_of_peer(p1.peer_id.as_str())
                        .expect(format!("failed to retrieve quality of {}", p1.peer_id).as_str());
                    let q2 = quality_of_peer(p2.peer_id.as_str())
                        .expect(format!("failed to retrieve quality of {}", p2.peer_id).as_str());
                    q1.partial_cmp(&q2).unwrap()
                });
            // Close the lowest-quality channels (those we did not mark for closing yet)
            sorted_channels
                .into_iter()
                .take(occupied-max_auto_channels)
                .for_each(|c| to_close.push(c.peer_id));
        }

        if max_auto_channels > occupied {
            // Sort the new channel candidates by best quality first, then truncate to the number of available slots
            // This way, we'll prefer candidates with higher quality, when we don't have enough node balance
            // Shuffle first, so the equal candidates are randomized and then use unstable sorting for that purpose.
            new_channel_candidates.shuffle(&mut OsRng);
            new_channel_candidates
                .sort_unstable_by(|(_, q1), (_, q2)| q1.partial_cmp(q2).unwrap().reverse());
            new_channel_candidates
                .truncate(max_auto_channels - occupied);

            // Go through the new candidates for opening channels allow them to open based on our available node balance
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
                        status: Open
                    });
                    remaining_balance = balance.sub(&self.new_channel_stake);
                }
            }
        }

        StrategyTickResult::new(max_auto_channels, to_open, to_close)
    }

    fn on_winning_ticket(&self, ack_ticket: &AcknowledgedTicket) {
        self.on_winning_ticket(ack_ticket)
    }

    fn on_channel_closing(&self, channel: &ChannelEntry) {
        self.on_channel_closing(channel)
    }

    fn should_commit_to_channel(&self, channel: &ChannelEntry) -> bool {
        self.should_commit_to_channel(channel)
    }
}

/// Unit tests of pure Rust code
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref PEERS: HashMap<String, f64> = {
            HashMap::from([
                ("Alice".to_string(), 0.1),
                ("Bob".to_string(), 0.7),
                ("Charlie".to_string(), 0.9),
                ("Dahlia".to_string(), 0.1),
                ("Eugene".to_string(), 0.8),
                ("Felicia".to_string(), 0.3),
                ("Gustave".to_string(), 1.0),
                ("Heather".to_string(), 0.1),
                ("Ian".to_string(), 0.2),
                ("Joe".to_string(), 0.3),
            ])
        };

        static ref DEFAULT_BALANCE: Balance = Balance::from_str("1000000000000000000", HOPR);
        static ref LOW_BALANCE: Balance = Balance::from_str("1000000000000000", HOPR);
    }

    fn make_channel(channel: (&str, &Balance)) -> OutgoingChannelStatus {
        OutgoingChannelStatus {
            peer_id: channel.0.to_string(),
            stake: channel.1.clone(),
            status: Open
        }
    }

    #[test]
    fn test_promiscuous_basic() {
        let strat = PromiscuousStrategy::new();

        assert_eq!(strat.name(), "promiscuous");

        let outgoing_channels = vec![
            make_channel(("Alice", &DEFAULT_BALANCE)),
            make_channel(("Charlie", &DEFAULT_BALANCE)),
            make_channel(("Gustave", &LOW_BALANCE)),
        ];

        let results = strat.tick(
            DEFAULT_BALANCE.clone(),
            PEERS.iter().map(|x| x.0.clone()),
            outgoing_channels,
            |s| PEERS.get(s).copied(),
        );

        assert_eq!(results.max_auto_channels, 4);

        assert_eq!(results.to_close().len(), 2);
        assert_eq!(results.to_open().len(), 3);

        assert!(results.to_close().contains(&"Alice".to_string()));
        assert!(results.to_close().contains(&"Gustave".to_string()));

        assert_eq!(results.to_open()[0].peer_id, "Gustave".to_string());
        assert_eq!(results.to_open()[1].peer_id, "Eugene".to_string());
        assert_eq!(results.to_open()[2].peer_id, "Bob".to_string());
    }

    #[test]
    fn test_promiscuous_max() {
        let mut strat = PromiscuousStrategy::new();
        strat.max_channels = Some(2);

        let outgoing_channels = vec![
            make_channel(("Charlie", &DEFAULT_BALANCE)),
            make_channel(("Gustave", &DEFAULT_BALANCE)),
            make_channel(("Eugene", &DEFAULT_BALANCE)),
        ];

        let results = strat.tick(
            DEFAULT_BALANCE.clone(),
            PEERS.iter().map(|x| x.0.clone()),
            outgoing_channels,
            |s| PEERS.get(s).copied(),
        );

        assert_eq!(results.max_auto_channels, 2);

        assert_eq!(results.to_close().len(), 1);
        assert_eq!(results.to_open().len(), 0);

        assert!(results.to_close().contains(&"Eugene".to_string()));
    }
}

/// WASM bindings
#[cfg(feature = "wasm")]
pub mod wasm {
    use js_sys::Promise;
    use wasm_bindgen::prelude::*;

    use utils_misc::utils::wasm::JsResult;
    use utils_types::channels::{AcknowledgedTicket, ChannelEntry};
    use utils_types::primitives::Balance;

    use crate::generic::StrategyTickResult;
    use crate::generic::ChannelStrategy;
    use crate::generic::wasm::WasmChannelStrategy;
    use crate::promiscuous::PromiscuousStrategy;

    #[wasm_bindgen(module = "@hoprnet/hopr-core-ethereum")]
    extern "C" {
        type HoprCoreEthereum;

        #[wasm_bindgen(static_method_of = HoprCoreEthereum, getter)]
        fn instance() -> HoprCoreEthereum;

        #[wasm_bindgen(method)]
        fn redeemTicketsInChannelByCounterparty(this: &HoprCoreEthereum, counterparty: &JsValue) -> Promise;

        #[wasm_bindgen(method)]
        fn redeemTicketsInChannel(this: &HoprCoreEthereum, channel: &JsValue) -> Promise;
    }

    impl WasmChannelStrategy for PromiscuousStrategy {}

    #[wasm_bindgen]
    impl PromiscuousStrategy {
        #[wasm_bindgen(getter, js_name="name")]
        pub fn strategy_name(&self) -> String {
            self.name().into()
        }

        #[wasm_bindgen(js_name = "tick")]
        pub fn strategy_tick(
            &self,
            balance: Balance,
            peer_ids: &js_sys::Iterator,
            outgoing_channels: JsValue,
            quality_of: &js_sys::Function,
        ) -> JsResult<StrategyTickResult> {
            self.wrapped_tick(
                balance,
                peer_ids,
                outgoing_channels,
                quality_of,
            )
        }


        pub fn on_winning_ticket(&self, ack_ticket: &AcknowledgedTicket) {
            todo!()
        }

        pub fn on_channel_closing(&self, channel: &ChannelEntry) {
            todo!()
        }
    }
}
