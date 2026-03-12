use rand::RngExt;

/// A collection of items with associated weights for probabilistic selection.
///
/// Weights must be positive (`> 0.0`); items with non-positive weights are
/// treated as having zero probability for [`pick_one`](Self::pick_one) /
/// [`pick_index`](Self::pick_index) and are placed at the end of the shuffled
/// output in [`into_shuffled`](Self::into_shuffled).
///
/// # Examples
///
/// ```rust
/// use hopr_statistics::WeightedCollection;
///
/// let wc = WeightedCollection::new(vec![("rare", 0.1), ("common", 10.0)]);
/// let picked = wc.pick_one().expect("non-empty collection");
/// assert!(picked == "rare" || picked == "common");
/// ```
pub struct WeightedCollection<T> {
    items: Vec<(T, f64)>,
}

impl<T> WeightedCollection<T> {
    /// Create a new weighted collection from items paired with their weights.
    pub fn new(items: Vec<(T, f64)>) -> Self {
        Self { items }
    }

    /// Returns `true` if the collection contains no items.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the number of items in the collection.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Iterates over `(item, weight)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = &(T, f64)> {
        self.items.iter()
    }

    /// Returns the index of a randomly selected item, weighted by probability
    /// proportional to its weight.
    ///
    /// Returns `None` if the collection is empty or all weights are non-positive.
    pub fn pick_index(&self) -> Option<usize> {
        if self.items.is_empty() {
            return None;
        }

        let total_weight: f64 = self.items.iter().map(|(_, w)| w.max(0.0)).sum();
        if total_weight <= 0.0 {
            return None;
        }

        if self.items.len() == 1 {
            return Some(0);
        }

        let mut rng = rand::rng();
        let r = rng.random_range(0.0..total_weight);
        let mut cumulative = 0.0;
        for (i, (_, weight)) in self.items.iter().enumerate() {
            cumulative += weight.max(0.0);
            if r < cumulative {
                return Some(i);
            }
        }

        // Floating-point edge case: return the last positive-weight item.
        self.items.iter().rposition(|(_, weight)| *weight > 0.0)
    }
}

impl<T> WeightedCollection<T> {
    /// Pick a reference to one item at random, with probability proportional
    /// to its weight.
    ///
    /// Returns `None` if the collection is empty or all weights are non-positive.
    pub fn pick_ref(&self) -> Option<&T> {
        self.pick_index().map(|i| &self.items[i].0)
    }
}

impl<T: Clone> WeightedCollection<T> {
    /// Pick one item at random, with probability proportional to its weight.
    ///
    /// Returns `None` if the collection is empty or all weights are non-positive.
    pub fn pick_one(&self) -> Option<T> {
        self.pick_ref().cloned()
    }
}

impl<T> WeightedCollection<T> {
    /// Consume the collection and return items in a weighted random permutation.
    ///
    /// Uses the Efraimidis–Spirakis algorithm: each item is assigned a key
    /// `random()^(1/weight)` and the items are sorted by descending key.
    /// Higher-weight items appear earlier with higher probability, but all
    /// items retain a nonzero chance of appearing at any position.
    pub fn into_shuffled(self) -> Vec<T> {
        let mut rng = rand::rng();

        let mut keyed: Vec<(T, f64)> = self
            .items
            .into_iter()
            .map(|(item, weight)| {
                let key = if weight > 0.0 {
                    let u: f64 = rng.random_range(f64::EPSILON..1.0);
                    u.powf(1.0 / weight)
                } else {
                    0.0
                };
                (item, key)
            })
            .collect();

        keyed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        keyed.into_iter().map(|(item, _)| item).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_one_returns_none_for_empty_collection() {
        let wc: WeightedCollection<&str> = WeightedCollection::new(vec![]);
        assert!(wc.pick_one().is_none());
    }

    #[test]
    fn pick_one_returns_sole_item_with_positive_weight() {
        let wc = WeightedCollection::new(vec![("only", 1.0)]);
        assert_eq!(wc.pick_one(), Some("only"));
    }

    #[test]
    fn pick_one_returns_none_for_sole_item_with_non_positive_weight() {
        let wc = WeightedCollection::new(vec![("only", 0.0)]);
        assert!(wc.pick_one().is_none());
        let wc = WeightedCollection::new(vec![("only", -1.0)]);
        assert!(wc.pick_one().is_none());
    }

    #[test]
    fn pick_one_favors_higher_weight() {
        let wc = WeightedCollection::new(vec![("low", 0.01), ("high", 100.0)]);
        let mut high_count = 0;
        let trials = 1000;
        for _ in 0..trials {
            if wc.pick_one() == Some("high") {
                high_count += 1;
            }
        }
        assert!(
            high_count > trials * 9 / 10,
            "high-weight item should be picked >90% of the time, was {high_count}/{trials}"
        );
    }

    #[test]
    fn pick_one_returns_none_for_all_non_positive_weights() {
        let wc = WeightedCollection::new(vec![("a", 0.0), ("b", -1.0)]);
        assert!(wc.pick_one().is_none());
    }

    #[test]
    fn pick_index_returns_valid_index() {
        let wc = WeightedCollection::new(vec![("a", 1.0), ("b", 2.0), ("c", 3.0)]);
        for _ in 0..100 {
            let idx = wc.pick_index().expect("should pick an index");
            assert!(idx < 3);
        }
    }

    #[test]
    fn pick_index_returns_none_for_non_positive_weights() {
        let wc = WeightedCollection::new(vec![("a", 0.0), ("b", -5.0)]);
        assert!(wc.pick_index().is_none());
    }

    #[test]
    fn shuffled_preserves_all_items() {
        let items: Vec<(u32, f64)> = (0..10).map(|i| (i, (i as f64 + 1.0) * 0.1)).collect();
        let shuffled = WeightedCollection::new(items).into_shuffled();
        assert_eq!(shuffled.len(), 10);
        let mut sorted = shuffled.clone();
        sorted.sort();
        assert_eq!(sorted, (0..10).collect::<Vec<_>>());
    }

    #[test]
    fn shuffled_favors_higher_weight_items() {
        let items = vec![("low", 0.1), ("high", 10.0)];
        let mut high_first_count = 0;
        let trials = 1000;
        for _ in 0..trials {
            let shuffled = WeightedCollection::new(items.clone()).into_shuffled();
            if shuffled[0] == "high" {
                high_first_count += 1;
            }
        }
        assert!(
            high_first_count > trials * 8 / 10,
            "high-weight item should appear first >80% of the time, was {high_first_count}/{trials}"
        );
    }

    #[test]
    fn shuffled_empty_collection_returns_empty() {
        let wc: WeightedCollection<&str> = WeightedCollection::new(vec![]);
        assert!(wc.into_shuffled().is_empty());
    }
}
