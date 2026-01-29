//! Infrastructure supporting converting a collection of [`PeerId`] split `libp2p_stream` managed
//! individual peer-to-peer `libp2p::swarm::Stream`s.

use std::sync::Arc;

use futures::{
    AsyncRead, AsyncReadExt, AsyncWrite, FutureExt, SinkExt as _, Stream, StreamExt,
    channel::mpsc::{Receiver, Sender, channel},
};
use hopr_transport_network::PeerPacketStats;
use libp2p::PeerId;
use tokio_util::{
    codec::{Decoder, Encoder, FramedRead, FramedWrite},
    compat::{FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt},
};

// TODO: see if these constants should be configurable instead

/// Global timeout for the [`BidirectionalStreamControl::open`] operation.
const GLOBAL_STREAM_OPEN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);
const MAX_CONCURRENT_PACKETS: usize = 30;

#[async_trait::async_trait]
pub trait BidirectionalStreamControl: std::fmt::Debug {
    fn accept(
        self,
    ) -> Result<impl Stream<Item = (PeerId, impl AsyncRead + AsyncWrite + Send)> + Send, impl std::error::Error>;

    async fn open(self, peer: PeerId) -> Result<impl AsyncRead + AsyncWrite + Send, impl std::error::Error>;
}

fn build_peer_stream_io<S, C>(
    peer: PeerId,
    stream: S,
    cache: moka::future::Cache<PeerId, Sender<<C as Decoder>::Item>>,
    codec: C,
    ingress_from_peers: Sender<(PeerId, <C as Decoder>::Item)>,
    packet_stats: Option<Arc<PeerPacketStats>>,
) -> Sender<<C as Decoder>::Item>
where
    S: AsyncRead + AsyncWrite + Send + 'static,
    C: Encoder<<C as Decoder>::Item> + Decoder + Send + Sync + Clone + 'static,
    <C as Encoder<<C as Decoder>::Item>>::Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    <C as Decoder>::Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    <C as Decoder>::Item: AsRef<[u8]> + Clone + Send + 'static,
{
    let (stream_rx, stream_tx) = stream.split();
    let (send, mut recv) = channel::<<C as Decoder>::Item>(1000);
    let cache_internal = cache.clone();

    let mut frame_writer = FramedWrite::new(stream_tx.compat_write(), codec.clone());

    // Lower the backpressure boundary to make sure each message is flushed after writing to buffer
    frame_writer.set_backpressure_boundary(1);

    // Clone stats for the egress and ingress spawns
    let packet_stats_out = packet_stats.clone();
    let packet_stats_in = packet_stats;

    // Send all outgoing data to the peer, recording stats only after a successful write
    hopr_async_runtime::prelude::spawn(async move {
        while let Some(data) = recv.next().await {
            let byte_len = data.as_ref().len();
            tracing::trace!(%peer, "writing message to peer stream");
            if let Err(error) = frame_writer.send(data).await {
                tracing::debug!(%peer, ?error, component = "stream", "writing stream with peer finished");
                return;
            }
            if let Some(ref stats) = packet_stats_out {
                stats.record_packet_out(byte_len);
            }
        }
        tracing::debug!(%peer, component = "stream", "writing stream with peer finished");
    });

    // Read all incoming data from that peer and pass it to the general ingress stream
    hopr_async_runtime::prelude::spawn(
        FramedRead::new(stream_rx.compat(), codec)
            .filter_map(move |v| {
                let stats_in = packet_stats_in.clone();
                async move {
                    match v {
                        Ok(v) => {
                            tracing::trace!(%peer, "read message from peer stream");
                            if let Some(ref stats) = stats_in {
                                stats.record_packet_in(v.as_ref().len());
                            }
                            Some((peer, v))
                        }
                        Err(error) => {
                            tracing::error!(%error, "Error decoding object from the underlying stream");
                            None
                        }
                    }
                }
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

pub async fn process_stream_protocol<C, V, F>(
    codec: C,
    control: V,
    get_packet_stats: F,
) -> crate::errors::Result<(
    Sender<(PeerId, <C as Decoder>::Item)>, // impl Sink<(PeerId, <C as Decoder>::Item)>,
    Receiver<(PeerId, <C as Decoder>::Item)>, // impl Stream<Item = (PeerId, <C as Decoder>::Item)>,
)>
where
    C: Encoder<<C as Decoder>::Item> + Decoder + Send + Sync + Clone + 'static,
    <C as Encoder<<C as Decoder>::Item>>::Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    <C as Decoder>::Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    <C as Decoder>::Item: AsRef<[u8]> + Clone + Send + 'static,
    V: BidirectionalStreamControl + Clone + Send + Sync + 'static,
    F: Fn(&PeerId) -> Option<Arc<PeerPacketStats>> + Clone + Send + Sync + 'static,
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

    let cache_ingress = cache_out.clone();
    let codec_ingress = codec.clone();
    let tx_in_ingress = tx_in.clone();
    let get_stats_ingress = get_packet_stats.clone();

    // terminated when the incoming is dropped
    let _ingress_process = hopr_async_runtime::prelude::spawn(
        incoming
            .for_each(move |(peer, stream)| {
                let codec = codec_ingress.clone();
                let cache = cache_ingress.clone();
                let tx_in = tx_in_ingress.clone();
                let get_stats = get_stats_ingress.clone();

                tracing::debug!(%peer, "received incoming peer-to-peer stream");

                let packet_stats = get_stats(&peer);
                let send =
                    build_peer_stream_io(peer, stream, cache.clone(), codec.clone(), tx_in.clone(), packet_stats);

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

    let max_concurrent_packets = std::env::var("HOPR_TRANSPORT_MAX_CONCURRENT_PACKETS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(MAX_CONCURRENT_PACKETS);

    let global_stream_open_timeout = std::env::var("HOPR_TRANSPORT_STREAM_OPEN_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(std::time::Duration::from_millis)
        .unwrap_or(GLOBAL_STREAM_OPEN_TIMEOUT);

    // terminated when the rx_in is dropped
    let _egress_process = hopr_async_runtime::prelude::spawn(
        rx_out
            .inspect(|(peer, _)| tracing::trace!(%peer, "proceeding to deliver message to peer"))
            .for_each_concurrent(max_concurrent_packets, move |(peer, msg)| {
                let cache = cache_out.clone();
                let control = control.clone();
                let codec = codec.clone();
                let tx_in = tx_in.clone();
                let get_stats = get_packet_stats.clone();

                async move {
                    let cache_clone = cache.clone();
                    tracing::trace!(%peer, "trying to deliver message to peer");

                    let packet_stats = get_stats(&peer);
                    let cached: Result<Sender<<C as Decoder>::Item>, Arc<anyhow::Error>> = cache
                        .try_get_with(peer, async move {
                            tracing::trace!(%peer, "peer is not in cache, opening new stream");

                            // There must be a timeout on the `open` operation; otherwise
                            //  a single impossible ` open ` operation will block the peer ID entry in the
                            // cache forever, having a direct detrimental effect on the packet
                            // processing pipeline.
                            use futures_time::future::FutureExt as TimeExt;
                            let stream = control
                                .open(peer)
                                .timeout(futures_time::time::Duration::from(global_stream_open_timeout))
                                .await
                                .map_err(|_| anyhow::anyhow!("timeout trying to open stream to {peer}"))?
                                .map_err(|e| anyhow::anyhow!("could not open outgoing peer-to-peer stream: {e}"))?;

                            tracing::debug!(%peer, "opening outgoing peer-to-peer stream");

                            Ok(build_peer_stream_io(
                                peer,
                                stream,
                                cache_clone.clone(),
                                codec.clone(),
                                tx_in.clone(),
                                packet_stats,
                            ))
                        })
                        .await;

                    match cached {
                        Ok(mut cached) => {
                            if let Err(error) = cached.send(msg).await {
                                tracing::error!(%peer, %error, "error sending message to peer");
                                cache.invalidate(&peer).await;
                            } else {
                                tracing::trace!(%peer, "message sent to peer");
                            }
                        }
                        Err(error) => {
                            tracing::error!(%peer, %error, "failed to open a stream to peer");
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
