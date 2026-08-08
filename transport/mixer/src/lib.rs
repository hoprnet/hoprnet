//! Packet mixing/delaying channels.
//!
//! Each mixer implementation is behind its own feature so a consumer compiles only what it
//! uses:
//! - `poisson` (default) — the exponential (Poisson) release engine, [`poisson_channel`].
//! - `uniform-channel` — the uniform-delay min-heap channel, [`channel`].
//! - `uniform-adapter` — the uniform-delay `Sink` adapter, [`MixerSink`].

pub mod config;
pub mod error;
pub mod metrics;

#[cfg(any(feature = "uniform-channel", feature = "uniform-adapter"))]
pub mod data;

#[cfg(feature = "uniform-channel")]
pub mod channel;

#[cfg(feature = "uniform-adapter")]
pub mod sink;

#[cfg(feature = "poisson")]
pub mod poisson;
#[cfg(feature = "poisson")]
mod pool;

#[cfg(feature = "uniform-channel")]
pub use channel::channel;
pub use config::MixerConfig;
pub use error::SenderError;
#[cfg(feature = "poisson")]
pub use poisson::poisson_channel;
#[cfg(feature = "uniform-adapter")]
pub use sink::MixerSink;
