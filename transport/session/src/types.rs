use std::{
    fmt::{Debug, Display, Formatter},
    hash::{Hash, Hasher},
    pin::Pin,
    str::FromStr,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use futures::{SinkExt, StreamExt, TryStreamExt};
use hopr_internal_types::prelude::HoprPseudonym;
use hopr_network_types::{
    prelude::{DestinationRouting, SealedHost},
    utils::{AsyncWriteSink, DuplexIO},
};
use hopr_primitive_types::{
    errors::GeneralError,
    prelude::{BytesRepresentable, ToHex},
};
use hopr_protocol_app::prelude::{ApplicationData, ApplicationDataIn, ApplicationDataOut, Tag};
use hopr_protocol_session::{
    AcknowledgementMode, AcknowledgementState, AcknowledgementStateConfig, SessionSocket, SessionSocketConfig,
    Stateless,
};
use hopr_protocol_start::StartProtocol;
use tracing::{debug, instrument};

use crate::{
    Capabilities, Capability,
    errors::TransportSessionError,
    metrics::{MetricsState, SessionMetrics},
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

/// Calculates the maximum number of decimal digits needed to represent an N-byte unsigned integer.
///
/// The calculation is based on the formula: ⌈8n × log_10(2)⌉
/// where n is the number of bytes.
const fn max_decimal_digits_for_n_bytes(n: usize) -> usize {
    // log_10(2) = 0.301029995664 multiplied by 1 000 000 to work with integers in a const function
    const LOG10_2_SCALED: u64 = 301030;
    const SCALE: u64 = 1_000_000;

    // 8n * log_10(2) scaled
    let scaled = 8 * n as u64 * LOG10_2_SCALED;

    scaled.div_ceil(SCALE) as usize
}

// Enough to fit HoprPseudonym in hex (with 0x prefix), delimiter and tag number
const MAX_SESSION_ID_STR_LEN: usize = 2 + 2 * HoprPseudonym::SIZE + 1 + max_decimal_digits_for_n_bytes(Tag::SIZE);

/// Unique ID of a specific Session in a certain direction.
///
/// Simple wrapper around the maximum range of the port like session unique identifier.
/// It is a simple combination of an application tag for the Session and
/// a [`HoprPseudonym`].
#[derive(Clone, Copy)]
pub struct SessionId {
    tag: Tag,
    pseudonym: HoprPseudonym,
    // Since this SessionId is commonly represented as a string,
    // we cache its string representation here.
    // Also, by using a statically allocated ArrayString, we allow the SessionId to remain Copy.
    // This representation is possibly truncated to MAX_SESSION_ID_STR_LEN.
    // This member is always computed and is therefore not serialized.
    cached: arrayvec::ArrayString<MAX_SESSION_ID_STR_LEN>,
}

impl SessionId {
    const DELIMITER: char = ':';

    pub fn new<T: Into<Tag>>(tag: T, pseudonym: HoprPseudonym) -> Self {
        let tag = tag.into();
        let mut cached = format!("{pseudonym}{}{tag}", Self::DELIMITER);
        cached.truncate(MAX_SESSION_ID_STR_LEN);

        Self {
            tag,
            pseudonym,
            cached: cached.parse().expect("cannot fail due to truncation"),
        }
    }

    pub fn tag(&self) -> Tag {
        self.tag
    }

    pub fn pseudonym(&self) -> &HoprPseudonym {
        &self.pseudonym
    }

    pub fn as_str(&self) -> &str {
        &self.cached
    }
}

impl FromStr for SessionId {
    type Err = TransportSessionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split_once(Self::DELIMITER)
            .ok_or(TransportSessionError::InvalidSessionId)
            .and_then(
                |(pseudonym, tag)| match (HoprPseudonym::from_hex(pseudonym), Tag::from_str(tag)) {
                    (Ok(p), Ok(t)) => Ok(Self::new(t, p)),
                    _ => Err(TransportSessionError::InvalidSessionId),
                },
            )
    }
}

impl serde::Serialize for SessionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("SessionId", 2)?;
        state.serialize_field("tag", &self.tag)?;
        state.serialize_field("pseudonym", &self.pseudonym)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for SessionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;

        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Tag,
            Pseudonym,
        }

        struct SessionIdVisitor;

        impl<'de> de::Visitor<'de> for SessionIdVisitor {
            type Value = SessionId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct SessionId")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<SessionId, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                Ok(SessionId::new(
                    seq.next_element::<Tag>()?
                        .ok_or_else(|| de::Error::invalid_length(0, &self))?,
                    seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?,
                ))
            }

            fn visit_map<V>(self, mut map: V) -> Result<SessionId, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut tag: Option<Tag> = None;
                let mut pseudonym: Option<HoprPseudonym> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Tag => {
                            if tag.is_some() {
                                return Err(de::Error::duplicate_field("tag"));
                            }
                            tag = Some(map.next_value()?);
                        }
                        Field::Pseudonym => {
                            if pseudonym.is_some() {
                                return Err(de::Error::duplicate_field("pseudonym"));
                            }
                            pseudonym = Some(map.next_value()?);
                        }
                    }
                }

                Ok(SessionId::new(
                    tag.ok_or_else(|| de::Error::missing_field("tag"))?,
                    pseudonym.ok_or_else(|| de::Error::missing_field("pseudonym"))?,
                ))
            }
        }

        const FIELDS: &[&str] = &["tag", "pseudonym"];
        deserializer.deserialize_struct("SessionId", FIELDS, SessionIdVisitor)
    }
}

impl Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Debug for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl PartialEq for SessionId {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag && self.pseudonym == other.pseudonym
    }
}

impl Eq for SessionId {}

impl Hash for SessionId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tag.hash(state);
        self.pseudonym.hash(state);
    }
}

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

/// Describes a node service target.
/// These are specialized [`SessionTargets`](SessionTarget::ExitNode)
/// that are local to the Exit node and have different purposes, such as Cover Traffic.
///
/// These targets cannot be [sealed](SealedHost) from the Entry node.
pub type ServiceId = u32;

/// Defines what should happen with the data at the recipient where the
/// data from the established session are supposed to be forwarded to some `target`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SessionTarget {
    /// Target is running over UDP with the given IP address and port.
    UdpStream(SealedHost),
    /// Target is running over TCP with the given address and port.
    TcpStream(SealedHost),
    /// Target is a service directly at the exit node with the given service ID.
    ExitNode(ServiceId),
}

/// Wrapper for incoming [`HoprSession`] along with other information
/// extracted from the Start protocol during the session establishment.
#[derive(Debug)]
pub struct IncomingSession {
    /// Actual incoming session.
    pub session: HoprSession,
    /// Desired [target](SessionTarget) of the data received over the session.
    pub target: SessionTarget,
}

/// Configures the Session protocol socket over HOPR.
#[derive(Copy, Clone, Debug, PartialEq, Eq, smart_default::SmartDefault)]
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
    pub frame_timeout: Duration,
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
    metrics: Arc<SessionMetrics>,
}

impl HoprSession {
    /// Creates a new HOPR Session.
    ///
    /// It builds an [`futures::io::AsyncRead`] + [`futures::io::AsyncWrite`] transport
    /// from the given `hopr` interface and passing it to the appropriate [`UnreliableSocket`] or [`ReliableSocket`]
    /// based on the given `capabilities`.
    ///
    /// The `on_close` closure can be optionally called when the Session has been closed via `poll_close`.
    #[tracing::instrument(skip(hopr, on_close), fields(session_id = %id))]
    pub fn new<Tx, Rx>(
        id: SessionId,
        routing: DestinationRouting,
        cfg: HoprSessionConfig,
        hopr: (Tx, Rx),
        metrics: Arc<SessionMetrics>,
        on_close: Option<Box<dyn FnOnce(SessionId, ClosureReason) + Send + Sync>>,
    ) -> Result<Self, TransportSessionError>
    where
        Tx: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Send + Sync + Unpin + 'static,
        Rx: futures::Stream<Item = ApplicationDataIn> + Send + Sync + Unpin + 'static,
        Tx::Error: std::error::Error + Send + Sync,
    {
        let routing_clone = routing.clone();

        // Wrap the HOPR transport so that it appears as regular transport to the SessionSocket
        let transport = DuplexIO(
            AsyncWriteSink::<{ ApplicationData::PAYLOAD_SIZE }, _>(hopr.0.sink_map_err(std::io::Error::other).with(
                move |buf: Box<[u8]>| {
                    // The Session protocol does not set any packet info on outgoing packets.
                    // However, the SessionManager on top usually overrides this.
                    futures::future::ready(
                        ApplicationData::new(id.tag(), buf.into_vec())
                            .map(|data| (routing_clone.clone(), ApplicationDataOut::with_no_packet_info(data)))
                            .map_err(std::io::Error::other),
                    )
                },
            )),
            // The Session protocol ignores the packet info on incoming packets.
            // It is typically SessionManager's job to interpret those.
            hopr.1
                .map(|data| Ok::<_, std::io::Error>(data.data.plain_text))
                .into_async_read(),
        );

        // Based on the requested capabilities, see if we should use the Session protocol
        let inner: Box<dyn AsyncReadWrite> = if cfg.capabilities.contains(Capability::Segmentation) {
            let socket_cfg = SessionSocketConfig {
                frame_size: cfg.frame_mtu,
                frame_timeout: cfg.frame_timeout,
                capacity: 16384,
                flush_immediately: cfg.capabilities.contains(Capability::NoDelay),
                ..Default::default()
            };

            // Need to test the capabilities separately, because any Retransmission capability
            // implies Segmentation, and therefore `is_disjoint` would fail
            if cfg.capabilities.contains(Capability::RetransmissionAck)
                || cfg.capabilities.contains(Capability::RetransmissionNack)
            {
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
                    max_outgoing_frame_retries: 2,
                    ..Default::default()
                };

                debug!(?socket_cfg, ?ack_cfg, "opening new stateful session socket");

                let state = MetricsState::new(
                    AcknowledgementState::<{ ApplicationData::PAYLOAD_SIZE }>::new(id, ack_cfg),
                    metrics.clone(),
                );
                Box::new(SessionSocket::new(transport, state, socket_cfg)?)
            } else {
                debug!(?socket_cfg, "opening new stateless session socket");

                let state = MetricsState::new(Stateless::<{ ApplicationData::PAYLOAD_SIZE }>::new(id), metrics.clone());
                Box::new(SessionSocket::new(transport, state, socket_cfg)?)
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
            metrics,
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

    pub fn metrics(&self) -> &Arc<SessionMetrics> {
        &self.metrics
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
    #[instrument(name = "Session::poll_read", level = "trace", skip(self, cx, buf), fields(session_id = %self.id), ret)]
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        let this = self.project();
        let read = futures::ready!(this.inner.poll_read(cx, buf))?;
        this.metrics.record_read(read);
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
    #[instrument(name = "Session::poll_write", level = "trace", skip(self, cx, buf), fields(session_id = %self.id), ret)]
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        let this = self.project();
        let res = this.inner.poll_write(cx, buf);
        if let Poll::Ready(Ok(written)) = &res {
            this.metrics.record_write(*written);
        }
        res
    }

    #[instrument(name = "Session::poll_flush", level = "trace", skip(self, cx), fields(session_id = %self.id), ret)]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.project().inner.poll_flush(cx)
    }

    #[instrument(name = "Session::poll_close", level = "trace", skip(self, cx), fields(session_id = %self.id), ret)]
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let this = self.project();
        futures::ready!(this.inner.poll_close(cx))?;
        tracing::trace!("hopr session closed");

        this.metrics.set_state(crate::metrics::SessionLifecycleState::Closing);

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
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::prelude::*;
    use hopr_network_types::prelude::*;
    use hopr_primitive_types::prelude::*;

    use super::*;
    use crate::{SESSION_MTU, metrics::SessionMetrics};

    #[test]
    fn test_session_id_to_str_from_str() -> anyhow::Result<()> {
        let id = SessionId::new(1234_u64, HoprPseudonym::random());
        assert_eq!(id.as_str(), id.to_string());
        assert_eq!(id, SessionId::from_str(id.as_str())?);

        Ok(())
    }

    #[test]
    fn test_max_decimal_digits_for_n_bytes() {
        assert_eq!(3, max_decimal_digits_for_n_bytes(size_of::<u8>()));
        assert_eq!(5, max_decimal_digits_for_n_bytes(size_of::<u16>()));
        assert_eq!(10, max_decimal_digits_for_n_bytes(size_of::<u32>()));
        assert_eq!(20, max_decimal_digits_for_n_bytes(size_of::<u64>()));
    }

    #[test]
    fn standard_session_id_must_fit_within_limit() {
        let id = format!("{}:{}", SimplePseudonym::random(), Tag::Application(Tag::MAX));
        assert!(id.len() <= MAX_SESSION_ID_STR_LEN);
    }

    #[test]
    fn session_id_should_serialize_and_deserialize_correctly() -> anyhow::Result<()> {
        let pseudonym = HoprPseudonym::random();
        let tag: Tag = 1234u64.into();

        let session_id_1 = SessionId::new(tag, pseudonym);
        let data = serde_cbor_2::to_vec(&session_id_1)?;
        let session_id_2: SessionId = serde_cbor_2::from_slice(&data)?;

        assert_eq!(tag, session_id_2.tag());
        assert_eq!(pseudonym, *session_id_2.pseudonym());

        assert_eq!(session_id_1.as_str(), session_id_2.as_str());
        assert_eq!(session_id_1, session_id_2);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_session_bidirectional_flow_without_segmentation() -> anyhow::Result<()> {
        let dst: Address = (&ChainKeypair::random()).into();
        let id = SessionId::new(1234_u64, HoprPseudonym::random());
        const DATA_LEN: usize = 5000;

        let alice_metrics = Arc::new(SessionMetrics::new(
            id,
            None,
            SESSION_MTU,
            Duration::from_millis(800),
            0,
        ));
        let bob_metrics = Arc::new(SessionMetrics::new(
            id,
            None,
            SESSION_MTU,
            Duration::from_millis(800),
            0,
        ));

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
            alice_metrics,
            None,
        )?;

        let mut bob_session = HoprSession::new(
            id,
            DestinationRouting::Return(id.pseudonym().into()),
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
            bob_metrics,
            None,
        )?;

        let alice_sent = hopr_crypto_random::random_bytes::<DATA_LEN>();
        let bob_sent = hopr_crypto_random::random_bytes::<DATA_LEN>();

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
        let id = SessionId::new(1234_u64, HoprPseudonym::random());
        const DATA_LEN: usize = 5000;

        let alice_metrics = Arc::new(SessionMetrics::new(
            id,
            None,
            SESSION_MTU,
            Duration::from_millis(800),
            16384,
        ));
        let bob_metrics = Arc::new(SessionMetrics::new(
            id,
            None,
            SESSION_MTU,
            Duration::from_millis(800),
            16384,
        ));

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
            alice_metrics,
            None,
        )?;

        let mut bob_session = HoprSession::new(
            id,
            DestinationRouting::Return(id.pseudonym().into()),
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
            bob_metrics,
            None,
        )?;

        let alice_sent = hopr_crypto_random::random_bytes::<DATA_LEN>();
        let bob_sent = hopr_crypto_random::random_bytes::<DATA_LEN>();

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
