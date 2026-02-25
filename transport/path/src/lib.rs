//! Graph-based path planning for the HOPR transport layer.
//!
//! This crate provides:
//! - [`traits::PathSelector`]: Trait for selecting multi-hop paths through the network.
//! - [`selector::GraphPathSelector`]: A path selector backed by the network graph and a moka
//!   LRU/TTL cache. Hot-path selections are served from cache; a background sweep periodically
//!   pre-warms the cache for all reachable `(destination, hops)` pairs.
//! - [`PathPlanner`]: Resolves [`DestinationRouting`] to [`ResolvedTransportRouting`], delegating
//!   path selection to any [`traits::PathSelector`] implementation.
//! - [`PathSelectorConfig`]: Configuration mostly for the cache and background refresh.

pub mod errors;
pub mod planner;
pub mod selector;
pub mod traits;

pub use errors::{PathPlannerError, Result};
pub use planner::PathPlanner;
pub use selector::{GraphPathSelector, PathSelectorConfig};
pub use traits::PathSelector;
#[cfg(feature = "runtime-tokio")]
pub use traits::BackgroundRefreshable;
