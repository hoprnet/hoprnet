// Copyright 2015 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
// ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
// OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

use crate::{calendar, time, Error};
pub(crate) use ring::io::der::{CONSTRUCTED, CONTEXT_SPECIFIC};

// Copied (and extended) from ring's src/der.rs
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub(crate) enum Tag {
    Boolean = 0x01,
    Integer = 0x02,
    BitString = 0x03,
    OctetString = 0x04,
    OID = 0x06,
    UTF8String = 0x0C,
    Sequence = CONSTRUCTED | 0x10, // 0x30
    Set = CONSTRUCTED | 0x11,      // 0x31
    UTCTime = 0x17,
    GeneralizedTime = 0x18,

    #[allow(clippy::identity_op)]
    ContextSpecificConstructed0 = CONTEXT_SPECIFIC | CONSTRUCTED | 0,
    ContextSpecificConstructed1 = CONTEXT_SPECIFIC | CONSTRUCTED | 1,
    ContextSpecificConstructed3 = CONTEXT_SPECIFIC | CONSTRUCTED | 3,
}

impl From<Tag> for usize {
    #[allow(clippy::as_conversions)]
    fn from(tag: Tag) -> Self {
        tag as Self
    }
}

impl From<Tag> for u8 {
    #[allow(clippy::as_conversions)]
    fn from(tag: Tag) -> Self {
        tag as Self
    } // XXX: narrowing conversion.
}

#[inline(always)]
pub(crate) fn expect_tag_and_get_value<'a>(
    input: &mut untrusted::Reader<'a>,
    tag: Tag,
) -> Result<untrusted::Input<'a>, Error> {
    let (actual_tag, inner) = read_tag_and_get_value(input)?;
    if usize::from(tag) != usize::from(actual_tag) {
        return Err(Error::BadDer);
    }
    Ok(inner)
}

// TODO: investigate taking decoder as a reference to reduce generated code
// size.
pub(crate) fn nested<'a, F, R, E: Copy>(
    input: &mut untrusted::Reader<'a>,
    tag: Tag,
    error: E,
    decoder: F,
) -> Result<R, E>
where
    F: FnOnce(&mut untrusted::Reader<'a>) -> Result<R, E>,
{
    let inner = expect_tag_and_get_value(input, tag).map_err(|_| error)?;
    inner.read_all(error, decoder)
}

pub struct Value<'a> {
    value: untrusted::Input<'a>,
}

impl<'a> Value<'a> {
    pub fn value(&self) -> untrusted::Input<'a> {
        self.value
    }
}

pub(crate) fn expect_tag<'a>(
    input: &mut untrusted::Reader<'a>,
    tag: Tag,
) -> Result<Value<'a>, Error> {
    let (actual_tag, value) = read_tag_and_get_value(input)?;
    if usize::from(tag) != usize::from(actual_tag) {
        return Err(Error::BadDer);
    }

    Ok(Value { value })
}

#[inline(always)]
pub(crate) fn read_tag_and_get_value<'a>(
    input: &mut untrusted::Reader<'a>,
) -> Result<(u8, untrusted::Input<'a>), Error> {
    ring::io::der::read_tag_and_get_value(input).map_err(|_| Error::BadDer)
}

// TODO: investigate taking decoder as a reference to reduce generated code
// size.
pub(crate) fn nested_of_mut<'a, E>(
    input: &mut untrusted::Reader<'a>,
    outer_tag: Tag,
    inner_tag: Tag,
    error: E,
    mut decoder: impl FnMut(&mut untrusted::Reader<'a>) -> Result<(), E>,
) -> Result<(), E>
where
    E: Copy,
{
    nested(input, outer_tag, error, |outer| {
        loop {
            nested(outer, inner_tag, error, |inner| decoder(inner))?;
            if outer.at_end() {
                break;
            }
        }
        Ok(())
    })
}

pub(crate) fn bit_string_with_no_unused_bits<'a>(
    input: &mut untrusted::Reader<'a>,
) -> Result<untrusted::Input<'a>, Error> {
    nested(input, Tag::BitString, Error::BadDer, |value| {
        let unused_bits_at_end = value.read_byte().map_err(|_| Error::BadDer)?;
        if unused_bits_at_end != 0 {
            return Err(Error::BadDer);
        }
        Ok(value.read_bytes_to_end())
    })
}

// Like mozilla::pkix, we accept the nonconformant explicit encoding of
// the default value (false) for compatibility with real-world certificates.
pub(crate) fn optional_boolean(input: &mut untrusted::Reader) -> Result<bool, Error> {
    if !input.peek(Tag::Boolean.into()) {
        return Ok(false);
    }
    nested(input, Tag::Boolean, Error::BadDer, |input| {
        match input.read_byte() {
            Ok(0xff) => Ok(true),
            Ok(0x00) => Ok(false),
            _ => Err(Error::BadDer),
        }
    })
}

pub(crate) fn small_nonnegative_integer(input: &mut untrusted::Reader) -> Result<u8, Error> {
    ring::io::der::small_nonnegative_integer(input).map_err(|_| Error::BadDer)
}

pub(crate) fn time_choice(input: &mut untrusted::Reader) -> Result<time::Time, Error> {
    let is_utc_time = input.peek(Tag::UTCTime.into());
    let expected_tag = if is_utc_time {
        Tag::UTCTime
    } else {
        Tag::GeneralizedTime
    };

    fn read_digit(inner: &mut untrusted::Reader) -> Result<u64, Error> {
        const DIGIT: core::ops::RangeInclusive<u8> = b'0'..=b'9';
        let b = inner.read_byte().map_err(|_| Error::BadDerTime)?;
        if DIGIT.contains(&b) {
            return Ok(u64::from(b - DIGIT.start()));
        }
        Err(Error::BadDerTime)
    }

    fn read_two_digits(inner: &mut untrusted::Reader, min: u64, max: u64) -> Result<u64, Error> {
        let hi = read_digit(inner)?;
        let lo = read_digit(inner)?;
        let value = (hi * 10) + lo;
        if value < min || value > max {
            return Err(Error::BadDerTime);
        }
        Ok(value)
    }

    nested(input, expected_tag, Error::BadDer, |value| {
        let (year_hi, year_lo) = if is_utc_time {
            let lo = read_two_digits(value, 0, 99)?;
            let hi = if lo >= 50 { 19 } else { 20 };
            (hi, lo)
        } else {
            let hi = read_two_digits(value, 0, 99)?;
            let lo = read_two_digits(value, 0, 99)?;
            (hi, lo)
        };

        let year = (year_hi * 100) + year_lo;
        let month = read_two_digits(value, 1, 12)?;
        let days_in_month = calendar::days_in_month(year, month);
        let day_of_month = read_two_digits(value, 1, days_in_month)?;
        let hours = read_two_digits(value, 0, 23)?;
        let minutes = read_two_digits(value, 0, 59)?;
        let seconds = read_two_digits(value, 0, 59)?;

        let time_zone = value.read_byte().map_err(|_| Error::BadDerTime)?;
        if time_zone != b'Z' {
            return Err(Error::BadDerTime);
        }

        calendar::time_from_ymdhms_utc(year, month, day_of_month, hours, minutes, seconds)
    })
}

macro_rules! oid {
    ( $first:expr, $second:expr, $( $tail:expr ),* ) =>
    (
        [(40 * $first) + $second, $( $tail ),*]
    )
}
