//! Infrastructure supporting converting a collection of [`libp2p::PeerId`] split [`libp2p_stream`] managed
//! individual peer-to-peer [`libp2p::swarm::Stream`]s.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use asynchronous_codec::{BytesCodec, Decoder, Encoder, FramedRead, FramedWrite};
use futures::{AsyncRead, AsyncReadExt, AsyncWrite, Sink, SinkExt, Stream, StreamExt};
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
        &mut self,
    ) -> Result<impl Stream<Item = impl AsyncRead + AsyncWrite + Send>, impl std::error::Error>;
    async fn open(&mut self) -> Result<impl AsyncRead + AsyncWrite + Send, impl std::error::Error>;
}

enum Dispatched<T> {
    Out(T),
    In((PeerId, T)),
}

async fn dispatch_protocol<C, V>(
    codec: C,
    control: V,
) -> crate::errors::Result<(
    impl Sink<(PeerId, <C as Decoder>::Item)>,
    impl Stream<Item = (PeerId, <C as Decoder>::Item)>,
)>
where
    C: Default + Encoder + Decoder + Send + Sync + Clone + 'static,
    <C as Decoder>::Error: Send,
    <C as Decoder>::Item: Send + 'static,
    V: StreamControl + Clone + Send + Sync + 'static,
    // U: Sink<T> + Send + Sync + Clone + 'static,
{
    let (tx_out, mut rx_out) = futures::channel::mpsc::channel::<(PeerId, <C as Decoder>::Item)>(1000);
    let (mut tx_in, rx_in) = futures::channel::mpsc::channel::<(PeerId, <C as Decoder>::Item)>(1000);

    let cache_out = moka::future::Cache::new(2000);

    let egress_cache = cache_out.clone();
    let egress_proc = hopr_async_runtime::prelude::spawn(async move {
        let cache = egress_cache;
        let control = control.clone();

        while let Some((peer_id, msg)) = rx_out.next().await {
            let cached = cache.get(&peer_id).await;

            if cached.is_none() {
                let mut control = control.clone();
                match control.open().await {
                    Ok(stream) => {
                        let (a, b) = split_with_codec(stream, codec.clone());
                        let (mut send, recv) = futures::channel::mpsc::channel::<<C as Decoder>::Item>(1000);

                        let mut tx_in_clone = tx_in.clone();
                        hopr_async_runtime::prelude::spawn(async move {
                            let outgoing_and_incoming =
                                recv.map(Dispatched::Out).merge(b.map(|v| Dispatched::In((peer_id, v))));

                            futures::pin_mut!(outgoing_and_incoming);
                            while let Some(msg) = outgoing_and_incoming.next().await {
                                match msg {
                                    Dispatched::Out(msg) => {
                                        // a.send(msg).await.unwrap();
                                    }
                                    Dispatched::In((peer_id, msg)) => {
                                        // tx_in_clone.send((peer_id, msg)).await.unwrap();
                                    }
                                }
                            }
                        });
                        cache.insert(peer_id, send);
                    }
                    Err(error) => {
                        tracing::error!(peer = %peer_id, %error, "Error opening stream to peer");
                        continue;
                    }
                }
            }
        }
    });

    // let (msg_sink, msg_stream) = split_with_codec(control.open()?, codec.clone());

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
