//! [`HoprSession`] object providing the session functionality over the HOPR transport
//!
//! The session proxies the user interactions with the transport to hide the
//! advanced interactions and functionality.
//!
//! The [`SessionManager`] allows for automatic management of sessions via the Start protocol.
//!
//! This crate implements [RFC-0007](https://github.com/hoprnet/rfc/tree/main/rfcs/RFC-0007-session-protocol).

pub(crate) mod balancer;
pub mod counters;
pub mod errors;
pub mod flow_control;
mod manager;
#[cfg(feature = "telemetry")]
mod telemetry;
mod types;
mod utils;

pub use balancer::{AtomicSurbFlowEstimator, BalancerStateValues, MIN_BALANCER_SAMPLING_INTERVAL, SurbBalancerConfig};
use hopr_api::types::internal::routing::RoutingOptions;
pub use hopr_protocol_session::{AcknowledgementMode, flow_control::FlowControlConfig};
pub use hopr_utils::network_types::types::*;
#[cfg(feature = "benchmark")]
pub use manager::SESSION_FORWARD_CAPACITY;
pub use manager::{DispatchResult, DropReason, MIN_SURB_BUFFER_DURATION, SessionManager, SessionManagerConfig};
#[cfg(feature = "telemetry")]
pub use telemetry::{SessionAckMode, SessionLifecycleState};
pub use types::{
    ByteCapabilities, HoprSession, HoprSessionConfig, HoprStartProtocol, IncomingSession, SESSION_APPLICATION_TAG,
    ServiceId, SessionId, SessionTarget,
};
#[cfg(feature = "runtime-tokio")]
pub use utils::transfer_session;

/// Number of bytes that can be sent in a single Session protocol payload.
///
/// In other words, this is the effective payload capacity of a single Session segment.
pub const SESSION_MTU: usize =
    hopr_protocol_session::session_socket_mtu::<{ hopr_protocol_app::v1::ApplicationData::PAYLOAD_SIZE }>();

/// Size of the HOPR SURB in bytes.
///
/// This is the re-export of [`hopr_crypto_packet::HoprSurb::SIZE`].
pub const SURB_SIZE: usize = hopr_crypto_packet::HoprSurb::SIZE;

flagset::flags! {
    /// Individual capabilities of a Session.
    #[repr(u8)]
    #[derive(PartialOrd, Ord, strum::EnumString, strum::Display, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
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
        /// UDP-like behavior: disable packet buffering, and — on stateless (non-retransmitting)
        /// sessions — preserve datagram boundaries by emitting each write as exactly one frame
        /// (delivered to the peer as exactly one read), regardless of `frame_mtu`. Use this for
        /// datagram-oriented targets (e.g. UDP/WireGuard) that must not have datagrams split or
        /// coalesced. On reliable (retransmitting) sessions only the buffering behavior applies;
        /// boundary preservation is stateless-only (the NACK missing-segment bitmap cannot address
        /// the segments of an oversized datagram frame).
        ///
        /// Boundary preservation holds only up to the reader's buffer: a single read cannot return
        /// more bytes than its buffer, so a datagram larger than the peer's read buffer is still
        /// delivered in multiple reads. Callers must therefore size the read buffer for the largest
        /// datagram they need preserved. In the UDP-forwarding path the datagram size is bounded at
        /// ingress by the forwarding buffer (`HOPR_UDP_BUFFER_SIZE`), so it never exceeds it there;
        /// a datagram-mode frame may otherwise be as large as `64 * SESSION_MTU`.
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
    #[default(RoutingOptions::Hops(hopr_api::types::primitive::bounded::BoundedSize::MIN))]
    pub forward_path_options: RoutingOptions,
    /// The return path options for the session.
    #[default(RoutingOptions::Hops(hopr_api::types::primitive::bounded::BoundedSize::MIN))]
    pub return_path_options: RoutingOptions,
    /// Capabilities offered by the session.
    #[default(_code = "Capability::Segmentation.into()")]
    pub capabilities: Capabilities,
    /// Optional pseudonym used for the session. Mostly useful for testing only.
    #[default(None)]
    pub pseudonym: Option<hopr_api::types::internal::protocol::HoprPseudonym>,
    /// Enable automatic SURB management for the Session.
    #[default(Some(SurbBalancerConfig::default()))]
    pub surb_management: Option<SurbBalancerConfig>,
    /// If set, the maximum number of possible SURBs will always be sent with Session data packets (if they fit).
    ///
    /// This does not affect `KeepAlive` messages used with SURB balancing, as they will always
    /// carry the maximum number of SURBs possible. Setting this to `true` will put additional CPU
    /// pressure on the local node as it will generate the maximum number of SURBs for each data packet.
    ///
    /// Set this to `true` only when the underlying traffic is highly asymmetric.
    ///
    /// Default is `false`.
    #[default(false)]
    pub always_max_out_surbs: bool,
    /// Opt-in client-side send-window flow control for this session.
    ///
    /// `None` (the default) leaves the session unpaced — today's behaviour. `Some(..)` enables the
    /// adaptive AIMD window on the entry (sending) side; use [`FlowControlConfig::default`] for the
    /// verified clean profile or [`FlowControlConfig::robust`] for the tail-tolerance bundle. This is
    /// the client's explicit dial (only meaningful on a reliable / `RetransmissionAck` session).
    #[default(None)]
    pub flow_control: Option<FlowControlConfig>,
    /// Abandon the frame due next once the sequence has advanced this far past it, instead of
    /// holding everything already received for the whole frame timeout.
    ///
    /// Head-of-line bound for this session's incoming direction. `None` inherits the node's
    /// setting, `Some(0)` disables it here, `Some(n)` sets it.
    ///
    /// Worth setting per session because the right value tracks reordering depth -- throughput x
    /// latency spread / frame size -- which is a property of the traffic, not of the node: a bulk
    /// data session and a control session on the same node differ by orders of magnitude.
    ///
    /// Has no effect on a session carrying a retransmission capability, where a missing frame can
    /// still be recovered and waiting for it is productive.
    pub max_frames_behind_gap: Option<usize>,
}

#[cfg(test)]
mod tests {
    use hopr_api::types::{crypto_random::Randomizable, internal::prelude::HoprPseudonym};
    use hopr_crypto_packet::prelude::HoprPacket;
    use hopr_protocol_app::v1::ApplicationData;
    use hopr_protocol_session::session_socket_mtu;
    use hopr_protocol_start::{
        KeepAliveMessage, StartChallenge, StartErrorReason, StartErrorType, StartEstablished, StartInitiation,
    };

    use super::*;
    use crate::types::HoprStartProtocol;

    #[test]
    fn test_session_mtu() {
        assert_eq!(SESSION_MTU, session_socket_mtu::<{ ApplicationData::PAYLOAD_SIZE }>());
        assert_eq!(1020, SESSION_MTU); // Needs to be changed when HOPR packet payload size changes
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
            session_id: HoprPseudonym::random(),
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
            session_id: HoprPseudonym::random(),
            flags: None.into(),
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
    fn hopr_start_protocol_message_session_initiation_message_should_allow_for_at_least_one_surb() -> anyhow::Result<()>
    {
        let msg = HoprStartProtocol::StartSession(StartInitiation {
            challenge: StartChallenge::MAX,
            target: SessionTarget::TcpStream(SealedHost::Plain(
                "example-of-a-very-very-long-second-level-name.on-a-very-very-long-domain-name.info:65530".parse()?,
            )),
            capabilities: Capabilities::full().into(),
            additional_data: 0xffffffff,
        });
        let len = msg.encode()?.1.len();
        assert!(
            HoprPacket::max_surbs_with_message(len) >= 1,
            "Hopr StartSession message size ({}) must allow for at least 1 SURB in packet",
            len
        );

        Ok(())
    }

    #[test]
    fn hopr_start_protocol_message_keep_alive_message_should_allow_for_maximum_surbs() -> anyhow::Result<()> {
        let msg = HoprStartProtocol::KeepAlive(KeepAliveMessage {
            session_id: HoprPseudonym::random(),
            flags: None.into(),
            additional_data: 0xffffffff,
        });
        let len = msg.encode()?.1.len();
        assert_eq!(
            KeepAliveMessage::<SessionId>::MIN_SURBS_PER_MESSAGE,
            HoprPacket::MAX_SURBS_IN_PACKET
        );
        assert!(
            HoprPacket::max_surbs_with_message(len) >= HoprPacket::MAX_SURBS_IN_PACKET,
            "Hopr KeepAlive message size ({}) must allow for at least {} SURBs in packet",
            len,
            HoprPacket::MAX_SURBS_IN_PACKET
        );

        Ok(())
    }
}
