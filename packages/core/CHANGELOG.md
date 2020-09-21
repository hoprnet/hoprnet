# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- HOPR uses its own STUN servers 🚀
- Use WebRTC over UDP by default and let WebRTC decide which transport protocol will be used
- `yarn demo` spawns its own mini-testnet, including bootstrap server and persistent blockchain

### Changed

- HOPR uses public keys as addresses instead of hashed public keys 🚀
- `node hopr <id>` spawns a HOPR node at port _9091 + <id>_
- crawling: crawling is not block anymore, leads to faster crawling
- heartbeat: every connection uses its own timer now

### Fixed

- catching various previously uncatched errors

## [0.3.0]

### Added

- Command-line Interface

### Fixed

- lots of issues around opening / closing procedure

### Known problems

- Empty responses occasionally lead crashes

## [0.2.0]

### Added:

- support for asynchronous acknowledgements
- promisification mostly done
- configuration inside `.env` file

### Fixed

- instabilities due to callbacks
