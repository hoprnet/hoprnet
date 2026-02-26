use hopr_crypto_types::types::OffchainPublicKey;

use crate::errors::Result;

/// Selects multi-hop paths through the network.
///
/// Implementors are responsible for determining how paths are found.
/// The caller (e.g. [`crate::planner::PathPlanner`]) is responsible for caching,
/// path selection strategy, and validation.
pub trait PathSelector: Send + Sync {
    /// Return **all** candidate paths from `src` to `dest` using `hops` relays.
    ///
    /// Each inner `Vec<OffchainPublicKey>` has length `hops + 1` and contains
    /// the intermediate relay nodes **and** `dest` (in that order); `src` is
    /// excluded from every path.
    ///
    /// Returns `Err` when no paths can be found.
    fn select_path(
        &self,
        src: OffchainPublicKey,
        dest: OffchainPublicKey,
        hops: usize,
    ) -> Result<Vec<Vec<OffchainPublicKey>>>;
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
