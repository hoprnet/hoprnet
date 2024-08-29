#[cfg(feature = "runtime-tokio")]
pub use tokio_utils::copy_bidirectional_client_server;

#[cfg(feature = "runtime-tokio")]
mod tokio_utils {
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

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

    pub async fn copy_bidirectional_client_server<T, U>(
        client: &mut T,
        server: &mut U,
        client_to_server_buf: usize,
        server_to_client_buf: usize,
    ) -> std::io::Result<(u64, u64)>
    where
        T: AsyncRead + AsyncWrite + Unpin + ?Sized,
        U: AsyncRead + AsyncWrite + Unpin + ?Sized,
    {
        let mut client_to_server = TransferState::Running(CopyBuffer::new(client_to_server_buf));
        let mut server_to_client = TransferState::Running(CopyBuffer::new(server_to_client_buf));

        std::future::poll_fn(|cx| {
            let mut client_to_server_result = transfer_one_direction(cx, &mut client_to_server, client, server)?;
            let mut server_to_client_result = transfer_one_direction(cx, &mut server_to_client, server, client)?;

            if let TransferState::Done(_) = server_to_client {
                if let TransferState::Running(buf) = &client_to_server {
                    tracing::trace!("server has completed, terminating client");
                    client_to_server = TransferState::ShuttingDown(buf.amt);
                    client_to_server_result = transfer_one_direction(cx, &mut client_to_server, client, server)?;
                }
            }

            if let TransferState::Done(_) = client_to_server {
                if let TransferState::Running(buf) = &server_to_client {
                    tracing::trace!("server has completed, terminate client");
                    server_to_client = TransferState::ShuttingDown(buf.amt);
                    server_to_client_result = transfer_one_direction(cx, &mut server_to_client, server, client)?;
                }
            }

            // Not a problem if ready! returns early
            let client_to_server = std::task::ready!(client_to_server_result);
            let server_to_client = std::task::ready!(server_to_client_result);

            Poll::Ready(Ok((client_to_server, server_to_client)))
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
                            return Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, err)))
                        }
                        Poll::Pending => {
                            // Try flushing when the reader has no progress to avoid deadlock
                            // when the reader depends on buffered writer.
                            if self.need_flush {
                                std::task::ready!(writer.as_mut().poll_flush(cx))?;
                                self.need_flush = false;
                            }

                            return Poll::Pending;
                        }
                    }
                }

                // If our buffer has some data, let's write it out!
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
                // In particular, user's wrong poll_write implementation returning
                // incorrect written length may lead to thread blocking.
                debug_assert!(self.pos <= self.cap, "writer returned length larger than input slice");

                // If we've written all the data and we've seen EOF, flush out the
                // data and finish the transfer.
                if self.pos == self.cap && self.read_done {
                    std::task::ready!(writer.as_mut().poll_flush(cx))?;
                    return Poll::Ready(Ok(self.amt));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[cfg(feature = "runtime-tokio")]
    mod tokio {
        use tokio::io::AsyncWriteExt;

        #[tokio::test]
        async fn test_client_to_server() -> anyhow::Result<()> {
            let (mut client_tx, mut client_rx) = tokio::io::duplex(8); // Create a mock duplex stream
            let (mut server_rx, mut server_tx) = tokio::io::duplex(32); // Create a mock duplex stream

            // Simulate 'a' finishing while there's still data for 'b'
            client_tx.write_all(b"hello").await?;
            client_tx.shutdown().await?;

            server_tx.write_all(b"data").await?;
            server_tx.shutdown().await?;

            let result = crate::utils::copy_bidirectional_client_server(&mut client_rx, &mut server_rx, 2, 2).await?;

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

            let result = crate::utils::copy_bidirectional_client_server(&mut client_rx, &mut server_rx, 2, 2).await?;

            let (client_to_server_count, server_to_client_count) = result;
            assert_eq!(server_to_client_count, 5); // 'hello' was transferred
            assert!(client_to_server_count <= 8); // response only partially transferred or not at all

            Ok(())
        }
    }
}
