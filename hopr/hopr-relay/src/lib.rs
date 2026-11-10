//! The HOPR **relay** entity.
//!
//! A pure relay forwards packets and runs the full ticket and acknowledgement path, and nothing
//! else: it is the `relay` feature set of [`hopr_lib`], which is what this crate is built with.
//! The Session stack — client API, Session server, incoming-Session admission, and the Session
//! manager's wiring — is behind a `cfg` in `hopr-lib` and `hopr-transport`, so a relay binary
//! constructs none of it; the `hopr-transport-session` crate those types come from is still
//! linked.
//!
//! The re-exports below are the whole surface a relay binary is expected to need. The role scoping
//! is not maintained by hand: [`builder`] only carries the build methods the compiled role has, and
//! the Session items are absent from the `hopr-lib` this facade re-exports.

// Ticket management is the relay's own responsibility, so its interface comes through the facade.
#[cfg(feature = "telemetry")]
pub use hopr_lib::collect_hopr_metrics;
#[cfg(feature = "runtime-tokio")]
pub use hopr_lib::prepare_tokio_runtime;
pub use hopr_lib::{
    Abortable, AbortableList, Hopr, HoprKeys, HoprTransportIO, IdentityRetrievalModes, MIN_NATIVE_BALANCE, SESSION_MTU,
    SUGGESTED_NATIVE_BALANCE, api,
    api::{
        node::{HoprNodeOperations, TicketEvent},
        tickets::{TicketFactory, TicketManagement},
    },
    builder, config, constants, errors, exports, mixer, utils,
};
