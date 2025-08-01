//! [`Session`] object providing the session functionality over the HOPR transport
//!
//! The session proxies the user interactions with the transport to hide the
//! advanced interactions and functionality.
//!
//! The [`SessionManager`] allows for automatic management of sessions via the Start protocol.
//!
//! This crate implements [RFC-0007](https://github.com/hoprnet/rfc/tree/main/rfcs/RFC-0007-session-protocol).
pub(crate) mod balancer;
pub mod errors;
mod initiation;
mod manager;
mod types;
mod utils;

pub use balancer::SurbBalancerConfig;
use hopr_internal_types::prelude::HoprPseudonym;
pub use hopr_network_types::types::*;
use hopr_protocol_session::session_socket_mtu;
use hopr_transport_packet::prelude::ApplicationData;
pub use manager::{DispatchResult, MIN_BALANCER_SAMPLING_INTERVAL, SessionManager, SessionManagerConfig};
pub use types::{IncomingSession, ServiceId, Session, SessionId, SessionTarget};
#[cfg(feature = "runtime-tokio")]
pub use utils::transfer_session;

// TODO: resolve this in #7145
#[cfg(not(feature = "serde"))]
compile_error!("The `serde` feature currently must be enabled, see #7145");

/// Number of bytes that can be sent in a single Session protocol payload.
pub const SESSION_MTU: usize = session_socket_mtu::<{ ApplicationData::PAYLOAD_SIZE }>();

flagset::flags! {
    /// Individual capabilities of a Session.
    #[repr(u8)]
    #[derive(strum::EnumString, strum::Display)]
    #[cfg_attr(feature = "serde", derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr))]
    pub enum Capability : u8 {
        /// Frame segmentation.
        Segmentation = 0b1000,
        /// Frame retransmission (ACK-based)
        ///
        /// Implies [`Segmentation`].
        RetransmissionAck = 0b1100,
        /// Frame retransmission (NACK-based)
        ///
        /// Implies [`Segmentation`].
        RetransmissionNack = 0b1010,
        /// Disable packet buffering.
        ///
        /// Implies [`Segmentation`].
        NoDelay = 0b1001,
    }
}

/// Set of Session [capabilities](Capability).
pub type Capabilities = flagset::FlagSet<Capability>;

/// Configuration for the session.
///
/// Relevant primarily for the client, since the server is only
/// a reactive component in regard to the session concept.
#[derive(Debug, PartialEq, Clone, smart_default::SmartDefault)]
pub struct SessionClientConfig {
    /// The forward path options for the session.
    #[default(RoutingOptions::Hops(hopr_primitive_types::bounded::BoundedSize::MIN))]
    pub forward_path_options: RoutingOptions,
    /// The return path options for the session.
    #[default(RoutingOptions::Hops(hopr_primitive_types::bounded::BoundedSize::MIN))]
    pub return_path_options: RoutingOptions,
    /// Capabilities offered by the session.
    #[default(_code = "Capability::Segmentation.into()")]
    pub capabilities: Capabilities,
    /// Optional pseudonym used for the session. Mostly useful for testing only.
    #[default(None)]
    pub pseudonym: Option<HoprPseudonym>,
    /// Enable automatic SURB management for the Session.
    #[default(Some(SurbBalancerConfig::default()))]
    pub surb_management: Option<SurbBalancerConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_mtu() {
        assert_eq!(SESSION_MTU, session_socket_mtu::<{ ApplicationData::PAYLOAD_SIZE }>());
        assert_eq!(1002, SESSION_MTU);
    }
}
