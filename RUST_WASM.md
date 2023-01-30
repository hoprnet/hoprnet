# Documentation of our usage of Rust & WASM

This file documents how we're using our Rust toolchain to build WASM compatible crates that are used in
our monorepo.

It is also meant as place that can contain various tips that make development of WASM-compatible creates easier,
including tips related to `wasm-bindgen`.

## Structure

All our packages contain the `crates` directory, that is meant to contain the Rust crates.
E.g. the `utils` package may look as follows:

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
  ... etc.

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
and are located in the `tests` directory of a crate.
The developer is strongly encouraged to avoid tests in the `tests` directory and use TypeScript tests instead.

## Adding a new crate

To add a new Rust WASM module (crate) into an existing package:

1. `cd packages/<package>/crates`
2. `wasm-pack new my-crate --template https://github.com/hoprnet/hopr-wasm-template`, this will create a new Rust crate `my-crate` from a template.
3. remove the cloned Git repo: `rm -rf my-crate/.git`
4. add `my-crate` to `PACKAGES` space separated list in `Makefile` inside the corresponding `crates` directory.
5. add `my-crate` to `workspace.members` in `Cargo.toml` in the root of the Monorepo.
6. run `make all && make install` for the first time.
7. commit all changes: `git add my-crate && git commit -a`

You can use the following pattern in TS to export Rust types or functions:

```typescript
// Load `my-crate` crate
import my_crate_panic_hook from '../lib/my_crate.js'
my_crate_panic_hook()
export { foo } from '../lib/my_crate.js'
```

Note, that the panic hook needs to be installed for each crate separately (each crate run in separate execution environment).

Triggering the build of the entire monorepo (`make deps build`) should now also build
and integrate your new Rust crate. Similarly `make test` in the root of the monorepo
will trigger your Rust unit tests and WASM integration tests.

## Other guidelines or tips

TBD
