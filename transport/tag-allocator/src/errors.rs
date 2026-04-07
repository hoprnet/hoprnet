use crate::Usage;

/// Errors returned by [`crate::create_allocators`].
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum TagAllocatorError {
    /// The sum of requested partition capacities exceeds the available range.
    #[error("partition capacities ({total_requested}) exceed range length ({range_size})")]
    CapacityExceedsRange { total_requested: u64, range_size: u64 },
    /// The supplied range is empty.
    #[error("tag range is empty")]
    EmptyRange,
    /// A partition was requested with zero capacity.
    #[error("partition {0:?} has zero capacity")]
    ZeroCapacity(Usage),
}
