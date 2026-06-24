//! Infrastructure supporting converting a collection of `PeerId` split `libp2p_stream` managed
//! individual peer-to-peer `libp2p::swarm::Stream`s.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use futures::{
    AsyncRead, AsyncReadExt, AsyncWrite, FutureExt, StreamExt,
    channel::mpsc::{Receiver, Sender, channel},
};
use hopr_api::network::NetworkStreamControl;
use libp2p::PeerId;
use tokio_util::{
    codec::{Decoder, Encoder, FramedRead, FramedWrite},
    compat::{FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt},
};

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_RING_BUFFER_DROPPED: hopr_api::types::telemetry::SimpleCounter =
        hopr_api::types::telemetry::SimpleCounter::new(
            "hopr_egress_ring_buffer_dropped",
            "Number of packets dropped due to per-peer egress ring buffer overflow (drop-oldest)",
        )
        .unwrap();
}

/// Per-peer egress buffer: a bounded flume channel used as a drop-oldest ring buffer.
///
/// Both `tx` and `rx` are stored in the cache entry so the single-producer
/// egress drain can implement drop-oldest overflow: when `tx.try_send` returns
/// `Full`, the drain calls `rx.try_recv` to evict the oldest packet and retries.
/// Keeping the `rx` clone in the cache also prevents the channel from reporting
/// `Disconnected` to the producer; dead-stream detection relies exclusively on
/// the `cache.invalidate` call made by both pump tasks when the stream ends.
#[derive(Clone)]
struct PeerSink<T: Send + 'static> {
    tx: flume::Sender<T>,
    rx: flume::Receiver<T>,
}

type PeerStreamCache<T> = moka::sync::Cache<PeerId, PeerSink<T>>;

/// Spawn the write and read pump tasks for an open peer stream.
///
/// The write pump drains `rx` into the framed stream writer; the read pump
/// forwards decoded frames to `ingress_from_peers`. Both tasks invalidate
/// `cache[peer]` when they complete — the dead-stream signal that causes the
/// next egress send to re-open a fresh stream.
fn spawn_stream_pumps<S, C>(
    peer: PeerId,
    stream: S,
    rx: flume::Receiver<<C as Decoder>::Item>,
    cache: PeerStreamCache<<C as Decoder>::Item>,
    codec: C,
    ingress_from_peers: Sender<(PeerId, <C as Decoder>::Item)>,
    frame_writer_backpressure_bytes: usize,
) where
    S: AsyncRead + AsyncWrite + Send + 'static,
    C: Encoder<<C as Decoder>::Item> + Decoder + Send + Sync + Clone + 'static,
    <C as Encoder<<C as Decoder>::Item>>::Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    <C as Decoder>::Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    <C as Decoder>::Item: AsRef<[u8]> + Clone + Send + 'static,
{
    let (stream_rx, stream_tx) = stream.split();
    let cache_for_write = cache.clone();
    let cache_for_read = cache.clone();

    let mut frame_writer = FramedWrite::new(stream_tx.compat_write(), codec.clone());

    // `set_backpressure_boundary` is a *byte* threshold on `FramedWrite`'s internal
    // pending-write buffer: once the encoded frames exceed this many bytes, the next
    // `poll_ready` call will issue a flush. A small value (e.g. `1`) flushes on every
    // message for the lowest latency at the cost of one syscall per frame; a larger
    // value lets adjacent small frames coalesce into a single write on busy relays.
    frame_writer.set_backpressure_boundary(frame_writer_backpressure_bytes);

    // Write pump: drain the per-peer ring buffer into the framed stream writer.
    // The channel closes (into_stream yields None) when all senders are dropped,
    // which signals the pump to stop and invalidate the cache.
    hopr_utils::runtime::prelude::spawn(
        rx.into_stream()
            .inspect(move |_| tracing::trace!(%peer, "writing message to peer stream"))
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

    // Read pump: forward decoded frames to the ingress channel.
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

    // Bounds the number of in-flight stream-open tasks across all distinct peers.
    // Without this, a flood to many unreachable peers could spawn one 2 s opener
    // task per peer, exhausting runtime resources. The limit is intentionally small
    // (50): once a stream is open its write pump runs independently, and the hot
    // path never blocks the drain regardless of how many opens are in flight.
    const MAX_CONCURRENT_STREAM_OPENS: usize = 50;
    let open_task_count = Arc::new(AtomicUsize::new(0));

    let incoming = control
        .clone()
        .accept()
        .map_err(|e| super::errors::ProtocolError::Logic(format!("failed to listen on protocol: {e}")))?;

    let stream_open_timeout = stream_cfg.stream_open_timeout;
    let frame_writer_backpressure_bytes = stream_cfg.frame_writer_backpressure_bytes;
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

                let (tx, rx) = flume::bounded::<<C as Decoder>::Item>(per_peer_channel_capacity);
                spawn_stream_pumps(
                    peer,
                    stream,
                    rx.clone(),
                    cache.clone(),
                    codec.clone(),
                    tx_in.clone(),
                    frame_writer_backpressure_bytes,
                );
                cache.insert(peer, PeerSink { tx, rx });

                futures::future::ready(())
            })
            .inspect(|_| {
                tracing::info!(
                    task = "ingress stream processing",
                    "long-running background task finished"
                )
            }),
    );

    // terminated when the rx_in is dropped
    //
    // The egress drain is fully non-blocking: `try_send` never awaits. On overflow
    // the oldest buffered packet is evicted (drop-oldest ring). `for_each` processes
    // items sequentially; since every iteration resolves immediately the loop runs at
    // the rate items arrive without holding any concurrency slots.
    //
    // On cache miss, `get_with` atomically creates the per-peer ring buffer and spawns
    // exactly one opener task. Concurrent misses for the same peer (while the opener is
    // in flight) share the same buffer. In-flight opens are bounded by `open_task_count`
    // (MAX_CONCURRENT_STREAM_OPENS = 50) to prevent resource exhaustion under dead-peer floods.
    let _egress_process = hopr_utils::runtime::prelude::spawn(
        rx_out
            .inspect(|(peer, _)| tracing::trace!(%peer, "proceeding to deliver message to peer"))
            .for_each(move |(peer, msg)| {
                tracing::trace!(%peer, "trying to deliver message to peer");

                let sink = if let Some(s) = cache_out.get(&peer) {
                    // Cache hit: cheapest path — one moka lookup, no clone on the hot path.
                    s
                } else {
                    // Cache miss: pre-clone before the `get_with` borrow so the init closure
                    // can capture independent copies without borrowing `cache_out` twice.
                    let cache2 = cache_out.clone();
                    let open_ctx2 = open_ctx.clone();
                    let open_count2 = open_task_count.clone();
                    cache_out.get_with(peer, move || {
                        let (tx, rx) = flume::bounded::<<C as Decoder>::Item>(per_peer_channel_capacity);
                        let rx_for_pump = rx.clone();

                        // Spawn one opener task per peer. `get_with` ensures this init closure
                        // runs at most once per key even under concurrent misses.
                        //
                        // Before spawning, acquire a slot from the in-flight counter. If the
                        // limit is reached the cache entry is immediately invalidated so the
                        // ring buffer's packets are dropped and the next send retries the open.
                        if open_count2.fetch_add(1, Ordering::Relaxed) < MAX_CONCURRENT_STREAM_OPENS {
                            hopr_utils::runtime::prelude::spawn(async move {
                                tracing::trace!(%peer, "peer is not in cache, opening new stream");
                                use futures_time::future::FutureExt as TimeExt;
                                let (control, codec, tx_in) = (&open_ctx2.0, &open_ctx2.1, &open_ctx2.2);

                                // A timeout is mandatory: a permanently-unreachable peer must not
                                // park the opener task indefinitely.
                                let stream = control
                                    .clone()
                                    .open(peer)
                                    .timeout(futures_time::time::Duration::from(stream_open_timeout))
                                    .await
                                    .map_err(|_| anyhow::anyhow!("timeout trying to open stream to {peer}"))
                                    .and_then(|s| {
                                        s.map_err(|e| anyhow::anyhow!("could not open outgoing peer-to-peer stream: {e}"))
                                    });

                                open_count2.fetch_sub(1, Ordering::Relaxed);

                                match stream {
                                    Ok(stream) => {
                                        tracing::debug!(%peer, "opening outgoing peer-to-peer stream");
                                        spawn_stream_pumps(
                                            peer,
                                            stream,
                                            rx_for_pump,
                                            cache2,
                                            codec.clone(),
                                            tx_in.clone(),
                                            frame_writer_backpressure_bytes,
                                        );
                                    }
                                    Err(error) => {
                                        tracing::debug!(
                                            %peer, %error,
                                            "stream open failed/timed out; dropping buffered packets"
                                        );
                                        cache2.invalidate(&peer);
                                    }
                                }
                            });
                        } else {
                            open_count2.fetch_sub(1, Ordering::Relaxed);
                            tracing::debug!(%peer, "stream-open concurrency limit reached; dropping buffered packets");
                            hopr_utils::runtime::prelude::spawn(async move { cache2.invalidate(&peer); });
                        }

                        PeerSink { tx, rx }
                    })
                };

                match sink.tx.try_send(msg) {
                    Ok(()) => tracing::trace!(%peer, "message queued to peer ring buffer"),
                    Err(flume::TrySendError::Full(msg)) => {
                        // Drop-oldest ring: evict the oldest buffered packet to make room,
                        // then enqueue the newest. The eviction is safe because the egress
                        // drain is the sole producer — no concurrent try_send racing here.
                        #[cfg(all(feature = "telemetry", not(test)))]
                        METRIC_RING_BUFFER_DROPPED.increment();
                        tracing::debug!(%peer, "per-peer ring buffer full; dropping oldest packet");
                        let _ = sink.rx.try_recv();
                        let _ = sink.tx.try_send(msg);
                    }
                    Err(flume::TrySendError::Disconnected(_)) => {
                        // The cache entry was invalidated concurrently (e.g. by the read pump
                        // detecting a dead connection). Invalidate again to ensure a clean
                        // state and let the next send re-open the stream.
                        tracing::debug!(%peer, "peer sink disconnected; invalidating cache");
                        cache_out.invalidate(&peer);
                    }
                }

                futures::future::ready(())
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
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        task::{Context as TaskContext, Poll, Waker},
    };

    use anyhow::Context;
    use async_trait::async_trait;
    use futures::{SinkExt, Stream};
    use parking_lot::Mutex;
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
        read_dead: std::sync::atomic::AtomicBool,
        write_dead: std::sync::atomic::AtomicBool,
        read_waker: Mutex<Option<Waker>>,
        write_waker: Mutex<Option<Waker>>,
    }

    impl std::fmt::Debug for DeadSignal {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("DeadSignal")
                .field("read_dead", &self.read_dead.load(Ordering::Relaxed))
                .field("write_dead", &self.write_dead.load(Ordering::Relaxed))
                .finish_non_exhaustive()
        }
    }

    impl DeadSignal {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                read_dead: std::sync::atomic::AtomicBool::new(false),
                write_dead: std::sync::atomic::AtomicBool::new(false),
                read_waker: Mutex::new(None),
                write_waker: Mutex::new(None),
            })
        }

        /// Simulate a dead read-half (e.g. NAT drop makes the peer's reader observe
        /// `ConnectionAborted` on the next poll). Wakes the read-side task only.
        fn kill_read(self: &Arc<Self>) {
            self.read_dead.store(true, Ordering::Release);
            // Extract the waker in a separate statement so the MutexGuard drops
            // before wake() runs — parking_lot mutexes are non-reentrant and wakers
            // may invoke arbitrary executor code.
            let waker = self.read_waker.lock().take();
            if let Some(w) = waker {
                w.wake();
            }
        }

        /// Simulate a dead write-half. Wakes the write-side task only.
        fn kill_write(self: &Arc<Self>) {
            self.write_dead.store(true, Ordering::Release);
            let waker = self.write_waker.lock().take();
            if let Some(w) = waker {
                w.wake();
            }
        }
    }

    struct FlaggedStream {
        signal: Arc<DeadSignal>,
    }

    impl AsyncRead for FlaggedStream {
        fn poll_read(self: Pin<&mut Self>, cx: &mut TaskContext<'_>, _buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
            // Store the waker BEFORE checking the flag: if kill() races between the
            // check and the store, it would consume a None waker and the task would
            // park forever. Storing first guarantees kill() always finds a waker to wake.
            *self.signal.read_waker.lock() = Some(cx.waker().clone());
            if self.signal.read_dead.load(Ordering::Acquire) {
                return Poll::Ready(Err(std::io::Error::from(std::io::ErrorKind::ConnectionAborted)));
            }
            Poll::Pending
        }
    }

    impl AsyncWrite for FlaggedStream {
        fn poll_write(self: Pin<&mut Self>, cx: &mut TaskContext<'_>, _buf: &[u8]) -> Poll<std::io::Result<usize>> {
            // Store the waker BEFORE checking the flag (same reasoning as poll_read).
            *self.signal.write_waker.lock() = Some(cx.waker().clone());
            if self.signal.write_dead.load(Ordering::Acquire) {
                return Poll::Ready(Err(std::io::Error::from(std::io::ErrorKind::ConnectionAborted)));
            }
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
            self.signals.lock().get(index).cloned()
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
            let signal = DeadSignal::new();
            // Publish the signal before bumping open_calls: tests wait for open_calls
            // to reach a threshold and then immediately fetch the signal, so the signal
            // must be visible before the counter is incremented.
            self.signals.lock().push(signal.clone());
            self.open_calls.fetch_add(1, Ordering::Relaxed);
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
                ..Default::default()
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

        // Kill the read half: simulates a libp2p ConnectionClosed event clearing the
        // liveness flag. The stored read waker is woken so the parked reader task
        // observes ConnectionAborted and calls cache.invalidate(peer).
        signal.kill_read();

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
                ..Default::default()
            },
        )
        .await?;

        let peer = PeerId::random();
        let msg = BytesMut::from(&b"payload"[..]);

        // Open a stream for the peer.
        tx_out.send((peer, msg.clone())).await.context("initial send")?;
        assert!(wait_for(2, || control.open_calls() >= 1).await, "stream not opened");

        let signal = control.signal(0).context("signal #1 must exist")?;

        // Kill the write half; the writer's forward-future will error on next poll.
        signal.kill_write();

        // Keep sending until the writer task observes the stream error and reopens,
        // capped at the per-peer channel capacity to bound the loop.
        let mut drained = 0usize;
        while control.open_calls() < 2 && drained < 128 {
            tx_out
                .send((peer, msg.clone()))
                .await
                .with_context(|| format!("drain send {drained} into egress queue should succeed"))?;
            drained += 1;
        }

        assert!(
            wait_for(3, || control.open_calls() >= 2).await,
            "stream was not reopened after writer kill (open_calls={})",
            control.open_calls()
        );

        Ok(())
    }

    /// Verifies that a full per-peer ring buffer does not invalidate the cache
    /// or trigger a pathological reopen.
    ///
    /// `CountingControl` returns a `StalledWriteIo` whose `poll_write` always
    /// returns `Pending`. The write pump stalls immediately after the first item;
    /// the ring buffer fills and the hot path applies drop-oldest overflow — but
    /// it must never call `cache.invalidate`, so the stream is never re-opened.
    #[tokio::test]
    async fn per_peer_stream_should_not_reopen_pathologically_on_send_failures() -> anyhow::Result<()> {
        let control = CountingControl::default();
        // Use a small channel so the 1 200 sends overflow it quickly, exercising
        // the drop-oldest ring path. On overflow the oldest packet is evicted and
        // the cached sender must not be invalidated — the stream must not reopen.
        let (mut tx_out, _rx_in) = process_stream_protocol(
            BytesCodec::new(),
            control.clone(),
            crate::config::StreamProtocolConfig {
                per_peer_channel_capacity: 16,
                ..Default::default()
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

    /// A control where the slow peer takes `open_delay` then fails, while all other
    /// peers open immediately with a working bidirectional channel.
    #[derive(Clone, Debug)]
    struct BimodalOpenControl {
        slow_peer: PeerId,
        open_delay: std::time::Duration,
        slow_open_calls: Arc<AtomicUsize>,
        fast_open_calls: Arc<AtomicUsize>,
    }

    impl BimodalOpenControl {
        #[allow(dead_code)]
        fn slow_open_calls(&self) -> usize {
            self.slow_open_calls.load(Ordering::Relaxed)
        }

        fn fast_open_calls(&self) -> usize {
            self.fast_open_calls.load(Ordering::Relaxed)
        }
    }

    #[async_trait]
    impl hopr_api::network::traits::NetworkStreamControl for BimodalOpenControl {
        fn accept(
            self,
        ) -> Result<impl Stream<Item = (PeerId, impl AsyncRead + AsyncWrite + Send)> + Send, impl std::error::Error>
        {
            Ok::<_, std::io::Error>(futures::stream::empty::<(PeerId, StalledWriteIo)>())
        }

        async fn open(self, peer: PeerId) -> Result<impl AsyncRead + AsyncWrite + Send, impl std::error::Error> {
            if peer == self.slow_peer {
                self.slow_open_calls.fetch_add(1, Ordering::Relaxed);
                tokio::time::sleep(self.open_delay).await;
                return Err::<AsyncBinaryStreamChannel, _>(std::io::Error::other("slow peer cannot connect"));
            }
            self.fast_open_calls.fetch_add(1, Ordering::Relaxed);
            Ok::<_, std::io::Error>(AsyncBinaryStreamChannel::new())
        }
    }

    /// Verifies that the egress drain does not head-of-line block cached (fast) peer
    /// sends behind slow opens to dead peers.
    ///
    /// With the old inline-open design, sending `max_concurrent_packets` packets to a
    /// slow peer would occupy all concurrency slots for the entire `stream_open_timeout`
    /// window. Any subsequent packet — even to a peer whose open is instant — could not
    /// start until a slot freed.
    ///
    /// With the detached-spawn design, the opener task runs independently of the drain
    /// loop, so a fast-peer open starts in parallel with the slow opener.
    #[tokio::test]
    async fn egress_should_not_hol_block_fast_peer_behind_slow_opens() -> anyhow::Result<()> {
        let slow_peer = PeerId::random();
        let fast_peer = PeerId::random();

        let control = BimodalOpenControl {
            slow_peer,
            // open_delay > stream_open_timeout → the pipeline times these opens out
            open_delay: std::time::Duration::from_millis(5_000),
            slow_open_calls: Default::default(),
            fast_open_calls: Default::default(),
        };

        let (mut tx_out, _rx_in) = process_stream_protocol(
            BytesCodec::new(),
            control.clone(),
            crate::config::StreamProtocolConfig {
                // Timeout shorter than open_delay so the pipeline gives up on slow_peer
                // well before our assertion deadline.
                stream_open_timeout: std::time::Duration::from_millis(2_000),
                ..Default::default()
            },
        )
        .await?;

        let msg = BytesMut::from(&b"x"[..]);

        // Enqueue several slow-peer packets; under the old inline design these would
        // hold the drain's concurrency slots for ~stream_open_timeout.
        for _ in 0..3 {
            tx_out
                .send((slow_peer, msg.clone()))
                .await
                .context("egress queue must accept slow-peer packet")?;
        }
        // Immediately enqueue a packet to the fast peer.
        tx_out
            .send((fast_peer, msg.clone()))
            .await
            .context("egress queue must accept fast-peer packet")?;

        // Assert that fast_peer's open was called within 1 s — well before the
        // 2 s stream_open_timeout would have freed a slot under the old design,
        // and generous enough to absorb CI scheduler jitter.
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(1_000);
        while control.fast_open_calls() < 1 && tokio::time::Instant::now() < deadline {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        assert!(
            control.fast_open_calls() >= 1,
            "fast peer's stream open was not called within 1 s — egress drain is likely head-of-line blocked by \
             slow-peer opens"
        );

        Ok(())
    }

    /// A `NetworkStreamControl` that delays `open()` by `delay` milliseconds before
    /// returning a working `AsyncBinaryStreamChannel` (loopback).
    #[derive(Clone, Debug)]
    struct DelayedControl {
        open_delay: std::time::Duration,
        open_calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl hopr_api::network::traits::NetworkStreamControl for DelayedControl {
        fn accept(
            self,
        ) -> Result<impl Stream<Item = (PeerId, impl AsyncRead + AsyncWrite + Send)> + Send, impl std::error::Error>
        {
            Ok::<_, std::io::Error>(futures::stream::empty::<(PeerId, AsyncBinaryStreamChannel)>())
        }

        async fn open(self, _peer: PeerId) -> Result<impl AsyncRead + AsyncWrite + Send, impl std::error::Error> {
            self.open_calls.fetch_add(1, Ordering::Relaxed);
            tokio::time::sleep(self.open_delay).await;
            Ok::<_, std::io::Error>(AsyncBinaryStreamChannel::new())
        }
    }

    /// Verifies that packets sent while the opener is in flight are buffered in
    /// the flume ring and all delivered once the stream opens — zero loss below
    /// ring capacity.
    ///
    /// This is the core regression test for the new buffering guarantee: the
    /// drop-oldest ring must not drop any packets when the burst is smaller than
    /// `per_peer_channel_capacity`.
    #[tokio::test]
    async fn egress_buffers_during_slow_open_then_drains() -> anyhow::Result<()> {
        let open_calls = Arc::new(AtomicUsize::new(0));
        let control = DelayedControl {
            open_delay: std::time::Duration::from_millis(100),
            open_calls: open_calls.clone(),
        };

        let (mut tx_out, mut rx_in) = process_stream_protocol(
            BytesCodec::new(),
            control,
            crate::config::StreamProtocolConfig {
                per_peer_channel_capacity: 64,
                ..Default::default()
            },
        )
        .await?;

        let peer = PeerId::random();
        let msg = BytesMut::from(&b"hello"[..]);

        // Send N < capacity packets while the opener is in flight.
        let n = 10usize;
        let expected_bytes = n * msg.len();
        for _ in 0..n {
            tx_out
                .send((peer, msg.clone()))
                .await
                .context("send into egress queue should succeed")?;
        }

        // Wait for the opener to complete.
        assert!(
            wait_for(2, || open_calls.load(Ordering::Relaxed) >= 1).await,
            "stream was never opened"
        );

        // All data should flow through the loopback channel and arrive on rx_in.
        // BytesCodec does not preserve packet boundaries so we assert on total bytes
        // received rather than item count — multiple sends may arrive as one read.
        let mut received_bytes = 0usize;
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(2);
        while received_bytes < expected_bytes && tokio::time::Instant::now() < deadline {
            match tokio::time::timeout(std::time::Duration::from_millis(100), rx_in.next()).await {
                Ok(Some((_, bytes))) => received_bytes += bytes.len(),
                _ => {}
            }
        }

        assert!(
            received_bytes >= expected_bytes,
            "expected at least {expected_bytes} bytes to be delivered after stream open; got {received_bytes}"
        );

        Ok(())
    }
}
