mod allocated_tag;
mod allocator;
mod bitmap;
pub mod errors;

use std::{ops::Range, sync::Arc};

pub use allocated_tag::AllocatedTag;
pub use errors::TagAllocatorError;
use hopr_protocol_app::prelude::ReservedTag;

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

/// Lower bound of the application tag range (exclusive end of [`ReservedTag`] range).
pub const TAG_RANGE_START: u64 = ReservedTag::UPPER_BOUND;
/// Upper bound of the application tag range (exclusive).
pub const TAG_RANGE_END: u64 = u16::MAX as u64 + 1;
/// Total number of available application tags.
pub const TAG_RANGE_SIZE: u64 = TAG_RANGE_END - TAG_RANGE_START;

// Compile-time assert: the tag range must fit within u16. A bitmap for a
// larger range would require a different allocation strategy.
const _: () = assert!(
    TAG_RANGE_END <= u16::MAX as u64 + 1,
    "tag range exceeds u16 — a different allocation strategy is needed"
);

/// Default number of tags reserved for sessions.
pub const DEFAULT_SESSION_CAPACITY: u64 = 2048;
/// Default number of tags reserved for session terminal telemetry.
pub const DEFAULT_SESSION_PROBING_CAPACITY: u64 = 4000;
/// Default number of tags reserved for probing telemetry (remainder of range).
pub const DEFAULT_PROBING_TELEMETRY_CAPACITY: u64 =
    TAG_RANGE_SIZE - DEFAULT_SESSION_CAPACITY - DEFAULT_SESSION_PROBING_CAPACITY;

/// Configuration for the tag allocator partitions.
///
/// Each field specifies the number of tags reserved for that usage category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct TagAllocatorConfig {
    /// Number of tags reserved for long-lived sessions.
    ///
    /// This also determines the maximum number of concurrent sessions.
    #[default(DEFAULT_SESSION_CAPACITY)]
    pub session: u64,

    /// Number of tags reserved for session terminal telemetry.
    #[default(DEFAULT_SESSION_PROBING_CAPACITY)]
    pub session_probing: u64,

    /// Number of tags reserved for probing telemetry.
    ///
    /// Defaults to the remainder of the available tag range.
    #[default(DEFAULT_PROBING_TELEMETRY_CAPACITY)]
    pub probing_telemetry: u64,
}

impl TagAllocatorConfig {
    /// Returns the tag range for the given [`Usage`] partition.
    ///
    /// Partitions are laid out contiguously starting at
    /// [`ReservedTag::UPPER_BOUND`] in the order: Session,
    /// SessionTerminalTelemetry, ProvingTelemetry.
    pub fn range_for(&self, usage: Usage) -> Range<u64> {
        let base = ReservedTag::UPPER_BOUND;
        match usage {
            Usage::Session => base..base + self.session,
            Usage::SessionTerminalTelemetry => {
                let start = base + self.session;
                start..start + self.session_probing
            }
            Usage::ProvingTelemetry => {
                let start = base + self.session + self.session_probing;
                start..start + self.probing_telemetry
            }
        }
    }

    /// The full tag range covered by this configuration.
    ///
    /// Starts at [`ReservedTag::UPPER_BOUND`] and spans the sum of all
    /// configured partition capacities.
    pub fn tag_range(&self) -> Range<u64> {
        let start = ReservedTag::UPPER_BOUND;
        start..start + self.session + self.session_probing + self.probing_telemetry
    }
}

/// Allocates unique tags from a fixed partition of the tag range.
pub trait TagAllocator {
    /// Obtain the next available tag, or `None` if the partition is exhausted.
    fn allocate(&self) -> Option<AllocatedTag>;

    /// The total number of tags managed by this allocator.
    fn capacity(&self) -> u64;

    /// The tag value range `[base, base + capacity)` managed by this allocator.
    fn tag_range(&self) -> Range<u64>;
}

/// Result type returned by [`create_allocators`].
pub type CreateAllocatorsResult = Result<Vec<(Usage, Arc<dyn TagAllocator + Send + Sync>)>, TagAllocatorError>;

/// Create allocators from a [`TagAllocatorConfig`].
///
/// Uses [`TagAllocatorConfig::tag_range`] as the available range and
/// partitions it according to the configured capacities.
///
/// # Errors
///
/// Returns [`TagAllocatorError`] if any partition has zero capacity or the
/// total requested capacity exceeds the range.
pub fn create_allocators_from_config(cfg: &TagAllocatorConfig) -> CreateAllocatorsResult {
    let partitions = [
        (Usage::Session, cfg.session),
        (Usage::SessionTerminalTelemetry, cfg.session_probing),
        (Usage::ProvingTelemetry, cfg.probing_telemetry),
    ];

    for (usage, capacity) in &partitions {
        if *capacity == 0 {
            return Err(TagAllocatorError::ZeroCapacity(*usage));
        }
    }

    Ok(partitions
        .iter()
        .map(|(usage, capacity)| {
            let range = cfg.range_for(*usage);
            let alloc = Arc::new(allocator::PartitionAllocator::new(range.start, *capacity));
            (*usage, alloc as Arc<dyn TagAllocator + Send + Sync>)
        })
        .collect())
}

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
            (*usage, alloc as Arc<dyn TagAllocator + Send + Sync>)
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

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
        let debug_str = format!("{tag:?}");
        assert!(
            debug_str.contains("AllocatedTag") && debug_str.contains(&expected.to_string()),
            "unexpected debug output: {debug_str}"
        );

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
