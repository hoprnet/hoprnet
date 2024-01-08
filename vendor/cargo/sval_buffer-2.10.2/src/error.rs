use crate::std::fmt;

/**
An error encountered buffering data.
*/
#[derive(Debug)]
pub struct Error(ErrorKind);

#[derive(Debug)]
enum ErrorKind {
    Unsupported {
        actual: &'static str,
        expected: &'static str,
    },
    #[cfg(feature = "alloc")]
    OutsideContainer {
        method: &'static str,
    },
    InvalidValue {
        reason: &'static str,
    },
    #[cfg(not(feature = "alloc"))]
    #[allow(dead_code)]
    NoAlloc {
        method: &'static str,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            ErrorKind::Unsupported { actual, expected } => {
                write!(f, "unexpected {}, expected {}", actual, expected)
            }
            #[cfg(feature = "alloc")]
            ErrorKind::OutsideContainer { method } => {
                write!(f, "expected a fragment while buffering {}", method)
            }
            ErrorKind::InvalidValue { reason } => {
                write!(f, "the value being buffered is invalid: {}", reason)
            }
            #[cfg(not(feature = "alloc"))]
            ErrorKind::NoAlloc { method } => write!(f, "cannot allocate for {}", method),
        }
    }
}

impl Error {
    pub(crate) fn unsupported(expected: &'static str, actual: &'static str) -> Self {
        Error(ErrorKind::Unsupported { actual, expected })
    }

    #[cfg(feature = "alloc")]
    pub(crate) fn outside_container(method: &'static str) -> Self {
        Error(ErrorKind::OutsideContainer { method })
    }

    pub(crate) fn invalid_value(reason: &'static str) -> Self {
        Error(ErrorKind::InvalidValue { reason })
    }

    #[cfg(not(feature = "alloc"))]
    #[track_caller]
    pub(crate) fn no_alloc(method: &'static str) -> Self {
        /*
        Users may not know they aren't depending on an allocator when using `sval_buffer`
        and have buffering fail unexpectedly. In debug builds we provide a more developer-centric
        panic message if this happens so they can decide whether failure to buffer is acceptable
        or not, and enable features accordingly.

        If you're not depending on `sval_buffer` directly, but through another library like
        `sval_serde`, then you can enable their `alloc` or `std` features instead.
        */

        #[cfg(all(debug_assertions, not(no_debug_assertions), not(test)))]
        {
            panic!("attempt to allocate for {} would fail; add the `alloc` feature of `sval_buffer` or the depdendent `sval_*` library to support allocation. This call will error instead of panicking in release builds. Add the `no_debug_assertions` feature of `sval_buffer` if this error is expected.", method);
        }
        #[cfg(not(all(debug_assertions, not(no_debug_assertions), not(test))))]
        {
            Error(ErrorKind::NoAlloc { method })
        }
    }
}

#[cfg(feature = "std")]
mod std_support {
    use super::*;

    use crate::std::error;

    impl error::Error for Error {}
}
