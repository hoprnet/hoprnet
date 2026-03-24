use hopr_api::{
    chain::ChainReadChannelOperations,
    types::{crypto::prelude::*, internal::prelude::*},
};
#[cfg(feature = "rayon")]
use hopr_parallelize::cpu::rayon::prelude::*;
use validator::ValidationError;

use crate::{HoprProtocolError, ResolvedAcknowledgement, TicketAcknowledgementError, UnacknowledgedTicketProcessor};

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
        /// Whether per-peer metrics are enabled (disabled by default to avoid cardinality explosion).
        static ref PER_PEER_ENABLED: bool = std::env::var("HOPR_METRICS_UNACK_PER_PEER")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

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
            "Total number of unacknowledged tickets evicted from cache due to TTL or capacity limits",
        )
        .unwrap();
        static ref UNACK_PEER_EVICTIONS: hopr_metrics::SimpleCounter = hopr_metrics::SimpleCounter::new(
            "hopr_tickets_unack_peer_evictions_total",
            "Total number of peer caches evicted from the outer unacknowledged ticket cache",
        )
        .unwrap();
        static ref UNACK_TICKETS_PER_PEER: hopr_metrics::MultiGauge = hopr_metrics::MultiGauge::new(
            "hopr_tickets_unack_tickets_per_peer",
            "Number of unacknowledged tickets per peer in cache (enable with HOPR_METRICS_UNACK_PER_PEER=1)",
            &["peer"],
        )
        .unwrap();
}

const MIN_UNACK_TICKET_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
fn validate_unack_ticket_timeout(timeout: &std::time::Duration) -> Result<(), ValidationError> {
    if timeout < &MIN_UNACK_TICKET_TIMEOUT {
        Err(ValidationError::new("unack_ticket_timeout too low"))
    } else {
        Ok(())
    }
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

/// Configuration for the [`HoprUnacknowledgedTicketProcessor`].

#[derive(Debug, Clone, Copy, smart_default::SmartDefault, PartialEq, validator::Validate)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(deny_unknown_fields)
)]
pub struct HoprUnacknowledgedTicketProcessorConfig {
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
    /// Indicates whether to use batch verification algorithm for acknowledgements.
    ///
    /// This has a positive performance impact on higher workloads.
    ///
    /// Default is true.
    #[default(just_true())]
    #[cfg_attr(feature = "serde", serde(default = "just_true"))]
    pub use_batch_verification: bool,
}

/// HOPR-specific implementation of [`UnacknowledgedTicketProcessor`].
#[derive(Clone)]
pub struct HoprUnacknowledgedTicketProcessor<Chain> {
    unacknowledged_tickets:
        moka::future::Cache<OffchainPublicKey, moka::future::Cache<HalfKeyChallenge, UnacknowledgedTicket>>,
    chain_api: Chain,
    chain_key: ChainKeypair,
    channels_dst: Hash,
    cfg: HoprUnacknowledgedTicketProcessorConfig,
}

impl<Chain> HoprUnacknowledgedTicketProcessor<Chain> {
    /// Creates a new instance of the HOPR unacknowledged ticket processor.
    pub fn new(
        chain_api: Chain,
        chain_key: ChainKeypair,
        channels_dst: Hash,
        cfg: HoprUnacknowledgedTicketProcessorConfig,
    ) -> Self {
        #[cfg(all(feature = "telemetry", not(test)))]
        {
            lazy_static::initialize(&PER_PEER_ENABLED);
            lazy_static::initialize(&UNACK_PEERS);
            lazy_static::initialize(&UNACK_TICKETS);
            lazy_static::initialize(&UNACK_INSERTIONS);
            lazy_static::initialize(&UNACK_LOOKUPS);
            lazy_static::initialize(&UNACK_LOOKUP_MISSES);
            lazy_static::initialize(&UNACK_EVICTIONS);
            lazy_static::initialize(&UNACK_PEER_EVICTIONS);
            if *PER_PEER_ENABLED {
                lazy_static::initialize(&UNACK_TICKETS_PER_PEER);
            }
        }

        Self {
            unacknowledged_tickets: moka::future::Cache::builder()
                .time_to_idle(cfg.unack_ticket_timeout)
                .max_capacity(100_000)
                .async_eviction_listener(
                    |_key,
                     value: moka::future::Cache<HalfKeyChallenge, UnacknowledgedTicket>,
                     cause|
                     -> moka::notification::ListenerFuture {
                        Box::pin(async move {
                            if !matches!(cause, moka::notification::RemovalCause::Replaced) {
                                #[cfg(all(feature = "telemetry", not(test)))]
                                UNACK_PEERS.decrement(1.0);

                                #[cfg(all(feature = "telemetry", not(test)))]
                                if matches!(
                                    cause,
                                    moka::notification::RemovalCause::Expired | moka::notification::RemovalCause::Size
                                ) {
                                    UNACK_PEER_EVICTIONS.increment();
                                }
                            }
                            // Explicitly invalidate all inner cache entries so their eviction
                            // listeners fire (decrementing UNACK_TICKETS and per-peer metrics).
                            // Without this, dropping the inner cache silently leaks those metrics.
                            value.invalidate_all();
                            value.run_pending_tasks().await;
                        })
                    },
                )
                .build(),
            chain_api,
            chain_key,
            channels_dst,
            cfg,
        }
    }
}

#[async_trait::async_trait]
impl<Chain> UnacknowledgedTicketProcessor for HoprUnacknowledgedTicketProcessor<Chain>
where
    Chain: ChainReadChannelOperations + Send + Sync,
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

        let inner_cache = self
            .unacknowledged_tickets
            .get_with_by_ref(next_hop, async {
                #[cfg(all(feature = "telemetry", not(test)))]
                UNACK_PEERS.increment(1.0);

                #[cfg(all(feature = "telemetry", not(test)))]
                let next_hop = *next_hop;
                moka::future::Cache::builder()
                    .time_to_live(self.cfg.unack_ticket_timeout)
                    .max_capacity(self.cfg.max_unack_tickets as u64)
                    .eviction_listener(move |_key, _value, cause| {
                        #[cfg(all(feature = "telemetry", not(test)))]
                        {
                            let peer_id_for_eviction = next_hop.to_peerid_str();
                            UNACK_TICKETS.decrement(1.0);
                            UNACK_TICKETS_PER_PEER.decrement(
                                &[if *PER_PEER_ENABLED {
                                    peer_id_for_eviction.as_str()
                                } else {
                                    "redacted"
                                }],
                                1.0,
                            );
                        }

                        // Only count Expired/Size removals as evictions (not Explicit or Replaced)
                        if matches!(
                            cause,
                            moka::notification::RemovalCause::Expired | moka::notification::RemovalCause::Size
                        ) {
                            #[cfg(all(feature = "telemetry", not(test)))]
                            UNACK_EVICTIONS.increment();
                        }
                    })
                    .build()
            })
            .await;

        inner_cache.insert(challenge, ticket).await;

        #[cfg(all(feature = "telemetry", not(test)))]
        {
            let peer_id = next_hop.to_peerid_str();
            UNACK_INSERTIONS.increment();
            UNACK_TICKETS.increment(1.0);
            UNACK_TICKETS_PER_PEER.increment(
                &[if *PER_PEER_ENABLED {
                    peer_id.as_str()
                } else {
                    "redacted"
                }],
                1.0,
            );
        }

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
            #[cfg(all(feature = "telemetry", not(test)))]
            UNACK_LOOKUPS.increment();

            let Some(unack_ticket) = awaiting_ack_from_peer.remove(&challenge).await else {
                #[cfg(all(feature = "telemetry", not(test)))]
                UNACK_LOOKUP_MISSES.increment();

                tracing::trace!(%challenge, "received acknowledgement for unknown ticket");
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
                            tracing::trace!(%channel, "found winning ticket");
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
            "ticket_into_redeemable",
        )
        .await
        .map_err(|e| TicketAcknowledgementError::Inner(HoprProtocolError::from(e)))?)
    }
}

#[cfg(test)]
mod tests {
    use hopr_api::types::crypto_random::Randomizable;

    use super::*;
    use crate::utils::*;

    #[tokio::test]
    async fn ticket_processor_should_acknowledge_previously_inserted_tickets() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;

        let node = create_node(1, &blokli_client).await?;

        let ticket_processor = HoprUnacknowledgedTicketProcessor::new(
            node.chain_api.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprUnacknowledgedTicketProcessorConfig::default(),
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

        let ticket_processor = HoprUnacknowledgedTicketProcessor::new(
            node.chain_api.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprUnacknowledgedTicketProcessorConfig::default(),
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
