//! Infrastructure supporting converting a collection of [`libp2p::PeerId`] split [`libp2p_stream`] managed
//! individual peer-to-peer [`libp2p::swarm::Stream`]s.

use futures::{AsyncRead, AsyncReadExt, AsyncWrite, Sink, SinkExt as _, Stream, StreamExt};
use futures_concurrency::stream::StreamExt as _;
use libp2p::PeerId;
use tokio_util::{
    codec::{Decoder, Encoder, FramedRead, FramedWrite},
    compat::{FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt},
};

#[async_trait::async_trait]
pub trait BidirectionalStreamControl {
    async fn accept(
        self,
    ) -> Result<impl Stream<Item = (PeerId, impl AsyncRead + AsyncWrite + Send)> + Send, impl std::error::Error>;
    async fn open(self) -> Result<impl AsyncRead + AsyncWrite + Send, impl std::error::Error>;
}

fn split_with_codec<T, C, U>(channel: T, codec: C) -> (impl Sink<U>, impl Stream<Item = <C as Decoder>::Item>)
where
    T: AsyncRead + AsyncWrite,
    C: Encoder<U> + Decoder + Clone,
    <C as Decoder>::Error: std::fmt::Display,
{
    let (rx, tx) = channel.split();
    (
        FramedWrite::new(tx.compat_write(), codec.clone()),
        FramedRead::new(rx.compat(), codec).filter_map(|read| async move {
            match read {
                Ok(v) => Some(v),
                Err(error) => {
                    tracing::error!(%error, "Error decoding object from the underlying stream");
                    None
                }
            }
        }),
    )
}

enum Dispatched<T> {
    Out(T),
    In((PeerId, T)),
}

async fn process_stream_protocol<T, E, C, V, U>(
    codec: C,
    control: V,
) -> crate::errors::Result<(
    impl Sink<(PeerId, <C as Decoder>::Item)>,
    impl Stream<Item = (PeerId, <C as Decoder>::Item)>,
)>
where
    C: Default + Encoder<U> + Decoder + Send + Sync + Clone + 'static,
    U: Send + 'static,
    <C as Decoder>::Error: std::fmt::Display + Send + 'static,
    <C as Decoder>::Item: Send + 'static,
    V: BidirectionalStreamControl + Clone + Send + Sync + 'static,
{
    let (tx_out, rx_out) = futures::channel::mpsc::channel::<(PeerId, <C as Decoder>::Item)>(1000);
    let (tx_in, rx_in) = futures::channel::mpsc::channel::<(PeerId, <C as Decoder>::Item)>(1000);

    let cache_out = moka::future::Cache::new(2000);

    let incoming = control
        .clone()
        .accept()
        .await
        .map_err(|e| crate::errors::ProtocolError::Logic(format!("failed to listen on protocol: {e}")))?;

    let cache_ingress = cache_out.clone();
    let codec_ingress = codec.clone();
    let tx_in_ingress = tx_in.clone();

    // terminated when the incoming is dropped
    let _ingress_process = hopr_async_runtime::prelude::spawn(incoming.for_each(move |(peer_id, stream)| {
        let codec = codec_ingress.clone();
        let cache = cache_ingress.clone();
        let tx_in = tx_in_ingress.clone();

        async move {
            let (_, b) = split_with_codec(stream, codec);
            let (send, recv) = futures::channel::mpsc::channel::<<C as Decoder>::Item>(1000);

            hopr_async_runtime::prelude::spawn(
                recv.map(Dispatched::Out)
                    .merge(b.map(move |v| Dispatched::In((peer_id, v))))
                    .for_each_concurrent(10, move |v| {
                        let mut tx_in = tx_in.clone();

                        async move {
                            match v {
                                Dispatched::Out(_) => {
                                    // a.send(msg).await.unwrap();
                                }
                                Dispatched::In((peer_id, msg)) => {
                                    tx_in.send((peer_id, msg)).await.unwrap(); // TODO: remove unwrap
                                }
                            }
                        }
                    }),
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
            let cached = cache
                .optionally_get_with(peer_id, async move {
                    let r = control.open().await;
                    match r {
                        Ok(stream) => {
                            let (a, b) = split_with_codec(stream, codec.clone());
                            let (send, recv) = futures::channel::mpsc::channel::<<C as Decoder>::Item>(1000);

                            hopr_async_runtime::prelude::spawn(
                                recv.map(Dispatched::Out)
                                    .merge(b.map(move |v| Dispatched::In((peer_id, v))))
                                    .for_each_concurrent(10, move |v| {
                                        let mut tx_in = tx_in.clone();

                                        async move {
                                            match v {
                                                Dispatched::Out(_) => {
                                                    // a.send(msg).await.unwrap();
                                                }
                                                Dispatched::In((peer_id, msg)) => {
                                                    tx_in.send((peer_id, msg)).await.unwrap();
                                                }
                                            }
                                        }
                                    }),
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
        }
    }));

    Ok((tx_out, rx_in))
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::SinkExt;

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

    #[async_std::test]
    async fn split_codec_should_always_produce_correct_data() -> anyhow::Result<()> {
        let stream = AsyncBinaryStreamChannel::new();
        let codec = tokio_util::codec::BytesCodec::new();

        let expected = [0u8, 1u8, 2u8, 3u8, 4u8, 5u8];
        let value = tokio_util::bytes::BytesMut::from(expected.as_ref());

        let (mut tx, rx) = split_with_codec(stream, codec);
        tx.send(value)
            .await
            .map_err(|_| anyhow::anyhow!("should not fail on send"))?;

        futures::pin_mut!(rx);

        assert_eq!(rx.next().await, Some(tokio_util::bytes::BytesMut::from(expected.as_ref())));

        Ok(())
    }
}
