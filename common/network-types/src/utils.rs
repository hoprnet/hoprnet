use std::{
    fmt::{Debug, Display, Formatter},
    hash::{Hash, Hasher},
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

use futures::io::{AsyncRead, AsyncWrite};

/// Joins [futures::AsyncRead] and [futures::AsyncWrite] into a single object.
pub struct DuplexIO<R, W>(pub R, pub W);

impl<R, W> From<(R, W)> for DuplexIO<R, W>
where
    R: AsyncRead,
    W: AsyncWrite,
{
    fn from(value: (R, W)) -> Self {
        Self(value.0, value.1)
    }
}

impl<R, W> AsyncRead for DuplexIO<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        let this = self.get_mut();
        Pin::new(&mut this.0).poll_read(cx, buf)
    }
}

impl<R, W> AsyncWrite for DuplexIO<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        let this = self.get_mut();
        Pin::new(&mut this.1).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();
        Pin::new(&mut this.1).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();
        Pin::new(&mut this.1).poll_close(cx)
    }
}

// IPv6 + ':' + 65535 = 45 + 1 + 5
const SOCKET_ADDRESS_MAX_LEN: usize = 52;

/// Caches the string representation of a SocketAddr for fast conversion to `&str`
#[derive(Copy, Clone)]
pub(crate) struct SocketAddrStr(SocketAddr, arrayvec::ArrayString<SOCKET_ADDRESS_MAX_LEN>);

impl SocketAddrStr {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        self.1.as_str()
    }
}

impl AsRef<SocketAddr> for SocketAddrStr {
    fn as_ref(&self) -> &SocketAddr {
        &self.0
    }
}

impl From<SocketAddr> for SocketAddrStr {
    fn from(value: SocketAddr) -> Self {
        let mut cached = value.to_string();
        cached.truncate(SOCKET_ADDRESS_MAX_LEN);
        Self(value, cached.parse().expect("cannot fail due to truncation"))
    }
}

impl PartialEq for SocketAddrStr {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for SocketAddrStr {}

impl Debug for SocketAddrStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.1)
    }
}

impl Display for SocketAddrStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.1)
    }
}

impl PartialEq<SocketAddrStr> for SocketAddr {
    fn eq(&self, other: &SocketAddrStr) -> bool {
        self.eq(&other.0)
    }
}

impl Hash for SocketAddrStr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[cfg(feature = "runtime-tokio")]
pub use tokio_utils::copy_duplex;

#[cfg(feature = "runtime-tokio")]
mod tokio_utils {
    use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

    use super::*;

    #[derive(Debug)]
    enum TransferState {
        Running(CopyBuffer),
        ShuttingDown(u64),
        Done(u64),
    }

    fn transfer_one_direction<A, B>(
        cx: &mut Context<'_>,
        state: &mut TransferState,
        r: &mut A,
        w: &mut B,
    ) -> Poll<std::io::Result<u64>>
    where
        A: AsyncRead + AsyncWrite + Unpin + ?Sized,
        B: AsyncRead + AsyncWrite + Unpin + ?Sized,
    {
        let mut r = Pin::new(r);
        let mut w = Pin::new(w);
        loop {
            match state {
                TransferState::Running(buf) => {
                    let count = std::task::ready!(buf.poll_copy(cx, r.as_mut(), w.as_mut()))?;
                    *state = TransferState::ShuttingDown(count);
                }
                TransferState::ShuttingDown(count) => {
                    std::task::ready!(w.as_mut().poll_shutdown(cx))?;
                    *state = TransferState::Done(*count);
                }
                TransferState::Done(count) => return Poll::Ready(Ok(*count)),
            }
        }
    }

    /// This is a proper re-implementation of Tokio's
    /// [`copy_bidirectional_with_sizes`](tokio::io::copy_bidirectional_with_sizes), which does not leave the stream
    /// in half-open-state when one side closes read or write side. Instead, if either side encounters and empty
    /// read (EOF indication), the write-side is closed as well and vice versa.
    pub async fn copy_duplex<A, B>(
        a: &mut A,
        b: &mut B,
        a_to_b_buffer_size: usize,
        b_to_a_buffer_size: usize,
    ) -> std::io::Result<(u64, u64)>
    where
        A: AsyncRead + AsyncWrite + Unpin + ?Sized,
        B: AsyncRead + AsyncWrite + Unpin + ?Sized,
    {
        let mut a_to_b = TransferState::Running(CopyBuffer::new(a_to_b_buffer_size));
        let mut b_to_a = TransferState::Running(CopyBuffer::new(b_to_a_buffer_size));

        std::future::poll_fn(|cx| {
            let mut a_to_b_result = transfer_one_direction(cx, &mut a_to_b, a, b)?;
            let mut b_to_a_result = transfer_one_direction(cx, &mut b_to_a, b, a)?;

            if let TransferState::Done(_) = b_to_a {
                if let TransferState::Running(buf) = &a_to_b {
                    tracing::trace!("B-side has completed, terminating A-side.");
                    a_to_b = TransferState::ShuttingDown(buf.amt);
                    a_to_b_result = transfer_one_direction(cx, &mut a_to_b, a, b)?;
                }
            }

            if let TransferState::Done(_) = a_to_b {
                if let TransferState::Running(buf) = &b_to_a {
                    tracing::trace!("A-side has completed, terminate B-side.");
                    b_to_a = TransferState::ShuttingDown(buf.amt);
                    b_to_a_result = transfer_one_direction(cx, &mut b_to_a, b, a)?;
                }
            }

            // Not a problem if ready! returns early
            let a_to_b_bytes_transferred = std::task::ready!(a_to_b_result);
            let b_to_a_bytes_transferred = std::task::ready!(b_to_a_result);

            Poll::Ready(Ok((a_to_b_bytes_transferred, b_to_a_bytes_transferred)))
        })
        .await
    }

    #[derive(Debug)]
    struct CopyBuffer {
        read_done: bool,
        need_flush: bool,
        pos: usize,
        cap: usize,
        amt: u64,
        buf: Box<[u8]>,
    }

    impl CopyBuffer {
        fn new(buf_size: usize) -> Self {
            Self {
                read_done: false,
                need_flush: false,
                pos: 0,
                cap: 0,
                amt: 0,
                buf: vec![0; buf_size].into_boxed_slice(),
            }
        }

        fn poll_fill_buf<R>(&mut self, cx: &mut Context<'_>, reader: Pin<&mut R>) -> Poll<std::io::Result<()>>
        where
            R: AsyncRead + ?Sized,
        {
            let me = &mut *self;
            let mut buf = ReadBuf::new(&mut me.buf);
            buf.set_filled(me.cap);

            let res = reader.poll_read(cx, &mut buf);
            if let Poll::Ready(Ok(())) = res {
                let filled_len = buf.filled().len();
                me.read_done = me.cap == filled_len;
                me.cap = filled_len;
            }
            res
        }

        fn poll_write_buf<R, W>(
            &mut self,
            cx: &mut Context<'_>,
            mut reader: Pin<&mut R>,
            mut writer: Pin<&mut W>,
        ) -> Poll<std::io::Result<usize>>
        where
            R: AsyncRead + ?Sized,
            W: AsyncWrite + ?Sized,
        {
            let this = &mut *self;
            match writer.as_mut().poll_write(cx, &this.buf[this.pos..this.cap]) {
                Poll::Pending => {
                    // Top up the buffer towards full if we can read a bit more
                    // data - this should improve the chances of a large write
                    if !this.read_done && this.cap < this.buf.len() {
                        std::task::ready!(this.poll_fill_buf(cx, reader.as_mut()))?;
                    }
                    Poll::Pending
                }
                res @ Poll::Ready(_) => res,
            }
        }

        pub(super) fn poll_copy<R, W>(
            &mut self,
            cx: &mut Context<'_>,
            mut reader: Pin<&mut R>,
            mut writer: Pin<&mut W>,
        ) -> Poll<std::io::Result<u64>>
        where
            R: AsyncRead + ?Sized,
            W: AsyncWrite + ?Sized,
        {
            loop {
                // If our buffer is empty, then we need to read some data to
                // continue.
                if self.pos == self.cap && !self.read_done {
                    self.pos = 0;
                    self.cap = 0;

                    match self.poll_fill_buf(cx, reader.as_mut()) {
                        Poll::Ready(Ok(())) => (),
                        Poll::Ready(Err(err)) => {
                            return Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, err)));
                        }
                        Poll::Pending => {
                            // Try flushing when the reader has no progress to avoid deadlock
                            // when the reader depends on a buffered writer.
                            if self.need_flush {
                                std::task::ready!(writer.as_mut().poll_flush(cx))?;
                                self.need_flush = false;
                            }

                            return Poll::Pending;
                        }
                    }
                }

                // If our buffer has some data, let's write it out
                while self.pos < self.cap {
                    let i = std::task::ready!(self.poll_write_buf(cx, reader.as_mut(), writer.as_mut()))?;
                    if i == 0 {
                        return Poll::Ready(Err(std::io::Error::new(
                            std::io::ErrorKind::WriteZero,
                            "write zero byte",
                        )));
                    }
                    self.pos += i;
                    self.amt += i as u64;
                    self.need_flush = true;
                }

                // If pos larger than cap, this loop will never stop.
                // In particular, a user's wrong poll_write implementation returning
                // incorrect written length may lead to thread blocking.
                debug_assert!(self.pos <= self.cap, "writer returned length larger than input slice");

                // If we've written all the data, and we've seen EOF, flush out the
                // data and finish the transfer.
                if self.pos == self.cap && self.read_done {
                    std::task::ready!(writer.as_mut().poll_flush(cx))?;
                    return Poll::Ready(Ok(self.amt));
                }
            }
        }
    }
}

/// Converts a [`AsyncRead`] into [`Stream`] by reading at most `S` bytes
/// in each call to [`Stream::poll_next`].
pub struct AsyncReadStreamer<const S: usize, R>(pub R);

impl<const S: usize, R: AsyncRead + Unpin> futures::Stream for AsyncReadStreamer<S, R> {
    type Item = std::io::Result<Box<[u8]>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buffer = vec![0u8; S];

        match futures::ready!(Pin::new(&mut self.0).poll_read(cx, &mut buffer)) {
            Ok(0) => Poll::Ready(None),
            Ok(size) => {
                buffer.truncate(size);
                Poll::Ready(Some(Ok(buffer.into_boxed_slice())))
            }
            Err(err) => Poll::Ready(Some(Err(err))),
        }
    }
}

#[cfg(all(feature = "runtime-tokio", test))]
mod tests {
    use futures::TryStreamExt;
    use tokio::io::AsyncWriteExt;

    use super::*;
    use crate::utils::DuplexIO;

    #[tokio::test]
    async fn test_copy_duplex() -> anyhow::Result<()> {
        const DATA_LEN: usize = 2000;

        let alice_tx = hopr_crypto_random::random_bytes::<DATA_LEN>();
        let mut alice_rx = [0u8; DATA_LEN];

        let bob_tx = hopr_crypto_random::random_bytes::<DATA_LEN>();
        let mut bob_rx = [0u8; DATA_LEN];

        let alice = DuplexIO(alice_tx.as_ref(), futures::io::Cursor::new(alice_rx.as_mut()));
        let bob = DuplexIO(bob_tx.as_ref(), futures::io::Cursor::new(bob_rx.as_mut()));

        let (a_to_b, b_to_a) = copy_duplex(
            &mut tokio_util::compat::FuturesAsyncReadCompatExt::compat(alice),
            &mut tokio_util::compat::FuturesAsyncReadCompatExt::compat(bob),
            128,
            128,
        )
        .await?;

        assert_eq!(DATA_LEN, a_to_b as usize);
        assert_eq!(DATA_LEN, b_to_a as usize);

        assert_eq!(alice_tx, bob_rx);
        assert_eq!(bob_tx, alice_rx);

        Ok(())
    }

    #[tokio::test]
    async fn test_copy_duplex_small() -> anyhow::Result<()> {
        const DATA_LEN: usize = 100;

        let alice_tx = hopr_crypto_random::random_bytes::<DATA_LEN>();
        let mut alice_rx = [0u8; DATA_LEN];

        let bob_tx = hopr_crypto_random::random_bytes::<DATA_LEN>();
        let mut bob_rx = [0u8; DATA_LEN];

        let alice = DuplexIO(alice_tx.as_ref(), futures::io::Cursor::new(alice_rx.as_mut()));
        let bob = DuplexIO(bob_tx.as_ref(), futures::io::Cursor::new(bob_rx.as_mut()));

        let (a_to_b, b_to_a) = copy_duplex(
            &mut tokio_util::compat::FuturesAsyncReadCompatExt::compat(alice),
            &mut tokio_util::compat::FuturesAsyncReadCompatExt::compat(bob),
            128,
            128,
        )
        .await?;

        assert_eq!(DATA_LEN, a_to_b as usize);
        assert_eq!(DATA_LEN, b_to_a as usize);

        assert_eq!(alice_tx, bob_rx);
        assert_eq!(bob_tx, alice_rx);

        Ok(())
    }

    #[tokio::test]
    async fn test_client_to_server() -> anyhow::Result<()> {
        let (mut client_tx, mut client_rx) = tokio::io::duplex(8); // Create a mock duplex stream
        let (mut server_rx, mut server_tx) = tokio::io::duplex(32); // Create a mock duplex stream

        // Simulate 'a' finishing while there's still data for 'b'
        client_tx.write_all(b"hello").await?;
        client_tx.shutdown().await?;

        server_tx.write_all(b"data").await?;
        server_tx.shutdown().await?;

        let result = crate::utils::copy_duplex(&mut client_rx, &mut server_rx, 2, 2).await?;

        let (client_to_server_count, server_to_client_count) = result;
        assert_eq!(client_to_server_count, 5); // 'hello' was transferred
        assert_eq!(server_to_client_count, 4); // response only partially transferred or not at all

        Ok(())
    }

    #[tokio::test]
    async fn test_server_to_client() -> anyhow::Result<()> {
        let (mut client_tx, mut client_rx) = tokio::io::duplex(32); // Create a mock duplex stream
        let (mut server_rx, mut server_tx) = tokio::io::duplex(8); // Create a mock duplex stream

        // Simulate 'a' finishing while there's still data for 'b'
        server_tx.write_all(b"hello").await?;
        server_tx.shutdown().await?;

        client_tx.write_all(b"some longer data to transfer").await?;

        let result = crate::utils::copy_duplex(&mut client_rx, &mut server_rx, 2, 2).await?;

        let (client_to_server_count, server_to_client_count) = result;
        assert_eq!(server_to_client_count, 5); // 'hello' was transferred
        assert!(client_to_server_count <= 8); // response only partially transferred or not at all

        Ok(())
    }

    #[tokio::test]
    async fn test_async_read_streamer_complete_chunk() {
        let data = b"Hello, World!!";
        let mut streamer = AsyncReadStreamer::<14, _>(&data[..]);
        let mut results = Vec::new();

        while let Some(res) = streamer.try_next().await.unwrap() {
            results.push(res);
        }

        assert_eq!(results, vec![Box::from(*data)]);
    }

    #[tokio::test]
    async fn test_async_read_streamer_complete_more_chunks() {
        let data = b"Hello, World and do it twice";
        let mut streamer = AsyncReadStreamer::<14, _>(&data[..]);
        let mut results = Vec::new();

        while let Some(res) = streamer.try_next().await.unwrap() {
            results.push(res);
        }

        let (data1, data2) = data.split_at(14);
        assert_eq!(results, vec![Box::from(data1), Box::from(data2)]);
    }

    #[tokio::test]
    async fn test_async_read_streamer_complete_more_chunks_with_incomplete() -> anyhow::Result<()> {
        let data = b"Hello, World and do it twice, ...";
        let streamer = AsyncReadStreamer::<14, _>(&data[..]);

        let results = streamer.try_collect::<Vec<_>>().await?;

        let (data1, rest) = data.split_at(14);
        let (data2, data3) = rest.split_at(14);
        assert_eq!(results, vec![Box::from(data1), Box::from(data2), Box::from(data3)]);

        Ok(())
    }

    #[tokio::test]
    async fn test_async_read_streamer_incomplete_chunk() -> anyhow::Result<()> {
        let data = b"Hello, World!!";
        let reader = &data[0..8]; // An incomplete chunk
        let mut streamer = AsyncReadStreamer::<14, _>(reader);

        assert_eq!(Some(Box::from(reader)), streamer.try_next().await?);

        Ok(())
    }
}
