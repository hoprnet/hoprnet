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
pub(crate) mod exit_control;
pub mod flow_control;
mod manager;
/// Exit-side PIX supervision: the lifecycle state machine, the egress gate, and the per-session
/// worker that drives them.
pub(crate) mod supervision;
#[cfg(feature = "telemetry")]
mod telemetry;
mod types;
mod utils;

pub use balancer::{AtomicSurbFlowEstimator, BalancerStateValues, MIN_BALANCER_SAMPLING_INTERVAL, SurbBalancerConfig};
pub use exit_control::{
    DEFAULT_REFUSAL_REPLY_BURST, DEFAULT_REFUSAL_REPLY_REFILL_PERIOD, MIN_REFUSAL_REPLY_REFILL_PERIOD,
    RefusalReplyBudget,
};
use hopr_api::types::internal::routing::RoutingOptions;
pub use hopr_protocol_session::{AcknowledgementMode, flow_control::FlowControlConfig};
pub use hopr_utils::network_types::types::*;
pub use manager::{
    DEFAULT_MAX_SSAS_PER_SSA_REQUEST, DEFAULT_SSAS_PER_SSA_REQUEST, DEPOSIT_DATA_REQUEST_TIMEOUT, DispatchResult,
    DropReason, IncomingSessionPixConfig, MAX_ASSUMED_SESSION_PACKET_RATE, MAX_OVERLAPPING_BATCHES, MAX_SSA_BATCH_SIZE,
    MIN_SURB_BUFFER_DURATION, PixToolbox, START_SESSION_ADMISSION_CONCURRENCY, SessionManager, SessionManagerConfig,
    cycle_budget_for, max_cycle_budget_for_quota, validate_incoming_session_pix_config,
};
pub use supervision::{FillRate, PixFillConfig, SupervisorConfig, validate_pix_supervision};
/// The supervisor state machine and its event/action vocabulary, for `benches/supervisor_bench.rs`,
/// and the egress gate, for `benches/gate_bench.rs`.
///
/// Behind the same `benchmark` gate as
/// [`SessionManager::pre_populate_session`](crate::SessionManager::pre_populate_session), and for
/// the same reason: a criterion bench is a separate crate, so a path that is `pub(crate)` is a path
/// it cannot measure. Nothing here is part of the crate's supported surface.
#[cfg(any(feature = "benchmark", test))]
pub use supervision::{
    GateBlockReason, GateClosed, GateVerdict, ServiceGate, SessionPixAction, SessionPixEvent, SessionPixSupervisor,
};
#[cfg(any(test, feature = "testing"))]
pub mod testing;
pub use hopr_api::types::internal::routing::DestinationRouting;
pub use hopr_protocol_app::prelude::{ApplicationDataIn, ApplicationDataOut};
pub use hopr_protocol_pix::{InvalidPixParams, PixParams};
pub use hopr_protocol_start::StartErrorReason;
#[cfg(feature = "telemetry")]
pub use telemetry::{SessionAckMode, SessionLifecycleState};
#[cfg(any(test, feature = "testing"))]
pub use testing::{MsgSender as MockMsgSender, SendMsg, mock_packet_planning, msg_type, start_msg_match};
pub use types::{
    AdmissionToken, AgreedSsaQuota, ClosureReason, DEFAULT_PIX_POLYS_PER_SSA, DEFAULT_PIX_SHARES_PER_POLY,
    DEFAULT_PIX_SSA_QUOTA, DEFAULT_PIX_SURPLUS_SHARES, HoprSession, HoprSessionCapabilities, HoprSessionConfig,
    HoprSessionInPixEvent, HoprSessionOutPixEvent, HoprStartProtocol, IncomingSession, LOCAL_PIX_SUITE,
    PIX_QUOTA_BYTES_PER_SHARE, ServiceId, SessionAdmissionReply, SessionAdmissionSink, SessionId, SessionTarget,
    SurbAdmissionControl,
};
#[cfg(feature = "runtime-tokio")]
pub use utils::{transfer_session, transfer_session_datagram};

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
    ///
    /// A recipient ignores capability bits it does not know, so a capability added later must be safe to ignore.
    /// A capability that the recipient must understand needs a Start protocol version bump.
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
        ///
        /// If not set, the lower half of additional data may contain information about the desired SURB buffer size.
        NoRateControl = 0b0001_0000,
        /// Indicates to the Session recipient (Exit) that this Session should use the PIX protocol.
        ///
        /// The Exit may still refuse the Session if the offered quota
        /// (from this node's  [`SsaShareGenerator`](hopr_protocol_pix::SsaShareGenerator)) falls
        /// outside its acceptable range, or (with `enforce_pix`) if this flag is *absent*.
        UsePIX = 0b0010_0000,
        /// Announces that the Entry can resume this Session later, i.e. it understands `SessionResumed`, stream
        /// bases and errors addressed by challenge.
        ///
        /// The session manager sets it on every PIX Session. An Exit that predates it ignores it, because
        /// unknown capability bits are ignored since hoprnet#8736.
        Resumable = 0b0100_0000,
    }
}

/// Set of Session [capabilities](Capability).
pub type Capabilities = flagset::FlagSet<Capability>;

/// Default maximum number of SURBs piggybacked on outgoing Session data packets.
///
/// Matches [`hopr_crypto_packet::prelude::HoprPacket::MAX_SURBS_IN_PACKET`]; every finite value
/// (`< usize::MAX`) is bounded by the remaining payload space of the packet beside application
/// data and (when SURB balancing is enabled) by the balancer's remaining SURB deficit.
pub const DEFAULT_MAX_SURBS_PER_DATA_PACKET: usize = hopr_crypto_packet::prelude::HoprPacket::MAX_SURBS_IN_PACKET;

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
    /// Optional pseudonym used for the session. Random when not given.
    ///
    /// Giving one is how a client reconnects under a pseudonym it used before in this process. A live local
    /// Session under it is closed and the new one takes its place. For a PIX Session the reconnect either resumes
    /// the Session the Exit holds, continuing its accounting and numbering the new socket's stream from the base
    /// the Exit chose, or is refused with `NotResumable`. After a refusal the client must use a fresh pseudonym.
    ///
    /// Only stateless Sessions (frame-based, no retransmission) can be resumed; the reconnect of a stateful one is
    /// refused with `NotResumable`. The Entry keeps a pseudonym's PIX accounting for 30 minutes after its Session
    /// ends (a live Session never loses it), so a resume is possible only within that time. Its `StartSession`
    /// challenges strictly increase, and an Exit takes a Session over only for a challenge greater than the one it
    /// was established by, so a delayed attempt cannot take over a newer Session. Who may take a Session over is
    /// decided by the pseudonym alone; see hoprnet#8723 for binding it to a secret.
    ///
    /// Reusing a pseudonym across process restarts is **not** supported: the share generator, the reply
    /// openers and the SURB generation are all held in memory.
    #[default(None)]
    pub pseudonym: Option<hopr_api::types::internal::protocol::HoprPseudonym>,
    /// Enable automatic SURB management for the Session.
    #[default(Some(SurbBalancerConfig::default()))]
    pub surb_management: Option<SurbBalancerConfig>,
    /// Sets the maximum number of SURBs sent with Session data packets (if they fit).
    ///
    /// Balancer-driven `KeepAlive` messages share the same remaining SURB deficit with outgoing
    /// data packets and are capped to a partial grant (or `0` refill SURBs when data packets have
    /// already filled the effective target) while still forwarding control flags. Setting this to
    /// a higher number will put additional CPU pressure on the local node as it will generate more
    /// SURBs per data packet while below target.
    ///
    /// When SURB balancing is enabled (`surb_management` is `Some` with a non-zero
    /// `target_surb_buffer_size`), every finite value follows the balancer's target: data packets
    /// carry no SURBs once the counterparty is estimated to have reached its effective target
    /// (except a 1-SURB/packet floor while the counterparty signals distress or during silence
    /// sustain), and up to `min(max_surbs_per_data_packet, remaining_deficit)` while it is below
    /// target, counted down per packet within each balancer tick so bursts cannot overflow the
    /// counterparty's store. When SURB balancing is disabled (`surb_management` is `None` or has
    /// `target_surb_buffer_size == 0`), the explicit finite limit applies verbatim to each data
    /// packet (bounded by remaining packet payload space); because [`SessionClientConfig::default`]
    /// sets this field to `8`, callers disabling balancing via `..Default::default()` inherit up to
    /// 8 SURBs per data packet with no balancer bound and should set `max_surbs_per_data_packet`
    /// explicitly (for example, `1`) when a lower unbalanced cap is desired. Setting this to `0`
    /// leaves all SURB production to the SURB balancer (if configured). `usize::MAX` bypasses both
    /// the balancer gate and per-packet cap: every packet carries as many SURBs as fit, whatever
    /// the balancer estimates.
    ///
    /// The maximum number is naturally limited by the remaining payload space of the HOPR packet
    /// beside the packet's own application data.
    ///
    /// Default is [`DEFAULT_MAX_SURBS_PER_DATA_PACKET`] (8).
    #[default(DEFAULT_MAX_SURBS_PER_DATA_PACKET)]
    pub max_surbs_per_data_packet: usize,
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
        assert_eq!(1452, SESSION_MTU); // Needs to be changed when the Session MTU cap changes
    }

    #[test]
    fn start_session_with_unknown_capability_bits_decodes_with_them_dropped() -> anyhow::Result<()> {
        let init = StartInitiation {
            challenge: 0x0102_0304_0506_0708,
            target: SessionTarget::TcpStream(SealedHost::Plain("example.com:80".parse()?)),
            capabilities: HoprSessionCapabilities::from(Capability::Segmentation | Capability::UsePIX),
            additional_data: 0xdead_beef_u128,
        };
        let known_byte = u8::from(init.capabilities);

        let (tag, mut bytes) = HoprStartProtocol::StartSession(init.clone()).encode()?;

        // Header (version, discriminant, u16 length) is followed by the challenge, then the capability byte
        let offset = 4 + size_of::<StartChallenge>();
        assert_eq!(bytes[offset], known_byte);

        let unknown = !Capabilities::full().bits();
        assert_ne!(unknown, 0, "all capability bits are now in use; revisit this test");
        bytes[offset] |= unknown;
        let decoded = HoprStartProtocol::decode(tag, &bytes)?;
        assert_eq!(decoded, HoprStartProtocol::StartSession(init));
        Ok(())
    }

    #[test]
    fn hopr_start_protocol_messages_must_fit_within_hopr_packet() -> anyhow::Result<()> {
        let msg = HoprStartProtocol::StartSession(StartInitiation {
            challenge: StartChallenge::MAX,
            target: SessionTarget::TcpStream(SealedHost::Plain(
                "example-of-a-very-very-long-second-level-name.on-a-very-very-long-domain-name.info:65530".parse()?,
            )),
            capabilities: Capabilities::full().into(),
            additional_data: (u64::MAX as u128) << u64::BITS,
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
            additional_data: (u64::MAX as u128) << u64::BITS,
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
        let keep_alive = KeepAliveMessage {
            session_id: HoprPseudonym::random(),
            flags: None.into(),
            additional_data: 0xffffffff,
        };
        let capacity = keep_alive.max_surbs()?;
        let data = ApplicationData::try_from(HoprStartProtocol::KeepAlive(keep_alive))?;

        // Eight generation-tagged SURBs fit in an empty packet, but the keep-alive and its tag
        // together need one byte more than the remaining space.
        assert_eq!(capacity, 7);
        assert_eq!(capacity, HoprPacket::max_surbs_with_message(data.total_len()));

        Ok(())
    }
}
