use std::sync::Arc;
use std::time::Duration;
use moka::future::Cache;
use moka::notification::RemovalCause;
use hopr_crypto_packet::prelude::{HoprSenderId, HoprSurbId};
use hopr_crypto_packet::{HoprSurb, ReplyOpener};
use hopr_db_node::errors::NodeDbError;
use hopr_internal_types::prelude::HoprPseudonym;
use hopr_network_types::prelude::SurbMatcher;
use crate::traits::SurbStore;
use crate::types::{FoundSurb, SurbRingBuffer};

/// Configuration for the SURB cache.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, smart_default::SmartDefault)]
pub struct SurbStoreConfig {
    /// Size of the SURB ring buffer per pseudonym.
    #[default(10_000)]
    pub rb_capacity: usize,
    /// Threshold for the number of SURBs in the ring buffer, below which it is
    /// considered low ("SURB distress").
    #[default(500)]
    pub distress_threshold: usize,
}

#[derive(Clone)]
pub struct MemorySurbStore {
    pseudonym_openers: moka::sync::Cache<HoprPseudonym, moka::sync::Cache<HoprSurbId, ReplyOpener>>,
    surbs_per_pseudonym: Cache<HoprPseudonym, SurbRingBuffer<HoprSurb>>,
    cfg: SurbStoreConfig,
}

impl MemorySurbStore {
    pub fn new(cfg: SurbStoreConfig) -> Self {
        Self {
            // Reply openers are indexed by entire Sender IDs (Pseudonym + SURB ID)
            // in a cascade fashion, allowing the entire batches (by Pseudonym) to be evicted
            // if not used.
            pseudonym_openers: moka::sync::Cache::builder()
                // TODO: Expose this as a config option
                .time_to_idle(Duration::from_secs(600))
                .eviction_policy(moka::policy::EvictionPolicy::lru())
                .eviction_listener(|sender_id, _reply_opener, cause| {
                    tracing::warn!(?sender_id, ?cause, "evicting reply opener for pseudonym");
                })
                // TODO: Expose this as a config option
                .max_capacity(10_000)
                .build(),
            // SURBs are indexed only by Pseudonyms, which have longer lifetimes.
            // For each Pseudonym, there's an RB of SURBs and their IDs.
            surbs_per_pseudonym: Cache::builder()
                // TODO: Expose this as a config option
                .time_to_idle(Duration::from_secs(600))
                .eviction_policy(moka::policy::EvictionPolicy::lru())
                .eviction_listener(|pseudonym, _reply_opener, cause| {
                    tracing::warn!(%pseudonym, ?cause, "evicting surb for pseudonym");
                })
                // TODO: Expose this as a config option
                .max_capacity(10_000)
                .build(),
            cfg
        }
    }
}

impl Default for MemorySurbStore {
    fn default() -> Self {
        Self::new(SurbStoreConfig::default())
    }
}

#[ async_trait::async_trait]
impl SurbStore for MemorySurbStore {
    async fn find_surb(&self, matcher: SurbMatcher) -> Option<FoundSurb> {
        let pseudonym = matcher.pseudonym();
        let surbs_for_pseudonym = self
            .surbs_per_pseudonym
            .get(&pseudonym)
            .await?;

        match matcher {
            SurbMatcher::Pseudonym(_) => surbs_for_pseudonym.pop_one().map(|popped_surb| FoundSurb {
                sender_id: HoprSenderId::from_pseudonym_and_id(&pseudonym, popped_surb.id),
                surb: popped_surb.surb,
                remaining: popped_surb.remaining,
            }),
            // The following code intentionally only checks the first SURB in the ring buffer
            // and does not search the entire RB.
            // This is because the exact match use-case is suited only for situations
            // when there is a single SURB in the RB.
            SurbMatcher::Exact(id) => surbs_for_pseudonym
                .pop_one_if_has_id(&id.surb_id())
                .map(|popped_surb| FoundSurb {
                    sender_id: HoprSenderId::from_pseudonym_and_id(&pseudonym, popped_surb.id),
                    surb: popped_surb.surb,
                    remaining: popped_surb.remaining, // = likely 0
                }),
        }
    }

    async fn insert_surbs(&self, pseudonym: HoprPseudonym, surbs: Vec<(HoprSurbId, HoprSurb)>) -> usize {
        self.surbs_per_pseudonym
            .entry_by_ref(&pseudonym)
            .or_insert_with(futures::future::lazy(|_| {
                SurbRingBuffer::new(self.cfg.rb_capacity.max(1024))
            }))
            .await
            .value()
            .push(surbs)
    }

    async fn insert_reply_opener(&self, sender_id: HoprSenderId, opener: ReplyOpener) {
        self.pseudonym_openers
            .get_with(sender_id.pseudonym(), move || {
                moka::sync::Cache::builder()
                    // TODO: Expose this as a config option
                    .time_to_live(Duration::from_secs(3600))
                    .eviction_listener(move |id: Arc<HoprSurbId>, _, cause| {
                        if cause != RemovalCause::Explicit {
                            tracing::warn!(pseudonym = %sender_id.pseudonym(), surb_id = hex::encode(id.as_slice()), ?cause, "evicting reply opener for sender id");
                        }
                    })
                    .max_capacity(100_000)
                    .build()
            })
            .insert(sender_id.surb_id(), opener);
    }

    fn find_reply_opener(&self, sender_id: &HoprSenderId) -> Option<ReplyOpener> {
        self.pseudonym_openers
            .get(&sender_id.pseudonym())
            .and_then(|cache| cache.remove(&sender_id.surb_id()))
    }
}