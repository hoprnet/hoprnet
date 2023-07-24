//! Extension traits for `std::Result`.

/// Extension trait with useful methods for [`std::result::Result`].
///
/// [`std::result::Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
pub trait ResultExt<T, E> {
    /// Combines `self` and another `Result`.
    ///
    /// If `self` is `Ok(s)` and `other` is `Ok(o)`, this method returns `Ok((s, o))`.
    /// Otherwise, if the `self` is `Ok(s)` and `other` is `Err(e)`, this method returns `Err(e)`.
    /// Otherwise, `self` is `Err(e)` and this method returns `Err(e)` (`other` is not taken into
    /// account, as in short circuit calculations).
    ///
    /// # Examples
    ///
    /// ```
    /// use stdext::prelude::*;
    ///
    /// let x = Ok(1);
    /// let y = Ok("hi");
    /// let z: Result<i32, &str> = Err("error");
    /// let z2: Result<i32, &str> = Err("other_error");
    ///
    /// assert_eq!(x.combine(y), Ok((1, "hi")));
    /// assert_eq!(x.combine(z), Err("error"));
    /// assert_eq!(z.combine(z2), Err("error"));
    /// ```
    fn combine<U>(self, other: Result<U, E>) -> Result<(T, U), E>;

    /// Combines `self` and another `Result` with function `f`.
    ///
    /// If `self` is `Ok(s)` and `other` is `Ok(o)`, this method returns `Ok(f(s, o))`.
    /// Otherwise, if the `self` is `Ok(s)` and `other` is `Err(e)`, this method returns `Err(e)`.
    /// Otherwise, `self` is `Err(e)` and this method returns `Err(e)` (`other` is not taken into
    /// account, as in short circuit calculations).
    ///
    /// # Examples
    ///
    /// ```
    /// use stdext::prelude::*;
    ///
    /// let x = Ok(1);
    /// let y = Ok(2);
    /// let z: Result<i32, &str> = Err("error");
    /// let z2: Result<i32, &str> = Err("other_error");
    ///
    /// assert_eq!(x.combine_with(y, |l, r| l + r), Ok(3));
    /// assert_eq!(x.combine_with(z, |l, r| l + r), Err("error"));
    /// assert_eq!(z.combine_with(z2, |l, r| l + r), Err("error"));
    /// ```
    ///
    /// [`zip_with`]: https://doc.rust-lang.org/std/Result/enum.Result.html#method.zip_with
    fn combine_with<U, F, R>(self, other: Result<U, E>, f: F) -> Result<R, E>
    where
        F: FnOnce(T, U) -> R;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn combine<U>(self, other: Result<U, E>) -> Result<(T, U), E> {
        match (self, other) {
            (Ok(left), Ok(right)) => Ok((left, right)),
            (Ok(_), Err(err)) => Err(err),
            (Err(err), _) => Err(err),
        }
    }

    fn combine_with<U, F, R>(self, other: Result<U, E>, f: F) -> Result<R, E>
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
            (Ok(1), Ok(2), Ok((1, 2))),
            (Ok(1), Err("right"), Err("right")),
            (Err("left"), Ok(2), Err("left")),
            (Err("left"), Err("right"), Err("left")),
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
            (Ok(1), Ok(2), Ok(3)),
            (Ok(1), Err("right"), Err("right")),
            (Err("left"), Ok(2), Err("left")),
            (Err("left"), Err("right"), Err("left")),
        ];

        for (left, right, expected) in test_vector {
            assert_eq!(left.combine_with(right, f), expected);
        }
    }
}
