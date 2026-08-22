use std::{
    convert::Into,
    fmt::{Debug, Formatter},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use futures::{SinkExt, StreamExt, TryStreamExt};
use hopr_api::types::{internal::routing::DestinationRouting, primitive::errors::GeneralError};
use hopr_protocol_app::prelude::{ApplicationData, ApplicationDataIn, ApplicationDataOut, ReservedTag, Tag};
#[cfg(feature = "telemetry")]
use hopr_protocol_session::NoopTracker;
use hopr_protocol_session::{
    AcknowledgementMode, AcknowledgementState, AcknowledgementStateConfig, ReliableSocket, SessionSocketConfig,
    UnreliableSocket,
    flow_control::{DeliveryClock, DeliveryMeter, DeliveryTap, FlowControlConfig},
};
use hopr_protocol_start::StartProtocol;
use hopr_utils::network_types::utils::{AsyncWriteSink, DuplexIO};
use tracing::{debug, instrument};

use crate::{
    Capabilities, Capability,
    balancer::BalancerStateValues,
    errors::TransportSessionError,
    flow_control::{PacedWriter, SurbSupply},
};

/// Wrapper for [`Capabilities`] that makes conversion to/from `u8` possible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ByteCapabilities(pub Capabilities);

impl TryFrom<u8> for ByteCapabilities {
    type Error = GeneralError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Capabilities::new(value)
            .map(Self)
            .map_err(|_| GeneralError::ParseError("capabilities".into()))
    }
}

impl From<ByteCapabilities> for u8 {
    fn from(value: ByteCapabilities) -> Self {
        *value.0.as_ref()
    }
}

impl From<ByteCapabilities> for Capabilities {
    fn from(value: ByteCapabilities) -> Self {
        value.0
    }
}

impl From<Capabilities> for ByteCapabilities {
    fn from(value: Capabilities) -> Self {
        Self(value)
    }
}

impl AsRef<Capabilities> for ByteCapabilities {
    fn as_ref(&self) -> &Capabilities {
        &self.0
    }
}

/// Start protocol instantiation for HOPR.
pub type HoprStartProtocol = StartProtocol<SessionId, SessionTarget, ByteCapabilities>;

/// Constant application tag used for all sessions.
/// Previously tags were dynamically allocated per session.
pub const SESSION_APPLICATION_TAG: Tag = Tag::Reserved(ReservedTag::Session as u64);

/// [`SessionId`], [`ServiceId`], and [`SessionTarget`] are provided by `hopr-types` and
/// re-exported here via `hopr-utils`, so they match the published `hopr-api` session types.
///
/// - `SessionId` is a type alias for `HoprPseudonym` (a constant application tag is used for all sessions instead
///   of dynamically allocating tags).
/// - `ServiceId` identifies a service local to the Exit node (e.g. Cover Traffic).
/// - `SessionTarget` describes where data received over the session is forwarded.
pub use hopr_utils::network_types::types::{ServiceId, SessionId, SessionTarget};

pub(crate) fn caps_to_ack_mode(caps: Capabilities) -> AcknowledgementMode {
    if caps.contains(Capability::RetransmissionAck | Capability::RetransmissionNack) {
        AcknowledgementMode::Both
    } else if caps.contains(Capability::RetransmissionAck) {
        AcknowledgementMode::Full
    } else {
        AcknowledgementMode::Partial
    }
}

/// Indicates the closure reason of a [`HoprSession`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display)]
pub enum ClosureReason {
    /// Write-half of the Session has been closed.
    WriteClosed,
    /// Read-part of the Session has been closed (encountered empty read).
    EmptyRead,
    /// Session has been evicted from the cache due to inactivity or capacity reasons.
    Eviction,
}

/// Helper trait to allow Box aliasing
trait AsyncReadWrite: futures::AsyncWrite + futures::AsyncRead + Send + Unpin {}
impl<T: futures::AsyncWrite + futures::AsyncRead + Send + Unpin> AsyncReadWrite for T {}

/// Wrapper for an incoming [`HoprSession`] carrying the [`SessionId`] and [`SessionTarget`]
/// extracted from the Start protocol during session establishment.
///
/// This is the published generic [`hopr_api::node::IncomingSession`] specialized to the
/// concrete [`HoprSession`] byte-stream.
pub type IncomingSession = hopr_api::node::IncomingSession<HoprSession>;

/// Configures the Session protocol socket over HOPR.
#[derive(Copy, Clone, Debug, PartialEq, Eq, smart_default::SmartDefault, serde::Serialize)]
pub struct HoprSessionConfig {
    /// Capabilities of the Session protocol socket.
    ///
    /// Default is no capabilities.
    #[default(Capabilities::empty())]
    pub capabilities: Capabilities,
    /// Expected frame size of the Session protocol socket.
    ///
    /// Default is 1500.
    #[default(1500)]
    pub frame_mtu: usize,
    /// Maximum amount of time an incomplete frame can be kept in the buffer.
    ///
    /// Default is 800 ms
    #[default(Duration::from_millis(800))]
    #[serde(with = "humantime_serde")]
    pub frame_timeout: Duration,
    /// Maximum number of segments to buffer in the downstream transport.
    /// If 0 is given, the transport is unbuffered.
    ///
    /// Default is 0.
    #[default(0)]
    pub max_buffered_segments: usize,
    /// Abandon the frame due next once this many later frames are waiting behind it.
    ///
    /// Head-of-line bound, distinct from [`Self::frame_timeout`]: that one waits for a frame that
    /// may still arrive, this bounds how much already-received data is held while it waits. `None`
    /// keeps the timeout as the only rule.
    pub max_frames_behind_gap: Option<usize>,
}

/// Represents the Session protocol socket over HOPR.
///
/// This is essentially a HOPR-specific wrapper for [`ReliableSocket`] and [`UnreliableSocket`]
/// Session protocol sockets.
#[pin_project::pin_project]
pub struct HoprSession {
    id: SessionId,
    #[pin]
    inner: Box<dyn AsyncReadWrite>,
    routing: DestinationRouting,
    cfg: HoprSessionConfig,
    on_close: Option<Box<dyn FnOnce(SessionId, ClosureReason) + Send + Sync>>,
}

pub(crate) const SESSION_SOCKET_CAPACITY: usize = 16384;

impl HoprSession {
    /// Creates a new HOPR Session.
    ///
    /// It builds an [`futures::io::AsyncRead`] + [`futures::io::AsyncWrite`] transport
    /// from the given `hopr` interface and passing it to the appropriate [`UnreliableSocket`] or [`ReliableSocket`]
    /// based on the given `capabilities`.
    ///
    /// The `on_close` closure can be optionally called when the Session has been closed via `poll_close`.
    #[tracing::instrument(skip_all, fields(id, routing, cfg, session_id = %id))]
    pub fn new<Tx, Rx>(
        id: SessionId,
        routing: DestinationRouting,
        cfg: HoprSessionConfig,
        hopr: (Tx, Rx),
        on_close: Option<Box<dyn FnOnce(SessionId, ClosureReason) + Send + Sync>>,
    ) -> Result<Self, TransportSessionError>
    where
        Tx: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Send + Unpin + 'static,
        Rx: futures::Stream<Item = ApplicationDataIn> + Send + Unpin + 'static,
        Tx::Error: std::error::Error + Send + Sync,
    {
        Self::new_with_surb_state(id, routing, cfg, hopr, on_close, None, None)
    }

    /// Like [`new`](Self::new) but threads the SURB balancer state and the opt-in client-side
    /// flow-control config. `flow_control` = the client's [`FlowControlConfig`] for this session
    /// (`None` leaves it unpaced); `surb_mgmt` gives the window its anti-grief down-only SURB ceiling.
    /// The entry (sending) side passes `Some(..)`; sites without them pass `None`.
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip_all, fields(id, routing, cfg, session_id = %id))]
    pub fn new_with_surb_state<Tx, Rx>(
        id: SessionId,
        routing: DestinationRouting,
        cfg: HoprSessionConfig,
        hopr: (Tx, Rx),
        on_close: Option<Box<dyn FnOnce(SessionId, ClosureReason) + Send + Sync>>,
        surb_mgmt: Option<Arc<BalancerStateValues>>,
        flow_control: Option<FlowControlConfig>,
    ) -> Result<Self, TransportSessionError>
    where
        Tx: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Send + Unpin + 'static,
        Rx: futures::Stream<Item = ApplicationDataIn> + Send + Unpin + 'static,
        Tx::Error: std::error::Error + Send + Sync,
    {
        let routing_clone = routing.clone();

        #[cfg(feature = "telemetry")]
        let (session_id_write, session_id_read) = (id, id);

        // Wrap the HOPR transport so that it appears as regular transport to the SessionSocket
        let transport = DuplexIO(
            AsyncWriteSink::<{ ApplicationData::PAYLOAD_SIZE }, _>(hopr.0.sink_map_err(std::io::Error::other).with(
                move |buf: Box<[u8]>| {
                    #[cfg(feature = "telemetry")]
                    crate::telemetry::record_session_write(&session_id_write, buf.len());
                    // The Session protocol does not set any packet info on outgoing packets.
                    // However, the SessionManager on top usually overrides this.
                    futures::future::ready(
                        ApplicationData::new(SESSION_APPLICATION_TAG, buf.into_vec())
                            .map(|data| (routing_clone.clone(), ApplicationDataOut::with_no_packet_info(data)))
                            .map_err(std::io::Error::other),
                    )
                },
            )),
            // The Session protocol ignores the packet info on incoming packets.
            // It is typically SessionManager's job to interpret those.
            hopr.1
                .map(move |data| {
                    #[cfg(feature = "telemetry")]
                    crate::telemetry::record_session_read(&session_id_read, data.data.plain_text.len());
                    Ok::<_, std::io::Error>(data.data.plain_text)
                })
                .into_async_read(),
        );

        // Based on the requested capabilities, see if we should use the Session protocol
        let inner: Box<dyn AsyncReadWrite> = if cfg.capabilities.contains(Capability::Segmentation) {
            // Datagram mode emits one frame per write with no `frame_size` cap; the reliable socket
            // cannot honor that. Its NACK missing-segment bitmap addresses at most
            // `MAX_MISSING_SEGMENTS_PER_FRAME` segments per frame, so any segment past that bound in
            // an oversized datagram frame could never be requested for retransmission — on loss the
            // frame never completes and the stream stalls. Reject the contradictory combination up
            // front instead of silently corrupting a reliable stream.
            if cfg.capabilities.contains(Capability::Datagram)
                && (cfg.capabilities.contains(Capability::RetransmissionAck)
                    || cfg.capabilities.contains(Capability::RetransmissionNack))
            {
                return Err(TransportSessionError::DatagramRequiresStateless);
            }

            let socket_cfg = SessionSocketConfig {
                frame_size: cfg.frame_mtu,
                frame_timeout: cfg.frame_timeout,
                capacity: SESSION_SOCKET_CAPACITY,
                flush_immediately: cfg.capabilities.contains(Capability::NoDelay),
                datagram: cfg.capabilities.contains(Capability::Datagram),
                max_buffered_segments: cfg.max_buffered_segments,
                // Anti-bufferbloat bound; only meaningful when flow control is enabled, which is
                // also where the honest clock that observes the resulting loss lives.
                max_frame_age: flow_control.and_then(|c| c.max_frame_age),
                // Head-of-line bound, and deliberately *not* gated on flow control the way
                // `max_frame_age` is. The reasoning there runs backwards for this one: a session
                // that can retransmit may still recover a missing frame, so waiting is
                // productive, while a session without retransmission is waiting for something
                // that is never coming and holds everything already received behind it. The
                // sessions that need this bound most are exactly the ones flow control excludes.
                max_frames_behind_gap: cfg.max_frames_behind_gap,
                ..Default::default()
            };

            // Need to test the capabilities separately, because any Retransmission capability
            // implies Segmentation, and therefore `is_disjoint` would fail
            if cfg.capabilities.contains(Capability::RetransmissionAck)
                || cfg.capabilities.contains(Capability::RetransmissionNack)
            {
                let fc = flow_control;

                // TODO: update config values
                let ack_cfg = AcknowledgementStateConfig {
                    // This is a very coarse assumption, that a single 3-hop packet
                    // takes on average 200 ms to deliver.
                    // We can no longer base this timeout on the number of hops because
                    // it is not known for SURB-based routing.
                    expected_packet_latency: Duration::from_millis(200),
                    mode: caps_to_ack_mode(cfg.capabilities),
                    backoff_base: 0.2,
                    max_incoming_frame_retries: 1,
                    // Under flow control the sender is paced to the SURB drain rate, so an un-acked
                    // frame is usually just a *delayed* ack on a temporarily-starved return path, not
                    // a genuine loss. The retry budget is a flow-control config knob (`frame_retries`,
                    // default 2 = original); a robust profile raises it so delayed frames recover
                    // instead of being abandoned (an abandoned frame leaves a gap → stream corruption).
                    // `.max(1)`: never drop the retry budget to 0 — an abandoned frame under
                    // reliable-mode flow control leaves a gap and corrupts the stream.
                    max_outgoing_frame_retries: fc.map(|c| c.frame_retries.max(1) as usize).unwrap_or(2),
                    // Retire an outgoing frame that is already too stale to be worth delivering,
                    // rather than spending the remaining retry budget on it.
                    max_frame_age: fc.and_then(|c| c.max_frame_age),
                    ..Default::default()
                };

                debug!(
                    ?socket_cfg,
                    ?ack_cfg,
                    flow_control = fc.is_some(),
                    "opening new stateful session socket"
                );

                // Opt-in client-side flow control: when enabled, install the honest-clock tap on the
                // ack state and keep the paired clock to drive the paced writer.
                let (ack_state, flow_control) = match fc {
                    Some(fc_cfg) => {
                        let meter = DeliveryMeter::default();
                        let ack_state = AcknowledgementState::<{ ApplicationData::PAYLOAD_SIZE }>::new(id, ack_cfg)
                            .with_delivery_tap(DeliveryTap::new(meter.clone(), cfg.frame_mtu));
                        let clock = DeliveryClock::new(meter, Some(ack_cfg.expected_packet_latency));
                        (ack_state, Some((fc_cfg, clock)))
                    }
                    None => (
                        AcknowledgementState::<{ ApplicationData::PAYLOAD_SIZE }>::new(id, ack_cfg),
                        None,
                    ),
                };

                let socket = ReliableSocket::new(
                    transport,
                    ack_state,
                    socket_cfg,
                    #[cfg(feature = "telemetry")]
                    NoopTracker,
                )?;

                match flow_control {
                    Some((fc_cfg, clock)) => {
                        let surb_state = surb_mgmt
                            .clone()
                            .unwrap_or_else(|| Arc::new(BalancerStateValues::default()));
                        let supply = SurbSupply::new(surb_state, cfg.frame_mtu);
                        debug!(?fc_cfg, "wrapping session socket with paced flow-control writer");
                        Box::new(PacedWriter::new(socket, fc_cfg, clock, supply))
                    }
                    None => Box::new(socket),
                }
            } else {
                debug!(?socket_cfg, "opening new stateless session socket");

                Box::new(UnreliableSocket::<{ ApplicationData::PAYLOAD_SIZE }>::new_stateless(
                    id,
                    transport,
                    socket_cfg,
                    #[cfg(feature = "telemetry")]
                    NoopTracker,
                )?)
            }
        } else {
            debug!("opening raw session socket");
            Box::new(transport)
        };

        Ok(Self {
            id,
            inner,
            routing,
            cfg,
            on_close,
        })
    }

    /// ID of this Session.
    pub fn id(&self) -> &SessionId {
        &self.id
    }

    /// Routing options used to deliver data.
    pub fn routing(&self) -> &DestinationRouting {
        &self.routing
    }

    /// Configuration of this Session.
    pub fn config(&self) -> &HoprSessionConfig {
        &self.cfg
    }
}

impl std::fmt::Debug for HoprSession {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("id", &self.id)
            .field("routing", &self.routing)
            .finish_non_exhaustive()
    }
}

impl futures::AsyncRead for HoprSession {
    #[instrument(name = "Session::poll_read", level = "trace", skip_all, fields(session_id = %self.id), ret)]
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        let this = self.project();
        let read = futures::ready!(this.inner.poll_read(cx, buf))?;
        if read == 0 {
            tracing::trace!("hopr session empty read");
            // Empty read signals end of the socket, notify if needed
            if let Some(notifier) = this.on_close.take() {
                tracing::trace!("notifying read half closure of session");
                notifier(*this.id, ClosureReason::EmptyRead);
            }
        }
        Poll::Ready(Ok(read))
    }
}

impl futures::AsyncWrite for HoprSession {
    #[instrument(name = "Session::poll_write", level = "trace", skip_all, fields(session_id = %self.id), ret)]
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        self.project().inner.poll_write(cx, buf)
    }

    #[instrument(name = "Session::poll_flush", level = "trace", skip_all, fields(session_id = %self.id), ret)]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.project().inner.poll_flush(cx)
    }

    #[instrument(name = "Session::poll_close", level = "trace", skip_all, fields(session_id = %self.id), ret)]
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let this = self.project();
        futures::ready!(this.inner.poll_close(cx))?;
        tracing::trace!("hopr session closed");

        #[cfg(feature = "telemetry")]
        crate::telemetry::set_session_state(this.id, crate::telemetry::SessionLifecycleState::Closing);

        if let Some(notifier) = this.on_close.take() {
            tracing::trace!("notifying write half closure of session");
            notifier(*this.id, ClosureReason::WriteClosed);
        }

        Poll::Ready(Ok(()))
    }
}

#[cfg(feature = "runtime-tokio")]
impl tokio::io::AsyncRead for HoprSession {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let slice = buf.initialize_unfilled();
        let n = std::task::ready!(futures::AsyncRead::poll_read(self.as_mut(), cx, slice))?;
        buf.advance(n);
        Poll::Ready(Ok(()))
    }
}

#[cfg(feature = "runtime-tokio")]
impl tokio::io::AsyncWrite for HoprSession {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, std::io::Error>> {
        futures::AsyncWrite::poll_write(self.as_mut(), cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        futures::AsyncWrite::poll_flush(self.as_mut(), cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        futures::AsyncWrite::poll_close(self.as_mut(), cx)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use futures::{AsyncReadExt, AsyncWriteExt};
    use hopr_api::types::{
        crypto::prelude::*,
        crypto_random::Randomizable,
        internal::{prelude::HoprPseudonym, routing::RoutingOptions},
        primitive::prelude::*,
    };
    use hopr_utils::network_types::prelude::SealedHost;

    use super::*;

    // --- ByteCapabilities tests ---

    #[test]
    fn byte_capabilities_roundtrip_via_u8() -> anyhow::Result<()> {
        let flags: Capabilities = Capability::Segmentation.into();
        let caps = ByteCapabilities::from(flags);
        let byte_val: u8 = caps.into();
        let restored = ByteCapabilities::try_from(byte_val)?;
        assert_eq!(caps, restored);
        Ok(())
    }

    #[test]
    fn byte_capabilities_invalid_bits_are_rejected() {
        // 0xFF has bits set that don't correspond to any Capability
        assert!(ByteCapabilities::try_from(0xFF_u8).is_err());
    }

    #[test]
    fn byte_capabilities_empty_is_zero() {
        let caps = ByteCapabilities::from(Capabilities::empty());
        let byte_val: u8 = caps.into();
        assert_eq!(byte_val, 0);
    }

    #[test]
    fn byte_capabilities_combined_flags() -> anyhow::Result<()> {
        let caps: Capabilities = Capability::Segmentation | Capability::NoRateControl;
        let byte_caps = ByteCapabilities::from(caps);
        let byte_val: u8 = byte_caps.into();
        let restored = ByteCapabilities::try_from(byte_val)?;
        assert_eq!(*restored.as_ref(), caps);
        Ok(())
    }

    // --- caps_to_ack_mode tests ---

    #[test]
    fn caps_to_ack_mode_both_when_ack_and_nack() {
        let caps: Capabilities = Capability::RetransmissionAck | Capability::RetransmissionNack;
        assert_eq!(caps_to_ack_mode(caps), AcknowledgementMode::Both);
    }

    #[test]
    fn caps_to_ack_mode_full_when_only_ack() {
        let caps: Capabilities = Capability::RetransmissionAck.into();
        assert_eq!(caps_to_ack_mode(caps), AcknowledgementMode::Full);
    }

    #[test]
    fn caps_to_ack_mode_partial_when_no_retransmission() {
        let caps: Capabilities = Capability::Segmentation.into();
        assert_eq!(caps_to_ack_mode(caps), AcknowledgementMode::Partial);
    }

    #[test]
    fn caps_to_ack_mode_partial_when_empty() {
        assert_eq!(caps_to_ack_mode(Capabilities::empty()), AcknowledgementMode::Partial);
    }

    #[test]
    fn caps_to_ack_mode_should_be_partial_when_only_nack() {
        let caps: Capabilities = Capability::RetransmissionNack.into();
        assert_eq!(caps_to_ack_mode(caps), AcknowledgementMode::Partial);
    }

    // --- ClosureReason tests ---

    #[test]
    fn closure_reason_display_values_are_stable() {
        let reasons = [
            ClosureReason::WriteClosed,
            ClosureReason::EmptyRead,
            ClosureReason::Eviction,
        ];
        insta::assert_debug_snapshot!(reasons);
    }

    // --- HoprSessionConfig tests ---

    #[test]
    fn hopr_session_config_default_snapshot() {
        let cfg = HoprSessionConfig::default();
        insta::assert_yaml_snapshot!(cfg);
    }

    // --- SessionTarget tests ---

    #[test]
    fn session_target_variants_debug_snapshot() -> anyhow::Result<()> {
        let targets: Vec<SessionTarget> = vec![
            SessionTarget::UdpStream(SealedHost::Plain(
                "127.0.0.1:8080".parse().context("parsing UDP target")?,
            )),
            SessionTarget::TcpStream(SealedHost::Plain("10.0.0.1:443".parse().context("parsing TCP target")?)),
            SessionTarget::ExitNode(42),
        ];
        insta::assert_debug_snapshot!(targets);
        Ok(())
    }

    // --- SessionId edge cases ---

    #[test]
    fn session_id_display_and_debug_should_be_identical() {
        let id = HoprPseudonym::random();
        assert_eq!(format!("{id}"), format!("{id:?}"));
    }

    #[test]
    fn session_id_hash_eq_consistency() {
        use std::collections::HashSet;
        let pseudonym = HoprPseudonym::random();
        let id1: SessionId = pseudonym;
        let id2: SessionId = pseudonym;
        let id3: SessionId = HoprPseudonym::random();

        let mut set = HashSet::new();
        set.insert(id1);
        assert!(set.contains(&id2));
        assert!(!set.contains(&id3), "different pseudonym should not be in the set");
    }

    // --- Existing tests ---

    #[test_log::test(tokio::test)]
    async fn test_session_bidirectional_flow_without_segmentation() -> anyhow::Result<()> {
        let dst: Address = (&ChainKeypair::random()).into();
        let id: SessionId = HoprPseudonym::random();
        const DATA_LEN: usize = 5000;

        let (alice_tx, bob_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();
        let (bob_tx, alice_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();

        let mut alice_session = HoprSession::new(
            id,
            DestinationRouting::forward_only(dst, RoutingOptions::Hops(0.try_into()?)),
            Default::default(),
            (
                alice_tx,
                alice_rx
                    .map(|(_, data)| ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    })
                    .inspect(|d| debug!("alice rcvd: {}", d.data.total_len())),
            ),
            None,
        )?;

        let mut bob_session = HoprSession::new(
            id,
            DestinationRouting::Return(id.into()),
            Default::default(),
            (
                bob_tx,
                bob_rx
                    .map(|(_, data)| ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    })
                    .inspect(|d| debug!("bob rcvd: {}", d.data.total_len())),
            ),
            None,
        )?;

        let alice_sent = hopr_api::types::crypto_random::random_bytes::<DATA_LEN>();
        let bob_sent = hopr_api::types::crypto_random::random_bytes::<DATA_LEN>();

        let mut bob_recv = [0u8; DATA_LEN];
        let mut alice_recv = [0u8; DATA_LEN];

        tokio::time::timeout(Duration::from_secs(1), alice_session.write_all(&alice_sent))
            .await
            .context("alice write failed")?
            .context("alice write timed out")?;
        alice_session.flush().await?;

        tokio::time::timeout(Duration::from_secs(1), bob_session.write_all(&bob_sent))
            .await
            .context("bob write failed")?
            .context("bob write timed out")?;
        bob_session.flush().await?;

        tokio::time::timeout(Duration::from_secs(1), bob_session.read_exact(&mut bob_recv))
            .await
            .context("bob read failed")?
            .context("bob read timed out")?;

        tokio::time::timeout(Duration::from_secs(1), alice_session.read_exact(&mut alice_recv))
            .await
            .context("alice read failed")?
            .context("alice read timed out")?;

        assert_eq!(&alice_sent, bob_recv.as_slice());
        assert_eq!(bob_sent, alice_recv);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_session_bidirectional_flow_with_segmentation() -> anyhow::Result<()> {
        let dst: Address = (&ChainKeypair::random()).into();
        let id: SessionId = HoprPseudonym::random();
        const DATA_LEN: usize = 5000;

        let (alice_tx, bob_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();
        let (bob_tx, alice_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();

        let mut alice_session = HoprSession::new(
            id,
            DestinationRouting::forward_only(dst, RoutingOptions::Hops(0.try_into()?)),
            HoprSessionConfig {
                capabilities: Capability::Segmentation.into(),
                ..Default::default()
            },
            (
                alice_tx,
                alice_rx
                    .map(|(_, data)| ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    })
                    .inspect(|d| debug!("alice rcvd: {}", d.data.total_len())),
            ),
            None,
        )?;

        let mut bob_session = HoprSession::new(
            id,
            DestinationRouting::Return(id.into()),
            HoprSessionConfig {
                capabilities: Capability::Segmentation.into(),
                ..Default::default()
            },
            (
                bob_tx,
                bob_rx
                    .map(|(_, data)| ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    })
                    .inspect(|d| debug!("bob rcvd: {}", d.data.total_len())),
            ),
            None,
        )?;

        let alice_sent = hopr_api::types::crypto_random::random_bytes::<DATA_LEN>();
        let bob_sent = hopr_api::types::crypto_random::random_bytes::<DATA_LEN>();

        let mut bob_recv = [0u8; DATA_LEN];
        let mut alice_recv = [0u8; DATA_LEN];

        tokio::time::timeout(Duration::from_secs(1), alice_session.write_all(&alice_sent))
            .await
            .context("alice write failed")?
            .context("alice write timed out")?;
        alice_session.flush().await?;

        tokio::time::timeout(Duration::from_secs(1), bob_session.write_all(&bob_sent))
            .await
            .context("bob write failed")?
            .context("bob write timed out")?;
        bob_session.flush().await?;

        tokio::time::timeout(Duration::from_secs(1), bob_session.read_exact(&mut bob_recv))
            .await
            .context("bob read failed")?
            .context("bob read timed out")?;

        tokio::time::timeout(Duration::from_secs(1), alice_session.read_exact(&mut alice_recv))
            .await
            .context("alice read failed")?
            .context("alice read timed out")?;

        assert_eq!(alice_sent, bob_recv);
        assert_eq!(bob_sent, alice_recv);

        Ok(())
    }
}
