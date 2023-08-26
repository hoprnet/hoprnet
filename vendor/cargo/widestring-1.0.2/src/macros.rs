macro_rules! implement_utf16_macro {
    ($(#[$m:meta])* $name:ident $extra_len:literal $str:ident $fn:ident) => {
        $(#[$m])*
        #[macro_export]
        macro_rules! $name {
            ($text:expr) => {{
                const _WIDESTRING_U16_MACRO_UTF8: &$crate::internals::core::primitive::str = $text;
                const _WIDESTRING_U16_MACRO_LEN: $crate::internals::core::primitive::usize =
                    $crate::internals::length_as_utf16(_WIDESTRING_U16_MACRO_UTF8) + $extra_len;
                const _WIDESTRING_U16_MACRO_UTF16: [$crate::internals::core::primitive::u16;
                        _WIDESTRING_U16_MACRO_LEN] = {
                    let mut _widestring_buffer: [$crate::internals::core::primitive::u16; _WIDESTRING_U16_MACRO_LEN] = [0; _WIDESTRING_U16_MACRO_LEN];
                    let mut _widestring_bytes = _WIDESTRING_U16_MACRO_UTF8.as_bytes();
                    let mut _widestring_i = 0;
                    while let $crate::internals::core::option::Option::Some((_widestring_ch, _widestring_rest)) = $crate::internals::next_code_point(_widestring_bytes) {
                        _widestring_bytes = _widestring_rest;
                        if $extra_len > 0 && _widestring_ch == 0 {
                            panic!("invalid NUL value found in string literal");
                        }
                        // https://doc.rust-lang.org/std/primitive.char.html#method.encode_utf16
                        if _widestring_ch & 0xFFFF == _widestring_ch {
                            _widestring_buffer[_widestring_i] = _widestring_ch as $crate::internals::core::primitive::u16;
                            _widestring_i += 1;
                        } else {
                            let _widestring_code = _widestring_ch - 0x1_0000;
                            _widestring_buffer[_widestring_i] = 0xD800 | ((_widestring_code >> 10) as $crate::internals::core::primitive::u16);
                            _widestring_buffer[_widestring_i + 1] = 0xDC00 | ((_widestring_code as $crate::internals::core::primitive::u16) & 0x3FF);
                            _widestring_i += 2;
                        }
                    }
                    _widestring_buffer
                };
                #[allow(unused_unsafe)]
                unsafe { $crate::$str::$fn(&_WIDESTRING_U16_MACRO_UTF16) }
            }};
        }
    }
}

implement_utf16_macro! {
    /// Converts a string literal into a `const` UTF-16 string slice of type
    /// [`Utf16Str`][crate::Utf16Str].
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "alloc")] {
    /// use widestring::{utf16str, Utf16Str, Utf16String};
    ///
    /// const STRING: &Utf16Str = utf16str!("My string");
    /// assert_eq!(Utf16String::from_str("My string"), STRING);
    /// # }
    /// ```
    utf16str 0 Utf16Str from_slice_unchecked
}

implement_utf16_macro! {
    /// Converts a string literal into a `const` UTF-16 string slice of type
    /// [`U16Str`][crate::U16Str].
    ///
    /// The resulting `const` string slice will always be valid UTF-16.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "alloc")] {
    /// use widestring::{u16str, U16Str, U16String};
    ///
    /// const STRING: &U16Str = u16str!("My string");
    /// assert_eq!(U16String::from_str("My string"), STRING);
    /// # }
    /// ```
    u16str 0 U16Str from_slice
}

implement_utf16_macro! {
    /// Converts a string literal into a `const` UTF-16 string slice of type
    /// [`U16CStr`][crate::U16CStr].
    ///
    /// The resulting `const` string slice will always be valid UTF-16 and include a nul terminator.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "alloc")] {
    /// use widestring::{u16cstr, U16CStr, U16CString};
    ///
    /// const STRING: &U16CStr = u16cstr!("My string");
    /// assert_eq!(U16CString::from_str("My string").unwrap(), STRING);
    /// # }
    /// ```
    u16cstr 1 U16CStr from_slice_unchecked
}

macro_rules! implement_utf32_macro {
    ($(#[$m:meta])* $name:ident $extra_len:literal $str:ident $fn:ident) => {
        $(#[$m])*
        #[macro_export]
        macro_rules! $name {
            ($text:expr) => {{
                const _WIDESTRING_U32_MACRO_UTF8: &$crate::internals::core::primitive::str = $text;
                const _WIDESTRING_U32_MACRO_LEN: $crate::internals::core::primitive::usize =
                    $crate::internals::length_as_utf32(_WIDESTRING_U32_MACRO_UTF8) + $extra_len;
                const _WIDESTRING_U32_MACRO_UTF32: [$crate::internals::core::primitive::u32;
                        _WIDESTRING_U32_MACRO_LEN] = {
                    let mut _widestring_buffer: [$crate::internals::core::primitive::u32; _WIDESTRING_U32_MACRO_LEN] = [0; _WIDESTRING_U32_MACRO_LEN];
                    let mut _widestring_bytes = _WIDESTRING_U32_MACRO_UTF8.as_bytes();
                    let mut _widestring_i = 0;
                    while let $crate::internals::core::option::Option::Some((_widestring_ch, _widestring_rest)) = $crate::internals::next_code_point(_widestring_bytes) {
                        if $extra_len > 0 && _widestring_ch == 0 {
                            panic!("invalid NUL value found in string literal");
                        }
                        _widestring_bytes = _widestring_rest;
                        _widestring_buffer[_widestring_i] = _widestring_ch;
                        _widestring_i += 1;
                    }
                    _widestring_buffer
                };
                #[allow(unused_unsafe)]
                unsafe { $crate::$str::$fn(&_WIDESTRING_U32_MACRO_UTF32) }
            }};
        }
    }
}

implement_utf32_macro! {
    /// Converts a string literal into a `const` UTF-32 string slice of type
    /// [`Utf32Str`][crate::Utf32Str].
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "alloc")] {
    /// use widestring::{utf32str, Utf32Str, Utf32String};
    ///
    /// const STRING: &Utf32Str = utf32str!("My string");
    /// assert_eq!(Utf32String::from_str("My string"), STRING);
    /// # }
    /// ```
    utf32str 0 Utf32Str from_slice_unchecked
}

implement_utf32_macro! {
    /// Converts a string literal into a `const` UTF-32 string slice of type
    /// [`U32Str`][crate::U32Str].
    ///
    /// The resulting `const` string slice will always be valid UTF-32.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "alloc")] {
    /// use widestring::{u32str, U32Str, U32String};
    ///
    /// const STRING: &U32Str = u32str!("My string");
    /// assert_eq!(U32String::from_str("My string"), STRING);
    /// # }
    /// ```
    u32str 0 U32Str from_slice
}

implement_utf32_macro! {
    /// Converts a string literal into a `const` UTF-32 string slice of type
    /// [`U32CStr`][crate::U32CStr].
    ///
    /// The resulting `const` string slice will always be valid UTF-32 and include a nul terminator.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "alloc")] {
    /// use widestring::{u32cstr, U32CStr, U32CString};
    ///
    /// const STRING: &U32CStr = u32cstr!("My string");
    /// assert_eq!(U32CString::from_str("My string").unwrap(), STRING);
    /// # }
    /// ```
    u32cstr 1 U32CStr from_slice_unchecked
}

/// Alias for [`u16str`] or [`u32str`] macros depending on platform. Intended to be used when using
/// [`WideStr`][crate::WideStr] type alias.
#[cfg(not(windows))]
#[macro_export]
macro_rules! widestr {
    ($text:expr) => {{
        use $crate::*;
        u32str!($text)
    }};
}

/// Alias for [`utf16str`] or [`utf32str`] macros depending on platform. Intended to be used when
/// using [`WideUtfStr`][crate::WideUtfStr] type alias.
#[cfg(not(windows))]
#[macro_export]
macro_rules! wideutfstr {
    ($text:expr) => {{
        use $crate::*;
        utf32str!($text)
    }};
}

/// Alias for [`u16cstr`] or [`u32cstr`] macros depending on platform. Intended to be used when
/// using [`WideCStr`][crate::WideCStr] type alias.
#[cfg(not(windows))]
#[macro_export]
macro_rules! widecstr {
    ($text:expr) => {{
        use $crate::*;
        u32cstr!($text)
    }};
}

/// Alias for [`u16str`] or [`u32str`] macros depending on platform. Intended to be used when using
/// [`WideStr`][crate::WideStr] type alias.
#[cfg(windows)]
#[macro_export]
macro_rules! widestr {
    ($text:expr) => {{
        use $crate::*;
        u16str!($text)
    }};
}

/// Alias for [`utf16str`] or [`utf32str`] macros depending on platform. Intended to be used when
/// using [`WideUtfStr`][crate::WideUtfStr] type alias.
#[cfg(windows)]
#[macro_export]
macro_rules! wideutfstr {
    ($text:expr) => {{
        use $crate::*;
        utf16str!($text)
    }};
}

/// Alias for [`u16cstr`] or [`u32cstr`] macros depending on platform. Intended to be used when
/// using [`WideCStr`][crate::WideCStr] type alias.
#[cfg(windows)]
#[macro_export]
macro_rules! widecstr {
    ($text:expr) => {{
        use $crate::*;
        u16cstr!($text)
    }};
}

#[doc(hidden)]
pub mod internals {
    pub use core;

    // A const implementation of https://github.com/rust-lang/rust/blob/d902752866cbbdb331e3cf28ff6bba86ab0f6c62/library/core/src/str/mod.rs#L509-L537
    // Assumes `utf8` is a valid &str
    pub const fn next_code_point(utf8: &[u8]) -> Option<(u32, &[u8])> {
        const CONT_MASK: u8 = 0b0011_1111;
        match utf8 {
            [one @ 0..=0b0111_1111, rest @ ..] => Some((*one as u32, rest)),
            [one @ 0b1100_0000..=0b1101_1111, two, rest @ ..] => Some((
                (((*one & 0b0001_1111) as u32) << 6) | ((*two & CONT_MASK) as u32),
                rest,
            )),
            [one @ 0b1110_0000..=0b1110_1111, two, three, rest @ ..] => Some((
                (((*one & 0b0000_1111) as u32) << 12)
                    | (((*two & CONT_MASK) as u32) << 6)
                    | ((*three & CONT_MASK) as u32),
                rest,
            )),
            [one, two, three, four, rest @ ..] => Some((
                (((*one & 0b0000_0111) as u32) << 18)
                    | (((*two & CONT_MASK) as u32) << 12)
                    | (((*three & CONT_MASK) as u32) << 6)
                    | ((*four & CONT_MASK) as u32),
                rest,
            )),
            [..] => None,
        }
    }

    // A const implementation of `s.chars().map(|ch| ch.len_utf16()).sum()`
    pub const fn length_as_utf16(s: &str) -> usize {
        let mut bytes = s.as_bytes();
        let mut len = 0;
        while let Some((ch, rest)) = next_code_point(bytes) {
            bytes = rest;
            len += if (ch & 0xFFFF) == ch { 1 } else { 2 };
        }
        len
    }

    // A const implementation of `s.chars().len()`
    pub const fn length_as_utf32(s: &str) -> usize {
        let mut bytes = s.as_bytes();
        let mut len = 0;
        while let Some((_, rest)) = next_code_point(bytes) {
            bytes = rest;
            len += 1;
        }
        len
    }
}

#[cfg(all(test, feature = "alloc"))]
mod test {
    use crate::{
        U16CStr, U16Str, U32CStr, U32Str, Utf16Str, Utf16String, Utf32Str, Utf32String, WideCStr,
        WideStr, WideString,
    };

    const UTF16STR_TEST: &Utf16Str = utf16str!("⚧️🏳️‍⚧️➡️s");
    const U16STR_TEST: &U16Str = u16str!("⚧️🏳️‍⚧️➡️s");
    const U16CSTR_TEST: &U16CStr = u16cstr!("⚧️🏳️‍⚧️➡️s");
    const UTF32STR_TEST: &Utf32Str = utf32str!("⚧️🏳️‍⚧️➡️s");
    const U32STR_TEST: &U32Str = u32str!("⚧️🏳️‍⚧️➡️s");
    const U32CSTR_TEST: &U32CStr = u32cstr!("⚧️🏳️‍⚧️➡️s");
    const WIDESTR_TEST: &WideStr = widestr!("⚧️🏳️‍⚧️➡️s");
    const WIDECSTR_TEST: &WideCStr = widecstr!("⚧️🏳️‍⚧️➡️s");

    #[test]
    fn str_macros() {
        let str = Utf16String::from_str("⚧️🏳️‍⚧️➡️s");
        assert_eq!(&str, UTF16STR_TEST);
        assert_eq!(&str, U16STR_TEST);
        assert_eq!(&str, U16CSTR_TEST);
        assert!(matches!(U16CSTR_TEST.as_slice_with_nul().last(), Some(&0)));

        let str = Utf32String::from_str("⚧️🏳️‍⚧️➡️s");
        assert_eq!(&str, UTF32STR_TEST);
        assert_eq!(&str, U32STR_TEST);
        assert_eq!(&str, U32CSTR_TEST);
        assert!(matches!(U32CSTR_TEST.as_slice_with_nul().last(), Some(&0)));

        let str = WideString::from_str("⚧️🏳️‍⚧️➡️s");
        assert_eq!(&str, WIDESTR_TEST);
        assert_eq!(&str, WIDECSTR_TEST);
        assert!(matches!(WIDECSTR_TEST.as_slice_with_nul().last(), Some(&0)));
    }
}
