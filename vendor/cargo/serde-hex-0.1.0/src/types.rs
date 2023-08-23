//! Miscellaneous type used by this crate.
use std::{error, fmt, io, result};

/// An alias of `std::result::Result` with this crate's
/// `Error` type inserted by default.
pub type Result<T> = result::Result<T, Error>;

/// error raised during hexadecimal parsing operations
#[derive(Debug)]
pub enum ParseHexError {
    /// hexadecimal buffer was outside allowed range
    Range {
        /// minimum allowed size
        min: usize,
        /// maximum allowed size
        max: usize,
        /// size the was found
        got: usize,
    },
    /// hexadecimal buffer was of expected size
    Size {
        /// expected size
        expect: usize,
        /// size that was found
        actual: usize,
    },
    /// non-hexadecimal character encountered
    Char {
        /// value encountered
        val: char,
    },
}

impl fmt::Display for ParseHexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseHexError::Range {
                ref min,
                ref max,
                ref got,
            } => write!(
                f,
                "expected buff size in `{}...{}`, got `{}`",
                min, max, got
            ),
            ParseHexError::Size {
                ref expect,
                ref actual,
            } => write!(f, "expected buff size `{}` got `{}`", expect, actual),
            ParseHexError::Char { ref val } => write!(f, "non-hex character `{}`", val),
        }
    }
}

impl error::Error for ParseHexError {
    fn description(&self) -> &str {
        match *self {
            ParseHexError::Range { .. } => "hexadecimal outside valid range",
            ParseHexError::Size { .. } => "invalid hexadecimal size",
            ParseHexError::Char { .. } => "non-hex character",
        }
    }
}

/// top-level error kind of this crate
#[derive(Debug)]
pub enum Error {
    /// error raised during i/o operations
    IoError(io::Error),
    /// error raised during parsing operations
    Parsing(ParseHexError),
}

// implement `Display` to allow user-facing errors.  Required
// by the `std::error::Error` trait.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::IoError(ref err) => err.fmt(f),
            Error::Parsing(ref err) => err.fmt(f),
        }
    }
}

// implement the standard error trait for hexadecimal errors.
impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IoError(ref err) => err.description(),
            Error::Parsing(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::IoError(ref err) => Some(err),
            Error::Parsing(ref err) => Some(err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<ParseHexError> for Error {
    fn from(err: ParseHexError) -> Self {
        Error::Parsing(err)
    }
}
