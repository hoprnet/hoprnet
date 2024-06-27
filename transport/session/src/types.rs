use std::{
    io::{Error, ErrorKind},
    task::Poll,
};

use futures::{channel::mpsc::UnboundedReceiver, FutureExt, StreamExt};
use libp2p_identity::PeerId;

use hopr_crypto_types::types::OffchainPublicKey;
use hopr_internal_types::protocol::{ApplicationData, PAYLOAD_SIZE};

use crate::{errors::TransportSessionError, traits::SendMsg, PathOptions};

/// ID tracking the session uniquely.
///
/// Simple wrapper around the maximum range of the port like session unique identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
}

pub struct Session {
    id: SessionId,
    me: PeerId,
    options: PathOptions,
    rx: UnboundedReceiver<Box<[u8]>>,
    tx: Box<dyn SendMsg + Send>,
    rx_buffer: [u8; PAYLOAD_SIZE],
    rx_buffer_range: (usize, usize),
}

impl Session {
    pub fn new(
        id: SessionId,
        me: PeerId,
        options: PathOptions,
        tx: Box<dyn SendMsg + Send>,
        rx: UnboundedReceiver<Box<[u8]>>,
    ) -> Self {
        Self {
            id,
            me,
            options,
            rx,
            tx,
            rx_buffer: [0; PAYLOAD_SIZE],
            rx_buffer_range: (0, 0),
        }
    }

    pub fn id(&self) -> &SessionId {
        &self.id
    }
}

impl futures::AsyncWrite for Session {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let tag = self.id.tag();
        let payload = wrap_with_offchain_key(&self.me, buf.to_vec().into_boxed_slice())
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))
            .and_then(move |payload| {
                ApplicationData::new_from_owned(Some(tag), payload.into_boxed_slice())
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e))
            })?;

        match self
            .tx
            .send_message(payload, *self.id.peer(), self.options.clone())
            .poll_unpin(cx)
        {
            Poll::Ready(Ok(_)) => Poll::Ready(Ok(buf.len())),
            Poll::Ready(Err(_)) => Poll::Ready(Err(Error::from(ErrorKind::BrokenPipe))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

impl futures::AsyncRead for Session {
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
            Poll::Ready(None) => Poll::Ready(Err(Error::from(ErrorKind::NotConnected))),
            Poll::Pending => Poll::Pending,
        }
    }
}

// TODO: 2.2 use a more compact representation of the PeerId in the binary form
const PEER_ID_USED_SIZE: usize = 38;

// TODO: 2.2 use a more compact representation of the PeerId in the binary form
// TODO: 3.0 remove if return path is implemented
pub fn wrap_with_offchain_key(peer: &PeerId, data: Box<[u8]>) -> crate::errors::Result<Vec<u8>> {
    if data.len() > PAYLOAD_SIZE.saturating_sub(PEER_ID_USED_SIZE) {
        return Err(TransportSessionError::PayloadSize);
    }

    let _ = OffchainPublicKey::try_from(peer).map_err(|_e| TransportSessionError::PeerId)?;

    let mut packet: Vec<u8> = Vec::with_capacity(PAYLOAD_SIZE);
    packet.extend_from_slice(peer.to_bytes().as_ref());
    packet.extend_from_slice(data.as_ref());

    Ok(packet)
}

// TODO: 2.2 use a more compact representation of the PeerId in the binary form
// TODO: 3.0 remove if return path is implemented
pub fn unwrap_offchain_key(payload: Box<[u8]>) -> crate::errors::Result<(PeerId, Box<[u8]>)> {
    if payload.len() > PAYLOAD_SIZE {
        return Err(TransportSessionError::PayloadSize);
    }

    let mut peer = payload.into_vec();
    let data = peer.split_off(PEER_ID_USED_SIZE).into_boxed_slice();

    let peer = PeerId::try_from(peer).map_err(|_e| TransportSessionError::PeerId)?;

    let _ = OffchainPublicKey::try_from(peer).map_err(|_e| TransportSessionError::PeerId)?;

    Ok((peer, data))
}

#[cfg(test)]
mod tests {
    use futures::{AsyncReadExt, AsyncWriteExt};
    use hopr_crypto_types::keypairs::{Keypair, OffchainKeypair};

    use super::*;
    use crate::traits::MockSendMsg;

    const PAYLOAD: usize = PAYLOAD_SIZE - PEER_ID_USED_SIZE;

    #[test]
    fn wrapping_and_unwrapping_with_offchain_key_should_be_an_identity() {
        let peer: PeerId = OffchainKeypair::random().public().into();
        let data = hopr_crypto_random::random_bytes::<PAYLOAD>()
            .as_ref()
            .to_vec()
            .into_boxed_slice();

        let wrapped = wrap_with_offchain_key(&peer, data.clone()).expect("Wrapping should work");

        let (peer_id, unwrapped) = unwrap_offchain_key(wrapped.into_boxed_slice()).expect("Unwrapping should work");

        assert_eq!(peer, peer_id);
        assert_eq!(data, unwrapped);
    }

    #[test]
    fn wrapping_with_offchain_key_should_succeed_for_valid_peer_id_and_valid_payload_size() {
        let peer: PeerId = OffchainKeypair::random().public().into();
        let data = hopr_crypto_random::random_bytes::<PAYLOAD>()
            .as_ref()
            .to_vec()
            .into_boxed_slice();

        let wrapped = wrap_with_offchain_key(&peer, data.clone());

        assert!(matches!(wrapped, Ok(_)));
    }

    #[test]
    fn wrapping_with_offchain_key_should_fail_for_invalid_peer_id() {
        let peer: PeerId = PeerId::random();
        let data = hopr_crypto_random::random_bytes::<PAYLOAD>()
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
    fn unwrapping_offchain_key_should_fail_for_invalid_peer_id() {
        let data = hopr_crypto_random::random_bytes::<PAYLOAD_SIZE>()
            .as_ref()
            .to_vec()
            .into_boxed_slice();

        let unwrapped = unwrap_offchain_key(data.clone());

        assert!(matches!(unwrapped, Err(TransportSessionError::PeerId)));
    }

    #[test]
    fn session_should_identify_with_its_own_id() {
        let id = SessionId::new(1, PeerId::random());
        let (_tx, rx) = futures::channel::mpsc::unbounded();
        let mock = MockSendMsg::new();

        let session = Session::new(id, PeerId::random(), PathOptions::Hops(1), Box::new(mock), rx);

        assert_eq!(session.id(), &id);
    }

    #[async_std::test]
    async fn session_should_read_data_in_one_swoop_if_the_buffer_is_sufficiently_large() {
        let id = SessionId::new(1, PeerId::random());
        let (tx, rx) = futures::channel::mpsc::unbounded();
        let mock = MockSendMsg::new();

        let mut session = Session::new(id, PeerId::random(), PathOptions::Hops(1), Box::new(mock), rx);

        let random_data = hopr_crypto_random::random_bytes::<PAYLOAD_SIZE>()
            .as_ref()
            .to_vec()
            .into_boxed_slice();

        assert!(tx.unbounded_send(random_data.clone()).is_ok());

        let mut buffer = vec![0; PAYLOAD_SIZE * 2];

        let bytes_read = session.read(&mut buffer[..]).await.expect("Read should work");

        assert_eq!(bytes_read, random_data.len());
        assert_eq!(&buffer[..bytes_read], random_data.as_ref());
    }

    #[async_std::test]
    async fn session_should_read_data_in_multiple_rounds_if_the_buffer_is_not_sufficiently_large() {
        let id = SessionId::new(1, PeerId::random());
        let (tx, rx) = futures::channel::mpsc::unbounded();
        let mock = MockSendMsg::new();

        let mut session = Session::new(id, PeerId::random(), PathOptions::Hops(1), Box::new(mock), rx);

        let random_data = hopr_crypto_random::random_bytes::<PAYLOAD_SIZE>()
            .as_ref()
            .to_vec()
            .into_boxed_slice();

        assert!(tx.unbounded_send(random_data.clone()).is_ok());

        const BUFFER_SIZE: usize = PAYLOAD_SIZE - 1;
        let mut buffer = vec![0; BUFFER_SIZE];

        let bytes_read = session.read(&mut buffer[..]).await.expect("Read should work #1");

        assert_eq!(bytes_read, BUFFER_SIZE);
        assert_eq!(&buffer[..bytes_read], &random_data[..BUFFER_SIZE]);

        let bytes_read = session.read(&mut buffer[..]).await.expect("Read should work #1");

        assert_eq!(bytes_read, PAYLOAD_SIZE - BUFFER_SIZE);
        assert_eq!(&buffer[..bytes_read], &random_data[BUFFER_SIZE..]);
    }

    #[async_std::test]
    async fn session_should_write_data() {
        let id = SessionId::new(1, OffchainKeypair::random().public().into());
        let (_tx, rx) = futures::channel::mpsc::unbounded();
        let mut mock = MockSendMsg::new();

        let data = b"Hello, world!".to_vec().into_boxed_slice();

        mock.expect_send_message()
            .times(1)
            .withf(move |data, _peer, options| {
                let (_peer_id, data) = unwrap_offchain_key(data.plain_text.clone()).expect("Unwrapping should work");
                assert_eq!(data, b"Hello, world!".to_vec().into_boxed_slice());
                assert_eq!(options, &PathOptions::Hops(1));
                true
            })
            .returning(|_, _, _| Ok(()));

        let mut session = Session::new(
            id,
            OffchainKeypair::random().public().into(),
            PathOptions::Hops(1),
            Box::new(mock),
            rx,
        );

        let bytes_written = session.write(&data).await.expect("Write should work #1");

        assert_eq!(bytes_written, data.len());
    }
}
