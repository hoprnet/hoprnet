use core::fmt::{self, Display, Formatter};

#[cfg(feature = "std")]
use std::error::Error;

/// Creates a static error description with file and line information
#[doc(hidden)]
#[macro_export]
macro_rules! e {
    () => {
        concat!("@", file!(), ":", line!())
    };
    ($str:expr) => {
        concat!($str, " @", file!(), ":", line!())
    };
}

/// Creates an `InOutError` variant
#[doc(hidden)]
#[macro_export]
macro_rules! eio {
    ($str:expr) => {
        $crate::error::Asn1DerError::new($crate::error::Asn1DerErrorVariant::InOutError(e!($str)))
    };
}
/// Creates an `InvalidData` variant
#[doc(hidden)]
#[macro_export]
macro_rules! einval {
    ($str:expr) => {
        $crate::error::Asn1DerError::new($crate::error::Asn1DerErrorVariant::InvalidData(e!($str)))
    };
}
/// Creates an `Unsupported` variant
#[doc(hidden)]
#[macro_export]
macro_rules! eunsupported {
    ($str:expr) => {
        $crate::error::Asn1DerError::new($crate::error::Asn1DerErrorVariant::Unsupported(e!($str)))
    };
}
/// Creates an `Other` variant
#[doc(hidden)]
#[macro_export]
macro_rules! eother {
    ($str:expr) => {
        $crate::error::Asn1DerError::new($crate::error::Asn1DerErrorVariant::Other(e!($str)))
    };
}

/// A trait to chain errors
pub trait ErrorChain {
    /// Chains another error to `self`
    ///
    /// _Info: does nothing if not build with `std`_
    fn propagate(self, desc: &'static str) -> Self;
}
impl<T> ErrorChain for Result<T, Asn1DerError> {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn propagate(self, _desc: &'static str) -> Self {
        #[cfg(any(not(feature = "std"), feature = "no_panic"))]
        return self;
        #[cfg(all(feature = "std", not(feature = "no_panic")))]
        {
            self.map_err(|e| {
                let new_error = match e.error {
                    Asn1DerErrorVariant::InOutError(_) => Asn1DerErrorVariant::InOutError(_desc),
                    Asn1DerErrorVariant::InvalidData(_) => Asn1DerErrorVariant::InvalidData(_desc),
                    Asn1DerErrorVariant::Unsupported(_) => Asn1DerErrorVariant::Unsupported(_desc),
                    Asn1DerErrorVariant::Other(_) => Asn1DerErrorVariant::Other(_desc),
                };
                Asn1DerError { error: new_error, source: Some(ErrorSource::new(e)) }
            })
        }
    }
}

/// An `Asn1DerError` variant
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Asn1DerErrorVariant {
    /// An in-out error occurred (e.g. failed to read/write some bytes)
    InOutError(&'static str),
    /// The data has an invalid encoding
    InvalidData(&'static str),
    /// The object type or length is not supported by this implementation
    Unsupported(&'static str),
    /// An unspecified error
    Other(&'static str),
}
impl Display for Asn1DerErrorVariant {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Asn1DerErrorVariant::InOutError(desc) => write!(f, "I/O error {}", desc),
            Asn1DerErrorVariant::InvalidData(desc) => write!(f, "Invalid encoding {}", desc),
            Asn1DerErrorVariant::Unsupported(desc) => write!(f, "Unsupported {}", desc),
            Asn1DerErrorVariant::Other(desc) => write!(f, "Other {}", desc),
        }
    }
}

/// An error source
#[doc(hidden)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ErrorSource {
    #[cfg(any(not(feature = "std"), feature = "no_panic"))]
    inner: &'static str,
    #[cfg(all(feature = "std", not(feature = "no_panic")))]
    inner: Box<Asn1DerError>,
}
impl ErrorSource {
    /// Creates a new error source
    #[cfg(all(feature = "std", not(feature = "no_panic")))]
    pub fn new(e: Asn1DerError) -> Self {
        Self { inner: Box::new(e) }
    }
}

/// An `asn1_der` error
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Asn1DerError {
    #[doc(hidden)]
    pub error: Asn1DerErrorVariant,
    #[doc(hidden)]
    pub source: Option<ErrorSource>,
}
impl Asn1DerError {
    /// Creates a new error with `variant`
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn new(variant: Asn1DerErrorVariant) -> Self {
        Self { error: variant, source: None }
    }
}
impl Display for Asn1DerError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.source.as_ref() {
            Some(source) => write!(f, "{}\n    caused by: {}", self.error, source.inner),
            None => write!(f, "{}", self.error),
        }
    }
}
#[cfg(feature = "std")]
impl Error for Asn1DerError {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        #[cfg(any(not(feature = "std"), feature = "no_panic"))]
        return None;
        #[cfg(all(feature = "std", not(feature = "no_panic")))]
        return self.source.as_ref().map(|s| s.inner.as_ref() as _);
    }
}
