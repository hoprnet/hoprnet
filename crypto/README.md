# core-crypto

This Rust crate contains cryptographic primitives and basic types that
are used in the HOPR protocol.

The higher-level operations are meant to be in the separate crates, like `core-packet`.

The crate is structured into the following modules:

- `derivation`: contains simple key derivation functions for different purposes
- `ec_groups`: implementation of `SphinxSuite` trait, see the chapter below.
- `error`: various errors produced by this crate
- `iterated_hash`: hash commitment implementation for the tickets
- `keypairs`: defines the two types of keypairs (`ChainKeypair` used for on-chain operations - based on secp256k1, and `OffchainKeypair` for packet operations - based on Ed25519)
- `parameters`: contains various cryptography related global constants
- `prg`: implementation of a pseudo-random generator function used in SPHINX packet header construction
- `primitives`: contains implementation of cryptographic primitives: Blake2s256 digest, Keccak256 digest, Mac using Blake2s256 and ChaCha20 stream cipher
- `prp`: implementation of the Lioness wide-block cipher using Chacha20 and Blake2b256
- `random`: all functions that generate something requiring randomness source are implemented here
- `routing`: implements the SPHINX header
- `shared_keys`: derivation of shared keys for SPHINX header (see below)
- `types`: general use types for the entire code base, e.g. public keys, signatures,...
- `utils`: generic utility types and functions used by this crate

## SPHINX shared keys derivation

The architecture of the SPHINX shared key derivation is done generically, so it can work with any elliptic curve group for which CDH problem is
hard. The generic Sphinx implementation only requires one to implement the `SphinxSuite` trait.
The trait requires to have the following building blocks:

- elliptic curve group (`GroupElement`) and corresponding the scalar type (`Scalar`)
- type representing public and private keypair and their conversion to `Scalar` and `GroupElement` (by the means of the corresponding `From` trait implementation)

Currently, there are the following `SphinxSuite` implementations :

- `Secp256k1Suite`: deprecated, used in previous HOPR versions
- `Ed25519Suite`: simple implementation using Ed25519, used for testing
- `X25519Suite`: currently used, implemented using Curve25519 Montgomery curve for faster computation

The implementation can be easily extended for different elliptic curves (or even arithmetic multiplicative groups).
In particular, as soon as there's way to represent `Ed448` PeerIDs, it would be easy to create e.g. `X448Suite`.
