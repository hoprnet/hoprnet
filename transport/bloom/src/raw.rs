use bloomfilter::Bloom;
use hopr_crypto_random::random_bytes;
use hopr_crypto_types::types::PacketTag;

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
    // The default maximum number of packet tags this Bloom filter can hold.
    // After these many packets, the Bloom filter resets and packet replays are possible.
    const DEFAULT_MAX_ITEMS: usize = 10_000_000;
    // Allowed false positive rate. This amounts to 0.001% chance
    const FALSE_POSITIVE_RATE: f64 = 0.00001_f64;

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
            tracing::warn!("maximum number of items in the Bloom filter reached!");
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
                tracing::warn!("maximum number of items in the Bloom filter reached!");
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

#[cfg(test)]
mod tests {
    use hopr_crypto_types::types::PACKET_TAG_LENGTH;

    use super::*;

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
