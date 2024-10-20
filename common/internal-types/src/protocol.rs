use crate::tickets::UnacknowledgedTicket;
use bloomfilter::Bloom;
use ethers::utils::hex;
use hopr_crypto_random::random_bytes;
use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use tracing::warn;

use crate::errors::{CoreTypesError, CoreTypesError::PayloadSizeExceeded, Result};

/// Number of intermediate hops: 3 relayers and 1 destination
pub const INTERMEDIATE_HOPS: usize = 3;

/// Maximum size of the packet payload in bytes.
pub const PAYLOAD_SIZE: usize = 500;

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

/// Bloom filter for packet tags to detect packet replays.
///
/// In addition, this structure also holds the number of items in the filter
/// to determine if the filter needs to be refreshed. Once this happens, packet replays
/// of past packets might be possible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagBloomFilter {
    bloom: Bloom<PacketTag>,
    count: usize,
}

impl TagBloomFilter {
    // Allowed false positive rate. This amounts to 0.01% chance
    const FALSE_POSITIVE_RATE: f64 = 0.0001_f64;

    // Maximum number of packet tags this Bloom filter can hold.
    // After this many packets, the Bloom filter resets and packet replays are possible.
    const MAX_ITEMS: usize = 10_000_000;

    /// Returns the current number of items in this Bloom filter.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Puts a packet tag into the Bloom filter
    pub fn set(&mut self, tag: &PacketTag) {
        if self.count == Self::MAX_ITEMS {
            warn!("maximum number of items in the Bloom filter reached!");
            self.bloom.clear();
            self.count = 0;
        }

        self.bloom.set(tag);
        self.count += 1;
    }

    /// Check if packet tag is in the Bloom filter.
    /// False positives are possible.
    pub fn check(&self, tag: &PacketTag) -> bool {
        self.bloom.check(tag)
    }

    /// Checks and sets a packet tag (if not present) in a single operation.
    pub fn check_and_set(&mut self, tag: &PacketTag) -> bool {
        if self.bloom.check_and_set(tag) {
            self.count += 1;
            true
        } else {
            false
        }
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data).map_err(|e| CoreTypesError::ParseError(e.to_string()))
    }

    pub fn to_bytes(&self) -> Box<[u8]> {
        bincode::serialize(&self)
            .expect("serialization must not fail")
            .into_boxed_slice()
    }
}

impl Default for TagBloomFilter {
    fn default() -> Self {
        Self {
            bloom: Bloom::new_for_fp_rate_with_seed(Self::MAX_ITEMS, Self::FALSE_POSITIVE_RATE, &random_bytes()),
            count: 0,
        }
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
            hex::encode(&self.plain_text)
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
            Err(GeneralError::ParseError)
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

        // Count number of items in the BF (incl. false positives)
        let match_count_1 = items.iter().filter(|item| filter1.check(item)).count();

        let filter2 = TagBloomFilter::from_bytes(&filter1.to_bytes())?;

        // Count number of items in the BF (incl. false positives)
        let match_count_2 = items.iter().filter(|item| filter2.check(item)).count();

        assert_eq!(
            match_count_1, match_count_2,
            "the number of false positives must be equal"
        );
        assert_eq!(filter1.count(), filter2.count(), "the number of items must be equal");

        // All zeroes must not be present in neither BF, we ensured we never insert a zero tag
        assert!(
            !filter1.check(&[0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8]),
            "bf 1 must not contain zero tag"
        );
        assert!(
            !filter2.check(&[0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8]),
            "bf 2 must not contain zero tag"
        );

        Ok(())
    }
}
