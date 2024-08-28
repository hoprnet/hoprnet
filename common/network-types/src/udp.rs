use std::io::Error;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::ReadBuf;
use tokio::net::UdpSocket;
use tracing::debug;

/// Mimics TCP-like stream functionality on a UDP socket by restricting it to a single
/// counterparty and implements [`tokio::io::AsyncRead`] and [`tokio::io::AsyncWrite`].
///
/// To set a counterparty, one of the following must happen:
/// 1) setting it via [`with_counterparty`](ConnectedUdpStream::with_counterparty) after binding
/// 2) receiving some data from the other side.
///
/// Whatever of the above happens first, sets the counterparty.
/// Once the counterparty is set, all data sent and received will be sent or filtered by this
/// counterparty address.
///
/// If data from another party is received, an error is raised, unless the object has been constructed
/// with [`with_foreign_data_discarding`](ConnectedUdpStream::with_foreign_data_discarding) setting.
#[derive(Debug)]
pub struct ConnectedUdpStream {
    sock: UdpSocket,
    counterparty: Option<std::net::SocketAddr>,
    closed: bool,
    foreign_data_mode: ForeignDataMode,
}

/// Defines what happens when data from another [`SocketAddr`](std::net::SocketAddr) arrives
/// into the [`ConnectedUdpStream`] (other than the one that is considered a counterparty for that
/// instance).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForeignDataMode {
    /// Foreign data are simply discarded.
    Discard,
    /// Foreign data are accepted as if they arrived from the set counterparty.
    Accept,
    /// Error is raised on the `poll_read` attempt.
    Error,
}

impl ConnectedUdpStream {
    /// Binds the UDP socket to the given address, without an assigned counterparty.
    pub async fn bind<A: tokio::net::ToSocketAddrs>(bind: A) -> tokio::io::Result<Self> {
        Ok(Self {
            sock: UdpSocket::bind(bind).await?,
            counterparty: None,
            closed: false,
            foreign_data_mode: ForeignDataMode::Error,
        })
    }

    /// Counterparty this instance is connected to, if yet any.
    pub fn counterparty(&self) -> &Option<std::net::SocketAddr> {
        &self.counterparty
    }

    /// Sets the counterparty and returns a new instance.
    pub fn with_counterparty(mut self, counterparty: std::net::SocketAddr) -> tokio::io::Result<Self> {
        if self.counterparty.is_none() {
            self.counterparty = Some(counterparty);
            Ok(self)
        } else {
            Err(Error::other("counterparty already set"))
        }
    }

    /// Changes the mode of behavior when foreign data (not from the counterparty) is received.
    /// See [ForeignDataMode] for details.
    pub fn with_foreign_data_mode(mut self, mode: ForeignDataMode) -> Self {
        self.foreign_data_mode = mode;
        self
    }

    /// Inner UDP socket.
    pub fn socket(&self) -> &UdpSocket {
        &self.sock
    }
}

impl tokio::io::AsyncRead for ConnectedUdpStream {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        match self.sock.poll_recv_from(cx, buf) {
            Poll::Ready(Ok(read_addr)) => match self.counterparty {
                Some(addr) if addr == read_addr => Poll::Ready(Ok(())),
                Some(addr) => match self.foreign_data_mode {
                    ForeignDataMode::Discard => {
                        buf.clear();
                        debug!("discarded data from foreign client {addr}");
                        Poll::Pending
                    }
                    ForeignDataMode::Accept => Poll::Ready(Ok(())),
                    ForeignDataMode::Error => {
                        buf.clear();
                        Poll::Ready(Err(Error::other(format!(
                            "expected data from {addr}, got from {read_addr}"
                        ))))
                    }
                },
                None => {
                    self.counterparty = Some(read_addr);
                    Poll::Ready(Ok(()))
                }
            },
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl tokio::io::AsyncWrite for ConnectedUdpStream {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Error>> {
        if !self.closed {
            if let Some(counterparty) = self.counterparty {
                self.sock.poll_send_to(cx, buf, counterparty)
            } else {
                Poll::Ready(Err(Error::other("cannot send, counterparty address not set")))
            }
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

        let mut stream = ConnectedUdpStream::bind(("127.0.0.1", 0))
            .await
            .context("connection")?
            .with_counterparty(listen_addr)?;

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
