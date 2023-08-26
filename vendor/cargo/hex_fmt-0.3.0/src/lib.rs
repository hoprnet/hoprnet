//! # Formatting and shortening byte slices as hexadecimal strings
//!
//! This crate provides wrappers for byte slices and lists of byte slices that implement the
//! standard formatting traits and print the bytes as a hexadecimal string. It respects the
//! alignment, width and precision parameters and applies padding and shortening.
//!
//! ```
//! # use hex_fmt::{HexFmt, HexList};
//! let bytes: &[u8] = &[0x0a, 0x1b, 0x2c, 0x3d, 0x4e, 0x5f];
//!
//! assert_eq!("0a1b2c3d4e5f", &format!("{}", HexFmt(bytes)));
//!
//! // By default the full slice is printed. Change the width to apply padding or shortening.
//! assert_eq!("0a..5f", &format!("{:6}", HexFmt(bytes)));
//! assert_eq!("0a1b2c3d4e5f", &format!("{:12}", HexFmt(bytes)));
//! assert_eq!("  0a1b2c3d4e5f  ", &format!("{:16}", HexFmt(bytes)));
//!
//! // The default alignment is centered. Use `<` or `>` to align left or right.
//! assert_eq!("0a1b..", &format!("{:<6}", HexFmt(bytes)));
//! assert_eq!("0a1b2c3d4e5f    ", &format!("{:<16}", HexFmt(bytes)));
//! assert_eq!("..4e5f", &format!("{:>6}", HexFmt(bytes)));
//! assert_eq!("    0a1b2c3d4e5f", &format!("{:>16}", HexFmt(bytes)));
//!
//! // Use e.g. `4.8` to set the minimum width to 4 and the maximum to 8.
//! assert_eq!(" 12 ", &format!("{:4.8}", HexFmt([0x12])));
//! assert_eq!("123456", &format!("{:4.8}", HexFmt([0x12, 0x34, 0x56])));
//! assert_eq!("123..89a", &format!("{:4.8}", HexFmt([0x12, 0x34, 0x56, 0x78, 0x9a])));
//!
//! // If you prefer uppercase, use `X`.
//! assert_eq!("0A1B2C3D4E5F", &format!("{:X}", HexFmt(bytes)));
//!
//! // All of the above can be combined.
//! assert_eq!("0A1B2C..", &format!("{:<4.8X}", HexFmt(bytes)));
//!
//! // With `HexList`, the parameters are applied to each entry.
//! let list = &[[0x0a; 3], [0x1b; 3], [0x2c; 3]];
//! assert_eq!("[0A.., 1B.., 2C..]", &format!("{:<4X}", HexList(list)));
//! ```

#![cfg_attr(not(test), no_std)]

use core::fmt::{Alignment, Debug, Display, Formatter, LowerHex, Result, UpperHex, Write};

const ELLIPSIS: &str = "..";

/// Wrapper for a byte array, whose `Debug`, `Display` and `LowerHex` implementations output
/// shortened hexadecimal strings.
pub struct HexFmt<T>(pub T);

impl<T: AsRef<[u8]>> Debug for HexFmt<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result {
        LowerHex::fmt(self, f)
    }
}

impl<T: AsRef<[u8]>> Display for HexFmt<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result {
        LowerHex::fmt(self, f)
    }
}

impl<T: AsRef<[u8]>> LowerHex for HexFmt<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result {
        Lowercase::fmt(self.0.as_ref(), f)
    }
}

impl<T: AsRef<[u8]>> UpperHex for HexFmt<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result {
        Uppercase::fmt(self.0.as_ref(), f)
    }
}

/// Wrapper for a list of byte arrays, whose `Debug`, `Display` and `LowerHex` implementations
/// output shortened hexadecimal strings.
pub struct HexList<T>(pub T);

impl<T> Debug for HexList<T>
where
    T: Clone + IntoIterator,
    T::Item: AsRef<[u8]>,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result {
        LowerHex::fmt(self, f)
    }
}

impl<T> Display for HexList<T>
where
    T: Clone + IntoIterator,
    T::Item: AsRef<[u8]>,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result {
        LowerHex::fmt(self, f)
    }
}

impl<T> LowerHex for HexList<T>
where
    T: Clone + IntoIterator,
    T::Item: AsRef<[u8]>,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result {
        let entries = self.0.clone().into_iter().map(HexFmt);
        f.debug_list().entries(entries).finish()
    }
}

impl<T> UpperHex for HexList<T>
where
    T: Clone + IntoIterator,
    T::Item: AsRef<[u8]>,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut iter = self.0.clone().into_iter();
        write!(f, "[")?;
        if let Some(item) = iter.next() {
            UpperHex::fmt(&HexFmt(item), f)?;
        }
        for item in iter {
            write!(f, ", ")?;
            UpperHex::fmt(&HexFmt(item), f)?;
        }
        write!(f, "]")
    }
}

trait Case {
    fn fmt_byte(f: &mut Formatter, byte: u8) -> Result;
    fn fmt_digit(f: &mut Formatter, digit: u8) -> Result;

    #[inline]
    fn fmt(bytes: &[u8], f: &mut Formatter) -> Result {
        let min_width = f.width().unwrap_or(0);
        let max_width = f
            .precision()
            .or_else(|| f.width())
            .unwrap_or_else(usize::max_value);
        let align = f.align().unwrap_or(Alignment::Center);

        // If the array is short enough, don't shorten it.
        if 2 * bytes.len() <= max_width {
            let fill = f.fill();
            let missing = min_width.saturating_sub(2 * bytes.len());
            let (left, right) = match align {
                Alignment::Left => (0, missing),
                Alignment::Right => (missing, 0),
                Alignment::Center => (missing / 2, missing - missing / 2),
            };
            for _ in 0..left {
                f.write_char(fill)?;
            }
            for byte in bytes {
                Self::fmt_byte(f, *byte)?;
            }
            for _ in 0..right {
                f.write_char(fill)?;
            }
            return Ok(());
        }

        // If the bytes don't fit and the ellipsis fills the maximum width, print only that.
        if max_width <= ELLIPSIS.len() {
            return write!(f, "{:.*}", max_width, ELLIPSIS);
        }

        // Compute the number of hex digits to display left and right of the ellipsis.
        let digits = max_width.saturating_sub(ELLIPSIS.len());
        let (left, right) = match align {
            Alignment::Left => (digits, 0),
            Alignment::Right => (0, digits),
            Alignment::Center => (digits - digits / 2, digits / 2),
        };

        // Print the bytes on the left.
        for byte in &bytes[..(left / 2)] {
            Self::fmt_byte(f, *byte)?;
        }
        // If odd, print only the first hex digit of the next byte.
        if left & 1 == 1 {
            Self::fmt_digit(f, bytes[left / 2] >> 4)?;
        }

        // Print the ellipsis.
        f.write_str(ELLIPSIS)?;

        // If `right` is odd, print the second hex digit of a byte.
        if right & 1 == 1 {
            Self::fmt_digit(f, bytes[(bytes.len() - right / 2 - 1)] & 0x0f)?;
        }
        // Print the remaining bytes on the right.
        for byte in &bytes[(bytes.len() - right / 2)..] {
            Self::fmt_byte(f, *byte)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct Uppercase;

impl Case for Uppercase {
    #[inline]
    fn fmt_byte(f: &mut Formatter, byte: u8) -> Result {
        write!(f, "{:02X}", byte)
    }

    #[inline]
    fn fmt_digit(f: &mut Formatter, digit: u8) -> Result {
        write!(f, "{:1X}", digit)
    }
}

#[derive(Clone, Copy)]
struct Lowercase;

impl Case for Lowercase {
    #[inline]
    fn fmt_byte(f: &mut Formatter, byte: u8) -> Result {
        write!(f, "{:02x}", byte)
    }

    #[inline]
    fn fmt_digit(f: &mut Formatter, digit: u8) -> Result {
        write!(f, "{:1x}", digit)
    }
}

#[cfg(test)]
mod tests {
    use super::HexFmt;

    #[test]
    fn test_fmt() {
        assert_eq!("", &format!("{:.0}", HexFmt(&[0x01])));
        assert_eq!(".", &format!("{:.1}", HexFmt(&[0x01])));
        assert_eq!("01", &format!("{:.2}", HexFmt(&[0x01])));
        assert_eq!("..", &format!("{:.2}", HexFmt(&[0x01, 0x23])));
    }
}
