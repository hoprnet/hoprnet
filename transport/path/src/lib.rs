//! Graph-based path planning for the HOPR transport layer.
//!
//! This crate provides:
//! - [`traits::PathSelector`]: Trait for selecting multi-hop paths through the network.
//! - [`selector::GraphPathSelector`]: A lightweight graph-backed path selector that returns all candidate paths for a
//!   given `(src, dest, hops)` query without any caching.
//! - [`PathPlanner`]: Resolves [`DestinationRouting`] to [`ResolvedTransportRouting`], delegating path discovery to any
//!   [`traits::PathSelector`] implementation and maintaining a `moka`-backed cache of fully-validated [`ValidatedPath`]
//!   objects keyed by `(source, destination, options)`.
//! - [`PathPlannerConfig`]: Configuration for the planner's cache and background refresh.

pub mod errors;
pub mod planner;
pub mod selector;
pub mod traits;

pub use errors::{PathPlannerError, Result};
pub use planner::{PathPlanner, PathPlannerConfig};
pub use selector::HoprGraphPathSelector;
#[cfg(feature = "runtime-tokio")]
pub use traits::BackgroundRefreshable;
pub use traits::PathSelector;
