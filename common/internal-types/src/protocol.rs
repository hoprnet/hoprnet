use bloomfilter::Bloom;
use hopr_crypto_random::{random_bytes, Randomizable};
use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
use std::fmt::{Display, Formatter};
use tracing::warn;

use crate::errors::{CoreTypesError, Result};
use crate::prelude::UnacknowledgedTicket;

/// Number of intermediate hops: 3 relayers and 1 destination
pub const INTERMEDIATE_HOPS: usize = 3;

/// Default required minimum incoming ticket winning probability
pub const DEFAULT_MINIMUM_INCOMING_TICKET_WIN_PROB: f64 = 1.0;

/// Default maximum incoming ticket winning probability, above which tickets will not be accepted
/// due to privacy.
pub const DEFAULT_MAXIMUM_INCOMING_TICKET_WIN_PROB: f64 = 1.0; // TODO: change this in 3.0

/// Tags are currently 16-bit unsigned integers
pub type Tag = u16;

/// Represent a default application tag if none is specified in `send_packet`.
pub const DEFAULT_APPLICATION_TAG: Tag = 0;

/// Alias for the [`Pseudonym`](`hopr_crypto_types::types::Pseudonym`) used in the HOPR protocol.
pub type HoprPseudonym = SimplePseudonym;

/// Represents packet acknowledgement
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Acknowledgement {
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))]
    data: [u8; Self::SIZE],
    #[cfg_attr(feature = "serde", serde(skip))]
    validated: bool,
}

impl AsRef<[u8]> for Acknowledgement {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl TryFrom<&[u8]> for Acknowledgement {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        if value.len() == Self::SIZE {
            Ok(Self {
                data: value.try_into().unwrap(),
                validated: false,
            })
        } else {
            Err(GeneralError::ParseError("Acknowledgement".into()))
        }
    }
}

impl Acknowledgement {
    pub fn new(ack_key_share: HalfKey, node_keypair: &OffchainKeypair) -> Self {
        let signature = OffchainSignature::sign_message(ack_key_share.as_ref(), node_keypair);
        let mut data = [0u8; Self::SIZE];
        data[0..HalfKey::SIZE].copy_from_slice(ack_key_share.as_ref());
        data[HalfKey::SIZE..HalfKey::SIZE + OffchainSignature::SIZE].copy_from_slice(signature.as_ref());

        Self { data, validated: true }
    }

    /// Generates random but still a valid acknowledgement.
    pub fn random(offchain_keypair: &OffchainKeypair) -> Self {
        Self::new(HalfKey::random(), offchain_keypair)
    }

    /// Validates the acknowledgement.
    ///
    /// Must be called immediately after deserialization, or otherwise
    /// any operations with the deserialized acknowledgement return an error.
    #[tracing::instrument(level = "debug", skip(self, sender_node_key))]
    pub fn validate(self, sender_node_key: &OffchainPublicKey) -> Result<Self> {
        if !self.validated {
            let signature =
                OffchainSignature::try_from(&self.data[HalfKey::SIZE..HalfKey::SIZE + OffchainSignature::SIZE])?;
            if signature.verify_message(&self.data[0..HalfKey::SIZE], sender_node_key) {
                Ok(Self {
                    data: self.data,
                    validated: true,
                })
            } else {
                Err(CoreTypesError::InvalidAcknowledgement)
            }
        } else {
            Ok(self)
        }
    }

    /// Gets the acknowledged key out of this acknowledgement.
    ///
    /// Returns [`InvalidAcknowledgement`]
    /// if the acknowledgement has not been [validated](Acknowledgement::validate).
    pub fn ack_key_share(&self) -> Result<HalfKey> {
        if self.validated {
            Ok(HalfKey::try_from(&self.data[0..HalfKey::SIZE])?)
        } else {
            Err(CoreTypesError::InvalidAcknowledgement)
        }
    }

    /// Gets the acknowledgement challenge out of this acknowledgement.
    ///
    /// Returns [`InvalidAcknowledgement`]
    /// if the acknowledgement has not been [validated](Acknowledgement::validate).
    pub fn ack_challenge(&self) -> Result<HalfKeyChallenge> {
        Ok(self.ack_key_share()?.to_challenge())
    }

    /// Indicates whether the acknowledgement has been [validated](Acknowledgement::validate).
    pub fn is_validated(&self) -> bool {
        self.validated
    }
}

impl BytesRepresentable for Acknowledgement {
    const SIZE: usize = HalfKey::SIZE + OffchainSignature::SIZE;
}

/// Contains either unacknowledged ticket if we're waiting for the acknowledgement as a relayer
/// or information if we wait for the acknowledgement as a sender.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PendingAcknowledgement {
    /// We're waiting for acknowledgement as a sender
    WaitingAsSender,
    /// We're waiting for the acknowledgement as a relayer with a ticket
    WaitingAsRelayer(UnacknowledgedTicket),
}

/// Bloom filter for packet tags to detect packet replays.
///
/// In addition, this structure also holds the number of items in the filter
/// to determine if the filter needs to be refreshed. Once this happens, packet replays
/// of past packets might be possible.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TagBloomFilter {
    bloom: SerializableBloomWrapper,
    count: usize,
    capacity: usize,
}

#[derive(Debug, Clone)]
struct SerializableBloomWrapper(Bloom<PacketTag>);

#[cfg(feature = "serde")]
impl serde::Serialize for SerializableBloomWrapper {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        bloomfilter::serialize(&self.0, serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for SerializableBloomWrapper {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        bloomfilter::deserialize(deserializer).map(Self)
    }
}

impl TagBloomFilter {
    // Allowed false positive rate. This amounts to 0.001% chance
    const FALSE_POSITIVE_RATE: f64 = 0.00001_f64;

    // The default maximum number of packet tags this Bloom filter can hold.
    // After these many packets, the Bloom filter resets and packet replays are possible.
    const DEFAULT_MAX_ITEMS: usize = 10_000_000;

    /// Returns the current number of items in this Bloom filter.
    pub fn count(&self) -> usize {
        self.count
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Puts a packet tag into the Bloom filter
    pub fn set(&mut self, tag: &PacketTag) {
        if self.count == self.capacity {
            warn!("maximum number of items in the Bloom filter reached!");
            self.bloom.0.clear();
            self.count = 0;
        }

        self.bloom.0.set(tag);
        self.count += 1;
    }

    /// Check if the packet tag is in the Bloom filter.
    /// False positives are possible.
    pub fn check(&self, tag: &PacketTag) -> bool {
        self.bloom.0.check(tag)
    }

    /// Checks and sets a packet tag (if not present) in a single operation.
    pub fn check_and_set(&mut self, tag: &PacketTag) -> bool {
        // If we're at full capacity, we do only "check" and conditionally reset with the new entry
        if self.count == self.capacity {
            let is_present = self.bloom.0.check(tag);
            if !is_present {
                // There cannot be false negatives, so we can reset the filter
                warn!("maximum number of items in the Bloom filter reached!");
                self.bloom.0.clear();
                self.bloom.0.set(tag);
                self.count = 1;
            }
            is_present
        } else {
            // If not at full capacity, we can do check_and_set
            let was_present = self.bloom.0.check_and_set(tag);
            if !was_present {
                self.count += 1;
            }
            was_present
        }
    }

    fn with_capacity(size: usize) -> Self {
        Self {
            bloom: SerializableBloomWrapper(
                Bloom::new_for_fp_rate_with_seed(size, Self::FALSE_POSITIVE_RATE, &random_bytes())
                    .expect("bloom filter with the specified capacity is constructible"),
            ),
            count: 0,
            capacity: size,
        }
    }
}

impl Default for TagBloomFilter {
    fn default() -> Self {
        Self::with_capacity(Self::DEFAULT_MAX_ITEMS)
    }
}

/// Represents the received decrypted packet carrying the application-layer data.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApplicationData {
    pub application_tag: Tag,
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))]
    pub plain_text: Box<[u8]>,
}

impl ApplicationData {
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

impl Display for ApplicationData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}): {}", self.application_tag, hex::encode(&self.plain_text))
    }
}

impl ApplicationData {
    const TAG_SIZE: usize = size_of::<Tag>(); // minimum size

    pub fn from_bytes(data: &[u8]) -> hopr_primitive_types::errors::Result<Self> {
        if data.len() >= Self::TAG_SIZE {
            Ok(Self {
                application_tag: Tag::from_be_bytes(
                    data[0..Self::TAG_SIZE]
                        .try_into()
                        .map_err(|_| GeneralError::ParseError("ApplicationData.tag".into()))?,
                ),
                plain_text: Box::from(&data[Self::TAG_SIZE..]),
            })
        } else {
            Err(GeneralError::ParseError("ApplicationData".into()))
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
        let ad_1 = ApplicationData::new(10, &[0_u8, 1_u8]);
        let ad_2 = ApplicationData::from_bytes(&ad_1.to_bytes())?;
        assert_eq!(ad_1, ad_2);

        let ad_1 = ApplicationData::new(0, &[]);
        let ad_2 = ApplicationData::from_bytes(&ad_1.to_bytes())?;
        assert_eq!(ad_1, ad_2);

        let ad_1 = ApplicationData::new(10, &[0_u8, 1_u8]);
        let ad_2 = ApplicationData::from_bytes(&ad_1.to_bytes())?;
        assert_eq!(ad_1, ad_2);

        Ok(())
    }

    const ZEROS_TAG: [u8; PACKET_TAG_LENGTH] = [0; PACKET_TAG_LENGTH];
    const ONES_TAG: [u8; PACKET_TAG_LENGTH] = [1; PACKET_TAG_LENGTH];

    #[cfg(feature = "serde")]
    const TAGBLOOM_BINCODE_CONFIGURATION: bincode::config::Configuration = bincode::config::standard()
        .with_little_endian()
        .with_variable_int_encoding();

    #[test]
    #[cfg(feature = "serde")]
    fn test_packet_tag_bloom_filter() -> anyhow::Result<()> {
        let mut filter1 = TagBloomFilter::default();

        let items = (0..10_000)
            .map(|i| {
                let mut ret = random_bytes::<{ hopr_crypto_types::types::PACKET_TAG_LENGTH }>();
                ret[i % hopr_crypto_types::types::PACKET_TAG_LENGTH] = 0xaa; // ensure it is not completely just zeroes
                ret
            })
            .collect::<Vec<_>>();

        // Insert items into the BF
        items.iter().for_each(|item| filter1.set(item));

        assert_eq!(items.len(), filter1.count(), "invalid number of items in bf");

        // Count the number of items in the BF (incl. false positives)
        let match_count_1 = items.iter().filter(|item| filter1.check(item)).count();

        let filter2: TagBloomFilter = bincode::serde::decode_from_slice(
            &bincode::serde::encode_to_vec(&filter1, TAGBLOOM_BINCODE_CONFIGURATION)?,
            TAGBLOOM_BINCODE_CONFIGURATION,
        )?
        .0;

        // Count the number of items in the BF (incl. false positives)
        let match_count_2 = items.iter().filter(|item| filter2.check(item)).count();

        assert_eq!(
            match_count_1, match_count_2,
            "the number of false positives must be equal"
        );
        assert_eq!(filter1.count(), filter2.count(), "the number of items must be equal");

        // All zeroes must be present in neither BF; we ensured we never insert a zero tag
        assert!(!filter1.check(&ZEROS_TAG), "bf 1 must not contain zero tag");
        assert!(!filter2.check(&ZEROS_TAG), "bf 2 must not contain zero tag");

        Ok(())
    }

    #[test]
    fn tag_bloom_filter_count() {
        let mut filter = TagBloomFilter::default();
        assert!(!filter.check_and_set(&ZEROS_TAG));
        assert_eq!(1, filter.count());

        assert!(filter.check_and_set(&ZEROS_TAG));
        assert_eq!(1, filter.count());

        assert!(!filter.check_and_set(&ONES_TAG));
        assert_eq!(2, filter.count());

        assert!(filter.check_and_set(&ZEROS_TAG));
        assert_eq!(2, filter.count());
    }

    #[test]
    fn tag_bloom_filter_wrap_around() {
        let mut filter = TagBloomFilter::with_capacity(1000);
        for _ in 1..filter.capacity() {
            let mut tag: PacketTag = hopr_crypto_random::random_bytes();
            tag[0] = 0xaa; // ensure it's not all zeroes
            assert!(!filter.check_and_set(&tag));
        }
        // Not yet at full capacity
        assert_eq!(filter.capacity() - 1, filter.count());

        // This entry is not there yet
        assert!(!filter.check_and_set(&ZEROS_TAG));

        // Now the filter is at capacity and contains the previously inserted entry
        assert_eq!(filter.capacity(), filter.count());
        assert!(filter.check(&ZEROS_TAG));

        // This will not clear out the filter, since the entry is there
        assert!(filter.check_and_set(&ZEROS_TAG));
        assert_eq!(filter.capacity(), filter.count());

        // This will clear out the filter, since this other entry is definitely not there
        assert!(!filter.check_and_set(&ONES_TAG));
        assert_eq!(1, filter.count());
        assert!(filter.check(&ONES_TAG));
    }
}
