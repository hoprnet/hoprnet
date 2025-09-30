//! ## Promiscuous Strategy
//! This strategy opens or closes automatically channels based the following rules:
//! - if node quality is below or equal to a threshold `network_quality_threshold` and we have a channel opened to it,
//!   the strategy will close it
//!   - if node quality is above `network_quality_threshold` and no channel is opened yet, it will try to open channel
//!     to it (with initial stake `new_channel_stake`). However, the channel is opened only if the following is both
//!     true:
//!   - the total node balance does not drop below `minimum_node_balance`
//!   - the number of channels opened by this strategy does not exceed `max_channels`
//!
//! Also, the candidates for opening (quality > `network_quality_threshold`), are sorted by best quality first.
//! So that means if some nodes cannot have a channel opened to them, because we hit `minimum_node_balance` or
//! `max_channels`, the better quality ones were taking precedence.
//!
//! The sorting algorithm is intentionally unstable, so that the nodes which have the same quality get random order.
//! The constant `k` can be also set to a value > 1, which will make the strategy to open more channels for smaller
//! networks, but it would keep the same asymptotic properties.
//! Per default `k` = 1.
//!
//! The strategy starts acting only after at least `min_network_size_samples` network size samples were gathered, which
//! means it does not start opening/closing channels earlier than `min_network_size_samples` number of minutes after the
//! node has started.
//!
//! For details on default parameters see [PromiscuousStrategyConfig].
use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
    time::Duration,
};

use async_trait::async_trait;
use futures::StreamExt;
use hopr_api::{
    chain::{
        ChainKeyOperations, ChainReadAccountOperations, ChainReadChannelOperations, ChainWriteChannelOperations,
        ChannelSelector,
    },
    db::{HoprDbPeersOperations, PeerSelector},
};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use tracing::{debug, error, info, trace, warn};

use crate::{
    Strategy,
    errors::{Result, StrategyError, StrategyError::CriteriaNotSatisfied},
    strategy::SingularStrategy,
};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_OPENS: hopr_metrics::SimpleCounter =
        hopr_metrics::SimpleCounter::new("hopr_strategy_promiscuous_opened_channels_count", "Count of open channel decisions").unwrap();
    static ref METRIC_COUNT_CLOSURES: hopr_metrics::SimpleCounter =
        hopr_metrics::SimpleCounter::new("hopr_strategy_promiscuous_closed_channels_count", "Count of close channel decisions").unwrap();
    static ref METRIC_MAX_AUTO_CHANNELS: hopr_metrics::SimpleGauge =
        hopr_metrics::SimpleGauge::new("hopr_strategy_promiscuous_max_auto_channels", "Count of maximum number of channels managed by the strategy").unwrap();
}

/// A decision made by the Promiscuous strategy on each tick,
/// represents which channels should be closed and which should be opened.
/// Also indicates a number of maximum channels this strategy can open given the current network size.
/// Note that the number changes as the network size changes.
#[derive(Clone, Debug, PartialEq, Default)]
struct ChannelDecision {
    to_close: Vec<ChannelEntry>,
    to_open: Vec<(Address, HoprBalance)>,
}

impl ChannelDecision {
    pub fn will_channel_be_closed(&self, counter_party: &Address) -> bool {
        self.to_close.iter().any(|c| &c.destination == counter_party)
    }

    pub fn add_to_close(&mut self, entry: ChannelEntry) {
        self.to_close.push(entry);
    }

    pub fn add_to_open(&mut self, address: Address, balance: HoprBalance) {
        self.to_open.push((address, balance));
    }

    pub fn get_to_close(&self) -> &Vec<ChannelEntry> {
        &self.to_close
    }

    pub fn get_to_open(&self) -> &Vec<(Address, HoprBalance)> {
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

#[inline]
fn default_new_channel_stake() -> HoprBalance {
    HoprBalance::new_base(10)
}

#[inline]
fn default_min_safe_balance() -> HoprBalance {
    HoprBalance::new_base(1000)
}

#[inline]
fn default_network_quality_open_threshold() -> f64 {
    0.9
}

#[inline]
fn default_network_quality_close_threshold() -> f64 {
    0.2
}

#[inline]
fn default_minimum_pings() -> u32 {
    50
}

#[inline]
fn just_true() -> bool {
    true
}

#[inline]
fn default_initial_delay() -> Duration {
    Duration::from_secs(5 * 60)
}

const MIN_AUTO_DETECTED_MAX_AUTO_CHANNELS: usize = 10;

/// Configuration of [PromiscuousStrategy].
#[serde_as]
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Serialize, Deserialize)]
pub struct PromiscuousStrategyConfig {
    /// A quality threshold between 0 and 1 used to determine whether the strategy should open channel with the peer.
    ///
    /// Default is 0.9
    #[serde(default = "default_network_quality_open_threshold")]
    #[default(default_network_quality_open_threshold())]
    pub network_quality_open_threshold: f64,

    /// A quality threshold between 0 and 1 used to determine whether the strategy should close channel with the peer.
    /// If set to 0, no channels will be closed.
    ///
    /// Default is 0.2
    #[serde(default = "default_network_quality_close_threshold")]
    #[default(default_network_quality_close_threshold())]
    pub network_quality_close_threshold: f64,

    /// Number of heartbeats sent to the peer before it is considered for selection.
    ///
    /// Default is 50.
    #[serde(default = "default_minimum_pings")]
    #[default(default_minimum_pings())]
    pub minimum_peer_pings: u32,

    /// Initial delay from startup before the strategy starts taking decisions.
    ///
    /// Default is 5 minutes.
    #[serde(default = "default_initial_delay")]
    #[default(default_initial_delay())]
    pub initial_delay: Duration,

    /// A stake of tokens that should be allocated to a channel opened by the strategy.
    ///
    /// Default is 10 wxHOPR
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "default_new_channel_stake")]
    #[default(default_new_channel_stake())]
    pub new_channel_stake: HoprBalance,

    /// Minimum token balance of the node's Safe.
    /// When reached, the strategy will not open any new channels.
    ///
    /// Default is 1000 wxHOPR
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "default_min_safe_balance")]
    #[default(default_min_safe_balance())]
    pub minimum_safe_balance: HoprBalance,

    /// The maximum number of opened channels the strategy should maintain.
    ///
    /// Defaults to square-root of the sampled network size, the minimum is 10.
    pub max_channels: Option<usize>,

    /// If set, the strategy will aggressively close channels
    /// (even with peers above the `network_quality_close_threshold`)
    /// if the number of opened outgoing channels (regardless if opened by the strategy or manually) exceeds the
    /// `max_channels` limit.
    ///
    /// Default is true.
    #[serde(default = "just_true")]
    #[default(true)]
    pub enforce_max_channels: bool,
}

impl validator::Validate for PromiscuousStrategyConfig {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        let mut errors = validator::ValidationErrors::new();

        if !(0.0..=1.0).contains(&self.network_quality_open_threshold) {
            errors.add(
                "network_quality_open_threshold",
                validator::ValidationError::new("must be in [0..1]"),
            );
        }

        if !(0.0..=1.0).contains(&self.network_quality_close_threshold) {
            errors.add(
                "network_quality_close_threshold",
                validator::ValidationError::new("must be in [0..1]"),
            );
        }

        if self.network_quality_open_threshold <= self.network_quality_close_threshold {
            errors.add(
                "network_quality_open_threshold,network_quality_close_threshold",
                validator::ValidationError::new(
                    "network_quality_open_threshold must be greater than network_quality_close_threshold",
                ),
            );
        }

        if self.minimum_peer_pings == 0 {
            errors.add(
                "minimum_peer_pings",
                validator::ValidationError::new("must be greater than 0"),
            );
        }

        if self.new_channel_stake.is_zero() {
            errors.add(
                "new_channel_stake",
                validator::ValidationError::new("must be greater than 0"),
            );
        }

        if self.max_channels.is_some_and(|m| m == 0) {
            errors.add(
                "max_channels",
                validator::ValidationError::new("must be greater than 0"),
            );
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// This strategy opens outgoing channels to peers, which have quality above a given threshold.
/// At the same time, it closes outgoing channels opened to peers whose quality dropped below this threshold.
pub struct PromiscuousStrategy<Db, A> {
    db: Db,
    hopr_chain_actions: A,
    cfg: PromiscuousStrategyConfig,
    started_at: std::time::Instant,
}

#[derive(Debug, Default)]
struct NetworkStats {
    pub peers_with_quality: HashMap<Address, (f64, u64)>,
    pub num_online_peers: usize,
}

impl<Db, A> PromiscuousStrategy<Db, A>
where
    Db: HoprDbPeersOperations,
    A: ChainReadAccountOperations
        + ChainReadChannelOperations
        + ChainWriteChannelOperations
        + ChainKeyOperations
        + Clone
        + Send
        + Sync,
{
    pub fn new(cfg: PromiscuousStrategyConfig, db: Db, hopr_chain_actions: A) -> Self {
        #[cfg(all(feature = "prometheus", not(test)))]
        {
            lazy_static::initialize(&METRIC_MAX_AUTO_CHANNELS);
            lazy_static::initialize(&METRIC_COUNT_CLOSURES);
            lazy_static::initialize(&METRIC_COUNT_OPENS);
        }

        Self {
            db,
            hopr_chain_actions,
            cfg,
            started_at: std::time::Instant::now(),
        }
    }

    async fn get_network_stats(&self) -> Result<NetworkStats> {
        let mut num_online_peers = 0;
        let chain_actions = self.hopr_chain_actions.clone();
        Ok(NetworkStats {
            peers_with_quality: self
                .db
                .get_network_peers(PeerSelector::default(), false)
                .await
                .map_err(|e| StrategyError::Other(e.into()))?
                .inspect(|status| {
                    if status.quality > 0.0 {
                        num_online_peers += 1;
                    } else {
                        trace!(peer = %status.id.1, "peer is not online");
                    }
                })
                .filter_map(move |status| {
                    let chain_actions = chain_actions.clone();
                    async move {
                        // Resolve peer's chain key and average quality
                        if let Ok(Some(addr)) = chain_actions.packet_key_to_chain_key(&status.id.0).await {
                            Some((addr, (status.get_average_quality(), status.heartbeats_sent)))
                        } else {
                            error!(address = %status.id.1, "could not find on-chain address");
                            None
                        }
                    }
                })
                .collect()
                .await,
            num_online_peers,
        })
    }

    async fn collect_tick_decision(&self) -> Result<ChannelDecision> {
        let mut tick_decision = ChannelDecision::default();
        let mut new_channel_candidates: Vec<(Address, f64)> = Vec::new();

        // Get all opened outgoing channels from this node
        let our_outgoing_open_channels = self
            .hopr_chain_actions
            .stream_channels(ChannelSelector {
                direction: vec![ChannelDirection::Outgoing],
                allowed_states: vec![ChannelStatusDiscriminants::Open],
                ..Default::default()
            })
            .await
            .map_err(|e| StrategyError::Other(e.into()))?
            .collect::<Vec<_>>()
            .await;
        debug!(
            count = our_outgoing_open_channels.len(),
            "tracking open outgoing channels"
        );

        let network_stats = self.get_network_stats().await?;
        debug!(?network_stats, "retrieved network stats");

        // Close all channels to nodes that are not in the network peers
        // The initial_delay should take care of prior heartbeats to take place.
        our_outgoing_open_channels
            .iter()
            .filter(|channel| !network_stats.peers_with_quality.contains_key(&channel.destination))
            .for_each(|channel| {
                debug!(destination = %channel.destination, "destination of opened channel is not between the network peers");
                tick_decision.add_to_close(*channel);
            });

        // Go through all the peer ids and their qualities
        // to find out which channels should be closed and
        // which peer ids should become candidates for a new channel
        for (address, (quality, num_pings)) in network_stats.peers_with_quality.iter() {
            // Get the channel we have opened with it
            let channel_with_peer = our_outgoing_open_channels.iter().find(|c| c.destination.eq(address));

            if let Some(channel) = channel_with_peer {
                if *quality < self.cfg.network_quality_close_threshold
                    && *num_pings >= self.cfg.minimum_peer_pings as u64
                {
                    // Need to close the channel because quality has dropped
                    debug!(destination = %channel.destination, quality = %quality, threshold = self.cfg.network_quality_close_threshold,
                        "strategy proposes to close existing channel"
                    );
                    tick_decision.add_to_close(*channel);
                }
            } else if *quality >= self.cfg.network_quality_open_threshold
                && *num_pings >= self.cfg.minimum_peer_pings as u64
            {
                // Try to open a channel with this peer, because it is high-quality,
                // and we don't yet have a channel with it
                debug!(destination = %address, quality = %quality, threshold = self.cfg.network_quality_open_threshold,
                    "strategy proposes to open a new channel");
                new_channel_candidates.push((*address, *quality));
            }
        }
        debug!(
            proposed_closures = tick_decision.get_to_close().len(),
            proposed_openings = new_channel_candidates.len(),
            "channel decision proposal summary"
        );

        // We compute the upper bound for channels as a square-root of the perceived network size
        let max_auto_channels = self.cfg.max_channels.unwrap_or(
            MIN_AUTO_DETECTED_MAX_AUTO_CHANNELS.max((network_stats.num_online_peers as f64).sqrt().ceil() as usize),
        );
        debug!(
            max_auto_channels,
            "current upper bound for maximum number of auto-channels"
        );

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_MAX_AUTO_CHANNELS.set(max_auto_channels as f64);

        // Count all the effectively opened channels (i.e., after the decisions have been made)
        let occupied = our_outgoing_open_channels
            .len()
            .saturating_sub(tick_decision.get_to_close().len());

        // If there are still more channels opened than we allow, close some
        // lowest-quality ones that passed the threshold
        if occupied > max_auto_channels && self.cfg.enforce_max_channels {
            warn!(
                count = occupied,
                max_auto_channels, "the strategy allows only less occupied channels"
            );

            // Get all open channels that are not planned to be closed
            let mut sorted_channels = our_outgoing_open_channels
                .iter()
                .filter(|c| !tick_decision.will_channel_be_closed(&c.destination))
                .collect::<Vec<_>>();

            // Sort by quality, lowest-quality first
            sorted_channels.sort_unstable_by(|p1, p2| {
                let q1 = match network_stats.peers_with_quality.get(&p1.destination) {
                    Some((q, _)) => *q,
                    None => {
                        error!(channel = ?p1, "could not determine peer quality");
                        0_f64
                    }
                };
                let q2 = match network_stats.peers_with_quality.get(&p2.destination) {
                    Some((q, _)) => *q,
                    None => {
                        error!(peer = %p2, "could not determine peer quality");
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
                    debug!(destination = %channel.destination, "enforcing channel closure");
                    tick_decision.add_to_close(*channel);
                });
        } else if max_auto_channels > occupied {
            // Sort the new channel candidates by the best quality first, then truncate to the number of available slots
            // This way, we'll prefer candidates with higher quality, when we don't have enough node balance.
            // Shuffle first, so the equal candidates are randomized and then use unstable sorting for that purpose.
            new_channel_candidates.shuffle(&mut hopr_crypto_random::rng());
            new_channel_candidates
                .sort_unstable_by(|(_, q1), (_, q2)| q1.partial_cmp(q2).expect("should be comparable").reverse());
            new_channel_candidates.truncate(max_auto_channels - occupied);
            debug!(count = new_channel_candidates.len(), "got new channel candidates");

            let current_safe_balance: HoprBalance = self
                .hopr_chain_actions
                .safe_balance()
                .await
                .map_err(|e| StrategyError::Other(e.into()))?;

            // Check if we do not surpass the minimum node's balance while opening new channels
            let max_to_open = ((current_safe_balance - self.cfg.minimum_safe_balance).amount()
                / self.cfg.new_channel_stake.amount())
            .as_usize();
            debug!(%current_safe_balance, max_to_open, num_candidates = new_channel_candidates.len(), "maximum number of channel openings with current balance");
            new_channel_candidates
                .into_iter()
                .take(max_to_open)
                .for_each(|(address, _)| tick_decision.add_to_open(address, self.cfg.new_channel_stake));
        } else {
            // max_channels == occupied
            info!(
                count = occupied,
                "not going to allocate new channels, maximum number of effective channels is reached"
            )
        }

        Ok(tick_decision)
    }
}

impl<Db, A> Debug for PromiscuousStrategy<Db, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Strategy::Promiscuous(self.cfg.clone()))
    }
}

impl<Db, A> Display for PromiscuousStrategy<Db, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::Promiscuous(self.cfg.clone()))
    }
}

#[async_trait]
impl<Db, A> SingularStrategy for PromiscuousStrategy<Db, A>
where
    Db: HoprDbPeersOperations + Clone + Send + Sync,
    A: ChainReadAccountOperations
        + ChainReadChannelOperations
        + ChainKeyOperations
        + ChainWriteChannelOperations
        + Clone
        + Send
        + Sync,
{
    async fn on_tick(&self) -> Result<()> {
        let safe_balance: HoprBalance = self
            .hopr_chain_actions
            .safe_balance()
            .await
            .map_err(|e| StrategyError::Other(e.into()))?;
        if safe_balance <= self.cfg.minimum_safe_balance {
            error!(
                "strategy cannot work with safe token balance already being less or equal than minimum node balance"
            );
            return Err(CriteriaNotSatisfied);
        }

        if self.started_at.elapsed() < self.cfg.initial_delay {
            debug!("strategy is not yet ready to execute, waiting for initial delay");
            return Err(CriteriaNotSatisfied);
        }

        let tick_decision = self.collect_tick_decision().await?;
        debug!(%tick_decision, "collected channel decision");

        for channel_to_close in tick_decision.get_to_close() {
            match self.hopr_chain_actions.close_channel(&channel_to_close.get_id()).await {
                Ok(_) => {
                    // Intentionally do not await result of the channel transaction
                    debug!(destination = %channel_to_close.destination, "issued channel closing");

                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_COUNT_CLOSURES.increment();
                }
                Err(e) => {
                    error!(error = %e, "error while closing channel");
                }
            }
        }

        for channel_to_open in tick_decision.get_to_open() {
            match self
                .hopr_chain_actions
                .open_channel(&channel_to_open.0, channel_to_open.1)
                .await
            {
                Ok(_) => {
                    // Intentionally do not await result of the channel transaction
                    debug!(destination = %channel_to_open.0, "issued channel opening");

                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_COUNT_OPENS.increment();
                }
                Err(e) => {
                    error!(error = %e, channel = %channel_to_open.0, "error while issuing channel opening");
                }
            }
        }

        info!(%tick_decision, "on tick executed");
        Ok(())
    }
}

/// Unit tests of pure Rust code
#[cfg(test)]
mod tests {
    use anyhow::Context;
    use futures::{FutureExt, future::ok};
    use hex_literal::hex;
    use hopr_chain_types::{actions::Action, chain_events::ChainEventType};
    use hopr_crypto_random::random_bytes;
    use hopr_crypto_types::prelude::*;
    use hopr_db_sql::{
        HoprDbGeneralModelOperations, accounts::HoprDbAccountOperations, channels::HoprDbChannelOperations,
        info::HoprDbInfoOperations,
    };
    use lazy_static::lazy_static;
    use mockall::mock;

    use super::*;

    lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))
        .expect("lazy static keypair should be valid");
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
            OffchainKeypair::from_secret(&privkey)
                .expect("lazy static keypair should be valid")
                .public()
                .into()
        ));
    }

    mock! {
        ChannelAct { }
        #[async_trait]
        impl ChannelActions for ChannelAct {
            async fn open_channel(&self, destination: Address, amount: HoprBalance) -> hopr_chain_actions::errors::Result<PendingAction>;
            async fn fund_channel(&self, channel_id: Hash, amount: HoprBalance) -> hopr_chain_actions::errors::Result<PendingAction>;
            async fn close_channel(
                &self,
                counterparty: Address,
                direction: ChannelDirection,
                redeem_before_close: bool,
            ) -> hopr_chain_actions::errors::Result<PendingAction>;
        }
    }

    async fn mock_channel(db: HoprDb, dst: Address, balance: HoprBalance) -> anyhow::Result<ChannelEntry> {
        let channel = ChannelEntry::new(
            PEERS[0].0,
            dst,
            balance,
            U256::zero(),
            ChannelStatus::Open,
            U256::zero(),
        );
        db.upsert_channel(None, channel).await?;

        Ok(channel)
    }

    async fn prepare_network(db: HoprDb, qualities: Vec<f64>) -> anyhow::Result<()> {
        assert_eq!(qualities.len(), PEERS.len() - 1, "invalid network setup");

        for (i, quality) in qualities.into_iter().enumerate() {
            let peer = &PEERS[i + 1].1;

            db.add_network_peer(peer, PeerOrigin::Initialization, vec![], 0.0, 10)
                .await?;

            let mut status = db.get_network_peer(peer).await?.expect("should be present");
            status.heartbeats_sent = 200;
            while status.get_average_quality() < quality {
                status.update_quality(quality);
            }
            db.update_network_peer(status).await?;
        }

        Ok(())
    }

    async fn init_db(db: HoprDb, node_balance: HoprBalance) -> anyhow::Result<()> {
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db.set_safe_hopr_balance(Some(tx), node_balance).await?;
                    db.set_safe_hopr_allowance(Some(tx), node_balance).await?;
                    for (chain_key, peer_id) in PEERS.iter() {
                        db.insert_account(
                            Some(tx),
                            AccountEntry {
                                public_key: OffchainPublicKey::from_peerid(peer_id).expect("should be valid PeerId"),
                                chain_addr: *chain_key,
                                entry_type: AccountType::NotAnnounced,
                                published_at: 1,
                            },
                        )
                        .await?;
                    }
                    Ok::<_, DbSqlError>(())
                })
            })
            .await?;

        Ok(())
    }

    fn mock_action_confirmation_closure(channel: ChannelEntry) -> ActionConfirmation {
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());
        ActionConfirmation {
            tx_hash: random_hash,
            event: Some(ChainEventType::ChannelClosureInitiated(channel)),
            action: Action::CloseChannel(channel, ChannelDirection::Outgoing),
        }
    }

    fn mock_action_confirmation_opening(address: Address, balance: HoprBalance) -> ActionConfirmation {
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

    #[test_log::test(tokio::test)]
    async fn test_promiscuous_strategy_tick_decisions() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;

        let qualities_that_alice_sees = vec![0.7, 0.9, 0.8, 0.98, 0.1, 0.3, 0.1, 0.2, 1.0];

        init_db(db.clone(), 1000.into()).await?;
        prepare_network(db.clone(), qualities_that_alice_sees).await?;

        mock_channel(db.clone(), PEERS[1].0, 10.into()).await?;
        mock_channel(db.clone(), PEERS[2].0, 10.into()).await?;
        let for_closing = mock_channel(db.clone(), PEERS[5].0, 10.into()).await?;

        // Peer 3 has an accepted pre-release version
        let status_3 = db
            .get_network_peer(&PEERS[3].1)
            .await?
            .context("peer should be present")?;
        db.update_network_peer(status_3).await?;

        // Peer 10 has an old node version
        let status_10 = db
            .get_network_peer(&PEERS[9].1)
            .await?
            .context("peer should be present")?;
        db.update_network_peer(status_10).await?;

        let strat_cfg = PromiscuousStrategyConfig {
            max_channels: Some(3),
            network_quality_open_threshold: 0.5,
            network_quality_close_threshold: 0.3,
            new_channel_stake: 10.into(),
            minimum_safe_balance: 50.into(),
            initial_delay: Duration::ZERO,
            ..Default::default()
        };

        // Situation:
        // - There are max 3 channels and also 3 are currently opened.
        // - Strategy will close channel to peer 5, because it has quality 0.1
        // - Because of the closure, this means there can be 1 additional channel opened:
        // - Strategy can open channel either to peer 3, 4 or 9 (with qualities 0.8, 0.98 and 1.0 respectively)
        // - It will ignore peer 9 even though it has the highest quality, but does not meet minimum node version
        // - It will prefer peer 4 because it has higher quality than node 3

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
            .withf(move |dst, b| PEERS[9].0.eq(dst) && new_stake.eq(b))
            .return_once(move |_, _| Ok(ok(mock_action_confirmation_opening(PEERS[4].0, new_stake)).boxed()));

        let strat = PromiscuousStrategy::new(strat_cfg.clone(), db, actions);

        tokio::time::sleep(Duration::from_millis(100)).await;

        strat.on_tick().await?;

        Ok(())
    }
}
