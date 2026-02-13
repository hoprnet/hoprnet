use hopr_crypto_random::random_bytes;
use hopr_crypto_types::types::PacketTag;

/// Bloom filter for packet tags to detect packet replays.
///
/// In addition, this structure also holds the number of items in the filter
/// to determine if the filter needs to be refreshed. Once this happens, packet replays
/// of past packets might be possible.
#[derive(Debug)]
pub struct TagBloomFilter {
    bloom: bloomfilter::Bloom<PacketTag>,
    count: usize,
    capacity: usize,
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
            self.bloom.clear();
            self.count = 0;
        }

        self.bloom.set(tag);
        self.count += 1;
    }

    /// Check if the packet tag is in the Bloom filter.
    ///
    /// Returns `true` if the given `tag` was already present in the filter.
    /// False positives are possible.
    pub fn check(&self, tag: &PacketTag) -> bool {
        self.bloom.check(tag)
    }

    /// Checks and sets a packet tag (if not present) in a single operation.
    ///
    /// Returns `true` if the given `tag` was already present in the filter.
    /// False positives are possible.
    pub fn check_and_set(&mut self, tag: &PacketTag) -> bool {
        // If we're at full capacity, we do only "check" and conditionally reset with the new entry
        if self.count == self.capacity {
            let is_present = self.bloom.check(tag);
            if !is_present {
                // There cannot be false negatives, so we can reset the filter
                tracing::warn!("maximum number of items in the Bloom filter reached!");
                self.bloom.clear();
                self.bloom.set(tag);
                self.count = 1;
            }
            is_present
        } else {
            // If not at full capacity, we can do check_and_set
            let was_present = self.bloom.check_and_set(tag);
            if !was_present {
                self.count += 1;
            }
            was_present
        }
    }

    fn with_capacity(size: usize) -> Self {
        Self {
            bloom: bloomfilter::Bloom::new_for_fp_rate_with_seed(size, Self::FALSE_POSITIVE_RATE, &random_bytes())
                .expect("bloom filter with the specified capacity is constructible"),
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
