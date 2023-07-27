# Changes for [`futures-ticker`](https://crates.io/crates/futures-ticker)

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [[0.0.3](https://docs.rs/futures-ticker/0.0.3/futures-ticker/)] - 2023-05-24

### Added
* The `Ticker` struct now derives `Debug` ([PR](https://github.com/antifuchs/futures-ticker/pull/11) by [@ozkanonur][ozkanonur])

### Changed
* Polling the ticker now calls `Instant::now` only once. ([PR](https://github.com/antifuchs/futures-ticker/pull/10) by [@mxinden][mxinden])


## [[0.0.2](https://docs.rs/futures-ticker/0.0.2/futures-ticker/)] - 2022-04-10
* Support for usage on wasm targets via the `instant` crate. Thanks, [vpzomtrrfrt](https://github.com/vpzomtrrfrt) in [#1](https://github.com/antifuchs/futures-ticker/pull/1)!

## [[0.0.1](https://docs.rs/futures-ticker/0.0.1/futures-ticker/)] - 2020-06-27

<!-- Contributors -->
[ozkanonur]: https://github.com/ozkanonur
[mxinden]: https://github.com/mxinden
