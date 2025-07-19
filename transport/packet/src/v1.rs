use std::{fmt::Formatter, ops::Range};

use strum::IntoEnumIterator;

/// List of all reserved application tags for the protocol.
#[repr(u64)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumIter)]
pub enum ReservedTag {
    /// Ping traffic for 0-hop detection.
    Ping = 0,

    /// Commands associated with session start protocol regarding session initiation.
    SessionStart = 1,

    /// Undefined catch all.
    Undefined = 15,
}

impl ReservedTag {
    /// The range of reserved tags
    pub fn range() -> Range<u64> {
        0..(Self::iter().max().unwrap_or(Self::Undefined) as u64 + 1)
    }
}

impl From<ReservedTag> for Tag {
    fn from(tag: ReservedTag) -> Self {
        (tag as u64).into()
    }
}

/// Tags are represented by 8 bytes ([`u64`]`).
///
/// [`u64`] should offer enough space to avoid collisions and tag attacks.
// #[repr(u64)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tag {
    Reserved(u64),
    Application(u64),
}

impl Tag {
    /// Application tag range for external usage
    pub const APPLICATION_TAG_RANGE: Range<Self> =
        (Self::Application(ReservedTag::Undefined as u64 + 1))..Self::Application(Self::MAX);
    pub const MAX: u64 = u64::MAX;
    pub const SIZE: usize = size_of::<u64>();

    pub fn from_be_bytes(bytes: [u8; Self::SIZE]) -> Self {
        let tag = u64::from_be_bytes(bytes);
        tag.into()
    }

    pub fn to_be_bytes(&self) -> [u8; Self::SIZE] {
        match self {
            Tag::Reserved(tag) | Tag::Application(tag) => tag.to_be_bytes(),
        }
    }

    pub fn as_u64(&self) -> u64 {
        match self {
            Tag::Reserved(tag) | Tag::Application(tag) => *tag,
        }
    }
}

impl<T: Into<u64>> From<T> for Tag {
    fn from(tag: T) -> Self {
        let tag: u64 = tag.into();

        if ReservedTag::range().contains(&tag) {
            Tag::Reserved(
                ReservedTag::iter()
                    .find(|&t| t as u64 == tag)
                    .unwrap_or(ReservedTag::Undefined) as u64,
            )
        } else {
            Tag::Application(tag)
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Tag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Tag::Reserved(tag) | Tag::Application(tag) => serializer.serialize_u64(*tag),
        }
    }
}

#[cfg(feature = "serde")]
impl<'a> serde::Deserialize<'a> for Tag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let value = u64::deserialize(deserializer)?;

        Ok(value.into())
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_u64())
    }
}

/// Represents the received decrypted packet carrying the application-layer data.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApplicationData {
    pub application_tag: Tag,
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))]
    pub plain_text: Box<[u8]>,
}

impl ApplicationData {
    pub const PAYLOAD_SIZE: usize = hopr_crypto_packet::prelude::HoprPacket::PAYLOAD_SIZE - Tag::SIZE;

    pub fn new<T: Into<Tag>>(application_tag: T, plain_text: &[u8]) -> Self {
        Self {
            application_tag: application_tag.into(),
            plain_text: plain_text.into(),
        }
    }

    pub fn new_from_owned<T: Into<Tag>>(tag: T, plain_text: Box<[u8]>) -> Self {
        Self {
            application_tag: tag.into(),
            plain_text,
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        Self::TAG_SIZE + self.plain_text.len()
    }
}

impl std::fmt::Debug for ApplicationData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApplicationData")
            .field("application_tag", &self.application_tag)
            .finish()
    }
}

impl std::fmt::Display for ApplicationData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}): {}", self.application_tag, hex::encode(&self.plain_text))
    }
}

impl ApplicationData {
    const TAG_SIZE: usize = Tag::SIZE;

    pub fn from_bytes(data: &[u8]) -> crate::errors::Result<Self> {
        if data.len() >= Self::TAG_SIZE {
            Ok(Self {
                application_tag: Tag::from_be_bytes(
                    data[0..Self::TAG_SIZE]
                        .try_into()
                        .map_err(|_e| crate::errors::PacketError::DecodingError("ApplicationData.tag".into()))?,
                ),
                plain_text: Box::from(&data[Self::TAG_SIZE..]),
            })
        } else {
            Err(crate::errors::PacketError::DecodingError("ApplicationData".into()))
        }
    }

    pub fn to_bytes(&self) -> Box<[u8]> {
        let mut buf = Vec::with_capacity(Self::TAG_SIZE + self.plain_text.len());
        buf.extend_from_slice(&self.application_tag.to_be_bytes());
        buf.extend_from_slice(&self.plain_text);
        buf.into_boxed_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_tag_v1_range_is_stable() {
        let range = ReservedTag::range();
        assert_eq!(range.start, 0);
        assert_eq!(range.count(), 16); // 0 to 15 inclusive
    }

    #[test]
    fn tag_should_be_obtainable_as_reserved_when_created_from_a_reserved_range() {
        let reserved_tag = ReservedTag::Ping as u64;

        assert_eq!(Tag::from(reserved_tag), Tag::Reserved(reserved_tag));
    }

    #[test]
    fn tag_should_be_obtainable_as_undefined_reserved_when_created_from_an_undefined_value_in_reserved_range() {
        let reserved_tag_without_assignment = 7u64;

        assert_eq!(
            Tag::from(reserved_tag_without_assignment),
            Tag::Reserved(ReservedTag::Undefined as u64)
        );
    }

    #[test]
    fn v1_format_is_binary_stable() -> anyhow::Result<()> {
        let original = ApplicationData::new(10u64, &[0_u8, 1_u8]);
        let reserialized = ApplicationData::from_bytes(&original.to_bytes())?;
        let reserialized = ApplicationData::from_bytes(&reserialized.to_bytes())?;

        assert_eq!(original, reserialized);

        Ok(())
    }

    #[test]
    fn test_application_data() -> anyhow::Result<()> {
        let ad_1 = ApplicationData::new(10u64, &[0_u8, 1_u8]);
        let ad_2 = ApplicationData::from_bytes(&ad_1.to_bytes())?;
        assert_eq!(ad_1, ad_2);

        let ad_1 = ApplicationData::new(0u64, &[]);
        let ad_2 = ApplicationData::from_bytes(&ad_1.to_bytes())?;
        assert_eq!(ad_1, ad_2);

        let ad_1 = ApplicationData::new(10u64, &[0_u8, 1_u8]);
        let ad_2 = ApplicationData::from_bytes(&ad_1.to_bytes())?;
        assert_eq!(ad_1, ad_2);

        Ok(())
    }
}
