//! A very small, no-std compatible toolbox of async utilities.
#![no_std]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

pub mod prelude;

#[doc(no_inline)]
pub use core::future::Future;
#[doc(no_inline)]
pub use core::pin::Pin;
#[doc(no_inline)]
pub use core::task::{Context, Poll, Waker};

use core::fmt;

use pin_project_lite::pin_project;

// ---------- futures using the poll api -----------

/// Creates a future from a function returning [`Poll`].
///
/// # Examples
///
/// ```
/// use futures_lite::future::block_on;
/// use futures_micro::poll_fn;
/// use std::task::{Context, Poll};
///
/// # block_on(async {
/// fn f(_ctx: &mut Context<'_>) -> Poll<i32> {
///     Poll::Ready(7)
/// }
///
/// assert_eq!(poll_fn(f).await, 7);
/// # })
/// ```
#[inline(always)]
pub fn poll_fn<F, T>(inner: F) -> PollFn<F>
where
    F: FnMut(&mut Context<'_>) -> Poll<T>,
{
    PollFn { inner }
}

pin_project! {
    /// Future for the [`poll_fn()`] function.
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct PollFn<F> {
        inner: F,
    }
}

impl<F> fmt::Debug for PollFn<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PollFn").finish()
    }
}

impl<F, T> Future for PollFn<F>
where
    F: FnMut(&mut Context<'_>) -> Poll<T>,
{
    type Output = T;
    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<T> {
        let this = self.project();
        (this.inner)(ctx)
    }
}

pin_project! {
    /// Returns the result of `left` or `right` future, preferring `left` if both are ready.
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct Or<F1, F2> {
        #[pin]
        future1: F1,
        #[pin]
        future2: F2,
    }
}

impl<F1, F2> Or<F1, F2>
where
    F1: Future,
    F2: Future,
{
    /// Returns the result of `left` or `right` future, preferring `left` if both are ready.
    pub fn new(future1: F1, future2: F2) -> Self {
        Or { future1, future2 }
    }
}

impl<T, F1, F2> Future for Or<F1, F2>
where
    F1: Future<Output = T>,
    F2: Future<Output = T>,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        if let Poll::Ready(t) = this.future1.poll(cx) {
            Poll::Ready(t)
        } else if let Poll::Ready(t) = this.future2.poll(cx) {
            Poll::Ready(t)
        } else {
            Poll::Pending
        }
    }
}

pin_project! {
    /// Waits for two [`Future`]s to complete, returning both results.
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct Zip<F1, F2>
    where
        F1: Future,
        F2: Future,
    {
        #[pin]
        future1: F1,
        output1: Option<F1::Output>,
        #[pin]
        future2: F2,
        output2: Option<F2::Output>,
    }
}

impl<F1, F2> Zip<F1, F2>
where
    F1: Future,
    F2: Future,
{
    /// Zips two futures, waiting for both to complete.
    ///
    /// # Examples
    ///
    /// ```
    /// use futures_micro::Zip;
    ///
    /// # futures_lite::future::block_on(async {
    /// let a = async { 1 };
    /// let b = async { 2 };
    ///
    /// assert_eq!(Zip::new(a, b).await, (1, 2));
    /// # })
    /// ```
    pub fn new(future1: F1, future2: F2) -> Self {
        Zip {
            future1,
            future2,
            output1: None,
            output2: None,
        }
    }
}

impl<F1, F2> Future for Zip<F1, F2>
where
    F1: Future,
    F2: Future,
{
    type Output = (F1::Output, F2::Output);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        if this.output1.is_none() {
            if let Poll::Ready(out) = this.future1.poll(cx) {
                *this.output1 = Some(out);
            }
        }

        if this.output2.is_none() {
            if let Poll::Ready(out) = this.future2.poll(cx) {
                *this.output2 = Some(out);
            }
        }

        if this.output1.is_some() && this.output2.is_some() {
            Poll::Ready((this.output1.take().unwrap(), this.output2.take().unwrap()))
        } else {
            Poll::Pending
        }
    }
}

/// Get the [`Waker`] inside an async fn where you aren't supposed to
/// have it.
///
/// This is a low level primitive for implementing more complex
/// patterns while avoiding the [`Poll`] API.
///
/// # Examples
///
/// ```
/// use futures_micro::{sleep, waker};
///
/// # futures_lite::future::block_on(async {
/// let waker = waker().await;
/// assert_eq!(async { waker.wake(); sleep().await; 1 }.await, 1)
/// # })
/// ```
pub fn waker() -> impl Future<Output = Waker> {
    poll_fn(|ctx| Poll::Ready(ctx.waker().clone()))
}

/// Goes to sleep until woken by its [`Waker`] being called.
///
/// This is a low level primitive for implementing more complex
/// patterns while avoiding the [`Poll`] API.
///
/// # Examples
///
/// ```
/// use futures_micro::{sleep, waker};
///
/// # futures_lite::future::block_on(async {
/// let waker = waker().await;
/// assert_eq!(async { waker.wake(); sleep().await; 1 }.await, 1)
/// # })
/// ```
pub fn sleep() -> impl Future<Output = ()> {
    let mut done = false;
    poll_fn(move |_| {
        if done {
            Poll::Ready(())
        } else {
            done = true;
            Poll::Pending
        }
    })
}


/// Pushes itself to the back of the executor queue so some other
/// tasks can do some work.
pub fn yield_once() -> impl Future<Output = ()> {
    let mut done = false;
    poll_fn(move |ctx| {
        if done {
            Poll::Ready(())
        } else {
            done = true;
            ctx.waker().wake_by_ref();
            Poll::Pending        }
    })
}

// --------- MACROS ---------

// Helper for `or!`
#[doc(hidden)]
#[macro_export]
macro_rules! __internal_fold_with {
    ($func:path, $e:expr) => { $e };
    ($func:path, $e:expr, $($es:expr),+) => {
        $func($e, $crate::__internal_fold_with!($func, $($es),+))
    };
}

/// Polls arbitrarily many futures, returning the first ready value.
///
/// All futures must have the same output type. Left biased when more
/// than one Future is ready at the same time.
#[macro_export]
macro_rules! or {
    ($($es:expr),+$(,)?) => { $crate::__internal_fold_with!($crate::Or::new, $($es),+) };
}

/// Pins a variable of type `T` on the stack and rebinds it as `Pin<&mut T>`.
///
/// ```
/// use futures_micro::*;
/// use std::fmt::Debug;
/// use std::time::Instant;
///
/// // Inspects each invocation of `Future::poll()`.
/// async fn inspect<T: Debug>(f: impl Future<Output = T>) -> T {
///     pin!(f);
///     poll_fn(|cx| dbg!(f.as_mut().poll(cx))).await
/// }
///
/// # futures_lite::future::block_on(async {
/// let f = async { 1 + 2 };
/// inspect(f).await;
/// # })
/// ```
#[macro_export]
macro_rules! pin {
    ($($x:ident),* $(,)?) => {
        $(
            let mut $x = $x;
            #[allow(unused_mut)]
            let mut $x = unsafe {
                core::pin::Pin::new_unchecked(&mut $x)
            };
        )*
    }
}

/// Unwraps `Poll<T>` or returns [`Pending`][`Poll::Pending`].
///
/// # Examples
///
/// ```
/// use futures_micro::*;
///
/// // Polls two futures and sums their results.
/// fn poll_sum(
///     cx: &mut Context<'_>,
///     mut a: impl Future<Output = i32> + Unpin,
///     mut b: impl Future<Output = i32> + Unpin,
/// ) -> Poll<i32> {
///     let x = ready!(Pin::new(&mut a).poll(cx));
///     let y = ready!(Pin::new(&mut b).poll(cx));
///     Poll::Ready(x + y)
/// }
/// ```
#[macro_export]
macro_rules! ready {
    ($e:expr $(,)?) => {
        match $e {
            core::task::Poll::Ready(t) => t,
            t @ core::task::Poll::Pending => return t,
        }
    };
}

/// Zips arbitrarily many futures, waiting for all to complete.
///
/// # Examples
///
/// ```
/// use futures_micro::zip;
///
/// # futures_lite::future::block_on(async {
/// let a = async { 1 };
/// let b = async { 2 };
///
/// assert_eq!(zip!(a, b).await, (1, 2));
/// # })
/// ```
#[macro_export]
macro_rules! zip {
    ($($es:expr),+ $(,)?) => {{
        let mut zips = $crate::__internal_fold_with!($crate::Zip::new, $($es),+);
        $crate::poll_fn(move |ctx| {
            use ::core::pin::Pin;
            use ::core::task::Poll;

            let zips = unsafe { Pin::new_unchecked(&mut zips) };
            if let Poll::Ready(val) = ::core::future::Future::poll(zips, ctx) {
                Poll::Ready($crate::zip!(@flatten; ; val; $($es),+))
            } else {
                Poll::Pending
            }
        })
    }};

    (@flatten; $($prev:expr,)*; $tuple:expr; $e:expr) => {
        ($($prev,)* $tuple)
    };

    (@flatten; $($prev:expr,)*; $tuple:expr; $e:expr, $($es:expr),+) => {
        $crate::zip!(@flatten; $($prev,)* $tuple.0,; $tuple.1; $($es),+)
    };
}
