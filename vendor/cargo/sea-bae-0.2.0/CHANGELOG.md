# Change Log

All user visible changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/), as described
for Rust libraries in [RFC #1105](https://github.com/rust-lang/rfcs/blob/master/text/1105-api-evolution.md)

## Unreleased

None.

### Breaking changes

None.

## 0.1.7

- Update dependencies.

## 0.1.6

- Also generate `try_from_attributes` which can be used to parse optional attributes.

## 0.1.5

- Use correct keywords used in Cargo.toml

## 0.1.4

- Fix potential panic caused by unwrapping a `None`. The panic could happen while computing the span for an error message if not compiling on nightly.

## 0.1.3

- Make version requirements for dependencies less strict.

## 0.1.2

Make docs.rs rebuild.

## 0.1.1

Add readme.

## 0.1.0

Initial release.
