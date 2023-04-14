[![docs.rs](https://docs.rs/asn1_der/badge.svg)](https://docs.rs/asn1_der)
[![License BSD-2-Clause](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/asn1_der.svg)](https://crates.io/crates/asn1_der)
[![Download numbers](https://img.shields.io/crates/d/asn1_der.svg)](https://crates.io/crates/asn1_der)
[![AppVeyor CI](https://ci.appveyor.com/api/projects/status/github/KizzyCode/asn1_der-rust?svg=true)](https://ci.appveyor.com/project/KizzyCode/asn1-der-rust)
[![dependency status](https://deps.rs/crate/asn1_der/latest/status.svg)](https://deps.rs/crate/asn1_der)

# asn1_der
Welcome to `asn1_der` 🎉

This crate provides a basic `no_std`-compatible, [no-panic](#no-panic) and [zero-copy](#zero-copy)
DER implementation. It is designed to be reliable and reasonable fast without getting too large or
sacrificing too much comfort. To achieve this, `asn1_der` makes extensive use of the
[`no-panic`](https://crates.io/crates/no-panic) crate and offers slice-based object views to avoid
allocations and unnecessary copies.


## Example
```ignore
use asn1_der::{
    DerObject,
    typed::{ DerEncodable, DerDecodable }
};

fn main() {
    /// An ASN.1-DER encoded integer `7`
    const INT7: &'static[u8] = b"\x02\x01\x07";

    // Decode an arbitrary DER object
    let object = DerObject::decode(INT7).expect("Failed to decode object");

    // Encode an arbitrary DER object
    let mut encoded_object = Vec::new();
    object.encode(&mut encoded_object).expect("Failed to encode object");

    // Decode a `u8`
    let number = u8::decode(INT7).expect("Failed to decode number");
    assert_eq!(number, 7);

    // Encode a new `u8`
    let mut encoded_number = Vec::new();
    7u8.encode(&mut encoded_number).expect("Failed to encode number");
}
```

For the (de-)serialization of structs and similar via `derive`, see 
[`serde_asn1_der`](https://crates.io/crates/serde_asn1_der).


## Typed Implementations
There are also some direct `DerDecodable`/`DerDecodable` implementations for native Rust type 
equivalents:
 - The ASN.1-`BOOLEAN` type as Rust-`bool`
 - The ASN.1-`INTEGER` type as Rust-[`u8`, `u16`, `u32`, `u64`, `u128`, `usize`]
 - The ASN.1-`NULL` type as either `()` or `Option::None` (which allows the encoding of
   optionals)
 - The ASN.1-`OctetString` type as `Vec<u8>`
 - The ASN.1-`SEQUENCE` type as `SequenceVec(Vec<T>)`
 - The ASN.1-`UTF8String` type as `String`


## No-Panic
`asn1_der` is designed to be as panic-free as possible. To ensure that, nearly every function is
attributed with `#[no_panic]`, which forces the compiler to prove that a function cannot panic in
the given circumstances. However since `no_panic` can cause a lot of false-positives, it is
currently only used by the CI-tests and disabled by default in normal builds. If you want to use
this crate with `no_panic` enabled, you can do so by specifying the `no_panic` feature.

### What No-Panic Does Not Cover
It is important to know that `no_panic` is no silver bullet and does not help against certain kinds
of errors that can also happen in this crate. This especially includes:
 - Dynamic memory allocation errors: Since it is not possible to predict memory allocation errors,
   everything that requires dynamic memory allocation is mutually exclusive to `no_panic` and will
   be omitted if `no_panic` is enabled.
   
   This crate might allocate memory in the following circumstances:
    - When writing to a dynamically allocating sink (e.g. `Vec<u8>`, `VecBacking(Vec<u8>)`)
    - When decoding a native owned type such as `Vec<u8>`, `SequenceVec(Vec<T>)` or `String`
    - During error propagation
   
   If the crate is compiled without `std` enabled, it does performy any dynamic memory allocation 
   directly by itself – however for foreign implementations passed to this crate may still allocate 
   memory and fail (e.g. a custom `Sink` implementation).
   
 - Stack overflows: Since the stack size is not necessarily known during compile time, it is not
   possible to predict stack overflow errors e.g. caused by recursion.
 - Calls to `abort` or similar: Since calls to `abort` or similar do not trigger stack unwinding,
   they can also no be detected by `no_panic`. __This also means that `no_panic` does not work for
   builds that use `panic = "abort"` in their config.__
   
   This crate by itself does never call `abort` directly.

Due to the limitations described above, the following functions are mutually exclusive to
`no_panic` and disabled if `no_panic` is set:
 - Error stacking/propagation (`propagate` is a no-op if compiled with `no_panic`)
 - The sink implementation for a byte vector (`impl Sink for Vec<u8>`)
 - The `VecBacking(Vec<u8>)` type
 - The native OctetString type which uses `Vec<u8>` (`impl<'a> DerDecodable<'a> for Vec<u8>` and
   `impl DerEncodable for Vec<u8>`)
 - The native Sequence type wrapper `SequenceVec` since it is based upon `Vec`
 - The native Utf8String type based upon `String` (`impl<'a> DerDecodable<'a> for String` and
   `impl DerEncodable for String`)


## Zero-Copy
The crate is designed to be as much zero-copy as possible. In fact this means that the `DerObject`
type and all typed views are zero-copy views over the underlying slice. Of course, zero-copy is not
always reasonable: The `new`-constructors are not zero-copy because they construct a new object into
a sink and the native type implementations are not zero-copy because they are either `Copy`-types
(e.g. `u128`) or owned (e.g. `String`).


## What happened to `asn1_der_derive`?
Since version 0.7.0, the `asn1_der_derive`-crates has been deprecated in favor of
[`serde_asn1_der`](https://crates.io/crates/serde_asn1_der). If you have a specific use-case why you
cannot use `serde`, let me know; it's probably not that hard to revive `asn1_der_derive` 😊