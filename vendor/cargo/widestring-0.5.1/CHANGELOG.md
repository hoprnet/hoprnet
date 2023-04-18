# Changelog

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.1] - 2021-10-23 <a name="0.5.1"></a>
### Fixed
- Fixed a regression in 0.5.0 where zero-length vectors and strings were incorrectly causing panics
  in `UCString::from_vec` and `UCString::from_str`. Fixes [#22].
- Modified an implentation detail in `ustr::to_string` & `ustr::to_string_lossy` to remove possibly
  unsafe behaviour.

## [0.5.0] - 2021-10-12 <a name="0.5.0"></a>
### Changed
- **Breaking Change** Renamed a number of types and functions to increase consistency and clarity.
  This also meant renaming errors to more clearly convey error and trying to be more consistent with
  name conventions and functionality across types. Check renamed function docs for any changes in
  functionality, as there have been some minor tweaks (mostly relaxing/removing error conditions and
  reducing panics). Old names have been deprecated to ease transition and will be removed in a
  future release. Fixes [#18].
  - `MissingNulError` => `error::MissingNulTerminator`
  - `FromUtf32Error` => `error::FromUtf32Error`
  - `NulError` => `error::ContainsNul`
  - `UCStr::from_ptr_with_nul` => `from_ptr_unchecked`
  - `UCStr::from_slice_with_nul` => `from_slice_truncate`
  - `UCStr::from_slice_with_nul_unchecked` => `from_slice_unchecked`
  - `U32CStr::from_char_ptr_with_nul` => `from_char_ptr_unchecked`
  - `U32CStr::from_char_slice_with_nul` => `from_char_slice_truncate`
  - `U32CStr::from_char_slice_with_nul_unchecked` => `from_char_slice_unchecked`
  - `UCString::new` => `from_vec`
  - `UCString::from_vec_with_nul` => `from_vec_truncate`
  - `UCString::from_ustr_with_nul` => `from_ustr_truncate`
  - `UCString::from_ptr_with_nul` => `from_ptr_truncate`
  - `UCString::from_str_with_nul` => `from_str_truncate`
  - `UCString::from_os_str_with_nul` => `from_os_str_truncate`
  - `U32CString::from_chars_with_nul` => `from_chars_truncate`
  - `U32CString::from_char_ptr_with_nul` => `from_char_ptr_truncate`
- Deprecated error types in the crate root. Use the errors directly from `error` module instead.
- Improved implementations in some areas to reduce unncessary double allocations.
- Improved `Debug` implementations. No more debugging lists of raw integer values.
- Migrated crate to Rust 2018 edition.
- Minimum supported Rust version is now 1.48.
- Made crate package [REUSE compliant](https://reuse.software/).
- Improved documentation and used intra-doc links.

### Added
- Added crate-level functions `decode_utf16`, `decode_utf16_lossy`, `decode_utf32`, and
  `decode_utf32_lossy` and associated iterators. Note that `decode_utf16` is an alias of
  `core::char::decode_utf16`, but provided for consistency.
- Added `display` method to to both `UStr` and `UCStr` to display strings in formatting without heap
  allocations, similar to `Path::display`. Fixes [#20].
- Added more trait implementations, including more index operations and string formatting via
  `Write` trait. Fixes [#19].
- Added new functions:
    - `UStr::from_ptr_mut`
    - `UStr::from_slice_mut`
    - `UStr::as_mut_slice`
    - `UStr::as_mut_ptr`
    - `UStr::as_ptr_range`
    - `UStr::as_mut_ptr_range`
    - `UStr::get`
    - `UStr::get_mut`
    - `UStr::get_unchecked`
    - `UStr::get_unchecked_mut`
    - `UStr::split_at`
    - `UStr::split_at_mut`
    - `UStr::chars`
    - `UStr::chars_lossy`
    - `U16Str::char_indices`
    - `U16Str::char_indices_lossy`
    - `U32Str::from_char_ptr_mut`
    - `U32Str::from_char_slice_mut`
    - `UCStr::from_ptr`
    - `UCStr::from_ptr_truncate`
    - `UCStr::from_slice`
    - `UCStr::as_ustr`
    - `UCStr::from_ptr_str_mut`
    - `UCStr::from_ptr_mut`
    - `UCStr::from_ptr_truncate_mut`
    - `UCStr::from_ptr_unchecked_mut`
    - `UCStr::from_slice_mut`
    - `UCStr::from_slice_truncate_mut`
    - `UCStr::from_slice_unchecked_mut`
    - `UCStr::as_mut_slice`
    - `UCStr::as_mut_ptr`
    - `UCStr::as_ustr_with_nul`
    - `UCStr::as_mut_ustr`
    - `UCStr::as_ptr_range`
    - `UCStr::as_mut_ptr_range`
    - `UCStr::chars`
    - `UCStr::chars_lossy`
    - `U16CStr::char_indices`
    - `U16CStr::char_indices_lossy`
    - `U32CStr::from_char_ptr_str_mut`
    - `U32CStr::from_char_ptr_mut`
    - `U32CStr::from_char_ptr_truncate_mut`
    - `U32CStr::from_char_ptr_unchecked_mut`
    - `U32CStr::from_char_slice_mut`
    - `U32CStr::from_char_slice_truncate_mut`
    - `U32CStr::from_char_slice_unchecked_mut`
    - `U32CStr::from_char_ptr`
    - `U32CStr::from_char_ptr_truncate`
    - `U32CStr::from_char_slice`
    - `UString::as_vec`
    - `UString::as_mut_vec`
    - `UString::push_char`
    - `UString::truncate`
    - `UString::pop`
    - `UString::remove`
    - `UString::insert`
    - `UString::insert_ustr`
    - `UString::split_off`
    - `UCString::as_mut_ucstr`
    - `UCString::into_ustring`
    - `UCString::into_ustring_with_nul`
    - `U32CString::from_char_ptr_str`

### Deprecated
- Deprecated functions as part of simplifying to increase clarity. These will be removed entirely
  in a future release.
    - `UCString::from_vec_with_nul_unchecked`. Use `from_vec_unchecked` instead.
    - `UCString::from_ustr_with_nul_unchecked`. Use `from_ustr_unchecked` instead.
    - `UCString::from_ptr_with_nul_unchecked`. Use `from_ptr_unchecked` instead.
    - `UCString::from_str_with_nul_unchecked`. Use `from_str_unchecked` instead.
    - `UCString::from_os_str_with_nul_unchecked`. Use `from_os_str_unchecked` instead.
    - `U32CString::from_chars_with_nul_unchecked`. Use `from_chars_unchecked` instead.
    - `U32CString::from_char_ptr_with_nul_unchecked`. Use `from_char_ptr_unchecked` instead.

## [0.4.3] - 2020-10-05 <a name="0.4.3"></a>
### Fixed
- Fixed undefined behaviours and cleaned up clippy warnings. By [@joshwd36].

## [0.4.2] - 2020-06-09 <a name="0.4.2"></a>
### Fixed
- Fixed compile errors on pre-1.36.0 Rust due to unstable `alloc` crate. Minimum supported version
  is Rust 1.34.2, the rust version for Debian stable. Fixes [#14].

## [0.4.1] - 2020-06-08 <a name="0.4.1"></a>
### ***Yanked***
### Changed
- Now supports `no_std`. Added the `std` and `alloc` features, enabled by default. `U16String`,
  `U32String`, `U16CString`, and `U32CString` and their aliases all require the `alloc` or `std`
  feature. By [@nicbn].

## [0.4.0] - 2018-08-18 <a name="0.4.0"></a>
### Added
- New `U32String`, `U32Str`, `U32CString`, and `U32CStr` types for dealing with UTF-32 FFI. These
  new types are roughly equivalent to the existing UTF-16 types.
- `WideChar` is a type alias to `u16` on Windows but `u32` on non-Windows platforms.
- The generic types `UString`, `UStr`, `UCString` and `UCStr` are used to implement the string
  types.

### Changed
- **Breaking Change** Existing wide string types have been renamed to `U16String`, `U16Str`,
  `U16CString`, and `U16CStr` (previously `WideString`, `WideStr`, etc.). Some function have
  also been renamed to reflect this change (`wide_str` to `u16_str`, etc.).
- **Breaking Change** `WideString`, `WideStr`, `WideCString`, and `WideCStr` are now type aliases
  that vary between platforms. On Windows, these are aliases to the `U16` types and are equivalent
  to the previous version, but on non-Windows platforms these alias the new `U32` types instead.
  See crate documentation for more details.

## [0.3.0] - 2018-03-17 <a name="0.3.0"></a>
### Added
- Additional unchecked functions on `WideCString`.
- All types now implement `Default`.
- `WideString::shrink_to_fit`
- `WideString::into_boxed_wide_str` and `Box<WideStr>::into_wide_string`.
- `WideCString::into_boxed_wide_c_str` and `Box<WideCStr>::into_wide_c_string`.
- `From` and `Default` implementations for boxed `WideStr` and boxed `WideCStr`.

### Changed
- Renamed `WideCString::from_vec` to replace `WideCString::new`. To create empty string, use
  `WideCString::default()` now.
- `WideCString` now implements `Drop`, which sets the string to an empty string to prevent invalid
  unsafe code from working correctly when it should otherwise break. Also see `Drop` implementation
  of `CString`.
- Writing changelog manually.
- Upgraded winapi dev dependency.
- Now requires at least Rust 1.17+ to compile (previously, was Rust 1.8).

## [0.2.2] - 2016-09-09 <a name="0.2.2"></a>
### Fixed
- Make `WideCString::into_raw` correctly forget the original self.

## [0.2.1] - 2016-08-12 <a name="0.2.1"></a>
### Added
- `into_raw`/`from_raw` on `WideCString`. Closes [#2].

## [0.2.0] - 2016-05-31 <a name="0.2.0"></a>
### Added
- `Default` trait to wide strings.
- Traits for conversion of strings to `Cow`.
### Changed
- Methods & traits to bring to parity with Rust 1.9 string APIs.

## 0.1.0 - 2016-02-06 <a name="0.1.0"></a>
### Added
- Initial release.

[#2]: https://github.com/starkat99/widestring-rs/issues/2
[#14]: https://github.com/starkat99/widestring-rs/issues/14
[#18]: https://github.com/starkat99/widestring-rs/issues/18
[#19]: https://github.com/starkat99/widestring-rs/issues/19
[#20]: https://github.com/starkat99/widestring-rs/issues/20
[#22]: https://github.com/starkat99/widestring-rs/issues/22

[@nicbn]: https://github.com/nicbn
[@joshwd36]: https://github.com/joshwb36

[Unreleased]: https://github.com/starkat99/widestring-rs/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/starkat99/widestring-rs/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/starkat99/widestring-rs/compare/v0.4.3...v0.5.0
[0.4.3]: https://github.com/starkat99/widestring-rs/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/starkat99/widestring-rs/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/starkat99/widestring-rs/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/starkat99/widestring-rs/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/starkat99/widestring-rs/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/starkat99/widestring-rs/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/starkat99/widestring-rs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/starkat99/widestring-rs/compare/v0.1.0...v0.2.0
