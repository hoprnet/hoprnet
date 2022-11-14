= v0.7.0 =
* Revert removal of `try_from` dependency

= v0.6.0 =
* Upgrades to `cookies` v0.12
* Drop dependency `try_from` in lieu of `std::convert::TryFrom` (@oherrala)
* Drop dependency on `serde_derive`, rely on `serde` only (@oherrala)

= v0.4.0 =
* Update to Rust 2018 edition

= v0.3.1 =

* Upgrades to `cookies` v0.11
* Minor dependency upgrades

= v0.3 =

* Upgrades to `reqwest` v0.9
* Replaces `error-chain` with `failure`

= v0.2 =

* Removes separate `ReqwestSession::ErrorKind`. Added as variant `::ErrorKind::Reqwest` instead.
