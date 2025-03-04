use std::{future::Future, pin::Pin};

/// A future which holds a deadline relative to now.
///
/// This is a future which will trigger at some point in the future. Operations
/// such as `debounce`, which need to move their deadline forward every time an
/// item is received from ther underlying stream. This method provides a way to
/// ask a future to resolve at some point in the future instead.
pub trait Timer: Future {
    /// Move the point at which this future resolves to some point in the
    /// future. If the future has already resolved before, calling this method
    /// will allow it to resolve again.
    fn reset_timer(self: Pin<&mut Self>);
}
