use futures::{channel::mpsc::UnboundedReceiver, pin_mut, StreamExt};
use hopr_crypto_packet::prelude::HoprPacket;
use hopr_internal_types::prelude::{HoprPseudonym, Tag};
use hopr_internal_types::protocol::ApplicationData;
use hopr_network_types::prelude::{DestinationRouting, SealedHost};
use hopr_network_types::session::state::{SessionConfig, SessionSocket};
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::atomic::AtomicU64;
use std::task::Context;
use std::time::Duration;
use std::{
    fmt::Display,
    io::{Error, ErrorKind},
    pin::Pin,
    sync::Arc,
    task::Poll,
};
use tracing::{debug, error};

use crate::{errors::TransportSessionError, traits::SendMsg, Capability};

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

// Enough to fit Ed25519 peer IDs and 2-byte tag number
const MAX_SESSION_ID_STR_LEN: usize = 64;

/// Unique ID of a specific session.
///
/// Simple wrapper around the maximum range of the port like session unique identifier.
/// It is a simple combination of an application tag and a peer id that will in future be
/// replaced by a more robust session id representation.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SessionId {
    tag: Tag,
    pseudonym: HoprPseudonym,
    // Since this SessionId is commonly represented as a string,
    // we cache its string representation here.
    // Also by using a statically allocated ArrayString, we allow the SessionId to remain Copy.
    // This representation is possibly truncated to MAX_SESSION_ID_STR_LEN.
    cached: arrayvec::ArrayString<MAX_SESSION_ID_STR_LEN>,
}

impl SessionId {
    pub fn new(tag: Tag, pseudonym: HoprPseudonym) -> Self {
        let mut cached = format!("{pseudonym}:{tag}");
        cached.truncate(MAX_SESSION_ID_STR_LEN);

        Self {
            tag,
            pseudonym,
            cached: cached.parse().expect("cannot fail due to truncation"),
        }
    }

    pub fn tag(&self) -> u16 {
        self.tag
    }

    pub fn pseudonym(&self) -> &HoprPseudonym {
        &self.pseudonym
    }

    pub fn as_str(&self) -> &str {
        &self.cached
    }
}

impl Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cached)
    }
}

impl Debug for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cached)
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

/// Inner MTU size of what the HOPR packet can tag as payload
pub const SESSION_USABLE_MTU_SIZE: usize = HoprPacket::PAYLOAD_SIZE - size_of::<Tag>();

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
    capabilities: HashSet<Capability>,
    on_close: Option<Box<dyn FnOnce(SessionId) + Send + Sync>>,
}

impl Session {
    pub fn new(
        id: SessionId,
        routing: DestinationRouting,
        capabilities: HashSet<Capability>,
        tx: Arc<dyn SendMsg + Send + Sync>,
        rx: UnboundedReceiver<Box<[u8]>>,
        on_close: Option<Box<dyn FnOnce(SessionId) + Send + Sync>>,
        rx_packet_counter: Arc<AtomicU64>,
    ) -> Self {
        let inner_session = InnerSession::new(id, routing.clone(), tx, rx, rx_packet_counter);

        // If we request any capability, we need to use Session protocol
        if !capabilities.is_empty() {
            // This is a very coarse assumption, that 3-hop takes at most 3 seconds.
            // We can no longer base this timeout on the number of hops because
            // it is not known for SURB-based routing.
            let rto_base = Duration::from_secs(3);

            let expiration_coefficient = if capabilities.contains(&Capability::Retransmission)
                || capabilities.contains(&Capability::RetransmissionAckOnly)
            {
                4
            } else {
                1
            };

            // TODO: tweak the default Session protocol config
            let cfg = SessionConfig {
                enabled_features: capabilities.iter().cloned().flatten().collect(),
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
                inner: Box::pin(SessionSocket::<SESSION_USABLE_MTU_SIZE>::new(id, inner_session, cfg)),
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
    pub fn capabilities(&self) -> &HashSet<Capability> {
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
    rx: UnboundedReceiver<Box<[u8]>>,
    tx: Arc<dyn SendMsg + Send + Sync>,
    tx_bytes: usize,
    tx_buffer: FuturesBuffer,
    rx_buffer: [u8; HoprPacket::PAYLOAD_SIZE],
    rx_buffer_range: (usize, usize),
    rx_packet_counter: Arc<AtomicU64>,
    closed: bool,
}

impl InnerSession {
    pub fn new(
        id: SessionId,
        routing: DestinationRouting,
        tx: Arc<dyn SendMsg + Send + Sync>,
        rx: UnboundedReceiver<Box<[u8]>>, // TODO: change this so it uses Box<dyn Stream<...>>
        rx_packet_counter: Arc<AtomicU64>,
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
            rx_packet_counter,
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
                    Poll::Ready(Some(Err(e))) => {
                        error!(error = %e, "failed to send the message chunk inside a session");
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

        for i in 0..(buf.len() / SESSION_USABLE_MTU_SIZE + ((buf.len() % SESSION_USABLE_MTU_SIZE != 0) as usize)) {
            let start = i * SESSION_USABLE_MTU_SIZE;
            let end = ((i + 1) * SESSION_USABLE_MTU_SIZE).min(buf.len());

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
                Poll::Ready(Some(Err(error))) => {
                    error!(%error, "failed to send the message chunk inside a session");
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
                error!(%error, "failed to send message chunk inside session during flush");
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
                self.rx_packet_counter
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                Poll::Ready(Ok(copy_len))
            }
            Poll::Ready(None) => {
                self.rx.close(); // TODO: see if this close is needed here
                Poll::Ready(Ok(0)) // due to convention, Ok(0) indicates EOF
            }
            //Poll::Ready(None) => Poll::Ready(Err(Error::from(ErrorKind::NotConnected))),
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
    // 1) If Session protocol is used for segmentation,
    //    we need to buffer up data at MAX_WRITE_SIZE.
    // 2) Otherwise, the bare session implements chunking, therefore,
    //    data can be written with arbitrary sizes.
    let into_session_len = if session.capabilities().contains(&Capability::Segmentation) {
        max_buffer.min(SessionSocket::<SESSION_USABLE_MTU_SIZE>::MAX_WRITE_SIZE)
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
    use super::*;
    use crate::traits::MockSendMsg;
    use futures::{AsyncReadExt, AsyncWriteExt};
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
    use hopr_network_types::prelude::{RoutingOptions, SurbMatcher};
    use hopr_primitive_types::prelude::Address;

    #[test]
    fn session_should_identify_with_its_own_id() -> anyhow::Result<()> {
        let addr: Address = (&ChainKeypair::random()).into();
        let id = SessionId::new(1, HoprPseudonym::random());
        let (_tx, rx) = futures::channel::mpsc::unbounded();
        let mock = MockSendMsg::new();

        let session = InnerSession::new(
            id,
            DestinationRouting::forward_only(addr, RoutingOptions::Hops(1_u32.try_into()?)),
            Arc::new(mock),
            rx,
            Arc::new(AtomicU64::new(0)),
        );

        assert_eq!(session.id, id);

        Ok(())
    }

    #[async_std::test]
    async fn session_should_read_data_in_one_swoop_if_the_buffer_is_sufficiently_large() -> anyhow::Result<()> {
        let addr: Address = (&ChainKeypair::random()).into();
        let id = SessionId::new(1, HoprPseudonym::random());
        let (tx, rx) = futures::channel::mpsc::unbounded();
        let mock = MockSendMsg::new();

        let mut session = InnerSession::new(
            id,
            DestinationRouting::forward_only(addr, RoutingOptions::Hops(1_u32.try_into()?)),
            Arc::new(mock),
            rx,
            Arc::new(AtomicU64::new(0)),
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

    #[async_std::test]
    async fn session_should_read_data_in_multiple_rounds_if_the_buffer_is_not_sufficiently_large() -> anyhow::Result<()>
    {
        let addr: Address = (&ChainKeypair::random()).into();
        let id = SessionId::new(1, HoprPseudonym::random());
        let (tx, rx) = futures::channel::mpsc::unbounded();
        let mock = MockSendMsg::new();

        let mut session = InnerSession::new(
            id,
            DestinationRouting::forward_only(addr, RoutingOptions::Hops(1_u32.try_into()?)),
            Arc::new(mock),
            rx,
            Arc::new(AtomicU64::new(0)),
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

    #[async_std::test]
    async fn session_should_write_data_on_forward_path() -> anyhow::Result<()> {
        let addr: Address = (&ChainKeypair::random()).into();
        let id = SessionId::new(1, HoprPseudonym::random());
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
            rx,
            Arc::new(AtomicU64::new(0)),
        );

        let bytes_written = session.write(&data).await?;
        assert_eq!(bytes_written, data.len());

        Ok(())
    }

    #[async_std::test]
    async fn session_should_write_data_on_return_path() -> anyhow::Result<()> {
        let id = SessionId::new(1, HoprPseudonym::random());
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
            rx,
            Arc::new(AtomicU64::new(0)),
        );

        let bytes_written = session.write(&data).await?;
        assert_eq!(bytes_written, data.len());

        Ok(())
    }

    #[async_std::test]
    async fn session_should_chunk_the_data_if_without_segmentation_the_write_size_is_greater_than_the_usable_mtu_size(
    ) -> anyhow::Result<()> {
        const TO_SEND: usize = SESSION_USABLE_MTU_SIZE * 2 + 10;

        let addr: Address = (&ChainKeypair::random()).into();
        let id = SessionId::new(1, HoprPseudonym::random());
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
            rx,
            Arc::new(AtomicU64::new(0)),
        );

        let bytes_written = session.write(&data).await?;
        assert_eq!(bytes_written, TO_SEND);

        Ok(())
    }
}
