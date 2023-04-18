use std::sync::atomic::{AtomicUsize, Ordering};

/// Specialized once_cell::race::OnceNonZeroUsize
///
/// Use MAX instead of MIN for uninit case to make it easier to work with for bitflags
#[derive(Debug)]
pub(crate) struct Lazy(AtomicUsize);

impl Lazy {
    const UNINIT: usize = usize::MAX;

    pub(crate) const fn new() -> Self {
        Self(AtomicUsize::new(Self::UNINIT))
    }

    pub(crate) fn get_or_init<F>(&self, f: F) -> usize
    where
        F: FnOnce() -> usize,
    {
        self.get().unwrap_or_else(|| {
            let mut val = f();
            assert_ne!(val, Self::UNINIT);
            let exchange =
                self.0
                    .compare_exchange(Self::UNINIT, val, Ordering::AcqRel, Ordering::Acquire);
            if let Err(old) = exchange {
                val = old;
            }
            debug_assert_ne!(val, Self::UNINIT);
            val
        })
    }

    fn get(&self) -> Option<usize> {
        let val = self.0.load(Ordering::Acquire);
        if val == Self::UNINIT {
            None
        } else {
            Some(val)
        }
    }
}

impl Default for Lazy {
    fn default() -> Self {
        Lazy::new()
    }
}
