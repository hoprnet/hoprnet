//! This module has extras that clash with names in [`futures-lite`],
//! which depends on us.
pub use crate::*;

/// Zips two futures, waiting for both to complete.
///
/// # Examples
///
/// ```
/// use futures_micro::prelude::zip;
///
/// # futures_lite::future::block_on(async {
/// let a = async { 1 };
/// let b = async { 2 };
///
/// assert_eq!(zip(a, b).await, (1, 2));
/// # })
/// ```
pub fn zip<F1, F2>(f1: F1, f2: F2) -> Zip<F1, F2>
where
    F1: Future,
    F2: Future,
{
    Zip::new(f1, f2)
}

/// Returns the result of `left` or `right` future, preferring `left` if both are ready.
pub fn or<F1, F2>(future1: F1, future2: F2) -> Or<F1, F2>
where
    F1: Future,
    F2: Future,
{
    Or { future1, future2 }
}
