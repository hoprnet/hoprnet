# subtle-ng

**Pure-Rust traits and utilities for constant-time cryptographic implementations.**

It consists of a `Choice` type, and a collection of traits using `Choice`
instead of `bool` which are intended to execute in constant-time.  The `Choice`
type is a wrapper around a `u8` that holds a `0` or `1`.

```toml
subtle-ng = "2.5"
```

To switch from the previous `subtle` crate without changing your code,
use the following:
```toml
subtle = { package = "subtle-ng", version = "2.4" }
```

This crate continues the `subtle` series under a different package name.
Unfortunately, one of the maintainers of the previous crate seized control of the
`dalek-cryptography` GitHub organization and the `subtle` and `curve25519-dalek`
crates by silently removing all other co-maintainers.

This crate represents a “best-effort” attempt, since side-channels
are ultimately a property of a deployed cryptographic system
including the hardware it runs on, not just of software.

The traits are implemented using bitwise operations, and should execute in
constant time provided that a) the bitwise operations are constant-time and
b) the bitwise operations are not recognized as a conditional assignment and
optimized back into a branch.

For a compiler to recognize that bitwise operations represent a conditional
assignment, it needs to know that the value used to generate the bitmasks is
really a boolean `i1` rather than an `i8` byte value. In an attempt to
prevent this refinement, the crate tries to hide the value of a `Choice`'s
inner `u8` by passing it through a volatile read. For more information, see
the _About_ section below.

Versions prior to `2.2` recommended use of the `nightly` feature to enable an
optimization barrier; this is not required in versions `2.2` and above.

Note: the `subtle` crate contains `debug_assert`s to check invariants during
debug builds. These invariant checks involve secret-dependent branches, and
are not present when compiled in release mode. This crate is intended to be
used in release mode.

## Documentation

Documentation is available [here][docs].

## Minimum Supported Rust Version

Rust **1.54** or higher.

Minimum supported Rust version can be changed in the future, but it will be
done with a minor version bump.

## About

This library aims to be the Rust equivalent of Go’s `crypto/subtle` module.

The optimization barrier in `impl From<u8> for Choice` was based on Tim
Maclean's [work on `rust-timing-shield`][rust-timing-shield], which attempts to
provide a more comprehensive approach for preventing software side-channels in
Rust code.

`subtle` is authored by isis agora lovecruft and Henry de Valence.

## Warning

This code is a low-level library, intended for specific use-cases implementing
cryptographic protocols.  It represents a best-effort attempt to protect
against some software side-channels.  Because side-channel resistance is not a
property of software alone, but of software together with hardware, any such
effort is fundamentally limited.

**USE AT YOUR OWN RISK**

[docs]: https://docs.rs/subtle-ng
[rust-timing-shield]: https://www.chosenplaintext.ca/open-source/rust-timing-shield/security
