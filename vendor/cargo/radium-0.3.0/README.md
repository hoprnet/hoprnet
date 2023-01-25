# radium

[![Latest Version](https://img.shields.io/crates/v/radium.svg)](https://crates.io/crates/radium)
[![Documentation](https://docs.rs/radium/badge.svg)](https://docs.rs/radium)

`radium` provides a helper trait with a uniform API for interacting with both
atomic types like [`AtomicUsize`], and non-atomic types like [`Cell<usize>`].

[`AtomicUsize`]: https://doc.rust-lang.org/core/sync/atomic/struct.AtomicUsize.html
[`Cell<usize>`]: https://doc.rust-lang.org/core/cell/struct.Cell.html

This crate is `#![no_std]`-compatible, and uses no non-core types.

For more details, see the trait's documentation.

---

**@kneecaw** - <https://twitter.com/kneecaw/status/1132695060812849154>
> Feelin' lazy: Has someone already written a helper trait abstracting
> operations over `AtomicUsize` and `Cell<usize>` for generic code which may
> not care about atomicity?

**@ManishEarth** - <https://twitter.com/ManishEarth/status/1132706585300496384>
> no but call the crate radium
>
> (since people didn't care that it was radioactive and used it in everything)
