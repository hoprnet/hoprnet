use core::fmt;
use core::convert::TryFrom;


/// Static String
///
/// Use `&&str` instead of `&str` to reduce type size.
pub type StaticStr = &'static &'static str;

/// Never type
pub type Never = core::convert::Infallible;

/// Length type
#[derive(Debug)]
pub enum Len {
    /// Indefinite length
    Indefinite,
    /// Small length
    Small(u16),
    /// Big length
    Big
}

#[derive(Debug)]
pub enum ArithmeticOverflow {
    Overflow,
    Underflow
}

/// Decode Error
#[derive(Debug)]
#[non_exhaustive]
pub enum DecodeError<E> {
    Read(E),
    Mismatch {
        name: StaticStr,
        found: u8
    },
    Unsupported {
        name: StaticStr,
        found: u8
    },
    Eof {
        name: StaticStr,
        expect: Len,
    },
    RequireLength {
        name: StaticStr,
        found: Len,
    },
    RequireBorrowed {
        name: StaticStr,
    },
    RequireUtf8 {
        name: StaticStr,
    },
    LengthOverflow {
        name: StaticStr,
        found: Len
    },
    CastOverflow {
        name: StaticStr
    },
    ArithmeticOverflow {
        name: StaticStr,
        ty: ArithmeticOverflow
    },
    DepthOverflow {
        name: StaticStr
    },
}

impl Len {
    #[inline]
    pub fn new(len: usize) -> Len {
        match u16::try_from(len) {
            Ok(len) => Len::Small(len),
            Err(_) => Len::Big
        }
    }
}

impl<E> From<E> for DecodeError<E> {
    #[cold]
    fn from(err: E) -> DecodeError<E> {
        DecodeError::Read(err)
    }
}

impl<E> DecodeError<E> {
    #[cold]
    pub(crate) fn mismatch(name: StaticStr, found: u8) -> DecodeError<E> {
        DecodeError::Mismatch { name, found }
    }

    #[cold]
    pub(crate) fn unsupported(name: StaticStr, found: u8) -> DecodeError<E> {
        DecodeError::Unsupported { name, found }
    }

    #[cold]
    pub(crate) fn eof(name: StaticStr, expect: usize) -> DecodeError<E> {
        DecodeError::Eof { name, expect: Len::new(expect) }
    }

    #[cold]
    pub(crate) fn require_length(name: StaticStr, found: Option<usize>) -> DecodeError<E> {
        let found = match found {
            Some(len) => Len::new(len),
            None => Len::Indefinite
        };
        DecodeError::RequireLength { name, found }
    }

    #[cold]
    pub(crate) fn require_borrowed(name: StaticStr) -> DecodeError<E> {
        DecodeError::RequireBorrowed { name }
    }

    #[cold]
    pub(crate) fn require_utf8(name: StaticStr) -> DecodeError<E> {
        DecodeError::RequireUtf8 { name }
    }

    #[cold]
    pub(crate) fn length_overflow(name: StaticStr, found: usize) -> DecodeError<E> {
        DecodeError::LengthOverflow { name, found: Len::new(found) }
    }

    #[cold]
    pub(crate) fn cast_overflow(name: StaticStr) -> DecodeError<E> {
        DecodeError::CastOverflow { name }
    }

    #[cold]
    pub(crate) fn arithmetic_overflow(name: StaticStr, ty: ArithmeticOverflow) -> DecodeError<E> {
        DecodeError::ArithmeticOverflow { name, ty }
    }

    #[cold]
    pub(crate) fn depth_overflow(name: StaticStr) -> DecodeError<E> {
        DecodeError::DepthOverflow { name }
    }
}

impl<E: fmt::Debug> fmt::Display for DecodeError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(feature = "use_std")]
impl<E: std::error::Error + 'static> std::error::Error for DecodeError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DecodeError::Read(err) => Some(err),
            _ => None
        }
    }
}

/// Encode Error
#[derive(Debug)]
#[non_exhaustive]
pub enum EncodeError<E> {
    Write(E)
}

impl<E> From<E> for EncodeError<E> {
    #[cold]
    fn from(err: E) -> EncodeError<E> {
        EncodeError::Write(err)
    }
}

impl<E: fmt::Debug> fmt::Display for EncodeError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(feature = "use_std")]
impl<E: std::error::Error + 'static> std::error::Error for EncodeError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EncodeError::Write(err) => Some(err),
        }
    }
}

#[test]
fn test_error_type_size() {
    // bottom type
    assert_eq!(core::mem::size_of::<DecodeError<Never>>(), 16);

    // unit type
    assert_eq!(core::mem::size_of::<DecodeError<()>>(), 16);

    // a word type
    assert_eq!(core::mem::size_of::<DecodeError<&'static ()>>(), 16);
}
