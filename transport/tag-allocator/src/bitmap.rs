use std::sync::atomic::{AtomicU64, Ordering};

/// Lock-free bitmap for tag availability tracking.
///
/// Each bit represents a tag: `1` = available, `0` = allocated.
/// Uses `AtomicU64` words so each word covers 64 tags.
pub(crate) struct TagBitmap {
    words: Box<[AtomicU64]>,
    capacity: u64,
}

impl TagBitmap {
    /// Create a new bitmap with all `capacity` tags marked as available.
    pub fn new(capacity: u64) -> Self {
        let num_words = capacity.div_ceil(64);
        let mut words: Vec<AtomicU64> = (0..num_words).map(|_| AtomicU64::new(u64::MAX)).collect();

        // Clear excess bits in the last word so only valid tags are available.
        let remainder = capacity % 64;
        if remainder != 0 {
            if let Some(last) = words.last_mut() {
                *last = AtomicU64::new((1u64 << remainder) - 1);
            }
        }

        Self {
            words: words.into_boxed_slice(),
            capacity,
        }
    }

    /// Allocate the next available tag. Returns the tag index (0-based), or
    /// `None` if all tags are in use.
    ///
    /// Scans words sequentially and within each word picks the lowest set bit.
    pub fn allocate(&self) -> Option<u64> {
        for (word_idx, word) in self.words.iter().enumerate() {
            loop {
                let val = word.load(Ordering::Acquire);
                if val == 0 {
                    break; // no available bits in this word
                }
                let bit = val.trailing_zeros() as u64;
                let mask = 1u64 << bit;
                if word
                    .compare_exchange_weak(val, val & !mask, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
                {
                    return Some(word_idx as u64 * 64 + bit);
                }
                // CAS failed — another thread grabbed a bit, retry this word.
            }
        }
        None
    }

    /// The total number of tags this bitmap can track.
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    /// Return a previously allocated tag, making it available again.
    pub fn deallocate(&self, index: u64) {
        debug_assert!(index < self.capacity, "index {index} out of range");
        let word_idx = (index / 64) as usize;
        let bit = index % 64;
        let mask = 1u64 << bit;
        self.words[word_idx].fetch_or(mask, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, sync::Arc, thread};

    use super::*;

    // ── Capacity edge cases ─────────────────────────────────────────

    #[test]
    fn zero_capacity() {
        let bm = TagBitmap::new(0);
        assert_eq!(bm.allocate(), None);
    }

    #[test]
    fn single_tag() {
        let bm = TagBitmap::new(1);
        assert_eq!(bm.allocate(), Some(0));
        assert_eq!(bm.allocate(), None);
        bm.deallocate(0);
        assert_eq!(bm.allocate(), Some(0));
        assert_eq!(bm.allocate(), None);
    }

    #[test]
    fn capacity_exactly_64() {
        let bm = TagBitmap::new(64);
        for i in 0..64 {
            assert_eq!(bm.allocate(), Some(i));
        }
        assert_eq!(bm.allocate(), None);
    }

    #[test]
    fn capacity_63() {
        let bm = TagBitmap::new(63);
        for i in 0..63 {
            assert_eq!(bm.allocate(), Some(i));
        }
        assert_eq!(bm.allocate(), None);
    }

    #[test]
    fn capacity_65() {
        let bm = TagBitmap::new(65);
        for i in 0..65 {
            assert_eq!(bm.allocate(), Some(i));
        }
        assert_eq!(bm.allocate(), None);
    }

    #[test]
    fn capacity_128() {
        let bm = TagBitmap::new(128);
        for i in 0..128 {
            assert_eq!(bm.allocate(), Some(i));
        }
        assert_eq!(bm.allocate(), None);
    }

    #[test]
    fn capacity_127() {
        let bm = TagBitmap::new(127);
        for i in 0..127 {
            assert_eq!(bm.allocate(), Some(i));
        }
        assert_eq!(bm.allocate(), None);
    }

    #[test]
    fn capacity_129() {
        let bm = TagBitmap::new(129);
        for i in 0..129 {
            assert_eq!(bm.allocate(), Some(i));
        }
        assert_eq!(bm.allocate(), None);
    }

    // ── Sequential allocation ───────────────────────────────────────

    #[test]
    fn allocate_sequential_small() {
        let bm = TagBitmap::new(4);
        assert_eq!(bm.allocate(), Some(0));
        assert_eq!(bm.allocate(), Some(1));
        assert_eq!(bm.allocate(), Some(2));
        assert_eq!(bm.allocate(), Some(3));
        assert_eq!(bm.allocate(), None);
    }

    #[test]
    fn allocate_sequential_across_word_boundary() {
        let bm = TagBitmap::new(130);
        let mut indices = Vec::new();
        while let Some(idx) = bm.allocate() {
            indices.push(idx);
        }
        assert_eq!(indices.len(), 130);
        // Must be strictly sequential: 0, 1, 2, ..., 129
        for (i, idx) in indices.iter().enumerate() {
            assert_eq!(*idx, i as u64);
        }
    }

    // ── Deallocation and reuse ──────────────────────────────────────

    #[test]
    fn deallocate_makes_tag_available() {
        let bm = TagBitmap::new(2);
        assert_eq!(bm.allocate(), Some(0));
        assert_eq!(bm.allocate(), Some(1));
        assert_eq!(bm.allocate(), None);

        bm.deallocate(0);
        assert_eq!(bm.allocate(), Some(0));
        assert_eq!(bm.allocate(), None);
    }

    #[test]
    fn deallocate_last_tag_in_word() {
        let bm = TagBitmap::new(64);
        let mut tags: Vec<u64> = (0..64).map(|_| bm.allocate().unwrap()).collect();
        assert_eq!(bm.allocate(), None);

        // Free the very last bit (index 63).
        bm.deallocate(63);
        assert_eq!(bm.allocate(), Some(63));
        assert_eq!(bm.allocate(), None);

        // Free the very first bit (index 0).
        bm.deallocate(tags[0]);
        assert_eq!(bm.allocate(), Some(0));
        assert_eq!(bm.allocate(), None);

        // Free all remaining and verify full recovery.
        for t in tags.drain(1..63) {
            bm.deallocate(t);
        }
        let mut recovered: Vec<u64> = (0..62).map(|_| bm.allocate().unwrap()).collect();
        recovered.sort();
        let expected: Vec<u64> = (1..63).collect();
        assert_eq!(recovered, expected);
    }

    #[test]
    fn deallocate_across_word_boundary() {
        let bm = TagBitmap::new(128);
        let tags: Vec<u64> = (0..128).map(|_| bm.allocate().unwrap()).collect();
        assert_eq!(bm.allocate(), None);

        // Free one tag from each word.
        bm.deallocate(tags[30]); // word 0, bit 30
        bm.deallocate(tags[90]); // word 1, bit 26

        let a = bm.allocate().unwrap();
        let b = bm.allocate().unwrap();
        assert_eq!(bm.allocate(), None);

        let mut pair = [a, b];
        pair.sort();
        assert_eq!(pair, [30, 90]);
    }

    #[test]
    fn deallocate_idempotent() {
        let bm = TagBitmap::new(4);
        let idx = bm.allocate().unwrap();
        bm.deallocate(idx);
        // Double-deallocate should not corrupt state — the bit is already set.
        bm.deallocate(idx);
        // Should still only yield one tag for this index.
        assert_eq!(bm.allocate(), Some(idx));
    }

    // ── Full cycle (allocate all → free all → allocate all) ─────────

    #[test]
    fn full_cycle_small() {
        let bm = TagBitmap::new(10);
        let tags: Vec<u64> = (0..10).map(|_| bm.allocate().unwrap()).collect();
        assert_eq!(bm.allocate(), None);

        for t in &tags {
            bm.deallocate(*t);
        }

        // All should be available again in sequential order.
        for i in 0..10 {
            assert_eq!(bm.allocate(), Some(i));
        }
        assert_eq!(bm.allocate(), None);
    }

    #[test]
    fn full_cycle_multi_word() {
        let bm = TagBitmap::new(200);
        let tags: Vec<u64> = (0..200).map(|_| bm.allocate().unwrap()).collect();
        assert_eq!(bm.allocate(), None);

        for t in &tags {
            bm.deallocate(*t);
        }

        let recovered: Vec<u64> = (0..200).map(|_| bm.allocate().unwrap()).collect();
        for (i, idx) in recovered.iter().enumerate() {
            assert_eq!(*idx, i as u64);
        }
        assert_eq!(bm.allocate(), None);
    }

    #[test]
    fn repeated_full_cycles() {
        let bm = TagBitmap::new(64);
        for _cycle in 0..100 {
            let tags: Vec<u64> = (0..64).map(|_| bm.allocate().unwrap()).collect();
            assert_eq!(bm.allocate(), None);
            for t in &tags {
                bm.deallocate(*t);
            }
        }
        // After 100 full cycles, bitmap should still be fully functional.
        for i in 0..64 {
            assert_eq!(bm.allocate(), Some(i));
        }
        assert_eq!(bm.allocate(), None);
    }

    // ── Interleaved allocate/deallocate patterns ────────────────────

    #[test]
    fn alternating_allocate_deallocate() {
        let bm = TagBitmap::new(3);
        let a = bm.allocate().unwrap(); // 0
        let b = bm.allocate().unwrap(); // 1
        bm.deallocate(a); // free 0
        let c = bm.allocate().unwrap(); // should get 0
        assert_eq!(c, 0);
        let d = bm.allocate().unwrap(); // should get 2
        assert_eq!(d, 2);
        assert_eq!(bm.allocate(), None);

        bm.deallocate(b); // free 1
        assert_eq!(bm.allocate(), Some(1));
    }

    #[test]
    fn free_in_reverse_order() {
        let bm = TagBitmap::new(8);
        let tags: Vec<u64> = (0..8).map(|_| bm.allocate().unwrap()).collect();
        assert_eq!(bm.allocate(), None);

        // Free in reverse: 7, 6, 5, ..., 0
        for t in tags.iter().rev() {
            bm.deallocate(*t);
        }

        // Re-allocation should still be sequential (lowest bit first).
        for i in 0..8 {
            assert_eq!(bm.allocate(), Some(i));
        }
    }

    #[test]
    fn free_even_then_odd() {
        let bm = TagBitmap::new(8);
        let tags: Vec<u64> = (0..8).map(|_| bm.allocate().unwrap()).collect();

        // Free even indices: 0, 2, 4, 6
        for t in tags.iter().step_by(2) {
            bm.deallocate(*t);
        }

        let mut evens = Vec::new();
        while let Some(idx) = bm.allocate() {
            evens.push(idx);
        }
        evens.sort();
        assert_eq!(evens, vec![0, 2, 4, 6]);

        // Free odd indices: 1, 3, 5, 7
        for t in tags.iter().skip(1).step_by(2) {
            bm.deallocate(*t);
        }

        let mut odds = Vec::new();
        while let Some(idx) = bm.allocate() {
            odds.push(idx);
        }
        odds.sort();
        assert_eq!(odds, vec![1, 3, 5, 7]);
    }

    // ── Larger capacities ───────────────────────────────────────────

    #[test]
    fn large_capacity_2048() {
        let bm = TagBitmap::new(2048);
        let mut tags = Vec::with_capacity(2048);
        for i in 0..2048 {
            let idx = bm.allocate().unwrap();
            assert_eq!(idx, i);
            tags.push(idx);
        }
        assert_eq!(bm.allocate(), None);

        // Free every 7th tag.
        let freed: Vec<u64> = tags.iter().copied().filter(|t| t % 7 == 0).collect();
        for t in &freed {
            bm.deallocate(*t);
        }

        let mut recovered = Vec::new();
        while let Some(idx) = bm.allocate() {
            recovered.push(idx);
        }
        recovered.sort();
        assert_eq!(recovered, freed);
    }

    #[test]
    fn large_capacity_10000() {
        let bm = TagBitmap::new(10_000);
        let tags: Vec<u64> = (0..10_000).map(|_| bm.allocate().unwrap()).collect();
        assert_eq!(bm.allocate(), None);

        let unique: HashSet<u64> = tags.iter().copied().collect();
        assert_eq!(unique.len(), 10_000);

        // All indices should be in [0, 10_000).
        assert_eq!(*tags.iter().min().unwrap(), 0);
        assert_eq!(*tags.iter().max().unwrap(), 9_999);
    }

    #[test]
    fn large_capacity_65535_full_u16_range() {
        let cap = u16::MAX as u64;
        let bm = TagBitmap::new(cap);

        // Allocate all.
        for i in 0..cap {
            assert_eq!(bm.allocate(), Some(i));
        }
        assert_eq!(bm.allocate(), None);

        // Free all.
        for i in 0..cap {
            bm.deallocate(i);
        }

        // Allocate all again — verifies full rollover.
        for i in 0..cap {
            assert_eq!(bm.allocate(), Some(i));
        }
        assert_eq!(bm.allocate(), None);
    }

    // ── Word-boundary stress tests ──────────────────────────────────

    #[test]
    fn word_boundary_bits_63_and_64() {
        let bm = TagBitmap::new(130);
        let tags: Vec<u64> = (0..130).map(|_| bm.allocate().unwrap()).collect();

        // Free bits at the boundary between word 0 and word 1.
        bm.deallocate(63); // last bit of word 0
        bm.deallocate(64); // first bit of word 1

        let a = bm.allocate().unwrap();
        let b = bm.allocate().unwrap();
        let mut pair = [a, b];
        pair.sort();
        assert_eq!(pair, [63, 64]);
        assert_eq!(bm.allocate(), None);

        // Restore for the rest.
        for t in tags.iter().filter(|&&t| t != 63 && t != 64) {
            bm.deallocate(*t);
        }
    }

    #[test]
    fn only_last_bit_of_partial_word_available() {
        // 65 tags = 1 full word (64) + 1 partial word (1 bit).
        let bm = TagBitmap::new(65);
        // Allocate first 64.
        for i in 0..64 {
            assert_eq!(bm.allocate(), Some(i));
        }
        // Only index 64 (bit 0 of word 1) should remain.
        assert_eq!(bm.allocate(), Some(64));
        assert_eq!(bm.allocate(), None);
    }

    #[test]
    fn partial_word_surplus_bits_not_allocatable() {
        // 3 tags → 1 word, bits 0-2 set, bits 3-63 must be zero.
        let bm = TagBitmap::new(3);
        assert_eq!(bm.allocate(), Some(0));
        assert_eq!(bm.allocate(), Some(1));
        assert_eq!(bm.allocate(), Some(2));
        // Must not hand out indices 3..63.
        assert_eq!(bm.allocate(), None);
    }

    // ── Repeated rollover (churn) ───────────────────────────────────

    #[test]
    fn churn_single_slot() {
        let bm = TagBitmap::new(1);
        for _ in 0..10_000 {
            let idx = bm.allocate().unwrap();
            assert_eq!(idx, 0);
            assert_eq!(bm.allocate(), None);
            bm.deallocate(idx);
        }
    }

    #[test]
    fn churn_multi_word() {
        let bm = TagBitmap::new(200);
        for _round in 0..50 {
            let tags: Vec<u64> = (0..200).map(|_| bm.allocate().unwrap()).collect();
            assert_eq!(bm.allocate(), None);
            // Free in shuffled order (deterministic: reverse).
            for t in tags.iter().rev() {
                bm.deallocate(*t);
            }
        }
        // Verify clean state.
        let tags: Vec<u64> = (0..200).map(|_| bm.allocate().unwrap()).collect();
        let unique: HashSet<u64> = tags.iter().copied().collect();
        assert_eq!(unique.len(), 200);
    }

    // ── Concurrent tests ────────────────────────────────────────────

    #[test]
    fn concurrent_no_duplicates() {
        let bm = Arc::new(TagBitmap::new(1000));
        let mut handles = Vec::new();

        for _ in 0..10 {
            let b = bm.clone();
            handles.push(thread::spawn(move || {
                let mut indices = Vec::new();
                for _ in 0..100 {
                    indices.push(b.allocate().unwrap());
                }
                indices
            }));
        }

        let all: Vec<u64> = handles.into_iter().flat_map(|h| h.join().unwrap()).collect();
        let unique: HashSet<u64> = all.iter().copied().collect();
        assert_eq!(unique.len(), 1000);
    }

    #[test]
    fn concurrent_allocate_deallocate() {
        let bm = Arc::new(TagBitmap::new(10));
        let mut handles = Vec::new();

        for _ in 0..4 {
            let b = bm.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    if let Some(idx) = b.allocate() {
                        b.deallocate(idx);
                    }
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // All tags should be available again.
        for i in 0..10 {
            assert_eq!(bm.allocate(), Some(i));
        }
        assert_eq!(bm.allocate(), None);
    }

    #[test]
    fn concurrent_exhaustion_returns_none() {
        // More threads than tags — some must get None.
        let bm = Arc::new(TagBitmap::new(5));
        let mut handles = Vec::new();

        for _ in 0..20 {
            let b = bm.clone();
            handles.push(thread::spawn(move || b.allocate()));
        }

        let results: Vec<Option<u64>> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let allocated: Vec<u64> = results.iter().filter_map(|r| *r).collect();
        let unique: HashSet<u64> = allocated.iter().copied().collect();

        assert_eq!(allocated.len(), 5);
        assert_eq!(unique.len(), 5);
        assert_eq!(results.iter().filter(|r| r.is_none()).count(), 15);
    }

    #[test]
    fn concurrent_churn_large() {
        let bm = Arc::new(TagBitmap::new(256));
        let mut handles = Vec::new();

        for _ in 0..8 {
            let b = bm.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..5000 {
                    if let Some(idx) = b.allocate() {
                        // Hold briefly, then return.
                        std::hint::black_box(idx);
                        b.deallocate(idx);
                    }
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // All 256 tags should be available.
        let mut recovered = Vec::new();
        while let Some(idx) = bm.allocate() {
            recovered.push(idx);
        }
        recovered.sort();
        let expected: Vec<u64> = (0..256).collect();
        assert_eq!(recovered, expected);
    }

    #[test]
    fn concurrent_mixed_producers_consumers() {
        // Half the threads allocate; the other half deallocate returned tags.
        use std::sync::Mutex;

        let bm = Arc::new(TagBitmap::new(500));
        let returned = Arc::new(Mutex::new(Vec::new()));
        let mut handles = Vec::new();

        // Allocate all upfront so consumers have something to free.
        let initial: Vec<u64> = (0..500).map(|_| bm.allocate().unwrap()).collect();
        assert_eq!(bm.allocate(), None);

        // Seed the return queue.
        *returned.lock().unwrap() = initial;

        // 4 consumer threads: take from returned queue, deallocate.
        for _ in 0..4 {
            let b = bm.clone();
            let r = returned.clone();
            handles.push(thread::spawn(move || {
                loop {
                    let idx = {
                        let mut guard = r.lock().unwrap();
                        guard.pop()
                    };
                    match idx {
                        Some(i) => b.deallocate(i),
                        None => break,
                    }
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // All tags should be free.
        let mut recovered = Vec::new();
        while let Some(idx) = bm.allocate() {
            recovered.push(idx);
        }
        assert_eq!(recovered.len(), 500);
    }
}
