//! ## One hop connection strategy
//! This strategy is designed to maintain 1-hop pathes to certain destination peers,
//! where there is at least one relay node that is not part of the destination peer set.
//!
//! There should be outgoing channels to the destination peers.
use async_trait::async_trait;
use chain_actions::channels::ChannelActions;
use futures::StreamExt;
use hopr_db_sql::api::peers::PeerSelector;
use hopr_db_sql::HoprDbAllOperations;
use hopr_internal_types::channels::{ChannelDirection, ChannelEntry};
use hopr_primitive_types::prelude::*;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, Sub};
use std::str::FromStr;
use tracing::{debug, error, info};
use validator::Validate;

use crate::errors::{self, Result};
use crate::strategy::SingularStrategy;
use crate::types::ChannelDecision;
use crate::Strategy;

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::SimpleCounter;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_OPENS: SimpleCounter =
        SimpleCounter::new("hopr_strategy_one_hop_connection_opened_channels_count", "Count of open channel decisions").unwrap();
    static ref METRIC_COUNT_CLOSURES: SimpleCounter =
        SimpleCounter::new("hopr_strategy_one_hop_connection_closed_channels_count", "Count of close channel decisions").unwrap();
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct OneHopConnectionStrategyConfig {
    /// Ethereum addresses of the destination peers.
    #[validate(length(min = 1))]
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub destination_peers: Vec<Address>,

    /// A quality threshold between 0 and 1 used to determine whether the strategy should open channel with the peer.
    #[validate(range(min = 0_f64, max = 1.0_f64))]
    #[default = 0.5]
    pub network_quality_threshold: f64,

    /// Specifies a minimum version (in semver syntax) of the peer the strategy should open a channel to.
    ///
    /// Default is ">=2.0.0"
    #[serde_as(as = "DisplayFromStr")]
    #[default(">=2.0.0".parse().expect("should be valid default version"))]
    pub minimum_peer_version: semver::VersionReq,

    /// Minimum stake that a channel's balance must not go below.
    ///
    /// Default is 1 HOPR
    #[serde_as(as = "DisplayFromStr")]
    #[default(Balance::new_from_str("1000000000000000000", BalanceType::HOPR))]
    pub min_stake_threshold: Balance,

    /// Funding amount.
    ///
    /// Defaults to 10 HOPR.
    #[serde_as(as = "DisplayFromStr")]
    #[default(Balance::new_from_str("10000000000000000000", BalanceType::HOPR))]
    pub funding_amount: Balance,
}

/// This strategy is designed to maintain 1-hop pathes to certain destination peers,
/// where there is at least one relay node that is not part of the destination peer set.
///
/// There should be outgoing channels to the destination peers.
///
/// When there's no relay node that is not part of the destination peer set,
/// the strategy should select a relay node and open an outgoing channel to it.
/// Selection of this new relay peer should prioritize those that have incoming channels
/// with the destination peers.
///
/// It choses channels to close among the channels that cannot be used for reaching
/// the destination peers with 1-hop.
/// The strategy only outgoing channels where the target has low network quality.
pub struct OneHopConnectionStrategy<A, Db>
where
    Db: HoprDbAllOperations + Clone,
    A: ChannelActions + Send + Sync,
{
    chain_actions: A,
    db: Db,
    cfg: OneHopConnectionStrategyConfig,
}

impl<A, Db> OneHopConnectionStrategy<A, Db>
where
    Db: HoprDbAllOperations + Clone,
    A: ChannelActions + Send + Sync,
{
    pub fn new(cfg: OneHopConnectionStrategyConfig, db: Db, chain_actions: A) -> Self {
        Self { chain_actions, db, cfg }
    }

    async fn get_peers_with_quality(&self) -> Result<HashMap<Address, f64>> {
        Ok(self
            .db
            .get_network_peers(PeerSelector::default(), false)
            .await?
            .filter_map(|status| {
                let version = status.peer_version.clone().and_then(|v| {
                    semver::Version::from_str(&v)
                        .ok() // Workaround for https://github.com/dtolnay/semver/issues/315
                        .map(|v| Version::new(v.major, v.minor, v.patch))
                });

                async move {
                    if let Some(version) = version {
                        if self.cfg.minimum_peer_version.matches(&version) {
                            if let Ok(addr) = self
                                .db
                                .resolve_chain_key(&status.id.0)
                                .await
                                .and_then(|addr| addr.ok_or(hopr_db_sql::api::errors::DbError::MissingAccount))
                            {
                                return Some((addr, status.get_average_quality()));
                            } else {
                                error!(address = %status.id.1, "could not find on-chain address");
                            }
                        } else {
                            debug!(peer = %status.id.1, ?version, "version of peer does not match the expectation");
                        }
                    } else {
                        error!(peer = %status.id.1, "cannot get version");
                    }
                    None
                }
            })
            .collect()
            .await)
    }

    async fn collect_tick_decision(&self) -> Result<ChannelDecision> {
        let mut tick_decision = ChannelDecision::default();
        let mut remaining_balance = self
            .db
            .get_safe_hopr_balance(None)
            .await
            .map_err(hopr_db_sql::api::errors::DbError::from)?;

        // Get all the outgoing channels
        let outgoing_channels = self
            .db
            .get_outgoing_channels(None)
            .await
            .map_err(hopr_db_sql::api::errors::DbError::from)?
            .into_iter();

        // Get all the peers with network quality above the threshold
        let peers_with_quality = self.get_peers_with_quality().await?;

        // Outgoing channels pointing to peers outside of destination peers with bad quality will be closed
        for channel in outgoing_channels.clone() {
            let quality = peers_with_quality.get(&channel.destination).unwrap_or(&0_f64);
            if *quality < self.cfg.network_quality_threshold
                && !tick_decision.will_channel_be_closed(&channel.destination)
                && !&self.cfg.destination_peers.contains(&channel.destination)
            {
                tick_decision.add_to_close(channel);
                remaining_balance = remaining_balance.add(&channel.balance);
                debug!(%channel.destination, "promoted for channel closing due to bad network quality");
            }
        }

        // Loop through all the destination peers
        for destination_peer in &self.cfg.destination_peers {
            // check if there's a direct channel to the destination peer, if not open it
            let has_direct_channel = outgoing_channels.clone().any(|channel| {
                channel.destination == *destination_peer && channel.balance.ge(&self.cfg.min_stake_threshold)
            });
            if !has_direct_channel && !tick_decision.will_address_be_opened(destination_peer) {
                tick_decision.add_to_open(destination_peer.clone(), self.cfg.funding_amount.clone());
                remaining_balance = remaining_balance.sub(&self.cfg.funding_amount);
                debug!(%destination_peer, "promoted for channel opening");
            }

            // Get outgoing channels from the destination peer, where the dst node
            // of those outgoing channels, which is on the destination peer list,
            // are candidates for relay nodes.
            let other_outgoing_channels_from_destination_peer = self
                .db
                .get_channels_via(None, ChannelDirection::Outgoing, destination_peer)
                .await
                .map_err(hopr_db_sql::api::errors::DbError::from)?
                .into_iter()
                .filter(|channel| !&self.cfg.destination_peers.contains(&channel.destination))
                .collect::<Vec<ChannelEntry>>();

            let mut should_find_new_relay_node = false;
            // when the destination_peer does not have any outgoing channels to nodes outside of the destination peer list,
            // open a channel to a peer outside of the destination peer list, prioritizing those that have good network quality
            if !other_outgoing_channels_from_destination_peer.is_empty() {
                // From all the outgoing channels from the destination peer that points to a relay node outside of the destination peer list,
                // get the subset of relay nodes that that we have an outgoing channel to, and has a good network quality.
                let relay_nodes = other_outgoing_channels_from_destination_peer
                    .iter()
                    .filter_map(|channel| {
                        let relay_node = channel.destination;
                        // Check if we have an outgoing channel to the relay node
                        if outgoing_channels
                            .clone()
                            .any(|outgoing_c| outgoing_c.destination == relay_node)
                        {
                            let peers_with_quality_cloned = peers_with_quality.clone();
                            let relay_node_quality = {
                                let quality = peers_with_quality_cloned.get(&relay_node);
                                quality.unwrap_or(&0_f64)
                            };
                            if *relay_node_quality >= self.cfg.network_quality_threshold {
                                Some((relay_node, channel.balance))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<(Address, Balance)>>();
                if relay_nodes.is_empty() {
                    should_find_new_relay_node = true;
                } else {
                    should_find_new_relay_node = false;

                    // Among those relay nodes with good network quality, top up channels to relay nodes with low balance.
                    for (relay_node, outgoing_channel_to_relay_node_balance) in relay_nodes {
                        // check if the channel needs to be topped up
                        if outgoing_channel_to_relay_node_balance.lt(&self.cfg.min_stake_threshold) {
                            if !tick_decision.will_address_be_opened(&relay_node) {
                                tick_decision.add_to_open(relay_node, self.cfg.funding_amount);
                                remaining_balance = remaining_balance.sub(&self.cfg.funding_amount);
                                debug!(%relay_node, "relay node will be topped up");
                            } else {
                                debug!(%relay_node, "relay node is already promoted for topping up");
                            }
                        } else {
                            // Keep existing relay nodes alive
                            debug!(%relay_node, "relay node is alive and has enough balance");
                        }
                    }
                }
            } else {
                should_find_new_relay_node = true;
            }

            // open a new channel to a random high quality peer, so that the destination peer has more chance of reaching this relay node
            // and thus open a channel. Note that this assumption can be abused by malicious nodes (destination peer) to drain the node's balance.
            // if the destination peer does not run this strategy and thus never opens a channel.
            // This is a trade-off between security and availability. Please only use this strategy with trusted peers.
            if should_find_new_relay_node {
                // sort peers by quality, with the highest quality first
                let mut peers_with_quality_vec: Vec<_> = peers_with_quality.clone().into_iter().collect();
                peers_with_quality_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

                // Get all the relay nodes that are not part of the destination peer list
                let outgoing_channels_vec: Vec<_> = outgoing_channels.clone().collect();
                let new_relay_node = peers_with_quality
                    .clone()
                    .into_iter()
                    .filter(|&(relay_node, quality)| {
                        !outgoing_channels_vec.iter().any(|c| c.destination == relay_node)
                            && quality >= self.cfg.network_quality_threshold
                    })
                    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                    .ok_or(errors::StrategyError::Other("Cannot find a new node".into()))?;

                // Open a channel to a relay node that we don't have an outgoing channel to
                if !tick_decision.will_address_be_opened(&new_relay_node.0) {
                    tick_decision.add_to_open(new_relay_node.0.clone(), self.cfg.funding_amount.clone());
                    remaining_balance = remaining_balance.sub(&self.cfg.funding_amount);
                    debug!(peer = %new_relay_node.0, "opened channel to relay node");
                } else {
                    debug!(peer = %new_relay_node.0, "relay node is already promoted for channel opening");
                }
            }
        }

        Ok(tick_decision)
    }
}

impl<A, Db> Display for OneHopConnectionStrategy<A, Db>
where
    Db: HoprDbAllOperations + Clone,
    A: ChannelActions + Send + Sync,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::OneHopConnection(self.cfg.clone()))
    }
}

#[async_trait]
impl<A, Db> SingularStrategy for OneHopConnectionStrategy<A, Db>
where
    Db: HoprDbAllOperations + Clone + Send + Sync + std::fmt::Debug + 'static,
    A: ChannelActions + Send + Sync,
{
    async fn on_tick(&self) -> crate::errors::Result<()> {
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
                    error!(error = %e, "error while closing channel");
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
                    error!(error = %e, channel = %channel_to_open.0, "error while issuing channel opening");
                }
            }
        }

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            METRIC_COUNT_OPENS.increment_by(tick_decision.get_to_open().len() as u64);
            METRIC_COUNT_CLOSURES.increment_by(tick_decision.get_to_close().len() as u64);
        }

        info!(%tick_decision, "on tick executed");
        Ok(())
    }
}
