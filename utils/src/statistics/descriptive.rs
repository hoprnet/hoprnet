/// Returns the median of `values`, sorting them in place.
/// Returns `None` if the slice is empty.
pub fn median(values: &mut [f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = values.len() / 2;
    if values.len().is_multiple_of(2) {
        Some((values[mid - 1] + values[mid]) / 2.0)
    } else {
        Some(values[mid])
    }
}

/// Returns the population variance of `values`.
/// Returns `None` if the slice is empty.
pub fn variance(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mean = values.iter().copied().sum::<f64>() / values.len() as f64;
    Some(values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64)
}

/// Returns the population standard deviation of `values`.
/// Returns `None` if the slice is empty.
pub fn std_dev(values: &[f64]) -> Option<f64> {
    variance(values).map(f64::sqrt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_empty_returns_none() {
        assert_eq!(median(&mut []), None);
    }

    #[test]
    fn median_single_returns_value() {
        assert_eq!(median(&mut [3.0]), Some(3.0));
    }

    #[test]
    fn median_odd_count() {
        assert_eq!(median(&mut [3.0, 1.0, 2.0]), Some(2.0));
    }

    #[test]
    fn median_even_count_averages_middle_two() {
        assert_eq!(median(&mut [4.0, 1.0, 3.0, 2.0]), Some(2.5));
    }

    #[test]
    fn variance_empty_returns_none() {
        assert_eq!(variance(&[]), None);
    }

    #[test]
    fn variance_constant_input_is_zero() {
        assert_eq!(variance(&[5.0, 5.0, 5.0]), Some(0.0));
    }

    #[test]
    fn variance_known_dataset() {
        // [2,4,4,4,5,5,7,9] → mean=5, variance=4
        let v = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let result = variance(&v).expect("non-empty");
        assert!((result - 4.0).abs() < 1e-10, "expected 4.0, got {result}");
    }

    #[test]
    fn std_dev_empty_returns_none() {
        assert_eq!(std_dev(&[]), None);
    }

    #[test]
    fn std_dev_known_dataset() {
        let v = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let result = std_dev(&v).expect("non-empty");
        assert!((result - 2.0).abs() < 1e-10, "expected 2.0, got {result}");
    }
}
