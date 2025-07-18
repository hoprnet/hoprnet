use std::{
    fmt::{Debug, Display, Formatter},
    hash::{Hash, Hasher},
    io::{Error, ErrorKind},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use futures::{StreamExt, pin_mut};
use hopr_crypto_packet::prelude::HoprPacket;
use hopr_internal_types::prelude::HoprPseudonym;
use hopr_network_types::{
    prelude::{DestinationRouting, SealedHost},
    session::state::{SessionConfig, SessionSocket},
};
use hopr_primitive_types::prelude::BytesRepresentable;
use hopr_transport_packet::prelude::{ApplicationData, Tag};
use tracing::{debug, error};

use crate::{Capabilities, Capability, capabilities_to_features, errors::TransportSessionError, traits::SendMsg};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_SESSION_INNER_SIZES: hopr_metrics::MultiHistogram =
        hopr_metrics::MultiHistogram::new(
            "hopr_session_inner_sizes",
            "Sizes of data chunks fed from inner session to HOPR protocol",
            vec![20.0, 40.0, 80.0, 160.0, 320.0, 640.0, 1280.0],
            &["session_id"]
    ).unwrap();
}

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
    pub fn new<T: Into<Tag>>(tag: T, pseudonym: HoprPseudonym) -> Self {
        let tag = tag.into();
        let mut cached = format!("{pseudonym}:{tag}");
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

#[cfg(feature = "serde")]
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

#[cfg(feature = "serde")]
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

/// Helper trait to allow Box aliasing
trait AsyncReadWrite: futures::AsyncWrite + futures::AsyncRead + Send {}
impl<T: futures::AsyncWrite + futures::AsyncRead + Send> AsyncReadWrite for T {}

/// Describes a node service target.
/// These are specialized [`SessionTargets`](SessionTarget::ExitNode)
/// that are local to the Exit node and have different purposes, such as Cover Traffic.
///
/// These targets cannot be [sealed](SealedHost) from the Entry node.
pub type ServiceId = u32;

/// Defines what should happen with the data at the recipient where the
/// data from the established session are supposed to be forwarded to some `target`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SessionTarget {
    /// Target is running over UDP with the given IP address and port.
    UdpStream(SealedHost),
    /// Target is running over TCP with the given address and port.
    TcpStream(SealedHost),
    /// Target is a service directly at the exit node with the given service ID.
    ExitNode(ServiceId),
}

/// Wrapper for incoming [Session] along with other information
/// extracted from the Start protocol during the session establishment.
#[derive(Debug)]
pub struct IncomingSession {
    /// Actual incoming session.
    pub session: Session,
    /// Desired [target](SessionTarget) of the data received over the session.
    pub target: SessionTarget,
}

// TODO: missing docs
pub struct Session {
    id: SessionId,
    inner: Pin<Box<dyn AsyncReadWrite>>,
    routing: DestinationRouting,
    capabilities: Capabilities,
    on_close: Option<Box<dyn FnOnce(SessionId) + Send + Sync>>,
}

impl Session {
    pub fn new(
        id: SessionId,
        routing: DestinationRouting,
        capabilities: Capabilities,
        tx: Arc<dyn SendMsg + Send + Sync>,
        rx: Pin<Box<dyn futures::Stream<Item = Box<[u8]>> + Send + Sync>>,
        on_close: Option<Box<dyn FnOnce(SessionId) + Send + Sync>>,
    ) -> Self {
        let inner_session = InnerSession::new(id, routing.clone(), tx, rx);

        // If we request any capability, we need to use Session protocol
        if !capabilities.is_empty() {
            // This is a very coarse assumption, that 3-hop takes at most 3 seconds.
            // We can no longer base this timeout on the number of hops because
            // it is not known for SURB-based routing.
            let rto_base = Duration::from_secs(3);

            let expiration_coefficient =
                if !capabilities.is_disjoint(Capability::RetransmissionAck | Capability::RetransmissionNack) {
                    4
                } else {
                    1
                };

            // TODO: tweak the default Session protocol config
            let cfg = SessionConfig {
                enabled_features: capabilities_to_features(&capabilities),
                acknowledged_frames_buffer: 100_000, // Can hold frames for > 40 sec at 2000 frames/sec
                frame_expiration_age: rto_base * expiration_coefficient,
                rto_base_receiver: rto_base, // Ask for segment resend, if not yet complete after this period
                rto_base_sender: rto_base * 2, // Resend frame if is not acknowledged after this period
                ..Default::default()
            };
            debug!(
                session_id = ?id,
                ?cfg,
                "opening new session socket"
            );

            Self {
                id,
                inner: Box::pin(SessionSocket::<{ ApplicationData::PAYLOAD_SIZE }>::new(
                    id,
                    inner_session,
                    cfg,
                )),
                routing,
                capabilities,
                on_close,
            }
        } else {
            // Otherwise, no additional sub protocol is necessary
            Self {
                id,
                inner: Box::pin(inner_session),
                routing,
                capabilities,
                on_close,
            }
        }
    }

    /// ID of this Session.
    pub fn id(&self) -> &SessionId {
        &self.id
    }

    /// Routing options used to deliver data.
    pub fn routing(&self) -> &DestinationRouting {
        &self.routing
    }

    /// Capabilities of this Session.
    pub fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("id", &self.id)
            .field("routing", &self.routing)
            .finish_non_exhaustive()
    }
}

impl futures::AsyncRead for Session {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        let inner = self.inner.as_mut();
        pin_mut!(inner);
        inner.poll_read(cx, buf)
    }
}

impl futures::AsyncWrite for Session {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        let inner = &mut self.inner;
        pin_mut!(inner);
        inner.poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let inner = &mut self.inner;
        pin_mut!(inner);
        inner.poll_flush(cx)
    }

    fn poll_close(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<std::io::Result<()>> {
        let inner = &mut self.inner;
        pin_mut!(inner);
        match inner.poll_close(cx) {
            Poll::Ready(res) => {
                // Notify about closure if desired
                if let Some(notifier) = self.on_close.take() {
                    notifier(self.id);
                }
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(feature = "runtime-tokio")]
impl tokio::io::AsyncRead for Session {
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
impl tokio::io::AsyncWrite for Session {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Error>> {
        futures::AsyncWrite::poll_write(self.as_mut(), cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        futures::AsyncWrite::poll_flush(self.as_mut(), cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        futures::AsyncWrite::poll_close(self.as_mut(), cx)
    }
}

type FuturesBuffer = futures::stream::FuturesUnordered<
    Pin<Box<dyn std::future::Future<Output = Result<(), TransportSessionError>> + Send>>,
>;
struct InnerSession {
    id: SessionId,
    routing: DestinationRouting,
    rx: Pin<Box<dyn futures::Stream<Item = Box<[u8]>> + Send + Sync>>,
    tx: Arc<dyn SendMsg + Send + Sync>,
    tx_bytes: usize,
    tx_buffer: FuturesBuffer,
    rx_buffer: [u8; HoprPacket::PAYLOAD_SIZE],
    rx_buffer_range: (usize, usize),
    closed: bool,
}

impl InnerSession {
    pub fn new(
        id: SessionId,
        routing: DestinationRouting,
        tx: Arc<dyn SendMsg + Send + Sync>,
        rx: Pin<Box<dyn futures::Stream<Item = Box<[u8]>> + Send + Sync>>,
    ) -> Self {
        Self {
            id,
            routing,
            rx,
            tx,
            tx_bytes: 0,
            tx_buffer: futures::stream::FuturesUnordered::new(),
            rx_buffer: [0; HoprPacket::PAYLOAD_SIZE],
            rx_buffer_range: (0, 0),
            closed: false,
        }
    }
}

impl futures::AsyncWrite for InnerSession {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        if self.closed {
            return Poll::Ready(Err(Error::new(ErrorKind::BrokenPipe, "session closed")));
        }

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_SESSION_INNER_SIZES.observe(&[self.id.as_str()], buf.len() as f64);

        if !self.tx_buffer.is_empty() {
            loop {
                match self.tx_buffer.poll_next_unpin(cx) {
                    Poll::Ready(Some(Ok(()))) => {
                        continue;
                    }
                    Poll::Ready(Some(Err(TransportSessionError::OutOfSurbs))) => {
                        // Discard messages until SURBs are available
                        error!(session_id = %self.id, "message discarded due to missing SURB for reply");
                        continue;
                    }
                    Poll::Ready(Some(Err(e))) => {
                        error!(session_id = %self.id, error = %e, "failed to send the message chunk inside a session");
                        return Poll::Ready(Err(Error::from(ErrorKind::BrokenPipe)));
                    }
                    Poll::Ready(None) => {
                        self.tx_buffer.clear();
                        return Poll::Ready(Ok(self.tx_bytes));
                    }
                    Poll::Pending => {
                        return Poll::Pending;
                    }
                }
            }
        }

        let tag = self.id.tag();

        self.tx_buffer.clear();
        self.tx_bytes = 0;

        for i in
            0..(buf.len() / ApplicationData::PAYLOAD_SIZE + ((buf.len() % ApplicationData::PAYLOAD_SIZE != 0) as usize))
        {
            let start = i * ApplicationData::PAYLOAD_SIZE;
            let end = ((i + 1) * ApplicationData::PAYLOAD_SIZE).min(buf.len());

            let payload = ApplicationData::new(tag, &buf[start..end]);
            let sender = self.tx.clone();
            let routing = self.routing.clone();

            self.tx_buffer
                .push(Box::pin(async move { sender.send_message(payload, routing).await }));

            self.tx_bytes += end - start;
        }

        loop {
            match self.tx_buffer.poll_next_unpin(cx) {
                Poll::Ready(Some(Ok(_))) => {
                    continue;
                }
                Poll::Ready(Some(Err(TransportSessionError::OutOfSurbs))) => {
                    // Discard messages until SURBs are available
                    error!(session_id = %self.id, "message discarded due to missing SURB for reply");
                    continue;
                }
                Poll::Ready(Some(Err(error))) => {
                    error!(session_id = %self.id, %error, "failed to send the message chunk inside a session");
                    break Poll::Ready(Err(Error::from(ErrorKind::BrokenPipe)));
                }
                Poll::Ready(None) => {
                    self.tx_buffer.clear();
                    break Poll::Ready(Ok(self.tx_bytes));
                }
                Poll::Pending => {
                    break Poll::Pending;
                }
            }
        }
    }

    fn poll_flush(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<std::io::Result<()>> {
        if self.closed {
            return Poll::Ready(Err(Error::new(ErrorKind::BrokenPipe, "session closed")));
        }

        while let Poll::Ready(Some(result)) = self.tx_buffer.poll_next_unpin(cx) {
            if let Err(error) = result {
                error!(session_id = %self.id, %error, "failed to send message chunk inside session during flush");
                return Poll::Ready(Err(Error::from(ErrorKind::BrokenPipe)));
            }
        }
        Poll::Ready(Ok(()))
    }

    fn poll_close(mut self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<std::io::Result<()>> {
        self.closed = true;
        Poll::Ready(Ok(()))
    }
}

impl futures::AsyncRead for InnerSession {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        if self.rx_buffer_range.0 != self.rx_buffer_range.1 {
            let start = self.rx_buffer_range.0;
            let copy_len = self.rx_buffer_range.1.min(buf.len());

            buf[..copy_len].copy_from_slice(&self.rx_buffer[start..start + copy_len]);

            self.rx_buffer_range.0 += copy_len;
            if self.rx_buffer_range.0 == self.rx_buffer_range.1 {
                self.rx_buffer_range = (0, 0);
            }

            return Poll::Ready(Ok(copy_len));
        }

        match self.rx.poll_next_unpin(cx) {
            Poll::Ready(Some(data)) => {
                let data_len = data.len();
                let copy_len = data_len.min(buf.len());
                if copy_len < data_len {
                    self.rx_buffer[0..data_len - copy_len].copy_from_slice(&data[copy_len..]);
                    self.rx_buffer_range = (0, data_len - copy_len);
                }

                buf[..copy_len].copy_from_slice(&data[..copy_len]);

                Poll::Ready(Ok(copy_len))
            }
            Poll::Ready(None) => {
                Poll::Ready(Ok(0)) // due to convention, Ok(0) indicates EOF
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Convenience function to copy data in both directions between a [Session] and arbitrary
/// async IO stream.
/// This function is only available with Tokio and will panic with other runtimes.
#[cfg(feature = "runtime-tokio")]
pub async fn transfer_session<S>(
    session: &mut Session,
    stream: &mut S,
    max_buffer: usize,
) -> std::io::Result<(usize, usize)>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    // We can always read as much as possible from the Session and then write it to the Stream.
    // There are two possibilities for the opposite direction:
    // 1) If Session protocol is used for segmentation, we need to buffer up data at MAX_WRITE_SIZE.
    // 2) Otherwise, the bare session implements chunking, therefore, data can be written with arbitrary sizes.
    let into_session_len = if session.capabilities().contains(Capability::Segmentation) {
        max_buffer.min(SessionSocket::<{ ApplicationData::PAYLOAD_SIZE }>::MAX_WRITE_SIZE)
    } else {
        max_buffer
    };

    debug!(
        session_id = ?session.id(),
        egress_buffer = max_buffer,
        ingress_buffer = into_session_len,
        "session buffers"
    );

    hopr_network_types::utils::copy_duplex(session, stream, max_buffer, into_session_len)
        .await
        .map(|(a, b)| (a as usize, b as usize))
}

#[cfg(test)]
mod tests {
    use futures::{AsyncReadExt, AsyncWriteExt};
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::{
        keypairs::{ChainKeypair, Keypair},
        types::SimplePseudonym,
    };
    use hopr_network_types::prelude::{RoutingOptions, SurbMatcher};
    use hopr_primitive_types::prelude::Address;

    use super::*;
    use crate::traits::MockSendMsg;

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

    #[cfg(feature = "serde")]
    #[test]
    fn session_id_should_serialize_and_deserialize_correctly() -> anyhow::Result<()> {
        const SESSION_BINCODE_CONFIGURATION: bincode::config::Configuration = bincode::config::standard()
            .with_little_endian()
            .with_variable_int_encoding();

        let pseudonym = HoprPseudonym::random();
        let tag: Tag = 1234u64.into();

        let session_id_1 = SessionId::new(tag, pseudonym);
        let data = bincode::serde::encode_to_vec(session_id_1, SESSION_BINCODE_CONFIGURATION)?;
        let session_id_2: SessionId = bincode::serde::decode_from_slice(&data, SESSION_BINCODE_CONFIGURATION)?.0;

        assert_eq!(tag, session_id_2.tag());
        assert_eq!(pseudonym, *session_id_2.pseudonym());

        assert_eq!(session_id_1.as_str(), session_id_2.as_str());
        assert_eq!(session_id_1, session_id_2);

        Ok(())
    }

    #[test]
    fn session_should_identify_with_its_own_id() -> anyhow::Result<()> {
        let addr: Address = (&ChainKeypair::random()).into();
        let id = SessionId::new(1u64, HoprPseudonym::random());
        let (_tx, rx) = futures::channel::mpsc::unbounded();
        let mock = MockSendMsg::new();

        let session = InnerSession::new(
            id,
            DestinationRouting::forward_only(addr, RoutingOptions::Hops(1_u32.try_into()?)),
            Arc::new(mock),
            Box::pin(rx),
        );

        assert_eq!(session.id, id);

        Ok(())
    }

    #[tokio::test]
    async fn session_should_read_data_in_one_swoop_if_the_buffer_is_sufficiently_large() -> anyhow::Result<()> {
        let addr: Address = (&ChainKeypair::random()).into();
        let id = SessionId::new(1u64, HoprPseudonym::random());
        let (tx, rx) = futures::channel::mpsc::unbounded();
        let mock = MockSendMsg::new();

        let mut session = InnerSession::new(
            id,
            DestinationRouting::forward_only(addr, RoutingOptions::Hops(1_u32.try_into()?)),
            Arc::new(mock),
            Box::pin(rx),
        );

        let random_data = hopr_crypto_random::random_bytes::<{ HoprPacket::PAYLOAD_SIZE }>()
            .as_ref()
            .to_vec()
            .into_boxed_slice();

        assert!(tx.unbounded_send(random_data.clone()).is_ok());

        let mut buffer = vec![0; HoprPacket::PAYLOAD_SIZE * 2];

        let bytes_read = session.read(&mut buffer[..]).await?;

        assert_eq!(bytes_read, random_data.len());
        assert_eq!(&buffer[..bytes_read], random_data.as_ref());

        Ok(())
    }

    #[tokio::test]
    async fn session_should_read_data_in_multiple_rounds_if_the_buffer_is_not_sufficiently_large() -> anyhow::Result<()>
    {
        let addr: Address = (&ChainKeypair::random()).into();
        let id = SessionId::new(1u64, HoprPseudonym::random());
        let (tx, rx) = futures::channel::mpsc::unbounded();
        let mock = MockSendMsg::new();

        let mut session = InnerSession::new(
            id,
            DestinationRouting::forward_only(addr, RoutingOptions::Hops(1_u32.try_into()?)),
            Arc::new(mock),
            Box::pin(rx),
        );

        let random_data = hopr_crypto_random::random_bytes::<{ HoprPacket::PAYLOAD_SIZE }>()
            .as_ref()
            .to_vec()
            .into_boxed_slice();

        assert!(tx.unbounded_send(random_data.clone()).is_ok());

        const BUFFER_SIZE: usize = HoprPacket::PAYLOAD_SIZE - 1;
        let mut buffer = vec![0; BUFFER_SIZE];

        let bytes_read = session.read(&mut buffer[..]).await?;

        assert_eq!(bytes_read, BUFFER_SIZE);
        assert_eq!(&buffer[..bytes_read], &random_data[..BUFFER_SIZE]);

        let bytes_read = session.read(&mut buffer[..]).await?;

        assert_eq!(bytes_read, HoprPacket::PAYLOAD_SIZE - BUFFER_SIZE);
        assert_eq!(&buffer[..bytes_read], &random_data[BUFFER_SIZE..]);

        Ok(())
    }

    #[tokio::test]
    async fn session_should_write_data_on_forward_path() -> anyhow::Result<()> {
        let addr: Address = (&ChainKeypair::random()).into();
        let id = SessionId::new(1u64, HoprPseudonym::random());
        let (_tx, rx) = futures::channel::mpsc::unbounded();
        let mut mock = MockSendMsg::new();

        let data = b"Hello, world!".to_vec().into_boxed_slice();

        mock.expect_send_message()
            .times(1)
            .withf(move |data, routing,| {
                assert_eq!(data.plain_text, b"Hello, world!".to_vec().into_boxed_slice());
                assert!(matches!(routing, DestinationRouting::Forward {forward_options,..} if forward_options == &RoutingOptions::Hops(1_u32.try_into().expect("must be convertible"))));
                true
            })
            .returning(|_, _| Ok(()));

        let mut session = InnerSession::new(
            id,
            DestinationRouting::forward_only(addr, RoutingOptions::Hops(1_u32.try_into()?)),
            Arc::new(mock),
            Box::pin(rx),
        );

        let bytes_written = session.write(&data).await?;
        assert_eq!(bytes_written, data.len());

        Ok(())
    }

    #[tokio::test]
    async fn session_should_write_data_on_return_path() -> anyhow::Result<()> {
        let id = SessionId::new(1u64, HoprPseudonym::random());
        let (_tx, rx) = futures::channel::mpsc::unbounded();
        let mut mock = MockSendMsg::new();

        let data = b"Hello, world!".to_vec().into_boxed_slice();

        mock.expect_send_message()
            .times(1)
            .withf(move |data, routing,| {
                assert_eq!(data.plain_text, b"Hello, world!".to_vec().into_boxed_slice());
                assert!(matches!(routing, DestinationRouting::Return(SurbMatcher::Pseudonym(pseudonym)) if pseudonym == &id.pseudonym));
                true
            })
            .returning(|_, _| Ok(()));

        let mut session = InnerSession::new(
            id,
            DestinationRouting::Return(SurbMatcher::Pseudonym(id.pseudonym)),
            Arc::new(mock),
            Box::pin(rx),
        );

        let bytes_written = session.write(&data).await?;
        assert_eq!(bytes_written, data.len());

        Ok(())
    }

    #[tokio::test]
    async fn session_should_chunk_the_data_if_without_segmentation_the_write_size_is_greater_than_the_usable_mtu_size()
    -> anyhow::Result<()> {
        const TO_SEND: usize = ApplicationData::PAYLOAD_SIZE * 2 + 10;

        let addr: Address = (&ChainKeypair::random()).into();
        let id = SessionId::new(1u64, HoprPseudonym::random());
        let (_tx, rx) = futures::channel::mpsc::unbounded();
        let mut mock = MockSendMsg::new();

        let data = hopr_crypto_random::random_bytes::<TO_SEND>()
            .as_ref()
            .to_vec()
            .into_boxed_slice();

        mock.expect_send_message().times(3).returning(|_, _| Ok(()));

        let mut session = InnerSession::new(
            id,
            DestinationRouting::forward_only(addr, RoutingOptions::Hops(1_u32.try_into()?)),
            Arc::new(mock),
            Box::pin(rx),
        );

        let bytes_written = session.write(&data).await?;
        assert_eq!(bytes_written, TO_SEND);

        Ok(())
    }
}
