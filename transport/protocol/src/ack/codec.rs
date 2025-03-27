use std::marker::PhantomData;

use tokio_util::bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use serde::{Deserialize, Serialize};

/// A codec for CBOR encoding and decoding using serde_cbor
///
/// Inspired by the [asynchronous_codec](`https://docs.rs/asynchronous-codec/latest/asynchronous_codec/`) crate
/// but better fitting this codebase.
///
/// TODO: Replace with cbor4ii

#[derive(Debug, thiserror::Error)]
pub enum CborCodecError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CBOR error: {0}")]
    Cbor(#[from] serde_cbor::Error),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CborCodec<T: Serialize + for<'de> Deserialize<'de>> {
    _phantom: PhantomData<T>,
}

impl<T: Serialize + for<'de> Deserialize<'de>> CborCodec<T> {
    /// Creates a new `CborCodec` with the associated types
    pub fn new() -> Self {
        Self { _phantom: PhantomData }
    }
}

/// Decode the type from bytes
impl<T: Serialize + for<'de> Deserialize<'de>> Decoder for CborCodec<T> {
    type Item = T;
    type Error = CborCodecError;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut de = serde_cbor::Deserializer::from_slice(&buf);

        let res: Result<T, _> = serde::de::Deserialize::deserialize(&mut de);

        let item = match res {
            Ok(item) => item,
            Err(e) => {
                if e.is_eof() {
                    return Ok(None);
                } else {
                    return Err(e.into());
                }
            }
        };

        let offset = de.byte_offset();

        buf.advance(offset);

        Ok(Some(item))
    }
}

/// Encoder impl encodes object streams to bytes
impl<T: Serialize + for<'de> Deserialize<'de>> Encoder<T> for CborCodec<T> {
    type Error = CborCodecError;

    fn encode(&mut self, data: T, buf: &mut BytesMut) -> Result<(), Self::Error> {
        let j = serde_cbor::to_vec(&data)?;

        buf.reserve(j.len());
        buf.put_slice(&j);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};

    use super::CborCodec;
    use super::{BytesMut, Decoder, Encoder};

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct TestStruct {
        pub name: String,
        pub data: u16,
    }

    #[test]
    fn cbor_codec_encode_decode_is_reversible() {
        let mut codec = CborCodec::<TestStruct>::new();
        let mut buff = BytesMut::new();

        let item1 = TestStruct {
            name: "Test name".to_owned(),
            data: 16,
        };
        codec.encode(item1.clone(), &mut buff).unwrap();

        let item2 = codec.decode(&mut buff).unwrap().unwrap();
        assert_eq!(item1, item2);

        assert_eq!(codec.decode(&mut buff).unwrap(), None);

        assert_eq!(buff.len(), 0);
    }

    #[test]
    fn cbor_codec_partial_decode() {
        let mut codec = CborCodec::<TestStruct>::new();
        let mut buff = BytesMut::new();

        let item1 = TestStruct {
            name: "Test name".to_owned(),
            data: 34,
        };
        codec.encode(item1, &mut buff).unwrap();

        let mut start = buff.clone().split_to(4);
        assert_eq!(codec.decode(&mut start).unwrap(), None);

        codec.decode(&mut buff).unwrap().unwrap();

        assert_eq!(buff.len(), 0);
    }

    #[test]
    fn cbor_codec_eof_reached() {
        let mut codec = CborCodec::<TestStruct>::new();
        let mut buff = BytesMut::new();

        let item1 = TestStruct {
            name: "Test name".to_owned(),
            data: 34,
        };
        codec.encode(item1.clone(), &mut buff).unwrap();

        // Split the buffer into two.
        let mut buff_start = buff.clone().split_to(4);
        let buff_end = buff.clone().split_off(4);

        // Attempt to decode the first half of the buffer. This should return `Ok(None)` and not
        // advance the buffer.
        assert_eq!(codec.decode(&mut buff_start).unwrap(), None);
        assert_eq!(buff_start.len(), 4);

        // Combine the buffer back together.
        buff_start.extend(buff_end.iter());

        // It should now decode successfully.
        let item2 = codec.decode(&mut buff).unwrap().unwrap();
        assert_eq!(item1, item2);
    }

    #[test]
    fn cbor_codec_decode_error() {
        let mut codec = CborCodec::<TestStruct>::new();
        let mut buff = BytesMut::new();

        let item1 = TestStruct {
            name: "Test name".to_owned(),
            data: 34,
        };
        codec.encode(item1.clone(), &mut buff).unwrap();

        // Split the end off the buffer.
        let mut buff_end = buff.clone().split_off(4);
        let buff_end_length = buff_end.len();

        // Attempting to decode should return an error.
        assert!(codec.decode(&mut buff_end).is_err());
        assert_eq!(buff_end.len(), buff_end_length);
    }
}
