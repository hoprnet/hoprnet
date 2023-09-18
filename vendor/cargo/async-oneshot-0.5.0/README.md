# async-oneshot

[![License](https://img.shields.io/crates/l/async-oneshot.svg)](https://github.com/irrustible/async-oneshot/blob/main/LICENSE)
[![Package](https://img.shields.io/crates/v/async-oneshot.svg)](https://crates.io/crates/async-oneshot)
[![Documentation](https://docs.rs/async-oneshot/badge.svg)](https://docs.rs/async-oneshot)

A fast, small, full-featured, async-aware oneshot channel.

Features:

* Blazing fast! See `Performance` section below.
* Tiny code, only one dependency and a lightning quick build.
* Complete `no_std` support (with `alloc` for `Arc`).
* Unique feature: sender may wait for a receiver to be waiting.

## Usage

```rust
#[test]
fn success_one_thread() {
    let (s,r) = oneshot::<bool>();
    assert_eq!((), s.send(true).unwrap());
    assert_eq!(Ok(true), future::block_on(r));
}
```

## Performance

async-oneshot comes with a benchmark suite which you can run with
`cargo bench`.

All benches are single-threaded and take double digit nanoseconds on
my machine. async benches use `futures_lite::future::block_on` as an
executor.

### Numbers from my machine

Here are benchmark numbers from my primary machine, a Ryzen 9 3900X
running alpine linux 3.12 that I attempted to peg at maximum cpu:

```
create_destroy          time:   [51.596 ns 51.710 ns 51.835 ns]
send/success            time:   [13.080 ns 13.237 ns 13.388 ns]
send/closed             time:   [25.304 ns 25.565 ns 25.839 ns]
try_recv/success        time:   [26.136 ns 26.246 ns 26.335 ns]
try_recv/empty          time:   [10.764 ns 11.161 ns 11.539 ns]
try_recv/closed         time:   [27.048 ns 27.159 ns 27.248 ns]
async.recv/success      time:   [30.532 ns 30.774 ns 31.011 ns]
async.recv/closed       time:   [28.112 ns 28.208 ns 28.287 ns]
async.wait/success      time:   [56.449 ns 56.603 ns 56.737 ns]
async.wait/closed       time:   [34.014 ns 34.154 ns 34.294 ns]
```

In short, we are very fast. Close to optimal, I think.

### Compared to other libraries

The oneshot channel in `futures` isn't very fast by comparison.

Tokio put up an excellent fight and made us work hard to improve. In
general I'd say we're slightly faster overall, but it's incredibly
tight.

## Note on safety

This crate uses UnsafeCell and manually synchronises with atomic
bitwise ops for performance. We believe it is correct, but we
would welcome more eyes on it.

# See Also

* [async-oneshot-local](https://github.com/irrustible/async-oneshot-local) (single threaded)
* [async-spsc](https://github.com/irrustible/async-spsc) (SPSC)
* [async-channel](https://github.com/stjepang/async-channel) (MPMC)

## Note on benchmarking

The benchmarks are synthetic and a bit of fun.

## Changelog

### v0.5.0

Breaking changes:

* Make `Sender.send()` only take a mut ref instead of move.

### v0.4.2

Improvements:

* Added some tests to cover repeated fix released in last version.
* Inline more aggressively for some nice benchmark boosts.

### v0.4.1

Fixes:

* Remove some overzealous `debug_assert`s that caused crashes in
  development in case of repeated waking. Thanks @nazar-pc!

Improvements:

* Better benchmarks, based on criterion.

### v0.4.0

Breaking changes:

* `Sender.wait()`'s function signature has changed to be a non-`async
  fn` returning an `impl Future`. This reduces binary size, runtime
  and possibly memory usage too. Thanks @zserik!

Fixes:

* Race condition where the sender closes in a narrow window during
  receiver poll and doesn't wake the Receiver. Thanks @zserik!

Improvements:

 * Static assertions. Thanks @zserik!

### v0.3.3

Improvements:

* Update `futures-micro` and improve the tests

### v0.3.2

Fixes:

* Segfault when dropping receiver. Caused by a typo, d'oh! Thanks @boardwalk!

### v0.3.1

Improvements:

* Remove redundant use of ManuallyDrop with UnsafeCell. Thanks @cynecx!

### v0.3.0

Improvements:

* Rewrote, benchmarked and optimised.

### v0.2.0

* First real release.

## Copyright and License

Copyright (c) 2020 James Laver, async-oneshot contributors.

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.
