# `sval`: Streaming, structured values

[![Rust](https://github.com/sval-rs/sval/workflows/sval/badge.svg)](https://github.com/sval-rs/sval/actions)
[![Latest version](https://img.shields.io/crates/v/sval.svg)](https://crates.io/crates/sval)
[![Documentation Latest](https://docs.rs/sval/badge.svg)](https://docs.rs/sval)

`sval` is a lightweight serialization-only framework that treats values like a flat stream of tokens.
It's well suited to self-describing text formats like JSON.

## How is this different from `serde`?

`serde` is the de-facto serialization framework for Rust and is well suited to the majority of
use cases. `sval` is like a light blend of `serde::ser` and `serde::de` that is smaller in scope.
It makes a few key different design decisions than `serde` that make it effective for working with
self-describing formats:

1. The API is flat rather than using recursion to stream nested datastructures.
2. All values with dynamic sizes, including text strings, can be streamed in multiple calls.
3. Borrowing is an optional optimization.
4. The core data model is small, with tags for extensibility.

# Data-model

- Values:
    - `null`: the absence of any other meaningful value.
    - Booleans: `true` and `false`.
    - Text strings: stream of UTF8-encoded bytes.
    - Binary strings: stream of arbtirary bytes.
    - Numbers:
        - Integers: `u8`-`u128`, `i8`-`i128`.
        - Binary floating point: `f32`-`f64`.
    - Maps: heterogeneous collection of key-value pairs.
    - Sequences: heterogeneous collection of values.
    - Tags: out-of-band type hints.
    - Tagged values: a tag associated with a value.
    - Records: tagged maps where keys are well-known labels.
    - Tuples: tagged sequences.
    - Enums: tagged variants, where variants are enums, tags, tagged values, records, or tuples.

`sval` includes built-in tags that extend its data-model with some common datatypes:

- Rust primitives:
    - `()`.
    - `Option<T>`.
- Arbitrary-precision decimal floating point numbers.

Other built-in tags may be added in the future. Libraries may also define their own tags.

## Current status

This project has a complete and stable API, but isn't well documented yet.
