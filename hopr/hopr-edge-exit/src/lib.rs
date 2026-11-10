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

// The exit builds Sessions in both directions, so it carries the client API and the explicit-path
// configuration, plus the server handle below that callers start it from.
#[cfg(feature = "explicit-path")]
#[allow(deprecated)]
pub use hopr_lib::HoprSessionClientExplicitPathConfig;
#[cfg(feature = "telemetry")]
pub use hopr_lib::collect_hopr_metrics;
// The Session copy helpers a binary needs to move data through an established Session.
pub use hopr_lib::exports::transport::session::{transfer_session, transfer_session_datagram};
#[cfg(feature = "runtime-tokio")]
pub use hopr_lib::prepare_tokio_runtime;
pub use hopr_lib::{
    Abortable, AbortableList, FlowControlConfig, HopRouting, Hopr, HoprKeys, HoprSession, HoprSessionClientConfig,
    HoprSessionConfigurator, HoprTransportIO, IdentityRetrievalModes, InvalidPixParams, LOCAL_PIX_SUITE,
    MIN_NATIVE_BALANCE, PixParams, SESSION_MTU, SUGGESTED_NATIVE_BALANCE, SessionCapabilities, SessionCapability,
    SessionTarget, SurbBalancerConfig, api, api::node::HoprSessionServer, builder, config, constants, errors, exports,
    mixer, utils,
};
