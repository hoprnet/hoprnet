use bloomfilter::Bloom;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use tracing::warn;

use hopr_crypto_random::random_bytes;
use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;

use crate::errors::{CoreTypesError, CoreTypesError::PayloadSizeExceeded, Result};
use crate::tickets::UnacknowledgedTicket;

/// Number of intermediate hops: 3 relayers and 1 destination
pub const INTERMEDIATE_HOPS: usize = 3;

/// Maximum size of the packet payload in bytes.
pub const PAYLOAD_SIZE: usize = 496;

/// Default required minimum incoming ticket winning probability
pub const DEFAULT_MINIMUM_INCOMING_TICKET_WIN_PROB: f64 = 1.0;

/// Default maximum incoming ticket winning probability, above which tickets will not be accepted
/// due to privacy.
pub const DEFAULT_MAXIMUM_INCOMING_TICKET_WIN_PROB: f64 = 1.0; // TODO: change this in 3.0

/// Default ticket winning probability that will be printed on outgoing tickets
pub const DEFAULT_OUTGOING_TICKET_WIN_PROB: f64 = 1.0;

/// The lowest possible ticket winning probability due to SC representation limit.
pub const LOWEST_POSSIBLE_WINNING_PROB: f64 = 0.00000001;

/// Tags are currently 16-bit unsigned integers
pub type Tag = u16;

/// Represent a default application tag if none is specified in `send_packet`.
pub const DEFAULT_APPLICATION_TAG: Tag = 0;

/// Represents packet acknowledgement
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Acknowledgement {
    ack_signature: OffchainSignature,
    pub ack_key_share: HalfKey,
    validated: bool,
}

impl Acknowledgement {
    pub fn new(ack_key_share: HalfKey, node_keypair: &OffchainKeypair) -> Self {
        Self {
            ack_signature: OffchainSignature::sign_message(ack_key_share.as_ref(), node_keypair),
            ack_key_share,
            validated: true,
        }
    }

    /// Generates random, but still a valid acknowledgement.
    pub fn random(offchain_keypair: &OffchainKeypair) -> Self {
        Self::new(HalfKey::random(), offchain_keypair)
    }

    /// Validates the acknowledgement.
    ///
    /// Must be called immediately after deserialization, or otherwise
    /// any operations with the deserialized acknowledgement will panic.
    #[tracing::instrument(level = "debug", skip(self, sender_node_key))]
    pub fn validate(&mut self, sender_node_key: &OffchainPublicKey) -> bool {
        self.validated = self
            .ack_signature
            .verify_message(self.ack_key_share.as_ref(), sender_node_key);

        self.validated
    }

    /// Obtains the acknowledged challenge out of this acknowledgment.
    pub fn ack_challenge(&self) -> HalfKeyChallenge {
        assert!(self.validated, "acknowledgement not validated");
        self.ack_key_share.to_challenge()
    }
}

/// Contains either unacknowledged ticket if we're waiting for the acknowledgement as a relayer
/// or information if we wait for the acknowledgement as a sender.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PendingAcknowledgement {
    /// We're waiting for acknowledgement as a sender
    WaitingAsSender,
    /// We're waiting for the acknowledgement as a relayer with a ticket
    WaitingAsRelayer(UnacknowledgedTicket),
}

const TAGBLOOM_BINCODE_CONFIGURATION: bincode::config::Configuration = bincode::config::standard()
    .with_little_endian()
    .with_variable_int_encoding();

#[derive(Debug, Clone)]
struct SerializableBloomWrapper(Bloom<PacketTag>);

impl Serialize for SerializableBloomWrapper {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        bloomfilter::serialize(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for SerializableBloomWrapper {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        bloomfilter::deserialize(deserializer).map(Self)
    }
}

/// Bloom filter for packet tags to detect packet replays.
///
/// In addition, this structure also holds the number of items in the filter
/// to determine if the filter needs to be refreshed. Once this happens, packet replays
/// of past packets might be possible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagBloomFilter {
    bloom: SerializableBloomWrapper,
    count: usize,
    capacity: usize,
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

    /// Deserializes the filter from the given byte array.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        bincode::serde::borrow_decode_from_slice(data, TAGBLOOM_BINCODE_CONFIGURATION)
            .map(|(v, _bytes)| v)
            .map_err(|e| CoreTypesError::ParseError(e.to_string()))
    }

    /// Serializes the filter to the given byte array.
    pub fn to_bytes(&self) -> Box<[u8]> {
        bincode::serde::encode_to_vec(self, TAGBLOOM_BINCODE_CONFIGURATION)
            .expect("serialization of bloom filter must not fail")
            .into_boxed_slice()
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApplicationData {
    // TODO: 3.0: Remove the Option and replace with the Tag.
    pub application_tag: Option<Tag>,
    #[serde(with = "serde_bytes")]
    pub plain_text: Box<[u8]>,
}

impl ApplicationData {
    pub fn new(application_tag: Option<Tag>, plain_text: &[u8]) -> Result<Self> {
        if plain_text.len() <= PAYLOAD_SIZE - Self::SIZE {
            Ok(Self {
                application_tag,
                plain_text: plain_text.into(),
            })
        } else {
            Err(PayloadSizeExceeded)
        }
    }

    pub fn new_from_owned(application_tag: Option<Tag>, plain_text: Box<[u8]>) -> Result<Self> {
        if plain_text.len() <= PAYLOAD_SIZE - Self::SIZE {
            Ok(Self {
                application_tag,
                plain_text,
            })
        } else {
            Err(PayloadSizeExceeded)
        }
    }
}

impl Display for ApplicationData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}): {}",
            self.application_tag.unwrap_or(DEFAULT_APPLICATION_TAG),
            alloy::hex::encode(&self.plain_text)
        )
    }
}

impl ApplicationData {
    const SIZE: usize = 2; // minimum size

    pub fn from_bytes(data: &[u8]) -> hopr_primitive_types::errors::Result<Self> {
        if data.len() <= PAYLOAD_SIZE && data.len() >= Self::SIZE {
            let mut tag = [0u8; 2];
            tag.copy_from_slice(&data[0..2]);
            let tag = u16::from_be_bytes(tag);
            Ok(Self {
                application_tag: if tag != DEFAULT_APPLICATION_TAG {
                    Some(tag)
                } else {
                    None
                },
                plain_text: (&data[2..]).into(),
            })
        } else {
            Err(GeneralError::ParseError("ApplicationData".into()))
        }
    }

    pub fn to_bytes(&self) -> Box<[u8]> {
        let mut buf = Vec::with_capacity(Self::SIZE + self.plain_text.len());
        let tag = self.application_tag.unwrap_or(DEFAULT_APPLICATION_TAG);
        buf.extend_from_slice(&tag.to_be_bytes());
        buf.extend_from_slice(&self.plain_text);
        buf.into_boxed_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hopr_crypto_random::random_bytes;

    use hex_literal::hex;

    const PRIVATE_KEY: [u8; 32] = hex!("51d3003d908045a4d76d0bfc0d84f6ff946b5934b7ea6a2958faf02fead4567a");

    #[test]
    fn acknowledgement_binary_compatibility_with_the_v2_format() -> anyhow::Result<()> {
        let offchain_kp = OffchainKeypair::from_secret(&PRIVATE_KEY)?;
        let mut ack = Acknowledgement::new(HalfKey::default(), &offchain_kp);

        assert!(ack.validate(offchain_kp.public()));

        let buf = Vec::new();
        let serialized = cbor4ii::serde::to_vec(buf, &ack)?;

        const EXPECTED_V2_BINARY_REPRESENTATION_CBOR_HEX: [u8; 213] = hex!("a36d61636b5f7369676e6174757265a1697369676e617475726598401859182418be184818a218c318cb1869186018270218391853186c18ff18e018b518d9187b187900188218da184e1869187518ec1828181b081821187718bb0c18ba18f418331218ea187c1880182318d6189f189f18d7141876186a1890186b1885189718a718b9189018fc18bc18260918e318a5182a006d61636b5f6b65795f7368617265a164686b6579982000000000000000000000000000000000000000000000000000000000000000016976616c696461746564f5");

        assert_eq!(&serialized, &EXPECTED_V2_BINARY_REPRESENTATION_CBOR_HEX);

        Ok(())
    }

    #[test]
    fn test_application_data() -> anyhow::Result<()> {
        let ad_1 = ApplicationData::new(Some(10), &[0_u8, 1_u8])?;
        let ad_2 = ApplicationData::from_bytes(&ad_1.to_bytes())?;
        assert_eq!(ad_1, ad_2);

        let ad_1 = ApplicationData::new(None, &[])?;
        let ad_2 = ApplicationData::from_bytes(&ad_1.to_bytes())?;
        assert_eq!(ad_1, ad_2);

        let ad_1 = ApplicationData::new(Some(10), &[0_u8, 1_u8])?;
        let ad_2 = ApplicationData::from_bytes(&ad_1.to_bytes())?;
        assert_eq!(ad_1, ad_2);

        Ok(())
    }

    const ZEROS_TAG: [u8; PACKET_TAG_LENGTH] = [0; PACKET_TAG_LENGTH];
    const ONES_TAG: [u8; PACKET_TAG_LENGTH] = [1; PACKET_TAG_LENGTH];

    #[test]
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

        //let len = filter1.to_bytes().len();

        // Count the number of items in the BF (incl. false positives)
        let match_count_1 = items.iter().filter(|item| filter1.check(item)).count();

        let filter2 = TagBloomFilter::from_bytes(&filter1.to_bytes())?;

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
