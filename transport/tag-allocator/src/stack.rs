use std::sync::atomic::{AtomicU64, Ordering};

const EMPTY: u64 = 0;

/// Lock-free, fixed-capacity array stack.
///
/// Supports concurrent push/pop with O(1) amortized performance.
/// Values must never be `0` (reserved as empty sentinel).
pub(crate) struct ArrayStack {
    slots: Box<[AtomicU64]>,
    top: AtomicU64,
    capacity: u64,
}

impl ArrayStack {
    pub fn new(capacity: u64) -> Self {
        let slots: Vec<AtomicU64> = (0..capacity).map(|_| AtomicU64::new(EMPTY)).collect();
        Self {
            slots: slots.into_boxed_slice(),
            top: AtomicU64::new(0),
            capacity,
        }
    }

    /// Push a value onto the stack. Returns `false` if full.
    pub fn push(&self, value: u64) -> bool {
        debug_assert_ne!(value, EMPTY, "cannot push sentinel value 0");
        loop {
            let top = self.top.load(Ordering::Acquire);
            if top >= self.capacity {
                return false;
            }
            if self
                .top
                .compare_exchange_weak(top, top + 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                self.slots[top as usize].store(value, Ordering::Release);
                return true;
            }
        }
    }

    /// Pop a value from the stack. Returns `None` if empty.
    pub fn pop(&self) -> Option<u64> {
        loop {
            let top = self.top.load(Ordering::Acquire);
            if top == 0 {
                return None;
            }
            if self
                .top
                .compare_exchange_weak(top, top - 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                let idx = (top - 1) as usize;
                // Spin until the concurrent push finishes writing the slot.
                // This window is at most a few nanoseconds.
                loop {
                    let val = self.slots[idx].swap(EMPTY, Ordering::AcqRel);
                    if val != EMPTY {
                        return Some(val);
                    }
                    std::hint::spin_loop();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop_basic() {
        let stack = ArrayStack::new(4);
        assert!(stack.push(1));
        assert!(stack.push(2));
        assert!(stack.push(3));
        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn push_full() {
        let stack = ArrayStack::new(2);
        assert!(stack.push(10));
        assert!(stack.push(20));
        assert!(!stack.push(30));
    }

    #[test]
    fn push_pop_reuse() {
        let stack = ArrayStack::new(2);
        assert!(stack.push(1));
        assert!(stack.push(2));
        assert!(!stack.push(3));
        assert_eq!(stack.pop(), Some(2));
        assert!(stack.push(4));
        assert_eq!(stack.pop(), Some(4));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn concurrent_push_pop() {
        use std::{sync::Arc, thread};

        let stack = Arc::new(ArrayStack::new(1000));
        let mut handles = Vec::new();

        // 10 threads push 100 values each
        for t in 0..10 {
            let s = stack.clone();
            handles.push(thread::spawn(move || {
                for i in 0..100 {
                    let val = (t * 100 + i + 1) as u64; // +1 to avoid 0
                    assert!(s.push(val));
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // Pop all values and verify uniqueness
        let mut values = Vec::new();
        while let Some(v) = stack.pop() {
            values.push(v);
        }
        values.sort();
        values.dedup();
        assert_eq!(values.len(), 1000);
    }
}
