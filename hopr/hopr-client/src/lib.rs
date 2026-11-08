//! The HOPR **client** entity.
//!
//! A pure client establishes Sessions and relays nothing: it is the `client` feature set of
//! [`hopr_lib`], which is what this crate is built with. Everything a relay or an exit needs —
//! ticket processing, the backwards ticket path, the Session server, incoming-Session admission —
//! is behind a `cfg` in `hopr-lib` and `hopr-transport` and is therefore not compiled here at all.
//!
//! The re-exports below are the whole surface a client binary is expected to need. The role
//! scoping is not maintained by hand: [`builder`] only carries the build methods the compiled role
//! has, so a client cannot reach a relay's or an exit's constructor through this crate.

#![doc = ""]

pub use hopr_lib::{
    Abortable, AbortableList, Hopr, HoprKeys, HoprTransportIO, IdentityRetrievalModes, MIN_NATIVE_BALANCE, SESSION_MTU,
    SUGGESTED_NATIVE_BALANCE, api, builder, config, constants, errors, exports, mixer, utils,
};

// The Session client API: establishing and driving Sessions towards an Exit.
#[cfg(feature = "explicit-path")]
#[allow(deprecated)]
pub use hopr_lib::HoprSessionClientExplicitPathConfig;
#[cfg(feature = "telemetry")]
pub use hopr_lib::collect_hopr_metrics;
pub use hopr_lib::{
    FlowControlConfig, HopRouting, HoprSession, HoprSessionClientConfig, HoprSessionConfigurator, InvalidPixParams,
    LOCAL_PIX_SUITE, PixParams, SessionCapabilities, SessionCapability, SessionTarget, SurbBalancerConfig,
};
#[cfg(feature = "runtime-tokio")]
pub use hopr_lib::{prepare_tokio_runtime, transfer_session, transfer_session_datagram};
