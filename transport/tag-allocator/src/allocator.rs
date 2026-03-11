use std::{ops::Range, sync::Arc};

use crate::{TagAllocator, allocated_tag::AllocatedTag, bitmap::TagBitmap};

/// A partition allocator that yields unique tags from a contiguous sub-range.
///
/// Tags are tracked via a lock-free bitmap — one bit per tag. Allocation
/// scans for the first available bit; deallocation (via [`AllocatedTag::drop`])
/// sets the bit back.
pub(crate) struct PartitionAllocator {
    base: u64,
    bitmap: Arc<TagBitmap>,
}

impl PartitionAllocator {
    pub fn new(base: u64, size: u64) -> Self {
        Self {
            base,
            bitmap: Arc::new(TagBitmap::new(size)),
        }
    }
}

impl TagAllocator for PartitionAllocator {
    fn allocate(&self) -> Option<AllocatedTag> {
        self.bitmap
            .allocate()
            .map(|index| AllocatedTag::new(self.base + index, index, self.bitmap.clone()))
    }

    fn capacity(&self) -> u64 {
        self.bitmap.capacity()
    }

    fn tag_range(&self) -> Range<u64> {
        self.base..self.base + self.bitmap.capacity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocates_sequentially_from_base() {
        let alloc = PartitionAllocator::new(100, 3);
        let t0 = alloc.allocate().unwrap();
        let t1 = alloc.allocate().unwrap();
        let t2 = alloc.allocate().unwrap();
        assert_eq!(t0.value(), 100);
        assert_eq!(t1.value(), 101);
        assert_eq!(t2.value(), 102);
    }

    #[test]
    fn exhaustion_returns_none() {
        let alloc = PartitionAllocator::new(10, 2);
        let _t0 = alloc.allocate().unwrap();
        let _t1 = alloc.allocate().unwrap();
        assert!(alloc.allocate().is_none());
    }

    #[test]
    fn drop_returns_tag_to_pool() {
        let alloc = PartitionAllocator::new(50, 2);
        let t0 = alloc.allocate().unwrap();
        let _t1 = alloc.allocate().unwrap();
        assert!(alloc.allocate().is_none());

        let val = t0.value();
        drop(t0);

        let t2 = alloc.allocate().unwrap();
        assert_eq!(t2.value(), val);
    }

    #[test]
    fn concurrent_no_duplicates() {
        use std::{collections::HashSet, thread};

        let alloc = Arc::new(PartitionAllocator::new(1, 1000));
        let mut handles = Vec::new();

        for _ in 0..10 {
            let a = alloc.clone();
            handles.push(thread::spawn(move || {
                let mut tags = Vec::new();
                for _ in 0..100 {
                    tags.push(a.allocate().unwrap());
                }
                tags
            }));
        }

        // Collect all AllocatedTags (keeping them alive to prevent recycling).
        let all_tags: Vec<Vec<AllocatedTag>> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let all_values: HashSet<u64> = all_tags.iter().flatten().map(|t| t.value()).collect();
        assert_eq!(all_values.len(), 1000);
    }
}
