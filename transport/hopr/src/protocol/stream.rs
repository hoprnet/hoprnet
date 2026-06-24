//! Infrastructure supporting converting a collection of `PeerId` split `libp2p_stream` managed
//! individual peer-to-peer `libp2p::swarm::Stream`s.

use std::sync::Arc;

use async_lock::Semaphore;
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

type PeerStreamCache<T> = moka::sync::Cache<PeerId, Sender<T>>;
type PeerOpenLockCache = moka::sync::Cache<PeerId, Arc<async_lock::Mutex<()>>>;

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

/// Sends `msg` to `sender`, enforcing a per-send deadline.
///
/// On a send error the peer's cache entry is invalidated so the next packet
/// triggers a fresh stream open.
async fn deliver<T>(
    peer: libp2p::PeerId,
    sender: futures::channel::mpsc::Sender<T>,
    msg: T,
    timeout: std::time::Duration,
    cache: PeerStreamCache<T>,
) where
    T: Send + 'static,
{
    match sender.with_timeout(timeout).send(msg).await {
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
    }
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

    let max_concurrent_packets = stream_cfg.max_concurrent_packets;
    let stream_open_timeout = stream_cfg.stream_open_timeout;
    let frame_writer_backpressure_bytes = stream_cfg.frame_writer_backpressure_bytes;
    let per_peer_send_timeout = stream_cfg.per_peer_send_timeout;
    let per_peer_channel_capacity = stream_cfg.per_peer_channel_capacity;
    let max_concurrent_stream_opens = stream_cfg.max_concurrent_stream_opens;

    // Semaphore bounding the number of in-flight cache-miss open tasks. Without this,
    // a flood to many distinct unreachable peers could spawn an unbounded number of
    // 2 s open tasks, exhausting memory and sockets.
    let open_semaphore = Arc::new(Semaphore::new(max_concurrent_stream_opens));

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
                let open_semaphore = open_semaphore.clone();

                async move {
                    tracing::trace!(%peer, "trying to deliver message to peer");

                    if let Some(sender) = cache.get(&peer) {
                        // Cache hit: deliver inline. The concurrency slot is held only
                        // for the brief per_peer_send_timeout window.
                        deliver(peer, sender, msg, per_peer_send_timeout, cache).await;
                    } else {
                        // Cache miss: check the per-peer open lock *before* deciding
                        // whether to spawn or wait.
                        //
                        // - Opener  (try_lock succeeds): detach the stream open to a
                        //   spawned task so this concurrency slot is freed immediately.
                        //   A flood to many *distinct* dead peers can no longer
                        //   head-of-line block cached-peer sends.
                        //
                        // - Waiter  (try_lock fails, open already in progress): wait
                        //   inline on the lock — this blocks one concurrency slot but
                        //   ensures the packet is not dropped when the peer is healthy.
                        //   Bounded by stream_open_timeout so dead peers cannot park
                        //   a slot indefinitely.
                        let open_lock = open_locks.get_with(peer, || Arc::new(async_lock::Mutex::new(())));

                        match open_lock.try_lock_arc() {
                            Some(guard) => {
                                // Opener path: acquire a semaphore permit to bound the
                                // total number of concurrent in-flight open tasks, then
                                // detach. If all permits are taken the packet is dropped.
                                let permit = match open_semaphore.try_acquire_arc() {
                                    Some(p) => p,
                                    None => {
                                        tracing::debug!(%peer, "stream-open concurrency limit reached, dropping packet");
                                        return;
                                    }
                                };
                                hopr_utils::runtime::prelude::spawn(async move {
                                    let _permit = permit; // released when task completes
                                    let _guard = guard;   // released when task completes

                                    let sender = if let Some(cached) = cache.get(&peer) {
                                        Ok(cached)
                                    } else {
                                        tracing::trace!(%peer, "peer is not in cache, opening new stream");

                                        // Only the cache-miss path needs the stream-open handles.
                                        // Clone them out of the shared `open_ctx` once here rather
                                        // than on every outgoing message.
                                        let (control, codec, tx_in) = (&open_ctx.0, &open_ctx.1, &open_ctx.2);

                                        // A timeout is mandatory: a permanently-unreachable peer must
                                        // not park the per-peer open lock (and therefore the dedup queue)
                                        // indefinitely.
                                        use futures_time::future::FutureExt as TimeExt;
                                        let stream = control
                                            .clone()
                                            .open(peer)
                                            .timeout(futures_time::time::Duration::from(stream_open_timeout))
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
                                    };

                                    match sender {
                                        Ok(sender) => deliver(peer, sender, msg, per_peer_send_timeout, cache).await,
                                        Err(error) => {
                                            tracing::debug!(%peer, %error, "failed to open a stream to peer")
                                        }
                                    }
                                });
                            }
                            None => {
                                // Waiter path: an open for this peer is already in flight.
                                // Block this concurrency slot until the opener releases the
                                // lock (success or timeout), then deliver from cache.
                                use futures_time::future::FutureExt as TimeExt;
                                let timeout = futures_time::time::Duration::from(stream_open_timeout);
                                let _ = open_lock.lock().timeout(timeout).await;
                                if let Some(sender) = cache.get(&peer) {
                                    deliver(peer, sender, msg, per_peer_send_timeout, cache).await;
                                } else {
                                    tracing::debug!(%peer, "stream open failed or timed out, dropping packet");
                                }
                            }
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
    /// With the detached-spawn design, the drain slot is freed immediately after spawning
    /// the open task, so a fast-peer open starts in parallel with the slow opens.
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

        // max_concurrent_packets = 3: three slow-peer packets would exhaust all slots
        // under the old inline design.
        let (mut tx_out, _rx_in) = process_stream_protocol(
            BytesCodec::new(),
            control.clone(),
            crate::config::StreamProtocolConfig {
                max_concurrent_packets: 3,
                // Timeout shorter than open_delay so the pipeline gives up on slow_peer
                // well before our assertion deadline.
                stream_open_timeout: std::time::Duration::from_millis(2_000),
                per_peer_send_timeout: std::time::Duration::from_millis(50),
                ..Default::default()
            },
        )
        .await?;

        let msg = BytesMut::from(&b"x"[..]);

        // Fill all 3 slots with slow-peer packets. Under the old inline design these
        // would hold the slots for ~300 ms (the stream_open_timeout).
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
            "fast peer's stream open was not called within 150 ms — egress drain is likely head-of-line blocked by \
             slow-peer opens"
        );

        Ok(())
    }
}
