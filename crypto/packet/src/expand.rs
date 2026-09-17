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
/// implementations are free to memoize; see [`CachingKeyExpander`] in `hopr-protocol-hopr`.
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
}
