//! Packet mixing/delaying channels.
//!
//! Each mixer implementation is behind its own feature so a consumer compiles only what it
//! uses:
//! - `poisson` — the exponential (Poisson) release engine on a dedicated thread, `poisson_channel()`.
//! - `poisson-shared` (default) — the Poisson engine sharing the pool on the consumer task (`Arc<Mutex<_>>`, no
//!   dedicated thread), `poisson_shared_channel()`.
//! - `uniform-channel` — the uniform-delay min-heap channel, `channel()`.
//! - `uniform-adapter` — the uniform-delay `Sink` adapter, `MixerSink`.

pub mod config;
#[cfg(any(feature = "uniform-channel", feature = "poisson", feature = "poisson-shared"))]
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

#[cfg(feature = "poisson-shared")]
pub mod poisson_shared;

#[cfg(feature = "uniform-channel")]
pub use channel::channel;
pub use config::{MixerConfig, MixerType, PoissonConfig, PoissonDelay};
#[cfg(any(feature = "uniform-channel", feature = "poisson", feature = "poisson-shared"))]
pub use dispatch::{AnyReceiver, AnySender, create};
pub use error::SenderError;
#[cfg(feature = "poisson")]
pub use poisson::poisson_channel;
#[cfg(feature = "poisson-shared")]
pub use poisson_shared::poisson_shared_channel;
#[cfg(feature = "uniform-adapter")]
pub use sink::MixerSink;
