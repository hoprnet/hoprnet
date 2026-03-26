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

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::equal_timestamps(0, 0, Ordering::Equal)]
    #[case::earlier_is_less(0, 1000, Ordering::Less)]
    #[case::later_is_greater(1000, 0, Ordering::Greater)]
    #[case::ordering_ignores_item_value(0, 0, Ordering::Equal)]
    fn delayed_data_ordering(#[case] delay_a_ms: u64, #[case] delay_b_ms: u64, #[case] expected: Ordering) {
        let now = Instant::now();
        let a = DelayedData {
            release_at: now + Duration::from_millis(delay_a_ms),
            item: 1,
        };
        let b = DelayedData {
            release_at: now + Duration::from_millis(delay_b_ms),
            item: 2,
        };
        assert_eq!(a.cmp(&b), expected);
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
        let mut items: Vec<DelayedData<&str>> = [(3, "c"), (1, "a"), (2, "b")]
            .into_iter()
            .map(|(secs, val)| DelayedData {
                release_at: now + Duration::from_secs(secs),
                item: val,
            })
            .collect();
        items.sort();
        let sorted: Vec<&str> = items.iter().map(|d| d.item).collect();
        insta::assert_yaml_snapshot!(sorted);
    }
}
