# tracing-test

[![Build status][workflow-badge]][workflow]
[![Crates.io Version][crates-io-badge]][crates-io]
[![Crates.io Downloads][crates-io-download-badge]][crates-io-download]

This crate provides an easy way to enable logging in tests that use
[tracing](https://tracing.rs/), even if they're async. Additionally, it adds a
way to assert that certain things were logged.

The focus is on testing the logging, not on debugging the tests. That's why the
library ensures that the logs do not depend on external state. For example, the
`RUST_LOG` env variable is not used for log filtering.

Similar crates:

- [test-log](https://crates.io/crates/test-log): Initialize loggers before
  running tests
- [tracing-fluent-assertions](https://crates.io/crates/tracing-fluent-assertions):
  More powerful assertions that also allow analyzing spans

## Docs / Usage / Example

See <https://docs.rs/tracing-test/>.

## License

Copyright © 2020-2023 Threema GmbH, Danilo Bargen and Contributors.

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT) at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.


<!-- Badges -->
[workflow]: https://github.com/dbrgn/tracing-test/actions?query=workflow%3ACI
[workflow-badge]: https://img.shields.io/github/actions/workflow/status/dbrgn/tracing-test/ci.yml?branch=main
[crates-io]: https://crates.io/crates/tracing-test
[crates-io-badge]: https://img.shields.io/crates/v/tracing-test.svg?maxAge=3600
[crates-io-download]: https://crates.io/crates/tracing-test
[crates-io-download-badge]: https://img.shields.io/crates/d/tracing-test.svg?maxAge=3600
