use tokio_util::codec::{Decoder, Encoder};

#[derive(Clone)]
pub struct FixedLengthCodec<const SIZE: usize>;

impl<const SIZE: usize> Encoder<Box<[u8]>> for FixedLengthCodec<SIZE> {
    type Error = std::io::Error;

    #[tracing::instrument(level = "trace", skip(self, item, dst), name = "encode data", fields(size = item.len(), protocol = "msg"), ret, err)]
    fn encode(&mut self, item: Box<[u8]>, dst: &mut tokio_util::bytes::BytesMut) -> Result<(), Self::Error> {
        dst.extend_from_slice(&item);
        Ok(())
    }
}

impl<const SIZE: usize> Decoder for FixedLengthCodec<SIZE> {
    type Error = std::io::Error;
    type Item = Box<[u8]>;

    #[tracing::instrument(level = "trace", skip(self, src), name = "decode data", fields(size = src.len(), protocol = "msg"), ret(Debug), err)]
    fn decode(&mut self, src: &mut tokio_util::bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let len = src.len();
        if len >= SIZE {
            let packet = src.split_to(SIZE).freeze();

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

#[cfg(test)]
mod tests {
    use anyhow::Context;

    use super::*;

    const TEST_SIZE: usize = 10;

    #[test]
    fn fixed_size_codec_serialization_and_deserialization_are_reverse_operations() -> anyhow::Result<()> {
        let mut codec = FixedLengthCodec::<TEST_SIZE>;
        let mut buf = tokio_util::bytes::BytesMut::new();

        let random_data_of_expected_packet_size: Box<[u8]> = Box::from(hopr_crypto_random::random_bytes::<TEST_SIZE>());

        codec.encode(random_data_of_expected_packet_size.clone(), &mut buf)?;

        let actual = codec.decode(&mut buf)?;

        assert_eq!(actual, Some(random_data_of_expected_packet_size));

        assert_eq!(buf.len(), 0);

        Ok(())
    }

    #[test]
    fn fixed_size_codec_deserialization_of_an_incomplete_byte_sequence_should_not_produce_an_item() -> anyhow::Result<()>
    {
        let mut codec = FixedLengthCodec::<TEST_SIZE>;
        let mut buf = tokio_util::bytes::BytesMut::new();

        const LESS_THAN_PAYLOAD_SIZE: usize = TEST_SIZE - 1;
        let random_data_too_few_bytes: Box<[u8]> =
            Box::from(hopr_crypto_random::random_bytes::<LESS_THAN_PAYLOAD_SIZE>());

        codec.encode(random_data_too_few_bytes, &mut buf)?;

        let actual = codec.decode(&mut buf)?;

        assert_eq!(actual, None);

        assert_eq!(buf.len(), LESS_THAN_PAYLOAD_SIZE);

        Ok(())
    }

    #[test]
    fn fixed_size_codec_deserialization_of_too_many_bytes_should_produce_the_value_from_only_the_bytes_needed_for_an_item()
    -> anyhow::Result<()> {
        let mut codec = FixedLengthCodec::<TEST_SIZE>;
        let mut buf = tokio_util::bytes::BytesMut::new();

        const MORE_THAN_PAYLOAD_SIZE: usize = TEST_SIZE + 1;
        let random_data_more_bytes_than_needed: Box<[u8]> =
            Box::from(hopr_crypto_random::random_bytes::<MORE_THAN_PAYLOAD_SIZE>());

        codec.encode(random_data_more_bytes_than_needed.clone(), &mut buf)?;

        let actual = codec.decode(&mut buf)?.context("The value should be available")?;

        assert_eq!(actual[..], random_data_more_bytes_than_needed[..TEST_SIZE]);

        assert_eq!(buf.len(), MORE_THAN_PAYLOAD_SIZE - TEST_SIZE);

        Ok(())
    }
}
