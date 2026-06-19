//! Infrastructure supporting converting a collection of `PeerId` split `libp2p_stream` managed
//! individual peer-to-peer `libp2p::swarm::Stream`s.

use std::sync::Arc;

use futures::{
    AsyncRead, AsyncReadExt, AsyncWrite, FutureExt, SinkExt as _, StreamExt,
    channel::mpsc::{Receiver, Sender, channel},
};
use hopr_api::network::NetworkStreamControl;
use hopr_utils::network_types::timeout::{SinkTimeoutError, TimeoutSinkExt};
use libp2p::PeerId;
use tokio_util::{
    codec::{Decoder, Encoder, FramedRead, FramedWrite},
    compat::{FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt},
};

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_PER_PEER_SEND_TIMEOUT: hopr_api::types::telemetry::SimpleCounter =
        hopr_api::types::telemetry::SimpleCounter::new(
            "hopr_egress_per_peer_send_timed_out",
            "Number of packets dropped due to per-peer egress send timeout",
        )
        .unwrap();
}

// TODO: see if these constants should be configurable instead

/// Global timeout for the `BidirectionalStreamControl::open` operation.
const GLOBAL_STREAM_OPEN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

/// Timeout for sending a single message into the per-peer mpsc buffer.
/// If the buffer stays full for longer than this, the message is dropped.
const DEFAULT_PER_PEER_SEND_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(50);
const MAX_CONCURRENT_PACKETS: usize = 30;

/// Default pending-write-buffer byte threshold on the framed writer before a flush is
/// forced. **This value is in bytes, not in messages** — that is how
/// `tokio_util::codec::FramedWrite::set_backpressure_boundary` is defined.
///
/// The previous value of `1` byte forced a flush syscall on every message (the buffer
/// exceeds one byte the moment a frame is encoded), making a relay forwarding N packets
/// issue N individual writes rather than coalescing adjacent small frames. `4096` bytes
/// is a conservative default that lets roughly ~4 HOPR packets (~1 KiB each) batch into
/// a single write while still flushing quickly under single-packet traffic.
///
/// Override with `HOPR_TRANSPORT_FRAME_WRITER_BACKPRESSURE_BYTES`.
const DEFAULT_FRAME_WRITER_BACKPRESSURE_BYTES: usize = 4096;

type PeerStreamCache<T> = moka::sync::Cache<PeerId, Sender<T>>;
type PeerOpenLockCache = moka::sync::Cache<PeerId, Arc<futures::lock::Mutex<()>>>;

fn build_peer_stream_io<S, C>(
    peer: PeerId,
    stream: S,
    cache: PeerStreamCache<<C as Decoder>::Item>,
    codec: C,
    ingress_from_peers: Sender<(PeerId, <C as Decoder>::Item)>,
    frame_writer_backpressure_bytes: usize,
    per_peer_channel_capacity: usize,
) -> Sender<<C as Decoder>::Item>
where
    S: AsyncRead + AsyncWrite + Send + 'static,
    C: Encoder<<C as Decoder>::Item> + Decoder + Send + Sync + Clone + 'static,
    <C as Encoder<<C as Decoder>::Item>>::Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    <C as Decoder>::Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    <C as Decoder>::Item: AsRef<[u8]> + Clone + Send + 'static,
{
    let (stream_rx, stream_tx) = stream.split();
    let (send, recv) = channel::<<C as Decoder>::Item>(per_peer_channel_capacity);
    let cache_for_write = cache.clone();
    let cache_for_read = cache.clone();

    let mut frame_writer = FramedWrite::new(stream_tx.compat_write(), codec.clone());

    // `set_backpressure_boundary` is a *byte* threshold on `FramedWrite`'s internal
    // pending-write buffer: once the encoded frames exceed this many bytes, the next
    // `poll_ready` call will issue a flush. A small value (e.g. `1`) flushes on every
    // message for the lowest latency at the cost of one syscall per frame; a larger
    // value lets adjacent small frames coalesce into a single write on busy relays.
    frame_writer.set_backpressure_boundary(frame_writer_backpressure_bytes);

    // Send all outgoing data to the peer
    hopr_utils::runtime::prelude::spawn(
        recv.inspect(move |_| tracing::trace!(%peer, "writing message to peer stream"))
            .map(Ok)
            .forward(frame_writer)
            .inspect(move |res| {
                tracing::debug!(%peer, ?res, component = "stream", "writing stream with peer finished");
            })
            .then(move |_| {
                // Make sure we invalidate the peer entry from the cache once the stream ends
                let peer = peer;
                async move {
                    cache_for_write.invalidate(&peer);
                }
            }),
    );

    // Read all incoming data from that peer and pass it to the general ingress stream
    hopr_utils::runtime::prelude::spawn(
        FramedRead::new(stream_rx.compat(), codec)
            .filter_map(move |v| {
                futures::future::ready(match v {
                    Ok(v) => {
                        tracing::trace!(%peer, "read message from peer stream");
                        Some((peer, v))
                    }
                    Err(error) => {
                        tracing::error!(%error, "Error decoding object from the underlying stream");
                        None
                    }
                })
            })
            .map(Ok)
            .forward(ingress_from_peers)
            .inspect(move |res| match res {
                Ok(_) => tracing::debug!(%peer, component = "stream", "incoming stream done reading"),
                Err(error) => {
                    tracing::error!(%peer, %error, component = "stream", "incoming stream failed on reading")
                }
            })
            .then(move |_| {
                // Make sure we invalidate the peer entry from the cache once the stream ends
                let peer = peer;
                async move {
                    cache_for_read.invalidate(&peer);
                }
            }),
    );

    tracing::trace!(%peer, "created new io for peer");
    send
}

pub async fn process_stream_protocol<C, V>(
    codec: C,
    control: V,
    stream_cfg: crate::config::StreamProtocolConfig,
) -> super::errors::Result<(
    Sender<(PeerId, <C as Decoder>::Item)>, // impl Sink<(PeerId, <C as Decoder>::Item)>,
    Receiver<(PeerId, <C as Decoder>::Item)>, // impl Stream<Item = (PeerId, <C as Decoder>::Item)>,
)>
where
    C: Encoder<<C as Decoder>::Item> + Decoder + Send + Sync + Clone + 'static,
    <C as Encoder<<C as Decoder>::Item>>::Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    <C as Decoder>::Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    <C as Decoder>::Item: AsRef<[u8]> + Clone + Send + 'static,
    V: NetworkStreamControl + Clone + Send + Sync + 'static,
{
    let (tx_out, rx_out) = channel::<(PeerId, <C as Decoder>::Item)>(100_000);
    let (tx_in, rx_in) = channel::<(PeerId, <C as Decoder>::Item)>(100_000);

    let cache_out: PeerStreamCache<<C as Decoder>::Item> = moka::sync::Cache::builder()
        .max_capacity(2000)
        .eviction_listener(|key: Arc<PeerId>, _, cause| {
            tracing::trace!(peer = %key.as_ref(), ?cause, "evicting stream for peer");
        })
        .build();

    // Serialize cache-miss stream opens per peer without holding a Moka initialization
    // guard across async network I/O.
    let open_locks: PeerOpenLockCache = moka::sync::Cache::builder().max_capacity(2000).build();

    let incoming = control
        .clone()
        .accept()
        .map_err(|e| super::errors::ProtocolError::Logic(format!("failed to listen on protocol: {e}")))?;

    let max_concurrent_packets = std::env::var("HOPR_TRANSPORT_MAX_CONCURRENT_PACKETS")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&n: &usize| n > 0)
        .unwrap_or(MAX_CONCURRENT_PACKETS);

    let global_stream_open_timeout = std::env::var("HOPR_TRANSPORT_STREAM_OPEN_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(std::time::Duration::from_millis)
        .unwrap_or(GLOBAL_STREAM_OPEN_TIMEOUT);

    let frame_writer_backpressure_bytes = std::env::var("HOPR_TRANSPORT_FRAME_WRITER_BACKPRESSURE_BYTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&n: &usize| n > 0)
        .unwrap_or(DEFAULT_FRAME_WRITER_BACKPRESSURE_BYTES);

    let per_peer_send_timeout = std::env::var("HOPR_TRANSPORT_PER_PEER_SEND_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(std::time::Duration::from_millis)
        .unwrap_or(DEFAULT_PER_PEER_SEND_TIMEOUT);

    let per_peer_channel_capacity = stream_cfg.per_peer_channel_capacity;

    // Pack the handles only needed to open a NEW peer stream into a single Arc. This
    // lets us pay one Arc bump per packet at the closure level instead of three (control,
    // codec, tx_in), and defers the per-field `.clone()` to the cache-miss path that
    // actually needs them.
    let open_ctx = Arc::new((control, codec, tx_in));

    let cache_ingress = cache_out.clone();
    let open_ctx_ingress = open_ctx.clone();

    // terminated when the incoming is dropped
    let _ingress_process = hopr_utils::runtime::prelude::spawn(
        incoming
            .for_each(move |(peer, stream)| {
                let cache = cache_ingress.clone();
                let open_ctx = open_ctx_ingress.clone();

                tracing::debug!(%peer, "received incoming peer-to-peer stream");
                let (_control, codec, tx_in) = (&open_ctx.0, &open_ctx.1, &open_ctx.2);
                let send = build_peer_stream_io(
                    peer,
                    stream,
                    cache.clone(),
                    codec.clone(),
                    tx_in.clone(),
                    frame_writer_backpressure_bytes,
                    per_peer_channel_capacity,
                );

                async move { cache.insert(peer, send) }
            })
            .inspect(|_| {
                tracing::info!(
                    task = "ingress stream processing",
                    "long-running background task finished"
                )
            }),
    );

    // terminated when the rx_in is dropped
    let _egress_process = hopr_utils::runtime::prelude::spawn(
        rx_out
            .inspect(|(peer, _)| tracing::trace!(%peer, "proceeding to deliver message to peer"))
            .for_each_concurrent(max_concurrent_packets, move |(peer, msg)| {
                let cache = cache_out.clone();
                let open_locks = open_locks.clone();
                let open_ctx = open_ctx.clone();

                async move {
                    tracing::trace!(%peer, "trying to deliver message to peer");

                    let cached = if let Some(cached) = cache.get(&peer) {
                        Ok(cached)
                    } else {
                        let open_lock = open_locks.get_with(peer, || Arc::new(futures::lock::Mutex::new(())));
                        let _guard = open_lock.lock().await;

                        if let Some(cached) = cache.get(&peer) {
                            Ok(cached)
                        } else {
                            tracing::trace!(%peer, "peer is not in cache, opening new stream");

                            // Only the cache-miss path needs the stream-open handles.
                            // Clone them out of the shared `open_ctx` once here rather
                            // than on every outgoing message.
                            let (control, codec, tx_in) = (&open_ctx.0, &open_ctx.1, &open_ctx.2);

                            // There must be a timeout on the `open` operation; otherwise
                            //  a single impossible ` open ` operation will block the peer ID entry in the
                            // cache forever, having a direct detrimental effect on the packet
                            // processing pipeline.
                            use futures_time::future::FutureExt as TimeExt;
                            let stream = control
                                .clone()
                                .open(peer)
                                .timeout(futures_time::time::Duration::from(global_stream_open_timeout))
                                .await
                                .map_err(|_| anyhow::anyhow!("timeout trying to open stream to {peer}"))
                                .and_then(|stream| {
                                    stream.map_err(|e| {
                                        anyhow::anyhow!("could not open outgoing peer-to-peer stream: {e}")
                                    })
                                });

                            match stream {
                                Ok(stream) => {
                                    tracing::debug!(%peer, "opening outgoing peer-to-peer stream");

                                    let send = build_peer_stream_io(
                                        peer,
                                        stream,
                                        cache.clone(),
                                        codec.clone(),
                                        tx_in.clone(),
                                        frame_writer_backpressure_bytes,
                                        per_peer_channel_capacity,
                                    );
                                    cache.insert(peer, send.clone());
                                    Ok(send)
                                }
                                Err(error) => Err(error),
                            }
                        }
                    };

                    match cached {
                        Ok(cached) => match cached.with_timeout(per_peer_send_timeout).send(msg).await {
                            Ok(()) => tracing::trace!(%peer, "message sent to peer"),
                            Err(SinkTimeoutError::Timeout) => {
                                #[cfg(all(feature = "telemetry", not(test)))]
                                METRIC_PER_PEER_SEND_TIMEOUT.increment();
                                tracing::warn!(%peer, "per-peer egress send timed out, dropping packet");
                            }
                            Err(SinkTimeoutError::Inner(error)) => {
                                tracing::error!(%peer, %error, "error sending message to peer");
                                cache.invalidate(&peer);
                            }
                        },
                        Err(error) => {
                            tracing::debug!(%peer, %error, "failed to open a stream to peer");
                        }
                    }
                }
            })
            .inspect(|_| {
                tracing::info!(
                    task = "egress stream processing",
                    "long-running background task finished"
                )
            }),
    );

    Ok((tx_out, rx_in))
}

#[cfg(test)]
mod tests {
    use std::{
        pin::Pin,
        sync::{
            Arc, Mutex,
            atomic::{AtomicUsize, Ordering},
        },
        task::{Context as TaskContext, Poll, Waker},
    };

    use anyhow::Context;
    use async_trait::async_trait;
    use futures::{SinkExt, Stream};
    use tokio_util::{bytes::BytesMut, codec::BytesCodec};

    use super::*;

    #[derive(Clone, Default, Debug)]
    struct CountingControl {
        open_calls: Arc<AtomicUsize>,
    }

    impl CountingControl {
        fn open_calls(&self) -> usize {
            self.open_calls.load(Ordering::Relaxed)
        }
    }

    #[derive(Default)]
    struct StalledWriteIo;

    impl AsyncRead for StalledWriteIo {
        fn poll_read(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>, _buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
            Poll::Pending
        }
    }

    impl AsyncWrite for StalledWriteIo {
        fn poll_write(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>, _buf: &[u8]) -> Poll<std::io::Result<usize>> {
            Poll::Pending
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>) -> Poll<std::io::Result<()>> {
            Poll::Pending
        }

        fn poll_close(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>) -> Poll<std::io::Result<()>> {
            Poll::Pending
        }
    }

    #[async_trait]
    impl hopr_api::network::traits::NetworkStreamControl for CountingControl {
        fn accept(
            self,
        ) -> Result<impl Stream<Item = (PeerId, impl AsyncRead + AsyncWrite + Send)> + Send, impl std::error::Error>
        {
            Ok::<_, std::io::Error>(futures::stream::empty::<(PeerId, StalledWriteIo)>())
        }

        async fn open(self, _peer: PeerId) -> Result<impl AsyncRead + AsyncWrite + Send, impl std::error::Error> {
            self.open_calls.fetch_add(1, Ordering::Relaxed);
            Ok::<_, std::io::Error>(StalledWriteIo)
        }
    }

    struct AsyncBinaryStreamChannel {
        read: async_channel_io::ChannelReader,
        write: async_channel_io::ChannelWriter,
    }

    impl AsyncBinaryStreamChannel {
        pub fn new() -> Self {
            let (write, read) = async_channel_io::pipe();
            Self { read, write }
        }
    }

    impl AsyncRead for AsyncBinaryStreamChannel {
        fn poll_read(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &mut [u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            let mut pinned = std::pin::pin!(&mut self.get_mut().read);
            pinned.as_mut().poll_read(cx, buf)
        }
    }

    impl AsyncWrite for AsyncBinaryStreamChannel {
        fn poll_write(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            let mut pinned = std::pin::pin!(&mut self.get_mut().write);
            pinned.as_mut().poll_write(cx, buf)
        }

        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            let pinned = std::pin::pin!(&mut self.get_mut().write);
            pinned.poll_flush(cx)
        }

        fn poll_close(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            let pinned = std::pin::pin!(&mut self.get_mut().write);
            pinned.poll_close(cx)
        }
    }

    #[tokio::test]
    async fn split_codec_should_always_produce_correct_data() -> anyhow::Result<()> {
        let stream = AsyncBinaryStreamChannel::new();
        let codec = tokio_util::codec::BytesCodec::new();

        let expected = [0u8, 1u8, 2u8, 3u8, 4u8, 5u8];
        let value = tokio_util::bytes::BytesMut::from(expected.as_ref());

        let (stream_rx, stream_tx) = stream.split();
        let (mut tx, rx) = (
            FramedWrite::new(stream_tx.compat_write(), codec),
            FramedRead::new(stream_rx.compat(), codec),
        );
        tx.send(value)
            .await
            .map_err(|_| anyhow::anyhow!("should not fail on send"))?;

        futures::pin_mut!(rx);

        assert_eq!(
            rx.next().await.context("Value must be present")??,
            tokio_util::bytes::BytesMut::from(expected.as_ref())
        );

        Ok(())
    }

    // ---------------------------------------------------------------------------
    // FlaggedStream — models a LivenessStream whose connection has been killed.
    //
    // While alive the stream stalls on every poll (like a half-open connection).
    // When `DeadSignal::kill()` is called, the stored waker is woken and the
    // next poll returns `Err(io::ErrorKind::ConnectionAborted)`, exactly as
    // `LivenessStream` behaves when the swarm clears the liveness flag.
    // ---------------------------------------------------------------------------

    struct DeadSignal {
        dead: std::sync::atomic::AtomicBool,
        read_waker: Mutex<Option<Waker>>,
        write_waker: Mutex<Option<Waker>>,
    }

    impl std::fmt::Debug for DeadSignal {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("DeadSignal")
                .field("dead", &self.dead.load(Ordering::Relaxed))
                .finish_non_exhaustive()
        }
    }

    impl DeadSignal {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                dead: std::sync::atomic::AtomicBool::new(false),
                read_waker: Mutex::new(None),
                write_waker: Mutex::new(None),
            })
        }

        /// Simulate a libp2p connection close: set the dead flag and re-wake
        /// any parked reader/writer tasks so they observe the error on their
        /// next poll.
        fn kill(self: &Arc<Self>) {
            self.dead.store(true, Ordering::Release);
            if let Some(w) = self.read_waker.lock().expect("lock ok").take() {
                w.wake();
            }
            if let Some(w) = self.write_waker.lock().expect("lock ok").take() {
                w.wake();
            }
        }
    }

    struct FlaggedStream {
        signal: Arc<DeadSignal>,
    }

    impl AsyncRead for FlaggedStream {
        fn poll_read(self: Pin<&mut Self>, cx: &mut TaskContext<'_>, _buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
            if self.signal.dead.load(Ordering::Acquire) {
                return Poll::Ready(Err(std::io::Error::from(std::io::ErrorKind::ConnectionAborted)));
            }
            *self.signal.read_waker.lock().expect("lock ok") = Some(cx.waker().clone());
            Poll::Pending
        }
    }

    impl AsyncWrite for FlaggedStream {
        fn poll_write(self: Pin<&mut Self>, cx: &mut TaskContext<'_>, _buf: &[u8]) -> Poll<std::io::Result<usize>> {
            if self.signal.dead.load(Ordering::Acquire) {
                return Poll::Ready(Err(std::io::Error::from(std::io::ErrorKind::ConnectionAborted)));
            }
            *self.signal.write_waker.lock().expect("lock ok") = Some(cx.waker().clone());
            Poll::Pending
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    /// A `NetworkStreamControl` that hands out `FlaggedStream`s from `open()`.
    ///
    /// Each `open()` call appends the new stream's `DeadSignal` to `signals`
    /// so the test can kill individual streams.
    #[derive(Clone, Debug, Default)]
    struct ScriptedControl {
        open_calls: Arc<AtomicUsize>,
        signals: Arc<Mutex<Vec<Arc<DeadSignal>>>>,
    }

    impl ScriptedControl {
        fn open_calls(&self) -> usize {
            self.open_calls.load(Ordering::Relaxed)
        }

        /// Returns the `DeadSignal` for the `index`-th stream that was opened.
        fn signal(&self, index: usize) -> Option<Arc<DeadSignal>> {
            self.signals.lock().expect("lock ok").get(index).cloned()
        }
    }

    #[async_trait]
    impl hopr_api::network::traits::NetworkStreamControl for ScriptedControl {
        fn accept(
            self,
        ) -> Result<impl Stream<Item = (PeerId, impl AsyncRead + AsyncWrite + Send)> + Send, impl std::error::Error>
        {
            Ok::<_, std::io::Error>(futures::stream::empty::<(PeerId, FlaggedStream)>())
        }

        async fn open(self, _peer: PeerId) -> Result<impl AsyncRead + AsyncWrite + Send, impl std::error::Error> {
            self.open_calls.fetch_add(1, Ordering::Relaxed);
            let signal = DeadSignal::new();
            self.signals.lock().expect("lock ok").push(signal.clone());
            Ok::<_, std::io::Error>(FlaggedStream { signal })
        }
    }

    /// Waits up to `secs` seconds for `condition()` to return `true`, polling
    /// every 25 ms. Returns `true` if the condition was met.
    async fn wait_for(secs: u64, condition: impl Fn() -> bool) -> bool {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(secs);
        while !condition() && tokio::time::Instant::now() < deadline {
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
        condition()
    }

    // ---------------------------------------------------------------------------
    // New tests: connection-death recovery
    // ---------------------------------------------------------------------------

    /// Verifies that a stream error on the **read path** invalidates the cache
    /// so the next egress send re-opens a fresh stream.
    ///
    /// This is the core regression scenario: a previously healthy stream is
    /// killed (simulating what `LivenessStream` does on `ConnectionClosed`).
    /// The reader task detects the `ConnectionAborted` error and invalidates
    /// the cache. The next outgoing packet causes a cache miss and a new stream
    /// is opened.
    #[tokio::test]
    async fn dead_stream_should_be_detected_on_read_path_and_allow_next_send_to_reopen() -> anyhow::Result<()> {
        let control = ScriptedControl::default();

        let (mut tx_out, _rx_in) = process_stream_protocol(
            BytesCodec::new(),
            control.clone(),
            crate::config::StreamProtocolConfig {
                per_peer_channel_capacity: 64,
            },
        )
        .await?;

        let peer = PeerId::random();
        let msg = BytesMut::from(&b"probe"[..]);

        // First send: triggers a cache miss → open() #1 is called, stream cached.
        tx_out
            .send((peer, msg.clone()))
            .await
            .context("first send should succeed")?;

        // Wait for the stream to be opened.
        assert!(
            wait_for(2, || control.open_calls() >= 1).await,
            "stream was never opened"
        );
        let signal = control.signal(0).context("signal for stream #1 must exist")?;

        // Kill the stream: simulates a libp2p ConnectionClosed event clearing the
        // liveness flag. The stored read waker is woken so the parked reader task
        // observes ConnectionAborted and calls cache.invalidate(peer).
        signal.kill();

        // Wait for the reader task to detect the error and invalidate the cache,
        // then send a second packet that causes a new cache miss → open() #2.
        tx_out
            .send((peer, msg.clone()))
            .await
            .context("second send into egress queue should succeed")?;

        assert!(
            wait_for(2, || control.open_calls() >= 2).await,
            "stream was not reopened after connection kill (open_calls={})",
            control.open_calls()
        );

        Ok(())
    }

    /// Verifies that killing a stream while the **writer** is active also leads
    /// to cache invalidation and reopen.
    ///
    /// Sends enough packets to keep the egress task busy writing, then kills
    /// the stream. The writer forward-future ends on the `ConnectionAborted`
    /// error and invalidates the cache, causing the next send to reopen.
    #[tokio::test]
    async fn dead_stream_should_be_detected_on_write_path_and_allow_next_send_to_reopen() -> anyhow::Result<()> {
        let control = ScriptedControl::default();

        let (mut tx_out, _rx_in) = process_stream_protocol(
            BytesCodec::new(),
            control.clone(),
            crate::config::StreamProtocolConfig {
                per_peer_channel_capacity: 128,
            },
        )
        .await?;

        let peer = PeerId::random();
        let msg = BytesMut::from(&b"payload"[..]);

        // Open a stream for the peer.
        tx_out.send((peer, msg.clone())).await.context("initial send")?;
        assert!(wait_for(2, || control.open_calls() >= 1).await, "stream not opened");

        let signal = control.signal(0).context("signal #1 must exist")?;

        // Kill the stream; the writer's forward-future will error on next poll.
        signal.kill();

        // Drain the per-peer channel so the writer unparks and observes the error.
        for _ in 0..32 {
            let _ = tx_out.send((peer, msg.clone())).await;
        }

        assert!(
            wait_for(3, || control.open_calls() >= 2).await,
            "stream was not reopened after writer kill (open_calls={})",
            control.open_calls()
        );

        Ok(())
    }

    #[tokio::test]
    async fn per_peer_stream_should_not_reopen_pathologically_on_send_failures() -> anyhow::Result<()> {
        let control = CountingControl::default();
        // Use a small channel so the 1 200 sends overflow it quickly, exercising
        // the send-timeout/drop path. The timeout fires when trying to push into a
        // full channel (not based on queued-packet age); on timeout the packet is
        // dropped as a transport loss, but the cached sender must not be evicted and
        // the stream must not be reopened.
        let (mut tx_out, _rx_in) = process_stream_protocol(
            BytesCodec::new(),
            control.clone(),
            crate::config::StreamProtocolConfig {
                per_peer_channel_capacity: 16,
            },
        )
        .await?;

        let peer = PeerId::random();
        let msg = BytesMut::from(&b"x"[..]);

        for _ in 0..1200 {
            tx_out
                .send((peer, msg.clone()))
                .await
                .context("egress queue should accept test packet")?;
        }

        // Wait for at least one stream open (first cache miss).
        let open_deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(1);
        while control.open_calls() < 1 && tokio::time::Instant::now() < open_deadline {
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
        assert!(
            control.open_calls() >= 1,
            "stream was never opened — egress task did not process any packet"
        );

        // Now give it another second to reopen pathologically; it must not.
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(1);
        while control.open_calls() < 2 && tokio::time::Instant::now() < deadline {
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }

        assert!(
            control.open_calls() <= 1,
            "pathological reopen churn detected for same peer under send failures (open_calls={})",
            control.open_calls()
        );

        Ok(())
    }
}
