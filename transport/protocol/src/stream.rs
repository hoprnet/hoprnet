//! Infrastructure supporting converting a collection of [`libp2p::PeerId`] split [`libp2p_stream`] managed
//! individual peer-to-peer [`libp2p::swarm::Stream`]s.

use futures::{AsyncRead, AsyncReadExt, AsyncWrite, SinkExt as _, Stream, StreamExt};
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

pub async fn process_stream_protocol<C, V>(
    codec: C,
    control: V,
) -> crate::errors::Result<(
    futures::channel::mpsc::Sender<(PeerId, <C as Decoder>::Item)>, // impl Sink<(PeerId, <C as Decoder>::Item)>,
    futures::channel::mpsc::Receiver<(PeerId, <C as Decoder>::Item)>, /* impl Stream<Item = (PeerId, <C as
                                                                     * Decoder>::Item)>, */
)>
where
    C: Encoder<<C as Decoder>::Item> + Decoder + Send + Sync + Clone + 'static,
    <C as Encoder<<C as Decoder>::Item>>::Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    <C as Decoder>::Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    <C as Decoder>::Item: Send + 'static,
    V: BidirectionalStreamControl + Clone + Send + Sync + 'static,
{
    let (tx_out, rx_out) = futures::channel::mpsc::channel::<(PeerId, <C as Decoder>::Item)>(10_000);
    let (tx_in, rx_in) = futures::channel::mpsc::channel::<(PeerId, <C as Decoder>::Item)>(10_000);

    let cache_out = moka::future::Cache::new(2000);

    let incoming = control
        .clone()
        .accept()
        .map_err(|e| crate::errors::ProtocolError::Logic(format!("failed to listen on protocol: {e}")))?;

    let cache_ingress = cache_out.clone();
    let codec_ingress = codec.clone();
    let tx_in_ingress = tx_in.clone();

    // terminated when the incoming is dropped
    let _ingress_process = hopr_async_runtime::prelude::spawn(incoming.for_each(move |(peer_id, stream)| {
        let codec = codec_ingress.clone();
        let cache = cache_ingress.clone();
        let tx_in = tx_in_ingress.clone();

        tracing::debug!(peer = %peer_id, "Received incoming peer-to-peer stream");

        async move {
            let (stream_rx, stream_tx) = stream.split();
            let (send, recv) = futures::channel::mpsc::channel::<<C as Decoder>::Item>(1000);

            hopr_async_runtime::prelude::spawn(
                recv.map(Ok)
                    .forward(FramedWrite::new(stream_tx.compat_write(), codec.clone())),
            );
            hopr_async_runtime::prelude::spawn(
                FramedRead::new(stream_rx.compat(), codec)
                    .filter_map(move |v| async move {
                        match v {
                            Ok(v) => Some((peer_id, v)),
                            Err(e) => {
                                tracing::error!(error = %e, "Error decoding object from the underlying stream");
                                None
                            }
                        }
                    })
                    .map(Ok)
                    .forward(tx_in),
            );
            cache.insert(peer_id, send).await;
        }
    }));

    // terminated when the rx_in is dropped
    let _egress_process = hopr_async_runtime::prelude::spawn(rx_out.for_each(move |(peer_id, msg)| {
        let cache = cache_out.clone();
        let control = control.clone();
        let codec = codec.clone();
        let tx_in = tx_in.clone();

        async move {
            let cache = cache.clone();

            let cached = cache
                .optionally_get_with(peer_id, async move {
                    let r = control.open(peer_id).await;
                    match r {
                        Ok(stream) => {
                            tracing::debug!(peer = %peer_id, "Opening outgoing peer-to-peer stream");

                            let (stream_rx, stream_tx) = stream.split();
                            let (send, recv) = futures::channel::mpsc::channel::<<C as Decoder>::Item>(1000);

                            hopr_async_runtime::prelude::spawn(
                                recv.map(Ok)
                                    .forward(FramedWrite::new(stream_tx.compat_write(), codec.clone())),
                            );
                            hopr_async_runtime::prelude::spawn(
                                FramedRead::new(stream_rx.compat(), codec)
                                    .filter_map(move |v| async move {
                                        match v {
                                            Ok(v) => Some((peer_id, v)),
                                            Err(e) => {
                                                tracing::error!(error = %e, "Error decoding object from the underlying stream");
                                                None
                                            }
                                        }
                                    })
                                    .map(Ok)
                                    .forward(tx_in),
                            );

                            Some(send)
                        }
                        Err(error) => {
                            tracing::error!(peer = %peer_id, %error, "Error opening stream to peer");
                            None
                        }
                    }
                })
                .await;

            if let Some(mut cached) = cached {
                if let Err(error) = cached.send(msg).await {
                    tracing::error!(peer = %peer_id, %error, "Error sending message to peer");
                    cache.invalidate(&peer_id).await;
                }
            } else {
                tracing::error!(peer = %peer_id, "Error sending message to peer: the stream failed to be created and cached");
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
