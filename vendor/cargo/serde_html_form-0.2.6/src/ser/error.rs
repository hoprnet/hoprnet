use std::{
    borrow::Cow,
    error, fmt,
    str::{self, Utf8Error},
};

use serde::ser;

/// Errors returned during serializing to `application/x-www-form-urlencoded`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error(ErrorKind);

#[derive(Clone, Debug, PartialEq, Eq)]
enum ErrorKind {
    Custom(Cow<'static, str>),
    Utf8(Utf8Error),
}

impl Error {
    pub(super) fn done() -> Self {
        Error(ErrorKind::Custom("this pair has already been serialized".into()))
    }

    pub(super) fn not_done() -> Self {
        Error(ErrorKind::Custom("this pair has not yet been serialized".into()))
    }

    pub(super) fn unsupported_key() -> Self {
        Error(ErrorKind::Custom("unsupported key".into()))
    }

    pub(super) fn unsupported_value() -> Self {
        Error(ErrorKind::Custom("unsupported value".into()))
    }

    pub(super) fn unsupported_pair() -> Self {
        Error(ErrorKind::Custom("unsupported pair".into()))
    }

    pub(super) fn top_level() -> Self {
        Error(ErrorKind::Custom("top-level serializer supports only maps and structs".into()))
    }

    pub(super) fn no_key() -> Self {
        Error(ErrorKind::Custom("tried to serialize a value before serializing key".into()))
    }

    pub(super) fn utf8(error: Utf8Error) -> Self {
        Error(ErrorKind::Utf8(error))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            ErrorKind::Custom(msg) => msg.fmt(f),
            ErrorKind::Utf8(err) => write!(f, "invalid UTF-8: {}", err),
        }
    }
}

impl error::Error for Error {
    /// The lower-level cause of this error, in the case of a `Utf8` error.
    fn cause(&self) -> Option<&dyn error::Error> {
        match &self.0 {
            ErrorKind::Custom(_) => None,
            ErrorKind::Utf8(err) => Some(err),
        }
    }

    /// The lower-level source of this error, in the case of a `Utf8` error.
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.0 {
            ErrorKind::Custom(_) => None,
            ErrorKind::Utf8(err) => Some(err),
        }
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self(ErrorKind::Custom(format!("{}", msg).into()))
    }
}
