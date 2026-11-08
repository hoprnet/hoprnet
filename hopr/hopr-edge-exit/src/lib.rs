//! The HOPR **exit** entity.
//!
//! The relaying exit accepts and terminates Sessions and keeps the full ticket path of a relay: it
//! is the `exit` feature set of [`hopr_lib`], which is what this crate is built with. It also
//! carries the Session client API, because an Exit establishes the return path of every Session it
//! terminates.
//!
//! The raw exit — a Session server without ticket processing — is the `raw-exit` set of
//! [`hopr_lib`] rather than a separate facade; a consumer wanting that variant depends on
//! `hopr-lib` directly.
//!
//! The re-exports below are the whole surface an exit binary is expected to need. The role scoping
//! is not maintained by hand: [`builder`] only carries the build methods the compiled role has.

pub use hopr_lib::{
    Abortable, AbortableList, Hopr, HoprKeys, HoprTransportIO, IdentityRetrievalModes, MIN_NATIVE_BALANCE, SESSION_MTU,
    SUGGESTED_NATIVE_BALANCE, api, builder, config, constants, errors, exports, mixer, utils,
};

// The Session server side: what the exit is started with, and the Sessions it terminates.
#[cfg(feature = "explicit-path")]
#[allow(deprecated)]
pub use hopr_lib::HoprSessionClientExplicitPathConfig;
pub use hopr_lib::api::node::HoprSessionServer;
#[cfg(feature = "telemetry")]
pub use hopr_lib::collect_hopr_metrics;
pub use hopr_lib::{
    FlowControlConfig, HopRouting, HoprSession, HoprSessionClientConfig, HoprSessionConfigurator, InvalidPixParams,
    LOCAL_PIX_SUITE, PixParams, SessionCapabilities, SessionCapability, SessionTarget, SurbBalancerConfig,
};
#[cfg(feature = "runtime-tokio")]
pub use hopr_lib::{prepare_tokio_runtime, transfer_session, transfer_session_datagram};
