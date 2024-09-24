use futures::{pin_mut, ready, FutureExt, Sink, SinkExt};
use std::io::ErrorKind;
use std::pin::Pin;
use std::sync::{Arc, OnceLock};
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, ReadBuf};
use tokio::net::UdpSocket;
use tracing::{debug, error, trace, warn};

type BoxIoSink<T> = Box<dyn Sink<T, Error = std::io::Error> + Send + Unpin>;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_UDP_INGRESS_LEN: hopr_metrics::MultiHistogram =
        hopr_metrics::MultiHistogram::new(
            "hopr_udp_ingress_packet_len",
            "UDP packet lengths on ingress per counterparty",
            vec![20.0, 40.0, 80.0, 160.0, 320.0, 640.0, 1280.0, 2560.0, 5120.0],
            &["counterparty"]
    ).unwrap();
    static ref METRIC_UDP_EGRESS_LEN: hopr_metrics::MultiHistogram =
        hopr_metrics::MultiHistogram::new(
            "hopr_udp_egress_packet_len",
            "UDP packet lengths on egress per counterparty",
            vec![20.0, 40.0, 80.0, 160.0, 320.0, 640.0, 1280.0, 2560.0, 5120.0],
            &["counterparty"]
    ).unwrap();
}

/// Mimics TCP-like stream functionality on a UDP socket by restricting it to a single
/// counterparty and implements [`tokio::io::AsyncRead`] and [`tokio::io::AsyncWrite`].
/// The instance is always constructed using a [`UdpStreamBuilder`].
///
/// To set a counterparty, one of the following must happen:
/// 1) setting it during build via [`UdpStreamBuilder::with_counterparty`]
/// 2) receiving some data from the other side.
///
/// Whatever of the above happens first, sets the counterparty.
/// Once the counterparty is set, all data sent and received will be sent or filtered by this
/// counterparty address.
///
/// If data from another party is received, an error is raised, unless the object has been constructed
/// with [`ForeignDataMode::Discard`] or [`ForeignDataMode::Accept`] setting.
///
/// This object is also capable of parallel processing on a UDP socket.
/// If [parallelism](UdpStreamBuilder::with_parallelism) is set, the instance will create
/// multiple sockets with `SO_REUSEADDR` and spin parallel tasks that will coordinate data and
/// transmission retrieval using these sockets. This is driven by RX/TX MPMC queues, which are
/// per-default unbounded (see [queue size](UdpStreamBuilder::with_queue_size) for details).
pub struct ConnectedUdpStream {
    socket_handles: Vec<(tokio::task::JoinHandle<()>, tokio::task::JoinHandle<()>)>,
    ingress_rx: Box<dyn AsyncRead + Send + Unpin>,
    egress_tx: Option<BoxIoSink<Box<[u8]>>>,
    counterparty: Arc<OnceLock<std::net::SocketAddr>>,
    bound_to: std::net::SocketAddr,
}

/// Defines what happens when data from another [`SocketAddr`](std::net::SocketAddr) arrives
/// into the [`ConnectedUdpStream`] (other than the one that is considered a counterparty for that
/// instance).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ForeignDataMode {
    /// Foreign data are simply discarded.
    Discard,
    /// Foreign data are accepted as if they arrived from the set counterparty.
    Accept,
    /// Error is raised on the `poll_read` attempt.
    #[default]
    Error,
}

/// Builder object for the [`ConnectedUdpStream`].
///
/// If you wish to use defaults, do `UdpStreamBuilder::default().build(addr)`.
#[derive(Debug)]
pub struct UdpStreamBuilder {
    foreign_data_mode: ForeignDataMode,
    buffer_size: usize,
    queue_size: Option<usize>,
    parallelism: Option<usize>,
    counterparty: Option<std::net::SocketAddr>,
}

impl Default for UdpStreamBuilder {
    fn default() -> Self {
        Self {
            buffer_size: 2048,
            foreign_data_mode: Default::default(),
            queue_size: None,
            parallelism: None,
            counterparty: None,
        }
    }
}

impl UdpStreamBuilder {
    /// Defines the behavior when data from an unexpected source arrive into the socket.
    /// See [`ForeignDataMode`] for details.
    ///
    /// Default is [`ForeignDataMode::Error`].
    pub fn with_foreign_data_mode(mut self, mode: ForeignDataMode) -> Self {
        self.foreign_data_mode = mode;
        self
    }

    /// The size of the UDP receive buffer.
    ///
    /// This size must be at least the size of the MTU, otherwise the unread UDP data that
    /// does not fit this buffer will be discarded.
    ///
    /// Default is 2048.
    pub fn with_buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;
        self
    }

    /// Size of the TX/RX queue that dispatches data of reads from/writings to
    /// the sockets.
    ///
    /// This an important back-pressure mechanism when dispatching received data from
    /// fast senders.
    /// Reduces the maximum memory consumed by the object, which is given by:
    /// [`buffer_size`](UdpStreamBuilder::with_buffer_size) *
    /// [`queue_size`](UdpStreamBuilder::with_queue_size)
    ///
    /// Default is unbounded.
    pub fn with_queue_size(mut self, queue_size: usize) -> Self {
        self.queue_size = Some(queue_size);
        self
    }

    /// Sets how many parallel readers/writer sockets should be bound.
    ///
    /// Each UDP socket is bound with `SO_REUSEADDR` to facilitate parallel processing
    /// of read and write operations.
    ///
    /// - If some value `n` > 0 is given, the stream will bind `n` sockets.
    /// - If 0 is given, the number of sockets is determined by [`std::thread::available_parallelism`].
    /// - If none is given, only a single socket will be created (no parallelism).
    ///
    /// Default is none.
    pub fn with_parallelism(mut self, parallelism: usize) -> Self {
        self.parallelism = Some(parallelism);
        self
    }

    /// Sets the expected counterparty for data sent/received by the UDP sockets.
    ///
    /// If not specified, the counterparty is determined from the first packet received.
    /// However, no data can be sent up until this point.
    /// Therefore, the value must be set if data are sent first rather than received.
    /// If data is expected to be received first, the value does not need to be set.
    ///
    /// See [`ConnectedUdpStream`] and [`ForeignDataMode`] for details.
    ///
    /// Default is none.
    pub fn with_counterparty(mut self, counterparty: std::net::SocketAddr) -> Self {
        self.counterparty = Some(counterparty);
        self
    }

    /// Builds the [`ConnectedUdpStream`] with UDP socket(s) bound to `bind_addr`.
    ///
    /// The number of sockets bound is determined by [parallelism](UdpStreamBuilder::with_parallelism).
    /// The returned instance is always ready to receive data.
    /// It is also ready to send data
    /// if the [counterparty](UdpStreamBuilder::with_counterparty) has been set.
    pub fn build<A: std::net::ToSocketAddrs>(self, bind_addr: A) -> std::io::Result<ConnectedUdpStream> {
        let avail_parallelism = std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or_else(|e| {
                warn!("failed to determine available parallelism, defaulting to 1: {e}");
                1
            })
            / 2; // Each socket gets one RX and one TX task
        let num_socks = self
            .parallelism
            .map(|n| {
                if n == 0 {
                    avail_parallelism
                } else {
                    n.max(avail_parallelism)
                }
            })
            .unwrap_or(1);

        let counterparty = Arc::new(self.counterparty.map(OnceLock::from).unwrap_or_default());
        let ((ingress_tx, ingress_rx), (egress_tx, egress_rx)) = if let Some(q) = self.queue_size {
            (flume::bounded(q), flume::bounded(q))
        } else {
            (flume::unbounded(), flume::unbounded())
        };

        let first_bind_addr = bind_addr.to_socket_addrs()?.next().ok_or(ErrorKind::AddrNotAvailable)?;
        debug!("UDP stream is going to bind {num_socks} sockets to {first_bind_addr}");

        let mut bound_addr: Option<std::net::SocketAddr> = None;

        let socket_handles = (0..num_socks)
            .map(|id| {
                let domain = match &first_bind_addr {
                    std::net::SocketAddr::V4(_) => socket2::Domain::IPV4,
                    std::net::SocketAddr::V6(_) => socket2::Domain::IPV6,
                };

                // Bind a new non-blocking UDP socket with SO_REUSEADDR
                let sock = socket2::Socket::new(domain, socket2::Type::DGRAM, None)?;
                sock.set_reuse_address(true)?;
                sock.set_nonblocking(true)?;
                sock.bind(&bound_addr.unwrap_or(first_bind_addr).into())?;

                // Determine the address we bound this socket to, so we can also bind the others
                let socket_bound_addr = sock
                    .local_addr()?
                    .as_socket()
                    .ok_or(std::io::Error::other("invalid socket type"))?;

                match bound_addr {
                    None => bound_addr = Some(socket_bound_addr),
                    Some(addr) if addr != socket_bound_addr => {
                        return Err(std::io::Error::other(format!(
                            "inconsistent binding address {addr} != {socket_bound_addr} on socket id {id}"
                        )))
                    }
                    _ => {}
                }

                debug!(socket_id = id, "bound UDP socket to {socket_bound_addr}");
                Ok((id, Arc::new(UdpSocket::from_std(sock.into())?)))
            })
            .map(|sock| {
                sock.and_then(|(sock_id, sock)| {
                    Ok((
                        tokio::task::spawn(ConnectedUdpStream::setup_tx_queue(
                            sock_id,
                            sock.clone(),
                            egress_rx.clone(),
                            counterparty.clone(),
                        )?),
                        tokio::task::spawn(ConnectedUdpStream::setup_rx_queue(
                            sock_id,
                            sock.clone(),
                            ingress_tx.clone(),
                            counterparty.clone(),
                            self.foreign_data_mode,
                            self.buffer_size,
                        )?),
                    ))
                })
            })
            .collect::<std::io::Result<Vec<_>>>()?;

        Ok(ConnectedUdpStream {
            ingress_rx: Box::new(tokio_util::io::StreamReader::new(ingress_rx.into_stream())),
            egress_tx: Some(Box::new(
                egress_tx
                    .into_sink()
                    .sink_map_err(|e| std::io::Error::other(e.to_string())),
            )),
            socket_handles,
            counterparty,
            bound_to: bound_addr.ok_or(ErrorKind::AddrNotAvailable)?,
        })
    }
}

impl ConnectedUdpStream {
    /// Creates a receiver queue for the UDP stream.
    fn setup_rx_queue(
        socket_id: usize,
        sock_rx: Arc<UdpSocket>,
        ingress_tx: flume::Sender<std::io::Result<tokio_util::bytes::Bytes>>,
        counterparty: Arc<OnceLock<std::net::SocketAddr>>,
        foreign_data_mode: ForeignDataMode,
        buf_size: usize,
    ) -> std::io::Result<futures::future::BoxFuture<'static, ()>> {
        let counterparty_rx = counterparty.clone();
        Ok(async move {
            let mut buffer = vec![0u8; buf_size];
            let mut done = false;
            loop {
                // Read data from the socket
                let out_res = match sock_rx.recv_from(&mut buffer).await {
                    Ok((read, read_addr)) if read > 0 => {
                        trace!(
                            socket_id,
                            udp_bound_addr = tracing::field::debug(sock_rx.local_addr()),
                            "got {read} bytes of data from {read_addr}"
                        );
                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_UDP_INGRESS_LEN.observe(&[&read.to_string()], read as f64);

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
                                        socket_id,
                                        udp_bound_addr = tracing::field::debug(sock_rx.local_addr()),
                                        "discarded data from {read_addr}, which didn't come from {addr}"
                                    );
                                    None
                                }
                                ForeignDataMode::Error => {
                                    // Terminate here, the ingress_tx gets dropped
                                    done = true;
                                    Some(Err(std::io::Error::new(
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
                            socket_id,
                            udp_bound_addr = tracing::field::debug(sock_rx.local_addr()),
                            "read EOF on socket"
                        );
                        done = true;
                        None
                    }
                    Err(e) => {
                        // Forward the error
                        debug!(
                            socket_id,
                            udp_bound_addr = tracing::field::debug(sock_rx.local_addr()),
                            "forwarded error {e}"
                        );
                        done = true;
                        Some(Err(e))
                    }
                };

                // Dispatch the received data to the queue.
                // If the underlying queue is bounded, it will wait until there is space.
                if let Some(out_res) = out_res {
                    if let Err(err) = ingress_tx.send_async(out_res).await {
                        error!(
                            socket_id,
                            udp_bound_addr = tracing::field::debug(sock_rx.local_addr()),
                            "failed to dispatch received data: {err}"
                        );
                        done = true;
                    }
                }

                if done {
                    trace!(
                        socket_id,
                        udp_bound_addr = tracing::field::debug(sock_rx.local_addr()),
                        "rx queue done"
                    );
                    break;
                }
            }
        }
        .boxed())
    }

    /// Creates a transmission queue for the UDP stream.
    fn setup_tx_queue(
        socket_id: usize,
        sock_tx: Arc<UdpSocket>,
        egress_rx: flume::Receiver<Box<[u8]>>,
        counterparty: Arc<OnceLock<std::net::SocketAddr>>,
    ) -> std::io::Result<futures::future::BoxFuture<'static, ()>> {
        let counterparty_tx = counterparty.clone();
        Ok(async move {
            loop {
                match egress_rx.recv_async().await {
                    Ok(data) => {
                        if let Some(target) = counterparty_tx.get() {
                            if let Err(e) = sock_tx.send_to(&data, target).await {
                                error!(
                                    socket_id,
                                    udp_bound_addr = tracing::field::debug(sock_tx.local_addr()),
                                    "failed to send data to {target}: {e}"
                                );
                            }
                            trace!(socket_id, "sent {} bytes of data to {target}", data.len());

                            #[cfg(all(feature = "prometheus", not(test)))]
                            METRIC_UDP_EGRESS_LEN.observe(&[&target.to_string()], data.len() as f64);
                        } else {
                            error!(
                                socket_id,
                                udp_bound_addr = tracing::field::debug(sock_tx.local_addr()),
                                "cannot send data, counterparty not set"
                            );
                            break;
                        }
                    }
                    Err(e) => {
                        error!(
                            socket_id,
                            udp_bound_addr = tracing::field::debug(sock_tx.local_addr()),
                            "cannot receive more data from egress channel: {e}"
                        );
                        break;
                    }
                }
                trace!(
                    socket_id,
                    udp_bound_addr = tracing::field::debug(sock_tx.local_addr()),
                    "tx queue done"
                );
            }
        }
        .boxed())
    }

    /// Local address that all UDP sockets in this instance are bound to.
    pub fn bound_address(&self) -> &std::net::SocketAddr {
        &self.bound_to
    }

    /// Creates a new [builder](UdpStreamBuilder).
    pub fn builder() -> UdpStreamBuilder {
        UdpStreamBuilder::default()
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
        trace!(
            "polling read of {} from udp stream with {:?}",
            buf.remaining(),
            self.counterparty.get()
        );
        match Pin::new(&mut self.ingress_rx).poll_read(cx, buf) {
            Poll::Ready(Ok(())) => {
                let read = buf.filled().len();
                trace!("read {read} bytes");
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl tokio::io::AsyncWrite for ConnectedUdpStream {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        trace!(
            "polling write of {} bytes to udp stream with {:?}",
            buf.len(),
            self.counterparty.get()
        );
        if let Some(sender) = &mut self.egress_tx {
            if let Err(e) = ready!(sender.poll_ready_unpin(cx)) {
                return Poll::Ready(Err(e));
            }

            let len = buf.iter().len();
            if let Err(e) = sender.start_send_unpin(Box::from(buf)) {
                return Poll::Ready(Err(e));
            }

            // Explicitly flush after each data sent
            pin_mut!(sender);
            sender.poll_flush(cx).map_ok(|_| len)
        } else {
            Poll::Ready(Err(std::io::Error::new(
                ErrorKind::NotConnected,
                "udp stream is closed",
            )))
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        trace!("polling flush to udp stream {:?}", self.counterparty.get());
        if let Some(sender) = &mut self.egress_tx {
            pin_mut!(sender);
            sender
                .poll_flush(cx)
                .map_err(|err| std::io::Error::other(err.to_string()))
        } else {
            Poll::Ready(Err(std::io::Error::new(
                ErrorKind::NotConnected,
                "udp stream is closed",
            )))
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        trace!("polling close on udp stream with {:?}", self.counterparty.get());
        // Take the sender to make sure it gets dropped
        let mut taken_sender = self.egress_tx.take();
        if let Some(sender) = &mut taken_sender {
            pin_mut!(sender);
            sender
                .poll_close(cx)
                .map_err(|err| std::io::Error::other(err.to_string()))
        } else {
            Poll::Ready(Err(std::io::Error::new(
                ErrorKind::NotConnected,
                "udp stream is closed",
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use futures::future::Either;
    use parameterized::parameterized;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UdpSocket;

    #[parameterized(parallelism = {None, Some(2), Some(0)})]
    #[parameterized_macro(tokio::test)]
    //#[parameterized_macro(test_log::test(tokio::test))]
    async fn basic_udp_stream_tests(parallelism: Option<usize>) -> anyhow::Result<()> {
        const DATA_SIZE: usize = 200;

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

        let mut builder = ConnectedUdpStream::builder()
            .with_buffer_size(1024)
            .with_queue_size(512)
            .with_counterparty(listen_addr);

        if let Some(parallelism) = parallelism {
            builder = builder.with_parallelism(parallelism);
        }

        let mut stream = builder.build(("127.0.0.1", 0)).context("connection")?;

        for _ in 1..1000 {
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

    #[tokio::test]
    async fn udp_stream_should_process_sequential_writes() -> anyhow::Result<()> {
        const BUF_SIZE: usize = 1024;
        const EXPECTED_DATA_LEN: usize = BUF_SIZE + 500;

        let mut listener = ConnectedUdpStream::builder()
            .with_buffer_size(BUF_SIZE)
            .with_queue_size(512)
            .build(("127.0.0.1", 0))
            .context("bind listener")?;

        let bound_addr = *listener.bound_address();

        let jh = tokio::task::spawn(async move {
            let mut buf = [0u8; BUF_SIZE / 4];
            let mut vec = Vec::<u8>::new();
            loop {
                let sz = listener.read(&mut buf).await.unwrap();
                if sz > 0 {
                    vec.extend_from_slice(&buf[..sz]);
                    if vec.len() >= EXPECTED_DATA_LEN {
                        return vec;
                    }
                } else {
                    return vec;
                }
            }
        });

        let msg = [1u8; EXPECTED_DATA_LEN];
        let sender = UdpSocket::bind(("127.0.0.1", 0)).await.context("bind")?;

        sender.send_to(&msg[..BUF_SIZE], bound_addr).await?;
        sender.send_to(&msg[BUF_SIZE..], bound_addr).await?;

        let timeout = tokio::time::sleep(std::time::Duration::from_millis(1000));
        pin_mut!(timeout);
        pin_mut!(jh);

        match futures::future::select(jh, timeout).await {
            Either::Left((Ok(v), _)) => {
                assert_eq!(v.len(), EXPECTED_DATA_LEN);
                assert_eq!(v.as_slice(), &msg);
                Ok(())
            }
            _ => Err(anyhow::anyhow!("timeout or invalid data")),
        }
    }
}
