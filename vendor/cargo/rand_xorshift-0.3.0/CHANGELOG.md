# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2020-12-18
- Bump `rand_core` version to 0.6 (#17)
- Derive PartialEq+Eq for XorShiftRng (#6)
- Bump serde to 1.0.118 so that `serde1` feature can also be no-std (#12)

## [0.2.0] - 2019-06-12
- Bump minor crate version since rand_core bump is a breaking change
- Switch to Edition 2018

## [0.1.2] - 2019-06-06 - yanked
- Bump `rand_core` version
- Make XorShiftRng::from_rng portable by enforcing Endianness (#815)

## [0.1.1] - 2019-01-04
- Reorganise code and tests; tweak doc

## [0.1.0] - 2018-07-16
- Pulled out of the Rand crate
