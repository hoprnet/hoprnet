use std::cmp::Ordering;

/// Data structure holding the data alongside a release timemestamp.
///
/// The ordering functionality is defined only over the release timestamp
/// to ensure proper mixing.
pub struct DelayedData<T> {
    pub release_at: std::time::SystemTime,
    pub item: T,
}

impl<T> PartialEq for DelayedData<T> {
    fn eq(&self, other: &Self) -> bool {
        self.release_at == other.release_at
    }
}

impl<T> PartialOrd for DelayedData<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.release_at.partial_cmp(&other.release_at)
    }
}

impl<T> Eq for DelayedData<T> {}

impl<T> Ord for DelayedData<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.release_at.cmp(&other.release_at)
    }
}

impl<T> From<(std::time::SystemTime, T)> for DelayedData<T> {
    fn from(value: (std::time::SystemTime, T)) -> Self {
        Self {
            release_at: value.0,
            item: value.1,
        }
    }
}
