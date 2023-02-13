# concolor

> **bin/lib API for managing terminal styling**

[![Documentation](https://img.shields.io/badge/docs-master-blue.svg)][Documentation]
![License](https://img.shields.io/crates/l/concolor.svg)
[![Crates Status](https://img.shields.io/crates/v/concolor.svg)](https://crates.io/crates/concolor)

Features
- Detects interactive `stdout` / `stderr`
- Detects terminal capabilities via `TERM`
- Detects and enables ANSI support on Windows
- Supports [CLICOLOR] and [NO_COLOR]

## [Contribute](../../CONTRIBUTING.md)

Special note: to be successful, this crate **cannot** break compatibility or
else different crates in the hierarchy will be reading different globals.
While end users can work around this, it isn't ideal.  Once we hit 1.0, we
should strive to keep the API compatible.  If we need a new API, we can make
the old API an adapter to the new logic.

Similarly, we should strive to reduce **risk** of breaking compatibility by
exposing as little as possible.  Anything more should be broken out into a
separate crate that this crate can call into.

## Special Thanks

Prior art for global colors control:

- [yansi](https://crates.io/crates/yansi)
- [clicolors-control](https://crates.io/crates/clicolors-control)

[termcolor](https://crates.io/crates/termcolor) for identifying various corner cases with environment detection.

[firestorm](https://crates.io/crates/firestorm) for zero-cost abstraction via bin/lib-specific `Cargo.toml` features.

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or [Apache 2.0](../../LICENSE-APACHE)

[Documentation]: https://docs.rs/concolor
[CLICOLOR]: https://bixense.com/clicolors/
[NO_COLOR]: https://no-color.org/
