//! ## Promiscuous Strategy
//! This strategy opens or closes automatically channels based the following rules:
//! - if node quality is below or equal to a threshold `network_quality_threshold` and we have a channel opened to it, the strategy will close it
//!   - if node quality is above `network_quality_threshold` and no channel is opened yet, it will try to open channel to it (with initial stake `new_channel_stake`).
//!     However, the channel is opened only if the following is both true:
//!   - the total node balance does not drop below `minimum_node_balance`
//!   - the number of channels opened by this strategy does not exceed `max_channels`
//!
//! Also, the candidates for opening (quality > `network_quality_threshold`), are sorted by best quality first.
//! So that means if some nodes cannot have a channel opened to them, because we hit `minimum_node_balance` or `max_channels`,
//! the better quality ones were taking precedence.
//!
//! The sorting algorithm is intentionally unstable, so that the nodes which have the same quality get random order.
//! The constant `k` can be also set to a value > 1, which will make the strategy to open more channels for smaller networks,
//! but it would keep the same asymptotic properties.
//! Per default `k` = 1.
//!
//! The strategy starts acting only after at least `min_network_size_samples` network size samples were gathered, which means
//! it does not start opening/closing channels earlier than `min_network_size_samples` number of minutes after the node has started.
//!
//! For details on default parameters see [PromiscuousStrategyConfig].
//!
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

use async_lock::RwLock;
use async_trait::async_trait;
use chain_actions::channels::ChannelActions;
use futures::StreamExt;
use hopr_crypto_random::OsRng;
use hopr_db_sql::errors::DbSqlError;
use hopr_db_sql::peers::PeerSelector;
use hopr_db_sql::HoprDbAllOperations;
use rand::seq::SliceRandom;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Sub;
use std::str::FromStr;
use validator::Validate;

use crate::errors::Result;
use crate::errors::StrategyError::CriteriaNotSatisfied;
use crate::strategy::SingularStrategy;
use crate::Strategy;

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::{SimpleCounter, SimpleGauge};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_OPENS: SimpleCounter =
        SimpleCounter::new("hopr_strategy_promiscuous_opened_channels_count", "Count of open channel decisions").unwrap();
    static ref METRIC_COUNT_CLOSURES: SimpleCounter =
        SimpleCounter::new("hopr_strategy_promiscuous_closed_channels_count", "Count of close channel decisions").unwrap();
    static ref METRIC_MAX_AUTO_CHANNELS: SimpleGauge =
        SimpleGauge::new("hopr_strategy_promiscuous_max_auto_channels", "Count of maximum number of channels managed by the strategy").unwrap();
}

/// A decision made by the Promiscuous strategy on each tick,
/// represents which channels should be closed and which should be opened.
/// Also indicates a number of maximum channels this strategy can open given the current network size.
/// Note that the number changes as the network size changes.
#[derive(Clone, Debug, PartialEq, Default)]
struct ChannelDecision {
    to_close: Vec<ChannelEntry>,
    to_open: Vec<(Address, Balance)>,
}

impl ChannelDecision {
    pub fn will_channel_be_closed(&self, counter_party: &Address) -> bool {
        self.to_close.iter().any(|c| &c.destination == counter_party)
    }

    pub fn will_address_be_opened(&self, address: &Address) -> bool {
        self.to_open.iter().any(|(addr, _)| addr == address)
    }

    pub fn add_to_close(&mut self, entry: ChannelEntry) {
        self.to_close.push(entry);
    }

    pub fn add_to_open(&mut self, address: Address, balance: Balance) {
        self.to_open.push((address, balance));
    }

    pub fn get_to_close(&self) -> &Vec<ChannelEntry> {
        &self.to_close
    }

    pub fn get_to_open(&self) -> &Vec<(Address, Balance)> {
        &self.to_open
    }
}

impl Display for ChannelDecision {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "channel decision: opening ({}), closing({})",
            self.to_open.len(),
            self.to_close.len()
        )
    }
}

/// Configuration of [PromiscuousStrategy].
#[serde_as]
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct PromiscuousStrategyConfig {
    /// A quality threshold between 0 and 1 used to determine whether the strategy should open channel with the peer.
    #[validate(range(min = 0_f64, max = 1.0_f64))]
    #[default = 0.5]
    pub network_quality_threshold: f64,

    /// Minimum number of network quality samples before the strategy can start making decisions.
    #[validate(range(min = 1_u32))]
    #[default = 10]
    pub min_network_size_samples: u32,

    /// A stake of tokens that should be allocated to a channel opened by the strategy.
    ///
    /// Defaults to 10 HOPR
    #[serde_as(as = "DisplayFromStr")]
    #[default(Balance::new_from_str("10000000000000000000", BalanceType::HOPR))]
    pub new_channel_stake: Balance,

    /// Minimum token balance of the node. When reached, the strategy will not open any new channels.
    ///
    /// Defaults to 10 HOPR
    #[serde_as(as = "DisplayFromStr")]
    #[default(Balance::new_from_str("10000000000000000000", BalanceType::HOPR))]
    pub minimum_node_balance: Balance,

    /// Maximum number of opened channels the strategy should maintain.
    ///
    /// Defaults to square-root of the sampled network size.
    #[validate(range(min = 1))]
    pub max_channels: Option<usize>,

    /// If set, the strategy will aggressively close channels (even with peers above the `network_quality_threshold`)
    /// if the number of opened outgoing channels (regardless if opened by the strategy or manually) exceeds the
    /// `max_channels` limit.
    #[default = true]
    pub enforce_max_channels: bool,

    /// Specifies a minimum version (in semver syntax) of the peer the strategy should open a channel to.
    ///
    /// Default is ">=2.0.0"
    #[serde_as(as = "DisplayFromStr")]
    #[default(">=2.0.0".parse().unwrap())]
    pub minimum_peer_version: semver::VersionReq,
}

/// This strategy opens outgoing channels to peers, which have quality above a given threshold.
/// At the same time, it closes outgoing channels opened to peers whose quality dropped below this threshold.
pub struct PromiscuousStrategy<Db, A>
where
    Db: HoprDbAllOperations + Clone,
    A: ChannelActions,
{
    db: Db,
    chain_actions: A,
    cfg: PromiscuousStrategyConfig,
    sma: RwLock<SingleSumSMA<u32>>,
}

impl<Db, A> PromiscuousStrategy<Db, A>
where
    Db: HoprDbAllOperations + Clone,
    A: ChannelActions,
{
    pub fn new(cfg: PromiscuousStrategyConfig, db: Db, chain_actions: A) -> Self {
        Self {
            db,
            chain_actions,
            sma: RwLock::new(SingleSumSMA::new(cfg.min_network_size_samples as usize)),
            cfg,
        }
    }

    async fn sample_size_and_evaluate_avg(&self, sample: u32) -> Option<u32> {
        self.sma.write().await.push(sample);
        info!("evaluated qualities of {sample} peers seen in the network");

        let sma = self.sma.read().await;
        if sma.len() >= sma.window_size() {
            sma.average()
        } else {
            info!(
                "not yet enough samples ({} out of {}) of network size to perform a strategy tick, skipping.",
                sma.len(),
                sma.window_size()
            );
            None
        }
    }

    async fn get_peers_with_quality(&self) -> Result<HashMap<Address, f64>> {
        Ok(self
            .db
            .get_network_peers(PeerSelector::default(), false)
            .await?
            .filter_map(|status| async move {
                // Check if peer reports any version
                if let Some(version) = status.peer_version.clone().and_then(|v| {
                    semver::Version::from_str(&v)
                        .ok() // Workaround for https://github.com/dtolnay/semver/issues/315
                        .map(|v| Version::new(v.major, v.major, v.patch))
                }) {
                    // Check if the reported version matches the version semver expression
                    if self.cfg.minimum_peer_version.matches(&version) {
                        // Resolve peer's chain key and average quality
                        if let Ok(addr) = self
                            .db
                            .resolve_chain_key(&status.id.0)
                            .await
                            .and_then(|addr| addr.ok_or(DbSqlError::MissingAccount))
                        {
                            Some((addr, status.get_average_quality()))
                        } else {
                            error!("could not find on-chain address for {}", status.id.1);
                            None
                        }
                    } else {
                        debug!("version of peer {} reports non-matching version {version}", status.id.1);
                        None
                    }
                } else {
                    error!("cannot get version for peer id: {}", status.id.1);
                    None
                }
            })
            .collect()
            .await)
    }

    async fn collect_tick_decision(&self) -> Result<ChannelDecision> {
        let mut tick_decision = ChannelDecision::default();
        let mut new_channel_candidates: Vec<(Address, f64)> = Vec::new();

        let outgoing_open_channels = self
            .db
            .get_outgoing_channels(None)
            .await?
            .into_iter()
            .filter(|channel| channel.status == ChannelStatus::Open)
            .collect::<Vec<_>>();
        debug!("tracking {} open outgoing channels", outgoing_open_channels.len());

        // Check if we have enough network size samples before proceeding quality-based evaluation
        let peers_with_quality = self.get_peers_with_quality().await?;
        let current_average_network_size =
            match self.sample_size_and_evaluate_avg(peers_with_quality.len() as u32).await {
                Some(avg) => avg,
                None => return Err(CriteriaNotSatisfied), // not enough samples yet
            };

        // Go through all the peer ids we know, get their qualities and find out which channels should be closed and
        // which peer ids should become candidates for a new channel
        for (address, quality) in peers_with_quality.iter() {
            // Get channels we have opened with it
            let channel_with_peer = outgoing_open_channels.iter().find(|c| c.destination.eq(address));

            if let Some(channel) = channel_with_peer {
                if *quality <= self.cfg.network_quality_threshold {
                    // Need to close the channel, because quality has dropped
                    debug!(
                        "closure of channel to {}: {quality} <= {}",
                        channel.destination, self.cfg.network_quality_threshold
                    );
                    tick_decision.add_to_close(*channel);
                }
            } else if *quality >= self.cfg.network_quality_threshold {
                // Try to open channel with this peer, because it is high-quality, and we don't yet have a channel with it
                new_channel_candidates.push((*address, *quality));
            }
        }
        debug!(
            "proposed closures: {}, proposed new candidates: {}",
            tick_decision.get_to_close().len(),
            new_channel_candidates.len()
        );

        // We compute the upper bound for channels as a square-root of the perceived network size
        let max_auto_channels = self
            .cfg
            .max_channels
            .unwrap_or((current_average_network_size as f64).sqrt().ceil() as usize);
        debug!("current upper bound for maximum number of auto-channels is {max_auto_channels}");

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_MAX_AUTO_CHANNELS.set(max_auto_channels as f64);

        // Count all the effectively opened channels (i.e. after the decision has been made)
        let occupied = outgoing_open_channels
            .len()
            .saturating_sub(tick_decision.get_to_close().len());

        // If there are still more channels opened than we allow, close some
        // lowest-quality ones that passed the threshold
        if occupied > max_auto_channels && self.cfg.enforce_max_channels {
            warn!("there are {occupied} effectively opened channels, but the strategy allows only {max_auto_channels}");

            // Get all open channels that are not planned to be closed
            let mut sorted_channels = outgoing_open_channels
                .iter()
                .filter(|c| !tick_decision.will_channel_be_closed(&c.destination))
                .collect::<Vec<_>>();

            // Sort by quality, lowest-quality first
            sorted_channels.sort_unstable_by(|p1, p2| {
                let q1 = match peers_with_quality.get(&p1.destination) {
                    Some(q) => *q,
                    None => {
                        error!("could not determine peer quality for {p1}");
                        0_f64
                    }
                };
                let q2 = match peers_with_quality.get(&p2.destination) {
                    Some(q) => *q,
                    None => {
                        error!("could not determine peer quality for {p2}");
                        0_f64
                    }
                };
                q1.partial_cmp(&q2).expect("invalid comparison")
            });

            // Close the lowest-quality channels (those we did not mark for closing yet) to enforce the limit
            sorted_channels
                .into_iter()
                .take(occupied - max_auto_channels)
                .for_each(|channel| {
                    debug!("enforcing channel closure of {channel}");
                    tick_decision.add_to_close(*channel);
                });
        } else if max_auto_channels > occupied {
            // Sort the new channel candidates by the best quality first, then truncate to the number of available slots
            // This way, we'll prefer candidates with higher quality, when we don't have enough node balance.
            // Shuffle first, so the equal candidates are randomized and then use unstable sorting for that purpose.
            new_channel_candidates.shuffle(&mut OsRng);
            new_channel_candidates.sort_unstable_by(|(_, q1), (_, q2)| q1.partial_cmp(q2).unwrap().reverse());
            new_channel_candidates.truncate(max_auto_channels - occupied);
            debug!("got {} new channel candidates", new_channel_candidates.len());

            let mut remaining_balance = self.db.get_safe_hopr_balance(None).await?;

            // Go through the new candidates for opening channels allow them to open based on our available node balance
            for (address, _) in new_channel_candidates {
                // Stop if we ran out of balance
                if remaining_balance.le(&self.cfg.minimum_node_balance) {
                    warn!("ran out of allowed node balance - balance is {remaining_balance}");
                    break;
                }

                // If we haven't added this peer yet, add it to the list for channel opening
                if !tick_decision.will_address_be_opened(&address) {
                    tick_decision.add_to_open(address, self.cfg.new_channel_stake);
                    remaining_balance = remaining_balance.sub(&self.cfg.new_channel_stake);
                    debug!("promoted peer {address} for channel opening");
                }
            }
        } else {
            // max_channels == occupied
            info!("not going to allocate new channels, maximum number of effective channels is reached ({occupied})")
        }

        Ok(tick_decision)
    }
}

impl<Db, A> Debug for PromiscuousStrategy<Db, A>
where
    Db: HoprDbAllOperations + Clone,
    A: ChannelActions,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Strategy::Promiscuous(self.cfg.clone()))
    }
}

impl<Db, A> Display for PromiscuousStrategy<Db, A>
where
    Db: HoprDbAllOperations + Clone,
    A: ChannelActions,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::Promiscuous(self.cfg.clone()))
    }
}

#[async_trait]
impl<Db, A> SingularStrategy for PromiscuousStrategy<Db, A>
where
    Db: HoprDbAllOperations + Clone + Send + Sync,
    A: ChannelActions + Send + Sync,
{
    async fn on_tick(&self) -> Result<()> {
        let tick_decision = self.collect_tick_decision().await?;

        debug!("on tick executing {tick_decision}");

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
                    debug!("issued channel closing tx: {}", channel_to_close);
                }
                Err(e) => {
                    error!("error while closing channel: {e}");
                }
            }
        }

        for channel_to_open in tick_decision.get_to_open() {
            match self
                .chain_actions
                .open_channel(channel_to_open.0, channel_to_open.1)
                .await
            {
                Ok(_) => {
                    // Intentionally do not await result of the channel transaction
                    debug!("issued channel opening tx: {}", channel_to_open.0);
                }
                Err(e) => {
                    error!("error while issuing channel opening to {}: {e}", channel_to_open.0);
                }
            }
        }

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            METRIC_COUNT_OPENS.increment_by(tick_decision.get_to_open().len() as u64);
            METRIC_COUNT_CLOSURES.increment_by(tick_decision.get_to_close().len() as u64);
        }

        info!("on tick executed {tick_decision}");
        Ok(())
    }
}

/// Unit tests of pure Rust code
#[cfg(test)]
mod tests {
    use super::*;
    use chain_actions::action_queue::{ActionConfirmation, PendingAction};
    use chain_types::actions::Action;
    use chain_types::chain_events::ChainEventType;
    use core_network::{network::PeerOrigin, PeerId};
    use futures::{future::ok, FutureExt};
    use hex_literal::hex;
    use hopr_crypto_random::random_bytes;
    use hopr_crypto_types::prelude::*;
    use hopr_db_sql::accounts::HoprDbAccountOperations;
    use hopr_db_sql::channels::HoprDbChannelOperations;
    use hopr_db_sql::db::HoprDb;
    use hopr_db_sql::info::HoprDbInfoOperations;
    use hopr_db_sql::peers::HoprDbPeersOperations;
    use hopr_db_sql::HoprDbGeneralModelOperations;
    use lazy_static::lazy_static;
    use mockall::mock;

    lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))
        .unwrap();
        static ref PEERS: [(Address, PeerId); 10] = [
            (
                ALICE.public().to_address().into(),
                hex!("e03640d3184c8aa6f9d4ccd533281c51974a170c0c4d0fe1da9296a081ab1fd9")
            ),
            (
                hex!("5f98dc63889681eb4306f0e3b5ee2e04b13af7c8"),
                hex!("82a3cec1660697d8f3eb798f82ae281fc885c3e5370ef700c95c17397846c1e7")
            ),
            (
                hex!("6e0bed94a8d2da952ad4468ff81157b6137a5566"),
                hex!("2b93fcca9db2c5c12d1add5c07dd81d20c68eb713e99aa5c488210179c7505e3")
            ),
            (
                hex!("8275b9ce8a3d2fe14029111f85b72ab05aa0f5d3"),
                hex!("5cfd16dc160fd43396bfaff06e7c2e62cd087317671c159ce7cbc31c34fc32b6")
            ),
            (
                hex!("3231673fd10c9ebeb9330745f1709c91db9cf40f"),
                hex!("7f5b421cc58cf8449f5565756697261723fb96bba5f0aa2ba83c4973e0e994bf")
            ),
            (
                hex!("585f4ca77b07ac7a3bf37de3069b641ba97bf76f"),
                hex!("848af931ce57f54fbf96d7250eda8b0f36e3d1988ec8048c892e8d8ff0798f2f")
            ),
            (
                hex!("ba413645edb6ddbd46d5911466264b119087dfea"),
                hex!("d79258fc521dba8ded208066fe98fd8a857cf2e8f42f1b71c8f6e29b8f47e406")
            ),
            (
                hex!("9ea8c0f3766022f84c41abd524c942971bd22d23"),
                hex!("cd7a06caebcb90f95690c72472127cae8732b415440a1783c6ff9f9cb0bacf1e")
            ),
            (
                hex!("9790b6cf8afe6a7d80102570fac18a322e26ef83"),
                hex!("2dc3ff226be59333127ebfd3c79517eac8f81e0333abaa45189aae309880e55a")
            ),
            (
                hex!("f6ab491cd4e2eccbe60a7f87aeaacfc408dabde8"),
                hex!("5826ed44f52b3a26c472621812165bb2d3e60a9929e06db8b8df4e4d23068eba")
            ),
        ]
        .map(|(addr, privkey)| (
            addr.into(),
            OffchainKeypair::from_secret(&privkey).unwrap().public().into()
        ));
    }

    mock! {
        ChannelAct { }
        #[async_trait]
        impl ChannelActions for ChannelAct {
            async fn open_channel(&self, destination: Address, amount: Balance) -> chain_actions::errors::Result<PendingAction>;
            async fn fund_channel(&self, channel_id: Hash, amount: Balance) -> chain_actions::errors::Result<PendingAction>;
            async fn close_channel(
                &self,
                counterparty: Address,
                direction: ChannelDirection,
                redeem_before_close: bool,
            ) -> chain_actions::errors::Result<PendingAction>;
        }
    }

    async fn mock_channel(db: HoprDb, dst: Address, balance: Balance) -> ChannelEntry {
        let channel = ChannelEntry::new(
            PEERS[0].0,
            dst,
            balance,
            U256::zero(),
            ChannelStatus::Open,
            U256::zero(),
        );
        db.upsert_channel(None, channel).await.unwrap();

        channel
    }

    async fn prepare_network(db: HoprDb, qualities: Vec<f64>) {
        assert_eq!(qualities.len(), PEERS.len() - 1, "invalid network setup");

        for (i, quality) in qualities.into_iter().enumerate() {
            let peer = &PEERS[i + 1].1;

            db.add_network_peer(peer, PeerOrigin::Initialization, vec![], 0.0, 10)
                .await
                .unwrap();

            let mut status = db.get_network_peer(peer).await.unwrap().unwrap();
            status.peer_version = Some("2.0.0".into());
            while status.get_average_quality() < quality {
                status.update_quality(quality);
            }
            db.update_network_peer(status).await.unwrap();
        }
    }

    async fn init_db(db: HoprDb, node_balance: Balance) {
        db.begin_transaction()
            .await
            .unwrap()
            .perform(|tx| {
                Box::pin(async move {
                    db.set_safe_hopr_balance(Some(tx), node_balance).await?;
                    db.set_safe_hopr_allowance(Some(tx), node_balance).await?;
                    for (chain_key, peer_id) in PEERS.iter() {
                        db.insert_account(
                            Some(tx),
                            AccountEntry::new(
                                OffchainPublicKey::try_from(*peer_id).unwrap(),
                                *chain_key,
                                AccountType::NotAnnounced,
                            ),
                        )
                        .await?;
                    }
                    Ok::<_, DbSqlError>(())
                })
            })
            .await
            .unwrap();
    }

    fn mock_action_confirmation_closure(channel: ChannelEntry) -> ActionConfirmation {
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());
        ActionConfirmation {
            tx_hash: random_hash,
            event: Some(ChainEventType::ChannelClosureInitiated(channel)),
            action: Action::CloseChannel(channel, ChannelDirection::Outgoing),
        }
    }

    fn mock_action_confirmation_opening(address: Address, balance: Balance) -> ActionConfirmation {
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());
        ActionConfirmation {
            tx_hash: random_hash,
            event: Some(ChainEventType::ChannelOpened(ChannelEntry::new(
                PEERS[0].0,
                address,
                balance,
                U256::zero(),
                ChannelStatus::Open,
                U256::zero(),
            ))),
            action: Action::OpenChannel(address, balance),
        }
    }

    #[test]
    fn test_semver() {
        // See https://github.com/dtolnay/semver/issues/315
        let ver: semver::Version = "2.1.0-rc.3+commit.f75bc6c8".parse().expect("should be valid version");
        let stripped = semver::Version::new(ver.major, ver.minor, ver.patch);
        let req = semver::VersionReq::from_str(">=2.0.0").unwrap();
        assert!(req.matches(&stripped), "constraint must match");
    }

    #[async_std::test]
    async fn test_promiscuous_strategy_tick_decisions() {
        let _ = env_logger::builder().is_test(true).try_init();

        let db = HoprDb::new_in_memory(ALICE.clone()).await;

        let qualities_that_alice_sees = vec![0.7, 0.9, 0.8, 0.98, 0.1, 0.3, 0.1, 0.2, 1.0];

        let balance = Balance::new_from_str("100000000000000000000", BalanceType::HOPR);

        init_db(db.clone(), balance).await;
        prepare_network(db.clone(), qualities_that_alice_sees).await;

        mock_channel(db.clone(), PEERS[1].0, balance).await;
        mock_channel(db.clone(), PEERS[2].0, balance).await;
        let for_closing = mock_channel(db.clone(), PEERS[5].0, balance).await;

        // Peer 3 has an accepted pre-release version
        let mut status_3 = db.get_network_peer(&PEERS[3].1).await.unwrap().unwrap();
        status_3.peer_version = Some("2.1.0-rc.3+commit.f75bc6c8".into());
        db.update_network_peer(status_3).await.unwrap();

        // Peer 10 has an old node version
        let mut status_10 = db.get_network_peer(&PEERS[9].1).await.unwrap().unwrap();
        status_10.peer_version = Some("1.92.0".into());
        db.update_network_peer(status_10).await.unwrap();

        let mut strat_cfg = PromiscuousStrategyConfig::default();
        strat_cfg.max_channels = Some(3); // Allow max 3 channels
        strat_cfg.network_quality_threshold = 0.5;

        /*
            Situation:
            - There are max 3 channels.
            - Strategy will close channel to peer 5, because it has quality 0.1
            - Because of the closure, this means there can be 1 additional channel opened
                - Strategy can open channel either to peer 3, 4 or 10 (with qualities 0.8, 0.98 and 1.0 respectively)
                - It will ignore peer 9 even though it is highest quality, but does not meet minimum node version
                - It will prefer peer 4 because it has higher quality than node 3
        */

        let mut actions = MockChannelAct::new();
        actions
            .expect_close_channel()
            .times(1)
            .withf(|dst, dir, _| PEERS[5].0.eq(dst) && ChannelDirection::Outgoing.eq(dir))
            .return_once(move |_, _, _| Ok(ok(mock_action_confirmation_closure(for_closing)).boxed()));

        let new_stake = strat_cfg.new_channel_stake;
        actions
            .expect_open_channel()
            .times(1)
            .withf(move |dst, b| PEERS[4].0.eq(dst) && new_stake.eq(b))
            .return_once(move |_, _| Ok(ok(mock_action_confirmation_opening(PEERS[4].0, new_stake)).boxed()));

        let strat = PromiscuousStrategy::new(strat_cfg.clone(), db, actions);

        for _ in 0..strat_cfg.min_network_size_samples - 1 {
            strat
                .on_tick()
                .await
                .expect_err("on tick should fail when criteria are not met");
        }

        strat.on_tick().await.expect("on tick should not fail");
    }
}
