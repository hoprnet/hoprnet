//! Small library providing some macros helpful for asserting. The API is very
//! similar to the API provided by the stdlib's own
//! [`assert_eq!`](core::assert_eq), [`assert_ne!`](core::assert_ne),
//! [`debug_assert_eq!`](core::debug_assert_eq), and
//! [`debug_assert_ne!`](core::debug_assert_ne).
//!
//! | Name                 | Enabled                       | Equivalent to                                |
//! | -------------------- | ----------------------------- | -------------------------------------------- |
//! | `assert_le!`         | Always                        | `assert!(a <= b)`                            |
//! | `assert_lt!`         | Always                        | `assert!(a < b)`                             |
//! | `assert_ge!`         | Always                        | `assert!(a >= b)`                            |
//! | `assert_gt!`         | Always                        | `assert!(a > b)`                             |
//! | `debug_assert_le!`   | `if cfg!(debug_assertions)`   | `debug_assert!(a <= b)`                      |
//! | `debug_assert_lt!`   | `if cfg!(debug_assertions)`   | `debug_assert!(a < b)`                       |
//! | `debug_assert_ge!`   | `if cfg!(debug_assertions)`   | `debug_assert!(a >= b)`                      |
//! | `debug_assert_gt!`   | `if cfg!(debug_assertions)`   | `debug_assert!(a > b)`                       |
//! | `debug_unreachable!` | `if cfg!(debug_assertions)`   | `unreachable!` when debug_assertions are on. |
//!
//! When one of the assertions fails, it prints out a message like the
//! following:
//!
//! ```text
//! thread 'main' panicked at 'assertion failed: `left < right`
//!   left: `4`,
//!  right: `3`', src/main.rs:47:5
//! note: Run with `RUST_BACKTRACE=1` for a backtrace.
//! ```
//!
//! # Example
//!
//! ```rust
//! use more_asserts as ma;
//!
//! #[derive(Debug, PartialEq, PartialOrd)]
//! enum Example { Foo, Bar }
//!
//! ma::assert_le!(3, 4);
//! ma::assert_ge!(
//!     10, 10,
//!     "You can pass a message too (just like `assert_eq!`)",
//! );
//! ma::debug_assert_lt!(
//!     1.3, 4.5,
//!     "Format syntax is supported ({}).",
//!     "also like `assert_eq!`"
//! );
//!
//! ma::assert_gt!(
//!     Example::Bar, Example::Foo,
//!     "It works on anything that implements PartialOrd and Debug!",
//! );
//! ```
#![no_std]
#![deny(missing_docs)]

mod inner;

// From use with macros. Not public API.
#[doc(hidden)]
pub extern crate core as __core;

// From use with macros. Not public API.
#[doc(hidden)]
pub mod __private {
    pub use crate::inner::AssertType;
    // Wrap the outlined functions with generic versions in an effort to improve
    // the error message given when using one of these macros on a type which
    // doesn't impl `core::fmt::Debug`.
    #[cold]
    #[track_caller]
    pub fn assert_failed_nomsg<A, B>(left: &A, right: &B, ty: AssertType) -> !
    where
        A: core::fmt::Debug,
        B: core::fmt::Debug,
    {
        crate::inner::assert_failed_nomsg_impl(left, right, ty);
    }

    #[cold]
    #[track_caller]
    #[doc(hidden)]
    pub fn assert_failed_msg<A, B>(
        left: &A,
        right: &B,
        ty: AssertType,
        msg: core::fmt::Arguments<'_>,
    ) -> !
    where
        A: core::fmt::Debug,
        B: core::fmt::Debug,
    {
        crate::inner::assert_failed_msg_impl(left, right, ty, msg);
    }
}

/// Panics if the first expression is not strictly less than the second.
///
/// Requires that the values implement [`Debug`](core::fmt::Debug) and
/// [`PartialOrd`](core::cmp::PartialOrd).
///
/// On failure, panics and prints the values out in a manner similar to
/// [`assert_eq!`](core::assert_eq).
///
/// # Example
///
/// ```rust
/// use more_asserts as ma;
///
/// ma::assert_lt!(3, 4);
/// ma::assert_lt!(3, 4, "With a message");
/// ma::assert_lt!(3, 4, "With a formatted message: {}", "oh no");
/// ```
#[macro_export]
macro_rules! assert_lt {
    ($left:expr, $right:expr) => {
        match (&$left, &$right) {
            (left, right) => if !(left < right) {
                $crate::__private::assert_failed_nomsg(
                    left, right, $crate::__private::AssertType::Lt,
                );
            }
        }
    };
    ($left:expr, $right:expr, ) => {
        $crate::assert_lt!($left, $right)
    };
    ($left:expr, $right:expr, $($msg_args:tt)+) => {
        match (&$left, &$right) {
            (left, right) => if !(left < right) {
                $crate::__private::assert_failed_msg(
                    left, right, $crate::__private::AssertType::Lt,
                    $crate::__core::format_args!($($msg_args)+),
                );
            }
        }
    };
}

/// Panics if the first expression is not strictly greater than the second.
///
/// Requires that the values implement [`Debug`](core::fmt::Debug) and
/// [`PartialOrd`](core::cmp::PartialOrd).
///
/// On failure, panics and prints the values out in a manner similar to
/// prelude's [`assert_eq!`](core::assert_eq).
///
/// # Example
///
/// ```rust
/// use more_asserts as ma;
///
/// ma::assert_gt!(5, 3);
/// ma::assert_gt!(5, 3, "With a message");
/// ma::assert_gt!(5, 3, "With a formatted message: {}", "oh no");
/// ```
#[macro_export]
macro_rules! assert_gt {
    ($left:expr, $right:expr) => {
        match (&$left, &$right) {
            (left, right) => if !(left > right) {
                $crate::__private::assert_failed_nomsg(
                    left, right, $crate::__private::AssertType::Gt,
                );
            }
        }
    };
    ($left:expr, $right:expr, ) => {
        $crate::assert_gt!($left, $right)
    };
    ($left:expr, $right:expr, $($msg_args:tt)+) => {
        match (&$left, &$right) {
            (left, right) => if !(left > right) {
                $crate::__private::assert_failed_msg(
                    left, right, $crate::__private::AssertType::Gt,
                    $crate::__core::format_args!($($msg_args)+),
                );
            }
        }
    };
}

/// Panics if the first expression is not less than or equal to the second.
///
/// Requires that the values implement [`Debug`](core::fmt::Debug) and
/// [`PartialOrd`](core::cmp::PartialOrd).
///
/// On failure, panics and prints the values out in a manner similar to
/// prelude's [`assert_eq!`](core::assert_eq).
///
/// # Example
///
/// ```rust
/// use more_asserts as ma;
///
/// ma::assert_le!(4, 4);
/// ma::assert_le!(4, 5);
/// ma::assert_le!(4, 5, "With a message");
/// ma::assert_le!(4, 4, "With a formatted message: {}", "oh no");
/// ```
#[macro_export]
macro_rules! assert_le {
    ($left:expr, $right:expr) => {
        match (&$left, &$right) {
            (left, right) => if !(left <= right) {
                $crate::__private::assert_failed_nomsg(
                    left, right, $crate::__private::AssertType::Le,
                );
            }
        }
    };
    ($left:expr, $right:expr, ) => {
        $crate::assert_le!($left, $right)
    };
    ($left:expr, $right:expr, $($msg_args:tt)+) => {
        match (&$left, &$right) {
            (left, right) => if !(left <= right) {
                $crate::__private::assert_failed_msg(
                    left, right, $crate::__private::AssertType::Le,
                    $crate::__core::format_args!($($msg_args)+),
                );
            }
        }
    };
}

/// Panics if the first expression is not greater than or equal to the second.
///
/// Requires that the values implement [`Debug`](core::fmt::Debug) and
/// [`PartialOrd`](core::cmp::PartialOrd).
///
/// On failure, panics and prints the values out in a manner similar to
/// prelude's [`assert_eq!`](core::assert_eq).
///
/// Optionally may take an additional message to display on failure, which is
/// formatted using standard format syntax.
///
/// # Example
///
/// ```rust
/// use more_asserts as ma;
///
/// ma::assert_ge!(4, 4);
/// ma::assert_ge!(4, 3);
/// ma::assert_ge!(4, 3, "With a message");
/// ma::assert_ge!(4, 4, "With a formatted message: {}", "oh no");
/// ```
#[macro_export]
macro_rules! assert_ge {
    ($left:expr, $right:expr) => {
        match (&$left, &$right) {
            (left, right) => if !(left >= right) {
                $crate::__private::assert_failed_nomsg(
                    left, right, $crate::__private::AssertType::Ge,
                );
            }
        }
    };
    ($left:expr, $right:expr, ) => {
        $crate::assert_ge!($left, $right)
    };
    ($left:expr, $right:expr, $($msg_args:tt)+) => {
        match (&$left, &$right) {
            (left, right) => if !(left >= right) {
                $crate::__private::assert_failed_msg(
                    left, right, $crate::__private::AssertType::Ge,
                    $crate::__core::format_args!($($msg_args)+),
                );
            }
        }
    };
}

/// Same as [`assert_lt!`] in builds with debug assertions enabled, and a no-op
/// otherwise.
///
/// # Example
///
/// ```rust
/// use more_asserts as ma;
///
/// // These are compiled to nothing if debug_assertions are off!
/// ma::debug_assert_lt!(3, 4);
/// ma::debug_assert_lt!(3, 4, "With a message");
/// ma::debug_assert_lt!(3, 4, "With a formatted message: {}", "oh no");
/// ```
#[macro_export]
macro_rules! debug_assert_lt {
    ($($arg:tt)+) => {
        if $crate::__core::cfg!(debug_assertions) {
            $crate::assert_lt!($($arg)+);
        }
    };
}

/// Same as [`assert_gt!`] in builds with debug assertions enabled, and a no-op
/// otherwise.
///
/// # Example
///
/// ```rust
/// use more_asserts as ma;
///
/// // These are compiled to nothing if debug_assertions are off!
/// ma::debug_assert_gt!(5, 3);
/// ma::debug_assert_gt!(5, 3, "With a message");
/// ma::debug_assert_gt!(5, 3, "With a formatted message: {}", "oh no");
/// ```
#[macro_export]
macro_rules! debug_assert_gt {
    ($($arg:tt)+) => {
        if $crate::__core::cfg!(debug_assertions) {
            $crate::assert_gt!($($arg)+);
        }
    };
}

/// Same as [`assert_le!`] in builds with debug assertions enabled, and a no-op
/// otherwise.
///
/// # Example
///
/// ```rust
/// use more_asserts as ma;
///
/// // These are compiled to nothing if debug_assertions are off!
/// ma::debug_assert_le!(4, 4);
/// ma::debug_assert_le!(4, 5);
/// ma::debug_assert_le!(4, 5, "With a message");
/// ma::debug_assert_le!(4, 4, "With a formatted message: {}", "oh no");
/// ```
#[macro_export]
macro_rules! debug_assert_le {
    ($($arg:tt)+) => {
        if $crate::__core::cfg!(debug_assertions) {
            $crate::assert_le!($($arg)+);
        }
    };
}

/// Same as [`assert_ge!`] in builds with debug assertions enabled, and a no-op
/// otherwise.
///
/// # Example
///
/// ```rust
/// use more_asserts as ma;
///
/// // These are compiled to nothing if debug_assertions are off!
/// ma::debug_assert_ge!(4, 4);
/// ma::debug_assert_ge!(4, 3);
/// ma::debug_assert_ge!(4, 3, "With a message");
/// ma::debug_assert_ge!(4, 4, "With a formatted message: {}", "oh no");
/// ```
#[macro_export]
macro_rules! debug_assert_ge {
    ($($arg:tt)+) => {
        if $crate::__core::cfg!(debug_assertions) {
            $crate::assert_ge!($($arg)+);
        }
    };
}

/// Panics if reached when debug assertions are enabled.
///
/// This is a variant of the standard library's [`unreachable!`] macro that is
/// controlled by `cfg!(debug_assertions)`. For builds without debug assertions
/// enabled (such as typical release builds), it is a no-op.
///
/// # Example
///
/// ```rust
/// use more_asserts as ma;
///
/// let mut value = 0.5;
/// if value < 0.0 {
///     ma::debug_unreachable!("Value out of range {}", value);
///     value = 0.0;
/// }
/// ```
#[macro_export]
macro_rules! debug_unreachable {
    ($($arg:tt)*) => {
        if $crate::__core::cfg!(debug_assertions) {
            $crate::__core::unreachable!($($arg)*);
        }
    };
}
