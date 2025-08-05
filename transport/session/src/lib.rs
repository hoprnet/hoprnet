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
mod manager;
mod types;
mod utils;

pub use balancer::SurbBalancerConfig;
pub use hopr_network_types::types::*;
pub use manager::{DispatchResult, MIN_BALANCER_SAMPLING_INTERVAL, SessionManager, SessionManagerConfig};
pub use types::{ByteCapabilities, IncomingSession, ServiceId, Session, SessionId, SessionTarget};
#[cfg(feature = "runtime-tokio")]
pub use utils::transfer_session;

/// Number of bytes that can be sent in a single Session protocol payload.
pub const SESSION_MTU: usize =
    hopr_protocol_session::session_socket_mtu::<{ hopr_transport_packet::v1::ApplicationData::PAYLOAD_SIZE }>();

/// Size of the HOPR SURB in bytes.
///
/// This is the re-export of [`hopr_crypto_packet::HoprSurb::SIZE`].
pub const SURB_SIZE: usize = hopr_crypto_packet::HoprSurb::SIZE;

flagset::flags! {
    /// Individual capabilities of a Session.
    #[repr(u8)]
    #[derive(strum::EnumString, strum::Display, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
    pub enum Capability : u8 {
        /// Frame segmentation.
        Segmentation = 0b0000_1000,
        /// Frame retransmission (ACK-based)
        ///
        /// Implies [`Segmentation`].
        RetransmissionAck = 0b0000_1100,
        /// Frame retransmission (NACK-based)
        ///
        /// Implies [`Segmentation`].
        RetransmissionNack = 0b000_1010,
        /// Disable packet buffering.
        ///
        /// Implies [`Segmentation`].
        NoDelay = 0b0000_1001,
        /// Disable SURB-based egress rate control.
        ///
        /// This applies only to the recipient of the Session (Exit).
        NoRateControl = 0b0001_0000,
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
    pub pseudonym: Option<hopr_internal_types::protocol::HoprPseudonym>,
    /// Enable automatic SURB management for the Session.
    #[default(Some(SurbBalancerConfig::default()))]
    pub surb_management: Option<SurbBalancerConfig>,
}

#[cfg(test)]
mod tests {
    use hopr_crypto_packet::prelude::HoprPacket;
    use hopr_crypto_random::Randomizable;
    use hopr_internal_types::prelude::HoprPseudonym;
    use hopr_protocol_session::session_socket_mtu;
    use hopr_protocol_start::{
        KeepAliveMessage, StartChallenge, StartErrorReason, StartErrorType, StartEstablished, StartInitiation,
    };
    use hopr_transport_packet::v1::ApplicationData;

    use super::*;
    use crate::types::HoprStartProtocol;

    #[test]
    fn test_session_mtu() {
        assert_eq!(SESSION_MTU, session_socket_mtu::<{ ApplicationData::PAYLOAD_SIZE }>());
        assert_eq!(1002, SESSION_MTU);
    }

    #[test]
    fn hopr_start_protocol_messages_must_fit_within_hopr_packet() -> anyhow::Result<()> {
        let msg = HoprStartProtocol::StartSession(StartInitiation {
            challenge: StartChallenge::MAX,
            target: SessionTarget::TcpStream(SealedHost::Plain(
                "example-of-a-very-very-long-second-level-name.on-a-very-very-long-domain-name.info:65530".parse()?,
            )),
            capabilities: Capabilities::full().into(),
            additional_data: 0xffffffff,
        });

        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "StartSession must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        let msg = HoprStartProtocol::SessionEstablished(StartEstablished {
            orig_challenge: StartChallenge::MAX,
            session_id: SessionId::new(u64::MAX, HoprPseudonym::random()),
        });

        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "SessionEstablished must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        let msg = HoprStartProtocol::SessionError(StartErrorType {
            challenge: StartChallenge::MAX,
            reason: StartErrorReason::NoSlotsAvailable,
        });

        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "SessionError must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        let msg = HoprStartProtocol::KeepAlive(KeepAliveMessage {
            session_id: SessionId::new(u64::MAX, HoprPseudonym::random()),
            flags: 0xff,
            additional_data: 0xffffffff,
        });
        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "KeepAlive must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        Ok(())
    }

    #[test]
    fn hopr_start_protocol_message_keep_alive_message_should_allow_for_maximum_surbs() -> anyhow::Result<()> {
        let msg = HoprStartProtocol::KeepAlive(KeepAliveMessage {
            session_id: SessionId::new(u64::MAX, HoprPseudonym::random()),
            flags: 0xff,
            additional_data: 0xffffffff,
        });
        let len = msg.encode()?.1.len();
        assert!(
            HoprPacket::max_surbs_with_message(len) >= HoprPacket::MAX_SURBS_IN_PACKET,
            "Hopr KeepAlive message size ({}) must allow for at least {} SURBs in packet",
            len,
            HoprPacket::MAX_SURBS_IN_PACKET
        );

        Ok(())
    }
}
