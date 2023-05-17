/// Iterate over `r`-length subsequences of elements from `values`.
///
/// Combinations are emitted in lexicographic sort order. So, if the
/// input iterable is sorted, the combination tuples will be produced
/// in sorted order.
///
/// Elements are treated as unique based on their position, not on
/// their value. So if the input elements are unique, there will be no
/// repeat values in each combination.
///
/// For a `values` vector of length *n*, the number of items emitted
/// is *n! / r! / (n-r)!* when *0 <= r <= n* or zero when *r > n*.
///
/// # Arguments
///
/// * `values` - A vector of values from which the combinations are
/// chosen
///
/// * `r` - The length of the emitted combinations
///
/// * `fun` - The function to iterate over the combinations
///
/// # See also
///
/// This function gleefully stolen from Python
/// [`itertools.combinations`](http://docs.python.org/2/library/itertools.html#itertools.combinations).
#[inline]
pub fn each_combination<T, F>(values: &[T], r: usize, mut fun: F) -> ()
where
    T: Clone,
    F: FnMut(&[T]) -> (),
{
    let length = values.len();
    if r == 0 || r > length {
        return;
    }
    let max_indices0 = length - r;
    let mut indices: Vec<usize> = (0..r).collect();
    let mut combination: Vec<T> = values[0..r].iter().cloned().collect();
    loop {
        fun(&combination);
        // Increment the indices
        let mut i = r - 1;
        indices[i] += 1;
        while i > 0 && indices[i] > max_indices0 + i {
            // indices[i] is now too large; decrement i, increment the
            // new indices[i] and we'll fix up the following indices
            // later
            i -= 1;
            indices[i] += 1;
        }
        // Can't fix up 'done'
        if indices[0] > max_indices0 {
            break;
        }
        // Fix up the indices and the combination from i to r-1
        combination[i] = values[indices[i]].clone();
        for i in i + 1..r {
            indices[i] = indices[i - 1] + 1;
            combination[i] = values[indices[i]].clone();
        }
    }
}

/// Iterate over `r`-length subsequences of elements from `values`.
///
/// This is an alternative to each_combination that uses references to
/// avoid copying the elements of the values vector.
///
/// To avoid memory allocations and copying, the iterator will be
/// passed a reference to a vector containing references to the
/// elements in the original `values` vector.
///
/// # Arguments
///
/// * `values` - A vector of values from which the combinations are
/// chosen
///
/// * `r` - The length of the emitted combinations
///
/// * `fun` - The function to iterate over the combinations
#[inline]
pub fn each_combination_ref<'v, T, F>(values: &'v [T], r: usize, fun: F) -> ()
where
    F: FnMut(&[&'v T]) -> (),
{
    let v: Vec<&T> = values.iter().map(|elt| elt).collect();
    each_combination(&v, r, fun);
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_zero() {
        let values = [1, 2, 3, 4];
        let mut v: Vec<Vec<i32>> = Vec::new();
        super::each_combination(&values, 0, |p| {
            v.push(p.iter().cloned().collect());
        });
        assert!(v.is_empty());
    }

    #[test]
    fn test_one() {
        let values = [1, 2, 3, 4];
        let mut v: Vec<Vec<i32>> = Vec::new();
        super::each_combination(&values, 1, |p| {
            v.push(p.iter().cloned().collect());
        });
        assert!(v == [[1], [2], [3], [4]]);
    }

    #[test]
    fn test_two() {
        let values = [1, 2, 3, 4];
        let mut v: Vec<Vec<i32>> = Vec::new();
        super::each_combination(&values, 2, |p| {
            v.push(p.iter().cloned().collect());
        });
        assert!(v == [[1, 2], [1, 3], [1, 4], [2, 3], [2, 4], [3, 4]]);
    }

    #[test]
    fn test_three() {
        let values = [1, 2, 3, 4];
        let mut v: Vec<Vec<i32>> = Vec::new();
        super::each_combination(&values, 3, |p| {
            v.push(p.iter().cloned().collect());
        });
        assert!(v == [[1, 2, 3], [1, 2, 4], [1, 3, 4], [2, 3, 4]]);
    }

    #[test]
    fn test_four() {
        let values = [1, 2, 3, 4];
        let mut v: Vec<Vec<i32>> = Vec::new();
        super::each_combination(&values, 4, |p| {
            v.push(p.iter().cloned().collect());
        });
        assert!(v == [[1, 2, 3, 4]]);
    }

    #[test]
    fn test_five() {
        let values = [1, 2, 3, 4];
        let mut v: Vec<Vec<i32>> = Vec::new();
        super::each_combination(&values, 5, |p| {
            v.push(p.iter().cloned().collect());
        });
        assert!(v.is_empty());
    }
}
