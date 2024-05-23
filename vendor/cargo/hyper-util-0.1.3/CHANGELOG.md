# 0.1.3 (2024-01-31)

### Added

- Add `Error::is_connect()` which returns true if error came from client `Connect`.
- Add timer support to `legacy` pool.
- Add support to enable http1/http2 parts of `auto::Builder` individually.

### Fixed

- Fix `auto` connection so it can handle requests shorter than the h2 preface.
- Fix `legacy::Client` to no longer error when keep-alive is diabled.

# 0.1.2 (2023-12-20)

### Added

- Add `graceful_shutdown()` method to `auto` connections.
- Add `rt::TokioTimer` type that implements `hyper::rt::Timer`.
- Add `service::TowerToHyperService` adapter, allowing using `tower::Service`s as a `hyper::service::Service`.
- Implement `Clone` for `auto::Builder`.
- Exports `legacy::{Builder, ResponseFuture}`.

### Fixed

- Enable HTTP/1 upgrades on the `legacy::Client`.
- Prevent divide by zero if DNS returns 0 addresses.

# 0.1.1 (2023-11-17)

### Added

- Make `server-auto` enable the `server` feature.

### Fixed

- Reduce `Send` bounds requirements for `auto` connections.
- Docs: enable all features when generating.

# 0.1.0 (2023-11-16)

Initial release.
