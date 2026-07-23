//! Common acknowledgement verification helper shared by acknowledgement processors.
//!
//! Both the `acknowledge_shares` method on [`crate::ExitAcknowledgementShareProcessor`] and the
//! `HoprUnacknowledgedTicketProcessor::acknowledge_tickets` over in `hopr-protocol-hopr`
//! need to perform the very same steps when an incoming batch of [`Acknowledgement`]s
//! arrives from a peer:
//!
//! 1. Check that *some* state is awaited from the given peer (i.e. that the local awaiting-acks cache contains the peer
//!    entry) without bumping its popularity estimator so that the entry can be evicted normally on inactivity.
//! 2. Verify all acknowledgements (using either per-acknowledgement verification or the more efficient batch
//!    verification algorithm) and translate the resulting half-keys to their challenges.
//!
//! The post-processing of the verified acknowledgements is specific to each caller
//! and is therefore left out of this helper.
use hopr_types::{
    crypto::prelude::{HalfKey, HalfKeyChallenge, OffchainPublicKey},
    internal::prelude::Acknowledgement,
};
#[cfg(feature = "rayon")]
use hopr_utils::parallelize::cpu::rayon::prelude::*;

/// Checks whether an acknowledgement from `peer` is currently expected (by looking at the
/// `awaiting_acks` cache) and, if so, verifies all incoming `acks`.
///
/// The lookup is performed in two steps so that the popularity estimator of the
/// `peer` entry inside `awaiting_acks` is not bumped on the initial existence check,
/// allowing the entry to time out naturally on inactivity.
///
/// When `use_batch_verification` is `true`, [`Acknowledgement::verify_batch`] is used,
/// which transparently falls back to per-acknowledgement verification for small batches
/// and switches to the more effective batch verification algorithm for larger ones.
/// Otherwise, each acknowledgement is verified individually.
///
/// Returns the cloned per-peer cache entry alongside the verified `(half-key, challenge)`
/// pairs, or [`None`] if no acknowledgement is expected from `peer`.
/// Invalid acknowledgements are logged and silently dropped.
pub fn verify_expected_acknowledgements<V>(
    peer: OffchainPublicKey,
    acks: Vec<Acknowledgement>,
    awaiting_acks: &moka::sync::Cache<OffchainPublicKey, V>,
    use_batch_verification: bool,
) -> Option<(V, Vec<(HalfKey, HalfKeyChallenge)>)>
where
    V: Clone + Send + Sync + 'static,
{
    // Check if we're even expecting an acknowledgement from this peer:
    // We need to first do a check that does not update the popularity estimator of `peer` in this cache,
    // so we actually allow the entry to time out eventually. However, this comes at the cost
    // of a double-lookup.
    if !awaiting_acks.contains_key(&peer) {
        tracing::trace!("not awaiting any acknowledgement from peer");
        return None;
    }
    let Some(awaiting_ack_from_peer) = awaiting_acks.get(&peer) else {
        tracing::trace!("not awaiting any acknowledgement from peer");
        return None;
    };

    // Verify all the acknowledgements and compute challenges from half-keys
    let half_keys_challenges = if use_batch_verification {
        // Uses regular verifications for small batches but switches to a more effective
        // batch verification algorithm for larger ones.
        let acks = Acknowledgement::verify_batch(acks.into_iter().map(|ack| (peer, ack)));

        #[cfg(feature = "rayon")]
        let iter = acks.into_par_iter();

        #[cfg(not(feature = "rayon"))]
        let iter = acks.into_iter();

        iter.map(|verified| {
            verified.and_then(|verified| Ok((*verified.ack_key_share(), verified.ack_key_share().to_challenge()?)))
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
            ack.verify(&peer)
                .and_then(|verified| Ok((*verified.ack_key_share(), verified.ack_key_share().to_challenge()?)))
        })
        .filter_map(|res| {
            res.inspect_err(|error| tracing::error!(%error, "failed to process acknowledgement"))
                .ok()
        })
        .collect::<Vec<_>>()
    };

    Some((awaiting_ack_from_peer, half_keys_challenges))
}

#[cfg(test)]
mod tests {
    use hopr_types::{
        crypto::prelude::{HalfKey, Keypair, OffchainKeypair},
        crypto_random::Randomizable,
        internal::prelude::VerifiedAcknowledgement,
    };

    use super::*;

    /// Marker cache value type to assert that the helper returns the entry
    /// associated with the peer (and not some default-constructed one).
    type Marker = u32;

    fn cache() -> moka::sync::Cache<OffchainPublicKey, Marker> {
        moka::sync::CacheBuilder::new(16).build()
    }

    fn make_acks(count: usize, signer: &OffchainKeypair) -> Vec<(HalfKey, Acknowledgement)> {
        (0..count)
            .map(|_| {
                let hk = HalfKey::random();
                let ack = VerifiedAcknowledgement::new(hk, signer).leak();
                (hk, ack)
            })
            .collect()
    }

    #[test]
    fn returns_none_when_peer_is_not_in_cache() {
        let cache = cache();
        let peer = OffchainKeypair::random();

        // Empty cache.
        assert!(verify_expected_acknowledgements(*peer.public(), vec![], &cache, false).is_none());
        assert!(verify_expected_acknowledgements(*peer.public(), vec![], &cache, true).is_none());

        // Cache has a different peer.
        let other = OffchainKeypair::random();
        cache.insert(*other.public(), 1);
        assert!(verify_expected_acknowledgements(*peer.public(), vec![], &cache, false).is_none());
        assert!(verify_expected_acknowledgements(*peer.public(), vec![], &cache, true).is_none());
    }

    #[test]
    fn returns_empty_pairs_for_empty_acks() -> anyhow::Result<()> {
        for use_batch in [false, true] {
            let cache = cache();
            let peer = OffchainKeypair::random();
            cache.insert(*peer.public(), 42);

            let (value, pairs) = verify_expected_acknowledgements(*peer.public(), vec![], &cache, use_batch)
                .ok_or_else(|| anyhow::anyhow!("expected Some for peer in cache"))?;
            assert_eq!(42, value, "must return the cached value for the peer");
            assert!(pairs.is_empty());
        }
        Ok(())
    }

    #[test]
    fn verifies_all_valid_acknowledgements() -> anyhow::Result<()> {
        for use_batch in [false, true] {
            let cache = cache();
            let peer = OffchainKeypair::random();
            cache.insert(*peer.public(), 7);

            const N: usize = 5;
            let prepared = make_acks(N, &peer);
            let mut expected: Vec<(HalfKey, HalfKeyChallenge)> = prepared
                .iter()
                .map(|(hk, _)| Ok::<_, anyhow::Error>((*hk, hk.to_challenge()?)))
                .collect::<Result<_, _>>()?;
            let acks = prepared.into_iter().map(|(_, a)| a).collect();

            let (value, mut pairs) = verify_expected_acknowledgements(*peer.public(), acks, &cache, use_batch)
                .ok_or_else(|| anyhow::anyhow!("expected Some"))?;

            assert_eq!(7, value);
            assert_eq!(N, pairs.len());

            // Order is not guaranteed (rayon iteration), so sort both sides
            // by half-key challenge bytes before comparing.
            let key = |(_, ch): &(HalfKey, HalfKeyChallenge)| ch.as_ref().to_vec();
            pairs.sort_by_key(key);
            expected.sort_by_key(key);
            assert_eq!(expected, pairs);
        }
        Ok(())
    }

    #[test]
    fn drops_acknowledgements_signed_by_wrong_peer() -> anyhow::Result<()> {
        for use_batch in [false, true] {
            let cache = cache();
            let peer = OffchainKeypair::random();
            let imposter = OffchainKeypair::random();
            cache.insert(*peer.public(), 0);

            // 3 valid acks from `peer`, 2 bogus acks signed by `imposter`.
            let valid = make_acks(3, &peer);
            let bogus = make_acks(2, &imposter);

            let expected_challenges: std::collections::HashSet<_> = valid
                .iter()
                .map(|(hk, _)| hk.to_challenge())
                .collect::<Result<_, _>>()?;

            let mut acks: Vec<Acknowledgement> = valid.into_iter().map(|(_, a)| a).collect();
            acks.extend(bogus.into_iter().map(|(_, a)| a));

            let (_, pairs) = verify_expected_acknowledgements(*peer.public(), acks, &cache, use_batch)
                .ok_or_else(|| anyhow::anyhow!("expected Some"))?;

            assert_eq!(3, pairs.len(), "only valid acks must survive (use_batch={use_batch})");
            let got: std::collections::HashSet<_> = pairs.into_iter().map(|(_, ch)| ch).collect();
            assert_eq!(expected_challenges, got);
        }
        Ok(())
    }

    #[test]
    fn existence_check_does_not_load_default_value() {
        // The cache uses `get`/`contains_key` — never `get_with` — so a peer absent
        // from the cache must not cause an entry to be created as a side effect.
        let cache = cache();
        let peer = OffchainKeypair::random();

        assert!(verify_expected_acknowledgements(*peer.public(), vec![], &cache, false).is_none());
        assert_eq!(0, cache.entry_count(), "no entry must be inserted on miss");
    }
}
