//! Infrastructure supporting converting a collection of `PeerId` split `libp2p_stream` managed
//! individual peer-to-peer `libp2p::swarm::Stream`s.

use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::{Context, Poll},
    time::Duration,
};

use crossfire::mpsc;
use futures::{
    AsyncRead, AsyncReadExt, AsyncWrite, FutureExt, Sink, StreamExt,
    channel::mpsc::{Receiver, Sender, channel},
};
use futures_timer::Delay;
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
            "Number of packets dropped due to per-peer egress channel overflow (drop-newest)",
        )
        .unwrap();
}

/// Per-peer egress buffer: a bounded MPSC crossfire channel.
///
/// Only the sender is stored in the cache entry; the receiver is owned exclusively by
/// the write pump spawned when the outgoing stream opens.  This is functionally SPSC
/// (one drain-loop producer, one write-pump consumer) even though the sender type is
/// Clone (required by `moka::sync::Cache`).
///
/// When the channel is full the drain loop drops the incoming packet (drop-newest)
/// and yields via `yield_now()` so the write pump has a chance to drain items and
/// free space before the next send attempt, preventing task starvation under high
/// producer rates.
///
/// `token` is a unique identity for this sink instance. Pump and opener tasks that
/// hold a clone compare it against the current cache entry before calling
/// `cache.invalidate`, so that a stale task finishing after the sink was replaced
/// (e.g. by an inbound stream arriving during an outgoing open) does not wipe the
/// newer entry.
#[derive(Clone)]
struct PeerSink<T: Send + 'static> {
    tx: crossfire::MAsyncTx<mpsc::Array<T>>,
    token: Arc<()>,
    /// `false` while the outgoing stream is still being opened, `true` once the write pump
    /// is draining the channel. The egress drain only applies blocking backpressure on a full
    /// channel when the stream is `ready`; while still opening it uses non-blocking drop-newest,
    /// so a slow open for one peer never head-of-line-blocks other peers.
    ready: Arc<AtomicBool>,
}

impl<T: Send + 'static> PeerSink<T> {
    fn new(tx: crossfire::MAsyncTx<mpsc::Array<T>>) -> Self {
        Self {
            tx,
            token: Arc::new(()),
            ready: Arc::new(AtomicBool::new(false)),
        }
    }
}

type PeerStreamCache<T> = moka::sync::Cache<PeerId, PeerSink<T>>;

/// Error produced by the per-peer write pump ([`StallGuardSink`]).
#[derive(Debug)]
enum EgressWriteError<E> {
    /// The wrapped framed-writer sink returned an error of its own.
    Sink(E),
    /// The wrapped sink made no forward progress within the stall timeout.
    Stalled { timeout: Duration },
}

impl<E: std::fmt::Display> std::fmt::Display for EgressWriteError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EgressWriteError::Sink(e) => write!(f, "egress sink error: {e}"),
            EgressWriteError::Stalled { timeout } => {
                write!(f, "egress write pump made no progress within {timeout:?}")
            }
        }
    }
}

/// A [`Sink`] adapter that fails the per-peer write pump once the inner sink stops making progress.
///
/// The write pump is `rx.forward(frame_writer)`, where `frame_writer` writes to the peer's quinn
/// substream. When a remote stops reading, quinn's `poll_write` parks (`Pending`) indefinitely
/// under flow control, so the framed writer's `poll_ready`/`poll_flush` never resolve and the pump
/// task parks forever — never completing, so the cache-eviction closure attached to it never runs.
/// The stalled peer's channel then stays full and the shared egress drain burns one backpressure
/// timeout per packet for that peer, head-of-line-blocking every other peer until the unbounded
/// mixer queue exhausts memory (the 2026-08-27 `jura-dev` OOM).
///
/// This adapter arms an idle deadline the first time the inner sink returns `Pending` on a
/// readiness/flush/close poll, and clears it the moment the inner sink returns `Ready`. If the
/// deadline elapses while the inner sink is still `Pending`, the poll resolves to
/// [`EgressWriteError::Stalled`], driving the pump to completion so its cache entry is evicted and
/// the next send reopens the stream. A merely-slow peer that keeps making progress within the
/// timeout is never killed — the deadline resets on every `Ready`.
struct StallGuardSink<S> {
    inner: S,
    timeout: Duration,
    /// Persistent idle timer, `reset` in place when a stall starts (the same persistent-`Delay`
    /// pattern as the mixer's `MixerSink`) so a busy-but-healthy peer that oscillates
    /// `Pending`/`Ready` does not churn a fresh `Delay` per flush cycle.
    timer: Delay,
    /// `true` while a readiness/flush/close poll is outstanding (inner last returned `Pending`).
    /// While set, the `timer` is polled without resetting, so a continuous stall is measured from
    /// its start rather than restarted on every poll.
    armed: bool,
}

impl<S> StallGuardSink<S> {
    fn new(inner: S, timeout: Duration) -> Self {
        Self {
            inner,
            timeout,
            timer: Delay::new(timeout),
            armed: false,
        }
    }

    /// Poll the inner sink, arming/clearing the idle timer and mapping a stall to an error.
    fn poll_guarded<T, E>(
        &mut self,
        cx: &mut Context<'_>,
        poll_inner: impl FnOnce(&mut S, &mut Context<'_>) -> Poll<Result<T, E>>,
    ) -> Poll<Result<T, EgressWriteError<E>>> {
        match poll_inner(&mut self.inner, cx) {
            Poll::Ready(Ok(v)) => {
                self.armed = false;
                Poll::Ready(Ok(v))
            }
            Poll::Ready(Err(e)) => {
                self.armed = false;
                Poll::Ready(Err(EgressWriteError::Sink(e)))
            }
            Poll::Pending => {
                if !self.armed {
                    self.timer.reset(self.timeout);
                    self.armed = true;
                }
                match self.timer.poll_unpin(cx) {
                    Poll::Ready(()) => {
                        self.armed = false;
                        Poll::Ready(Err(EgressWriteError::Stalled { timeout: self.timeout }))
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }
}

impl<S, T> Sink<T> for StallGuardSink<S>
where
    S: Sink<T> + Unpin,
{
    type Error = EgressWriteError<S::Error>;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut().poll_guarded(cx, |s, cx| Pin::new(s).poll_ready(cx))
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        Pin::new(&mut self.get_mut().inner)
            .start_send(item)
            .map_err(EgressWriteError::Sink)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut().poll_guarded(cx, |s, cx| Pin::new(s).poll_flush(cx))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut().poll_guarded(cx, |s, cx| Pin::new(s).poll_close(cx))
    }
}

/// Spawn the write and read pump tasks for an open peer stream.
///
/// The write pump drains `rx` into the framed stream writer; the read pump
/// forwards decoded frames to `ingress_from_peers`. Both tasks invalidate
/// `cache[peer]` when they complete, but only if the cache entry still holds
/// the same `token` — this prevents a stale task from wiping a newer sink.
#[allow(clippy::too_many_arguments)]
fn spawn_stream_pumps<S, C>(
    peer: PeerId,
    stream: S,
    rx: crossfire::AsyncRx<mpsc::Array<<C as Decoder>::Item>>,
    cache: PeerStreamCache<<C as Decoder>::Item>,
    token: Arc<()>,
    codec: C,
    ingress_from_peers: Sender<(PeerId, <C as Decoder>::Item)>,
    frame_writer_backpressure_bytes: usize,
    write_stall_timeout: Duration,
    ready: Arc<AtomicBool>,
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
    let token_write = token.clone();

    let mut frame_writer = FramedWrite::new(stream_tx.compat_write(), codec.clone());

    // `set_backpressure_boundary` is a *byte* threshold on `FramedWrite`'s internal
    // pending-write buffer: once the encoded frames exceed this many bytes, the next
    // `poll_ready` call will issue a flush. A larger value lets adjacent small frames
    // coalesce into a single quinn write, significantly reducing the number of
    // connection-mutex acquisitions and driver wake-ups on the hot path.
    frame_writer.set_backpressure_boundary(frame_writer_backpressure_bytes);

    // Guard the framed writer so a remote that stops reading (quinn `poll_write` parked forever)
    // fails the pump after `write_stall_timeout` instead of parking it indefinitely. On failure the
    // pump completes and the eviction closure below removes the cache entry, so the next send reopens.
    let frame_writer = StallGuardSink::new(frame_writer, write_stall_timeout);

    // Write pump: drain the per-peer channel into the framed stream writer.
    hopr_utils::runtime::prelude::spawn(
        futures::future::lazy(move |_| {
            // Flip `ready` only once this task is actually running and about to drain the
            // channel, so the egress drain never applies blocking backpressure on a peer whose
            // write pump has not started draining yet.
            ready.store(true, Ordering::Relaxed);
        })
        .then(move |_| rx.into_stream().map(Ok).forward(frame_writer))
        .inspect(move |res| {
            tracing::debug!(%peer, ?res, component = "stream", "writing stream with peer finished");
        })
        .then(move |_| async move {
            if cache_for_write
                .get(&peer)
                .is_some_and(|s| Arc::ptr_eq(&s.token, &token_write))
            {
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
            .then(move |_| async move {
                if cache_for_read.get(&peer).is_some_and(|s| Arc::ptr_eq(&s.token, &token)) {
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
    // `Unpin` is required so the egress drain can `await` a crossfire `send` (backpressure path).
    <C as Decoder>::Item: AsRef<[u8]> + Clone + Send + Unpin + 'static,
    V: NetworkStreamControl + Clone + Send + Sync + 'static,
{
    let (tx_out, mut rx_out) = channel::<(PeerId, <C as Decoder>::Item)>(100_000);
    let (tx_in, rx_in) = channel::<(PeerId, <C as Decoder>::Item)>(100_000);

    let cache_out: PeerStreamCache<<C as Decoder>::Item> = moka::sync::Cache::builder()
        .max_capacity(2000)
        .eviction_listener(|key: Arc<PeerId>, _, cause| {
            tracing::trace!(peer = %key.as_ref(), ?cause, "evicting stream for peer");
        })
        .build();

    // Bounds the number of in-flight stream-open tasks across all distinct peers.
    const MAX_CONCURRENT_STREAM_OPENS: usize = 50;
    let open_task_count = Arc::new(AtomicUsize::new(0));

    let incoming = control
        .clone()
        .accept()
        .map_err(|e| super::errors::ProtocolError::Logic(format!("failed to listen on protocol: {e}")))?;

    let stream_open_timeout = stream_cfg.stream_open_timeout;
    let frame_writer_backpressure_bytes = stream_cfg.frame_writer_backpressure_bytes;
    let per_peer_channel_capacity = stream_cfg.per_peer_channel_capacity;
    let egress_backpressure_timeout = stream_cfg.egress_backpressure_timeout;

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

                let (tx, rx) = mpsc::bounded_async::<<C as Decoder>::Item>(per_peer_channel_capacity);
                let sink = PeerSink::new(tx);
                let token = sink.token.clone();
                let ready = sink.ready.clone();
                spawn_stream_pumps(
                    peer,
                    stream,
                    rx,
                    cache.clone(),
                    token,
                    codec.clone(),
                    tx_in.clone(),
                    frame_writer_backpressure_bytes,
                    egress_backpressure_timeout,
                    ready,
                );
                cache.insert(peer, sink);

                futures::future::ready(())
            })
            .inspect(|_| {
                tracing::info!(
                    task = "ingress stream processing",
                    "long-running background task finished"
                )
            }),
    );

    // Egress drain: reads outgoing packets from `rx_out` and enqueues them into
    // the per-peer crossfire channel.
    //
    // The drain is non-blocking: `try_send` never awaits. On overflow the incoming
    // packet is dropped (drop-newest) and `yield_now()` is called to let the write
    // pump task drain the channel before the next send attempt, preventing task
    // starvation under high producer rates.
    //
    // On cache miss, `get_with` atomically creates the per-peer channel and spawns
    // exactly one opener task. In-flight opens are bounded by `open_task_count`
    // (MAX_CONCURRENT_STREAM_OPENS = 50) to prevent resource exhaustion.
    let _egress_process = hopr_utils::runtime::prelude::spawn(async move {
        use futures::StreamExt as _;

        while let Some((peer, msg)) = rx_out.next().await {
            tracing::trace!(%peer, "trying to deliver message to peer");

            let sink = if let Some(s) = cache_out.get(&peer) {
                s
            } else {
                let cache2 = cache_out.clone();
                let open_ctx2 = open_ctx.clone();
                let open_count2 = open_task_count.clone();
                cache_out.get_with(peer, move || {
                    let (tx, rx) = mpsc::bounded_async::<<C as Decoder>::Item>(per_peer_channel_capacity);
                    let sink = PeerSink::new(tx);
                    let token = sink.token.clone();
                    let ready = sink.ready.clone();

                    if open_count2.fetch_add(1, Ordering::Relaxed) < MAX_CONCURRENT_STREAM_OPENS {
                        hopr_utils::runtime::prelude::spawn(async move {
                            tracing::trace!(%peer, "peer is not in cache, opening new stream");
                            use futures_time::future::FutureExt as TimeExt;
                            let (control, codec, tx_in) = (&open_ctx2.0, &open_ctx2.1, &open_ctx2.2);

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
                                        rx,
                                        cache2.clone(),
                                        token,
                                        codec.clone(),
                                        tx_in.clone(),
                                        frame_writer_backpressure_bytes,
                                        egress_backpressure_timeout,
                                        ready,
                                    );
                                }
                                Err(error) => {
                                    tracing::debug!(
                                        %peer, %error,
                                        "stream open failed/timed out; dropping buffered packets"
                                    );
                                    if cache2.get(&peer).is_some_and(|s| Arc::ptr_eq(&s.token, &token)) {
                                        cache2.invalidate(&peer);
                                    }
                                }
                            }
                        });
                    } else {
                        open_count2.fetch_sub(1, Ordering::Relaxed);
                        tracing::debug!(%peer, "stream-open concurrency limit reached; dropping buffered packets");
                        hopr_utils::runtime::prelude::spawn(async move {
                            if cache2.get(&peer).is_some_and(|s| Arc::ptr_eq(&s.token, &token)) {
                                cache2.invalidate(&peer);
                            }
                        });
                    }

                    sink
                })
            };

            match sink.tx.try_send(msg) {
                Ok(()) => tracing::trace!(%peer, "message queued to peer channel"),
                Err(crossfire::TrySendError::Full(msg)) => {
                    if sink.ready.load(Ordering::Relaxed) {
                        // Stream is open and draining; a full channel means the wire is slower
                        // than the producer. Wait (bounded) for space so wire-rate backpressure
                        // propagates upstream — through the mixer, the outgoing CrossfireSink and
                        // the session socket — to the application writer, instead of dropping.
                        // Fall back to drop-newest only if the peer stays full past the timeout,
                        // so a stalled peer cannot head-of-line-block other peers forever.
                        use futures_time::future::FutureExt as _;
                        match async { sink.tx.send(msg).await }
                            .timeout(futures_time::time::Duration::from(egress_backpressure_timeout))
                            .await
                        {
                            Ok(Ok(())) => {
                                tracing::trace!(%peer, "message queued to peer channel after backpressure")
                            }
                            Ok(Err(_disconnected)) => {
                                tracing::debug!(%peer, "peer sink disconnected while awaiting space; invalidating cache");
                                if cache_out.get(&peer).is_some_and(|s| Arc::ptr_eq(&s.token, &sink.token)) {
                                    cache_out.invalidate(&peer);
                                }
                            }
                            Err(_timeout) => {
                                #[cfg(all(feature = "telemetry", not(test)))]
                                METRIC_RING_BUFFER_DROPPED.increment();
                                tracing::debug!(
                                    %peer,
                                    "per-peer egress channel full past backpressure timeout; dropping newest packet"
                                );
                            }
                        }
                    } else {
                        // Stream still opening: never block — a slow open for this peer must not
                        // head-of-line-block delivery to other peers. Drop newest and yield so the
                        // opener task can make progress.
                        #[cfg(all(feature = "telemetry", not(test)))]
                        METRIC_RING_BUFFER_DROPPED.increment();
                        tracing::debug!(%peer, "per-peer egress channel full during open; dropping newest packet");
                        hopr_utils::runtime::prelude::yield_now().await;
                    }
                }
                Err(crossfire::TrySendError::Disconnected(_)) => {
                    // Receiver dropped (write pump died): invalidate and reopen on next send.
                    // Guard with the token so we don't wipe a replacement sink that was
                    // inserted (e.g. from an inbound stream) between our cache.get() above
                    // and this branch.
                    tracing::debug!(%peer, "peer sink disconnected; invalidating cache");
                    if cache_out.get(&peer).is_some_and(|s| Arc::ptr_eq(&s.token, &sink.token)) {
                        cache_out.invalidate(&peer);
                    }
                }
            }
        }

        tracing::info!(
            task = "egress stream processing",
            "long-running background task finished"
        );
    });

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
            Poll::Ready(Ok(()))
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

        fn kill_read(self: &Arc<Self>) {
            self.read_dead.store(true, Ordering::Release);
            let waker = self.read_waker.lock().take();
            if let Some(w) = waker {
                w.wake();
            }
        }

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
            *self.signal.read_waker.lock() = Some(cx.waker().clone());
            if self.signal.read_dead.load(Ordering::Acquire) {
                return Poll::Ready(Err(std::io::Error::from(std::io::ErrorKind::ConnectionAborted)));
            }
            Poll::Pending
        }
    }

    impl AsyncWrite for FlaggedStream {
        fn poll_write(self: Pin<&mut Self>, cx: &mut TaskContext<'_>, _buf: &[u8]) -> Poll<std::io::Result<usize>> {
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

    #[derive(Clone, Debug, Default)]
    struct ScriptedControl {
        open_calls: Arc<AtomicUsize>,
        signals: Arc<Mutex<Vec<Arc<DeadSignal>>>>,
    }

    impl ScriptedControl {
        fn open_calls(&self) -> usize {
            self.open_calls.load(Ordering::Relaxed)
        }

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
            self.signals.lock().push(signal.clone());
            self.open_calls.fetch_add(1, Ordering::Relaxed);
            Ok::<_, std::io::Error>(FlaggedStream { signal })
        }
    }

    async fn wait_for(secs: u64, condition: impl Fn() -> bool) -> bool {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(secs);
        while !condition() && tokio::time::Instant::now() < deadline {
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
        condition()
    }

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

        tx_out
            .send((peer, msg.clone()))
            .await
            .context("first send should succeed")?;

        assert!(
            wait_for(2, || control.open_calls() >= 1).await,
            "stream was never opened"
        );
        let signal = control.signal(0).context("signal for stream #1 must exist")?;

        signal.kill_read();

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

        tx_out.send((peer, msg.clone())).await.context("initial send")?;
        assert!(wait_for(2, || control.open_calls() >= 1).await, "stream not opened");

        let signal = control.signal(0).context("signal #1 must exist")?;

        signal.kill_write();

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

    /// Reopening a permanently stalled peer must self-heal at the stall-timeout cadence — never as
    /// tight per-send churn.
    ///
    /// `CountingControl` returns a `StalledWriteIo` whose `poll_write`/`poll_flush` always return
    /// `Pending`. Before the sink-level stall mitigation the write pump parked forever and the entry
    /// was never evicted (the previous version of this test asserted "must not reopen"). It now
    /// terminates after the stall timeout so the entry is evicted and the next send reopens — but
    /// the reopen cadence must stay bounded by the stall timeout, not spin once per send.
    #[tokio::test]
    async fn stalled_peer_reopen_cadence_should_be_bounded_by_the_stall_timeout() -> anyhow::Result<()> {
        const STALL_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(300);
        const WINDOW: std::time::Duration = std::time::Duration::from_millis(1_500);

        let control = CountingControl::default();
        let (mut tx_out, _rx_in) = process_stream_protocol(
            BytesCodec::new(),
            control.clone(),
            crate::config::StreamProtocolConfig {
                per_peer_channel_capacity: 4,
                frame_writer_backpressure_bytes: 1,
                egress_backpressure_timeout: STALL_TIMEOUT,
                ..Default::default()
            },
        )
        .await?;

        let peer = PeerId::random();
        let msg = BytesMut::from(&b"x"[..]);

        // Keep offering packets throughout the window so every eviction is promptly observable as a
        // fresh open.
        let deadline = tokio::time::Instant::now() + WINDOW;
        while tokio::time::Instant::now() < deadline {
            let _ = tx_out.send((peer, msg.clone())).await;
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }

        let opens = control.open_calls();
        assert!(
            opens >= 2,
            "a permanently stalled peer must self-heal by reopening (open_calls={opens})"
        );

        // Cadence bound: at most one reopen per stall timeout, plus generous slack for scheduling.
        // A regression to per-send reopen churn (dozens/hundreds of opens) trips this.
        let max_expected = (WINDOW.as_millis() / STALL_TIMEOUT.as_millis()) as usize + 3;
        assert!(
            opens <= max_expected,
            "reopen churn detected: {opens} opens in {WINDOW:?} exceeds the ~{max_expected} bounded by the \
             {STALL_TIMEOUT:?} stall timeout"
        );

        Ok(())
    }

    /// Reproduces the 2026-08-27 `jura-dev` OOM at the integration boundary.
    ///
    /// A peer whose remote stops reading makes the underlying `poll_write` park (`Pending`)
    /// forever under flow control. The per-peer write pump (`rx.forward(frame_writer)`) then
    /// parks with it and never completes, so its cache-eviction closure never runs: the sink is
    /// never evicted, the peer's channel stays full, and the shared egress drain burns one
    /// backpressure timeout per packet for that peer — head-of-line-blocking every other peer
    /// until the unbounded mixer queue exhausts memory.
    ///
    /// The fix mitigates the stall *at the sink*: the write pump must fail once it makes no
    /// progress for `egress_backpressure_timeout`, driving the pump to completion so the entry is
    /// evicted and a later send reopens the stream. A merely-slow peer that keeps making progress
    /// within the timeout is never killed.
    ///
    /// Under the unfixed code the pump parks forever, the entry is never evicted, and
    /// `open_calls` stays pinned at 1 — this test fails.
    #[tokio::test]
    async fn stalled_write_pump_should_terminate_and_reopen_after_stall_timeout() -> anyhow::Result<()> {
        const STALL_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(500);

        let control = CountingControl::default();
        let (mut tx_out, _rx_in) = process_stream_protocol(
            BytesCodec::new(),
            control.clone(),
            crate::config::StreamProtocolConfig {
                per_peer_channel_capacity: 4,
                // Flush every frame so the pump reaches the stalled writer immediately, rather than
                // buffering frames inside `FramedWrite` until the byte threshold forces a flush.
                frame_writer_backpressure_bytes: 1,
                // Reused as the write-pump stall timeout.
                egress_backpressure_timeout: STALL_TIMEOUT,
                ..Default::default()
            },
        )
        .await?;

        let peer = PeerId::random();
        let msg = BytesMut::from(&b"payload"[..]);

        // First send opens the stream; its writer is permanently stalled (poll_write == Pending).
        tx_out.send((peer, msg.clone())).await.context("first send")?;
        assert!(
            wait_for(2, || control.open_calls() >= 1).await,
            "stream was never opened"
        );

        // A merely-slow peer must not be evicted: no reopen before the stall timeout elapses.
        tokio::time::sleep(STALL_TIMEOUT / 2).await;
        assert_eq!(
            control.open_calls(),
            1,
            "stream reopened before the stall timeout elapsed — over-eager eviction would kill merely-slow peers"
        );

        // Past the stall timeout the pump must terminate, evict the cache entry, and a subsequent
        // send must reopen the stream. Keep sending so an eviction is observable as a fresh open.
        let mut sends = 0;
        while control.open_calls() < 2 && sends < 200 {
            let _ = tx_out.send((peer, msg.clone())).await;
            sends += 1;
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }

        assert!(
            wait_for(3, || control.open_calls() >= 2).await,
            "stalled write pump never terminated: the stream was not reopened after the stall timeout (open_calls={}) \
             — the pump parked forever and the peer became a permanent silent black hole, exactly the field failure \
             mode",
            control.open_calls(),
        );

        Ok(())
    }

    // -----------------------------------------------------------------------
    // StallGuardSink unit tests — isolate the sink-level stall mitigation from
    // the surrounding stream-protocol machinery. Each covers one way the wrapped
    // sink can stall (or make progress) and asserts the adapter's response.
    // -----------------------------------------------------------------------

    /// Outcome a [`ScriptedSink`] returns for a given poll.
    #[derive(Clone, Copy)]
    enum Op {
        Ready,
        Pending,
        Err,
    }

    fn apply(op: Op) -> Poll<Result<(), std::io::Error>> {
        match op {
            Op::Ready => Poll::Ready(Ok(())),
            Op::Pending => Poll::Pending,
            Op::Err => Poll::Ready(Err(std::io::Error::other("scripted sink error"))),
        }
    }

    /// A `Sink` whose `poll_ready`/`poll_flush`/`poll_close` outcomes are fixed per-op. A `Pending`
    /// op never registers a waker (modelling a quinn substream whose remote stopped reading) — the
    /// wake-up must come from `StallGuardSink`'s own timer.
    struct ScriptedSink {
        ready: Op,
        flush: Op,
        close: Op,
    }

    impl Sink<u8> for ScriptedSink {
        type Error = std::io::Error;

        fn poll_ready(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
            apply(self.ready)
        }

        fn start_send(self: Pin<&mut Self>, _item: u8) -> Result<(), Self::Error> {
            Ok(())
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
            apply(self.flush)
        }

        fn poll_close(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
            apply(self.close)
        }
    }

    /// A `Sink` whose first `poll_flush` parks once (waking itself so it is re-polled immediately),
    /// then succeeds — a peer that briefly stalls but recovers well within the timeout.
    #[derive(Default)]
    struct TransientFlushSink {
        flushed_once: bool,
    }

    impl Sink<u8> for TransientFlushSink {
        type Error = std::io::Error;

        fn poll_ready(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(self: Pin<&mut Self>, _item: u8) -> Result<(), Self::Error> {
            Ok(())
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
            let this = self.get_mut();
            if this.flushed_once {
                Poll::Ready(Ok(()))
            } else {
                this.flushed_once = true;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }

        fn poll_close(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn stall_guard_sink_should_error_when_poll_ready_stalls_past_timeout() {
        let mut sink = StallGuardSink::new(
            ScriptedSink {
                ready: Op::Pending,
                flush: Op::Ready,
                close: Op::Ready,
            },
            Duration::from_millis(150),
        );
        let res = tokio::time::timeout(Duration::from_secs(2), sink.send(1u8))
            .await
            .expect("StallGuardSink must resolve on its own timer, not hang");
        assert!(
            matches!(res, Err(EgressWriteError::Stalled { .. })),
            "a sink that never becomes ready must fail with Stalled, got {res:?}"
        );
    }

    #[tokio::test]
    async fn stall_guard_sink_should_error_when_poll_flush_stalls_past_timeout() {
        let mut sink = StallGuardSink::new(
            ScriptedSink {
                ready: Op::Ready,
                flush: Op::Pending,
                close: Op::Ready,
            },
            Duration::from_millis(150),
        );
        let res = tokio::time::timeout(Duration::from_secs(2), sink.send(1u8))
            .await
            .expect("StallGuardSink must resolve on its own timer, not hang");
        assert!(
            matches!(res, Err(EgressWriteError::Stalled { .. })),
            "a sink that accepts but never flushes must fail with Stalled, got {res:?}"
        );
    }

    #[tokio::test]
    async fn stall_guard_sink_should_error_when_poll_close_stalls_past_timeout() {
        let mut sink = StallGuardSink::new(
            ScriptedSink {
                ready: Op::Ready,
                flush: Op::Ready,
                close: Op::Pending,
            },
            Duration::from_millis(150),
        );
        let res = tokio::time::timeout(Duration::from_secs(2), sink.close())
            .await
            .expect("StallGuardSink must resolve on its own timer, not hang");
        assert!(
            matches!(res, Err(EgressWriteError::Stalled { .. })),
            "a sink that never closes must fail with Stalled, got {res:?}"
        );
    }

    #[tokio::test]
    async fn stall_guard_sink_should_pass_through_a_healthy_sink_without_error() {
        let mut sink = StallGuardSink::new(
            ScriptedSink {
                ready: Op::Ready,
                flush: Op::Ready,
                close: Op::Ready,
            },
            Duration::from_millis(50),
        );
        for i in 0..100u8 {
            sink.send(i)
                .await
                .expect("a healthy sink must never be failed by the stall guard");
        }
        sink.close().await.expect("closing a healthy sink must succeed");
    }

    #[tokio::test]
    async fn stall_guard_sink_should_surface_inner_errors_verbatim_not_as_a_stall() {
        let mut sink = StallGuardSink::new(
            ScriptedSink {
                ready: Op::Ready,
                flush: Op::Err,
                close: Op::Ready,
            },
            Duration::from_millis(150),
        );
        let res = sink.send(1u8).await;
        assert!(
            matches!(res, Err(EgressWriteError::Sink(_))),
            "an inner sink error must surface as Sink(_), not be masked as Stalled, got {res:?}"
        );
    }

    #[tokio::test]
    async fn stall_guard_sink_should_not_error_on_a_transient_stall_that_recovers_in_time() {
        // Timeout far larger than the transient stall: the deadline is armed on the first (parked)
        // flush poll and must be cleared when the sink makes progress, so no Stalled error fires.
        let mut sink = StallGuardSink::new(TransientFlushSink::default(), Duration::from_secs(10));
        tokio::time::timeout(Duration::from_secs(2), sink.send(1u8))
            .await
            .expect("a sink that recovers before the timeout must not hang")
            .expect("a sink that recovers before the timeout must not be failed as Stalled");
    }

    /// The stalled peer's stream opens exactly once (then its writer is gated shut by the test); any
    /// reopen fails. Every other peer opens a writer that accepts immediately.
    #[derive(Clone, Debug)]
    struct HeadOfLineControl {
        stalled_peer: PeerId,
        stalled_io: GatedWriteIo,
        healthy_io: GatedWriteIo,
        open_calls: Arc<AtomicUsize>,
        stalled_opens: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl hopr_api::network::traits::NetworkStreamControl for HeadOfLineControl {
        fn accept(
            self,
        ) -> Result<impl Stream<Item = (PeerId, impl AsyncRead + AsyncWrite + Send)> + Send, impl std::error::Error>
        {
            Ok::<_, std::io::Error>(futures::stream::empty::<(PeerId, GatedWriteIo)>())
        }

        async fn open(self, peer: PeerId) -> Result<impl AsyncRead + AsyncWrite + Send, impl std::error::Error> {
            self.open_calls.fetch_add(1, Ordering::Relaxed);
            if peer == self.stalled_peer {
                // Only the first open establishes the stream. A permanently-shut peer that could
                // reopen would re-stall for another full timeout each cycle, making the burst take an
                // unbounded number of ~timeout cycles; failing the reopen bounds the fix's side to a
                // single eviction cycle, while the unfixed pump never evicts and so never reopens.
                if self.stalled_opens.fetch_add(1, Ordering::Relaxed) == 0 {
                    Ok::<GatedWriteIo, std::io::Error>(self.stalled_io.clone())
                } else {
                    Err(std::io::Error::other("stalled peer refuses reopen"))
                }
            } else {
                Ok(self.healthy_io.clone())
            }
        }
    }

    /// End-to-end proof that the sink-level stall mitigation clears the head-of-line block that
    /// caused the 2026-08-27 `jura-dev` OOM: a peer whose write pump parks must not hold the shared
    /// drain loop hostage while a healthy peer waits behind it.
    ///
    /// The healthy packet is queued *after* the stalled burst, so it only arrives promptly if the
    /// stalled peer's sink is evicted and the loop stops serialising on it. Under the unfixed code
    /// the burst holds the loop for one backpressure timeout per packet forever, and the healthy
    /// peer is starved.
    #[tokio::test]
    async fn stalled_peer_must_not_head_of_line_block_a_healthy_peer() -> anyhow::Result<()> {
        const STALL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);
        const OVERFLOW: usize = 8;

        let stalled_peer = PeerId::random();
        let healthy_peer = PeerId::random();

        // Both writers start open; the stalled peer is gated shut only after its stream is
        // established (below), modelling a remote that stops reading mid-stream.
        let stalled_io = GatedWriteIo {
            open: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            ..Default::default()
        };
        let healthy_io = GatedWriteIo {
            open: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            ..Default::default()
        };

        let open_calls = Arc::new(AtomicUsize::new(0));
        let control = HeadOfLineControl {
            stalled_peer,
            stalled_io: stalled_io.clone(),
            healthy_io: healthy_io.clone(),
            open_calls: open_calls.clone(),
            stalled_opens: Arc::new(AtomicUsize::new(0)),
        };

        let (mut tx_out, _rx_in) = process_stream_protocol(
            BytesCodec::new(),
            control,
            crate::config::StreamProtocolConfig {
                per_peer_channel_capacity: 2,
                // Flush every frame so the channel fills, not the `FramedWrite` buffer.
                frame_writer_backpressure_bytes: 1,
                egress_backpressure_timeout: STALL_TIMEOUT,
                ..Default::default()
            },
        )
        .await?;

        let msg = BytesMut::from(&b"hello"[..]);

        // Establish the ready precondition race-free: prime the stalled peer *while it is still
        // open* and wait until the byte is actually written to the wire. That positively proves the
        // stream opened and its write pump drained (`ready == true`), with no stall or eviction in
        // play yet — unlike waiting on a stalled poll, which could race the pump's own stall timeout.
        tx_out
            .send((stalled_peer, msg.clone()))
            .await
            .context("egress queue must accept priming stalled-peer packet")?;
        assert!(
            wait_for(2, || stalled_io.written() >= msg.len()).await,
            "priming packet was never written — stalled peer's stream/pump did not establish"
        );
        assert_eq!(
            open_calls.load(Ordering::Relaxed),
            1,
            "exactly one open expected during priming"
        );

        // Now the remote stops reading: gate the writer shut so subsequent writes park. Only from
        // here does the stall (and its eviction timer) begin.
        stalled_io.gate_shut();

        // Overflow the ready+full stalled channel. Under the old code each excess packet costs the
        // shared drain loop a full backpressure timeout, starving the healthy peer behind it; the fix
        // evicts the parked pump after one timeout so the loop moves on.
        for _ in 0..OVERFLOW {
            tx_out
                .send((stalled_peer, msg.clone()))
                .await
                .context("egress queue must accept stalled-peer packet")?;
        }

        tx_out
            .send((healthy_peer, msg.clone()))
            .await
            .context("egress queue must accept healthy-peer packet")?;

        let delivered = wait_for(3, || healthy_io.written() >= msg.len()).await;

        assert_eq!(
            stalled_io.written(),
            msg.len(),
            "only the priming packet should reach the stalled writer; the post-shut burst must not"
        );

        assert!(
            delivered,
            "a single stalled peer head-of-line-blocked the shared egress drain: the healthy peer got {} of {} bytes \
             within 3s, while {OVERFLOW} overflow packets to the stalled peer each held the loop for {STALL_TIMEOUT:?}",
            healthy_io.written(),
            msg.len(),
        );

        Ok(())
    }

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

    #[tokio::test]
    async fn egress_should_not_hol_block_fast_peer_behind_slow_opens() -> anyhow::Result<()> {
        let slow_peer = PeerId::random();
        let fast_peer = PeerId::random();

        let control = BimodalOpenControl {
            slow_peer,
            open_delay: std::time::Duration::from_millis(5_000),
            slow_open_calls: Default::default(),
            fast_open_calls: Default::default(),
        };

        let (mut tx_out, _rx_in) = process_stream_protocol(
            BytesCodec::new(),
            control.clone(),
            crate::config::StreamProtocolConfig {
                stream_open_timeout: std::time::Duration::from_millis(2_000),
                ..Default::default()
            },
        )
        .await?;

        let msg = BytesMut::from(&b"x"[..]);

        for _ in 0..3 {
            tx_out
                .send((slow_peer, msg.clone()))
                .await
                .context("egress queue must accept slow-peer packet")?;
        }
        tx_out
            .send((fast_peer, msg.clone()))
            .await
            .context("egress queue must accept fast-peer packet")?;

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

    /// Verifies that packets sent while the opener is in flight are buffered and
    /// all delivered once the stream opens — zero loss below channel capacity.
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

        let n = 10usize;
        let expected_bytes = n * msg.len();
        for _ in 0..n {
            tx_out
                .send((peer, msg.clone()))
                .await
                .context("send into egress queue should succeed")?;
        }

        assert!(
            wait_for(2, || open_calls.load(Ordering::Relaxed) >= 1).await,
            "stream was never opened"
        );

        let mut received_bytes = 0usize;
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(2);
        while received_bytes < expected_bytes && tokio::time::Instant::now() < deadline {
            if let Ok(Some((_, bytes))) =
                tokio::time::timeout(std::time::Duration::from_millis(100), rx_in.next()).await
            {
                received_bytes += bytes.len();
            }
        }

        assert!(
            received_bytes >= expected_bytes,
            "expected at least {expected_bytes} bytes to be delivered after stream open; got {received_bytes}"
        );

        Ok(())
    }

    /// A writer whose `poll_write` accepts bytes while `open` and parks (`Pending`) while shut,
    /// counting the bytes it accepts. Gating it shut after it has drained a packet models a remote
    /// that stops reading only once the stream is established, so a burst can be forced onto the
    /// ready+full egress path with no dependence on scheduler/pipe-buffer timing.
    #[derive(Clone, Default, Debug)]
    struct GatedWriteIo {
        open: Arc<std::sync::atomic::AtomicBool>,
        written: Arc<AtomicUsize>,
        waker: Arc<Mutex<Option<Waker>>>,
    }

    impl GatedWriteIo {
        fn release(&self) {
            self.open.store(true, Ordering::Relaxed);
            if let Some(waker) = self.waker.lock().take() {
                waker.wake();
            }
        }

        /// Gate the writer shut so subsequent `poll_write`s park — models the remote ceasing to read.
        fn gate_shut(&self) {
            self.open.store(false, Ordering::Relaxed);
        }

        fn written(&self) -> usize {
            self.written.load(Ordering::Relaxed)
        }
    }

    impl AsyncRead for GatedWriteIo {
        fn poll_read(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>, _buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
            Poll::Pending
        }
    }

    impl AsyncWrite for GatedWriteIo {
        fn poll_write(self: Pin<&mut Self>, cx: &mut TaskContext<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
            if self.open.load(Ordering::Relaxed) {
                self.written.fetch_add(buf.len(), Ordering::Relaxed);
                Poll::Ready(Ok(buf.len()))
            } else {
                *self.waker.lock() = Some(cx.waker().clone());
                Poll::Pending
            }
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    #[derive(Clone, Debug)]
    struct GatedControl {
        io: GatedWriteIo,
        open_calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl hopr_api::network::traits::NetworkStreamControl for GatedControl {
        fn accept(
            self,
        ) -> Result<impl Stream<Item = (PeerId, impl AsyncRead + AsyncWrite + Send)> + Send, impl std::error::Error>
        {
            Ok::<_, std::io::Error>(futures::stream::empty::<(PeerId, GatedWriteIo)>())
        }

        async fn open(self, _peer: PeerId) -> Result<impl AsyncRead + AsyncWrite + Send, impl std::error::Error> {
            self.open_calls.fetch_add(1, Ordering::Relaxed);
            Ok::<_, std::io::Error>(self.io.clone())
        }
    }

    /// Regression guard for the egress backpressure fix.
    ///
    /// Once the stream is open and draining, a burst that overflows the per-peer channel must apply
    /// (bounded) backpressure so the producer is slowed to wire rate — no packet loss. Under the old
    /// `try_send` + drop-newest behavior, every packet that arrived while the channel was full was
    /// silently dropped.
    ///
    /// The [`GatedWriteIo`] writer is held shut so the per-peer channel is deterministically full while
    /// the stream is open (`ready == true`) — the exact ready+full path — regardless of scheduler
    /// timing. The writer is released *before* the backpressure timeout: with backpressure the queued
    /// overflow then drains to the wire (zero loss); with drop-newest the overflow was already gone.
    #[tokio::test]
    async fn egress_backpressures_when_open_channel_full() -> anyhow::Result<()> {
        let io = GatedWriteIo::default();
        let open_calls = Arc::new(AtomicUsize::new(0));
        let control = GatedControl {
            io: io.clone(),
            open_calls: open_calls.clone(),
        };

        // Small per-peer channel so a modest burst overflows it while the stream is open.
        let (mut tx_out, _rx_in) = process_stream_protocol(
            BytesCodec::new(),
            control,
            crate::config::StreamProtocolConfig {
                per_peer_channel_capacity: 4,
                // Flush every frame so the per-peer channel (not the FramedWrite buffer) is the
                // bottleneck that fills — making the ready+full path unambiguous.
                frame_writer_backpressure_bytes: 1,
                ..Default::default()
            },
        )
        .await?;

        let peer = PeerId::random();
        let msg = BytesMut::from(&b"hello"[..]);
        let n = 50usize;
        let expected_bytes = n * msg.len();

        // Burst far beyond the per-peer channel capacity while the writer is gated shut. The stream
        // opens and the write pump starts (ready == true) but cannot drain, so the channel fills and the
        // drain loop enters the ready+full path for the overflow.
        for _ in 0..n {
            tx_out
                .send((peer, msg.clone()))
                .await
                .context("send into egress queue should succeed")?;
        }
        assert!(
            wait_for(2, || open_calls.load(Ordering::Relaxed) >= 1).await,
            "stream was never opened"
        );

        // Let the drain loop hit the full channel and enter the (bounded) backpressure wait — well under
        // EGRESS_BACKPRESSURE_TIMEOUT. Under drop-newest the overflow is already dropped by now.
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // Release the writer: with backpressure the queued overflow now drains; with drop-newest only
        // the few packets that fit the channel were ever kept.
        io.release();

        // Allow a single frame to remain buffered inside `FramedWrite` (flushed only on the next write
        // cycle) — that is a framing artifact, not a drop. Drop-newest would lose the whole overflow
        // (only ~`per_peer_channel_capacity` packets ever kept), which is far below this bound.
        let min_delivered = expected_bytes - msg.len();
        assert!(
            wait_for(3, || io.written() >= min_delivered).await,
            "essentially all {n} packets ({expected_bytes} bytes) must reach the wire under egress backpressure; a \
             regression to drop-newest on a full open channel would lose the overflow (wrote {} bytes, need >= \
             {min_delivered})",
            io.written()
        );

        Ok(())
    }
}
