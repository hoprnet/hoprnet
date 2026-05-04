//! Infrastructure supporting converting a collection of [`PeerId`] split `libp2p_stream` managed
//! individual peer-to-peer `libp2p::swarm::Stream`s.

use std::sync::Arc;

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
    static ref METRIC_PEER_BUFFER_FULL_DROP: hopr_metrics::SimpleCounter =
        hopr_metrics::SimpleCounter::new(
            "hopr_egress_per_peer_buffer_full_dropped",
            "Number of packets dropped because the per-peer egress buffer was full",
        )
        .unwrap();
}

// TODO: see if these constants should be configurable instead

/// Global timeout for the `BidirectionalStreamControl::open` operation.
const GLOBAL_STREAM_OPEN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);
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

fn build_peer_stream_io<S, C>(
    peer: PeerId,
    stream: S,
    cache: moka::future::Cache<PeerId, Sender<<C as Decoder>::Item>>,
    codec: C,
    ingress_from_peers: Sender<(PeerId, <C as Decoder>::Item)>,
    frame_writer_backpressure_bytes: usize,
) -> Sender<<C as Decoder>::Item>
where
    S: AsyncRead + AsyncWrite + Send + 'static,
    C: Encoder<<C as Decoder>::Item> + Decoder + Send + Sync + Clone + 'static,
    <C as Encoder<<C as Decoder>::Item>>::Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    <C as Decoder>::Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    <C as Decoder>::Item: AsRef<[u8]> + Clone + Send + 'static,
{
    let (stream_rx, stream_tx) = stream.split();
    let (send, recv) = channel::<<C as Decoder>::Item>(1000);
    let cache_internal = cache.clone();

    let mut frame_writer = FramedWrite::new(stream_tx.compat_write(), codec.clone());

    // `set_backpressure_boundary` is a *byte* threshold on `FramedWrite`'s internal
    // pending-write buffer: once the encoded frames exceed this many bytes, the next
    // `poll_ready` call will issue a flush. A small value (e.g. `1`) flushes on every
    // message for the lowest latency at the cost of one syscall per frame; a larger
    // value lets adjacent small frames coalesce into a single write on busy relays.
    frame_writer.set_backpressure_boundary(frame_writer_backpressure_bytes);

    // Send all outgoing data to the peer
    hopr_async_runtime::prelude::spawn(
        recv.inspect(move |_| tracing::trace!(%peer, "writing message to peer stream"))
            .map(Ok)
            .forward(frame_writer)
            .inspect(move |res| {
                tracing::debug!(%peer, ?res, component = "stream", "writing stream with peer finished");
            }),
    );

    // Read all incoming data from that peer and pass it to the general ingress stream
    hopr_async_runtime::prelude::spawn(
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
                    cache_internal.invalidate(&peer).await;
                }
            }),
    );

    tracing::trace!(%peer, "created new io for peer");
    send
}

pub async fn process_stream_protocol<C, V>(
    codec: C,
    control: V,
) -> crate::errors::Result<(
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

    let cache_out = moka::future::Cache::builder()
        .max_capacity(2000)
        .eviction_listener(|key: Arc<PeerId>, _, cause| {
            tracing::trace!(peer = %key.as_ref(), ?cause, "evicting stream for peer");
        })
        .build();

    let incoming = control
        .clone()
        .accept()
        .map_err(|e| crate::errors::ProtocolError::Logic(format!("failed to listen on protocol: {e}")))?;

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

    // Pack the handles only needed to open a NEW peer stream into a single Arc. This
    // lets us pay one Arc bump per packet at the closure level instead of three (control,
    // codec, tx_in), and defers the per-field `.clone()` to the cache-miss path that
    // actually needs them.
    let open_ctx = Arc::new((control, codec, tx_in));

    let cache_ingress = cache_out.clone();
    let open_ctx_ingress = open_ctx.clone();

    // terminated when the incoming is dropped
    let _ingress_process = hopr_async_runtime::prelude::spawn(
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
                );

                async move {
                    cache.insert(peer, send).await;
                }
            })
            .inspect(|_| {
                tracing::info!(
                    task = "ingress stream processing",
                    "long-running background task finished"
                )
            }),
    );

    // terminated when the rx_in is dropped
    let _egress_process = hopr_async_runtime::prelude::spawn(
        rx_out
            .inspect(|(peer, _)| tracing::trace!(%peer, "proceeding to deliver message to peer"))
            .for_each_concurrent(max_concurrent_packets, move |(peer, msg)| {
                let cache = cache_out.clone();
                let open_ctx = open_ctx.clone();

                async move {
                    tracing::trace!(%peer, "trying to deliver message to peer");

                    let cache_clone = cache.clone();
                    let cached: Result<Sender<<C as Decoder>::Item>, Arc<anyhow::Error>> = cache
                        .try_get_with(peer, async move {
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
                                .map_err(|_| anyhow::anyhow!("timeout trying to open stream to {peer}"))?
                                .map_err(|e| anyhow::anyhow!("could not open outgoing peer-to-peer stream: {e}"))?;

                            tracing::debug!(%peer, "opening outgoing peer-to-peer stream");

                            Ok(build_peer_stream_io(
                                peer,
                                stream,
                                cache_clone,
                                codec.clone(),
                                tx_in.clone(),
                                frame_writer_backpressure_bytes,
                            ))
                        })
                        .await;

                    match cached {
                        Ok(mut cached) => {
                            if let Err(err) = cached.try_send(msg) {
                                if err.is_full() {
                                    #[cfg(all(feature = "telemetry", not(test)))]
                                    METRIC_PEER_BUFFER_FULL_DROP.increment();
                                    tracing::warn!(%peer, "per-peer egress buffer full, dropping packet");
                                } else {
                                    tracing::error!(%peer, "per-peer egress channel disconnected");
                                }
                                cache.invalidate(&peer).await;
                            } else {
                                tracing::trace!(%peer, "message sent to peer");
                            }
                        }
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
    use anyhow::Context;
    use futures::SinkExt;

    use super::*;

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
}
