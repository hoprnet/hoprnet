use hopr_types::crypto::types::OffchainPublicKey;

use crate::errors::Result;

/// A candidate path paired with its accumulated traversal cost.
///
/// The cost is a multiplicative product of per-edge quality scores in
/// `(0.0, 1.0]` — higher means better quality.
#[derive(Debug, Clone)]
pub struct PathWithCost {
    /// The path nodes (excluding source): `[intermediates..., dest]`.
    pub path: Vec<OffchainPublicKey>,
    /// Accumulated traversal cost.
    pub cost: f64,
}

/// Selects multi-hop paths through the network.
///
/// Implementors are responsible for determining how paths are found.
/// The caller (e.g. [`crate::planner::PathPlanner`]) is responsible for caching,
/// path selection strategy, and validation.
///
/// # Cycle-free invariant
///
/// Implementations **must** return only cycle-free (simple) paths — no node may
/// appear more than once in any returned path.  Cycles destroy path entropy and
/// worsen anonymity.  The built-in [`crate::selector::HoprGraphPathSelector`]
/// guarantees this by using the `simple_paths` graph algorithm, which by
/// definition never revisits a node.  Alternative implementations must uphold
/// the same invariant.
pub trait PathSelector {
    /// Return **all** candidate paths from `src` to `dest` using `hops` relays.
    ///
    /// Each returned [`PathWithCost`] contains a path `Vec<OffchainPublicKey>`
    /// of length `hops + 1` (`[intermediates..., dest]`; `src` excluded) paired
    /// with its accumulated traversal cost.
    ///
    /// Every returned path must be cycle-free (see trait-level docs).
    ///
    /// Returns `Err` when no paths can be found.
    fn select_path(&self, src: OffchainPublicKey, dest: OffchainPublicKey, hops: usize) -> Result<Vec<PathWithCost>>;
}

/// A selector that can run a background path-cache refresh loop.
///
/// Implementors pre-warm their internal caches on a periodic schedule,
/// so that steady-state traffic is always served without a blocking query.
///
/// The returned future is `'static` because it is intended to be
/// spawned as a long-lived background task.
pub trait BackgroundPathCacheRefreshable: Send + Sync {
    /// Returns a future that runs the periodic cache-refresh loop.
    ///
    /// The future never completes under normal operation.
    fn run_background_refresh(&self) -> impl std::future::Future<Output = ()> + Send + 'static;
}
