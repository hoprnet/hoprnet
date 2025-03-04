use std::future::Future;

/// Conversion into a `Future`.
///
/// By implementing `Intofuture` for a type, you define how it will be
/// converted to a future. This is common for types which describe an
/// asynchronous builder of some kind.
///
/// One benefit of implementing `IntoFuture` is that your type will [work
/// with Rust's `.await` syntax](https://doc.rust-lang.org/std/keyword.await.html).
///
/// # Examples
///
/// Basic usage:
///
/// ```no_run
/// use futures_time::future::IntoFuture;
///
/// # async fn foo() {
/// let v = async { "meow" };
/// let mut fut = v.into_future();
/// assert_eq!("meow", fut.await);
/// # }
/// ```
///
/// It is common to use `IntoFuture` as a trait bound. This allows
/// the input type to change, so long as it is still a future.
/// Additional bounds can be specified by restricting on `Output`:
///
/// ```rust
/// use futures_time::future::IntoFuture;
/// async fn fut_to_string<Fut>(fut: Fut) -> String
/// where
///     Fut: IntoFuture,
///     Fut::Output: std::fmt::Debug,
/// {
///     format!("{:?}", fut.into_future().await)
/// }
/// ```
pub trait IntoFuture {
    /// The output that the future will produce on completion.
    type Output;

    /// Which kind of future are we turning this into?
    type IntoFuture: Future<Output = Self::Output>;

    /// Creates a future from a value.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_run
    /// use futures_time::future::IntoFuture;
    ///
    /// # async fn foo() {
    /// let v = async { "meow" };
    /// let mut fut = v.into_future();
    /// assert_eq!("meow", fut.await);
    /// # }
    /// ```
    fn into_future(self) -> Self::IntoFuture;
}

impl<F: Future> IntoFuture for F {
    type Output = F::Output;
    type IntoFuture = F;

    fn into_future(self) -> Self::IntoFuture {
        self
    }
}
