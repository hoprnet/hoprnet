use futures::future::BoxFuture;
use futures::{pin_mut, FutureExt, Sink, SinkExt};
use std::io::{Error, ErrorKind};
use std::pin::Pin;
use std::sync::{Arc, OnceLock};
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, ReadBuf};
use tokio::net::UdpSocket;
use tokio_util::io::StreamReader;
use tracing::{error, trace, warn};

/// Mimics TCP-like stream functionality on a UDP socket by restricting it to a single
/// counterparty and implements [`tokio::io::AsyncRead`] and [`tokio::io::AsyncWrite`].
///
/// To set a counterparty, one of the following must happen:
/// 1) setting it during binding via [`bind`](ConnectedUdpStream::bind)
/// 2) receiving some data from the other side.
///
/// Whatever of the above happens first, sets the counterparty.
/// Once the counterparty is set, all data sent and received will be sent or filtered by this
/// counterparty address.
///
/// If data from another party is received, an error is raised, unless the object has been constructed
/// with [`ForeignDataMode::Discard`] or [`ForeignDataMode::Accept`] setting.
//#[derive(Debug)]
pub struct ConnectedUdpStream {
    socket_handles: Vec<(tokio::task::JoinHandle<()>, tokio::task::JoinHandle<()>)>,
    ingress_rx: Box<dyn AsyncRead + Send + Unpin>,
    egress_tx: Option<Box<dyn Sink<Box<[u8]>, Error = flume::SendError<Box<[u8]>>> + Send + Unpin>>,
    counterparty: Arc<OnceLock<std::net::SocketAddr>>,
    bound_to: std::net::SocketAddr,
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
    fn bind_socket_rx(
        sock_rx: Arc<UdpSocket>,
        ingress_tx: flume::Sender<std::io::Result<tokio_util::bytes::Bytes>>,
        counterparty: Arc<OnceLock<std::net::SocketAddr>>,
        foreign_data_mode: ForeignDataMode,
        buf_size: usize,
    ) -> std::io::Result<BoxFuture<'static, ()>> {
        // Create an RX task for the socket
        let counterparty_rx = counterparty.clone();
        Ok(async move {
            let mut buffer = vec![0u8; buf_size];
            let mut done = false;
            loop {
                // Read data from the socket
                let out_res = match sock_rx.recv_from(&mut buffer).await {
                    Ok((read, read_addr)) if read > 0 => {
                        trace!("got {read} bytes of data data from {read_addr}");
                        let addr = counterparty_rx.get_or_init(|| read_addr);

                        // If the data is from a counterparty, or we accept anything, pass it
                        if read_addr.eq(addr) || foreign_data_mode == ForeignDataMode::Accept {
                            let out_buffer = tokio_util::bytes::Bytes::copy_from_slice(&buffer[..read]);
                            Some(Ok(out_buffer))
                        } else {
                            match foreign_data_mode {
                                ForeignDataMode::Discard => {
                                    // Don't even bother sending an error about discarded stuff
                                    warn!(
                                        udp_bound_addr = tracing::field::debug(sock_rx.local_addr()),
                                        "discarded data from {read_addr}, which didn't come from {addr}"
                                    );
                                    None
                                }
                                ForeignDataMode::Error => {
                                    // Terminate here, the ingress_tx gets dropped
                                    done = true;
                                    Some(Err(Error::new(
                                        ErrorKind::ConnectionRefused,
                                        "data from foreign client not allowed",
                                    )))
                                }
                                // ForeignDataMode::Accept has been handled above
                                _ => unreachable!(),
                            }
                        }
                    }
                    Ok(_) => {
                        // Read EOF, terminate here, the ingress_tx gets dropped
                        trace!(
                            udp_bound_addr = tracing::field::debug(sock_rx.local_addr()),
                            "read EOF on socket"
                        );
                        done = true;
                        None
                    }
                    Err(e) => {
                        // Forward the error
                        done = true;
                        Some(Err(e))
                    }
                };

                // Dispatch the received data to the queue
                if let Some(out_res) = out_res {
                    if let Err(err) = ingress_tx.send_async(out_res).await {
                        error!(
                            udp_bound_addr = tracing::field::debug(sock_rx.local_addr()),
                            "failed to dispatch received data: {err}"
                        );
                        done = true;
                    }
                }

                if done {
                    break;
                }
            }
        }
        .boxed())
    }

    fn bind_socket_tx(
        sock_tx: Arc<UdpSocket>,
        egress_rx: flume::Receiver<Box<[u8]>>,
        counterparty: Arc<OnceLock<std::net::SocketAddr>>,
    ) -> std::io::Result<BoxFuture<'static, ()>> {
        let counterparty_tx = counterparty.clone();
        Ok(async move {
            loop {
                match egress_rx.recv_async().await {
                    Ok(data) => {
                        if let Some(target) = counterparty_tx.get() {
                            if let Err(e) = sock_tx.send_to(&data, target).await {
                                error!(
                                    udp_bound_addr = tracing::field::debug(sock_tx.local_addr()),
                                    "failed to send data to {target}: {e}"
                                );
                            }
                        } else {
                            error!(
                                udp_bound_addr = tracing::field::debug(sock_tx.local_addr()),
                                "cannot send data, counterparty not set"
                            );
                            break;
                        }
                    }
                    Err(e) => {
                        error!(
                            udp_bound_addr = tracing::field::debug(sock_tx.local_addr()),
                            "cannot receive more data from egress channel: {e}"
                        );
                        break;
                    }
                }
            }
        }
        .boxed())
    }

    /// Binds the UDP socket to the given address.
    pub fn bind<A: std::net::ToSocketAddrs>(
        bind: A,
        buf_size: usize,
        counterparty: Option<std::net::SocketAddr>,
        foreign_data_mode: ForeignDataMode,
        parallelism: Option<usize>,
    ) -> tokio::io::Result<Self> {
        let num_socks = parallelism
            .and_then(|i| {
                (i > 0)
                    .then_some(i)
                    .or(std::thread::available_parallelism().ok().map(usize::from))
            })
            .unwrap_or(1);

        let counterparty = Arc::new(counterparty.map(OnceLock::from).unwrap_or_default());
        let (ingress_tx, ingress_rx) = flume::unbounded();
        let (egress_tx, egress_rx) = flume::unbounded();

        let first_bind_addr = bind.to_socket_addrs()?.next().ok_or(ErrorKind::AddrNotAvailable)?;
        let mut bound_addr: Option<std::net::SocketAddr> = None;

        let socket_handles = (0..num_socks)
            .map(|_| {
                // Bind a new non-blocking UDP socket with SO_REUSEADDR
                let sock = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None)?;
                sock.set_reuse_port(true)?;
                sock.set_nonblocking(true)?;
                sock.bind(&first_bind_addr.into())?;

                let socket_bound_addr = sock
                    .local_addr()?
                    .as_socket()
                    .ok_or(Error::other("invalid socket type"))?;
                match bound_addr {
                    None => bound_addr = Some(socket_bound_addr),
                    Some(addr) if addr != socket_bound_addr => {
                        return Err(Error::other("inconsistent binding address"))
                    }
                    _ => {}
                }

                Ok(Arc::new(UdpSocket::from_std(sock.into())?))
            })
            .map(|sock| {
                sock.and_then(|sock| {
                    Ok((
                        tokio::task::spawn(Self::bind_socket_tx(
                            sock.clone(),
                            egress_rx.clone(),
                            counterparty.clone(),
                        )?),
                        tokio::task::spawn(Self::bind_socket_rx(
                            sock.clone(),
                            ingress_tx.clone(),
                            counterparty.clone(),
                            foreign_data_mode,
                            buf_size,
                        )?),
                    ))
                })
            })
            .collect::<std::io::Result<Vec<_>>>()?;

        Ok(Self {
            ingress_rx: Box::new(StreamReader::new(ingress_rx.into_stream())),
            egress_tx: Some(Box::new(egress_tx.into_sink())),
            socket_handles,
            counterparty,
            bound_to: bound_addr.ok_or(ErrorKind::AddrNotAvailable)?,
        })
    }

    pub fn bound_address(&self) -> &std::net::SocketAddr {
        &self.bound_to
    }
}

impl Drop for ConnectedUdpStream {
    fn drop(&mut self) {
        self.socket_handles.iter().for_each(|(tx, rx)| {
            tx.abort();
            rx.abort();
        })
    }
}

impl tokio::io::AsyncRead for ConnectedUdpStream {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        trace!("polling read from udp stream with {:?}", self.counterparty.get());
        match Pin::new(&mut self.ingress_rx).poll_read(cx, buf) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl tokio::io::AsyncWrite for ConnectedUdpStream {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Error>> {
        trace!("polling write to udp stream with {:?}", self.counterparty.get());
        if let Some(sender) = &mut self.egress_tx {
            match sender.poll_ready_unpin(cx) {
                Poll::Ready(Ok(_)) => {
                    pin_mut!(sender);
                    let len = buf.len(); // We always send the entire buffer at once
                    Poll::Ready(
                        sender
                            .start_send(Box::from(buf))
                            .map(|_| len)
                            .map_err(|e| Error::other(e)),
                    )
                    // TODO: should we also flush here with every write?
                }
                Poll::Ready(Err(e)) => Poll::Ready(Err(Error::other(e.to_string()))),
                Poll::Pending => Poll::Pending,
            }
        } else {
            Poll::Ready(Err(Error::new(ErrorKind::NotConnected, "udp stream is closed")))
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        trace!("polling flush to udp stream {:?}", self.counterparty.get());
        if let Some(sender) = &mut self.egress_tx {
            pin_mut!(sender);
            sender.poll_flush(cx).map_err(|err| Error::other(err.to_string()))
        } else {
            Poll::Ready(Err(Error::new(ErrorKind::NotConnected, "udp stream is closed")))
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        trace!("polling close on udp stream with {:?}", self.counterparty.get());
        // Take the sender to make sure it gets dropped
        let mut taken_sender = self.egress_tx.take();
        if let Some(sender) = &mut taken_sender {
            pin_mut!(sender);
            sender.poll_close(cx).map_err(|err| Error::other(err.to_string()))
        } else {
            Poll::Ready(Err(Error::new(ErrorKind::NotConnected, "udp stream is closed")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UdpSocket;

    #[test_log::test(tokio::test)]
    //#[tokio::test]
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

        let mut stream =
            ConnectedUdpStream::bind(("127.0.0.1", 0), 128, Some(listen_addr), ForeignDataMode::Error, None)
                .context("connection")?;

        for _ in 1..1000 {
            let mut w_buf = [0u8; DATA_SIZE];
            hopr_crypto_random::random_fill(&mut w_buf);
            let written = stream.write(&w_buf).await?;
            stream.flush().await?;
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
