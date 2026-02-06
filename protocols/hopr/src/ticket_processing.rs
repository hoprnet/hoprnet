use std::sync::{Arc, atomic::AtomicU64};

use futures::StreamExt;
use hopr_api::{chain::ChainReadChannelOperations, db::HoprDbTicketOperations};
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
#[cfg(feature = "rayon")]
use hopr_parallelize::cpu::rayon::prelude::*;
use hopr_primitive_types::balance::HoprBalance;
use validator::ValidationError;

use crate::{
    HoprProtocolError, ResolvedAcknowledgement, TicketAcknowledgementError, TicketTracker,
    UnacknowledgedTicketProcessor,
};

/// Metrics for unacknowledged ticket cache diagnostics.
///
/// These help investigate "unknown ticket" acknowledgement failures by tracking
/// cache insertions, lookups, misses, and evictions.
mod metrics {
    #[cfg(any(not(feature = "prometheus"), test))]
    pub use noop::*;
    #[cfg(all(feature = "prometheus", not(test)))]
    pub use real::*;

    #[cfg(all(feature = "prometheus", not(test)))]
    mod real {
        lazy_static::lazy_static! {
            static ref UNACK_PEERS: hopr_metrics::SimpleGauge = hopr_metrics::SimpleGauge::new(
                "hopr_tickets_unack_peers_total",
                "Number of peers with unacknowledged tickets in cache",
            )
            .unwrap();
            static ref UNACK_TICKETS: hopr_metrics::SimpleGauge = hopr_metrics::SimpleGauge::new(
                "hopr_tickets_unack_tickets_total",
                "Total number of unacknowledged tickets across all peer caches",
            )
            .unwrap();
            static ref UNACK_INSERTIONS: hopr_metrics::SimpleCounter = hopr_metrics::SimpleCounter::new(
                "hopr_tickets_unack_insertions_total",
                "Total number of unacknowledged tickets inserted into cache",
            )
            .unwrap();
            static ref UNACK_LOOKUPS: hopr_metrics::SimpleCounter = hopr_metrics::SimpleCounter::new(
                "hopr_tickets_unack_lookups_total",
                "Total number of ticket acknowledgement lookups",
            )
            .unwrap();
            static ref UNACK_LOOKUP_MISSES: hopr_metrics::SimpleCounter = hopr_metrics::SimpleCounter::new(
                "hopr_tickets_unack_lookup_misses_total",
                "Total number of ticket lookup failures (unknown ticket)",
            )
            .unwrap();
            static ref UNACK_EVICTIONS: hopr_metrics::SimpleCounter = hopr_metrics::SimpleCounter::new(
                "hopr_tickets_unack_evictions_total",
                "Total number of unacknowledged tickets evicted from cache",
            )
            .unwrap();
            static ref UNACK_TICKETS_PER_PEER: hopr_metrics::MultiGauge = hopr_metrics::MultiGauge::new(
                "hopr_tickets_unack_tickets_per_peer",
                "Number of unacknowledged tickets per peer in cache",
                &["peer"],
            )
            .unwrap();
        }

        pub fn initialize() {
            lazy_static::initialize(&UNACK_PEERS);
            lazy_static::initialize(&UNACK_TICKETS);
            lazy_static::initialize(&UNACK_INSERTIONS);
            lazy_static::initialize(&UNACK_LOOKUPS);
            lazy_static::initialize(&UNACK_LOOKUP_MISSES);
            lazy_static::initialize(&UNACK_EVICTIONS);
            lazy_static::initialize(&UNACK_TICKETS_PER_PEER);
        }

        #[inline]
        pub fn set_unack_peers(count: u64) {
            UNACK_PEERS.set(count as f64);
        }

        #[inline]
        pub fn inc_unack_tickets() {
            UNACK_TICKETS.increment(1.0);
        }

        #[inline]
        pub fn dec_unack_tickets() {
            UNACK_TICKETS.decrement(1.0);
        }

        #[inline]
        pub fn inc_insertions() {
            UNACK_INSERTIONS.increment();
        }

        #[inline]
        pub fn inc_lookups() {
            UNACK_LOOKUPS.increment();
        }

        #[inline]
        pub fn inc_lookup_misses() {
            UNACK_LOOKUP_MISSES.increment();
        }

        #[inline]
        pub fn inc_evictions() {
            UNACK_EVICTIONS.increment();
        }

        #[inline]
        pub fn inc_unack_tickets_for_peer(peer: &str) {
            UNACK_TICKETS_PER_PEER.increment(&[peer], 1.0);
        }

        #[inline]
        pub fn dec_unack_tickets_for_peer(peer: &str) {
            UNACK_TICKETS_PER_PEER.decrement(&[peer], 1.0);
        }

        #[inline]
        pub fn reset_unack_tickets_for_peer(peer: &str) {
            UNACK_TICKETS_PER_PEER.set(&[peer], 0.0);
        }
    }

    #[cfg(any(not(feature = "prometheus"), test))]
    mod noop {
        #[inline]
        pub fn initialize() {}
        #[inline]
        pub fn set_unack_peers(_: u64) {}
        #[inline]
        pub fn inc_unack_tickets() {}
        #[inline]
        pub fn dec_unack_tickets() {}
        #[inline]
        pub fn inc_insertions() {}
        #[inline]
        pub fn inc_lookups() {}
        #[inline]
        pub fn inc_lookup_misses() {}
        #[inline]
        pub fn inc_evictions() {}
        #[inline]
        pub fn inc_unack_tickets_for_peer(_: &str) {}
        #[inline]
        pub fn dec_unack_tickets_for_peer(_: &str) {}
        #[inline]
        pub fn reset_unack_tickets_for_peer(_: &str) {}
    }
}

const MIN_UNACK_TICKET_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
fn validate_unack_ticket_timeout(timeout: &std::time::Duration) -> Result<(), ValidationError> {
    if timeout < &MIN_UNACK_TICKET_TIMEOUT {
        Err(ValidationError::new("unack_ticket_timeout too low"))
    } else {
        Ok(())
    }
}

const MIN_OUTGOING_INDEX_RETENTION: std::time::Duration = std::time::Duration::from_secs(10);

fn validate_outgoing_index_retention(retention: &std::time::Duration) -> Result<(), ValidationError> {
    if retention < &MIN_OUTGOING_INDEX_RETENTION {
        Err(ValidationError::new("outgoing_index_cache_retention too low"))
    } else {
        Ok(())
    }
}

fn default_outgoing_index_retention() -> std::time::Duration {
    std::time::Duration::from_secs(10)
}

fn default_unack_ticket_timeout() -> std::time::Duration {
    std::time::Duration::from_secs(30)
}

fn default_max_unack_tickets() -> usize {
    10_000_000
}

fn just_true() -> bool {
    true
}

/// Configuration for the HOPR ticket processor within the packet pipeline.

#[derive(Debug, Clone, Copy, smart_default::SmartDefault, PartialEq, validator::Validate)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(deny_unknown_fields)
)]
pub struct HoprTicketProcessorConfig {
    /// Time after which an unacknowledged ticket is considered stale and is removed from the cache.
    ///
    /// If the counterparty does not send an acknowledgement within this period, the ticket is lost forever.
    ///
    /// Default is 30 seconds.
    #[default(default_unack_ticket_timeout())]
    #[validate(custom(function = "validate_unack_ticket_timeout"))]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_unack_ticket_timeout", with = "humantime_serde")
    )]
    pub unack_ticket_timeout: std::time::Duration,
    /// Maximum number of unacknowledged tickets that can be stored in the cache at any given time.
    ///
    /// When more tickets are received, the oldest ones are discarded and lost forever.
    ///
    /// Default is 10 000 000.
    #[default(default_max_unack_tickets())]
    #[validate(range(min = 100))]
    #[cfg_attr(feature = "serde", serde(default = "default_max_unack_tickets"))]
    pub max_unack_tickets: usize,
    /// Period for which the outgoing ticket index is cached in memory for each channel.
    ///
    /// Default is 10 seconds.
    #[default(default_outgoing_index_retention())]
    #[validate(custom(function = "validate_outgoing_index_retention"))]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_outgoing_index_retention", with = "humantime_serde")
    )]
    pub outgoing_index_cache_retention: std::time::Duration,
    /// Indicates whether to use batch verification algorithm for acknowledgements.
    ///
    /// This has a positive performance impact on higher workloads.
    ///
    /// Default is true.
    #[default(just_true())]
    #[cfg_attr(feature = "serde", serde(default = "just_true"))]
    pub use_batch_verification: bool,
}

/// HOPR-specific implementation of [`UnacknowledgedTicketProcessor`] and [`TicketTracker`].
#[derive(Clone)]
pub struct HoprTicketProcessor<Chain, Db> {
    unacknowledged_tickets:
        moka::future::Cache<OffchainPublicKey, moka::future::Cache<HalfKeyChallenge, UnacknowledgedTicket>>,
    out_ticket_index: moka::future::Cache<(ChannelId, u32), Arc<AtomicU64>>,
    db: Db,
    chain_api: Chain,
    chain_key: ChainKeypair,
    channels_dst: Hash,
    cfg: HoprTicketProcessorConfig,
}

impl<Chain, Db> HoprTicketProcessor<Chain, Db> {
    /// Creates a new instance of the HOPR ticket processor.
    pub fn new(
        chain_api: Chain,
        db: Db,
        chain_key: ChainKeypair,
        channels_dst: Hash,
        cfg: HoprTicketProcessorConfig,
    ) -> Self {
        metrics::initialize();

        Self {
            out_ticket_index: moka::future::Cache::builder()
                .time_to_idle(cfg.outgoing_index_cache_retention)
                .max_capacity(10_000)
                .build(),
            unacknowledged_tickets: moka::future::Cache::builder()
                .time_to_idle(cfg.unack_ticket_timeout)
                .max_capacity(100_000)
                .build(),
            chain_api,
            db,
            chain_key,
            channels_dst,
            cfg,
        }
    }
}

impl<Chain, Db> HoprTicketProcessor<Chain, Db>
where
    Db: HoprDbTicketOperations + Clone + Send + 'static,
{
    /// Task that performs periodic synchronization of the outgoing ticket index cache
    /// to the underlying database.
    ///
    /// If this task is not started, the outgoing ticket indices will not survive a node
    /// restart and will result in invalid tickets received by the counterparty.
    pub fn outgoing_index_sync_task(
        &self,
        reg: futures::future::AbortRegistration,
    ) -> impl Future<Output = ()> + use<Db, Chain> {
        let index_save_stream = futures::stream::Abortable::new(
            futures_time::stream::interval(futures_time::time::Duration::from(
                self.cfg.outgoing_index_cache_retention.div_f32(2.0),
            )),
            reg,
        );

        let db = self.db.clone();
        let out_ticket_index = self.out_ticket_index.clone();

        index_save_stream
            .for_each(move |_| {
                let db = db.clone();
                let out_ticket_index = out_ticket_index.clone();
                async move {
                    // This iteration does not alter the popularity estimator of the cache
                    // and therefore still allows the unused entries to expire
                    for (channel_key, out_idx) in out_ticket_index.iter() {
                        if let Err(error) = db
                            .update_outgoing_ticket_index(
                                &channel_key.0,
                                channel_key.1,
                                out_idx.load(std::sync::atomic::Ordering::SeqCst),
                            )
                            .await
                        {
                            tracing::error!(%error, channel_id = %channel_key.0, epoch = channel_key.1, "failed to sync outgoing ticket index to db");
                        }
                    }
                    tracing::trace!("synced outgoing ticket indices to db");
                }
            })
    }
}

#[async_trait::async_trait]
impl<Chain, Db> UnacknowledgedTicketProcessor for HoprTicketProcessor<Chain, Db>
where
    Chain: ChainReadChannelOperations + Send + Sync,
    Db: Send + Sync,
{
    type Error = HoprProtocolError;

    #[tracing::instrument(skip(self, next_hop, challenge, ticket), level = "trace", fields(next_hop = next_hop.to_peerid_str()))]
    async fn insert_unacknowledged_ticket(
        &self,
        next_hop: &OffchainPublicKey,
        challenge: HalfKeyChallenge,
        ticket: UnacknowledgedTicket,
    ) -> Result<(), Self::Error> {
        tracing::trace!(%ticket, "received unacknowledged ticket");

        let peer_id = next_hop.to_peerid_str();
        let inner_cache = self
            .unacknowledged_tickets
            .get_with_by_ref(next_hop, async {
                let peer_id_for_eviction = peer_id.clone();
                moka::future::Cache::builder()
                    .time_to_live(self.cfg.unack_ticket_timeout)
                    .max_capacity(self.cfg.max_unack_tickets as u64)
                    .eviction_listener(move |_key, _value, cause| {
                        metrics::dec_unack_tickets();
                        metrics::dec_unack_tickets_for_peer(&peer_id_for_eviction);

                        // Only count Expired/Size removals as evictions (not Explicit or Replaced)
                        if matches!(
                            cause,
                            moka::notification::RemovalCause::Expired | moka::notification::RemovalCause::Size
                        ) {
                            metrics::inc_evictions();
                        }
                    })
                    .build()
            })
            .await;

        inner_cache.insert(challenge, ticket).await;

        metrics::inc_insertions();
        metrics::inc_unack_tickets();
        metrics::inc_unack_tickets_for_peer(&peer_id);
        metrics::set_unack_peers(self.unacknowledged_tickets.entry_count());

        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", fields(peer = peer.to_peerid_str()))]
    async fn acknowledge_tickets(
        &self,
        peer: OffchainPublicKey,
        acks: Vec<Acknowledgement>,
    ) -> Result<Vec<ResolvedAcknowledgement>, TicketAcknowledgementError<Self::Error>> {
        // Check if we're even expecting an acknowledgement from this peer:
        // We need to first do a check that does not update the popularity estimator of `peer` in this cache,
        // so we actually allow the entry to time out eventually. However, this comes at the cost
        // double-lookup.
        if !self.unacknowledged_tickets.contains_key(&peer) {
            tracing::trace!("not awaiting any acknowledgement from peer");
            return Err(TicketAcknowledgementError::UnexpectedAcknowledgement);
        }
        let Some(awaiting_ack_from_peer) = self.unacknowledged_tickets.get(&peer).await else {
            tracing::trace!("not awaiting any acknowledgement from peer");
            return Err(TicketAcknowledgementError::UnexpectedAcknowledgement);
        };

        // Verify all the acknowledgements and compute challenges from half-keys
        let use_batch_verify = self.cfg.use_batch_verification;
        let half_keys_challenges = hopr_parallelize::cpu::spawn_fifo_blocking(
            move || {
                if use_batch_verify {
                    // Uses regular verifications for small batches but switches to a more effective
                    // batch verification algorithm for larger ones.
                    let acks = Acknowledgement::verify_batch(acks.into_iter().map(|ack| (peer, ack)));

                    #[cfg(feature = "rayon")]
                    let iter = acks.into_par_iter();

                    #[cfg(not(feature = "rayon"))]
                    let iter = acks.into_iter();

                    iter.map(|verified| {
                        verified.and_then(|verified| {
                            Ok((*verified.ack_key_share(), verified.ack_key_share().to_challenge()?))
                        })
                    })
                    .filter_map(|res| {
                        res.inspect_err(|error| tracing::error!(%error, "failed to process acknowledgement"))
                            .ok()
                    })
                    .collect::<Vec<_>>()
                } else {
                    #[cfg(feature = "rayon")]
                    let iter = acks.into_par_iter();

                    #[cfg(not(feature = "rayon"))]
                    let iter = acks.into_iter();

                    iter.map(|ack| {
                        ack.verify(&peer).and_then(|verified| {
                            Ok((*verified.ack_key_share(), verified.ack_key_share().to_challenge()?))
                        })
                    })
                    .filter_map(|res| {
                        res.inspect_err(|error| tracing::error!(%error, "failed to process acknowledgement"))
                            .ok()
                    })
                    .collect::<Vec<_>>()
                }
            },
            "ack_verify",
        )
        .await
        .map_err(|e| TicketAcknowledgementError::Inner(HoprProtocolError::from(e)))?;

        // Find all the tickets that we're awaiting acknowledgement for
        let mut unack_tickets = Vec::with_capacity(half_keys_challenges.len());
        for (half_key, challenge) in half_keys_challenges {
            metrics::inc_lookups();

            let Some(unack_ticket) = awaiting_ack_from_peer.remove(&challenge).await else {
                tracing::debug!(%challenge, "received acknowledgement for unknown ticket");
                metrics::inc_lookup_misses();
                continue;
            };

            let issuer_channel = match self
                .chain_api
                .channel_by_parties(unack_ticket.ticket.verified_issuer(), self.chain_key.as_ref())
                .await
            {
                Ok(Some(channel)) => {
                    if channel.channel_epoch != unack_ticket.verified_ticket().channel_epoch {
                        tracing::error!(%unack_ticket, "received acknowledgement for ticket issued in a different epoch");
                        continue;
                    }
                    channel
                }
                Ok(None) => {
                    tracing::error!(%unack_ticket, "received acknowledgement for ticket issued for unknown channel");
                    continue;
                }
                Err(error) => {
                    tracing::error!(%error, %unack_ticket, "failed to resolve channel for unacknowledged ticket");
                    continue;
                }
            };

            unack_tickets.push((issuer_channel, half_key, unack_ticket));
        }

        let domain_separator = self.channels_dst;
        let chain_key = self.chain_key.clone();
        Ok(hopr_parallelize::cpu::spawn_fifo_blocking(
            move || {
                #[cfg(feature = "rayon")]
                let iter = unack_tickets.into_par_iter();

                #[cfg(not(feature = "rayon"))]
                let iter = unack_tickets.into_iter();

                iter.filter_map(|(channel, half_key, unack_ticket)| {
                    // This explicitly checks whether the acknowledgement
                    // solves the challenge on the ticket.
                    // It must be done before we check that the ticket is winning,
                    // which is a lengthy operation and should not be done for
                    // bogus unacknowledged tickets
                    let Ok(ack_ticket) = unack_ticket.acknowledge(&half_key) else {
                        tracing::error!(%unack_ticket, "failed to acknowledge ticket");
                        return None;
                    };

                    // This operation checks if the ticket is winning, and if it is, it
                    // turns it into a redeemable ticket.
                    match ack_ticket.into_redeemable(&chain_key, &domain_separator) {
                        Ok(redeemable) => {
                            tracing::debug!(%channel, "found winning ticket");
                            Some(ResolvedAcknowledgement::RelayingWin(Box::new(redeemable)))
                        }
                        Err(CoreTypesError::TicketNotWinning) => {
                            tracing::trace!(%channel, "found losing ticket");
                            Some(ResolvedAcknowledgement::RelayingLoss(*channel.get_id()))
                        }
                        Err(error) => {
                            tracing::error!(%error, %channel, "error when acknowledging ticket");
                            Some(ResolvedAcknowledgement::RelayingLoss(*channel.get_id()))
                        }
                    }
                })
                .collect::<Vec<_>>()
            },
            "ticket_redeem",
        )
        .await
        .map_err(|e| TicketAcknowledgementError::Inner(HoprProtocolError::from(e)))?)
    }
}

#[async_trait::async_trait]
impl<Chain, Db> TicketTracker for HoprTicketProcessor<Chain, Db>
where
    Chain: Send + Sync,
    Db: HoprDbTicketOperations + Clone + Send + Sync + 'static,
{
    type Error = Arc<Db::Error>;

    async fn next_outgoing_ticket_index(&self, channel_id: &ChannelId, epoch: u32) -> Result<u64, Self::Error> {
        let channel_id = *channel_id;
        self.out_ticket_index
            .try_get_with((channel_id, epoch), async {
                self.db
                    .get_or_create_outgoing_ticket_index(&channel_id, epoch)
                    .await
                    .map(|maybe_idx| Arc::new(AtomicU64::new(maybe_idx.unwrap_or_default())))
            })
            .await
            .map(|idx| idx.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
    }

    async fn incoming_channel_unrealized_balance(
        &self,
        channel_id: &ChannelId,
        epoch: u32,
    ) -> Result<HoprBalance, Self::Error> {
        // This value cannot be cached here and must be cached in the DB
        // because the cache invalidation logic can be only done from within the DB.
        self.db.get_tickets_value(channel_id, epoch).await.map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use hopr_crypto_random::Randomizable;

    use super::*;
    use crate::utils::*;

    #[tokio::test]
    async fn ticket_processor_should_acknowledge_previously_inserted_tickets() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;

        let node = create_node(1, &blokli_client).await?;

        let ticket_processor = HoprTicketProcessor::new(
            node.chain_api.clone(),
            node.node_db.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprTicketProcessorConfig::default(),
        );

        const NUM_TICKETS: usize = 5;

        let mut acks = Vec::with_capacity(5);
        for index in 0..NUM_TICKETS {
            let own_share = HalfKey::random();
            let ack_share = HalfKey::random();
            let challenge = Challenge::from_own_share_and_half_key(&own_share.to_challenge()?, &ack_share)?;

            let unack_ticket = TicketBuilder::default()
                .counterparty(&PEERS[1].0)
                .index(index as u64)
                .channel_epoch(1)
                .amount(10_u32)
                .challenge(challenge)
                .build_signed(&PEERS[0].0, &Hash::default())?
                .into_unacknowledged(own_share);

            ticket_processor
                .insert_unacknowledged_ticket(PEERS[2].1.public(), ack_share.to_challenge()?, unack_ticket)
                .await?;

            acks.push(VerifiedAcknowledgement::new(ack_share, &PEERS[2].1).leak());
        }

        let resolutions = ticket_processor.acknowledge_tickets(*PEERS[2].1.public(), acks).await?;
        assert_eq!(NUM_TICKETS, resolutions.len());
        assert!(
            resolutions
                .iter()
                .all(|res| matches!(res, ResolvedAcknowledgement::RelayingWin(_)))
        );

        Ok(())
    }

    #[tokio::test]
    async fn ticket_processor_should_reject_acknowledgements_from_unexpected_sender() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;

        let node = create_node(1, &blokli_client).await?;

        let ticket_processor = HoprTicketProcessor::new(
            node.chain_api.clone(),
            node.node_db.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprTicketProcessorConfig::default(),
        );

        const NUM_ACKS: usize = 5;

        let mut acks = Vec::with_capacity(5);
        for _ in 0..NUM_ACKS {
            let ack_share = HalfKey::random();
            acks.push(VerifiedAcknowledgement::new(ack_share, &PEERS[2].1).leak());
        }

        assert!(matches!(
            ticket_processor.acknowledge_tickets(*PEERS[2].1.public(), acks).await,
            Err(TicketAcknowledgementError::UnexpectedAcknowledgement)
        ));

        Ok(())
    }
}
