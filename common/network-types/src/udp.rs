use std::io::Error;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::ReadBuf;
use tokio::net::{ToSocketAddrs, UdpSocket};

pub struct ConnectedUdpStream {
    sock: UdpSocket,
    closed: bool,
}

impl ConnectedUdpStream {
    pub async fn bind_and_connect<A: ToSocketAddrs>(bind: A, connect: A) -> tokio::io::Result<Self> {
        let sock = UdpSocket::bind(bind).await?;
        sock.connect(connect).await?;
        Ok(Self { sock, closed: false })
    }
}

impl From<UdpSocket> for ConnectedUdpStream {
    fn from(value: UdpSocket) -> Self {
        Self {
            sock: value,
            closed: false,
        }
    }
}

impl tokio::io::AsyncRead for ConnectedUdpStream {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        self.sock.poll_recv(cx, buf)
    }
}

impl tokio::io::AsyncWrite for ConnectedUdpStream {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Error>> {
        if !self.closed {
            self.sock.poll_send(cx, buf)
        } else {
            Poll::Ready(Ok(0))
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Error>> {
        self.closed = true;
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UdpSocket;

    #[tokio::test]
    async fn basic_udp_stream_tests() -> anyhow::Result<()> {
        const DATA_SIZE: usize = 128;

        let listener = UdpSocket::bind("127.0.0.1:0").await.context("bind listener")?;
        let listen_addr = listener.local_addr()?;

        // Simple echo UDP server
        tokio::task::spawn(async move {
            loop {
                let mut buf = [0u8; DATA_SIZE];
                let (read, addr) = listener.recv_from(&mut buf).await.expect("recv must not fail");
                if read > 0 {
                    assert_eq!(DATA_SIZE, read, "read size must be exactly {DATA_SIZE}");
                    listener.send_to(&buf, addr).await.expect("send must not fail");
                }
            }
        });

        let mut stream = ConnectedUdpStream::bind_and_connect("127.0.0.1:0".parse()?, listen_addr)
            .await
            .context("connection")?;

        for _ in 1..10 {
            let mut w_buf = [0u8; DATA_SIZE];
            hopr_crypto_random::random_fill(&mut w_buf);
            let written = stream.write(&w_buf).await?;
            assert_eq!(written, DATA_SIZE);

            let mut r_buf = [0u8; DATA_SIZE];
            let read = stream.read_exact(&mut r_buf).await?;
            assert_eq!(read, DATA_SIZE);

            assert_eq!(w_buf, r_buf);
        }

        stream.shutdown().await?;

        Ok(())
    }
}
