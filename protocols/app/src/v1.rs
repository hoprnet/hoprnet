use std::{fmt::Formatter, ops::Range, str::FromStr};

use hopr_crypto_packet::prelude::HoprPacket;
use hopr_primitive_types::to_hex_shortened;
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

/// Tags distinguishing different application-layer protocols.
///
/// Currently, 8 bytes represent tags (`u64`).
///
/// `u64` should offer enough space to avoid collisions and tag attacks.
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

impl FromStr for Tag {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u64::from_str(s).map(Tag::from)
    }
}

flagset::flags! {
   /// Individual flags passed up from the HOPR protocol layer to the Application layer.
   ///
   /// The upper 4 bits are reserved for signaling by the Application layer to the HOPR protocol layer
   /// when sending data, the lower 4 bits are reserved for signaling of the HOPR protocol layer
   /// to the Application layer when receiving data.
    #[repr(u8)]
    #[derive(PartialOrd, Ord, strum::EnumString, strum::Display)]
   pub enum ApplicationFlag: u8 {
        /// The other party is in a "SURB distress" state, potentially running out of SURBs soon.
        SurbDistress = 0b0000_0001,
        /// The other party has run out of SURBs, and this was potentially the last message they could
        /// send.
        ///
        /// Implies [`SurbDistress`].
        OutOfSurbs = 0b0000_0011,
   }
}

/// Additional flags passed between the HOPR protocol layer and the Application layer.
pub type ApplicationFlags = flagset::FlagSet<ApplicationFlag>;

/// Represents the received decrypted packet carrying the application-layer data.
///
/// This structure always owns the data.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApplicationData {
    /// Tag identifying the application-layer protocol.
    pub application_tag: Tag,
    /// The actual application-layer data.
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))]
    pub plain_text: Box<[u8]>,
    /// Additional flags passed to/from the HOPR protocol layer and the Application layer, serving
    /// as a means of local signaling.
    ///
    /// For outgoing data, these flags can be used to signal certain modes of operation by the Application layer
    /// to the HOPR protocol.
    /// For incoming data, the flags can be set by the HOPR protocol to signal various states to the
    /// Applications layer.
    ///
    /// The flags are never serialized nor deserialized. Whether these flags are eventually
    /// materialized over the wire is decided solely by the HOPR protocol layer underneath.
    /// The HOPR protocol calls such flags "packet signals".
    #[cfg_attr(feature = "serde", serde(skip))]
    pub flags: ApplicationFlags,
    #[cfg_attr(feature = "serde", serde(skip))]
    _d: u8, // prevents bypassing the constructor
}

impl ApplicationData {
    pub const PAYLOAD_SIZE: usize = HoprPacket::PAYLOAD_SIZE - Tag::SIZE;

    pub fn new<T: Into<Tag>>(application_tag: T, plain_text: &[u8]) -> Self {
        Self {
            application_tag: application_tag.into(),
            plain_text: plain_text.into(),
            flags: ApplicationFlags::empty(),
            _d: 0,
        }
    }

    pub fn new_from_owned<T: Into<Tag>>(tag: T, plain_text: Box<[u8]>) -> Self {
        Self {
            application_tag: tag.into(),
            plain_text,
            flags: ApplicationFlags::empty(),
            _d: 0,
        }
    }

    /// Creates a new instance with the given `flags` set.
    pub fn with_flags<F: Into<ApplicationFlags>>(mut self, flags: F) -> Self {
        self.flags = flags.into();
        self
    }

    #[inline]
    pub fn len(&self) -> usize {
        Self::TAG_SIZE + self.plain_text.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.plain_text.is_empty()
    }

    /// Returns the estimated number of SURBs the HOPR packet carrying an `ApplicationData` instance
    /// with `payload` could hold (or could have held).
    pub fn estimate_surbs_with_msg<T: AsRef<[u8]>>(payload: &T) -> usize {
        HoprPacket::max_surbs_with_message(payload.as_ref().len().min(Self::PAYLOAD_SIZE) + Tag::SIZE)
    }
}

impl std::fmt::Debug for ApplicationData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApplicationData")
            .field("application_tag", &self.application_tag)
            .field("plain_text", &to_hex_shortened(&self.plain_text, 32))
            .finish()
    }
}

impl std::fmt::Display for ApplicationData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}): {}",
            self.application_tag,
            to_hex_shortened(&self.plain_text, 16)
        )
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
                flags: ApplicationFlags::empty(),
                _d: 0,
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
