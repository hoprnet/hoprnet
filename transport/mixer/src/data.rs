use std::cmp::Ordering;

/// Data structure holding the data alongside a release timemestamp.
///
/// The ordering functionality is defined only over the release timestamp
/// to ensure proper mixing.
pub struct DelayedData<T> {
    pub release_at: std::time::Instant,
    pub item: T,
}

impl<T> PartialEq for DelayedData<T> {
    fn eq(&self, other: &Self) -> bool {
        self.release_at == other.release_at
    }
}

impl<T> PartialOrd for DelayedData<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Eq for DelayedData<T> {}

impl<T> Ord for DelayedData<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.release_at.cmp(&other.release_at)
    }
}

impl<T> From<(std::time::Instant, T)> for DelayedData<T> {
    fn from(value: (std::time::Instant, T)) -> Self {
        Self {
            release_at: value.0,
            item: value.1,
        }
    }
}
