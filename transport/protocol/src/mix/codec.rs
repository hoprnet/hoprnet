use tokio_util::codec::{Decoder, Encoder};

pub mod v1 {
    use super::*;
    use hopr_crypto_packet::chain::ChainPacketComponents;

    #[derive(Clone)]
    pub struct MsgCodec;

    impl Encoder<Box<[u8]>> for MsgCodec {
        type Error = std::io::Error;

        fn encode(&mut self, item: Box<[u8]>, dst: &mut tokio_util::bytes::BytesMut) -> Result<(), Self::Error> {
            tracing::trace!(size = item.len(), protocol = "msg", "Encoding data");

            dst.extend_from_slice(&item);
            Ok(())
        }
    }

    impl Decoder for MsgCodec {
        type Item = Box<[u8]>;

        type Error = std::io::Error;

        fn decode(&mut self, src: &mut tokio_util::bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
            let len = src.len();
            if len >= ChainPacketComponents::SIZE {
                let packet = src.split_to(ChainPacketComponents::SIZE).freeze();

                tracing::trace!(size = packet.len(), protocol = "msg", "Decoding data");
                Ok(Some(Box::from_iter(packet)))
            } else {
                tracing::trace!(
                    available_bytes = len,
                    protocol = "msg",
                    "Skipping decoding operation, insufficient bytes available"
                );
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use hopr_crypto_packet::chain::ChainPacketComponents;

    #[test]
    fn codec_serialization_and_deserialization_are_reverse_operations() -> anyhow::Result<()> {
        let mut codec = v1::MsgCodec;
        let mut buf = tokio_util::bytes::BytesMut::new();

        const PAYLOAD_SIZE: usize = ChainPacketComponents::SIZE;
        let random_data_of_expected_packet_size: Box<[u8]> =
            Box::from(hopr_crypto_random::random_bytes::<PAYLOAD_SIZE>());

        codec.encode(random_data_of_expected_packet_size.clone(), &mut buf)?;

        let actual = codec.decode(&mut buf)?;

        assert_eq!(actual, Some(random_data_of_expected_packet_size));

        assert_eq!(buf.len(), 0);

        Ok(())
    }

    #[test]
    fn codec_deserialization_of_an_incomplete_byte_sequence_should_not_produce_an_item() -> anyhow::Result<()> {
        let mut codec = v1::MsgCodec;
        let mut buf = tokio_util::bytes::BytesMut::new();

        const LESS_THAN_PAYLOAD_SIZE: usize = ChainPacketComponents::SIZE - 1;
        let random_data_too_few_bytes: Box<[u8]> =
            Box::from(hopr_crypto_random::random_bytes::<LESS_THAN_PAYLOAD_SIZE>());

        codec.encode(random_data_too_few_bytes, &mut buf)?;

        let actual = codec.decode(&mut buf)?;

        assert_eq!(actual, None);

        assert_eq!(buf.len(), LESS_THAN_PAYLOAD_SIZE);

        Ok(())
    }

    #[test]
    fn codec_deserialization_of_too_many_bytes_should_produce_the_value_from_only_the_bytes_needed_for_an_item(
    ) -> anyhow::Result<()> {
        let mut codec = v1::MsgCodec;
        let mut buf = tokio_util::bytes::BytesMut::new();

        const MORE_THAN_PAYLOAD_SIZE: usize = ChainPacketComponents::SIZE + 1;
        let random_data_more_bytes_than_needed: Box<[u8]> =
            Box::from(hopr_crypto_random::random_bytes::<MORE_THAN_PAYLOAD_SIZE>());

        codec.encode(random_data_more_bytes_than_needed.clone(), &mut buf)?;

        let actual = codec.decode(&mut buf)?.context("The value should be available")?;

        assert_eq!(
            actual[..],
            random_data_more_bytes_than_needed[..ChainPacketComponents::SIZE]
        );

        assert_eq!(buf.len(), MORE_THAN_PAYLOAD_SIZE - ChainPacketComponents::SIZE);

        Ok(())
    }
}
