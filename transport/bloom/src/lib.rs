/// Bloom filter capable of persisting to disk
#[cfg(feature = "persistent")]
pub mod persistent;

/// Raw bloom filter adapted for HOPR packet tags
pub mod raw;

#[cfg(feature = "persistent")]
pub use persistent::WrappedTagBloomFilter;
pub use raw::TagBloomFilter;
