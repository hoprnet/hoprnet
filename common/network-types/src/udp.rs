use std::{
    fmt::Debug,
    io::ErrorKind,
    num::NonZeroUsize,
    pin::Pin,
    sync::{Arc, OnceLock},
    task::{Context, Poll},
};

use futures::{FutureExt, Sink, SinkExt, ready};
use tokio::net::UdpSocket;
use tracing::{debug, error, instrument, trace, warn};

use crate::utils::SocketAddrStr;

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
/// The instance must always be constructed using a [`UdpStreamBuilder`].
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
/// If [parallelism](UdpStreamBuilder::with_receiver_parallelism) is set, the instance will create
/// multiple sockets with `SO_REUSEADDR` and spin parallel tasks that will coordinate data and
/// transmission retrieval using these sockets. This is driven by RX/TX MPMC queues, which are
/// per-default unbounded (see [queue size](UdpStreamBuilder::with_queue_size) for details).
#[pin_project::pin_project]
pub struct ConnectedUdpStream {
    socket_handles: Vec<tokio::task::JoinHandle<()>>,
    #[pin]
    ingress_rx: Box<dyn tokio::io::AsyncRead + Send + Unpin>,
    #[pin]
    egress_tx: Option<BoxIoSink<Box<[u8]>>>,
    counterparty: Arc<OnceLock<SocketAddrStr>>,
    bound_to: std::net::SocketAddr,
    state: State,
}

#[derive(Copy, Clone)]
enum State { Writing, Flushing(usize) }

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

/// Determines how many parallel readers or writer sockets should be bound in [`ConnectedUdpStream`].
///
/// Each UDP socket is bound with `SO_REUSEADDR` and `SO_REUSEPORT` to facilitate parallel processing
/// of send and/or receive operations.
///
/// **NOTE**: This is a Linux-specific optimization, and it will have no effect on other systems.
///
/// - If some [`Specific`](UdpStreamParallelism::Specific) value `n` > 0 is given, the [`ConnectedUdpStream`] will bind
///   `n` sockets.
/// - If [`Auto`](UdpStreamParallelism::Auto) is given, the number of sockets bound by [`ConnectedUdpStream`] is
///   determined by [`std::thread::available_parallelism`].
///
/// The default is `Specific(1)`.
///
/// Always use [`into_num_tasks`](UdpStreamParallelism::into_num_tasks) or
/// [`split_evenly_with`](UdpStreamParallelism::split_evenly_with) to determine the correct number of sockets to spawn.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UdpStreamParallelism {
    /// Bind as many sender or receiver sockets as given by [`std::thread::available_parallelism`].
    Auto,
    /// Bind a specific number of sender or receiver sockets.
    Specific(NonZeroUsize),
}

impl Default for UdpStreamParallelism {
    fn default() -> Self {
        Self::Specific(NonZeroUsize::MIN)
    }
}

impl UdpStreamParallelism {
    fn avail_parallelism() -> usize {
        // On non-Linux system this will always default to 1, since the
        // multiple UDP socket optimization is not possible for those platforms.
        std::thread::available_parallelism()
            .map(|n| {
                if cfg!(target_os = "linux") {
                    n
                } else {
                    NonZeroUsize::MIN
                }
            })
            .unwrap_or_else(|e| {
                warn!(error = %e, "failed to determine available parallelism, defaulting to 1.");
                NonZeroUsize::MIN
            })
            .into()
    }

    /// Returns the number of sockets for this and the `other` instance
    /// when they evenly split the available CPU parallelism.
    pub fn split_evenly_with(self, other: UdpStreamParallelism) -> (usize, usize) {
        let cpu_half = (Self::avail_parallelism() / 2).max(1);

        match (self, other) {
            (UdpStreamParallelism::Auto, UdpStreamParallelism::Auto) => (cpu_half, cpu_half),
            (UdpStreamParallelism::Specific(a), UdpStreamParallelism::Auto) => {
                let a = cpu_half.min(a.into());
                (a, cpu_half * 2 - a)
            }
            (UdpStreamParallelism::Auto, UdpStreamParallelism::Specific(b)) => {
                let b = cpu_half.min(b.into());
                (cpu_half * 2 - b, b)
            }
            (UdpStreamParallelism::Specific(a), UdpStreamParallelism::Specific(b)) => {
                (cpu_half.min(a.into()), cpu_half.min(b.into()))
            }
        }
    }

    /// Calculates the actual number of tasks for this instance.
    ///
    /// The returned value is never more than the maximum available CPU parallelism.
    pub fn into_num_tasks(self) -> usize {
        let avail_parallelism = Self::avail_parallelism();
        match self {
            UdpStreamParallelism::Auto => avail_parallelism,
            UdpStreamParallelism::Specific(n) => usize::from(n).min(avail_parallelism),
        }
    }
}

impl From<usize> for UdpStreamParallelism {
    fn from(value: usize) -> Self {
        NonZeroUsize::new(value).map(Self::Specific).unwrap_or_default()
    }
}

impl From<Option<usize>> for UdpStreamParallelism {
    fn from(value: Option<usize>) -> Self {
        value.map(UdpStreamParallelism::from).unwrap_or_default()
    }
}

/// Builder object for the [`ConnectedUdpStream`].
///
/// If you wish to use defaults, do `UdpStreamBuilder::default().build(addr)`.
#[derive(Debug, Clone)]
pub struct UdpStreamBuilder {
    foreign_data_mode: ForeignDataMode,
    buffer_size: usize,
    queue_size: Option<usize>,
    receiver_parallelism: UdpStreamParallelism,
    sender_parallelism: UdpStreamParallelism,
    counterparty: Option<std::net::SocketAddr>,
}

impl Default for UdpStreamBuilder {
    fn default() -> Self {
        Self {
            buffer_size: 2048,
            foreign_data_mode: Default::default(),
            queue_size: None,
            receiver_parallelism: Default::default(),
            sender_parallelism: Default::default(),
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

    /// Sets how many parallel receiving sockets should be bound.
    ///
    /// Has no effect on non-Linux machines. See [`UdpStreamParallelism`] for details.
    ///
    /// Default is `1`.
    pub fn with_receiver_parallelism<T: Into<UdpStreamParallelism>>(mut self, parallelism: T) -> Self {
        self.receiver_parallelism = parallelism.into();
        self
    }

    /// Sets how many parallel sending sockets should be bound.
    ///
    /// Has no effect on non-Linux machines. See [`UdpStreamParallelism`] for details.
    ///
    /// Default is `1`.
    pub fn with_sender_parallelism<T: Into<UdpStreamParallelism>>(mut self, parallelism: T) -> Self {
        self.sender_parallelism = parallelism.into();
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
    /// The number of RX sockets bound is determined by [receiver
    /// parallelism](UdpStreamBuilder::with_receiver_parallelism), and similarly, the number of TX sockets bound is
    /// determined by [sender parallelism](UdpStreamBuilder::with_sender_parallelism). On non-Linux platforms, only
    /// a single receiver and sender will be bound, regardless of the above.
    ///
    /// The returned instance is always ready to receive data.
    /// It is also ready to send data
    /// if the [counterparty](UdpStreamBuilder::with_counterparty) has been set.
    ///
    /// If `bind_addr` yields multiple addresses, binding will be attempted with each of the addresses
    /// until one succeeds. If none of the addresses succeed in binding the socket(s),
    /// the `AddrNotAvailable` error is returned.
    ///
    /// Note that wildcard addresses (such as `0.0.0.0`) are *not* considered as multiple addresses,
    /// and such socket(s) will bind to all available interfaces at the system level.
    pub fn build<A: std::net::ToSocketAddrs>(self, bind_addr: A) -> std::io::Result<ConnectedUdpStream> {
        let (num_rx_socks, num_tx_socks) = self.receiver_parallelism.split_evenly_with(self.sender_parallelism);

        let counterparty = Arc::new(
            self.counterparty
                .map(|s| OnceLock::from(SocketAddrStr::from(s)))
                .unwrap_or_default(),
        );
        let ((ingress_tx, ingress_rx), (egress_tx, egress_rx)) = if let Some(q) = self.queue_size {
            (flume::bounded(q), flume::bounded(q))
        } else {
            (flume::unbounded(), flume::unbounded())
        };

        let num_socks_to_bind = num_rx_socks.max(num_tx_socks);
        let mut socket_handles = Vec::with_capacity(num_socks_to_bind);
        let mut bound_addr: Option<std::net::SocketAddr> = None;

        // Try binding on all network addresses in `bind_addr`
        for binding_to in bind_addr.to_socket_addrs()? {
            debug!(
                %binding_to,
                num_socks_to_bind, num_rx_socks, num_tx_socks, "binding UDP stream"
            );

            // TODO: split bound sockets into a separate cloneable object

            // Try to bind sockets on the current network interface address
            (0..num_socks_to_bind)
                .map(|sock_id| {
                    let domain = match &binding_to {
                        std::net::SocketAddr::V4(_) => socket2::Domain::IPV4,
                        std::net::SocketAddr::V6(_) => socket2::Domain::IPV6,
                    };

                    // Bind a new non-blocking UDP socket
                    let sock = socket2::Socket::new(domain, socket2::Type::DGRAM, None)?;
                    if num_socks_to_bind > 1 {
                        sock.set_reuse_address(true)?; // Needed for every next socket with non-wildcard IP
                        sock.set_reuse_port(true)?; // Needed on Linux to evenly distribute datagrams
                    }
                    sock.set_nonblocking(true)?;
                    sock.bind(&bound_addr.unwrap_or(binding_to).into())?;

                    // Determine the address we bound this socket to, so we can also bind the others
                    let socket_bound_addr = sock
                        .local_addr()?
                        .as_socket()
                        .ok_or(std::io::Error::other("invalid socket type"))?;

                    match bound_addr {
                        None => bound_addr = Some(socket_bound_addr),
                        Some(addr) if addr != socket_bound_addr => {
                            return Err(std::io::Error::other(format!(
                                "inconsistent binding address {addr} != {socket_bound_addr} on socket id {sock_id}"
                            )));
                        }
                        _ => {}
                    }

                    let sock = Arc::new(UdpSocket::from_std(sock.into())?);
                    debug!(
                        socket_id = sock_id,
                        addr = %socket_bound_addr,
                        "bound UDP socket"
                    );

                    Ok((sock_id, sock))
                })
                .filter_map(|result| match result {
                    Ok(bound) => Some(bound),
                    Err(e) => {
                        error!(
                            %binding_to,
                            "failed to bind udp socket: {e}"
                        );
                        None
                    }
                })
                .for_each(|(sock_id, sock)| {
                    if sock_id < num_tx_socks {
                        socket_handles.push(tokio::task::spawn(ConnectedUdpStream::setup_tx_queue(
                            sock_id,
                            sock.clone(),
                            egress_rx.clone(),
                            counterparty.clone(),
                        )));
                    }
                    if sock_id < num_rx_socks {
                        socket_handles.push(tokio::task::spawn(ConnectedUdpStream::setup_rx_queue(
                            sock_id,
                            sock.clone(),
                            ingress_tx.clone(),
                            counterparty.clone(),
                            self.foreign_data_mode,
                            self.buffer_size,
                        )));
                    }
                });
        }

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
            state: State::Writing
        })
    }
}

impl ConnectedUdpStream {
    /// Creates a receiver queue for the UDP stream.
    fn setup_rx_queue(
        socket_id: usize,
        sock_rx: Arc<UdpSocket>,
        ingress_tx: flume::Sender<std::io::Result<tokio_util::bytes::Bytes>>,
        counterparty: Arc<OnceLock<SocketAddrStr>>,
        foreign_data_mode: ForeignDataMode,
        buf_size: usize,
    ) -> futures::future::BoxFuture<'static, ()> {
        let counterparty_rx = counterparty.clone();
        async move {
            let mut buffer = vec![0u8; buf_size];
            let mut done = false;
            loop {
                // Read data from the socket
                let out_res = match sock_rx.recv_from(&mut buffer).await {
                    Ok((read, read_addr)) if read > 0 => {
                        trace!(
                            socket_id,
                            udp_bound_addr = ?sock_rx.local_addr(),
                            bytes = read,
                            from = %read_addr,
                            "received data from"
                        );

                        let addr = counterparty_rx.get_or_init(|| read_addr.into());

                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_UDP_INGRESS_LEN.observe(&[addr.as_str()], read as f64);

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
                                        udp_bound_addr = ?sock_rx.local_addr(),
                                        ?read_addr,
                                        expected_addr = ?addr,
                                        "discarded data, which didn't come from the expected address"
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
                            udp_bound_addr = ?sock_rx.local_addr(),
                            "read EOF on socket"
                        );
                        done = true;
                        None
                    }
                    Err(e) => {
                        // Forward the error
                        debug!(
                            socket_id,
                            udp_bound_addr = ?sock_rx.local_addr(),
                            error = %e,
                            "forwarded error from socket"
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
                            udp_bound_addr = ?sock_rx.local_addr(),
                            error = %err,
                            "failed to dispatch received data"
                        );
                        done = true;
                    }
                }

                if done {
                    trace!(
                        socket_id,
                        udp_bound_addr = ?sock_rx.local_addr(),
                        "rx queue done"
                    );
                    break;
                }
            }
        }
        .boxed()
    }

    /// Creates a transmission queue for the UDP stream.
    fn setup_tx_queue(
        socket_id: usize,
        sock_tx: Arc<UdpSocket>,
        egress_rx: flume::Receiver<Box<[u8]>>,
        counterparty: Arc<OnceLock<SocketAddrStr>>,
    ) -> futures::future::BoxFuture<'static, ()> {
        let counterparty_tx = counterparty.clone();
        async move {
            loop {
                match egress_rx.recv_async().await {
                    Ok(data) => {
                        if let Some(target) = counterparty_tx.get() {
                            if let Err(e) = sock_tx.send_to(&data, target.as_ref()).await {
                                error!(
                                    ?socket_id,
                                    udp_bound_addr = ?sock_tx.local_addr(),
                                    ?target,
                                    error = %e,
                                    "failed to send data"
                                );
                            }
                            trace!(socket_id, bytes = data.len(), ?target, "sent bytes to");

                            #[cfg(all(feature = "prometheus", not(test)))]
                            METRIC_UDP_EGRESS_LEN.observe(&[target.as_str()], data.len() as f64);
                        } else {
                            error!(
                                ?socket_id,
                                udp_bound_addr = ?sock_tx.local_addr(),
                                "cannot send data, counterparty not set"
                            );
                            break;
                        }
                    }
                    Err(e) => {
                        error!(
                            ?socket_id,
                            udp_bound_addr = ?sock_tx.local_addr(),
                            error = %e,
                            "cannot receive more data from egress channel"
                        );
                        break;
                    }
                }
                trace!(
                    ?socket_id,
                    udp_bound_addr = tracing::field::debug(sock_tx.local_addr()),
                    "tx queue done"
                );
            }
        }
        .boxed()
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

impl tokio::io::AsyncRead for ConnectedUdpStream {
    #[instrument(name = "ConnectedUdpStream::poll_read", level = "trace", skip(self, cx), fields(counterparty = ?self.counterparty.get(), rem = buf.remaining()) , ret)]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        ready!(self.project().ingress_rx.poll_read(cx, buf))?;
        trace!(bytes = buf.filled().len(), "read bytes");
        Poll::Ready(Ok(()))
    }
}

impl tokio::io::AsyncWrite for ConnectedUdpStream {
    #[instrument(name = "ConnectedUdpStream::poll_write", level = "trace", skip(self, cx), fields(counterparty = ?self.counterparty.get(), len = buf.len()) , ret)]
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        let this = self.project();
        if let Some(sender) = this.egress_tx.get_mut() {
            loop {
                match *this.state {
                    State::Writing => {
                        ready!(sender.poll_ready_unpin(cx))?;

                        let len = buf.iter().len();
                        sender.start_send_unpin(Box::from(buf))?;
                        *this.state = State::Flushing(len);
                    }
                    State::Flushing(len) => {
                        // Explicitly flush after each data sent
                        ready!(sender.poll_flush_unpin(cx))?;
                        *this.state = State::Writing;

                        return Poll::Ready(Ok(len));
                    }
                }
            }
        } else {
            Poll::Ready(Err(std::io::Error::new(
                ErrorKind::NotConnected,
                "udp stream is closed",
            )))
        }
    }

    #[instrument(name = "ConnectedUdpStream::poll_flush", level = "trace", skip(self, cx), fields(counterparty = ?self.counterparty.get()) , ret)]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let this = self.project();
        if let Some(sender) = this.egress_tx.as_pin_mut() {
            sender
                .poll_flush(cx)
                .map_err(std::io::Error::other)
        } else {
            Poll::Ready(Err(std::io::Error::new(
                ErrorKind::NotConnected,
                "udp stream is closed",
            )))
        }
    }

    #[instrument(name = "ConnectedUdpStream::poll_shutdown", level = "trace", skip(self, cx), fields(counterparty = ?self.counterparty.get()) , ret)]
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let this = self.project();
        if let Some(sender) = this.egress_tx.as_pin_mut() {
            let ret = ready!(sender.poll_close(cx));

            this.socket_handles.iter().for_each(|handle| {
                handle.abort();
            });

            Poll::Ready(ret)
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
    use anyhow::Context;
    use futures::future::Either;
    use futures::pin_mut;
    use parameterized::parameterized;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::UdpSocket,
    };

    use super::*;

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
            builder = builder.with_receiver_parallelism(parallelism);
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
