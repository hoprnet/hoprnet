use core::fmt;
use crate::core::{ enc, dec };


#[derive(Debug)]
pub enum DecodeError<E> {
    Core(dec::Error<E>),
    Custom(crate::alloc::boxed::Box<str>)
}

impl<E> From<dec::Error<E>> for DecodeError<E> {
    #[inline]
    #[cold]
    fn from(err: dec::Error<E>) -> DecodeError<E> {
        DecodeError::Core(err)
    }
}

impl<E> From<E> for DecodeError<E> {
    #[inline]
    #[cold]
    fn from(err: E) -> DecodeError<E> {
        DecodeError::Core(dec::Error::Read(err))
    }
}

#[cfg(feature = "serde1")]
#[cfg(feature = "use_std")]
impl<E: std::error::Error + 'static> serde::de::Error for DecodeError<E> {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        DecodeError::Custom(msg.to_string().into_boxed_str())
    }
}

#[cfg(feature = "serde1")]
#[cfg(not(feature = "use_std"))]
impl<E: fmt::Debug> serde::de::Error for DecodeError<E> {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        #[cfg(not(feature = "use_std"))]
        use crate::alloc::string::ToString;

        DecodeError::Custom(msg.to_string().into_boxed_str())
    }
}

impl<E: fmt::Debug> fmt::Display for DecodeError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(feature = "serde1")]
#[cfg(not(feature = "use_std"))]
impl<E: fmt::Debug> serde::ser::StdError for DecodeError<E> {}

#[cfg(feature = "use_std")]
impl<E: std::error::Error + 'static> std::error::Error for DecodeError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DecodeError::Core(err) => Some(err),
            _ => None
        }
    }
}

#[derive(Debug)]
pub enum EncodeError<E> {
    Core(enc::Error<E>),
    Custom(crate::alloc::boxed::Box<str>)
}

impl<E> From<enc::Error<E>> for EncodeError<E> {
    #[inline]
    #[cold]
    fn from(err: enc::Error<E>) -> EncodeError<E> {
        EncodeError::Core(err)
    }
}

impl<E> From<E> for EncodeError<E> {
    #[inline]
    #[cold]
    fn from(err: E) -> EncodeError<E> {
        EncodeError::Core(enc::Error::Write(err))
    }
}

#[cfg(feature = "serde1")]
#[cfg(feature = "use_std")]
impl<E: std::error::Error + 'static> serde::ser::Error for EncodeError<E> {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        EncodeError::Custom(msg.to_string().into_boxed_str())
    }
}

#[cfg(feature = "serde1")]
#[cfg(not(feature = "use_std"))]
impl<E: fmt::Debug> serde::ser::Error for EncodeError<E> {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        #[cfg(not(feature = "use_std"))]
        use crate::alloc::string::ToString;

        EncodeError::Custom(msg.to_string().into_boxed_str())
    }
}

impl<E: fmt::Debug> fmt::Display for EncodeError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(feature = "serde1")]
#[cfg(not(feature = "use_std"))]
impl<E: fmt::Debug> serde::ser::StdError for EncodeError<E> {}

#[cfg(feature = "use_std")]
impl<E: std::error::Error + 'static> std::error::Error for EncodeError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EncodeError::Core(err) => Some(err),
            _ => None
        }
    }
}
