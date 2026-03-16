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

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    #[test]
    fn equal_timestamps_are_equal() {
        let now = Instant::now();
        let a = DelayedData {
            release_at: now,
            item: "a",
        };
        let b = DelayedData {
            release_at: now,
            item: "b",
        };
        assert!(a == b);
        assert_eq!(a.cmp(&b), Ordering::Equal);
    }

    #[test]
    fn earlier_timestamp_is_less() {
        let now = Instant::now();
        let earlier = DelayedData {
            release_at: now,
            item: 1,
        };
        let later = DelayedData {
            release_at: now + Duration::from_secs(1),
            item: 2,
        };
        assert!(earlier < later);
        assert_eq!(earlier.cmp(&later), Ordering::Less);
    }

    #[test]
    fn later_timestamp_is_greater() {
        let now = Instant::now();
        let earlier = DelayedData {
            release_at: now,
            item: 1,
        };
        let later = DelayedData {
            release_at: now + Duration::from_secs(1),
            item: 2,
        };
        assert!(later > earlier);
        assert_eq!(later.cmp(&earlier), Ordering::Greater);
    }

    #[test]
    fn ordering_ignores_item_value() {
        let now = Instant::now();
        let a = DelayedData {
            release_at: now,
            item: 999,
        };
        let b = DelayedData {
            release_at: now,
            item: 1,
        };
        // Items differ but timestamps are equal → they are equal
        assert!(a == b);
    }

    #[test]
    fn from_tuple_sets_fields() {
        let now = Instant::now();
        let data = DelayedData::from((now, "hello"));
        assert_eq!(data.release_at, now);
        assert_eq!(data.item, "hello");
    }

    #[test]
    fn sorting_respects_release_time() {
        let now = Instant::now();
        let mut items = [
            DelayedData {
                release_at: now + Duration::from_secs(3),
                item: "c",
            },
            DelayedData {
                release_at: now + Duration::from_secs(1),
                item: "a",
            },
            DelayedData {
                release_at: now + Duration::from_secs(2),
                item: "b",
            },
        ];
        items.sort();
        assert_eq!(items[0].item, "a");
        assert_eq!(items[1].item, "b");
        assert_eq!(items[2].item, "c");
    }
}
