//! Packet mixing/delaying channels.
//!
//! Each mixer implementation is behind its own feature so a consumer compiles only what it
//! uses:
//! - `poisson` (default) — the virtual-clock timing-wheel release engine, pool shared behind `Arc<Mutex<_>>` on the
//!   consumer task (no dedicated thread), `poisson_channel()`.
//! - `uniform-channel` — the uniform-delay min-heap channel, `channel()`.
//! - `uniform-adapter` — the uniform-delay `Sink` adapter, `MixerSink`.

pub mod config;
#[cfg(any(feature = "uniform-channel", feature = "poisson"))]
mod dispatch;
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
pub use config::{MixerConfig, MixerType, PoissonConfig};
#[cfg(any(feature = "uniform-channel", feature = "poisson"))]
pub use dispatch::{AnyReceiver, AnySender, create};
pub use error::SenderError;
#[cfg(feature = "poisson")]
pub use poisson::poisson_channel;
#[cfg(feature = "uniform-adapter")]
pub use sink::MixerSink;
