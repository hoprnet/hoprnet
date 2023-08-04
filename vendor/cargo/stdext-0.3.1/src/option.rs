//! Extension traits for `std::Option`.

/// Extension trait with useful methods for [`std::option::Option`].
///
/// [`std::time::Option`]: https://doc.rust-lang.org/std/option/enum.Option.html
pub trait OptionExt<T> {
    /// Combines `self` and another `Option`.
    ///
    /// If `self` is `Some(s)` and `other` is `Some(o)`, this method returns `Some((s, o))`.
    /// Otherwise, `None` is returned.
    ///
    /// **Note:** `std::Option` already provides a [`zip`] method which serves exact same purpose,
    /// but currently it's unstable. Once it's stabilized, this method will be marked as deprecated
    /// with a prompt to use the stanard method instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use stdext::prelude::*;
    ///
    /// let x = Some(1);
    /// let y = Some("hi");
    /// let z = None::<u8>;
    ///
    /// assert_eq!(x.combine(y), Some((1, "hi")));
    /// assert_eq!(x.combine(z), None);
    /// ```
    ///
    /// [`zip`]: https://doc.rust-lang.org/std/option/enum.Option.html#method.zip
    fn combine<U>(self, other: Option<U>) -> Option<(T, U)>;

    /// Combines `self` and another `Option` with function `f`.
    ///
    /// If `self` is `Some(s)` and `other` is `Some(o)`, this method returns `Some(f(s, o))`.
    /// Otherwise, `None` is returned.
    ///
    /// **Note:** `std::Option` already provides a [`zip_with`] method which serves exact same purpose,
    /// but currently it's unstable. Once it's stabilized, this method will be marked as deprecated
    /// with a prompt to use the stanard method instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use stdext::prelude::*;
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Point {
    ///     x: f64,
    ///     y: f64,
    /// }
    ///
    /// impl Point {
    ///     fn new(x: f64, y: f64) -> Self {
    ///         Self { x, y }
    ///     }
    /// }
    ///
    /// let x = Some(17.5);
    /// let y = Some(42.7);
    ///
    /// assert_eq!(x.combine_with(y, Point::new), Some(Point { x: 17.5, y: 42.7 }));
    /// assert_eq!(x.combine_with(None, Point::new), None);
    /// ```
    ///
    /// [`zip_with`]: https://doc.rust-lang.org/std/option/enum.Option.html#method.zip_with
    fn combine_with<U, F, R>(self, other: Option<U>, f: F) -> Option<R>
    where
        F: FnOnce(T, U) -> R;
}

impl<T> OptionExt<T> for Option<T> {
    fn combine<U>(self, other: Option<U>) -> Option<(T, U)> {
        match (self, other) {
            (Some(left), Some(right)) => Some((left, right)),
            _ => None,
        }
    }

    fn combine_with<U, F, R>(self, other: Option<U>, f: F) -> Option<R>
    where
        F: FnOnce(T, U) -> R,
    {
        self.combine(other).map(|(l, r)| f(l, r))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combine() {
        // Test vector of (left, right, expected) values.
        let test_vector = vec![
            (Some(1), Some(2), Some((1, 2))),
            (Some(1), None, None),
            (None, Some(2), None),
            (None, None, None),
        ];

        for (left, right, expected) in test_vector {
            assert_eq!(left.combine(right), expected);
        }
    }

    #[test]
    fn combine_with() {
        fn f(l: i32, r: i32) -> i32 {
            l + r
        }

        // Test vector of (left, right, expected) values.
        let test_vector = vec![
            (Some(1), Some(2), Some(3)),
            (Some(1), None, None),
            (None, Some(2), None),
            (None, None, None),
        ];

        for (left, right, expected) in test_vector {
            assert_eq!(left.combine_with(right, f), expected);
        }
    }
}
