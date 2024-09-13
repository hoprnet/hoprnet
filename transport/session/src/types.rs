use crate::{errors::TransportSessionError, traits::SendMsg, Capability};
use futures::{channel::mpsc::UnboundedReceiver, pin_mut, StreamExt};
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_internal_types::protocol::{ApplicationData, PAYLOAD_SIZE};
use hopr_network_types::prelude::state::SessionFeature;
use hopr_network_types::prelude::{IpOrHost, RoutingOptions};
use hopr_network_types::session::state::{SessionConfig, SessionSocket};
use hopr_primitive_types::traits::BytesRepresentable;
use libp2p_identity::PeerId;
use std::collections::HashSet;
use std::fmt::Formatter;
use std::time::Duration;
use std::{
    fmt::Display,
    io::{Error, ErrorKind},
    pin::Pin,
    sync::Arc,
    task::Poll,
};
use strum::IntoEnumIterator;
use tracing::{debug, error};

/// Unique ID of a specific session.
///
/// Simple wrapper around the maximum range of the port like session unique identifier.
/// It is a simple combination of an application tag and a peer id that will in future be
/// replaced by a more robust session id representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SessionId {
    tag: u16,
    peer: PeerId,
}

impl SessionId {
    pub fn new(tag: u16, peer: PeerId) -> Self {
        Self { tag, peer }
    }

    pub fn tag(&self) -> u16 {
        self.tag
    }

    pub fn peer(&self) -> &PeerId {
        &self.peer
    }

    pub fn with_peer(mut self, peer: PeerId) -> Self {
        self.peer = peer;
        self
    }
}

impl Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.peer, self.tag)
    }
}

const PADDING_HEADER_SIZE: usize = 4;
// Inner MTU size of what the HOPR payload can take (payload - peer_id - application_tag)
pub const SESSION_USABLE_MTU_SIZE: usize = PAYLOAD_SIZE
    - OffchainPublicKey::SIZE
    - std::mem::size_of::<hopr_internal_types::protocol::Tag>()
    - PADDING_HEADER_SIZE;

/// Helper trait to allow Box aliasing
trait AsyncReadWrite: futures::AsyncWrite + futures::AsyncRead + Send {}
impl<T: futures::AsyncWrite + futures::AsyncRead + Send> AsyncReadWrite for T {}

/// Defines what should happen with the data at the recipient where the
/// data from the established session are supposed to be forwarded to some `target`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SessionTarget {
    /// Target is running over UDP with the given IP address and port.
    UdpStream(IpOrHost),
    /// Target is running over TCP with the given address and port.
    TcpStream(IpOrHost),
    /// Target is a service directly at the exit node with a given service ID.
    ExitNode(u32),
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
    routing_options: RoutingOptions,
    shutdown_notifier: Option<futures::channel::mpsc::UnboundedSender<SessionId>>,
}

impl Session {
    pub fn new(
        id: SessionId,
        me: PeerId,
        routing_options: RoutingOptions,
        capabilities: HashSet<Capability>,
        tx: Arc<dyn SendMsg + Send + Sync>,
        rx: UnboundedReceiver<Box<[u8]>>,
        shutdown_notifier: Option<futures::channel::mpsc::UnboundedSender<SessionId>>,
    ) -> Self {
        let inner_session = InnerSession::new(id, me, routing_options.clone(), tx, rx);

        // If we request any capability, we need to use Session protocol
        if Capability::iter().any(|c| capabilities.contains(&c)) {
            // In addition, if retransmission is requested,
            // we need to enable more Session protocol features.
            let mut enabled_features = HashSet::new();
            if capabilities.contains(&Capability::Retransmission) {
                enabled_features.extend([
                    SessionFeature::AcknowledgeFrames,
                    SessionFeature::RequestIncompleteFrames,
                    SessionFeature::RetransmitFrames,
                ]);
            }

            // TODO: tweak the default Session protocol config
            let cfg = SessionConfig {
                enabled_features,
                acknowledged_frames_buffer: 10_000,
                frame_expiration_age: Duration::from_secs(60),
                rto_base_receiver: Duration::from_secs(20),
                rto_base_sender: Duration::from_secs(30),
                ..Default::default()
            };
            debug!(
                session_id = tracing::field::debug(id),
                "opening new session socket with {cfg:?}"
            );

            Self {
                id,
                inner: Box::pin(SessionSocket::<SESSION_USABLE_MTU_SIZE>::new(id, inner_session, cfg)),
                routing_options,
                shutdown_notifier,
            }
        } else {
            // Otherwise, no additional sub protocol is necessary
            Self {
                id,
                inner: Box::pin(inner_session),
                routing_options,
                shutdown_notifier,
            }
        }
    }

    /// ID of this Session.
    pub fn id(&self) -> &SessionId {
        &self.id
    }

    /// Routing options used to deliver data.
    pub fn routing_options(&self) -> &RoutingOptions {
        &self.routing_options
    }
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("id", &self.id)
            .field("routing_options", &self.routing_options)
            .finish_non_exhaustive()
    }
}

impl futures::AsyncRead for Session {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let inner = self.inner.as_mut();
        pin_mut!(inner);
        inner.poll_read(cx, buf)
    }
}

impl futures::AsyncWrite for Session {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let inner = &mut self.inner;
        pin_mut!(inner);
        inner.poll_write(cx, buf)
    }

    fn poll_flush(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<std::io::Result<()>> {
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
                if let Some(notifier) = self.shutdown_notifier.take() {
                    if let Err(err) = notifier.unbounded_send(self.id) {
                        error!(
                            session_id = tracing::field::debug(self.id),
                            "failed to notify session closure: {err}"
                        );
                    }
                }
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

type FuturesBuffer = futures::stream::FuturesUnordered<
    Pin<Box<dyn std::future::Future<Output = Result<(), TransportSessionError>> + Send>>,
>;
pub struct InnerSession {
    id: SessionId,
    me: PeerId,
    options: RoutingOptions,
    rx: UnboundedReceiver<Box<[u8]>>,
    tx: Arc<dyn SendMsg + Send + Sync>,
    tx_bytes: usize,
    tx_buffer: FuturesBuffer,
    rx_buffer: [u8; PAYLOAD_SIZE],
    rx_buffer_range: (usize, usize),
}

impl InnerSession {
    pub fn new(
        id: SessionId,
        me: PeerId,
        options: RoutingOptions,
        tx: Arc<dyn SendMsg + Send + Sync>,
        rx: UnboundedReceiver<Box<[u8]>>,
    ) -> Self {
        Self {
            id,
            me,
            options,
            rx,
            tx,
            tx_bytes: 0,
            tx_buffer: futures::stream::FuturesUnordered::new(),
            rx_buffer: [0; PAYLOAD_SIZE],
            rx_buffer_range: (0, 0),
        }
    }

    pub fn id(&self) -> &SessionId {
        &self.id
    }
}

impl futures::AsyncWrite for InnerSession {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        if !self.tx_buffer.is_empty() {
            loop {
                match self.tx_buffer.poll_next_unpin(cx) {
                    Poll::Ready(Some(Ok(()))) => {
                        continue;
                    }
                    Poll::Ready(Some(Err(e))) => {
                        error!("failed to send the message chunk inside a session: {e}");
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

            let payload = wrap_with_offchain_key(&self.me, buf[start..end].to_vec().into_boxed_slice())
                .map_err(|e| {
                    error!("failed to wrap the payload with offchain key: {e}");
                    Error::new(ErrorKind::InvalidData, e)
                })
                .and_then(move |payload| {
                    ApplicationData::new_from_owned(Some(tag), payload.into_boxed_slice()).map_err(|e| {
                        error!("failed to extract application data from the payload: {e}");
                        Error::new(ErrorKind::InvalidData, e)
                    })
                })?;

            let sender = self.tx.clone();
            let peer_id = *self.id.peer();
            let options = self.options.clone();

            self.tx_buffer.push(Box::pin(
                async move { sender.send_message(payload, peer_id, options).await },
            ));

            self.tx_bytes += end - start;
        }

        loop {
            match self.tx_buffer.poll_next_unpin(cx) {
                Poll::Ready(Some(Ok(_))) => {
                    continue;
                }
                Poll::Ready(Some(Err(e))) => {
                    error!("failed to send the message chunk inside a session: {e}");
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

    fn poll_flush(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<std::io::Result<()>> {
        self.tx.close();
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
                self.rx.close();
                Poll::Ready(Ok(0)) // due to convention, Ok(0) indicates EOF
            }
            //Poll::Ready(None) => Poll::Ready(Err(Error::from(ErrorKind::NotConnected))),
            Poll::Pending => Poll::Pending,
        }
    }
}

// TODO: 3.0 remove once return path is implemented
pub fn wrap_with_offchain_key(peer: &PeerId, data: Box<[u8]>) -> crate::errors::Result<Vec<u8>> {
    if data.len() > PAYLOAD_SIZE.saturating_sub(OffchainPublicKey::SIZE + PADDING_HEADER_SIZE) {
        return Err(TransportSessionError::PayloadSize);
    }

    let opk = OffchainPublicKey::try_from(peer).map_err(|_e| TransportSessionError::PeerId)?;

    let mut packet: Vec<u8> = Vec::with_capacity(PAYLOAD_SIZE);
    packet.extend_from_slice(opk.as_ref());
    packet.extend_from_slice(data.as_ref());

    Ok(packet)
}

// TODO: 3.0 remove if return path is implemented
pub fn unwrap_offchain_key(payload: Box<[u8]>) -> crate::errors::Result<(PeerId, Box<[u8]>)> {
    if payload.len() > PAYLOAD_SIZE {
        return Err(TransportSessionError::PayloadSize);
    }

    let mut payload = payload.into_vec();
    let data = payload.split_off(OffchainPublicKey::SIZE).into_boxed_slice();

    let opk = OffchainPublicKey::try_from(payload.as_slice()).map_err(|_e| TransportSessionError::PeerId)?;

    Ok((opk.into(), data))
}

#[cfg(test)]
mod tests {
    use futures::{AsyncReadExt, AsyncWriteExt};
    use hopr_crypto_types::keypairs::{Keypair, OffchainKeypair};

    use super::*;
    use crate::traits::MockSendMsg;

    #[test]
    fn use_the_offchain_binary_form_because_it_is_more_compact() {
        let opk = OffchainKeypair::random().public().clone();
        let peer: PeerId = OffchainKeypair::random().public().into();

        assert!(opk.as_ref().len() < peer.to_bytes().len());
    }

    #[test]
    fn wrapping_and_unwrapping_with_offchain_key_should_be_an_identity() -> anyhow::Result<()> {
        let peer: PeerId = OffchainKeypair::random().public().into();
        let data = hopr_crypto_random::random_bytes::<SESSION_USABLE_MTU_SIZE>()
            .as_ref()
            .to_vec()
            .into_boxed_slice();

        let wrapped = wrap_with_offchain_key(&peer, data.clone())?;

        let (peer_id, unwrapped) = unwrap_offchain_key(wrapped.into_boxed_slice())?;

        assert_eq!(peer, peer_id);
        assert_eq!(data, unwrapped);

        Ok(())
    }

    #[test]
    fn wrapping_with_offchain_key_should_succeed_for_valid_peer_id_and_valid_payload_size() {
        let peer: PeerId = OffchainKeypair::random().public().into();
        let data = hopr_crypto_random::random_bytes::<SESSION_USABLE_MTU_SIZE>()
            .as_ref()
            .to_vec()
            .into_boxed_slice();

        let wrapped = wrap_with_offchain_key(&peer, data.clone());

        assert!(matches!(wrapped, Ok(_)));
    }

    #[test]
    fn wrapping_with_offchain_key_should_fail_for_invalid_peer_id() {
        let peer: PeerId = PeerId::random();
        let data = hopr_crypto_random::random_bytes::<SESSION_USABLE_MTU_SIZE>()
            .as_ref()
            .to_vec()
            .into_boxed_slice();

        let wrapped = wrap_with_offchain_key(&peer, data.clone());

        assert!(matches!(wrapped, Err(TransportSessionError::PeerId)));
    }

    #[test]
    fn wrapping_with_offchain_key_should_fail_for_invalid_payload_size() {
        const INVALID_PAYLOAD_SIZE: usize = PAYLOAD_SIZE + 1;
        let peer: PeerId = OffchainKeypair::random().public().into();
        let data = hopr_crypto_random::random_bytes::<INVALID_PAYLOAD_SIZE>()
            .as_ref()
            .to_vec()
            .into_boxed_slice();

        let wrapped = wrap_with_offchain_key(&peer, data.clone());

        assert!(matches!(wrapped, Err(TransportSessionError::PayloadSize)));
    }

    #[test]
    fn unwrapping_offchain_key_should_fail_for_invalid_payload_size() {
        const INVALID_PAYLOAD_SIZE: usize = PAYLOAD_SIZE + 1;
        let data = hopr_crypto_random::random_bytes::<INVALID_PAYLOAD_SIZE>()
            .as_ref()
            .to_vec()
            .into_boxed_slice();

        let unwrapped = unwrap_offchain_key(data.clone());

        assert!(matches!(unwrapped, Err(TransportSessionError::PayloadSize)));
    }

    #[test]
    fn session_should_identify_with_its_own_id() -> anyhow::Result<()> {
        let id = SessionId::new(1, PeerId::random());
        let (_tx, rx) = futures::channel::mpsc::unbounded();
        let mock = MockSendMsg::new();

        let session = InnerSession::new(
            id,
            PeerId::random(),
            RoutingOptions::Hops(1_u32.try_into()?),
            Arc::new(mock),
            rx,
        );

        assert_eq!(session.id(), &id);

        Ok(())
    }

    #[async_std::test]
    async fn session_should_read_data_in_one_swoop_if_the_buffer_is_sufficiently_large() -> anyhow::Result<()> {
        let id = SessionId::new(1, PeerId::random());
        let (tx, rx) = futures::channel::mpsc::unbounded();
        let mock = MockSendMsg::new();

        let mut session = InnerSession::new(
            id,
            PeerId::random(),
            RoutingOptions::Hops(1_u32.try_into()?),
            Arc::new(mock),
            rx,
        );

        let random_data = hopr_crypto_random::random_bytes::<PAYLOAD_SIZE>()
            .as_ref()
            .to_vec()
            .into_boxed_slice();

        assert!(tx.unbounded_send(random_data.clone()).is_ok());

        let mut buffer = vec![0; PAYLOAD_SIZE * 2];

        let bytes_read = session.read(&mut buffer[..]).await?;

        assert_eq!(bytes_read, random_data.len());
        assert_eq!(&buffer[..bytes_read], random_data.as_ref());

        Ok(())
    }

    #[async_std::test]
    async fn session_should_read_data_in_multiple_rounds_if_the_buffer_is_not_sufficiently_large() -> anyhow::Result<()>
    {
        let id = SessionId::new(1, PeerId::random());
        let (tx, rx) = futures::channel::mpsc::unbounded();
        let mock = MockSendMsg::new();

        let mut session = InnerSession::new(
            id,
            PeerId::random(),
            RoutingOptions::Hops(1_u32.try_into()?),
            Arc::new(mock),
            rx,
        );

        let random_data = hopr_crypto_random::random_bytes::<PAYLOAD_SIZE>()
            .as_ref()
            .to_vec()
            .into_boxed_slice();

        assert!(tx.unbounded_send(random_data.clone()).is_ok());

        const BUFFER_SIZE: usize = PAYLOAD_SIZE - 1;
        let mut buffer = vec![0; BUFFER_SIZE];

        let bytes_read = session.read(&mut buffer[..]).await?;

        assert_eq!(bytes_read, BUFFER_SIZE);
        assert_eq!(&buffer[..bytes_read], &random_data[..BUFFER_SIZE]);

        let bytes_read = session.read(&mut buffer[..]).await?;

        assert_eq!(bytes_read, PAYLOAD_SIZE - BUFFER_SIZE);
        assert_eq!(&buffer[..bytes_read], &random_data[BUFFER_SIZE..]);

        Ok(())
    }

    #[async_std::test]
    async fn session_should_write_data() -> anyhow::Result<()> {
        let id = SessionId::new(1, OffchainKeypair::random().public().into());
        let (_tx, rx) = futures::channel::mpsc::unbounded();
        let mut mock = MockSendMsg::new();

        let data = b"Hello, world!".to_vec().into_boxed_slice();

        mock.expect_send_message()
            .times(1)
            .withf(move |data, _peer, options| {
                let (_peer_id, data) = unwrap_offchain_key(data.plain_text.clone()).expect("Unwrapping should work");
                assert_eq!(data, b"Hello, world!".to_vec().into_boxed_slice());
                assert_eq!(
                    options,
                    &RoutingOptions::Hops(1_u32.try_into().expect("must be convertible"))
                );
                true
            })
            .returning(|_, _, _| Ok(()));

        let mut session = InnerSession::new(
            id,
            OffchainKeypair::random().public().into(),
            RoutingOptions::Hops(1_u32.try_into()?),
            Arc::new(mock),
            rx,
        );

        let bytes_written = session.write(&data).await?;

        assert_eq!(bytes_written, data.len());

        Ok(())
    }

    #[async_std::test]
    async fn session_should_chunk_the_data_if_without_segmentation_the_write_size_is_greater_than_the_usable_mtu_size(
    ) -> anyhow::Result<()> {
        const TO_SEND: usize = SESSION_USABLE_MTU_SIZE * 2 + 10;

        let id = SessionId::new(1, OffchainKeypair::random().public().into());
        let (_tx, rx) = futures::channel::mpsc::unbounded();
        let mut mock = MockSendMsg::new();

        let data = hopr_crypto_random::random_bytes::<TO_SEND>()
            .as_ref()
            .to_vec()
            .into_boxed_slice();

        mock.expect_send_message().times(3).returning(|_, _, _| Ok(()));

        let mut session = InnerSession::new(
            id,
            OffchainKeypair::random().public().into(),
            RoutingOptions::Hops(1_u32.try_into()?),
            Arc::new(mock),
            rx,
        );

        let bytes_written = session.write(&data).await?;

        assert_eq!(bytes_written, TO_SEND);

        Ok(())
    }
}
