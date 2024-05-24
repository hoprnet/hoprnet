use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use futures::stream::BoxStream;
use libp2p_identity::PeerId;
use multiaddr::Multiaddr;
use tracing::warn;

use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_primitive_types::prelude::*;

use crate::errors::Result;

/// Actual origin.
///
/// First occurence of the peer in the network mechanism.
#[derive(Debug, Copy, Clone, PartialEq, Eq, strum::Display, num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum PeerOrigin {
    #[strum(to_string = "node initialization")]
    Initialization = 0,
    #[strum(to_string = "network registry")]
    NetworkRegistry = 1,
    #[strum(to_string = "incoming connection")]
    IncomingConnection = 2,
    #[strum(to_string = "outgoing connection attempt")]
    OutgoingConnection = 3,
    #[strum(to_string = "strategy monitors existing channel")]
    StrategyExistingChannel = 4,
    #[strum(to_string = "strategy considers opening a channel")]
    StrategyConsideringChannel = 5,
    #[strum(to_string = "strategy decided to open new channel")]
    StrategyNewChannel = 6,
    #[strum(to_string = "manual ping")]
    ManualPing = 7,
    #[strum(to_string = "testing")]
    Testing = 8,
}

/// Statistical observation related to peers in the network. statistics on all peer entries stored
/// in the [crate::network::Network] object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stats {
    /// Number of good quality public nodes.
    pub good_quality_public: u32,
    /// Number of bad quality public nodes.
    pub bad_quality_public: u32,
    /// Number of good quality nodes non-public nodes.
    pub good_quality_non_public: u32,
    /// Number of bad quality nodes non-public nodes.
    pub bad_quality_non_public: u32,
}

// #[cfg(all(feature = "prometheus", not(test)))]
impl Stats {
    /// Returns count of all peers.
    pub fn all_count(&self) -> usize {
        self.good_quality_public as usize
            + self.bad_quality_public as usize
            + self.good_quality_non_public as usize
            + self.bad_quality_non_public as usize
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct PeerSelector {
    /// Lower and upper bounds (both inclusive) on last seen timestamp.
    pub last_seen: (Option<SystemTime>, Option<SystemTime>),
    /// Lower and upper bounds (both inclusive) on peer quality.
    pub quality: (Option<f64>, Option<f64>),
}

impl PeerSelector {
    pub fn with_last_seen_gte(mut self, lower_bound: SystemTime) -> Self {
        self.last_seen.0 = Some(lower_bound);
        self
    }

    pub fn with_last_seen_lte(mut self, upper_bound: SystemTime) -> Self {
        self.last_seen.1 = Some(upper_bound);
        self
    }

    pub fn with_quality_gte(mut self, lower_bound: f64) -> Self {
        self.quality.0 = Some(lower_bound);
        self
    }

    pub fn with_quality_lte(mut self, upper_bound: f64) -> Self {
        self.quality.1 = Some(upper_bound);
        self
    }
}

/// Status of the peer as recorded by the [Network].
#[derive(Debug, Clone, PartialEq)]
pub struct PeerStatus {
    pub id: (OffchainPublicKey, PeerId),
    pub origin: PeerOrigin,
    pub is_public: bool,
    pub last_seen: SystemTime,
    pub last_seen_latency: Duration,
    pub heartbeats_sent: u64,
    pub heartbeats_succeeded: u64,
    pub backoff: f64,
    pub ignored: Option<SystemTime>,
    pub peer_version: Option<String>,
    pub multiaddresses: Vec<Multiaddr>,
    // Should be public(crate) but the separation through traits does not allow direct SQL ORM serde
    pub quality: f64,
    // Should be public(crate) but the separation through traits does not allow direct SQL ORM serde
    pub quality_avg: SingleSumSMA<f64>,
}

impl PeerStatus {
    pub fn new(id: PeerId, origin: PeerOrigin, backoff: f64, quality_window: u32) -> PeerStatus {
        PeerStatus {
            id: (OffchainPublicKey::try_from(&id).expect("invalid peer id given"), id),
            origin,
            is_public: true,
            heartbeats_sent: 0,
            heartbeats_succeeded: 0,
            last_seen: SystemTime::UNIX_EPOCH,
            last_seen_latency: Duration::default(),
            ignored: None,
            backoff,
            quality: 0.0,
            peer_version: None,
            quality_avg: SingleSumSMA::new(quality_window as usize),
            multiaddresses: vec![],
        }
    }

    // Update both the immediate last quality and the average windowed quality
    pub fn update_quality(&mut self, new_value: f64) {
        if (0.0f64..=1.0f64).contains(&new_value) {
            self.quality = new_value;
            self.quality_avg.push(new_value);
        } else {
            warn!("Quality failed to update with value outside the [0,1] range")
        }
    }

    /// Gets the average quality of this peer
    pub fn get_average_quality(&self) -> f64 {
        self.quality_avg.average().unwrap_or_default()
    }

    /// Gets the immediate node quality
    pub fn get_quality(&self) -> f64 {
        self.quality
    }
}

impl std::fmt::Display for PeerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Entry: [id={}, origin={}, last seen on={:?}, quality={}, heartbeats sent={}, heartbeats succeeded={}, backoff={}]",
            self.id.1, self.origin, self.last_seen, self.quality, self.heartbeats_sent, self.heartbeats_succeeded, self.backoff)
    }
}

#[async_trait]
pub trait HoprDbPeersOperations {
    /// Adds a peer to the backend.
    ///
    /// Should fail if the given peer id already exists in the store.
    async fn add_network_peer(
        &self,
        peer: &PeerId,
        origin: PeerOrigin,
        mas: Vec<Multiaddr>,
        backoff: f64,
        quality_window: u32,
    ) -> Result<()>;

    /// Removes the peer from the backend.
    ///
    /// Should fail if the given peer id does not exist.
    async fn remove_network_peer(&self, peer: &PeerId) -> Result<()>;

    /// Updates stored information about the peer.
    /// Should fail if the peer does not exist in the store.
    async fn update_network_peer(&self, new_status: PeerStatus) -> Result<()>;

    /// Gets stored information about the peer.
    ///
    /// Should return `None` if such peer does not exist in the store.
    async fn get_network_peer(&self, peer: &PeerId) -> Result<Option<PeerStatus>>;

    /// Returns a stream of all stored peers, optionally matching the given [SimpleExpr] filter.
    ///
    /// The `sort_last_seen_asc` indicates whether the results should be sorted in ascending
    /// or descending order of the `last_seen` field.
    async fn get_network_peers<'a>(
        &'a self,
        selector: PeerSelector,
        sort_last_seen_asc: bool,
    ) -> Result<BoxStream<'a, PeerStatus>>;

    /// Returns the [statistics](Stats) on the stored peers.
    async fn network_peer_stats(&self, quality_threshold: f64) -> Result<Stats>;
}
