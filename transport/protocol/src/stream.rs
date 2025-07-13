//! Infrastructure supporting converting a collection of [`libp2p::PeerId`] split [`libp2p_stream`] managed
//! individual peer-to-peer [`libp2p::swarm::Stream`]s.

use std::sync::Arc;

use futures::{AsyncRead, AsyncReadExt, AsyncWrite, SinkExt as _, Stream, StreamExt, channel::mpsc::{Receiver, Sender, channel}, FutureExt};
use libp2p::PeerId;
use tokio_util::{
    codec::{Decoder, Encoder, FramedRead, FramedWrite},
    compat::{FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt},
};

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
    ingress_from_peers: Sender<(PeerId,  <C as Decoder>::Item)>) -> Sender<<C as Decoder>::Item>
where
    S: AsyncRead + AsyncWrite + Send + 'static,
    C: Encoder<<C as Decoder>::Item> + Decoder + Send + Sync + Clone + 'static,
    <C as Encoder<<C as Decoder>::Item>>::Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    <C as Decoder>::Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    <C as Decoder>::Item: Clone + Send + 'static,
{
    let (stream_rx, stream_tx) = stream.split();
    let (send, recv) = channel::<<C as Decoder>::Item>(1000);
    let cache_internal = cache.clone();

    let mut fw = FramedWrite::new(stream_tx.compat_write(), codec.clone());
    fw.set_backpressure_boundary(1); // Low backpressure boundary to make sure each message is flushed after writing to buffer

    hopr_async_runtime::prelude::spawn(recv
        .inspect(move |_| tracing::trace!(%peer, "writing message to peer stream"))
        .map(Ok)
        .forward(fw)
        .then(move |res| {
            tracing::debug!(%peer, ?res, "writing stream with peer done");
            futures::future::ready(())
        })
    );

    hopr_async_runtime::prelude::spawn(FramedRead::new(stream_rx.compat(), codec)
        .filter_map(move |v| async move {
            match v {
                Ok(v) => {
                    tracing::trace!(%peer, "read message from peer stream");
                    Some((peer, v))
                },
                Err(error) => {
                    tracing::error!(%error, "Error decoding object from the underlying stream");
                    None
                }
            }
        })
        .map(Ok)
        .forward(ingress_from_peers)
        .then(move |res| match res {
            Ok(_) => {
                tracing::debug!(%peer, "received incoming stream done reading");
                futures::future::ready(())
            }
            Err(error) => {
                tracing::error!(%peer, %error, "received incoming stream failed on reading");
                futures::future::ready(())
            }
        })
        .then(move |_| {
            let peer = peer.clone();
            async move {
                cache_internal.invalidate(&peer).await;
            }
        })
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
    <C as Decoder>::Item: Clone + Send + 'static,
    V: BidirectionalStreamControl + Clone + Send + Sync + 'static,
{
    let (tx_out, rx_out) = channel::<(PeerId, <C as Decoder>::Item)>(10_000);
    let (tx_in, rx_in) = channel::<(PeerId, <C as Decoder>::Item)>(10_000);

    let cache_out = moka::future::Cache::builder()
        .max_capacity(2000)
        .eviction_listener(|key: Arc<PeerId>, _value, cause| {
            tracing::trace!(?key, ?cause, "Evicting stream for peer");
        })
        .build();

    let incoming = control
        .clone()
        .accept()
        .map_err(|e| crate::errors::ProtocolError::Logic(format!("failed to listen on protocol: {e}")))?;

    let cache_ingress = cache_out.clone();
    let codec_ingress = codec.clone();
    let tx_in_ingress = tx_in.clone();

    // terminated when the incoming is dropped
    let _ingress_process = hopr_async_runtime::prelude::spawn(incoming.for_each(move |(peer, stream)| {
        let codec = codec_ingress.clone();
        let cache = cache_ingress.clone();
        let tx_in = tx_in_ingress.clone();

        tracing::debug!(%peer, "Received incoming peer-to-peer stream");

        let send = build_peer_stream_io(peer, stream, cache.clone(), codec.clone(), tx_in.clone());

        async move {
            cache.insert(peer, send).await;
        }
    }));

    // terminated when the rx_in is dropped
    let _egress_process = hopr_async_runtime::prelude::spawn(rx_out
        .inspect(|(peer, _)| tracing::trace!(%peer, "proceeding to deliver message to peer"))
        .for_each_concurrent(None, move |(peer, msg)| {
        let cache = cache_out.clone();
        let control = control.clone();
        let codec = codec.clone();
        let tx_in = tx_in.clone();

        async move {
            let cache = cache.clone();

            tracing::trace!(%peer, "trying to deliver message to peer");
            if let Some(mut cached) = cache.get(&peer).await {
                if let Err(error) = cached.send(msg.clone()).await {
                    tracing::error!(%peer, %error, "Error sending message to peer from the cached connection");
                    cache.invalidate(&peer).await;
                } else {
                    tracing::trace!(%peer, "message sent to peer from the cached connection");
                    return;
                }
            }

            tracing::trace!(%peer, "peer stream not found on the first try");

            let cache_clone = cache.clone();
            let cached: std::result::Result<Sender<<C as Decoder>::Item>, Arc<anyhow::Error>> = cache
                .try_get_with(peer, async move {
                    tracing::trace!(%peer, "peer is not in cache, opening new stream...");

                    let stream = control
                        .open(peer)
                        .await
                        .map_err(|e| anyhow::anyhow!("Could not open outgoing peer-to-peer stream: {e}"))?;
                    tracing::debug!(%peer, "Opening outgoing peer-to-peer stream");

                    Ok(build_peer_stream_io(peer, stream, cache_clone.clone(), codec.clone(), tx_in.clone()))
                })
                .await;

            match cached {
                Ok(mut cached) => {
                    if let Err(error) = cached.send(msg).await {
                        tracing::error!(%peer, %error, "Error sending message to peer");
                        cache.invalidate(&peer).await;
                    } else {
                        tracing::trace!(%peer, "message sent to peer");
                    }
                }
                Err(error) => {
                    tracing::error!(%peer, %error, "Failed to open a stream to peer");
                }
            }
        }
    }));

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
