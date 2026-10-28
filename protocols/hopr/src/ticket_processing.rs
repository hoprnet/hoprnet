use hopr_api::{
    chain::ChainReadChannelOperations,
    types::{crypto::prelude::*, internal::prelude::*},
};
#[cfg(feature = "rayon")]
use hopr_utils::parallelize::cpu::rayon::prelude::*;
use validator::ValidationError;

use crate::{HoprProtocolError, ResolvedAcknowledgement, TicketAcknowledgementError, UnacknowledgedTicketProcessor};

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
        /// Whether per-peer metrics are enabled (disabled by default to avoid cardinality explosion).
        static ref PER_PEER_ENABLED: bool = std::env::var("HOPR_METRICS_UNACK_PER_PEER")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        static ref UNACK_PEERS: hopr_api::types::telemetry::SimpleGauge = hopr_api::types::telemetry::SimpleGauge::new(
            "hopr_tickets_unack_peers_total",
            "Number of peers with unacknowledged tickets in cache",
        )
        .unwrap();
        static ref UNACK_TICKETS: hopr_api::types::telemetry::SimpleGauge = hopr_api::types::telemetry::SimpleGauge::new(
            "hopr_tickets_unack_tickets_total",
            "Total number of unacknowledged tickets across all peer caches",
        )
        .unwrap();
        static ref UNACK_INSERTIONS: hopr_api::types::telemetry::SimpleCounter = hopr_api::types::telemetry::SimpleCounter::new(
            "hopr_tickets_unack_insertions_total",
            "Total number of unacknowledged tickets inserted into cache",
        )
        .unwrap();
        static ref UNACK_LOOKUPS: hopr_api::types::telemetry::SimpleCounter = hopr_api::types::telemetry::SimpleCounter::new(
            "hopr_tickets_unack_lookups_total",
            "Total number of ticket acknowledgement lookups",
        )
        .unwrap();
        static ref UNACK_LOOKUP_MISSES: hopr_api::types::telemetry::SimpleCounter = hopr_api::types::telemetry::SimpleCounter::new(
            "hopr_tickets_unack_lookup_misses_total",
            "Total number of ticket lookup failures (unknown ticket)",
        )
        .unwrap();
        static ref UNACK_EVICTIONS: hopr_api::types::telemetry::SimpleCounter = hopr_api::types::telemetry::SimpleCounter::new(
            "hopr_tickets_unack_evictions_total",
            "Total number of unacknowledged tickets evicted from cache due to TTL or capacity limits",
        )
        .unwrap();
        static ref UNACK_PEER_EVICTIONS: hopr_api::types::telemetry::SimpleCounter = hopr_api::types::telemetry::SimpleCounter::new(
            "hopr_tickets_unack_peer_evictions_total",
            "Total number of peer caches evicted from the outer unacknowledged ticket cache",
        )
        .unwrap();
        static ref UNACK_TICKETS_PER_PEER: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
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
    /// This also bounds replay protection: the record that a ticket index was already accepted expires
    /// on the same timeout, so an index can be replayed once this period has passed. Raising it
    /// strengthens that protection at the cost of memory, but unless
    /// [`max_unack_tickets`](Self::max_unack_tickets) is raised accordingly, the capacity becomes the binding limit
    /// sooner.
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
    /// This also sizes the separate replay-protection cache of accepted ticket indices (additional
    /// memory). Once it is full, its least-recently-used records are evicted and those indices become
    /// replayable again; fresh indices are never refused. Size it for the number of tickets accepted
    /// per [`unack_ticket_timeout`](Self::unack_ticket_timeout) window.
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
        moka::sync::Cache<OffchainPublicKey, moka::sync::Cache<HalfKeyChallenge, UnacknowledgedTicket>>,
    /// Tracks ticket indices already accepted per (issuer -> this node channel, epoch), so a
    /// counterparty cannot replay the same index in many tickets (only one of which would ever be
    /// redeemable).
    ///
    /// An entry is never removed upon acknowledgement: a ticket whose acknowledgement is withheld,
    /// or that turns out to be losing, must not allow its index to be replayed either.
    ///
    /// Entries expire after
    /// [`unack_ticket_timeout`](HoprUnacknowledgedTicketProcessorConfig::unack_ticket_timeout) and, once
    /// [`max_unack_tickets`](HoprUnacknowledgedTicketProcessorConfig::max_unack_tickets) is reached, the
    /// least-recently-used ones are evicted (see the LRU note in `insert_unacknowledged_ticket`).
    /// The tracker is in-memory only and does not survive a restart.
    seen_ticket_indices: moka::sync::Cache<(ChannelId, u32, u64), ()>,
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

        #[allow(unused_mut)]
        let mut builder = moka::sync::Cache::builder()
            .time_to_idle(cfg.unack_ticket_timeout)
            .max_capacity(100_000);

        #[cfg(all(feature = "telemetry", not(test)))]
        {
            builder = builder.eviction_listener(
                |_, value: moka::sync::Cache<HalfKeyChallenge, UnacknowledgedTicket>, cause| {
                    if !matches!(cause, moka::notification::RemovalCause::Replaced) {
                        UNACK_PEERS.decrement(1.0);

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
                    // TODO: Improve the metrics in the module in general, because run_pending_tasks() is lengthy
                    // See https://github.com/hoprnet/hoprnet/issues/8014
                    // value.run_pending_tasks();
                },
            );
        }

        Self {
            unacknowledged_tickets: builder.build(),
            seen_ticket_indices: moka::sync::Cache::builder()
                .time_to_live(cfg.unack_ticket_timeout)
                .max_capacity(cfg.max_unack_tickets as u64)
                .eviction_policy(moka::policy::EvictionPolicy::lru())
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
    fn insert_unacknowledged_ticket(
        &self,
        next_hop: &OffchainPublicKey,
        challenge: HalfKeyChallenge,
        ticket: UnacknowledgedTicket,
    ) -> Result<(), Self::Error> {
        tracing::trace!(%ticket, "received unacknowledged ticket");

        // Duplicate detection on (channel, epoch, index): a counterparty must not be able
        // to have many tickets accepted on the same index, since only one of them would
        // ever be redeemable on-chain.
        // NOTE: entries are never removed upon acknowledgement, so an index whose ticket
        // was losing or whose acknowledgement was withheld cannot be replayed either.
        //
        // The channel is derived here from the signature-verified issuer and this node, never from the
        // ticket's counterparty field, so it cannot be varied by the sender even if a decoder skipped
        // the counterparty check. It is the same channel `acknowledge_tickets` resolves via
        // `channel_by_parties`, and the decoder has already validated the epoch against it (EpochMismatch).
        let channel_id = generate_channel_id(ticket.ticket.verified_issuer(), self.chain_key.as_ref());
        let channel_epoch = ticket.verified_ticket().channel_epoch;
        let index = ticket.verified_ticket().index;
        let key = (channel_id, channel_epoch, index);

        // The claim is atomic: moka coalesces concurrent calls on the same absent key, so only
        // one of them observes `is_fresh() == true`.
        //
        // The tracker uses LRU eviction because moka's default TinyLFU admission filter rejects new keys
        // once the cache is at capacity. That would either make `is_fresh()` unreliable (failing open), or,
        // if every insert were verified, refuse honest traffic on all channels. With LRU a fresh index is
        // always admitted and, at capacity, the least-recently-used records are evicted: those indices
        // become replayable again, but fresh indices are never refused.
        if !self
            .seen_ticket_indices
            .entry_by_ref(&key)
            .or_insert_with(|| ())
            .is_fresh()
        {
            tracing::debug!(%channel_id, channel_epoch, index, "duplicate ticket index rejected");
            return Err(HoprProtocolError::DuplicateTicketIndex(
                channel_id,
                channel_epoch,
                index,
            ));
        }

        self.unacknowledged_tickets
            .get_with_by_ref(next_hop, || {
                #[allow(unused_mut)]
                let mut builder = moka::sync::Cache::builder()
                    .time_to_live(self.cfg.unack_ticket_timeout)
                    .max_capacity(self.cfg.max_unack_tickets as u64);

                #[cfg(all(feature = "telemetry", not(test)))]
                {
                    UNACK_PEERS.increment(1.0);

                    let next_hop = *next_hop;
                    builder = builder.eviction_listener(move |_, _, cause| {
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

                        // Only count Expired/Size removals as evictions (not Explicit or Replaced)
                        if matches!(
                            cause,
                            moka::notification::RemovalCause::Expired | moka::notification::RemovalCause::Size
                        ) {
                            UNACK_EVICTIONS.increment();
                        }
                    });
                }

                builder.build()
            })
            .insert(challenge, ticket);

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

    #[inline]
    fn is_duplicate_ticket_index_error(error: &Self::Error) -> bool {
        error.is_duplicate_ticket_index()
    }

    #[tracing::instrument(skip_all, level = "trace", fields(peer = peer.to_peerid_str()))]
    fn acknowledge_tickets(
        &self,
        peer: OffchainPublicKey,
        acks: Vec<Acknowledgement>,
    ) -> Result<Vec<ResolvedAcknowledgement>, TicketAcknowledgementError<Self::Error>> {
        let Some((awaiting_ack_from_peer, half_keys_challenges)) =
            hopr_protocol_pix::ack_verify::verify_expected_acknowledgements(
                peer,
                acks,
                &self.unacknowledged_tickets,
                self.cfg.use_batch_verification,
            )
        else {
            return Err(TicketAcknowledgementError::UnexpectedAcknowledgement);
        };

        // Find all the tickets that we're awaiting acknowledgement for
        let mut unack_tickets = Vec::with_capacity(half_keys_challenges.len());
        for (half_key, challenge) in half_keys_challenges {
            #[cfg(all(feature = "telemetry", not(test)))]
            UNACK_LOOKUPS.increment();

            let Some(unack_ticket) = awaiting_ack_from_peer.remove(&challenge) else {
                #[cfg(all(feature = "telemetry", not(test)))]
                UNACK_LOOKUP_MISSES.increment();
                tracing::trace!(%challenge, "received acknowledgement for unknown ticket");
                continue;
            };

            let issuer_channel = match self
                .chain_api
                .channel_by_parties(unack_ticket.ticket.verified_issuer(), self.chain_key.as_ref())
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

        #[cfg(feature = "rayon")]
        let iter = unack_tickets.into_par_iter();

        #[cfg(not(feature = "rayon"))]
        let iter = unack_tickets.into_iter();

        Ok(iter
            .filter_map(|(channel, half_key, unack_ticket)| {
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
                match ack_ticket.into_redeemable(&self.chain_key, &self.channels_dst) {
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
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use hopr_api::types::crypto_random::Randomizable;

    use super::*;
    use crate::utils::*;

    // Channel id of the channel the test tickets are issued on: the utils.rs channels go
    // `PEERS[i] -> PEERS[(i+1)%5]` with epoch 1, and the test tickets are signed by `PEERS[0]`
    // with counterparty `PEERS[1]`, i.e. they travel on the `PEERS[0] -> PEERS[1]` channel.
    // The processor under test is node 1 (`PEERS[1]`), the destination of that channel.
    lazy_static::lazy_static! {
        static ref TEST_CHANNEL_ID: ChannelId =
            generate_channel_id(&PEERS[0].0.public().to_address(), &PEERS[1].0.public().to_address());
    }

    /// Builds an unacknowledged ticket on the `PEERS[0] -> PEERS[1]` channel with the given
    /// `index` and `channel_epoch`, solved by the given `ack_share`.
    fn build_unack_ticket(index: u64, channel_epoch: u32, ack_share: &HalfKey) -> anyhow::Result<UnacknowledgedTicket> {
        build_unack_ticket_from(&PEERS[0].0, &PEERS[1].0, index, channel_epoch, ack_share)
    }

    /// Same as [`build_unack_ticket`], but signed by `issuer` with the ticket's counterparty field set to
    /// `counterparty`.
    fn build_unack_ticket_from(
        issuer: &ChainKeypair,
        counterparty: &ChainKeypair,
        index: u64,
        channel_epoch: u32,
        ack_share: &HalfKey,
    ) -> anyhow::Result<UnacknowledgedTicket> {
        let own_share = HalfKey::random();
        let challenge = Challenge::from_own_share_and_half_key(&own_share.to_challenge()?, ack_share)?;

        Ok(TicketBuilder::default()
            .counterparty(counterparty)
            .index(index)
            .channel_epoch(channel_epoch)
            .amount(10_u32)
            .challenge(challenge)
            .build_signed(issuer, &Hash::default())?
            .into_unacknowledged(own_share))
    }

    /// Inserts a fresh ticket with the given `index` on the `PEERS[0] -> PEERS[1]` channel and epoch 1.
    fn insert_index<Chain>(
        ticket_processor: &HoprUnacknowledgedTicketProcessor<Chain>,
        index: u64,
    ) -> anyhow::Result<Result<(), HoprProtocolError>>
    where
        Chain: ChainReadChannelOperations + Send + Sync,
    {
        let ack_share = HalfKey::random();
        let ticket = build_unack_ticket(index, 1, &ack_share)?;
        Ok(ticket_processor.insert_unacknowledged_ticket(PEERS[2].1.public(), ack_share.to_challenge()?, ticket))
    }

    #[parameterized::parameterized(batch = { true, false })]
    #[parameterized_macro(tokio::test)]
    async fn ticket_processor_should_acknowledge_previously_inserted_tickets(batch: bool) -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;

        let node = create_node(1, &blokli_client).await?;

        let ticket_processor = HoprUnacknowledgedTicketProcessor::new(
            node.chain_api.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprUnacknowledgedTicketProcessorConfig {
                use_batch_verification: batch,
                ..HoprUnacknowledgedTicketProcessorConfig::default()
            },
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

            ticket_processor.insert_unacknowledged_ticket(
                PEERS[2].1.public(),
                ack_share.to_challenge()?,
                unack_ticket,
            )?;

            acks.push(VerifiedAcknowledgement::new(ack_share, &PEERS[2].1).leak());
        }

        let resolutions = ticket_processor.acknowledge_tickets(*PEERS[2].1.public(), acks)?;
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
            ticket_processor.acknowledge_tickets(*PEERS[2].1.public(), acks),
            Err(TicketAcknowledgementError::UnexpectedAcknowledgement)
        ));

        Ok(())
    }

    #[tokio::test]
    async fn ticket_processor_should_ignore_bogus_acknowledgements() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let node = create_node(1, &blokli_client).await?;
        let ticket_processor = HoprUnacknowledgedTicketProcessor::new(
            node.chain_api.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprUnacknowledgedTicketProcessorConfig::default(),
        );

        let own_share = HalfKey::random();
        let ack_share = HalfKey::random();
        let challenge = Challenge::from_own_share_and_half_key(&own_share.to_challenge()?, &ack_share)?;

        let unack_ticket = TicketBuilder::default()
            .counterparty(&PEERS[1].0)
            .index(0)
            .channel_epoch(1)
            .amount(10_u32)
            .challenge(challenge)
            .build_signed(&PEERS[0].0, &Hash::default())?
            .into_unacknowledged(own_share);

        ticket_processor.insert_unacknowledged_ticket(PEERS[2].1.public(), ack_share.to_challenge()?, unack_ticket)?;

        // Create an acknowledgement from a DIFFERENT peer, but we'll try to pass it as if it's from PEERS[2]
        let bogus_ack_share = HalfKey::random();
        let bogus_ack = VerifiedAcknowledgement::new(bogus_ack_share, &PEERS[1].1).leak();

        let resolutions = ticket_processor.acknowledge_tickets(*PEERS[2].1.public(), vec![bogus_ack])?;
        // Should ignore because signature is invalid (it's from PEERS[1] but we're telling the processor it's from
        // PEERS[2])
        assert_eq!(0, resolutions.len());

        Ok(())
    }

    #[tokio::test]
    async fn ticket_processor_should_ignore_unknown_challenges() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let node = create_node(1, &blokli_client).await?;
        let ticket_processor = HoprUnacknowledgedTicketProcessor::new(
            node.chain_api.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprUnacknowledgedTicketProcessorConfig::default(),
        );

        // We insert ONE ticket
        let own_share = HalfKey::random();
        let ack_share = HalfKey::random();
        let challenge = Challenge::from_own_share_and_half_key(&own_share.to_challenge()?, &ack_share)?;

        let unack_ticket = TicketBuilder::default()
            .counterparty(&PEERS[1].0)
            .index(0)
            .channel_epoch(1)
            .amount(10_u32)
            .challenge(challenge)
            .build_signed(&PEERS[0].0, &Hash::default())?
            .into_unacknowledged(own_share);

        ticket_processor.insert_unacknowledged_ticket(PEERS[2].1.public(), ack_share.to_challenge()?, unack_ticket)?;

        // Now we send TWO acks: one for the ticket we inserted, and one random/unknown
        let unknown_ack_share = HalfKey::random();
        let acks = vec![
            VerifiedAcknowledgement::new(ack_share, &PEERS[2].1).leak(),
            VerifiedAcknowledgement::new(unknown_ack_share, &PEERS[2].1).leak(),
        ];

        let resolutions = ticket_processor.acknowledge_tickets(*PEERS[2].1.public(), acks)?;
        // Should only resolve the one we knew about
        assert_eq!(1, resolutions.len());

        Ok(())
    }

    #[tokio::test]
    async fn ticket_processor_should_ignore_epoch_mismatch() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let node = create_node(1, &blokli_client).await?;
        let ticket_processor = HoprUnacknowledgedTicketProcessor::new(
            node.chain_api.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprUnacknowledgedTicketProcessorConfig::default(),
        );

        let own_share = HalfKey::random();
        let ack_share = HalfKey::random();
        let challenge = Challenge::from_own_share_and_half_key(&own_share.to_challenge()?, &ack_share)?;

        // Channel in utils.rs has epoch 1. We'll create a ticket with epoch 2.
        let unack_ticket = TicketBuilder::default()
            .counterparty(&PEERS[1].0)
            .index(0)
            .channel_epoch(2) // Mismatch!
            .amount(10_u32)
            .challenge(challenge)
            .build_signed(&PEERS[0].0, &Hash::default())?
            .into_unacknowledged(own_share);

        ticket_processor.insert_unacknowledged_ticket(PEERS[2].1.public(), ack_share.to_challenge()?, unack_ticket)?;

        let ack = VerifiedAcknowledgement::new(ack_share, &PEERS[2].1).leak();
        let resolutions = ticket_processor.acknowledge_tickets(*PEERS[2].1.public(), vec![ack])?;
        assert_eq!(0, resolutions.len());

        Ok(())
    }

    #[tokio::test]
    async fn ticket_processor_should_ignore_missing_channel() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let node = create_node(1, &blokli_client).await?;
        let ticket_processor = HoprUnacknowledgedTicketProcessor::new(
            node.chain_api.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprUnacknowledgedTicketProcessorConfig::default(),
        );

        let own_share = HalfKey::random();
        let ack_share = HalfKey::random();
        let challenge = Challenge::from_own_share_and_half_key(&own_share.to_challenge()?, &ack_share)?;

        let unknown_issuer = ChainKeypair::random();

        let unack_ticket = TicketBuilder::default()
            .counterparty(&node.chain_key)
            .index(0)
            .channel_epoch(1)
            .amount(10_u32)
            .challenge(challenge)
            .build_signed(&unknown_issuer, &Hash::default())?
            .into_unacknowledged(own_share);

        ticket_processor.insert_unacknowledged_ticket(PEERS[2].1.public(), ack_share.to_challenge()?, unack_ticket)?;

        let ack = VerifiedAcknowledgement::new(ack_share, &PEERS[2].1).leak();
        let resolutions = ticket_processor.acknowledge_tickets(*PEERS[2].1.public(), vec![ack])?;
        assert_eq!(
            0,
            resolutions.len(),
            "Expected 0 resolutions for missing channel, got: {:?}",
            resolutions
        );

        Ok(())
    }

    #[tokio::test]
    async fn ticket_processor_should_handle_losing_tickets() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let node = create_node(1, &blokli_client).await?;
        let ticket_processor = HoprUnacknowledgedTicketProcessor::new(
            node.chain_api.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprUnacknowledgedTicketProcessorConfig::default(),
        );

        let own_share = HalfKey::random();
        let ack_share = HalfKey::random();
        let challenge = Challenge::from_own_share_and_half_key(&own_share.to_challenge()?, &ack_share)?;

        let unack_ticket = TicketBuilder::default()
            .counterparty(&PEERS[1].0)
            .index(0)
            .win_prob(WinningProbability::NEVER)
            .channel_epoch(1)
            .amount(0_u32)
            .challenge(challenge)
            .build_signed(&PEERS[0].0, &Hash::default())?
            .into_unacknowledged(own_share);

        ticket_processor.insert_unacknowledged_ticket(PEERS[2].1.public(), ack_share.to_challenge()?, unack_ticket)?;

        let ack = VerifiedAcknowledgement::new(ack_share, &PEERS[2].1).leak();
        let resolutions = ticket_processor.acknowledge_tickets(*PEERS[2].1.public(), vec![ack])?;
        assert_eq!(1, resolutions.len());
        assert!(
            matches!(resolutions[0], ResolvedAcknowledgement::RelayingLoss(_)),
            "Expected RelayingLoss for 0-amount ticket, got: {:?}",
            resolutions[0]
        );

        Ok(())
    }

    #[tokio::test]
    async fn ticket_processor_should_reject_duplicate_ticket_index_on_same_channel_and_epoch() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let node = create_node(1, &blokli_client).await?;
        let ticket_processor = HoprUnacknowledgedTicketProcessor::new(
            node.chain_api.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprUnacknowledgedTicketProcessorConfig::default(),
        );

        let first_ack_share = HalfKey::random();
        let first_ticket = build_unack_ticket(5, 1, &first_ack_share).context("should build first ticket")?;
        ticket_processor.insert_unacknowledged_ticket(
            PEERS[2].1.public(),
            first_ack_share.to_challenge()?,
            first_ticket,
        )?;

        // Same index on the same channel and epoch, but a different challenge: must be rejected
        let second_ack_share = HalfKey::random();
        let second_ticket = build_unack_ticket(5, 1, &second_ack_share).context("should build second ticket")?;
        let result = ticket_processor.insert_unacknowledged_ticket(
            PEERS[2].1.public(),
            second_ack_share.to_challenge()?,
            second_ticket,
        );

        assert!(
            matches!(
                result,
                Err(HoprProtocolError::DuplicateTicketIndex(channel_id, epoch, index))
                    if channel_id == *TEST_CHANNEL_ID && epoch == 1 && index == 5
            ),
            "expected DuplicateTicketIndex, got: {:?}",
            result
        );

        Ok(())
    }

    #[tokio::test]
    async fn ticket_processor_should_accept_reordered_but_distinct_ticket_indices() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let node = create_node(1, &blokli_client).await?;
        let ticket_processor = HoprUnacknowledgedTicketProcessor::new(
            node.chain_api.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprUnacknowledgedTicketProcessorConfig::default(),
        );

        // The mixer deliberately reorders packets, so indices may arrive out of order;
        // only exact duplicates may be rejected.
        for index in [5u64, 3, 7, 4] {
            let ack_share = HalfKey::random();
            let ticket = build_unack_ticket(index, 1, &ack_share)
                .with_context(|| format!("should build ticket with index {index}"))?;
            ticket_processor
                .insert_unacknowledged_ticket(PEERS[2].1.public(), ack_share.to_challenge()?, ticket)
                .with_context(|| format!("ticket with index {index} must be accepted"))?;
        }

        Ok(())
    }

    #[tokio::test]
    async fn ticket_processor_should_accept_same_ticket_index_in_different_epoch() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let node = create_node(1, &blokli_client).await?;
        let ticket_processor = HoprUnacknowledgedTicketProcessor::new(
            node.chain_api.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprUnacknowledgedTicketProcessorConfig::default(),
        );

        for epoch in [1u32, 2] {
            let ack_share = HalfKey::random();
            let ticket = build_unack_ticket(5, epoch, &ack_share)
                .with_context(|| format!("should build ticket with epoch {epoch}"))?;
            ticket_processor
                .insert_unacknowledged_ticket(PEERS[2].1.public(), ack_share.to_challenge()?, ticket)
                .with_context(|| format!("ticket with epoch {epoch} must be accepted"))?;
        }

        Ok(())
    }

    /// Documents and pins the tracker's remaining, *accepted* residual: replay protection expires
    /// with `unack_ticket_timeout`, independently of on-chain redemption.
    ///
    /// An index that was accepted but never redeemed becomes replayable once its record expires,
    /// so a counterparty that withholds acknowledgements (leaving the on-chain channel index
    /// unchanged) can recycle a batch of indices once per window. This is materially weaker than
    /// the reported attack — each replay needs a fresh window rather than a fresh index, and the
    /// amplification is not unbounded — but it is not nothing, and it is why this timeout is a
    /// replay-protection knob and not merely a cache knob.
    ///
    /// Closing it properly requires retiring indices on redemption rather than on a timer, which
    /// needs state outside this in-memory tracker. If that lands, this test should be inverted
    /// rather than deleted, so the guarantee is asserted in whichever direction is intended.
    #[tokio::test]
    async fn ticket_processor_replay_is_possible_only_after_the_tracker_ttl_expires() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let node = create_node(1, &blokli_client).await?;
        let ticket_processor = HoprUnacknowledgedTicketProcessor::new(
            node.chain_api.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprUnacknowledgedTicketProcessorConfig {
                // Below the production minimum of 10s, but `new` does not run the validator and the
                // test only needs to observe the expiry, not the production default.
                unack_ticket_timeout: std::time::Duration::from_millis(300),
                ..HoprUnacknowledgedTicketProcessorConfig::default()
            },
        );

        let insert = |index: u64| -> anyhow::Result<Result<(), HoprProtocolError>> {
            let ack_share = HalfKey::random();
            let ticket = build_unack_ticket(index, 1, &ack_share)?;
            Ok(ticket_processor.insert_unacknowledged_ticket(PEERS[2].1.public(), ack_share.to_challenge()?, ticket))
        };

        insert(42)
            .context("first insert must be attempted")?
            .context("a fresh index must be accepted")?;
        assert!(
            insert(42).context("immediate replay must be attempted")?.is_err(),
            "an index must not be replayable while its record is live"
        );

        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
        ticket_processor.seen_ticket_indices.run_pending_tasks();

        assert!(
            insert(42).context("post-expiry replay must be attempted")?.is_ok(),
            "if this now fails the tracker retains indices beyond its TTL, which is strictly stronger than documented \
             and should update the docs on unack_ticket_timeout"
        );

        Ok(())
    }

    #[tokio::test]
    async fn ticket_processor_should_accept_same_ticket_index_on_different_channel() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let node = create_node(1, &blokli_client).await?;
        let ticket_processor = HoprUnacknowledgedTicketProcessor::new(
            node.chain_api.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprUnacknowledgedTicketProcessorConfig::default(),
        );

        // The tracker key is the issuer -> this node channel (this node is `PEERS[1]`), so the same
        // index from a different issuer is on a different channel and must be accepted.
        for issuer in [&PEERS[0].0, &PEERS[2].0] {
            let ack_share = HalfKey::random();
            let ticket =
                build_unack_ticket_from(issuer, &PEERS[1].0, 5, 1, &ack_share).context("should build ticket")?;
            ticket_processor
                .insert_unacknowledged_ticket(PEERS[2].1.public(), ack_share.to_challenge()?, ticket)
                .with_context(|| format!("ticket from issuer {} must be accepted", issuer.public().to_address()))?;
        }

        Ok(())
    }

    /// The tracker key must not depend on the ticket's counterparty field, which the issuer chooses
    /// freely: were it part of the key (as it is of `Ticket::channel_id`), one issuer could mint an
    /// unlimited number of distinct keys for the same index and replay it at will.
    #[tokio::test]
    async fn ticket_processor_should_reject_same_ticket_index_with_different_counterparty() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let node = create_node(1, &blokli_client).await?;
        let ticket_processor = HoprUnacknowledgedTicketProcessor::new(
            node.chain_api.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprUnacknowledgedTicketProcessorConfig::default(),
        );

        let first_ack_share = HalfKey::random();
        let first_ticket = build_unack_ticket_from(&PEERS[0].0, &PEERS[1].0, 5, 1, &first_ack_share)
            .context("should build first ticket")?;
        ticket_processor.insert_unacknowledged_ticket(
            PEERS[2].1.public(),
            first_ack_share.to_challenge()?,
            first_ticket,
        )?;

        // Same issuer, epoch and index, but the counterparty field differs, so the ticket's own channel
        // id differs as well.
        let second_ack_share = HalfKey::random();
        let second_ticket = build_unack_ticket_from(&PEERS[0].0, &PEERS[3].0, 5, 1, &second_ack_share)
            .context("should build second ticket")?;
        assert_ne!(
            first_ticket.ticket.channel_id(),
            second_ticket.ticket.channel_id(),
            "sanity: the tickets must differ in their own channel id"
        );

        let result = ticket_processor.insert_unacknowledged_ticket(
            PEERS[2].1.public(),
            second_ack_share.to_challenge()?,
            second_ticket,
        );
        assert!(
            matches!(
                result,
                Err(HoprProtocolError::DuplicateTicketIndex(channel_id, epoch, index))
                    if channel_id == *TEST_CHANNEL_ID && epoch == 1 && index == 5
            ),
            "expected DuplicateTicketIndex, got: {:?}",
            result
        );

        Ok(())
    }

    #[tokio::test]
    async fn ticket_processor_should_still_reject_index_replayed_after_acknowledgement() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let node = create_node(1, &blokli_client).await?;
        let ticket_processor = HoprUnacknowledgedTicketProcessor::new(
            node.chain_api.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprUnacknowledgedTicketProcessorConfig::default(),
        );

        let ack_share = HalfKey::random();
        let ticket = build_unack_ticket(5, 1, &ack_share).context("should build first ticket")?;
        ticket_processor.insert_unacknowledged_ticket(PEERS[2].1.public(), ack_share.to_challenge()?, ticket)?;

        // The index is NOT removed from the tracker upon acknowledgement: a counterparty could
        // otherwise replay any index whose ticket was losing or whose acknowledgement it withheld.
        let ack = VerifiedAcknowledgement::new(ack_share, &PEERS[2].1).leak();
        let resolutions = ticket_processor.acknowledge_tickets(*PEERS[2].1.public(), vec![ack])?;
        assert_eq!(1, resolutions.len());

        let replay_ack_share = HalfKey::random();
        let replay_ticket = build_unack_ticket(5, 1, &replay_ack_share).context("should build replayed ticket")?;
        let result = ticket_processor.insert_unacknowledged_ticket(
            PEERS[2].1.public(),
            replay_ack_share.to_challenge()?,
            replay_ticket,
        );
        assert!(
            matches!(result, Err(HoprProtocolError::DuplicateTicketIndex(..))),
            "expected DuplicateTicketIndex after acknowledgement, got: {:?}",
            result
        );

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn ticket_processor_should_accept_duplicate_index_only_once_concurrently() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let node = create_node(1, &blokli_client).await?;
        let ticket_processor = std::sync::Arc::new(HoprUnacknowledgedTicketProcessor::new(
            node.chain_api.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprUnacknowledgedTicketProcessorConfig::default(),
        ));

        const NUM_CONCURRENT: usize = 8;

        // Hold every task on the barrier so all inserts race on the SAME absent key at
        // once, instead of running to completion sequentially on one worker.
        let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(NUM_CONCURRENT));

        let handles = (0..NUM_CONCURRENT)
            .map(|_| {
                let ticket_processor = ticket_processor.clone();
                let barrier = barrier.clone();
                tokio::task::spawn(async move {
                    let inner = async {
                        let ack_share = HalfKey::random();
                        let ticket = build_unack_ticket(5, 1, &ack_share).context("should build ticket")?;
                        let challenge = ack_share.to_challenge().context("should derive challenge")?;
                        barrier.wait().await;
                        ticket_processor
                            .insert_unacknowledged_ticket(PEERS[2].1.public(), challenge, ticket)
                            .map_err(anyhow::Error::from)
                    }
                    .await;
                    inner.map_err(|error| anyhow::anyhow!("concurrent insert failed: {error}"))
                })
            })
            .collect::<Vec<_>>();

        let mut results = Vec::with_capacity(NUM_CONCURRENT);
        for handle in handles {
            results.push(handle.await.context("concurrent insert task should not panic")?);
        }

        let accepted = results.iter().filter(|r| r.is_ok()).count();
        assert_eq!(
            1, accepted,
            "exactly one concurrent insert of the same index must succeed, got: {:?}",
            results
        );

        Ok(())
    }

    /// Capacity used by the LRU tests below. It equals the minimum accepted by the config validator.
    const SMALL_TRACKER_CAPACITY: usize = 100;

    /// Once the tracker is full, every further fresh index must still be accepted: the tracker evicts
    /// its least-recently-used records instead of refusing newcomers, so honest traffic (on any
    /// channel) is never dropped because the tracker is saturated.
    ///
    /// With moka's default TinyLFU policy plus a retention probe after each insert (an earlier design of
    /// this tracker), this test fails: TinyLFU's admission filter rejects new keys at capacity, so the
    /// probe refused fresh indices. `ticket_processor_should_reject_recent_ticket_index_when_the_tracker_is_full`
    /// is the one that fails if the LRU policy alone is reverted.
    #[tokio::test]
    async fn ticket_processor_should_accept_every_fresh_ticket_index_once_the_tracker_is_full() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let node = create_node(1, &blokli_client).await?;
        let ticket_processor = HoprUnacknowledgedTicketProcessor::new(
            node.chain_api.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprUnacknowledgedTicketProcessorConfig {
                max_unack_tickets: SMALL_TRACKER_CAPACITY,
                ..HoprUnacknowledgedTicketProcessorConfig::default()
            },
        );

        for index in 0..(5 * SMALL_TRACKER_CAPACITY) as u64 {
            insert_index(&ticket_processor, index)
                .with_context(|| format!("should build ticket with index {index}"))?
                .with_context(|| format!("fresh index {index} must be accepted, even at capacity"))?;
        }

        ticket_processor.seen_ticket_indices.run_pending_tasks();
        assert!(
            ticket_processor.seen_ticket_indices.entry_count() <= SMALL_TRACKER_CAPACITY as u64,
            "the tracker must stay bounded by its capacity"
        );

        Ok(())
    }

    /// A recently accepted index must still be rejected once the tracker is full: LRU eviction only
    /// ever drops the oldest records.
    #[tokio::test]
    async fn ticket_processor_should_reject_recent_ticket_index_when_the_tracker_is_full() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let node = create_node(1, &blokli_client).await?;
        let ticket_processor = HoprUnacknowledgedTicketProcessor::new(
            node.chain_api.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprUnacknowledgedTicketProcessorConfig {
                max_unack_tickets: SMALL_TRACKER_CAPACITY,
                ..HoprUnacknowledgedTicketProcessorConfig::default()
            },
        );

        let last_index = (5 * SMALL_TRACKER_CAPACITY) as u64 - 1;
        for index in 0..=last_index {
            insert_index(&ticket_processor, index)
                .with_context(|| format!("should build ticket with index {index}"))?
                .with_context(|| format!("fresh index {index} must be accepted"))?;
        }
        ticket_processor.seen_ticket_indices.run_pending_tasks();

        let result = insert_index(&ticket_processor, last_index).context("should build replayed ticket")?;
        assert!(
            matches!(
                result,
                Err(HoprProtocolError::DuplicateTicketIndex(channel_id, 1, index))
                    if channel_id == *TEST_CHANNEL_ID && index == last_index
            ),
            "a recently accepted index must be rejected at capacity, got: {:?}",
            result
        );

        Ok(())
    }

    /// Documents and pins the second *accepted* residual of the tracker: replay protection is bounded
    /// by `max_unack_tickets` as well as by `unack_ticket_timeout`.
    ///
    /// Once the tracker is full, its least-recently-used records are evicted, so the oldest index
    /// becomes replayable after enough fresh indices were accepted, even within its TTL. This is the
    /// price of never refusing a fresh index; the capacity should be sized for the number of tickets
    /// accepted per `unack_ticket_timeout` window (see the docs on `max_unack_tickets`).
    ///
    /// If the tracker ever gains stronger retention, this test should be inverted rather than deleted,
    /// so the guarantee is asserted in whichever direction is intended.
    #[tokio::test]
    async fn ticket_processor_replay_is_possible_once_the_tracker_evicts_the_oldest_index() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let node = create_node(1, &blokli_client).await?;
        let ticket_processor = HoprUnacknowledgedTicketProcessor::new(
            node.chain_api.clone(),
            node.chain_key.clone(),
            Hash::default(),
            HoprUnacknowledgedTicketProcessorConfig {
                max_unack_tickets: SMALL_TRACKER_CAPACITY,
                ..HoprUnacknowledgedTicketProcessorConfig::default()
            },
        );

        let oldest_index = 0u64;
        insert_index(&ticket_processor, oldest_index)
            .context("first insert must be attempted")?
            .context("a fresh index must be accepted")?;
        assert!(
            insert_index(&ticket_processor, oldest_index)
                .context("immediate replay must be attempted")?
                .is_err(),
            "an index must not be replayable while its record is retained"
        );

        for index in 1..(5 * SMALL_TRACKER_CAPACITY) as u64 {
            insert_index(&ticket_processor, index)
                .with_context(|| format!("should build ticket with index {index}"))?
                .with_context(|| format!("fresh index {index} must be accepted"))?;
        }
        ticket_processor.seen_ticket_indices.run_pending_tasks();

        assert!(
            insert_index(&ticket_processor, oldest_index)
                .context("post-eviction replay must be attempted")?
                .is_ok(),
            "if this now fails the tracker retains the oldest index beyond its capacity, which is strictly stronger \
             than documented and should update the docs on max_unack_tickets"
        );

        Ok(())
    }

    /// The predicate that the packet pipeline consults to pick the rejection metric label must be
    /// true for a duplicate-index rejection and false for every other error.
    ///
    /// NOTE: this pins the *predicate* only. The label it selects is applied in
    /// `transport/hopr`'s `unack_rejection_metric_label`, which is where the mapping itself is
    /// pinned — the counter there is behind `not(test)` and cannot be asserted from here.
    #[tokio::test]
    async fn ticket_processor_only_duplicate_index_error_maps_to_its_own_metric_label() -> anyhow::Result<()> {
        use hopr_api::types::primitive::prelude::{Address, HoprBalance};

        type TestProcessor = HoprUnacknowledgedTicketProcessor<
            std::sync::Arc<
                hopr_chain_connector::HoprBlockchainSafeConnector<
                    hopr_chain_connector::testing::BlokliTestClient<hopr_chain_connector::testing::StaticState>,
                >,
            >,
        >;

        let duplicate = HoprProtocolError::DuplicateTicketIndex(Hash::default(), 1, 5);
        assert!(
            TestProcessor::is_duplicate_ticket_index_error(&duplicate),
            "DuplicateTicketIndex must be reported under its own metric label"
        );

        let other_errors = [
            HoprProtocolError::Replay,
            HoprProtocolError::InvalidSender,
            HoprProtocolError::KeyNotFound,
            HoprProtocolError::ChannelNotFound(Address::default(), Address::default()),
            HoprProtocolError::OutOfFunds(Default::default(), HoprBalance::zero()),
        ];
        for error in other_errors {
            assert!(
                !TestProcessor::is_duplicate_ticket_index_error(&error),
                "{error} must not be reported as a duplicate ticket index"
            );
        }

        Ok(())
    }
}
