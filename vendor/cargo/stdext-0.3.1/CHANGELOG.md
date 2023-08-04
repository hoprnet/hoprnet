# `stdext` changelog

## [[Unreleased]]

## 0.3.0 (18.06.2021)

- BREAKING: MSRV got bumped to 1.53.

- `try_match` and `unwrap_match` macros to get a certain variant from an enum [#11].
- `return_ok` and `return_some` macros for early return of successful calculation [#11].
- `Integer` trait that unifies all the built-in integer types under a single interface [#12].
- `FloatConvert` trait that adds an interface for converting floating point numbers into integers [#12].

[#11]: https://github.com/popzxc/stdext-rs/pull/11
[#12]: https://github.com/popzxc/stdext-rs/pull/12

## 0.2.1 (09.07.2020)

- `VecExt::remove_item` method was added [#9].

[#9]: https://github.com/popzxc/stdext-rs/pull/9

## 0.2.0 (02.07.2020)

- `compile_warning` and `function_name` macros were added [#4].

[#4]: https://github.com/popzxc/stdext-rs/pull/4
