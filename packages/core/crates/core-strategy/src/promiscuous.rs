use core_crypto::types::OffchainPublicKey;
use core_types::channels::{
    ChannelDirection, ChannelEntry,
    ChannelStatus::{Open, PendingToClose},
};
use rand::rngs::OsRng;
use rand::seq::SliceRandom;
use simple_moving_average::{SumTreeSMA, SMA};
use std::collections::HashMap;
use utils_log::{debug, error, info, warn};
use utils_types::primitives::{Address, Balance, BalanceType};

use async_std::sync::RwLock;
use async_trait::async_trait;
use core_ethereum_actions::channels::ChannelActions;
use core_ethereum_actions::CoreEthereumActions;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_network::network::{Network, NetworkExternalActions};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use validator::Validate;

use crate::errors::Result;
use crate::strategy::SingularStrategy;
use crate::{decision::ChannelDecision, Strategy};
use utils_types::traits::PeerIdLike;

/// Size of the simple moving average window used to smoothen the number of registered peers.
pub const SMA_WINDOW_SIZE: usize = 3;

type SimpleMovingAvg = SumTreeSMA<usize, usize, SMA_WINDOW_SIZE>;

/// Config of promiscuous strategy.
#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Validate, Serialize, Deserialize)]
pub struct PromiscuousStrategyConfig {
    /// A quality threshold between 0 and 1 used to determine whether the strategy should open channel with the peer.
    /// Defaults to 0.5
    #[validate(range(min = 0_f32, max = 1.0_f32))]
    pub network_quality_threshold: f64,

    /// A stake of tokens that should be allocated to a channel opened by the strategy.
    /// Defaults to 10 HOPR
    #[serde_as(as = "DisplayFromStr")]
    pub new_channel_stake: Balance,

    /// Minimum token balance of the node. When reached, the strategy will not open any new channels.
    /// Defaults to 10 HOPR
    #[serde_as(as = "DisplayFromStr")]
    pub minimum_node_balance: Balance,

    /// Maximum number of opened channels the strategy should maintain.
    /// Defaults to square-root of the sampled network size.
    #[validate(range(min = 1))]
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
            new_channel_stake: Balance::new_from_str("10000000000000000000", BalanceType::HOPR),
            minimum_node_balance: Balance::new_from_str("10000000000000000000", BalanceType::HOPR),
            max_channels: None,
            enforce_max_channels: true,
        }
    }
}

/// This strategy opens outgoing channels to peers, which have quality above a given threshold.
/// At the same time, it closes outgoing channels opened to peers whose quality dropped below this threshold.
pub struct PromiscuousStrategy<Db, Net>
where
    Db: HoprCoreEthereumDbActions + Clone,
    Net: NetworkExternalActions,
{
    db: Arc<RwLock<Db>>,
    network: Arc<RwLock<Network<Net>>>,
    chain_actions: CoreEthereumActions<Db>,
    cfg: PromiscuousStrategyConfig,
    sma: RwLock<SimpleMovingAvg>,
}

impl<Db, Net> PromiscuousStrategy<Db, Net>
where
    Db: HoprCoreEthereumDbActions + Clone,
    Net: NetworkExternalActions,
{
    pub fn new(
        cfg: PromiscuousStrategyConfig,
        db: Arc<RwLock<Db>>,
        network: Arc<RwLock<Network<Net>>>,
        chain_actions: CoreEthereumActions<Db>,
    ) -> Self {
        Self {
            db,
            network,
            chain_actions,
            cfg,
            sma: RwLock::new(SimpleMovingAvg::new()),
        }
    }

    async fn collect_tick_decision(&self) -> Result<ChannelDecision> {
        let mut tick_decision = ChannelDecision::default();
        let mut new_channel_candidates: Vec<(Address, f64)> = Vec::new();
        let mut active_addresses: HashMap<Address, f64> = HashMap::new();
        let mut network_size: usize = 0;

        let balance: Balance = self.db.read().await.get_hopr_balance().await?;
        let outgoing_channels = self.db.read().await.get_outgoing_channels().await?;

        // Go through all the peer ids we know, get their qualities and find out which channels should be closed and
        // which peer ids should become candidates for a new channel
        // Also re-open all the channels that have dropped under minimum given balance
        let peers_with_quality = self.network.read().await.all_peers_with_quality();
        for (peer, quality) in peers_with_quality {
            let packet_key = OffchainPublicKey::from_peerid(&peer)?;
            // get the Ethereum address of the peer
            if let Some(address) = self.db.read().await.get_chain_key(&packet_key).await? {
                if tick_decision.will_channel_be_closed(&address)
                    || new_channel_candidates.iter().any(|(p, _)| p.eq(&address))
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
                    if quality <= self.cfg.network_quality_threshold {
                        // Need to close the channel, because quality has dropped
                        debug!("new channel closure candidate with {} (quality {})", address, quality);
                        tick_decision.add_to_close(*channel);
                    }
                } else if quality >= self.cfg.network_quality_threshold {
                    // Try to open channel with this peer, because it is high-quality
                    debug!("new channel opening candidate {} with quality {}", address, quality);
                    new_channel_candidates.push((address, quality));
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
            return Ok(tick_decision);
        }

        // Also mark for closing all channels which are in PendingToClose state
        let before_pending = outgoing_channels.len();
        outgoing_channels
            .iter()
            .filter(|c| c.status == PendingToClose)
            .for_each(|c| {
                tick_decision.add_to_close(*c);
            });
        debug!(
            "{} channels are in PendingToClose, so strategy will mark them for closing too",
            outgoing_channels.len() - before_pending
        );

        // We compute the upper bound for channels as a square-root of the perceived network size
        let max_auto_channels = self
            .cfg
            .max_channels
            .unwrap_or((self.sma.read().await.get_average() as f64).sqrt().ceil() as usize);
        debug!(
            "current upper bound for maximum number of auto-channels is {}",
            max_auto_channels
        );

        // Count all the opened channels
        let count_opened = outgoing_channels.iter().filter(|c| c.status == Open).count();
        let occupied = if count_opened > tick_decision.get_to_close().len() {
            count_opened - tick_decision.get_to_close().len()
        } else {
            0
        };

        // If there is still more channels opened than we allow, close some
        // lowest-quality ones which passed the threshold
        if occupied > max_auto_channels && self.cfg.enforce_max_channels {
            warn!(
                "there are {} opened channels, but the strategy allows only {}",
                occupied, max_auto_channels
            );

            let mut sorted_channels: Vec<ChannelEntry> = outgoing_channels
                .iter()
                .filter(|c| !tick_decision.will_channel_be_closed(&c.destination))
                .cloned()
                .collect();

            // Sort by quality, lowest-quality first
            sorted_channels.sort_unstable_by(|p1, p2| {
                active_addresses
                    .get(&p1.destination)
                    .zip(active_addresses.get(&p2.destination))
                    .and_then(|(q1, q2)| q1.partial_cmp(q2))
                    .expect(format!("failed to retrieve quality of {} or {}", p1.destination, p2.destination).as_str())
            });

            // Close the lowest-quality channels (those we did not mark for closing yet)
            sorted_channels
                .into_iter()
                .take(occupied - max_auto_channels)
                .for_each(|c| {
                    tick_decision.add_to_close(c);
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
            let mut remaining_balance = balance;
            for address in new_channel_candidates.into_iter().map(|(p, _)| p) {
                // Stop if we ran out of balance
                if remaining_balance.lte(&self.cfg.minimum_node_balance) {
                    warn!(
                        "strategy ran out of allowed node balance - balance is {}",
                        remaining_balance.to_string()
                    );
                    break;
                }

                // If we haven't added this peer yet, add it to the list for channel opening
                if !tick_decision.will_address_be_opened(&address) {
                    debug!("promoting peer {} for channel opening", address);
                    tick_decision.add_to_open(address, self.cfg.new_channel_stake);
                    remaining_balance = balance.sub(&self.cfg.new_channel_stake);
                }
            }
        }

        debug!(
            "strategy tick #{} result: {} peers for channel opening, {} peer for channel closure",
            self.sma.read().await.get_num_samples(),
            tick_decision.get_to_open().len(),
            tick_decision.get_to_close().len()
        );
        Ok(tick_decision)
    }
}

impl<Db, Net> Debug for PromiscuousStrategy<Db, Net>
where
    Db: HoprCoreEthereumDbActions + Clone,
    Net: NetworkExternalActions,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Strategy::Promiscuous(self.cfg))
    }
}

impl<Db, Net> Display for PromiscuousStrategy<Db, Net>
where
    Db: HoprCoreEthereumDbActions + Clone,
    Net: NetworkExternalActions,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::Promiscuous(self.cfg))
    }
}

#[async_trait(? Send)]
impl<Db, Net> SingularStrategy for PromiscuousStrategy<Db, Net>
where
    Db: HoprCoreEthereumDbActions + Clone,
    Net: NetworkExternalActions,
{
    async fn on_tick(&self) -> Result<()> {
        let tick_decision = self.collect_tick_decision().await?;

        // close all the channels, need to be synchronous because Ethereum transactions
        // are synchronous, especially due nonces
        for channel_to_close in tick_decision.get_to_close() {
            match self
                .chain_actions
                .close_channel(
                    channel_to_close.destination,
                    ChannelDirection::Outgoing,
                    false, // TODO: get this value from config
                )
                .await
            {
                Ok(_) => {
                    // Intentionally do not await result of the channel transaction
                    info!("{self} strategy: issued channel closing tx: {}", channel_to_close);
                }
                Err(e) => {
                    error!("{self} strategy: error while closing channel: {e}");
                }
            }
        }
        debug!("{self} strategy: close channels done");

        // open all the channels, need to be synchronous because Ethereum
        // transactions are synchronous, especially due to nonces
        for channel_to_open in tick_decision.get_to_open() {
            match self
                .chain_actions
                .open_channel(channel_to_open.0, channel_to_open.1)
                .await
            {
                Ok(_) => {
                    // Intentionally do not await result of the channel transaction
                    info!("{self} strategy: issued channel opening tx: {}", channel_to_open.0);
                }
                Err(e) => {
                    error!(
                        "{self} strategy: error while issuing channel opening to {}: {e}",
                        channel_to_open.0
                    );
                }
            }
        }

        debug!("{self} strategy: open channels done");

        //while let Some(_) = receivers.next().await {}

        debug!("{self} strategy: channel operations done");

        Ok(())
    }
}

/// Unit tests of pure Rust code
#[cfg(test)]
mod tests {
    use super::*;
    use core_crypto::{
        keypairs::{Keypair, OffchainKeypair},
        types::Hash,
    };
    use core_ethereum_actions::transaction_queue::{TransactionExecutor, TransactionQueue, TransactionResult};
    use core_ethereum_db::db::CoreEthereumDb;
    use core_network::{
        network::{NetworkConfig, NetworkEvent, NetworkExternalActions, PeerOrigin},
        PeerId,
    };
    use core_types::acknowledgement::AcknowledgedTicket;
    use core_types::channels::ChannelStatus;
    use futures::future::join_all;
    use utils_db::{db::DB, rusty::RustyLevelDbShim};
    use utils_misc::time::native::current_timestamp;
    use utils_types::primitives::{Snapshot, U256};

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
        async fn fund_channel(&self, _destination: Address, _amount: Balance) -> TransactionResult {
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

        fn create_timestamp(&self) -> u64 {
            current_timestamp()
        }
    }

    fn generate_random_address_and_peer_id_pairs(num: u32) -> Vec<(Address, PeerId)> {
        (0..num)
            .map(|_| (Address::random(), OffchainKeypair::random().public().to_peerid()))
            .collect()
    }

    async fn add_peer_and_bump_its_network_quality_many_times<Db, Net>(
        strategy: &PromiscuousStrategy<Db, Net>,
        peer: &PeerId,
        steps: u32,
    ) where
        Db: HoprCoreEthereumDbActions + Clone,
        Net: NetworkExternalActions,
    {
        strategy.network.write().await.add(peer, PeerOrigin::Initialization);

        assert_eq!(
            strategy.network.read().await.get_peer_status(peer).unwrap().quality,
            0f64
        );
        join_all((0..steps).map(|_| async { strategy.network.write().await.update(peer, Ok(current_timestamp())) }))
            .await;
        assert_eq!(
            (strategy.network.read().await.get_peer_status(peer).unwrap().quality / 0.1f64).round() as u32,
            steps
        );
    }

    async fn mock_promiscuous_strategy() -> (
        PromiscuousStrategy<CoreEthereumDb<RustyLevelDbShim>, MockNetworkExternalActions>,
        Vec<(Address, PeerId)>,
    ) {
        let address_peer_id_pairs = generate_random_address_and_peer_id_pairs(10);
        let (alice_address, alice_peer_id) = address_peer_id_pairs[0];
        let (bob_address, bob_peer_id) = address_peer_id_pairs[1];
        let (charlie_address, charlie_peer_id) = address_peer_id_pairs[2];
        let (_eugene_address, eugene_peer_id) = address_peer_id_pairs[3];
        let (gustave_address, gustave_peer_id) = address_peer_id_pairs[4];

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            alice_address,
        )));

        let network = Arc::new(RwLock::new(Network::new(
            alice_peer_id,
            NetworkConfig::default(),
            MockNetworkExternalActions {},
        )));

        // Start the TransactionQueue with the mock TransactionExecutor
        let tx_queue = TransactionQueue::new(db.clone(), Box::new(MockTransactionExecutor::new()));
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn_local(async move {
            tx_queue.transaction_loop().await;
        });

        let actions = CoreEthereumActions::new(alice_address, db.clone(), tx_sender);

        let strat_cfg = PromiscuousStrategyConfig::default();

        let strat = PromiscuousStrategy::new(strat_cfg, db, network, actions);

        // create a network with:
        let peers: Vec<(PeerId, u32)> = vec![
            (bob_peer_id.clone(), 7),        // - bob: 0.7
            (charlie_peer_id.clone(), 9),    // - charlie: 0.9
            (eugene_peer_id.clone(), 10),    // - eugene: 0.8
            (gustave_peer_id.clone(), 2),    // - gustave: 1.0
            (address_peer_id_pairs[5].1, 1), // - random_peer: 0.1
            (address_peer_id_pairs[6].1, 3), // - random_peer: 0.3
            (address_peer_id_pairs[7].1, 1), // - random_peer: 0.1
            (address_peer_id_pairs[8].1, 2), // - random_peer: 0.2
            (address_peer_id_pairs[9].1, 3), // - random_peer: 0.3
        ];

        join_all(peers.iter().map(|(peer_id, step)| async {
            add_peer_and_bump_its_network_quality_many_times(&strat, peer_id, *step).await;
        }))
        .await;

        let balance = Balance::new_from_str("11000000000000000000", BalanceType::HOPR); // 11 HOPR
        let low_balance = Balance::new_from_str("1000000000000000", BalanceType::HOPR); // 0.001 HOPR
                                                                                        // set HOPR balance in DB
        strat.db.write().await.set_hopr_balance(&balance).await.unwrap();
        // link chain key and packet key
        join_all(address_peer_id_pairs.iter().map(|pair| async {
            strat
                .db
                .write()
                .await
                .link_chain_and_packet_keys(
                    &pair.0,
                    &OffchainPublicKey::from_peerid(&pair.1).unwrap(),
                    &Snapshot::default(),
                )
                .await
                .unwrap();
        }))
        .await;
        // add some channels in DB
        let outgoing_channels = vec![
            ChannelEntry::new(
                alice_address.clone(),
                bob_address.clone(),
                balance.clone(),
                U256::zero(),
                ChannelStatus::Open,
                U256::zero(),
                U256::zero(),
            ),
            ChannelEntry::new(
                alice_address.clone(),
                charlie_address.clone(),
                balance.clone(),
                U256::zero(),
                ChannelStatus::Open,
                U256::zero(),
                U256::zero(),
            ),
            ChannelEntry::new(
                alice_address.clone(),
                gustave_address.clone(),
                low_balance.clone(),
                U256::zero(),
                ChannelStatus::Open,
                U256::zero(),
                U256::zero(),
            ),
        ];
        join_all(outgoing_channels.iter().map(|channel| async {
            strat
                .db
                .write()
                .await
                .update_channel_and_snapshot(&channel.get_id(), channel, &Snapshot::default())
                .await
                .unwrap()
        }))
        .await;
        // set allowance
        strat
            .db
            .write()
            .await
            .set_staking_safe_allowance(&balance, &Snapshot::default())
            .await
            .unwrap();

        // Add fake samples to allow the test to run
        strat.sma.write().await.add_sample(peers.len());
        strat.sma.write().await.add_sample(peers.len());
        (strat, address_peer_id_pairs)
    }

    #[async_std::test]
    async fn test_promiscuous_strategy_config() {
        let (strat, _) = mock_promiscuous_strategy().await;
        assert_eq!(strat.to_string(), "promiscuous");
    }

    #[async_std::test]
    async fn test_promiscuous_strategy_tick_decisions() {
        let (strat, address_peer_pairs) = mock_promiscuous_strategy().await;
        let tick_decision = strat.collect_tick_decision().await.unwrap();

        // let (to_close, to_open) = strat.on_tick().await.unwrap();

        // assert that there's 0 channel closed and 1 opened (eugene, at index 3).
        assert_eq!(tick_decision.get_to_close().len(), 1usize, "should close 1 channel");
        assert_eq!(tick_decision.get_to_open().len(), 1usize, "should open 1 channel");
        assert_eq!(
            tick_decision.get_to_close()[0].destination,
            address_peer_pairs[4].0,
            "should close channel to gustave"
        );
        assert_eq!(
            tick_decision.get_to_open()[0].0,
            address_peer_pairs[3].0,
            "should open channel to eugene"
        );
    }

    #[async_std::test]
    async fn test_promiscuous_strategy_on_tick() {
        let (strat, _) = mock_promiscuous_strategy().await;

        strat.on_tick().await.unwrap();
    }
}
