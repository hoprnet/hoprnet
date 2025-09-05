use std::{borrow::Cow, fmt::Formatter, ops::Range, str::FromStr};

use hopr_crypto_packet::prelude::{HoprPacket, PacketSignals};
use hopr_primitive_types::to_hex_shortened;
use strum::IntoEnumIterator;

use crate::errors::ApplicationLayerError;

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
    /// The maximum value of a tag.
    ///
    /// The maximum value is determined by the fact that the 3 most significant bits
    /// must be set to 0 in version 1.
    pub const MAX: u64 = 0x1fffffffffffffff_u64;
    /// Size of a tag in bytes.
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
            Tag::Reserved(tag) | Tag::Application(tag) => (*tag) & Self::MAX,
        }
    }
}

impl<T: Into<u64>> From<T> for Tag {
    fn from(tag: T) -> Self {
        // In version 1, the 3 most significant bits are always 0.
        let tag: u64 = tag.into() & Self::MAX;

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
        serializer.serialize_u64(self.as_u64())
    }
}

#[cfg(feature = "serde")]
impl<'a> serde::Deserialize<'a> for Tag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        Ok(u64::deserialize(deserializer)?.into())
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

/// Holds packet transient information when [`ApplicationData`] is passed from the HOPR protocol layer to the
/// Application layer.
///
/// The HOPR protocol layer typically takes care of properly populating this structure
/// as the packet arrives.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct IncomingPacketInfo {
    /// Packet signals that were passed by the sender.
    pub signals_from_sender: PacketSignals,
    /// The number of SURBs the HOPR packet was carrying along with the [`ApplicationData`] instance.
    pub num_saved_surbs: usize,
}

/// Holds packet transient information when [`ApplicationData`] is passed to the HOPR protocol layer from the
/// Application layer.
///
/// The information passed to the HOPR protocol only serves as a suggestion, and the HOPR protocol
/// may choose to ignore it, based on its configuration.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct OutgoingPacketInfo {
    /// Packet signals that should be passed to the recipient.
    pub signals_to_destination: PacketSignals,
    /// The maximum number of SURBs the HOPR packet should be carrying when sent.
    pub max_surbs_in_packet: usize,
}

impl Default for OutgoingPacketInfo {
    fn default() -> Self {
        Self {
            signals_to_destination: PacketSignals::empty(),
            max_surbs_in_packet: usize::MAX,
        }
    }
}

/// Wrapper for incoming [`ApplicationData`] with optional [`IncomingPacketInfo`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApplicationDataIn {
    /// The actual application-layer data.
    pub data: ApplicationData,
    /// Additional transient information about the incoming packet.
    ///
    /// This information is always populated by the HOPR packet layer.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub packet_info: IncomingPacketInfo,
}

impl ApplicationDataIn {
    /// Returns how many SURBs were carried with this packet.
    pub fn num_surbs_with_msg(&self) -> usize {
        self.packet_info
            .num_saved_surbs
            .min(HoprPacket::max_surbs_with_message(self.data.total_len()))
    }
}

/// Wrapper for outgoing [`ApplicationData`] with optional [`IncomingPacketInfo`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApplicationDataOut {
    /// The actual application-layer data.
    pub data: ApplicationData,
    /// Additional transient information about the outgoing packet.
    ///
    /// This field is optional and acts mainly as a suggestion to the HOPR packet layer,
    /// as it may choose to completely ignore it, based on its configuration.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub packet_info: Option<OutgoingPacketInfo>,
}

impl ApplicationDataOut {
    /// Creates a new instance with `packet_info` set to `None`.
    pub fn with_no_packet_info(data: ApplicationData) -> Self {
        Self {
            data,
            packet_info: None,
        }
    }

    /// Returns the upper bound of how many SURBs that will be carried with this packet.
    pub fn estimate_surbs_with_msg(&self) -> usize {
        let max_possible = HoprPacket::max_surbs_with_message(self.data.total_len());
        self.packet_info
            .map(|info| info.max_surbs_in_packet.min(max_possible))
            .unwrap_or(max_possible)
    }
}

/// Represents the to-be-sent or received decrypted packet carrying the application-layer data.
///
/// This type is already HOPR specific, as it enforces the maximum payload size to be at most
/// [`HoprPacket::PAYLOAD_SIZE`] bytes-long. This structure always owns the data.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApplicationData {
    /// Tag identifying the application-layer protocol.
    pub application_tag: Tag,
    /// The actual application-layer data.
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))]
    pub plain_text: Box<[u8]>,
}

impl ApplicationData {
    /// The payload size is the [`HoprPacket::PAYLOAD_SIZE`] minus the [`Tag::SIZE`].
    pub const PAYLOAD_SIZE: usize = HoprPacket::PAYLOAD_SIZE - Tag::SIZE;

    /// Creates a new instance with the given tag and application layer data.
    ///
    /// Fails if the `plain_text` is larger than [`ApplicationData::PAYLOAD_SIZE`].
    pub fn new<'a, T: Into<Tag>, D: Into<Cow<'a, [u8]>>>(
        application_tag: T,
        plain_text: D,
    ) -> crate::errors::Result<Self> {
        let data = plain_text.into();
        if data.len() <= Self::PAYLOAD_SIZE {
            Ok(Self {
                application_tag: application_tag.into(),
                plain_text: data.into(),
            })
        } else {
            Err(ApplicationLayerError::PayloadTooLarge)
        }
    }

    /// Length of the payload plus the [`Tag`].
    ///
    /// Can never be zero due to the `Tag`.
    #[inline]
    pub fn total_len(&self) -> usize {
        Tag::SIZE + self.plain_text.len()
    }

    /// Indicates if the payload is empty.
    #[inline]
    pub fn is_payload_empty(&self) -> bool {
        self.plain_text.is_empty()
    }

    /// Serializes the structure into binary representation.
    pub fn to_bytes(&self) -> Box<[u8]> {
        let mut buf = Vec::with_capacity(Tag::SIZE + self.plain_text.len());
        buf.extend_from_slice(&self.application_tag.to_be_bytes());
        buf.extend_from_slice(&self.plain_text);
        buf.into_boxed_slice()
    }
}

impl std::fmt::Debug for ApplicationData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApplicationData")
            .field("application_tag", &self.application_tag)
            .field("plain_text", &to_hex_shortened::<32>(&self.plain_text))
            .finish()
    }
}

impl std::fmt::Display for ApplicationData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}): {}",
            self.application_tag,
            to_hex_shortened::<16>(&self.plain_text)
        )
    }
}

impl TryFrom<&[u8]> for ApplicationData {
    type Error = ApplicationLayerError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() >= Tag::SIZE && value.len() <= HoprPacket::PAYLOAD_SIZE {
            Ok(Self {
                application_tag: Tag::from_be_bytes(
                    value[0..Tag::SIZE]
                        .try_into()
                        .map_err(|_e| ApplicationLayerError::DecodingError("ApplicationData.tag".into()))?,
                ),
                plain_text: Box::from(&value[Tag::SIZE..]),
            })
        } else {
            Err(ApplicationLayerError::DecodingError("ApplicationData.size".into()))
        }
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
    fn v1_tags_should_have_3_most_significant_bits_unset() {
        let tag: Tag = u64::MAX.into();
        assert_eq!(tag.as_u64(), Tag::MAX);
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
        let original = ApplicationData::new(10u64, &[0_u8, 1_u8])?;
        let reserialized = ApplicationData::try_from(original.to_bytes().as_ref())?;
        let reserialized = ApplicationData::try_from(reserialized.to_bytes().as_ref())?;

        assert_eq!(original, reserialized);

        Ok(())
    }

    #[test]
    fn test_application_data() -> anyhow::Result<()> {
        let ad_1 = ApplicationData::new(10u64, &[0_u8, 1_u8])?;
        let ad_2 = ApplicationData::try_from(ad_1.to_bytes().as_ref())?;
        assert_eq!(ad_1, ad_2);

        let ad_1 = ApplicationData::new(0u64, &[])?;
        let ad_2 = ApplicationData::try_from(ad_1.to_bytes().as_ref())?;
        assert_eq!(ad_1, ad_2);

        let ad_1 = ApplicationData::new(10u64, &[0_u8, 1_u8])?;
        let ad_2 = ApplicationData::try_from(ad_1.to_bytes().as_ref())?;
        assert_eq!(ad_1, ad_2);

        let ad_1 = ApplicationData::new(10u64, &[0_u8; ApplicationData::PAYLOAD_SIZE])?;
        let ad_2 = ApplicationData::try_from(ad_1.to_bytes().as_ref())?;
        assert_eq!(ad_1, ad_2);

        assert!(ApplicationData::try_from([0_u8; Tag::SIZE - 1].as_ref()).is_err());
        assert!(ApplicationData::try_from([0_u8; ApplicationData::PAYLOAD_SIZE + Tag::SIZE + 1].as_ref()).is_err());

        Ok(())
    }

    #[test]
    fn application_data_should_not_allow_payload_larger_than_hopr_packet_payload_size() {
        assert!(ApplicationData::new(10u64, [0_u8; HoprPacket::PAYLOAD_SIZE + 1].as_ref()).is_err());
    }
}
