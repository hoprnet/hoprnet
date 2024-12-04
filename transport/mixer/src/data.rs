use std::cmp::Ordering;

/// Data structure holding the data alongside a release timemestamp.
///
/// The ordering functionality is defined only over the release timestamp
/// to ensure proper mixing.
pub struct DelayedData<T> {
    pub release: std::time::SystemTime,
    pub data: T,
}

impl<T> PartialEq for DelayedData<T> {
    fn eq(&self, other: &Self) -> bool {
        self.release == other.release
    }
}

impl<T> PartialOrd for DelayedData<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // self.release.partial_cmp(&other.release)
        self.release.partial_cmp(&other.release).map(|v| match v {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            Ordering::Equal => Ordering::Equal,
        })
    }
}

impl<T> Eq for DelayedData<T> {}

impl<T> Ord for DelayedData<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // self.release.cmp(&other.release)
        match self.release.cmp(&other.release) {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            Ordering::Equal => Ordering::Equal,
        }
    }
}

impl<T> From<(std::time::SystemTime, T)> for DelayedData<T> {
    fn from(value: (std::time::SystemTime, T)) -> Self {
        Self {
            release: value.0,
            data: value.1,
        }
    }
}
