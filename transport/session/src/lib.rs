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
pub use manager::{
    DEFAULT_MAX_SSAS_PER_SSA_REQUEST, DEFAULT_SSAS_PER_SSA_REQUEST, DEPOSIT_DATA_REQUEST_TIMEOUT, DispatchResult,
    DropReason, IncomingSessionPixConfig, MAX_SSA_BATCH_SIZE, MIN_SURB_BUFFER_DURATION, PixToolbox, SessionManager,
    SessionManagerConfig,
};
#[cfg(any(test, feature = "testing"))]
pub mod testing;
pub use hopr_api::types::internal::routing::DestinationRouting;
pub use hopr_protocol_app::prelude::{ApplicationDataIn, ApplicationDataOut};
pub use hopr_protocol_pix::{InvalidPixParams, PixParams};
#[cfg(feature = "telemetry")]
pub use telemetry::{SessionAckMode, SessionLifecycleState};
#[cfg(any(test, feature = "testing"))]
pub use testing::{MsgSender as MockMsgSender, SendMsg, mock_packet_planning, msg_type, start_msg_match};
pub use types::{
    AgreedSsaQuota, ClosureReason, DEFAULT_PIX_POLYS_PER_SSA, DEFAULT_PIX_SHARES_PER_POLY, DEFAULT_PIX_SSA_QUOTA,
    DEFAULT_PIX_SURPLUS_SHARES, HoprSession, HoprSessionCapabilities, HoprSessionConfig, HoprSessionInPixEvent,
    HoprSessionOutPixEvent, HoprStartProtocol, IncomingSession, LOCAL_PIX_SUITE, ServiceId, SessionId, SessionTarget,
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
        /// Disable packet buffering.
        ///
        /// Implies [`Segmentation`].
        NoDelay = 0b0000_1001,
        /// Disable SURB-based egress rate control.
        ///
        /// This applies only to the recipient of the Session (Exit).
        ///
        /// If not set, the lower half of additional data may contain information about the desired SURB buffer size.
        NoRateControl = 0b0001_0000,
        /// Indicates to the Session recipient (Exit) that this Session should use the PIX protocol.
        ///
        /// The upper half of additional data may be used to configure the PIX protocol parameters.
        UsePIX = 0b0010_0000,
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
    /// Sets the maximum number of possible SURBs which will always be sent with Session data packets (if they fit).
    ///
    /// This does not affect `KeepAlive` messages used with SURB balancing, as they will always
    /// carry the maximum number of SURBs possible. Setting this to a higher number will put additional CPU
    /// pressure on the local node as it will generate the given number of SURBs for each data packet.
    ///
    /// Set this to more than 1 only if the underlying traffic is highly asymmetric. Setting
    /// this to `0` will leave all the job of SURB production to the SURB balancer (if configured).
    ///
    /// The maximum number is naturally limited by the maximum payload size of a HOPR packet.
    ///
    /// Default is `1`.
    #[default(1)]
    pub max_surbs_per_data_packet: usize,
    /// PIX parameters for SSAs.
    ///
    /// When not set, the Session will not advertise any PIX capability and may
    /// get refused by the Exit (if it requires PIX).
    ///
    /// The Exit may also refuse to accept the Session if the given values
    /// evaluate to a PIX quota that is not within Exit's acceptable PIX quota range.
    ///
    /// These are not free parameters: the shares this node puts on the wire come from the installed
    /// [`SsaShareGenerator`](hopr_protocol_pix::SsaShareGenerator), so
    /// [`SessionManager::new_session`] refuses any value that disagrees with it rather than
    /// advertising dimensions it cannot honour. Setting this is therefore an assertion about the
    /// node's own PIX configuration — build it with
    /// [`PixParams::try_from_config`](hopr_protocol_pix::PixParams::try_from_config) over that
    /// generator's config if you do not want to restate it.
    ///
    /// The fourth component, the curve suite, is fixed by how this node was built rather than
    /// configured; [`LOCAL_PIX_SUITE`] names it for anyone restating the values by hand.
    ///
    /// Defaults to `None`.
    pub pix_ssa_quota: Option<PixParams>,
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
        ErrorIdentifier, KeepAliveMessage, StartChallenge, StartErrorReason, StartErrorType, StartEstablished,
        StartInitiation,
    };

    use super::*;
    use crate::types::HoprStartProtocol;

    #[test]
    fn test_session_mtu() {
        assert_eq!(SESSION_MTU, session_socket_mtu::<{ ApplicationData::PAYLOAD_SIZE }>());
        assert_eq!(1452, SESSION_MTU); // Needs to be changed when HOPR packet payload size changes
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
            identifier: ErrorIdentifier::Challenge(StartChallenge::MAX),
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
