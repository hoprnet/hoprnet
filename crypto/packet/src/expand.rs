use hopr_types::crypto::{
    errors::Result,
    types::{ExpandedOffchainPublicKey, OffchainPublicKey},
};

use crate::{HoprSphinxSuite, sphinx::prelude::SphinxSuite};

/// Supplies the expanded form of a packet key to the Sphinx sender path.
///
/// Packet keys are stored, routed and compared in their compact form, but deriving the shared
/// secrets for an outgoing packet needs the decompressed point for every hop. That is the only
/// place in the protocol where the expanded form is required, so it is reached through this trait
/// rather than by widening the types that carry a path.
///
/// Expansion is CPU-intensive and a sender keeps using the same handful of relays, so
/// implementations are free to memoize; see `CachingKeyExpander` in `hopr-protocol-hopr`.
pub trait KeyExpander: Sync {
    /// Returns the expanded form of `key`, failing if it is not a point on the curve.
    fn expand(&self, key: &OffchainPublicKey) -> Result<ExpandedOffchainPublicKey>;

    /// Expands an entire path, in order.
    fn expand_path(&self, keys: &[OffchainPublicKey]) -> Result<Vec<ExpandedOffchainPublicKey>> {
        keys.iter().map(|key| self.expand(key)).collect()
    }
}

/// A [`KeyExpander`] that decompresses on every call.
///
/// Correct everywhere, but only appropriate where a key is expanded once: tests, benchmarks, and
/// paths that are not reused. A node sending packets should memoize instead.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DirectKeyExpander;

impl KeyExpander for DirectKeyExpander {
    fn expand(&self, key: &OffchainPublicKey) -> Result<ExpandedOffchainPublicKey> {
        HoprSphinxSuite::expand_public(key)
    }
}

impl<T: KeyExpander + ?Sized> KeyExpander for &T {
    fn expand(&self, key: &OffchainPublicKey) -> Result<ExpandedOffchainPublicKey> {
        (**self).expand(key)
    }

    // Forwarded so that an override of `expand_path` (batching, one lock per path) is not lost through a reference.
    fn expand_path(&self, keys: &[OffchainPublicKey]) -> Result<Vec<ExpandedOffchainPublicKey>> {
        (**self).expand_path(keys)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use anyhow::Context;
    use hopr_types::crypto::keypairs::{Keypair, OffchainKeypair};

    use super::*;

    /// Counts how each method is reached.
    #[derive(Default)]
    struct CountingExpander {
        expand_calls: AtomicUsize,
        expand_path_calls: AtomicUsize,
    }

    impl KeyExpander for CountingExpander {
        fn expand(&self, key: &OffchainPublicKey) -> Result<ExpandedOffchainPublicKey> {
            self.expand_calls.fetch_add(1, Ordering::Relaxed);
            DirectKeyExpander.expand(key)
        }

        fn expand_path(&self, keys: &[OffchainPublicKey]) -> Result<Vec<ExpandedOffchainPublicKey>> {
            self.expand_path_calls.fetch_add(1, Ordering::Relaxed);
            keys.iter().map(|key| DirectKeyExpander.expand(key)).collect()
        }
    }

    /// Takes the expander by value, so that `&CountingExpander` selects the blanket `&T` impl.
    fn expand_with<K: KeyExpander>(expander: K, keys: &[OffchainPublicKey]) -> Result<Vec<ExpandedOffchainPublicKey>> {
        expander.expand_path(keys)
    }

    fn path(len: usize) -> Vec<OffchainPublicKey> {
        (0..len).map(|_| *OffchainKeypair::random().public()).collect()
    }

    #[test]
    fn expanding_a_path_through_a_reference_reaches_the_override() -> anyhow::Result<()> {
        let expander = CountingExpander::default();
        let keys = path(3);

        let expanded = expand_with(&expander, &keys).context("expanding the path through a reference")?;

        assert_eq!(expanded.len(), 3);
        assert_eq!(expander.expand_path_calls.load(Ordering::Relaxed), 1);
        assert_eq!(
            expander.expand_calls.load(Ordering::Relaxed),
            0,
            "the override must not fall back to per-key calls"
        );
        Ok(())
    }

    #[test]
    fn the_default_expand_path_expands_every_key_in_order() -> anyhow::Result<()> {
        let keys = path(4);

        let expanded = DirectKeyExpander.expand_path(&keys).context("expanding the path")?;

        for (key, expanded) in keys.iter().zip(&expanded) {
            assert_eq!(&OffchainPublicKey::from(expanded), key);
        }
        Ok(())
    }
}
