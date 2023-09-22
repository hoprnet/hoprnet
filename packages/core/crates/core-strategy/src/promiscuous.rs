use core_crypto::types::OffchainPublicKey;
use core_types::channels::ChannelEntry;
use core_types::channels::ChannelStatus::{Open, PendingToClose};
use rand::rngs::OsRng;
use rand::seq::SliceRandom;
use simple_moving_average::{SumTreeSMA, SMA};
use std::collections::HashMap;
use utils_log::{debug, error, info, warn};
use utils_types::primitives::{Address, Balance, BalanceType};

use async_std::sync::RwLock;
use async_trait::async_trait;
use core_ethereum_actions::transaction_queue::{Transaction, TransactionSender};
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_network::network::{Network, NetworkExternalActions};
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::config::StrategyConfig;
use crate::errors::Result;
use crate::strategy::SingularStrategy;
use utils_types::traits::PeerIdLike;

/// Size of the simple moving average window used to smoothen the number of registered peers.
pub const SMA_WINDOW_SIZE: usize = 3;

type SimpleMovingAvg = SumTreeSMA<usize, usize, SMA_WINDOW_SIZE>;

/// Config of promiscuous strategy.
pub struct PromiscuousStrategyConfig {
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

    /// If set, the strategy will aggressively close channels (even with peers above the `network_quality_threshold`)
    /// if the number of opened outgoing channels (regardless if opened by the strategy or manually) exceeds the
    /// `max_channels` limit.
    /// Defaults to true
    pub enforce_max_channels: bool,
}

impl Default for PromiscuousStrategyConfig {
    fn default() -> Self {
        PromiscuousStrategyConfig {
            network_quality_threshold: 0.5,
            new_channel_stake: Balance::from_str("100000000000000000", BalanceType::HOPR),
            minimum_channel_balance: Balance::from_str("10000000000000000", BalanceType::HOPR),
            minimum_node_balance: Balance::from_str("100000000000000000", BalanceType::HOPR),
            max_channels: None,
            enforce_max_channels: true,
        }
    }
}

/// This strategy opens outgoing channels to peers, which have quality above a given threshold.
/// At the same time, it closes outgoing channels opened to peers whose quality dropped below this threshold.
pub struct PromiscuousStrategy<Db, Net>
where
    Db: HoprCoreEthereumDbActions,
    Net: NetworkExternalActions,
{
    db: Arc<RwLock<Db>>,
    network: Arc<RwLock<Network<Net>>>,
    tx_sender: TransactionSender,
    config: PromiscuousStrategyConfig,
    sma: Arc<RwLock<SimpleMovingAvg>>,
}

impl<Db, Net> PromiscuousStrategy<Db, Net>
where
    Db: HoprCoreEthereumDbActions,
    Net: NetworkExternalActions,
{
    pub fn new(
        cfg: StrategyConfig,
        db: Arc<RwLock<Db>>,
        network: Arc<RwLock<Network<Net>>>,
        tx_sender: TransactionSender,
    ) -> Self {
        Self {
            db,
            network,
            tx_sender,
            config: PromiscuousStrategyConfig {
                network_quality_threshold: cfg
                    .network_quality_threshold
                    .unwrap_or(PromiscuousStrategyConfig::default().network_quality_threshold),
                new_channel_stake: cfg
                    .new_channel_stake
                    .unwrap_or(PromiscuousStrategyConfig::default().new_channel_stake),
                minimum_channel_balance: cfg
                    .minimum_channel_balance
                    .unwrap_or(PromiscuousStrategyConfig::default().minimum_channel_balance),
                minimum_node_balance: cfg
                    .minimum_node_balance
                    .unwrap_or(PromiscuousStrategyConfig::default().minimum_node_balance),
                max_channels: cfg.max_auto_channels.map(|m| m as usize),
                enforce_max_channels: cfg
                    .enforce_max_channels
                    .unwrap_or(PromiscuousStrategyConfig::default().enforce_max_channels),
            },
            sma: Arc::new(RwLock::new(SimpleMovingAvg::new())),
        }
    }
}

impl<Db, Net> Display for PromiscuousStrategy<Db, Net>
where
    Db: HoprCoreEthereumDbActions,
    Net: NetworkExternalActions,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "promiscuous")
    }
}
#[async_trait(? Send)]
impl<Db, Net> SingularStrategy for PromiscuousStrategy<Db, Net>
where
    Db: HoprCoreEthereumDbActions,
    Net: NetworkExternalActions,
{
    async fn on_tick(&self) -> Result<()> {
        let balance: Balance = self.db.read().await.get_hopr_balance().await?;
        // addresses: impl Iterator<Item = (Address, f64)>,
        // outgoing_channels: Vec<OutgoingChannelStatus>,

        let mut to_close: Vec<ChannelEntry> = vec![];
        let mut to_open: Vec<(Address, Balance)> = vec![];
        let mut new_channel_candidates: Vec<(Address, f64)> = vec![];
        let mut network_size: usize = 0;
        let mut active_addresses: HashMap<Address, f64> = HashMap::new();
        let outgoing_channels = self.db.read().await.get_outgoing_channels().await?;

        // Go through all the peer ids we know, get their qualities and find out which channels should be closed and
        // which peer ids should become candidates for a new channel
        // Also re-open all the channels that have dropped under minimum given balance
        let peers_with_quality = self.network.read().await.all_peers_with_quality();
        for (peer, quality) in peers_with_quality {
            let packet_key = OffchainPublicKey::from_peerid(&peer)?;
            // get the Ethereum address of the peer
            if let Some(address) = self.db.read().await.get_chain_key(&packet_key).await? {
                if to_close.iter().any(|c| c.destination == address)
                    || new_channel_candidates.iter().find(|(p, _)| p.eq(&address)).is_some()
                {
                    // Skip this peer if we already processed it (iterator may have duplicates)
                    debug!("encountered duplicate peer {}", peer);
                    continue;
                }

                // Also get channels we have opened with it
                let channel_with_peer = outgoing_channels
                    .iter()
                    .find(|c| c.status == Open && c.destination.eq(&address));

                if let Some(channel) = channel_with_peer {
                    if quality <= self.config.network_quality_threshold {
                        // Need to close the channel, because quality has dropped
                        debug!("new channel closure candidate with {} (quality {})", address, quality);
                        to_close.push(channel.clone());
                    } else if channel.balance.lt(&self.config.minimum_channel_balance) {
                        // Need to re-open channel, because channel stake has dropped
                        debug!("new channel closure & re-stake candidate with {}", address);
                        to_close.push(channel.clone());
                        new_channel_candidates.push((address.clone(), quality));
                    }
                } else if quality >= self.config.network_quality_threshold {
                    // Try to open channel with this peer, because it is high-quality
                    debug!("new channel opening candidate {} with quality {}", address, quality);
                    new_channel_candidates.push((address.clone(), quality));
                }

                active_addresses.insert(address, quality);
                network_size += 1;
            } else {
                error!("failed to get chain key from a packet key");
                continue;
            }
        }

        self.sma.write().await.add_sample(network_size);
        info!("evaluated qualities of {} peers seen in the network", network_size);

        if self.sma.read().await.get_num_samples() < self.sma.read().await.get_sample_window_size() {
            info!(
                "not yet enough samples ({} out of {}) of network size to perform a strategy tick, skipping.",
                self.sma.read().await.get_num_samples(),
                self.sma.read().await.get_sample_window_size()
            );
            return Ok(());
        }

        // Also mark for closing all channels which are in PendingToClose state
        let before_pending = outgoing_channels.len();
        outgoing_channels
            .iter()
            .filter(|c| c.status == PendingToClose)
            .for_each(|c| {
                to_close.push(c.clone());
            });
        debug!(
            "{} channels are in PendingToClose, so strategy will mark them for closing too",
            outgoing_channels.len() - before_pending
        );

        // We compute the upper bound for channels as a square-root of the perceived network size
        let max_auto_channels = self
            .config
            .max_channels
            .unwrap_or((self.sma.read().await.get_average() as f64).sqrt().ceil() as usize);
        debug!(
            "current upper bound for maximum number of auto-channels is {}",
            max_auto_channels
        );

        // Count all the opened channels
        let count_opened = outgoing_channels.iter().filter(|c| c.status == Open).count();
        let occupied = if count_opened > to_close.len() {
            count_opened - to_close.len()
        } else {
            0
        };

        // If there is still more channels opened than we allow, close some
        // lowest-quality ones which passed the threshold
        if occupied > max_auto_channels && self.config.enforce_max_channels {
            warn!(
                "there are {} opened channels, but the strategy allows only {}",
                occupied, max_auto_channels
            );

            let mut sorted_channels: Vec<ChannelEntry> = outgoing_channels
                .iter()
                .filter(|c| !to_close.contains(&c))
                .cloned()
                .collect();

            // Sort by quality, lowest-quality first
            sorted_channels.sort_unstable_by(|p1, p2| {
                active_addresses
                    .get(&p1.destination)
                    .zip(active_addresses.get(&p2.destination))
                    .and_then(|(q1, q2)| q1.partial_cmp(&q2))
                    .expect(format!("failed to retrieve quality of {} or {}", p1.destination, p2.destination).as_str())
            });

            // Close the lowest-quality channels (those we did not mark for closing yet)
            sorted_channels
                .into_iter()
                .take(occupied - max_auto_channels)
                .for_each(|c| {
                    to_close.push(c);
                });
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
            for address in new_channel_candidates.into_iter().map(|(p, _)| p) {
                // Stop if we ran out of balance
                if remaining_balance.lte(&self.config.minimum_node_balance) {
                    warn!(
                        "strategy ran out of allowed node balance - balance is {}",
                        remaining_balance.to_string()
                    );
                    break;
                }

                // If we haven't added this peer yet, add it to the list for channel opening
                if to_open
                    .iter()
                    .find(|(open_to_address, _)| open_to_address.eq(&address))
                    .is_none()
                {
                    debug!("promoting peer {} for channel opening", address);
                    to_open.push((address.clone(), self.config.new_channel_stake.clone()));
                    remaining_balance = balance.sub(&self.config.new_channel_stake);
                }
            }
        }

        info!(
            "strategy tick #{} result: {} peers for channel opening, {} peer for channel closure",
            self.sma.read().await.get_num_samples(),
            to_open.len(),
            to_close.len()
        );
        // close all the channels
        futures::future::join_all(to_close.iter().map(|channel_to_close| async {
            self.tx_sender
                .send(Transaction::CloseChannel(channel_to_close.clone()))
                .await
        }))
        .await;
        // open all the channels
        futures::future::join_all(to_open.iter().map(|channel_to_open| async {
            self.tx_sender
                .send(Transaction::OpenChannel(channel_to_open.0, channel_to_open.1))
                .await
        }))
        .await;
        Ok(())
    }
}

/// Unit tests of pure Rust code
#[cfg(test)]
mod tests {
    use super::*;
    use core_crypto::types::Hash;
    use core_ethereum_actions::transaction_queue::{TransactionExecutor, TransactionQueue, TransactionResult};
    use core_ethereum_db::db::CoreEthereumDb;
    use core_network::{
        network::{NetworkConfig, NetworkEvent, NetworkExternalActions, PeerOrigin},
        PeerId,
    };
    use core_types::acknowledgement::AcknowledgedTicket;
    use core_types::channels::ChannelStatus;
    use utils_db::{db::DB, rusty::RustyLevelDbShim};
    // use utils_misc::time::native::current_timestamp;
    struct MockTransactionExecutor;

    impl MockTransactionExecutor {
        pub fn new() -> Self {
            Self {}
        }
    }

    #[async_trait(? Send)]
    impl TransactionExecutor for MockTransactionExecutor {
        async fn redeem_ticket(&self, _ticket: AcknowledgedTicket) -> TransactionResult {
            TransactionResult::RedeemTicket {
                tx_hash: Hash::default(),
            }
        }
        async fn open_channel(&self, _destination: Address, _balance: Balance) -> TransactionResult {
            TransactionResult::OpenChannel {
                tx_hash: Hash::default(),
                channel_id: Hash::default(),
            }
        }
        async fn fund_channel(&self, _channel_id: Hash, _amount: Balance) -> TransactionResult {
            TransactionResult::FundChannel {
                tx_hash: Hash::default(),
            }
        }
        async fn close_channel_initialize(&self, _src: Address, _dst: Address) -> TransactionResult {
            TransactionResult::CloseChannel {
                tx_hash: Hash::default(),
                status: ChannelStatus::Open,
            }
        }
        async fn close_channel_finalize(&self, _src: Address, _dst: Address) -> TransactionResult {
            TransactionResult::CloseChannel {
                tx_hash: Hash::default(),
                status: ChannelStatus::PendingToClose,
            }
        }
        async fn withdraw(&self, _recipient: Address, _amount: Balance) -> TransactionResult {
            TransactionResult::Withdraw {
                tx_hash: Hash::default(),
            }
        }
    }
    struct MockNetworkExternalActions;
    impl NetworkExternalActions for MockNetworkExternalActions {
        fn is_public(&self, _: &PeerId) -> bool {
            false
        }

        fn emit(&self, _: NetworkEvent) {}
    }

    fn generate_random_peer() -> (Address, PeerId) {
        (Address::random(), PeerId::random())
    }

    #[async_std::test]
    async fn test_promiscuous_strategy_config() {
        let (alice_address, alice_peer_id) = generate_random_peer();

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            alice_address,
        )));

        let network = Arc::new(RwLock::new(Network::new(
            alice_peer_id,
            NetworkConfig::default(),
            MockNetworkExternalActions {},
        )));
        let tx_sender = TransactionQueue::new(db.clone(), Box::new(MockTransactionExecutor::new())).new_sender();

        let strat_cfg = StrategyConfig::default();

        let strat = PromiscuousStrategy::new(strat_cfg, db, network, tx_sender);
        assert_eq!(strat.to_string(), "promiscuous");
    }

    #[async_std::test]
    async fn test_promiscuous_strategy_basic() {
        let (alice_address, alice_peer_id) = generate_random_peer();
        let (_, bob_peer_id) = generate_random_peer();
        // let (charlie_address, charlie_peer_id) = generate_random_peer();

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            alice_address,
        )));

        let network = Arc::new(RwLock::new(Network::new(
            alice_peer_id,
            NetworkConfig::default(),
            MockNetworkExternalActions {},
        )));
        
        let tx_sender = TransactionQueue::new(db.clone(), Box::new(MockTransactionExecutor::new())).new_sender();
        
        let strat_cfg = StrategyConfig::default();
        
        let strat = PromiscuousStrategy::new(strat_cfg, db, network, tx_sender);
        
        // add peers to network
        //     let peers = HashMap::from([
            //         (alice.clone(), 0.1),
            //         (bob.clone(), 0.7),
            //         (charlie.clone(), 0.9),
            //         (Address::random(), 0.1),
            //         (eugene.clone(), 0.8),
            //         (Address::random(), 0.3),
            //         (gustave.clone(), 1.0),
            //         (Address::random(), 0.1),
            //         (Address::random(), 0.2),
            //         (Address::random(), 0.3),
            //     ]);
        strat.network.write().await.add(&bob_peer_id, PeerOrigin::Initialization);
        assert_eq!(strat.network.read().await.get_peer_status(&bob_peer_id).unwrap().quality, 0f64);
        // strat.network.write().await.update(&bob_peer_id, Ok(current_timestamp()));
        // assert_eq!(strat.network.read().await.get_peer_status(&bob_peer_id).unwrap().quality, 0.2f64);
    }

    // #[async_std::test]
    // fn test_promiscuous_strategy_basic() {
    //     let strat_cft = PromiscuousStrategyConfig::default();

    //     assert_eq!(strat_cft.name(), "promiscuous");
    //     // let mut strat = PromiscuousStrategy::default();

    //     let alice = Address::from_str("0x5cb1d93aea1fc219a936a708576bf553042993ea").unwrap();
    //     let bob = Address::from_str("0xcc57a920e27f6eaa78c3ccb7976c0fb1e4b94ea1").unwrap();
    //     let charlie = Address::from_str("0xf6a2be4680ab2051f2906922c210da0508553ce1").unwrap();
    //     let eugene = Address::from_str("0x242c983f08f896e1b3a51ec6d646c20bf1b494fc").unwrap();
    //     let gustave = Address::from_str("0xf6c287af75350dc241255f369368044a02c65160").unwrap();

    //     let peers = HashMap::from([
    //         (alice.clone(), 0.1),
    //         (bob.clone(), 0.7),
    //         (charlie.clone(), 0.9),
    //         (Address::random(), 0.1),
    //         (eugene.clone(), 0.8),
    //         (Address::random(), 0.3),
    //         (gustave.clone(), 1.0),
    //         (Address::random(), 0.1),
    //         (Address::random(), 0.2),
    //         (Address::random(), 0.3),
    //     ]);

    //     let balance = Balance::from_str("1000000000000000000", BalanceType::HOPR);
    //     let low_balance = Balance::from_str("1000000000000000", BalanceType::HOPR);

    //     let outgoing_channels = vec![
    //         OutgoingChannelStatus {
    //             address: alice.clone(),
    //             stake: balance.clone(),
    //             status: Open,
    //         },
    //         OutgoingChannelStatus {
    //             address: charlie.clone(),
    //             stake: balance.clone(),
    //             status: Open,
    //         },
    //         OutgoingChannelStatus {
    //             address: gustave.clone(),
    //             stake: low_balance,
    //             status: Open,
    //         },
    //     ];

    //     // Add fake samples to allow the test to run
    //     strat.sma.add_sample(peers.len());
    //     strat.sma.add_sample(peers.len());

    //     let results = strat.tick(
    //         balance,
    //         peers.iter().map(|(x, q)| (x.clone(), q.clone())),
    //         outgoing_channels,
    //     );

    //     assert_eq!(results.max_auto_channels(), 4);

    //     assert_eq!(results.to_close().len(), 2);
    //     assert_eq!(results.to_open().len(), 3);

    //     assert_vec_eq!(results.to_close(), vec![alice, gustave]);
    //     assert_vec_eq!(
    //         results
    //             .to_open()
    //             .into_iter()
    //             .map(|r| r.address)
    //             .collect::<Vec<Address>>(),
    //         vec![gustave, eugene, bob]
    //     );
    // }

    // #[test]
    // fn test_promiscuous_strategy_more_channels_than_allowed() {
    //     let mut strat = PromiscuousStrategy::default();
    //     let mut peers = HashMap::new();
    //     let mut outgoing_channels = Vec::new();
    //     for i in 0..100 {
    //         let address = Address::random();
    //         peers.insert(address.clone(), 0.9 - i as f64 * 0.02);
    //         if outgoing_channels.len() < 20 {
    //             outgoing_channels.push(OutgoingChannelStatus {
    //                 address,
    //                 stake: Balance::from_str("100000000000000000", BalanceType::HOPR),
    //                 status: Open,
    //             });
    //         }
    //     }

    //     // Add fake samples to allow the test to run
    //     strat.sma.add_sample(peers.len());
    //     strat.sma.add_sample(peers.len());

    //     let results = strat.tick(
    //         Balance::from_str("1000000000000000000", BalanceType::HOPR),
    //         peers.iter().map(|(&x, q)| (x.clone(), q.clone())),
    //         outgoing_channels.clone(),
    //     );

    //     assert_eq!(results.max_auto_channels(), 10);
    //     assert_eq!(results.to_open().len(), 0);
    //     assert_eq!(results.to_close().len(), 10);

    //     // Only the last 10 lowest quality channels get closed
    //     assert_vec_eq!(
    //         results.to_close(),
    //         outgoing_channels
    //             .into_iter()
    //             .rev()
    //             .map(|s| s.address)
    //             .take(10)
    //             .collect::<Vec<Address>>()
    //     );
    // }
}
