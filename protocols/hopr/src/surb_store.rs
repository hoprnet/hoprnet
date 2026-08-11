use std::{collections::VecDeque, sync::Arc, time::Duration};

use hopr_api::types::internal::{prelude::HoprPseudonym, routing::SurbMatcher};
use hopr_crypto_packet::prelude::*;
use moka::notification::RemovalCause;
use validator::ValidationError;

use crate::{FoundSurb, traits::SurbStore};

const MINIMUM_SURB_LIFETIME: Duration = Duration::from_secs(30);
const MINIMUM_OPENER_PSEUDONYMS: usize = 1000;
const MINIMUM_OPENERS_PER_PSEUDONYM: usize = 1000;
const MINIMUM_SURBS_PER_PSEUDONYM: usize = 1000;
const MINIMUM_OPENER_LIFETIME: Duration = Duration::from_secs(60);
const MIN_SURB_RB_CAPACITY: usize = 1024;

fn validate_pseudonyms_lifetime(lifetime: &Duration) -> Result<(), ValidationError> {
    if lifetime < &MINIMUM_SURB_LIFETIME {
        Err(ValidationError::new("pseudonyms_lifetime is too low"))
    } else {
        Ok(())
    }
}

fn validate_reply_opener_lifetime(lifetime: &Duration) -> Result<(), ValidationError> {
    if lifetime < &MINIMUM_OPENER_LIFETIME {
        Err(ValidationError::new("reply_opener_lifetime is too low"))
    } else {
        Ok(())
    }
}

fn default_rb_capacity() -> usize {
    15_000
}

fn default_distress_threshold() -> usize {
    500
}

fn default_max_openers_per_pseudonym() -> usize {
    100_000
}

fn default_max_pseudonyms() -> usize {
    10_000
}

fn default_pseudonyms_lifetime() -> Duration {
    Duration::from_secs(600)
}

fn default_reply_opener_lifetime() -> Duration {
    Duration::from_secs(3600)
}

/// Which end of the per-pseudonym buffer a pop consumes from. Replying side only.
///
/// Overflow always evicts the oldest SURB, in either order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumString, strum::Display)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[strum(serialize_all = "lowercase")]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum SurbPopOrder {
    /// Oldest first. Default; preserves the historical behaviour.
    #[default]
    Fifo,
    /// Newest first, so a return-path change applies immediately instead of only after the
    /// buffered SURBs drain. Stale ones are shed from the other end on overflow.
    Lifo,
}

/// Configuration for the SURB cache.
///
/// The configuration options affect both the sending side (SURB creator) and the
/// replying side (SURB consumer).
///
/// In the classical scenario (`Entry - Relay 1 -... - Exit`), the sending side is
/// the `Entry` and the replying side is the `Exit`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, smart_default::SmartDefault, validator::Validate)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(deny_unknown_fields)
)]
pub struct SurbStoreConfig {
    /// Size of the SURB ring buffer per pseudonym.
    ///
    /// Affects only the replying side.
    ///
    /// This indicates how many SURBs can be at most held to be used to send a reply
    /// back to the sending side.
    ///
    /// Default is 15 000.
    #[default(default_rb_capacity())]
    #[validate(range(min = 1024, message = "rb_capacity must be at least 1024"))]
    #[cfg_attr(feature = "serde", serde(default = "default_rb_capacity"))]
    pub rb_capacity: usize,
    /// Which end of the per-pseudonym buffer a pop consumes from; see [`SurbPopOrder`].
    ///
    /// Affects only the replying side. Default is [`SurbPopOrder::Fifo`].
    #[cfg_attr(feature = "serde", serde(default))]
    pub pop_order: SurbPopOrder,
    /// Threshold for the number of SURBs in the ring buffer, below which it is
    /// considered low ("SURB distress").
    ///
    /// Default is 500.
    #[default(default_distress_threshold())]
    #[validate(range(min = 10, message = "distress_threshold must be at least 10"))]
    #[cfg_attr(feature = "serde", serde(default = "default_distress_threshold"))]
    pub distress_threshold: usize,
    /// Maximum number of reply openers (SURB counterparts) per pseudonym.
    ///
    /// Affects only the sending side when decrypting a received reply.
    ///
    /// This mostly affects Sessions, as they use a fixed pseudonym.
    /// It reflects how many reply openers the initiator-side of a Session can hold,
    /// until the oldest ones are dropped. If the other party uses a SURB corresponding
    /// to a dropped reply opener, the reply message will be undecryptable by the initiator-side.
    ///
    /// Default is 100 000.
    #[default(default_max_openers_per_pseudonym())]
    #[validate(range(min = 100, message = "max_openers_per_pseudonym must be at least 100"))]
    #[cfg_attr(feature = "serde", serde(default = "default_max_openers_per_pseudonym"))]
    pub max_openers_per_pseudonym: usize,
    /// The maximum number of distinct pseudonyms for which we hold a SURB ringbuffer.
    ///
    /// Affects only the replying side.
    ///
    /// For each pseudonym, there is a ring-buffer with capacity `rb_capacity`.
    ///
    /// Default is 10 000.
    #[default(default_max_pseudonyms())]
    #[validate(range(min = 100, message = "max_pseudonyms must be at least 100"))]
    #[cfg_attr(feature = "serde", serde(default = "default_max_pseudonyms"))]
    pub max_pseudonyms: usize,
    /// Maximum lifetime of ring-buffer for each pseudonym.
    ///
    /// # Effects on sending side
    /// This is the period for which we hold all reply openers for a pseudonym.
    /// If no more messages carrying SURBs are sent during this period, the entire stash of
    /// reply openers is dropped. Preventing receiving any more replies for that pseudonym.
    ///
    /// # Effects on replying side
    /// If a pseudonym has not received any SURBs for this period,
    /// the entire ring buffer with `rb_capacity` (= all SURBs for this pseudonym) is dropped.
    /// Preventing from sending any more replies for that pseudonym.
    ///
    /// Default is 600 seconds.
    #[default(default_pseudonyms_lifetime())]
    #[validate(custom(function = "validate_pseudonyms_lifetime"))]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_pseudonyms_lifetime", with = "humantime_serde")
    )]
    pub pseudonyms_lifetime: Duration,
    /// Maximum lifetime of a reply opener.
    ///
    /// Affects only the sending side.
    ///
    /// A reply opener is distinguished using [`HoprSurbId`] and a pseudonym it belongs to.
    /// If a reply opener is not used to decrypt the received packet within this period,
    /// it is dropped. If the replying side uses the corresponding SURB to send a reply,
    /// it won't be possible to decrypt it when received.
    ///
    /// Default is 3600 seconds.
    #[default(default_reply_opener_lifetime())]
    #[validate(custom(function = "validate_reply_opener_lifetime"))]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_reply_opener_lifetime", with = "humantime_serde")
    )]
    pub reply_opener_lifetime: Duration,
}

/// Basic [`SurbStore`] implementation based on an in-memory cache.
///
/// This SURB store offers no persistence, and all SURBs and Reply Openers are lost once dropped.
///
/// The instance can be cheaply cloned.
#[derive(Clone)]
pub struct MemorySurbStore {
    pseudonym_openers: moka::sync::Cache<HoprPseudonym, moka::sync::Cache<HoprSurbId, ReplyOpener>>,
    surbs_per_pseudonym: moka::sync::Cache<HoprPseudonym, SurbRingBuffer<HoprSurb>>,
    /// Relayers this node can no longer pay. Holds at most a handful of entries (our own closing
    /// channels), so a plain set behind an `RwLock` beats a concurrent map on this read-heavy path.
    invalidated_relayers: Arc<parking_lot::RwLock<std::collections::HashSet<HoprKeyIdent>>>,
    cfg: Arc<SurbStoreConfig>,
}

impl MemorySurbStore {
    /// Creates a new instance with the given configuration.
    pub fn new(cfg: SurbStoreConfig) -> Self {
        Self {
            // Reply openers are indexed by entire Sender IDs (Pseudonym + SURB ID)
            // in a cascade fashion, allowing the entire batches (by Pseudonym) to be evicted
            // if not used.
            pseudonym_openers: moka::sync::Cache::builder()
                .time_to_idle(cfg.pseudonyms_lifetime.max(MINIMUM_SURB_LIFETIME))
                .eviction_policy(moka::policy::EvictionPolicy::lru())
                .eviction_listener(|sender_id, _reply_opener, cause| {
                    tracing::warn!(?sender_id, ?cause, "evicting reply opener for pseudonym");
                })
                .max_capacity(cfg.max_openers_per_pseudonym.max(MINIMUM_OPENER_PSEUDONYMS) as u64)
                .build(),
            // SURBs are indexed only by Pseudonyms, which have longer lifetimes.
            // For each Pseudonym, there's an RB of SURBs and their IDs.
            surbs_per_pseudonym: moka::sync::Cache::builder()
                .time_to_idle(cfg.pseudonyms_lifetime.max(MINIMUM_SURB_LIFETIME))
                .eviction_policy(moka::policy::EvictionPolicy::lru())
                .eviction_listener(|pseudonym, _reply_opener, cause| {
                    tracing::warn!(%pseudonym, ?cause, "evicting surb for pseudonym");
                })
                .max_capacity(cfg.max_pseudonyms.max(MINIMUM_SURBS_PER_PSEUDONYM) as u64)
                .build(),
            invalidated_relayers: Default::default(),
            cfg: cfg.into(),
        }
    }

    /// Whether `relayer` is currently unusable as a return path's first hop.
    pub fn is_relayer_invalidated(&self, relayer: &HoprKeyIdent) -> bool {
        self.invalidated_relayers.read().contains(relayer)
    }

    /// Whether a stored SURB can still be used to reply: its first relayer must still be payable.
    ///
    /// A direct return path is exempt — its "first relayer" is the final recipient, which needs no
    /// channel (RFC-0003 §3.2, RFC-0006 §6.1). Without that exemption, closing an unrelated channel
    /// to a session's originator would discard perfectly good SURBs.
    fn is_surb_usable(&self, surb: &HoprSurb) -> bool {
        match surb.additional_data_receiver.proof_of_relay_values().chain_length() {
            // Direct return path: the "first relayer" is the final recipient, which needs no channel.
            1 => true,
            // A chain length is hops + 1, so 0 cannot occur on a well-formed SURB. Refuse it rather
            // than let a malformed value pass as "direct" and bypass the check below.
            0 => false,
            _ => !self.invalidated_relayers.read().contains(&surb.first_relayer),
        }
    }
}

impl Default for MemorySurbStore {
    fn default() -> Self {
        Self::new(SurbStoreConfig::default())
    }
}

#[async_trait::async_trait]
impl SurbStore for MemorySurbStore {
    #[tracing::instrument(skip_all, level = "trace", fields(?matcher), ret)]
    fn find_surb(&self, matcher: SurbMatcher) -> Option<FoundSurb> {
        let pseudonym = matcher.pseudonym();
        let surbs_for_pseudonym = self.surbs_per_pseudonym.get(&pseudonym)?;

        match matcher {
            // SURBs whose return path no longer has a usable first edge are dropped on the way,
            // rather than handed out only to have the reply fail to be paid for.
            SurbMatcher::Pseudonym(_) => surbs_for_pseudonym
                .pop_next_valid(|_, surb| self.is_surb_usable(surb))
                .map(|popped_surb| FoundSurb {
                    sender_id: HoprSenderId::from_pseudonym_and_id(&pseudonym, popped_surb.id),
                    surb: popped_surb.surb,
                    remaining: popped_surb.remaining,
                }),
            // The following code intentionally only checks the SURB at the popping end of the
            // ring buffer and does not search the entire RB.
            // This is because the exact match use-case is suited only for situations
            // when there is a single SURB in the RB.
            SurbMatcher::Exact(id) => {
                surbs_for_pseudonym
                    .pop_one_if_has_id(&id.surb_id())
                    .map(|popped_surb| FoundSurb {
                        sender_id: HoprSenderId::from_pseudonym_and_id(&pseudonym, popped_surb.id),
                        surb: popped_surb.surb,
                        remaining: popped_surb.remaining, // = likely 0
                    })
            }
        }
    }

    #[tracing::instrument(skip_all, level = "trace", fields(%pseudonym, num_surbs = surbs.len()))]
    fn insert_surbs(&self, pseudonym: HoprPseudonym, surbs: Vec<(HoprSurbId, HoprSurb)>) -> usize {
        self.surbs_per_pseudonym
            .entry_by_ref(&pseudonym)
            .or_insert_with(|| SurbRingBuffer::new(self.cfg.rb_capacity.max(MIN_SURB_RB_CAPACITY), self.cfg.pop_order))
            .value()
            .push(surbs)
    }

    #[tracing::instrument(skip_all, level = "trace", fields(?sender_id))]
    fn insert_reply_opener(&self, sender_id: HoprSenderId, opener: ReplyOpener) {
        let opener_lifetime = self.cfg.reply_opener_lifetime.max(MINIMUM_OPENER_LIFETIME);
        let max_openers_per_pseudonym = self.cfg.max_openers_per_pseudonym.max(MINIMUM_OPENERS_PER_PSEUDONYM);
        self.pseudonym_openers
            .get_with(sender_id.pseudonym(), move || {
                moka::sync::Cache::builder()
                    .time_to_live(opener_lifetime)
                    .eviction_listener(move |id: Arc<HoprSurbId>, _, cause| {
                        if cause != RemovalCause::Explicit {
                            tracing::warn!(
                                pseudonym = %sender_id.pseudonym(),
                                surb_id = const_hex::encode(id.as_slice()),
                                ?cause,
                                "evicting reply opener for sender id"
                            );
                        }
                    })
                    .max_capacity(max_openers_per_pseudonym as u64)
                    .build()
            })
            .insert(sender_id.surb_id(), opener);
    }

    #[tracing::instrument(skip_all, level = "trace")]
    fn invalidate_relayer(&self, relayer: &HoprKeyIdent) {
        if self.invalidated_relayers.write().insert(*relayer) {
            tracing::info!(
                %relayer,
                "invalidating stored SURBs whose return path starts with this relayer"
            );
        }
    }

    #[tracing::instrument(skip_all, level = "trace")]
    fn revalidate_relayer(&self, relayer: &HoprKeyIdent) {
        if self.invalidated_relayers.write().remove(relayer) {
            tracing::info!(%relayer, "relayer is usable again for SURB return paths");
        }
    }

    #[tracing::instrument(skip_all, level = "trace", fields(?sender_id), ret)]
    fn find_reply_opener(&self, sender_id: &HoprSenderId) -> Option<ReplyOpener> {
        self.pseudonym_openers
            .get(&sender_id.pseudonym())
            .and_then(|cache| cache.remove(&sender_id.surb_id()))
    }
}

/// Represents a single SURB along with its ID popped from the [`SurbRingBuffer`].
#[derive(Debug, Clone)]
pub struct PoppedSurb<S> {
    /// Complete SURB sender ID.
    pub id: HoprSurbId,
    /// The popped SURB.
    pub surb: S,
    /// Number of SURBs left in the RB after the pop.
    pub remaining: usize,
}

/// Ring buffer of SURBs and their IDs, all normally belonging to one pseudonym and therefore
/// identified only by [`HoprSurbId`].
///
/// Backed by a [`VecDeque`] pre-allocated to `capacity` and never allowed to exceed it, so it
/// never reallocates: a push into a full buffer evicts the oldest element first. [`SurbPopOrder`]
/// picks which end a pop consumes from; overflow always evicts the oldest, in either order.
#[derive(Clone, Debug)]
pub struct SurbRingBuffer<S> {
    surbs: Arc<parking_lot::Mutex<VecDeque<(HoprSurbId, S)>>>,
    capacity: usize,
    pop_order: SurbPopOrder,
}

impl<S> SurbRingBuffer<S> {
    /// Creates a buffer holding at most `capacity` (min 1, so a push is never a no-op) SURBs,
    /// popped in the given order.
    pub fn new(capacity: usize, pop_order: SurbPopOrder) -> Self {
        let capacity = capacity.max(1);
        Self {
            surbs: Arc::new(parking_lot::Mutex::new(VecDeque::with_capacity(capacity))),
            capacity,
            pop_order,
        }
    }

    /// Pushes all SURBs with their IDs, evicting the oldest ones past capacity.
    ///
    /// Returns the number of elements held after the push.
    pub fn push<I: IntoIterator<Item = (HoprSurbId, S)>>(&self, surbs: I) -> usize {
        let mut rb = self.surbs.lock();
        for surb in surbs {
            // Evict before inserting, so that the length never exceeds the pre-allocated
            // capacity and the backing allocation stays put.
            if rb.len() == self.capacity {
                rb.pop_front();
            }
            rb.push_back(surb);
        }
        rb.len()
    }

    /// Pops the next SURB that `is_valid` accepts, in the buffer's [`SurbPopOrder`].
    ///
    /// **Destructive:** rejected entries are discarded, not skipped, so an unusable SURB neither is
    /// handed out nor blocks those behind it. Pass only a validity test — a selective predicate
    /// (say, a routing preference) would drain the buffer. `None` once it is exhausted without a
    /// match.
    ///
    /// `is_valid` runs *outside* the lock: it is caller-supplied and may take locks of its own, so
    /// calling it inside the critical section would invite lock-order inversion.
    pub fn pop_next_valid<F>(&self, is_valid: F) -> Option<PoppedSurb<S>>
    where
        F: Fn(&HoprSurbId, &S) -> bool,
    {
        loop {
            let (id, surb, remaining) = {
                let mut rb = self.surbs.lock();
                let (id, surb) = match self.pop_order {
                    SurbPopOrder::Fifo => rb.pop_front()?,
                    SurbPopOrder::Lifo => rb.pop_back()?,
                };
                (id, surb, rb.len())
            };

            if is_valid(&id, &surb) {
                return Some(PoppedSurb { id, surb, remaining });
            }
        }
    }

    /// Pops the next SURB (in the buffer's [`SurbPopOrder`]) only if it has the given ID.
    pub fn pop_one_if_has_id(&self, id: &HoprSurbId) -> Option<PoppedSurb<S>> {
        let mut rb = self.surbs.lock();

        let next = match self.pop_order {
            SurbPopOrder::Fifo => rb.front(),
            SurbPopOrder::Lifo => rb.back(),
        };

        if next.is_some_and(|(surb_id, _)| surb_id == id) {
            let (id, surb) = match self.pop_order {
                SurbPopOrder::Fifo => rb.pop_front()?,
                SurbPopOrder::Lifo => rb.pop_back()?,
            };
            Some(PoppedSurb {
                id,
                surb,
                remaining: rb.len(),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use hopr_api::types::crypto::crypto_traits::Randomizable;
    use hopr_crypto_packet::sphinx::prelude::SphinxHeaderSpec;
    use rstest::rstest;

    use super::*;

    impl<S> SurbRingBuffer<S> {
        /// Pops the next SURB regardless of validity — the buffer-ordering tests below are about
        /// which end is consumed, not about which SURBs are usable.
        fn pop_any(&self) -> Option<PoppedSurb<S>> {
            self.pop_next_valid(|_, _| true)
        }
    }

    /// Builds a SURB with the given first relayer and PoR chain length (= return path length).
    ///
    /// Only those two fields are read by the store, so the SURB is assembled straight from its
    /// wire layout — `first_relayer | alpha | header | sender_key | additional_data_receiver` —
    /// whose parser performs no cryptographic validation. That avoids a full Sphinx key exchange
    /// per fixture and keeps the chain length exactly controllable.
    fn surb_via(first_relayer: HoprKeyIdent, chain_length: u8) -> anyhow::Result<HoprSurb> {
        let mut bytes = vec![0u8; HoprSurb::SIZE];

        let key_id_size = HoprSphinxHeaderSpec::KEY_ID_SIZE.get();
        bytes[..key_id_size].copy_from_slice(first_relayer.as_ref());

        // The chain length is the leading byte of the receiver's proof-of-relay values, which
        // in turn lead the trailing `additional_data_receiver` block.
        bytes[HoprSurb::SIZE - HoprSphinxHeaderSpec::SURB_RECEIVER_DATA_SIZE] = chain_length;

        let surb = HoprSurb::try_from(bytes.as_slice())?;

        // Guard the hand-rolled layout: a wrong offset would silently yield chain length 0 and
        // make the assertions below pass for the wrong reason.
        assert_eq!(first_relayer, surb.first_relayer, "fixture: wrong first relayer");
        assert_eq!(
            chain_length,
            surb.additional_data_receiver.proof_of_relay_values().chain_length(),
            "fixture: wrong chain length"
        );

        Ok(surb)
    }

    /// A return path with one intermediate relayer: `me -> relayer -> recipient`.
    const TWO_HOP: u8 = 2;
    /// A return path straight to the recipient, which needs no payment channel.
    const DIRECT: u8 = 1;

    #[test]
    fn memory_surb_store_should_skip_surbs_whose_first_relayer_was_invalidated() -> anyhow::Result<()> {
        let (dead, alive) = (HoprKeyIdent::from(1u32), HoprKeyIdent::from(2u32));

        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();

        // Two SURBs return via the dead relay, one via a healthy one; all are two-hop.
        store.insert_surbs(
            pseudonym,
            vec![
                ([1u8; 8], surb_via(dead, TWO_HOP)?),
                ([2u8; 8], surb_via(dead, TWO_HOP)?),
                ([3u8; 8], surb_via(alive, TWO_HOP)?),
            ],
        );

        store.invalidate_relayer(&dead);

        let found = store
            .find_surb(SurbMatcher::Pseudonym(pseudonym))
            .ok_or(anyhow::anyhow!("expected a usable SURB"))?;
        assert_eq!([3u8; 8], found.sender_id.surb_id(), "must skip past the dead relayer");
        assert_eq!(
            0, found.remaining,
            "the invalidated SURBs must be discarded, not left behind"
        );

        assert!(
            store.find_surb(SurbMatcher::Pseudonym(pseudonym)).is_none(),
            "no usable SURB should remain"
        );

        Ok(())
    }

    #[test]
    fn memory_surb_store_should_not_invalidate_surbs_with_a_direct_return_path() -> anyhow::Result<()> {
        // A single-element path means the "first relayer" is the final recipient, which needs no
        // payment channel — closing a channel to it must not discard the SURB.
        let recipient = HoprKeyIdent::from(1u32);

        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();

        store.insert_surbs(pseudonym, vec![([7u8; 8], surb_via(recipient, DIRECT)?)]);
        store.invalidate_relayer(&recipient);

        let found = store
            .find_surb(SurbMatcher::Pseudonym(pseudonym))
            .ok_or(anyhow::anyhow!("a direct-return-path SURB must stay usable"))?;
        assert_eq!([7u8; 8], found.sender_id.surb_id());

        Ok(())
    }

    #[test]
    fn memory_surb_store_should_reject_a_surb_with_a_malformed_chain_length() -> anyhow::Result<()> {
        // A chain length is hops + 1, so 0 is malformed. It must not pass as "direct" and thereby
        // skip the invalidation check.
        let relayer = HoprKeyIdent::from(1u32);

        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();

        store.insert_surbs(pseudonym, vec![([5u8; 8], surb_via(relayer, 0)?)]);
        store.invalidate_relayer(&relayer);

        assert!(store.find_surb(SurbMatcher::Pseudonym(pseudonym)).is_none());

        Ok(())
    }

    #[test]
    fn memory_surb_store_should_make_a_relayer_usable_again_after_revalidation() -> anyhow::Result<()> {
        let relayer = HoprKeyIdent::from(1u32);

        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();

        store.insert_surbs(pseudonym, vec![([9u8; 8], surb_via(relayer, TWO_HOP)?)]);

        store.invalidate_relayer(&relayer);
        store.revalidate_relayer(&relayer);

        let found = store
            .find_surb(SurbMatcher::Pseudonym(pseudonym))
            .ok_or(anyhow::anyhow!("expected the revalidated SURB"))?;
        assert_eq!([9u8; 8], found.sender_id.surb_id());

        Ok(())
    }

    #[test]
    fn surb_store_config_should_default_to_fifo() {
        assert_eq!(SurbPopOrder::Fifo, SurbStoreConfig::default().pop_order);
        assert_eq!(SurbPopOrder::Fifo, SurbPopOrder::default());
    }

    /// Eviction always removes the oldest, so both orders see the same surviving set {2,3,4},
    /// but consume it from opposite ends.
    #[rstest]
    #[case::fifo(SurbPopOrder::Fifo, [[2u8; 8], [3u8; 8], [4u8; 8]])]
    #[case::lifo(SurbPopOrder::Lifo, [[4u8; 8], [3u8; 8], [2u8; 8]])]
    fn surb_ring_buffer_should_drop_oldest_items_when_capacity_is_reached(
        #[case] order: SurbPopOrder,
        #[case] expected: [HoprSurbId; 3],
    ) -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(3, order);
        rb.push([([1u8; 8], 0)]);
        rb.push([([2u8; 8], 0)]);
        rb.push([([3u8; 8], 0)]);
        rb.push([([4u8; 8], 0)]);

        for (i, expected_id) in expected.into_iter().enumerate() {
            let popped = rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?;
            assert_eq!(expected_id, popped.id, "unexpected id at index {i}");
            assert_eq!(expected.len() - 1 - i, popped.remaining, "unexpected remaining");
        }

        assert!(rb.pop_any().is_none(), "buffer should be drained");

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_should_pop_fifo_by_default() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5, SurbPopOrder::default());

        let len = rb.push([([1u8; 8], 0)]);
        assert_eq!(1, len);

        let len = rb.push([([2u8; 8], 0)]);
        assert_eq!(2, len);

        let popped = rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!([1u8; 8], popped.id);
        assert_eq!(1, popped.remaining);

        let popped = rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!([2u8; 8], popped.id);
        assert_eq!(0, popped.remaining);

        let len = rb.push([([1u8; 8], 0), ([2u8; 8], 0)]);
        assert_eq!(2, len);

        assert_eq!([1u8; 8], rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?.id);
        assert_eq!([2u8; 8], rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?.id);

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_should_pop_lifo_when_configured() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5, SurbPopOrder::Lifo);

        let len = rb.push([([1u8; 8], 0)]);
        assert_eq!(1, len);

        let len = rb.push([([2u8; 8], 0)]);
        assert_eq!(2, len);

        let popped = rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!([2u8; 8], popped.id);
        assert_eq!(1, popped.remaining);

        let popped = rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!([1u8; 8], popped.id);
        assert_eq!(0, popped.remaining);

        let len = rb.push([([1u8; 8], 0), ([2u8; 8], 0)]);
        assert_eq!(2, len);

        assert_eq!([2u8; 8], rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?.id);
        assert_eq!([1u8; 8], rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?.id);

        Ok(())
    }

    #[rstest]
    #[case::fifo(SurbPopOrder::Fifo)]
    #[case::lifo(SurbPopOrder::Lifo)]
    fn surb_ring_buffer_should_skip_entries_failing_the_predicate(#[case] order: SurbPopOrder) -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5, order);
        rb.push([([1u8; 8], 0), ([2u8; 8], 0), ([3u8; 8], 0)]);

        // Only the middle entry is acceptable, so the two rejected ones must be discarded.
        let popped = rb
            .pop_next_valid(|id, _| id == &[2u8; 8])
            .ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!([2u8; 8], popped.id);

        // The rejected entries are gone, not merely skipped over.
        assert_eq!(1, popped.remaining);
        assert!(rb.pop_next_valid(|id, _| id == &[2u8; 8]).is_none());

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_should_return_none_when_no_entry_satisfies_the_predicate() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5, SurbPopOrder::Lifo);
        rb.push([([1u8; 8], 0), ([2u8; 8], 0)]);

        assert!(rb.pop_next_valid(|_, _| false).is_none());
        // The buffer is fully drained by the exhaustive search.
        assert!(rb.pop_any().is_none());

        Ok(())
    }

    #[rstest]
    #[case::fifo(SurbPopOrder::Fifo)]
    #[case::lifo(SurbPopOrder::Lifo)]
    fn surb_ring_buffer_should_not_reallocate_under_steady_overflow(#[case] order: SurbPopOrder) -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(8, order);
        let initial_capacity = rb.surbs.lock().capacity();

        for i in 0..1_000u32 {
            rb.push([(((i as u64).to_be_bytes()), 0)]);
            if i % 3 == 0 {
                rb.pop_any();
            }
            assert!(rb.surbs.lock().len() <= 8, "length exceeded capacity");
        }

        assert_eq!(initial_capacity, rb.surbs.lock().capacity(), "buffer reallocated");

        Ok(())
    }

    #[rstest]
    #[case::fifo(SurbPopOrder::Fifo)]
    #[case::lifo(SurbPopOrder::Lifo)]
    fn surb_ring_buffer_should_not_pop_if_id_does_not_match(#[case] order: SurbPopOrder) -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5, order);

        rb.push([([1u8; 8], 0)]);

        assert!(rb.pop_one_if_has_id(&[2u8; 8]).is_none());
        assert_eq!(
            [1u8; 8],
            rb.pop_one_if_has_id(&[1u8; 8])
                .ok_or(anyhow::anyhow!("expected pop"))?
                .id
        );

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_should_check_the_popping_end_for_an_exact_id() -> anyhow::Result<()> {
        let fifo = SurbRingBuffer::new(5, SurbPopOrder::Fifo);
        fifo.push([([1u8; 8], 0), ([2u8; 8], 0)]);
        assert!(fifo.pop_one_if_has_id(&[2u8; 8]).is_none());
        assert_eq!(
            [1u8; 8],
            fifo.pop_one_if_has_id(&[1u8; 8])
                .ok_or(anyhow::anyhow!("expected pop"))?
                .id
        );

        let lifo = SurbRingBuffer::new(5, SurbPopOrder::Lifo);
        lifo.push([([1u8; 8], 0), ([2u8; 8], 0)]);
        assert!(lifo.pop_one_if_has_id(&[1u8; 8]).is_none());
        assert_eq!(
            [2u8; 8],
            lifo.pop_one_if_has_id(&[2u8; 8])
                .ok_or(anyhow::anyhow!("expected pop"))?
                .id
        );

        Ok(())
    }
}
