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
#[cfg(any(feature = "uniform-channel", feature = "uniform-adapter"))]
pub use config::UniformConfig;
pub use config::{MixerConfig, MixerType, PoissonConfig};
#[cfg(any(feature = "uniform-channel", feature = "poisson"))]
pub use dispatch::{AnyReceiver, AnySender, create};
pub use error::SenderError;
#[cfg(feature = "poisson")]
pub use poisson::poisson_channel;
#[cfg(feature = "uniform-adapter")]
pub use sink::MixerSink;

#[cfg(test)]
mod tests {
    #[cfg(any(feature = "uniform-channel", feature = "uniform-adapter"))]
    use std::time::Duration;

    #[cfg(any(feature = "uniform-channel", feature = "uniform-adapter"))]
    use super::UniformConfig;
    use super::{MixerConfig, MixerType, PoissonConfig, SenderError};

    /// Naming each root re-export here makes the crate surface a compile-time contract: dropping
    /// a re-export breaks this test before it breaks a consumer.
    #[test]
    fn config_types_should_all_be_reachable_at_crate_root() {
        let _: MixerConfig = MixerConfig::default();
        let _: MixerType = MixerType::default();
        let _: PoissonConfig = PoissonConfig::default();
        let _ = SenderError::Full;
    }

    /// The uniform config selects an engine through `MixerType::Uniform`, so the selector and the
    /// config must stay reachable together at the root.
    #[cfg(any(feature = "uniform-channel", feature = "uniform-adapter"))]
    #[test]
    fn uniform_config_should_be_reachable_at_crate_root() {
        let uniform = UniformConfig {
            min_delay: Duration::from_millis(1),
            delay_range: Duration::from_millis(5),
        };
        let cfg = MixerConfig::new_uniform(uniform.min_delay, uniform.delay_range);

        assert_eq!(cfg.mixer_type, MixerType::Uniform(uniform));
    }

    #[cfg(feature = "poisson")]
    #[test]
    fn engine_should_be_reachable_at_crate_root() {
        let _ = super::poisson_channel::<u32>;
        let _ = super::create::<u32>;
    }
}
