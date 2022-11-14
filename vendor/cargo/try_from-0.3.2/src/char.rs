// Conversion between machine integers and `char`.

cfg_if! (
    if #[cfg(feature="no_std")] {
        use core::char;
        use core::fmt::{self, Display, Formatter};
    } else {
        use std::char;
        use std::error::Error;
        use std::fmt::{self, Display, Formatter};
    }
);

use {TryFrom, TryFromIntError, Void};

impl<T> TryFrom<char> for T
where
    T: TryFrom<u32, Err = TryFromIntError>,
{
    type Err = TryFromIntError;

    fn try_from(c: char) -> Result<T, TryFromIntError> {
        T::try_from(c as u32)
    }
}

#[test]
fn test_char_to_int() {
    assert_eq!(u8::try_from('~'), Ok(0x7e));
    assert_eq!(u8::try_from('\u{100}'), Err(TryFromIntError::Overflow));
}

macro_rules! impl_infallible {
    ($($ty:ty),*) => { $(
        impl TryFrom<$ty> for char {
            type Err = Void;
            fn try_from (n: $ty) -> Result<char, Void> { Ok(n as char) }
        }
    )* };
}

impl_infallible!(u8, char);

#[test]
fn test_to_char_infallible() {
    assert_eq!(char::try_from(0x7e), Ok('~'));
    assert_eq!(char::try_from('~'), Ok('~'));
}

macro_rules! impl_int_to_char {
    ($($ty:ty),*) => { $(
        impl TryFrom<$ty> for char {
            type Err = TryFromIntToCharError;

            fn try_from (n: $ty) -> Result<char, TryFromIntToCharError> {
                match u32::try_from(n)? {
                    n @ 0...0x10ffff => match char::from_u32(n) {
                        None => Err(TryFromIntToCharError::Reserved),
                        Some(c) => Ok(c),
                    },
                    _ => Err(TryFromIntToCharError::Overflow)
                }
            }
        }
    )* };
}

impl_int_to_char!(i8, i16, i32, i64, isize, u16, u32, u64, usize);

#[test]
fn test_int_to_char() {
    assert_eq!(char::try_from(-1), Err(TryFromIntToCharError::Underflow));
    assert_eq!(char::try_from(0x7eu32), Ok('~'));
    assert_eq!(char::try_from(0xd888), Err(TryFromIntToCharError::Reserved));
    assert_eq!(char::try_from(0x10ffff), Ok('\u{10ffff}'));
    assert_eq!(
        char::try_from(0x110000),
        Err(TryFromIntToCharError::Overflow)
    );
}

/// Error which occurs when conversion from an integer to a `char` fails.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TryFromIntToCharError {
    Overflow,
    Underflow,
    Reserved,
}

impl TryFromIntToCharError {
    fn as_str(self) -> &'static str {
        match self {
            TryFromIntToCharError::Overflow => "integer overflow",
            TryFromIntToCharError::Underflow => "integer underflow",
            TryFromIntToCharError::Reserved => "reserved code point",
        }
    }
}

impl Display for TryFromIntToCharError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}


cfg_if! (
    if #[cfg(not(feature="no_std"))] {
        impl Error for TryFromIntToCharError {
            fn description(&self) -> &str {
                self.as_str()
            }
        }
    }
);

impl From<TryFromIntError> for TryFromIntToCharError {
    fn from(other: TryFromIntError) -> TryFromIntToCharError {
        match other {
            TryFromIntError::Overflow => TryFromIntToCharError::Overflow,
            TryFromIntError::Underflow => TryFromIntToCharError::Underflow,
        }
    }
}

impl From<Void> for TryFromIntToCharError {
    fn from(_: Void) -> TryFromIntToCharError {
        unreachable!()
    }
}
