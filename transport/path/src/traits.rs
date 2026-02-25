use hopr_crypto_types::types::OffchainPublicKey;

use crate::errors::Result;

/// Selects a multi-hop path through the network.
///
/// Implementors are responsible for determining how paths are found.
pub trait PathSelector: Send + Sync {
    /// Select a path from `src` to `dest` using `hops` number of relays.
    ///
    /// Returns a `Vec<OffchainPublicKey>` of length `hops + 1` containing the
    /// intermediate relay nodes **and** `dest` (in that order); `src` is
    /// excluded from the result.
    fn select_path(
        &self,
        src: OffchainPublicKey,
        dest: OffchainPublicKey,
        hops: usize,
    ) -> impl std::future::Future<Output = Result<Vec<OffchainPublicKey>>> + Send;
}

/// A selector that can run a background path-cache refresh loop.
///
/// Implementors pre-warm their internal caches on a periodic schedule,
/// so that steady-state traffic is always served without a blocking query.
///
/// The returned future is `'static` because it is intended to be
/// spawned as a long-lived background task.
///
/// Only meaningful when the `runtime-tokio` feature is enabled.
#[cfg(feature = "runtime-tokio")]
pub trait BackgroundRefreshable: Send + Sync {
    /// Returns a future that runs the periodic cache-refresh loop.
    ///
    /// The future never completes under normal operation.
    fn run_background_refresh(&self) -> impl std::future::Future<Output = ()> + Send + 'static;
}
