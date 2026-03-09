mod allocated_tag;
mod allocator;
mod stack;

use std::{fmt, ops::Range, sync::Arc};

pub use allocated_tag::AllocatedTag;

/// Identifies which component a partition belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Usage {
    /// Long-lived session tags (e.g. hundreds).
    Session,
    /// Session terminal telemetry tags (e.g. hundreds to low thousands).
    SessionTerminalTelemetry,
    /// Probing telemetry tags — high volume, short-lived (e.g. configurable via env, default ~1000).
    ProvingTelemetry,
}

/// Errors returned by [`create_allocators`].
#[derive(Debug, PartialEq, Eq)]
pub enum TagAllocatorError {
    /// The sum of requested partition capacities exceeds the available range.
    CapacityExceedsRange { total_requested: u64, range_size: u64 },
    /// The supplied range is empty.
    EmptyRange,
    /// A partition was requested with zero capacity.
    ZeroCapacity(Usage),
}

impl fmt::Display for TagAllocatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapacityExceedsRange {
                total_requested,
                range_size,
            } => write!(
                f,
                "partition capacities ({total_requested}) exceed range length ({range_size})"
            ),
            Self::EmptyRange => write!(f, "tag range is empty"),
            Self::ZeroCapacity(usage) => write!(f, "partition {usage:?} has zero capacity"),
        }
    }
}

impl std::error::Error for TagAllocatorError {}

/// Allocates unique tags from a fixed partition of the tag range.
pub trait TagAllocator: Send + Sync {
    /// Obtain the next available tag, or `None` if the partition is exhausted.
    fn allocate(&self) -> Option<AllocatedTag>;
}

/// Result type returned by [`create_allocators`].
pub type CreateAllocatorsResult = Result<Vec<(Usage, Arc<dyn TagAllocator>)>, TagAllocatorError>;

/// Create one [`TagAllocator`] per partition from a contiguous tag range.
///
/// The `range` is divided into non-overlapping sub-ranges according to the
/// capacities carried by each [`Usage`] variant.  Each returned allocator
/// yields tags exclusively from its own sub-range.
///
/// # Errors
///
/// Returns [`TagAllocatorError`] if the range is empty, any partition has
/// zero capacity, or the total requested capacity exceeds the range.
pub fn create_allocators(range: Range<u64>, partitions: [(Usage, u64); 3]) -> CreateAllocatorsResult {
    let range_size = range.end.saturating_sub(range.start);
    if range_size == 0 {
        return Err(TagAllocatorError::EmptyRange);
    }

    for (usage, capacity) in &partitions {
        if *capacity == 0 {
            return Err(TagAllocatorError::ZeroCapacity(*usage));
        }
    }

    let total_requested: u64 = partitions.iter().map(|(_, cap)| cap).sum();
    if total_requested > range_size {
        return Err(TagAllocatorError::CapacityExceedsRange {
            total_requested,
            range_size,
        });
    }

    let mut base = range.start;
    Ok(partitions
        .iter()
        .map(|(usage, capacity)| {
            let alloc = Arc::new(allocator::PartitionAllocator::new(base, *capacity));
            base += capacity;
            (*usage, alloc as Arc<dyn TagAllocator>)
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use hopr_protocol_app::prelude::ReservedTag;

    use super::*;

    #[test]
    fn partitions_are_non_overlapping() {
        let allocators = create_allocators(
            ReservedTag::range().end..u16::MAX as u64 + 1,
            [
                (Usage::Session, 100),
                (Usage::SessionTerminalTelemetry, 200),
                (Usage::ProvingTelemetry, 300),
            ],
        )
        .unwrap();

        let mut all_tags = Vec::new();
        let counts = [100, 200, 300];
        for (i, (_, alloc)) in allocators.iter().enumerate() {
            for _ in 0..counts[i] {
                all_tags.push(alloc.allocate().unwrap());
            }
        }

        // Check uniqueness while tags are still alive (not returned to pool).
        let all_values: HashSet<u64> = all_tags.iter().map(|t| t.value()).collect();
        assert_eq!(all_values.len(), 600);

        // Verify ranges
        let base = ReservedTag::range().end;
        let session_tags: Vec<u64> = (base..base + 100).collect();
        let telemetry_tags: Vec<u64> = (base + 100..base + 300).collect();
        let probing_tags: Vec<u64> = (base + 300..base + 600).collect();

        for t in &session_tags {
            assert!(all_values.contains(t));
        }
        for t in &telemetry_tags {
            assert!(all_values.contains(t));
        }
        for t in &probing_tags {
            assert!(all_values.contains(t));
        }
    }

    #[test]
    fn error_if_sizes_exceed_range() {
        let result = create_allocators(
            ReservedTag::range().end..100,
            [
                (Usage::Session, 50),
                (Usage::SessionTerminalTelemetry, 50),
                (Usage::ProvingTelemetry, 50),
            ],
        );
        assert!(matches!(
            result,
            Err(TagAllocatorError::CapacityExceedsRange {
                total_requested: 150,
                range_size: 84,
            })
        ));
    }

    #[test]
    fn error_if_empty_range() {
        let result = create_allocators(
            0..0,
            [
                (Usage::Session, 1),
                (Usage::SessionTerminalTelemetry, 1),
                (Usage::ProvingTelemetry, 1),
            ],
        );
        assert!(matches!(result, Err(TagAllocatorError::EmptyRange)));
    }

    #[test]
    fn error_if_zero_capacity() {
        let result = create_allocators(
            ReservedTag::range().end..u16::MAX as u64 + 1,
            [
                (Usage::Session, 0),
                (Usage::SessionTerminalTelemetry, 10),
                (Usage::ProvingTelemetry, 10),
            ],
        );
        assert!(matches!(result, Err(TagAllocatorError::ZeroCapacity(Usage::Session))));
    }

    #[test]
    fn allocated_tag_traits() {
        let allocators = create_allocators(
            ReservedTag::range().end..u16::MAX as u64 + 1,
            [
                (Usage::Session, 10),
                (Usage::SessionTerminalTelemetry, 10),
                (Usage::ProvingTelemetry, 10),
            ],
        )
        .unwrap();
        let (_, alloc) = &allocators[0];
        let tag = alloc.allocate().unwrap();

        let expected = ReservedTag::range().end;

        // From<&AllocatedTag> for u64
        let val: u64 = (&tag).into();
        assert_eq!(val, expected);

        // PartialEq<u64>
        assert_eq!(tag, expected);

        // Display
        assert_eq!(format!("{tag}"), expected.to_string());

        // Debug
        assert_eq!(format!("{tag:?}"), format!("AllocatedTag({expected})"));

        // Hash + Eq (usable as HashMap key)
        let mut map = std::collections::HashMap::new();
        map.insert(tag.value(), "test");
    }

    #[test]
    fn drop_recycles_tag() {
        let allocators = create_allocators(
            ReservedTag::range().end..u16::MAX as u64 + 1,
            [
                (Usage::Session, 2),
                (Usage::SessionTerminalTelemetry, 10),
                (Usage::ProvingTelemetry, 10),
            ],
        )
        .unwrap();
        let (_, alloc) = &allocators[0];

        let t0 = alloc.allocate().unwrap();
        let _t1 = alloc.allocate().unwrap();
        assert!(alloc.allocate().is_none());

        let val = t0.value();
        drop(t0);

        let t2 = alloc.allocate().unwrap();
        assert_eq!(t2.value(), val);
    }
}
