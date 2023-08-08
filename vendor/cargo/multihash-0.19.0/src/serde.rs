//! Multihash Serde (de)serialization

use std::fmt;

use serde::{
    de::{self, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::Multihash;

impl<const SIZE: usize> Serialize for Multihash<SIZE> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.to_bytes())
    }
}

struct BytesVisitor<const SIZE: usize>;

impl<'de, const SIZE: usize> Visitor<'de> for BytesVisitor<SIZE> {
    type Value = Multihash<SIZE>;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "a valid Multihash in bytes")
    }

    fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Multihash::<SIZE>::from_bytes(bytes)
            .map_err(|err| de::Error::custom(format!("Failed to deserialize Multihash: {}", err)))
    }

    // Some Serde data formats interpret a byte stream as a sequence of bytes (e.g. `serde_json`).
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut bytes = Vec::new();
        while let Some(byte) = seq.next_element()? {
            bytes.push(byte);
        }
        Multihash::<SIZE>::from_bytes(&bytes)
            .map_err(|err| de::Error::custom(format!("Failed to deserialize Multihash: {}", err)))
    }
}

impl<'de, const SIZE: usize> Deserialize<'de> for Multihash<SIZE> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(BytesVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_test::{assert_tokens, Token};

    const SHA2_256_CODE: u64 = 0x12;
    const DIGEST: [u8; 32] = [
        159, 228, 204, 198, 222, 22, 114, 79, 58, 48, 199, 232, 242, 84, 243, 198, 71, 25, 134,
        172, 177, 248, 216, 207, 142, 150, 206, 42, 215, 219, 231, 251,
    ];

    #[test]
    fn test_serde_json() {
        // This is a concatenation of `SHA2_256_CODE + DIGEST_LENGTH + DIGEST`.
        let expected_json = format!("[{},{},159,228,204,198,222,22,114,79,58,48,199,232,242,84,243,198,71,25,134,172,177,248,216,207,142,150,206,42,215,219,231,251]", SHA2_256_CODE as u8, DIGEST.len() as u8);

        let mh = Multihash::<32>::wrap(SHA2_256_CODE, &DIGEST).unwrap();

        let json = serde_json::to_string(&mh).unwrap();
        assert_eq!(json, expected_json);

        let mh_decoded: Multihash<32> = serde_json::from_str(&json).unwrap();
        assert_eq!(mh, mh_decoded);
    }

    #[test]
    fn test_serde_test() {
        // This is a concatenation of `SHA2_256_CODE + DIGEST_LENGTH + DIGEST`.
        const ENCODED_MULTIHASH_BYTES: [u8; 34] = [
            SHA2_256_CODE as u8,
            DIGEST.len() as u8,
            159,
            228,
            204,
            198,
            222,
            22,
            114,
            79,
            58,
            48,
            199,
            232,
            242,
            84,
            243,
            198,
            71,
            25,
            134,
            172,
            177,
            248,
            216,
            207,
            142,
            150,
            206,
            42,
            215,
            219,
            231,
            251,
        ];

        let mh = Multihash::<32>::wrap(SHA2_256_CODE, &DIGEST).unwrap();

        // As bytes.
        assert_tokens(&mh, &[Token::Bytes(&ENCODED_MULTIHASH_BYTES)]);

        // As sequence.
        serde_test::assert_de_tokens(
            &mh,
            &[
                Token::Seq { len: Some(34) },
                Token::U8(SHA2_256_CODE as u8),
                Token::U8(DIGEST.len() as u8),
                Token::U8(159),
                Token::U8(228),
                Token::U8(204),
                Token::U8(198),
                Token::U8(222),
                Token::U8(22),
                Token::U8(114),
                Token::U8(79),
                Token::U8(58),
                Token::U8(48),
                Token::U8(199),
                Token::U8(232),
                Token::U8(242),
                Token::U8(84),
                Token::U8(243),
                Token::U8(198),
                Token::U8(71),
                Token::U8(25),
                Token::U8(134),
                Token::U8(172),
                Token::U8(177),
                Token::U8(248),
                Token::U8(216),
                Token::U8(207),
                Token::U8(142),
                Token::U8(150),
                Token::U8(206),
                Token::U8(42),
                Token::U8(215),
                Token::U8(219),
                Token::U8(231),
                Token::U8(251),
                Token::SeqEnd,
            ],
        );
    }
}
