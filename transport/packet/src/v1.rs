use std::{
    fmt::Formatter,
    ops::{Add, Range, Sub},
};

/// Tags are represented as 6 bytes.
///
/// 2 ^ 48 should offer enough space to even avoid collisions and tag attacks.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tag(pub u64);

impl Tag {
    pub const MAX: Tag = Tag(u64::MAX);
    pub const SIZE: usize = size_of::<u64>();
    pub const USABLE_RANGE: Range<Tag> = ReservedTag::UPPER_BOUND..CustomTag::UPPER_BOUND;

    pub const fn new(tag: u64) -> Self {
        Self(tag)
    }

    pub fn from_be_bytes(bytes: [u8; Self::SIZE]) -> Self {
        Self(u64::from_be_bytes(bytes))
    }

    pub fn to_be_bytes(&self) -> [u8; Self::SIZE] {
        self.0.to_be_bytes()
    }
}

impl serde::Serialize for Tag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

impl<'a> serde::Deserialize<'a> for Tag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let value = u64::deserialize(deserializer)?;
        Ok(Self(value))
    }
}

impl Add for Tag {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl Sub for Tag {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tag({})", self.0)
    }
}

impl From<u64> for Tag {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<u32> for Tag {
    fn from(value: u32) -> Self {
        Self(value as u64)
    }
}

impl From<u8> for Tag {
    fn from(value: u8) -> Self {
        Self(value as u64)
    }
}

impl PartialEq<u64> for Tag {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<u64> for Tag {
    fn partial_cmp(&self, other: &u64) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

/// Resolved tag type with translated tag annotation.
#[repr(u64)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ResolvedTag {
    Reserved(ReservedTag),
    Custom(CustomTag),
}

impl From<ResolvedTag> for Tag {
    fn from(tag: ResolvedTag) -> Self {
        match tag {
            ResolvedTag::Reserved(reserved_tag) => reserved_tag.into(),
            ResolvedTag::Custom(custom_tag) => custom_tag.0,
        }
    }
}

impl From<Tag> for ResolvedTag {
    fn from(tag: Tag) -> Self {
        if tag < ReservedTag::UPPER_BOUND {
            match tag.0 {
                x if x == ReservedTag::SessionInit as u64 => ResolvedTag::Reserved(ReservedTag::SessionInit),
                x if x == ReservedTag::Ping as u64 => ResolvedTag::Reserved(ReservedTag::Ping),
                _ => ResolvedTag::Reserved(ReservedTag::Undefined),
            }
        } else {
            ResolvedTag::Custom(CustomTag(tag))
        }
    }
}

/// List of all reserved application tags for the protocol.
#[repr(u64)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]

pub enum ReservedTag {
    /// Opening a new session.
    SessionInit = 0,
    /// Ping traffic for 0-hop detection.
    Ping = 14,
    /// Undefined catch all.
    Undefined = Self::UPPER_BOUND.0 - 1,
}

impl ReservedTag {
    /// The upper limit value for the session reserved tag range.
    pub const UPPER_BOUND: Tag = Tag(1 << 4); // 16
}

impl From<ReservedTag> for Tag {
    fn from(tag: ReservedTag) -> Self {
        Self(tag as u64)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CustomTag(Tag);

impl CustomTag {
    pub const UPPER_BOUND: Tag = Tag(u64::MAX);
}

impl From<CustomTag> for Tag {
    fn from(tag: CustomTag) -> Self {
        tag.0
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
    pub const PAYLOAD_SIZE: usize = hopr_crypto_packet::prelude::HoprPacket::PAYLOAD_SIZE - size_of::<Tag>();

    pub fn new(application_tag: Tag, plain_text: &[u8]) -> Self {
        Self {
            application_tag,
            plain_text: plain_text.into(),
        }
    }

    pub fn new_from_owned(application_tag: Tag, plain_text: Box<[u8]>) -> Self {
        Self {
            application_tag,
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
    const TAG_SIZE: usize = size_of::<Tag>();

    // minimum size

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
    fn test_application_data() -> anyhow::Result<()> {
        let ad_1 = ApplicationData::new(10u64.into(), &[0_u8, 1_u8]);
        let ad_2 = ApplicationData::from_bytes(&ad_1.to_bytes())?;
        assert_eq!(ad_1, ad_2);

        let ad_1 = ApplicationData::new(0u64.into(), &[]);
        let ad_2 = ApplicationData::from_bytes(&ad_1.to_bytes())?;
        assert_eq!(ad_1, ad_2);

        let ad_1 = ApplicationData::new(10u64.into(), &[0_u8, 1_u8]);
        let ad_2 = ApplicationData::from_bytes(&ad_1.to_bytes())?;
        assert_eq!(ad_1, ad_2);

        Ok(())
    }
}
