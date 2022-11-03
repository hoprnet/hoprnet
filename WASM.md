# Adding WASM-compatible Rust crates

This guide describes the process of adding a new Rust crate to a package in our monorepo.

## Structure

All our packages contain the `crates` directory, that is meant to contain the Rust crates.
E.g. the `utils` package looks as follows:

```text
utils
    ├── crates
    │   ├── Makefile
    │   ├── utils-metrics
    │   │   ├── Cargo.toml
    │   │   ├── README.md
    │   │   ├── src
    │   │   │   ├── lib.rs
    │   │   │   └── metrics.rs
    │   │   └── tests
    │   │       └── wasm.rs
    │   └── utils-misc
    │       ├── Cargo.toml
    │       ├── src
    │       │   ├── lib.rs
    │       │   └── utils.rs
    │       └── tests
    │           └── node.rs

```

Each crate is prefixed using the name of the package, therefore the crate names are

- `utils-metrics`
- `utils-misc`

In addition, all Rust crates across all packages must be enumerated in `Cargo.toml` in the root of the Monorepo:

```toml
[workspace]

members = [
    "packages/real/crates/real-base",
    "packages/hoprd/crates/hoprd-misc",
    "packages/utils/crates/utils-misc",
    "packages/utils/crates/utils-metrics"
]

# ... etc.
```

The `Makefile` which is present in `crates` directory of each package takes care of building and testing the
crates for that respective package. The crates in a package must be enumerated on the first line of the respective `Makefile`.
e.g. in `utils` crate:

```makefile
# Append here when new Rust WASM crate is added (the list is space separated)
PACKAGES = utils-misc utils-metrics
##########################################

# ... etc.
```

The `Makefile` supports basic actions:

- `clean`
- `all`
- `install`
- `test`

### Testing

The unit tests are meant to be in the same module file, for each Rust module.
They are pure-Rust and are easy to debug with IDE.

The integration tests, that run in WASM runtime are currently not possible to be debugged
and are located in the `test` directory of a crate.
