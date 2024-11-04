# RustCrypto: `crypto_box`

[![crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
![Apache2/MIT licensed][license-image]
![Rust Version][rustc-image]
[![Project Chat][chat-image]][chat-link]
[![Build Status][build-image]][build-link]

Pure Rust implementation of [NaCl]'s [`crypto_box`] primitive, providing
public-key authenticated encryption which combines the [X25519] Diffie-Hellman
function and the [XSalsa20Poly1305] authenticated encryption cipher into an
Elliptic Curve Integrated Encryption Scheme ([ECIES]).

[Documentation][docs-link]

## About

Imagine Alice wants something valuable shipped to her. Because it's
valuable, she wants to make sure it arrives securely (i.e. hasn't been
opened or tampered with) and that it's not a forgery (i.e. it's actually
from the sender she's expecting it to be from and nobody's pulling the old
switcheroo).

One way she can do this is by providing the sender (let's call him Bob)
with a high-security box of her choosing. She provides Bob with this box,
and something else: a padlock, but a padlock without a key. Alice is
keeping that key all to herself. Bob can put items in the box then put the
padlock onto it, but once the padlock snaps shut, the box cannot be opened
by anyone who doesn't have Alice's private key.

Here's the twist though, Bob also puts a padlock onto the box. This padlock
uses a key Bob has published to the world, such that if you have one of
Bob's keys, you know a box came from him because Bob's keys will open Bob's
padlocks (let's imagine a world where padlocks cannot be forged even if you
know the key). Bob then sends the box to Alice.

In order for Alice to open the box, she needs two keys: her private key
that opens her own padlock, and Bob's well-known key. If Bob's key doesn't
open the second padlock then Alice knows that this is not the box she was
expecting from Bob, it's a forgery.

## Security Notes

This crate has received one [security audit by Cure53][audit-2022] (version
0.7.1), with no significant findings. We would like to thank [Threema][threema]
for funding the audit.

## License

Licensed under either of:

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/crypto_box.svg
[crate-link]: https://crates.io/crates/crypto_box
[docs-image]: https://docs.rs/crypto_box/badge.svg
[docs-link]: https://docs.rs/crypto_box/
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[rustc-image]: https://img.shields.io/badge/rustc-1.60+-blue.svg
[chat-image]: https://img.shields.io/badge/zulip-join_chat-blue.svg
[chat-link]: https://rustcrypto.zulipchat.com/#narrow/stream/260038-AEADs
[build-image]: https://github.com/RustCrypto/nacl-compat/actions/workflows/crypto_box.yml/badge.svg
[build-link]: https://github.com/RustCrypto/nacl-compat/actions/workflows/crypto_box.yml

[//]: # (general links)

[NaCl]: https://nacl.cr.yp.to/
[`crypto_box`]: https://nacl.cr.yp.to/box.html
[X25519]: https://cr.yp.to/ecdh.html
[XSalsa20Poly1305]: https://github.com/RustCrypto/AEADs/tree/master/xsalsa20poly1305
[ECIES]: https://en.wikipedia.org/wiki/Integrated_Encryption_Scheme
[audit-2022]: https://cure53.de/pentest-report_rust-libs_2022.pdf
[threema]: https://threema.ch/
