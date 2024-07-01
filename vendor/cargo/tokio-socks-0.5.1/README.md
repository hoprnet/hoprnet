# tokio-socks

[![Build Status](https://travis-ci.org/sticnarf/tokio-socks.svg?branch=master)](https://travis-ci.org/sticnarf/tokio-socks)
[![Crates Version](https://img.shields.io/crates/v/tokio-socks.svg)](https://crates.io/crates/tokio-socks)
[![docs](https://docs.rs/tokio-socks/badge.svg)](https://docs.rs/tokio-socks)

Asynchronous SOCKS proxy support for Rust.

## Features

- [x] `CONNECT` command
- [x] `BIND` command
- [ ] `ASSOCIATE` command
- [x] Username/password authentication
- [ ] GSSAPI authentication
- [ ] Asynchronous DNS resolution
- [X] Chain proxies ([see example](examples/chainproxy.rs))
- [ ] SOCKS4

## License

This project is licensed under the MIT License - see the [LICENSE](/LICENSE) file for details.

## Acknowledgments

* [sfackler/rust-socks](https://github.com/sfackler/rust-socks)
