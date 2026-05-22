//! Graph-based path planning for the HOPR transport layer.
//!
//! This crate provides:
//! - [`traits::PathSelector`][crate::path::traits::PathSelector]: Trait for selecting multi-hop paths through the
//!   network.
//! - [`PathPlanner`]: Resolves `DestinationRouting` to `ResolvedTransportRouting`, delegating path discovery to any
//!   [`PathSelector`][crate::path::traits::PathSelector] implementation and maintaining a `moka`-backed cache of
//!   fully-validated `ValidatedPath` objects keyed by `(source, destination, options)`. Entries are evicted by
//!   configurable TTL and TTI; an optional background refresh task can be spawned when
//!   [`PathPlannerConfig::background_refresh_period`] is set.
//! - [`PathPlannerConfig`][crate::path::PathPlannerConfig]: Configuration for the planner's cache (TTL, TTI, size cap)
//!   and optional background refresh.

pub mod errors;
pub mod planner;
pub mod selector;
pub mod traits;

pub use errors::{PathPlannerError, Result};
pub use planner::{PathPlanner, PathPlannerConfig};
pub use selector::HoprGraphPathSelector;
pub use traits::{BackgroundPathCacheRefreshable, PathSelector, PathWithCost};
