//! Infrastructure supporting converting a collection of [`libp2p::PeerId`] split [`libp2p_stream`] managed
//! individual peer-to-peer [`libp2p::swarm::Stream`]s.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use asynchronous_codec::{BytesCodec, Decoder, Encoder, FramedRead, FramedWrite};
use futures::{stream::BoxStream, AsyncRead, AsyncReadExt, AsyncWrite, Sink, SinkExt, Stream, StreamExt};
use futures_concurrency::stream::StreamExt as _;
use hopr_internal_types::protocol::Acknowledgement;
use libp2p::PeerId;

mod msg_v1 {
    use hopr_crypto_packet::chain::ChainPacketComponents;

    pub(crate) struct MsgCodec;

    impl asynchronous_codec::Encoder for MsgCodec {
        type Item<'a> = Box<[u8]>;

        type Error = std::io::Error;

        fn encode(&mut self, item: Self::Item<'_>, dst: &mut asynchronous_codec::BytesMut) -> Result<(), Self::Error> {
            dst.extend_from_slice(&item);
            Ok(())
        }
    }

    impl asynchronous_codec::Decoder for MsgCodec {
        type Item = Box<[u8]>;

        type Error = std::io::Error;

        fn decode(&mut self, src: &mut asynchronous_codec::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
            let len = src.len();
            if len >= ChainPacketComponents::SIZE {
                let packet = src.split_to(ChainPacketComponents::SIZE).freeze();

                Ok(Some(Box::from_iter(packet.into_iter())))
            } else {
                Ok(None)
            }
        }
    }
}

type MsgCodec = msg_v1::MsgCodec;
type AckCodec = asynchronous_codec::CborCodec<Acknowledgement, Acknowledgement>;

fn split_with_codec<T, C>(
    channel: T,
    codec: C,
) -> (
    impl Sink<<C as Encoder>::Item<'static>>,
    impl Stream<Item = <C as Decoder>::Item>,
)
where
    T: AsyncRead + AsyncWrite,
    C: asynchronous_codec::Encoder + asynchronous_codec::Decoder + Clone,
{
    let (rx, tx) = channel.split();
    (
        FramedWrite::new(tx, codec.clone()),
        FramedRead::new(rx, codec).filter_map(|read| async move {
            match read {
                Ok(v) => Some(v),
                Err(_) => {
                    tracing::error!("Error decoding object from the underlying stream");
                    None
                }
            }
        }),
    )
}

#[async_trait::async_trait]
trait StreamControl {
    async fn accept(
        self,
    ) -> Result<impl Stream<Item = (PeerId, impl AsyncRead + AsyncWrite + Send)> + Send, impl std::error::Error>;
    async fn open(self) -> Result<impl AsyncRead + AsyncWrite + Send, impl std::error::Error>;
}

enum Dispatched<T> {
    Out(T),
    In((PeerId, T)),
}

async fn process_logical_in_task<T, U, V>(
    peer: PeerId,
    a: impl Sink<U> + Send + 'static,
    b: impl Stream<Item = V> + Send + 'static,
    tx_in: T,
) -> futures::channel::mpsc::Sender<V>
where
    T: Sink<(PeerId, V)> + Clone + Send + std::marker::Unpin + 'static,
    V: Send + 'static,
{
    let (send, recv) = futures::channel::mpsc::channel::<V>(1000);

    hopr_async_runtime::prelude::spawn(
        recv.map(Dispatched::Out)
            .merge(b.map(move |v| Dispatched::In((peer, v))))
            .for_each_concurrent(10, move |v| {
                let mut tx_in = tx_in.clone();

                async move {
                    match v {
                        Dispatched::Out(_) => {
                            // a.send(msg).await.unwrap();
                        }
                        Dispatched::In((peer, msg)) => {
                            // let result = tx_in.send((peer, msg));
                        }
                    }
                }
            }),
    );

    send
}

async fn process_stream_in_task<S, C, T>(
    peer: PeerId,
    stream: S,
    codec: C,
    tx_in: T,
) -> futures::channel::mpsc::Sender<<C as Decoder>::Item>
where
    C: Default + Encoder + Decoder + Send + Sync + Clone + 'static,
    <C as Decoder>::Error: Send + 'static,
    <C as Decoder>::Item: Send + 'static,
    S: AsyncRead + AsyncWrite + Send + 'static,
    T: Sink<(PeerId, <C as Decoder>::Item)> + Clone + Send + std::marker::Unpin + 'static,
{
    let (a, b) = split_with_codec(stream, codec);
    let (send, recv) = futures::channel::mpsc::channel::<<C as Decoder>::Item>(1000);

    hopr_async_runtime::prelude::spawn(
        recv.map(Dispatched::Out)
            .merge(b.map(move |v| Dispatched::In((peer, v))))
            .for_each_concurrent(10, move |v| {
                let mut tx_in = tx_in.clone();

                async move {
                    match v {
                        Dispatched::Out(_) => {
                            // a.send(msg).await.unwrap();
                        }
                        Dispatched::In((peer, msg)) => {
                            // let result = tx_in.send((peer, msg));
                        }
                    }
                }
            }),
    );

    send
}

async fn dispatch_protocol<T, E, C, V>(
    codec: C,
    control: V,
) -> crate::errors::Result<(
    impl Sink<(PeerId, <C as Decoder>::Item)>,
    impl Stream<Item = (PeerId, <C as Decoder>::Item)>,
)>
where
    C: Default + Encoder + Decoder + Send + Sync + Clone + 'static,
    <C as Decoder>::Error: Send + 'static,
    <C as Decoder>::Item: Send + 'static,
    V: StreamControl + Clone + Send + Sync + 'static,
    // V: StreamControl<T, E> + Clone + Send + Sync + 'static,
    // T: AsyncRead + AsyncWrite + Send + 'static,
    // E: std::error::Error + 'static,
{
    let (tx_out, mut rx_out) = futures::channel::mpsc::channel::<(PeerId, <C as Decoder>::Item)>(1000);
    let (mut tx_in, rx_in) = futures::channel::mpsc::channel::<(PeerId, <C as Decoder>::Item)>(1000);

    let cache_out = moka::future::Cache::new(2000);
    let control = control.clone();
    let control_ingress = control.clone();

    let egress_cache = cache_out.clone();

    let cache_in = cache_out.clone();
    // cache_in.insert(PeerId::random(), 32); // TODO: remove

    let codec_in = codec.clone();
    let tx_in_in = tx_in.clone();
    let incoming = control_ingress
        .accept()
        .await
        .map_err(|e| crate::errors::ProtocolError::Logic(format!("failed to listen on protocol: {e}")))?;

    let ingress_fut = async move {
        incoming.for_each(move |(peer_id, stream)| {
            let codec = codec_in.clone();
            let cache = cache_in.clone();
            let tx_in = tx_in_in.clone();

            async move {
                // TODO: Duplicated
                let (_, b) = split_with_codec(stream, codec);
                let (send, recv) = futures::channel::mpsc::channel::<<C as Decoder>::Item>(1000);

                hopr_async_runtime::prelude::spawn(
                    recv.map(Dispatched::Out)
                        .merge(b.map(move |v| Dispatched::In((peer_id, v))))
                        .for_each_concurrent(10, move |v| async move {
                            match v {
                                Dispatched::Out(_) => {
                                    // a.send(msg).await.unwrap();
                                }
                                Dispatched::In((_, _)) => {
                                    // tx_in_clone.send((peer_id, msg)).await.unwrap();
                                }
                            }
                        }),
                );

                // let (a, b) = split_with_codec(stream, codec);
                // let send = process_logical_in_task(peer_id, a, b, tx_in).await;

                // let send = process_stream_in_task(peer_id, stream, codec, tx_in).await;
                cache.insert(peer_id, send).await;
            }
        })
    };

    let ingress_proc = hopr_async_runtime::prelude::spawn(ingress_fut);

    // let cache_egress = egress_cache.clone();
    let control_egress = control.clone();
    // let codec_egress = codec.clone();
    // let tx_in_egress = tx_in.clone();
    let egress_proc = hopr_async_runtime::prelude::spawn(rx_out.for_each(move |(peer_id, msg)| {
        let cache = egress_cache.clone();
        let control = control_egress.clone();
        let codec = codec.clone();
        let tx_in = tx_in.clone();

        async move {
            let cached = cache
                .optionally_get_with(peer_id, async move {
                    let r = control.open().await;
                    match r {
                        Ok(stream) => {
                            let (a, b) = split_with_codec(stream, codec.clone());
                            let (mut send, recv) = futures::channel::mpsc::channel::<<C as Decoder>::Item>(1000);

                            let mut tx_in_clone = tx_in.clone();
                            hopr_async_runtime::prelude::spawn(
                                recv.map(Dispatched::Out)
                                    .merge(b.map(move |v| Dispatched::In((peer_id, v))))
                                    .for_each_concurrent(10, move |v| async move {
                                        match v {
                                            Dispatched::Out(msg) => {
                                                // a.send(msg).await.unwrap();
                                            }
                                            Dispatched::In((peer_id, msg)) => {
                                                // tx_in_clone.send((peer_id, msg)).await.unwrap();
                                            }
                                        }
                                    }),
                            );

                            // let send = process_stream_in_task(peer_id, stream, codec, tx_in).await;

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
    // use super::ProtocolCodec;

    #[async_std::test]
    async fn sanity() {
        // let codec = ProtocolCodec::new();

        assert!(false);
    }
}
