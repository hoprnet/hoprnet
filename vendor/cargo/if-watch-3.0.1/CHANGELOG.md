# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.0.1]

### Fixed

- For all architectures running the fallback option (e.g. Android) reverse the logic when checking if a recorded interface still exists in the new list to avoid reporting all interfaces as down and then up in the same resync().
  See [PR 31].

[PR 31]: https://github.com/mxinden/if-watch/pull/31

## [3.0.0]

### Changed
- Feature gate async runtime, allowing opting between Tokio or smol. For every OS each `IfWatcher` is
  under the `tokio` or `smol` module. This makes it a breaking change as there
  is no more a default implementation. See [PR 27](https://github.com/mxinden/if-watch/pull/27).

## [2.0.0]

### Changed
- Add `IfWatcher::poll_if_event`. Implement `Stream` instead of `Future` for `IfWatcher`.
  See [PR 23] and [PR 25].
- Make `IfWatcher::new` synchronous. See [PR 24].

[PR 23]: https://github.com/mxinden/if-watch/pull/23
[PR 24]: https://github.com/mxinden/if-watch/pull/24
[PR 25]: https://github.com/mxinden/if-watch/pull/25

## [1.1.1]

### Fixed
- Update to `rtnetlink` `v0.10`. See [PR 19].

[PR 19]: https://github.com/mxinden/if-watch/pull/19

## [1.1.0]
### Added
- Return socket closure as error. See [PR 15].

### Fixed
- Update to `windows` `v0.34`. See [PR 16].

[PR 15]: https://github.com/mxinden/if-watch/pull/15
[PR 16]: https://github.com/mxinden/if-watch/pull/16

## [1.0.0] - 2022-01-12
### Added
- macos/ios backend

### Changed
- linux backend rewritten to use rtnetlink
- windows backend rewritten to use windows crate instead of winapi
