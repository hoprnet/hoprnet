/// Return early with an error.
#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return $crate::private::Err($crate::format_err!($msg))
    };
    ($msg:expr $(,)?) => {
        return $crate::private::Err($crate::format_err!($msg))
    };
    ($msg:expr, $($arg:tt)*) => {
        return $crate::private::Err($crate::format_err!($msg, $($arg)*))
    };
}

/// Return early with an error if a condition is not satisfied.
///
/// This macro is equivalent to `if !$cond { return Err(From::from($err)); }`.
///
/// Analogously to `assert!`, `ensure!` takes a condition and exits the function
/// if the condition fails. Unlike `assert!`, `ensure!` returns an `Error`
/// rather than panicking.
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $msg:literal $(,)?) => {
        if !$cond {
            return $crate::private::Err($crate::format_err!($msg))
        }
    };
    ($cond:expr, $msg:expr $(,)?) => {
        if !$cond {
            return $crate::private::Err($crate::format_err!($msg))
        }
    };
    ($cond:expr, $msg:expr, $($arg:tt)*) => {
        if !$cond {
            return $crate::private::Err($crate::format_err!($msg, $($arg)*))
        }
    };
}

/// Return early with an error if two expressions are not equal to each other.
///
/// This macro is equivalent to `if $left != $right { return Err(From::from($err)); }`.
///
/// Analogously to `assert_eq!`, `ensure_eq!` takes two expressions and exits the function
/// if the expressions are not equal. Unlike `assert_eq!`, `ensure_eq!` returns an `Error`
/// rather than panicking.
#[macro_export]
macro_rules! ensure_eq {
    ($left:expr, $right:expr, $msg:literal $(,)?) => {
        if $left != $right {
            return $crate::private::Err($crate::format_err!($msg))
        }
    };
    ($left:expr, $right:expr, $msg:expr $(,)?) => {
        if $left != $right {
            return $crate::private::Err($crate::format_err!($msg))
        }
    };
    ($left:expr, $right:expr, $msg:expr, $($arg:tt)*) => {
        if $left != $right {
            return $crate::private::Err($crate::format_err!($msg, $($arg)*))
        }
    };
}

/// Construct an ad-hoc error from a string.
///
/// This evaluates to an `Error`. It can take either just a string, or a format
/// string with arguments. It also can take any custom type which implements
/// `Debug` and `Display`.
#[macro_export]
macro_rules! format_err {
    ($msg:literal $(,)?) => {
        // Handle $:literal as a special case to make cargo-expanded code more
        // concise in the common case.
        $crate::private::new_adhoc($msg)
    };
    ($err:expr $(,)?) => ({
        let error = $err;
        Error::new_adhoc(error)
    });
    ($fmt:expr, $($arg:tt)*) => {
        $crate::private::new_adhoc(format!($fmt, $($arg)*))
    };
}

/// Return early with an error and a status code.
#[doc(hidden)]
#[macro_export]
macro_rules! bail_status {
    ($status:literal, $msg:literal $(,)?) => {{
        return $crate::private::Err($crate::format_err_status!($status, $msg))
    }};
    ($status:literal, $msg:expr $(,)?) => {
        return $crate::private::Err($crate::format_err_status!($status, $msg))
    };
    ($status:literal, $msg:expr, $($arg:tt)*) => {
        return $crate::private::Err($crate::format_err_status!($status, $msg, $($arg)*))
    };
}

/// Return early with an error if a condition is not satisfied.
///
/// This macro is equivalent to `if !$cond { return Err(From::from($err)); }`.
///
/// Analogously to `assert!`, `ensure!` takes a condition and exits the function
/// if the condition fails. Unlike `assert!`, `ensure!` returns an `Error`
/// rather than panicking.
#[doc(hidden)]
#[macro_export]
macro_rules! ensure_status {
    ($cond:expr, $status:literal, $msg:literal $(,)?) => {
        if !$cond {
            return $crate::private::Err($crate::format_err_status!($status, $msg))
        }
    };
    ($cond:expr, $status:literal, $msg:expr $(,)?) => {
        if !$cond {
            return $crate::private::Err($crate::format_err_status!($status, $msg))
        }
    };
    ($cond:expr, $status:literal, $msg:expr, $($arg:tt)*) => {
        if !$cond {
            return $crate::private::Err($crate::format_err_status!($status, $msg, $($arg)*))
        }
    };
}

/// Return early with an error if two expressions are not equal to each other.
///
/// This macro is equivalent to `if $left != $right { return Err(From::from($err)); }`.
///
/// Analogously to `assert_eq!`, `ensure_eq!` takes two expressions and exits the function
/// if the expressions are not equal. Unlike `assert_eq!`, `ensure_eq!` returns an `Error`
/// rather than panicking.
#[doc(hidden)]
#[macro_export]
macro_rules! ensure_eq_status {
    ($left:expr, $right:expr, $status:literal, $msg:literal $(,)?) => {
        if $left != $right {
            return $crate::private::Err($crate::format_err_status!($status, $msg))
        }
    };
    ($left:expr, $right:expr, $status:literal, $msg:expr $(,)?) => {
        if $left != $right {
            return $crate::private::Err($crate::format_err_status!($status, $msg))
        }
    };
    ($left:expr, $right:expr, $status:literal, $msg:expr, $($arg:tt)*) => {
        if $left != $right {
            return $crate::private::Err($crate::format_err_status!($status, $msg, $($arg)*))
        }
    };
}

/// Construct an ad-hoc error from a string.
///
/// This evaluates to an `Error`. It can take either just a string, or a format
/// string with arguments. It also can take any custom type which implements
/// `Debug` and `Display`.
#[doc(hidden)]
#[macro_export]
macro_rules! format_err_status {
    ($status:literal, $msg:literal $(,)?) => {{
        // Handle $:literal as a special case to make cargo-expanded code more
        // concise in the common case.
        let mut err = $crate::private::new_adhoc($msg);
        err.set_status($status);
        err
    }};
    ($status:literal, $msg:expr $(,)?) => {{
        let mut err = $crate::private::new_adhoc($msg);
        err.set_status($status);
        err
    }};
    ($status:literal, $msg:expr, $($arg:tt)*) => {{
        let mut err = $crate::private::new_adhoc(format!($msg, $($arg)*));
        err.set_status($status);
        err
    }};
}
