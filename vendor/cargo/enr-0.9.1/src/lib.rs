//! # Ethereum Node Record (ENR)
//!
//! This crate contains an implementation of an Ethereum Node Record (ENR) as specified by
//! [EIP-778](https://eips.ethereum.org/EIPS/eip-778) extended to allow for the use of ed25519 keys.
//!
//! An ENR is a signed, key-value record which has an associated [`NodeId`] (a 32-byte identifier).
//! Updating/modifying an ENR requires an [`EnrKey`] in order to re-sign the recrd with the
//! associated key-pair.
//!
//! ENR's are identified by their sequence number. When updating an ENR, the sequence number is
//! increased.
//!
//! Different identity schemes can be used to define the node id and signatures. Currently only the
//! "v4" identity is supported and is set by default.
//!
//! ## Signing Algorithms
//!
//! User's wishing to implement their own singing algorithms simply need to
//! implement the [`EnrKey`] trait and apply it to an [`Enr`].
//!
//! By default, `k256::SigningKey` implement [`EnrKey`] and can be used to sign and
//! verify ENR records. This library also implements [`EnrKey`] for `ed25519_dalek::SigningKey` via the `ed25519`
//! feature flag.
//!
//! Furthermore, a [`CombinedKey`] is provided if the `ed25519` feature flag is set, which provides an
//! ENR type that can support both `secp256k1` and `ed25519` signed ENR records. Examples of the
//! use of each of these key types is given below.
//!
//! ## Features
//!
//! This crate supports a number of features.
//!
//! - `serde`: Allows for serde serialization and deserialization for ENRs.
//! - `ed25519`: Provides support for `ed25519_dalek` keypair types.
//! - `k256`: Uses `k256` for secp256k1 keys.
//! - `rust-secp256k1`: Uses `rust-secp256k1` for secp256k1 keys.
//!
//! These can be enabled via adding the feature flag in your `Cargo.toml`
//!
//! ```toml
//! enr = { version = "*", features = ["serde", "ed25519"] }
//! ```
//!
//! ## Examples
//!
//! To build an ENR, an [`EnrBuilder`] is provided.
//!
//! ### Building an ENR with the default `k256` `secp256k1` key type
//!
//! ```rust
//! use enr::{EnrBuilder, k256};
//! use std::net::Ipv4Addr;
//! use rand::thread_rng;
//!
//! // generate a random secp256k1 key
//! let mut rng = thread_rng();
//! let key = k256::ecdsa::SigningKey::random(&mut rng);
//!
//! let ip = Ipv4Addr::new(192,168,0,1);
//! let enr = EnrBuilder::new("v4").ip4(ip).tcp4(8000).build(&key).unwrap();
//!
//! assert_eq!(enr.ip4(), Some("192.168.0.1".parse().unwrap()));
//! assert_eq!(enr.id(), Some("v4".into()));
//! ```
//!
//! ### Building an ENR with the `CombinedKey` type (support for multiple signing
//! algorithms).
//!
//! Note the `ed25519` feature flag must be set. This makes use of the
//! [`EnrBuilder`] struct.
//! ```rust
//! # #[cfg(feature = "ed25519")] {
//! use enr::{EnrBuilder, CombinedKey};
//! use std::net::Ipv4Addr;
//!
//! // create a new secp256k1 key
//! let key = CombinedKey::generate_secp256k1();
//!
//! // or create a new ed25519 key
//! let key = CombinedKey::generate_ed25519();
//!
//! let ip = Ipv4Addr::new(192,168,0,1);
//! let enr = EnrBuilder::new("v4").ip4(ip).tcp4(8000).build(&key).unwrap();
//!
//! assert_eq!(enr.ip4(), Some("192.168.0.1".parse().unwrap()));
//! assert_eq!(enr.id(), Some("v4".into()));
//! # }
//! ```
//!
//! ### Modifying an [`Enr`]
//!
//! ENR fields can be added and modified using the getters/setters on [`Enr`]. A custom field
//! can be added using [`insert`] and retrieved with [`get`].
//!
//! ```rust
//! use enr::{EnrBuilder, k256::ecdsa::SigningKey, Enr};
//! use std::net::Ipv4Addr;
//! use rand::thread_rng;
//!
//! // specify the type of ENR
//! type DefaultEnr = Enr<SigningKey>;
//!
//! // generate a random secp256k1 key
//! let mut rng = thread_rng();
//! let key = SigningKey::random(&mut rng);
//!
//! let ip = Ipv4Addr::new(192,168,0,1);
//! let mut enr = EnrBuilder::new("v4").ip4(ip).tcp4(8000).build(&key).unwrap();
//!
//! enr.set_tcp4(8001, &key);
//! // set a custom key
//! enr.insert("custom_key", &vec![0,0,1], &key);
//!
//! // encode to base64
//! let base_64_string = enr.to_base64();
//!
//! // decode from base64
//! let decoded_enr: DefaultEnr = base_64_string.parse().unwrap();
//!
//! assert_eq!(decoded_enr.ip4(), Some("192.168.0.1".parse().unwrap()));
//! assert_eq!(decoded_enr.id(), Some("v4".into()));
//! assert_eq!(decoded_enr.tcp4(), Some(8001));
//! assert_eq!(decoded_enr.get("custom_key"), Some(vec![0,0,1].as_slice()));
//! ```
//!
//! ### Encoding/Decoding ENR's of various key types
//!
//! ```rust
//! # #[cfg(feature = "ed25519")] {
//! use enr::{EnrBuilder, k256::ecdsa, Enr, ed25519_dalek as ed25519, CombinedKey};
//! use std::net::Ipv4Addr;
//! use rand::thread_rng;
//! use rand::Rng;
//!
//! // generate a random secp256k1 key
//! let mut rng = thread_rng();
//! let key = ecdsa::SigningKey::random(&mut rng);
//! let ip = Ipv4Addr::new(192,168,0,1);
//! let enr_secp256k1 = EnrBuilder::new("v4").ip4(ip).tcp4(8000).build(&key).unwrap();
//!
//! // encode to base64
//! let base64_string_secp256k1 = enr_secp256k1.to_base64();
//!
//! // generate a random ed25519 key
//! let key = ed25519::SigningKey::generate(&mut rng);
//! let enr_ed25519 = EnrBuilder::new("v4").ip4(ip).tcp4(8000).build(&key).unwrap();
//!
//! // encode to base64
//! let base64_string_ed25519 = enr_ed25519.to_base64();
//!
//! // decode base64 strings of varying key types
//! // decode the secp256k1 with default Enr
//! let decoded_enr_secp256k1: Enr<k256::ecdsa::SigningKey> = base64_string_secp256k1.parse().unwrap();
//! // decode ed25519 ENRs
//! let decoded_enr_ed25519: Enr<ed25519_dalek::SigningKey> = base64_string_ed25519.parse().unwrap();
//!
//! // use the combined key to be able to decode either
//! let decoded_enr: Enr<CombinedKey> = base64_string_secp256k1.parse().unwrap();
//! let decoded_enr: Enr<CombinedKey> = base64_string_ed25519.parse().unwrap();
//! # }
//! ```
//!
//!
//! [`CombinedKey`]: enum.CombinedKey.html
//! [`EnrKey`]: trait.EnrKey.html
//! [`Enr`]: struct.Enr.html
//! [`EnrBuilder`]: struct.EnrBuilder.html
//! [`NodeId`]: struct.NodeId.html
//! [`insert`]: struct.Enr.html#method.insert
//! [`get`]: struct.Enr.html#method.get

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::map_err_ignore,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::option_if_let_else
)]

mod builder;
mod error;
mod keys;
mod node_id;

use bytes::{Bytes, BytesMut};
use log::debug;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use std::{
    collections::BTreeMap,
    hash::{Hash, Hasher},
    net::{SocketAddrV4, SocketAddrV6},
};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
#[cfg(feature = "serde")]
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use sha3::{Digest, Keccak256};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    str::FromStr,
};

pub use builder::EnrBuilder;
pub use error::EnrError;

#[cfg(feature = "k256")]
pub use keys::k256;
#[cfg(feature = "rust-secp256k1")]
pub use keys::secp256k1;
#[cfg(all(feature = "ed25519", feature = "k256"))]
pub use keys::{ed25519_dalek, CombinedKey, CombinedPublicKey};

pub use keys::{EnrKey, EnrKeyUnambiguous, EnrPublicKey};
pub use node_id::NodeId;
use std::marker::PhantomData;

/// The "key" in an ENR record can be arbitrary bytes.
type Key = Vec<u8>;
type PreviousRlpEncodedValues = Vec<Option<Bytes>>;

const MAX_ENR_SIZE: usize = 300;

/// The ENR, allowing for arbitrary signing algorithms.
///
/// This struct will always have a valid signature, known public key type, sequence number and `NodeId`. All other parameters are variable/optional.
pub struct Enr<K: EnrKey> {
    /// ENR sequence number.
    seq: u64,

    /// The `NodeId` of the ENR record.
    node_id: NodeId,

    /// Key-value contents of the ENR. A BTreeMap is used to get the keys in sorted order, which is
    /// important for verifying the signature of the ENR.
    /// Everything is stored as raw RLP bytes.
    content: BTreeMap<Key, Bytes>,

    /// The signature of the ENR record, stored as bytes.
    signature: Vec<u8>,

    /// Marker to pin the generic.
    phantom: PhantomData<K>,
}

impl<K: EnrKey> Enr<K> {
    // getters //

    /// The `NodeId` for the record.
    #[must_use]
    pub const fn node_id(&self) -> NodeId {
        self.node_id
    }

    /// The current sequence number of the ENR record.
    #[must_use]
    pub const fn seq(&self) -> u64 {
        self.seq
    }

    /// Reads a custom key from the record if it exists, decoded as data.
    #[allow(clippy::missing_panics_doc)]
    pub fn get(&self, key: impl AsRef<[u8]>) -> Option<&[u8]> {
        // It's ok to decode any valid RLP value as data
        self.get_raw_rlp(key).map(|rlp_data| {
            rlp::Rlp::new(rlp_data)
                .data()
                .expect("All data is sanitized")
        })
    }

    /// Reads a custom key from the record if it exists, decoded as `T`.
    pub fn get_decodable<T: Decodable>(
        &self,
        key: impl AsRef<[u8]>,
    ) -> Option<Result<T, DecoderError>> {
        self.get_raw_rlp(key).map(|rlp_data| rlp::decode(rlp_data))
    }

    /// Reads a custom key from the record if it exists as raw RLP bytes.
    pub fn get_raw_rlp(&self, key: impl AsRef<[u8]>) -> Option<&[u8]> {
        self.content.get(key.as_ref()).map(AsRef::as_ref)
    }

    /// Returns an iterator over all key/value pairs in the ENR.
    pub fn iter(&self) -> impl Iterator<Item = (&Key, &[u8])> {
        self.content.iter().map(|(k, v)| (k, v.as_ref()))
    }

    /// Returns the IPv4 address of the ENR record if it is defined.
    #[must_use]
    pub fn ip4(&self) -> Option<Ipv4Addr> {
        if let Some(ip_bytes) = self.get("ip") {
            return match ip_bytes.len() {
                4 => {
                    let mut ip = [0_u8; 4];
                    ip.copy_from_slice(ip_bytes);
                    Some(Ipv4Addr::from(ip))
                }
                _ => None,
            };
        }
        None
    }

    /// Returns the IPv6 address of the ENR record if it is defined.
    #[must_use]
    pub fn ip6(&self) -> Option<Ipv6Addr> {
        if let Some(ip_bytes) = self.get("ip6") {
            return match ip_bytes.len() {
                16 => {
                    let mut ip = [0_u8; 16];
                    ip.copy_from_slice(ip_bytes);
                    Some(Ipv6Addr::from(ip))
                }
                _ => None,
            };
        }
        None
    }

    /// The `id` of ENR record if it is defined.
    #[must_use]
    pub fn id(&self) -> Option<String> {
        if let Some(id_bytes) = self.get("id") {
            return Some(String::from_utf8_lossy(id_bytes).to_string());
        }
        None
    }

    /// The TCP port of ENR record if it is defined.
    #[must_use]
    pub fn tcp4(&self) -> Option<u16> {
        self.get_decodable("tcp").and_then(Result::ok)
    }

    /// The IPv6-specific TCP port of ENR record if it is defined.
    #[must_use]
    pub fn tcp6(&self) -> Option<u16> {
        self.get_decodable("tcp6").and_then(Result::ok)
    }

    /// The UDP port of ENR record if it is defined.
    #[must_use]
    pub fn udp4(&self) -> Option<u16> {
        self.get_decodable("udp").and_then(Result::ok)
    }

    /// The IPv6-specific UDP port of ENR record if it is defined.
    #[must_use]
    pub fn udp6(&self) -> Option<u16> {
        self.get_decodable("udp6").and_then(Result::ok)
    }

    /// Provides a socket (based on the UDP port), if the IPv4 and UDP fields are specified.
    #[must_use]
    pub fn udp4_socket(&self) -> Option<SocketAddrV4> {
        if let Some(ip) = self.ip4() {
            if let Some(udp) = self.udp4() {
                return Some(SocketAddrV4::new(ip, udp));
            }
        }
        None
    }

    /// Provides a socket (based on the UDP port), if the IPv6 and UDP fields are specified.
    #[must_use]
    pub fn udp6_socket(&self) -> Option<SocketAddrV6> {
        if let Some(ip6) = self.ip6() {
            if let Some(udp6) = self.udp6() {
                return Some(SocketAddrV6::new(ip6, udp6, 0, 0));
            }
        }
        None
    }

    /// Provides a socket (based on the TCP port), if the IP and TCP fields are specified.
    #[must_use]
    pub fn tcp4_socket(&self) -> Option<SocketAddrV4> {
        if let Some(ip) = self.ip4() {
            if let Some(tcp) = self.tcp4() {
                return Some(SocketAddrV4::new(ip, tcp));
            }
        }
        None
    }

    /// Provides a socket (based on the TCP port), if the IPv6 and TCP6 fields are specified.
    #[must_use]
    pub fn tcp6_socket(&self) -> Option<SocketAddrV6> {
        if let Some(ip6) = self.ip6() {
            if let Some(tcp6) = self.tcp6() {
                return Some(SocketAddrV6::new(ip6, tcp6, 0, 0));
            }
        }
        None
    }

    /// The signature of the ENR record.
    #[must_use]
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }

    /// Returns the public key of the ENR record.
    /// # Panics
    ///
    /// Will panic if the public key is not supported.
    #[must_use]
    pub fn public_key(&self) -> K::PublicKey {
        K::enr_to_public(&self.content).expect("ENR's can only be created with supported keys")
    }

    /// Verify the signature of the ENR record.
    #[must_use]
    pub fn verify(&self) -> bool {
        let pubkey = self.public_key();
        match self.id() {
            Some(ref id) if id == "v4" => pubkey.verify_v4(&self.rlp_content(), &self.signature),
            // unsupported identity schemes
            _ => false,
        }
    }

    /// Provides the URL-safe base64 encoded "text" version of the ENR prefixed by "enr:".
    #[must_use]
    pub fn to_base64(&self) -> String {
        let hex = URL_SAFE_NO_PAD.encode(&rlp::encode(self));
        format!("enr:{hex}")
    }

    /// Returns the current size of the ENR.
    #[must_use]
    pub fn size(&self) -> usize {
        rlp::encode(self).len()
    }

    // Setters //

    /// Allows setting the sequence number to an arbitrary value.
    pub fn set_seq(&mut self, seq: u64, key: &K) -> Result<(), EnrError> {
        self.seq = seq;

        // sign the record
        self.sign(key)?;

        // update the node id
        self.node_id = NodeId::from(key.public());

        // check the size of the record
        if self.size() > MAX_ENR_SIZE {
            return Err(EnrError::ExceedsMaxSize);
        }

        Ok(())
    }

    /// Adds or modifies a key/value to the ENR record. A `EnrKey` is required to re-sign the record once
    /// modified.
    ///
    /// Returns the previous value as rlp encoded bytes in the record if it exists.
    pub fn insert<T: Encodable>(
        &mut self,
        key: impl AsRef<[u8]>,
        value: &T,
        enr_key: &K,
    ) -> Result<Option<Bytes>, EnrError> {
        self.insert_raw_rlp(key, rlp::encode(value).freeze(), enr_key)
    }

    /// Adds or modifies a key/value to the ENR record. A `EnrKey` is required to re-sign the record once
    /// modified. The value here is interpreted as raw RLP data.
    ///
    /// Returns the previous value as rlp encoded bytes in the record if it exists.
    pub fn insert_raw_rlp(
        &mut self,
        key: impl AsRef<[u8]>,
        value: Bytes,
        enr_key: &K,
    ) -> Result<Option<Bytes>, EnrError> {
        check_spec_reserved_keys(key.as_ref(), &value)?;

        let previous_value = self.content.insert(key.as_ref().to_vec(), value);
        // add the new public key
        let public_key = enr_key.public();
        let previous_key = self.content.insert(
            public_key.enr_key(),
            rlp::encode(&public_key.encode().as_ref()).freeze(),
        );

        // check the size of the record
        if self.size() > MAX_ENR_SIZE {
            // if the size of the record is too large, revert and error
            // revert the public key
            if let Some(key) = previous_key {
                self.content.insert(public_key.enr_key(), key);
            } else {
                self.content.remove(&public_key.enr_key());
            }
            // revert the content
            if let Some(prev_value) = previous_value {
                self.content.insert(key.as_ref().to_vec(), prev_value);
            } else {
                self.content.remove(key.as_ref());
            }
            return Err(EnrError::ExceedsMaxSize);
        }
        // increment the sequence number
        self.seq = self
            .seq
            .checked_add(1)
            .ok_or(EnrError::SequenceNumberTooHigh)?;

        // sign the record
        self.sign(enr_key)?;

        // update the node id
        self.node_id = NodeId::from(enr_key.public());

        if self.size() > MAX_ENR_SIZE {
            // in case the signature size changes, inform the user the size has exceeded the maximum
            return Err(EnrError::ExceedsMaxSize);
        }

        Ok(previous_value)
    }

    /// Sets the `ip` field of the ENR. Returns any pre-existing IP address in the record.
    pub fn set_ip(&mut self, ip: IpAddr, key: &K) -> Result<Option<IpAddr>, EnrError> {
        match ip {
            IpAddr::V4(addr) => {
                let prev_value = self.insert("ip", &addr.octets().as_ref(), key)?;
                if let Some(bytes) = prev_value {
                    if bytes.len() == 4 {
                        let mut v = [0_u8; 4];
                        v.copy_from_slice(&bytes);
                        return Ok(Some(IpAddr::V4(Ipv4Addr::from(v))));
                    }
                }
            }
            IpAddr::V6(addr) => {
                let prev_value = self.insert("ip6", &addr.octets().as_ref(), key)?;
                if let Some(bytes) = prev_value {
                    if bytes.len() == 16 {
                        let mut v = [0_u8; 16];
                        v.copy_from_slice(&bytes);
                        return Ok(Some(IpAddr::V6(Ipv6Addr::from(v))));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Sets the `udp` field of the ENR. Returns any pre-existing UDP port in the record.
    pub fn set_udp4(&mut self, udp: u16, key: &K) -> Result<Option<u16>, EnrError> {
        if let Some(udp_bytes) = self.insert("udp", &udp, key)? {
            return Ok(rlp::decode(&udp_bytes).ok());
        }
        Ok(None)
    }

    /// Sets the `udp6` field of the ENR. Returns any pre-existing UDP port in the record.
    pub fn set_udp6(&mut self, udp: u16, key: &K) -> Result<Option<u16>, EnrError> {
        if let Some(udp_bytes) = self.insert("udp6", &udp, key)? {
            return Ok(rlp::decode(&udp_bytes).ok());
        }
        Ok(None)
    }

    /// Sets the `tcp` field of the ENR. Returns any pre-existing tcp port in the record.
    pub fn set_tcp4(&mut self, tcp: u16, key: &K) -> Result<Option<u16>, EnrError> {
        if let Some(tcp_bytes) = self.insert("tcp", &tcp, key)? {
            return Ok(rlp::decode(&tcp_bytes).ok());
        }
        Ok(None)
    }

    /// Sets the `tcp6` field of the ENR. Returns any pre-existing tcp6 port in the record.
    pub fn set_tcp6(&mut self, tcp: u16, key: &K) -> Result<Option<u16>, EnrError> {
        if let Some(tcp_bytes) = self.insert("tcp6", &tcp, key)? {
            return Ok(rlp::decode(&tcp_bytes).ok());
        }
        Ok(None)
    }

    /// Sets the IP and UDP port in a single update with a single increment in sequence number.
    pub fn set_udp_socket(&mut self, socket: SocketAddr, key: &K) -> Result<(), EnrError> {
        self.set_socket(socket, key, false)
    }

    /// Sets the IP and TCP port in a single update with a single increment in sequence number.
    pub fn set_tcp_socket(&mut self, socket: SocketAddr, key: &K) -> Result<(), EnrError> {
        self.set_socket(socket, key, true)
    }

    /// Helper function for `set_tcp_socket()` and `set_udp_socket`.
    fn set_socket(&mut self, socket: SocketAddr, key: &K, is_tcp: bool) -> Result<(), EnrError> {
        let (port_string, port_v6_string): (Key, Key) = if is_tcp {
            ("tcp".into(), "tcp6".into())
        } else {
            ("udp".into(), "udp6".into())
        };

        let (prev_ip, prev_port) = match socket.ip() {
            IpAddr::V4(addr) => (
                self.content.insert(
                    "ip".into(),
                    rlp::encode(&(&addr.octets() as &[u8])).freeze(),
                ),
                self.content
                    .insert(port_string.clone(), rlp::encode(&socket.port()).freeze()),
            ),
            IpAddr::V6(addr) => (
                self.content.insert(
                    "ip6".into(),
                    rlp::encode(&(&addr.octets() as &[u8])).freeze(),
                ),
                self.content
                    .insert(port_v6_string.clone(), rlp::encode(&socket.port()).freeze()),
            ),
        };

        let public_key = key.public();
        let previous_key = self.content.insert(
            public_key.enr_key(),
            rlp::encode(&public_key.encode().as_ref()).freeze(),
        );

        // check the size and revert on failure
        if self.size() > MAX_ENR_SIZE {
            // if the size of the record is too large, revert and error
            // revert the public key
            if let Some(key) = previous_key {
                self.content.insert(public_key.enr_key(), key);
            } else {
                self.content.remove(&public_key.enr_key());
            }
            // revert the content
            match socket.ip() {
                IpAddr::V4(_) => {
                    if let Some(ip) = prev_ip {
                        self.content.insert("ip".into(), ip);
                    } else {
                        self.content.remove(b"ip".as_ref());
                    }
                    if let Some(udp) = prev_port {
                        self.content.insert(port_string, udp);
                    } else {
                        self.content.remove(&port_string);
                    }
                }
                IpAddr::V6(_) => {
                    if let Some(ip) = prev_ip {
                        self.content.insert("ip6".into(), ip);
                    } else {
                        self.content.remove(b"ip6".as_ref());
                    }
                    if let Some(udp) = prev_port {
                        self.content.insert(port_v6_string, udp);
                    } else {
                        self.content.remove(&port_v6_string);
                    }
                }
            }
            return Err(EnrError::ExceedsMaxSize);
        }

        // increment the sequence number
        self.seq = self
            .seq
            .checked_add(1)
            .ok_or(EnrError::SequenceNumberTooHigh)?;

        // sign the record
        self.sign(key)?;

        // update the node id
        self.node_id = NodeId::from(key.public());

        Ok(())
    }

    /// Removes key/value mappings and adds or overwrites key/value mappings to the ENR record as
    /// one sequence number update. An `EnrKey` is required to re-sign the record once modified.
    /// Reverts whole ENR record on error.
    ///
    /// Returns the previous values as rlp encoded bytes if they exist for the removed and added/
    /// overwritten keys.
    pub fn remove_insert<'a>(
        &mut self,
        remove_keys: impl Iterator<Item = impl AsRef<[u8]>>,
        insert_key_values: impl Iterator<Item = (impl AsRef<[u8]>, &'a [u8])>,
        enr_key: &K,
    ) -> Result<(PreviousRlpEncodedValues, PreviousRlpEncodedValues), EnrError> {
        let enr_backup = self.clone();

        let mut removed = Vec::new();
        for key in remove_keys {
            removed.push(self.content.remove(key.as_ref()));
        }

        // add the new public key
        let public_key = enr_key.public();
        self.content.insert(
            public_key.enr_key(),
            rlp::encode(&public_key.encode().as_ref()).freeze(),
        );

        let mut inserted = Vec::new();
        for (key, value) in insert_key_values {
            // currently only support "v4" identity schemes
            if key.as_ref() == b"id" && value != b"v4" {
                *self = enr_backup;
                return Err(EnrError::UnsupportedIdentityScheme);
            }

            let value = rlp::encode(&(value)).freeze();
            // Prevent inserting invalid RLP integers
            if is_keyof_u16(key.as_ref()) {
                rlp::decode::<u16>(&value)
                    .map_err(|err| EnrError::InvalidRlpData(err.to_string()))?;
            }

            inserted.push(self.content.insert(key.as_ref().to_vec(), value));
        }

        // increment the sequence number
        self.seq = self
            .seq
            .checked_add(1)
            .ok_or(EnrError::SequenceNumberTooHigh)?;

        // sign the record
        self.sign(enr_key)?;

        // update the node id
        self.node_id = NodeId::from(enr_key.public());

        if self.size() > MAX_ENR_SIZE {
            // in case the signature size changes, inform the user the size has exceeded the
            // maximum
            *self = enr_backup;
            return Err(EnrError::ExceedsMaxSize);
        }

        Ok((removed, inserted))
    }

    /// Sets a new public key for the record.
    pub fn set_public_key(&mut self, public_key: &K::PublicKey, key: &K) -> Result<(), EnrError> {
        self.insert(&public_key.enr_key(), &public_key.encode().as_ref(), key)
            .map(|_| {})
    }

    /// Returns wether the node can be reached over UDP or not.
    #[must_use]
    pub fn is_udp_reachable(&self) -> bool {
        self.udp4_socket().is_some() || self.udp6_socket().is_some()
    }

    /// Returns wether the node can be reached over TCP or not.
    #[must_use]
    pub fn is_tcp_reachable(&self) -> bool {
        self.tcp4_socket().is_some() || self.tcp6_socket().is_some()
    }

    // Private Functions //

    /// Evaluates the RLP-encoding of the content of the ENR record.
    fn rlp_content(&self) -> BytesMut {
        let mut stream = RlpStream::new_with_buffer(BytesMut::with_capacity(MAX_ENR_SIZE));
        stream.begin_list(self.content.len() * 2 + 1);
        stream.append(&self.seq);
        for (k, v) in &self.content {
            // Keys are bytes
            stream.append(k);
            // Values are raw RLP encoded data
            stream.append_raw(v, 1);
        }
        stream.out()
    }

    /// Signs the ENR record based on the identity scheme. Currently only "v4" is supported.
    fn sign(&mut self, key: &K) -> Result<(), EnrError> {
        self.signature = {
            match self.id() {
                Some(ref id) if id == "v4" => key
                    .sign_v4(&self.rlp_content())
                    .map_err(|_| EnrError::SigningError)?,
                // other identity schemes are unsupported
                _ => return Err(EnrError::SigningError),
            }
        };
        Ok(())
    }
}

// traits //

impl<K: EnrKey> Clone for Enr<K> {
    fn clone(&self) -> Self {
        Self {
            seq: self.seq,
            node_id: self.node_id,
            content: self.content.clone(),
            signature: self.signature.clone(),
            phantom: self.phantom,
        }
    }
}

impl<K: EnrKey> std::cmp::Eq for Enr<K> {}

impl<K: EnrKey> PartialEq for Enr<K> {
    fn eq(&self, other: &Self) -> bool {
        self.seq == other.seq && self.node_id == other.node_id && self.signature == other.signature
    }
}

impl<K: EnrKey> Hash for Enr<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.seq.hash(state);
        self.node_id.hash(state);
        // since the struct should always have a valid signature, we can hash the signature
        // directly, rather than hashing the content.
        self.signature.hash(state);
    }
}

impl<K: EnrKey> std::fmt::Display for Enr<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_base64())
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl<K: EnrKey> std::fmt::Debug for Enr<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        struct OtherPairs<'a>(&'a BTreeMap<Key, Bytes>);

        impl<'a> std::fmt::Debug for OtherPairs<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_list()
                    .entries(
                        self.0
                            .iter()
                            .filter(|(key, _)| {
                                // skip all pairs already covered as fields
                                !["id", "ip", "ip6", "udp", "udp6", "tcp", "tcp6"]
                                    .iter()
                                    .any(|k| k.as_bytes() == key.as_slice())
                            })
                            .map(|(key, val)| (String::from_utf8_lossy(key), hex::encode(val))),
                    )
                    .finish()
            }
        }

        f.debug_struct("Enr")
            .field("id", &self.id())
            .field("seq", &self.seq())
            .field("NodeId", &self.node_id())
            .field("signature", &hex::encode(&self.signature))
            .field("IpV4 UDP Socket", &self.udp4_socket())
            .field("IpV6 UDP Socket", &self.udp6_socket())
            .field("IpV4 TCP Socket", &self.tcp4_socket())
            .field("IpV6 TCP Socket", &self.tcp6_socket())
            .field("Other Pairs", &OtherPairs(&self.content))
            .finish()
    }
}

/// Convert a URL-SAFE base64 encoded ENR into an ENR.
impl<K: EnrKey> FromStr for Enr<K> {
    type Err = String;

    fn from_str(base64_string: &str) -> Result<Self, Self::Err> {
        if base64_string.len() < 4 {
            return Err("Invalid ENR string".to_string());
        }
        // support both enr prefix and not
        let mut decode_string = base64_string;
        if base64_string.starts_with("enr:") {
            decode_string = decode_string
                .get(4..)
                .ok_or_else(|| "Invalid ENR string".to_string())?;
        }
        let bytes = URL_SAFE_NO_PAD
            .decode(decode_string)
            .map_err(|e| format!("Invalid base64 encoding: {e:?}"))?;
        if bytes.len() > MAX_ENR_SIZE {
            return Err("enr exceeds max size".to_string());
        }
        rlp::decode(&bytes).map_err(|e| format!("Invalid ENR: {e:?}"))
    }
}

#[cfg(feature = "serde")]
impl<K: EnrKey> Serialize for Enr<K> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_base64())
    }
}

#[cfg(feature = "serde")]
impl<'de, K: EnrKey> Deserialize<'de> for Enr<K> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Self::from_str(&s).map_err(D::Error::custom)
    }
}

impl<K: EnrKey> rlp::Encodable for Enr<K> {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(self.content.len() * 2 + 2);
        s.append(&self.signature);
        s.append(&self.seq);
        // must use rlp_content to preserve ordering.
        for (k, v) in &self.content {
            // Keys are byte data
            s.append(k);
            // Values are raw RLP encoded data
            s.append_raw(v, 1);
        }
    }
}

impl<K: EnrKey> rlp::Decodable for Enr<K> {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        if !rlp.is_list() {
            debug!("Failed to decode ENR. Not an RLP list: {}", rlp);
            return Err(DecoderError::RlpExpectedToBeList);
        }

        // verify there is no extra data
        let payload_info = rlp.payload_info()?;
        if rlp.as_raw().len() != payload_info.header_len + payload_info.value_len {
            return Err(DecoderError::RlpInconsistentLengthAndData);
        }

        let mut rlp_iter = rlp.iter();

        if rlp_iter.len() == 0 || rlp_iter.len() % 2 != 0 {
            debug!("Failed to decode ENR. List size is not a multiple of 2.");
            return Err(DecoderError::Custom("List not a multiple of two"));
        }

        let signature = rlp_iter
            .next()
            .ok_or(DecoderError::Custom("List is empty"))?
            .data()?;
        let seq = rlp_iter
            .next()
            .ok_or(DecoderError::Custom("List has only one item"))?
            .as_val()?;

        let mut content = BTreeMap::new();
        let mut prev: Option<&[u8]> = None;
        while let Some(key) = rlp_iter.next() {
            let key = key.data()?;
            let item = rlp_iter
                .next()
                .ok_or(DecoderError::Custom("List not a multiple of 2"))?;

            // Sanitize the data
            if is_keyof_u16(key) {
                item.as_val::<u16>()?;
            } else {
                item.data()?;
            }
            let value = item.as_raw();

            if prev.is_some() && prev >= Some(key) {
                return Err(DecoderError::Custom("Unsorted keys"));
            }
            prev = Some(key);
            content.insert(key.to_vec(), Bytes::copy_from_slice(value));
        }

        // verify we know the signature type
        let public_key = K::enr_to_public(&content)?;

        // calculate the node id
        let node_id = NodeId::from(public_key);

        let enr = Self {
            seq,
            node_id,
            signature: signature.into(),
            content,
            phantom: PhantomData,
        };

        // verify the signature before returning
        // if the public key is of an unknown type, this will fail.
        // An ENR record will always have a valid public-key and therefore node-id
        if !enr.verify() {
            return Err(DecoderError::Custom("Invalid Signature"));
        }
        Ok(enr)
    }
}

/// Owning iterator over all key/value pairs in the ENR.
pub struct EnrIntoIter {
    inner: <BTreeMap<Key, Bytes> as IntoIterator>::IntoIter,
}

impl Iterator for EnrIntoIter {
    type Item = (Key, Bytes);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<K: EnrKey> IntoIterator for Enr<K> {
    type Item = (Key, Bytes);

    type IntoIter = EnrIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        EnrIntoIter {
            inner: self.content.into_iter(),
        }
    }
}

pub(crate) fn digest(b: &[u8]) -> [u8; 32] {
    let mut output = [0_u8; 32];
    output.copy_from_slice(&Keccak256::digest(b));
    output
}

const fn is_keyof_u16(key: &[u8]) -> bool {
    matches!(key, b"tcp" | b"tcp6" | b"udp" | b"udp6")
}

fn check_spec_reserved_keys(key: &[u8], value: &[u8]) -> Result<(), EnrError> {
    match key {
        b"tcp" | b"tcp6" | b"udp" | b"udp6" => {
            rlp::decode::<u16>(value).map_err(|err| EnrError::InvalidRlpData(err.to_string()))?;
        }
        b"id" => {
            let id_bytes = rlp::decode::<Vec<u8>>(value)
                .map_err(|err| EnrError::InvalidRlpData(err.to_string()))?;
            if id_bytes != b"v4" {
                return Err(EnrError::UnsupportedIdentityScheme);
            }
        }
        b"ip" => {
            let ip4_bytes = rlp::decode::<Vec<u8>>(value)
                .map_err(|err| EnrError::InvalidRlpData(err.to_string()))?;
            if ip4_bytes.len() != 4 {
                return Err(EnrError::InvalidRlpData("Invalid Ipv4 size".to_string()));
            }
        }
        b"ip6" => {
            let ip6_bytes = rlp::decode::<Vec<u8>>(value)
                .map_err(|err| EnrError::InvalidRlpData(err.to_string()))?;
            if ip6_bytes.len() != 16 {
                return Err(EnrError::InvalidRlpData("Invalid Ipv6 size".to_string()));
            }
        }
        b"secp256k1" => {
            rlp::decode::<Enr<k256::ecdsa::SigningKey>>(value)
                .map_err(|err| EnrError::InvalidRlpData(err.to_string()))?;
        }
        _ => return Ok(()),
    };
    Ok(())
}

#[cfg(test)]
#[cfg(feature = "k256")]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::net::Ipv4Addr;

    type DefaultEnr = Enr<k256::ecdsa::SigningKey>;

    #[cfg(feature = "k256")]
    #[test]
    fn test_vector_k256() {
        let valid_record = hex::decode("f884b8407098ad865b00a582051940cb9cf36836572411a47278783077011599ed5cd16b76f2635f4e234738f30813a89eb9137e3e3df5266e3a1f11df72ecf1145ccb9c01826964827634826970847f00000189736563703235366b31a103ca634cae0d49acb401d8a4c6b6fe8c55b70d115bf400769cc1400f3258cd31388375647082765f").unwrap();
        let signature = hex::decode("7098ad865b00a582051940cb9cf36836572411a47278783077011599ed5cd16b76f2635f4e234738f30813a89eb9137e3e3df5266e3a1f11df72ecf1145ccb9c").unwrap();
        let expected_pubkey =
            hex::decode("03ca634cae0d49acb401d8a4c6b6fe8c55b70d115bf400769cc1400f3258cd3138")
                .unwrap();

        let enr = rlp::decode::<DefaultEnr>(&valid_record).unwrap();

        let pubkey = enr.public_key().encode();

        assert_eq!(enr.ip4(), Some(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(enr.id(), Some(String::from("v4")));
        assert_eq!(enr.udp4(), Some(30303));
        assert_eq!(enr.tcp4(), None);
        assert_eq!(enr.signature(), &signature[..]);
        assert_eq!(pubkey.to_vec(), expected_pubkey);
        assert!(enr.verify());
    }

    #[cfg(feature = "k256")]
    #[test]
    fn test_vector_2() {
        let text = "enr:-IS4QHCYrYZbAKWCBRlAy5zzaDZXJBGkcnh4MHcBFZntXNFrdvJjX04jRzjzCBOonrkTfj499SZuOh8R33Ls8RRcy5wBgmlkgnY0gmlwhH8AAAGJc2VjcDI1NmsxoQPKY0yuDUmstAHYpMa2_oxVtw0RW_QAdpzBQA8yWM0xOIN1ZHCCdl8";
        let signature = hex::decode("7098ad865b00a582051940cb9cf36836572411a47278783077011599ed5cd16b76f2635f4e234738f30813a89eb9137e3e3df5266e3a1f11df72ecf1145ccb9c").unwrap();
        let expected_pubkey =
            hex::decode("03ca634cae0d49acb401d8a4c6b6fe8c55b70d115bf400769cc1400f3258cd3138")
                .unwrap();
        let expected_node_id =
            hex::decode("a448f24c6d18e575453db13171562b71999873db5b286df957af199ec94617f7")
                .unwrap();

        let enr = text.parse::<DefaultEnr>().unwrap();
        let pubkey = enr.public_key().encode();
        assert_eq!(enr.ip4(), Some(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(enr.ip6(), None);
        assert_eq!(enr.id(), Some(String::from("v4")));
        assert_eq!(enr.udp4(), Some(30303));
        assert_eq!(enr.udp6(), None);
        assert_eq!(enr.tcp4(), None);
        assert_eq!(enr.tcp6(), None);
        assert_eq!(enr.signature(), &signature[..]);
        assert_eq!(pubkey.to_vec(), expected_pubkey);
        assert_eq!(enr.node_id().raw().to_vec(), expected_node_id);

        assert!(enr.verify());
    }

    #[cfg(feature = "k256")]
    #[test]
    fn test_vector_2_k256() {
        let text = "enr:-IS4QHCYrYZbAKWCBRlAy5zzaDZXJBGkcnh4MHcBFZntXNFrdvJjX04jRzjzCBOonrkTfj499SZuOh8R33Ls8RRcy5wBgmlkgnY0gmlwhH8AAAGJc2VjcDI1NmsxoQPKY0yuDUmstAHYpMa2_oxVtw0RW_QAdpzBQA8yWM0xOIN1ZHCCdl8";
        let signature = hex::decode("7098ad865b00a582051940cb9cf36836572411a47278783077011599ed5cd16b76f2635f4e234738f30813a89eb9137e3e3df5266e3a1f11df72ecf1145ccb9c").unwrap();
        let expected_pubkey =
            hex::decode("03ca634cae0d49acb401d8a4c6b6fe8c55b70d115bf400769cc1400f3258cd3138")
                .unwrap();
        let expected_node_id =
            hex::decode("a448f24c6d18e575453db13171562b71999873db5b286df957af199ec94617f7")
                .unwrap();

        let enr = text.parse::<Enr<k256::ecdsa::SigningKey>>().unwrap();
        let pubkey = enr.public_key().encode();
        assert_eq!(enr.ip4(), Some(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(enr.ip6(), None);
        assert_eq!(enr.id(), Some(String::from("v4")));
        assert_eq!(enr.udp4(), Some(30303));
        assert_eq!(enr.udp6(), None);
        assert_eq!(enr.tcp4(), None);
        assert_eq!(enr.tcp6(), None);
        assert_eq!(enr.signature(), &signature[..]);
        assert_eq!(pubkey.to_vec(), expected_pubkey);
        assert_eq!(enr.node_id().raw().to_vec(), expected_node_id);

        assert!(enr.verify());
    }

    // the values in the content are rlp lists
    #[test]
    fn test_rlp_list_value() {
        let text = "enr:-Je4QH0uN2HkMRmscUp6yvyTOPGtOg9U6lCxBFvCGynyystnDNRJbfz5GhXXY2lcu9tsghMxRiYHoznBwG46GQ7dfm0og2V0aMfGhMvbiDiAgmlkgnY0gmlwhA6hJmuJc2VjcDI1NmsxoQJBP4kg9GNBurV3uVXgR72u1n-XIABibUZLT1WvJLKwvIN0Y3CCdyeDdWRwgncn";
        let signature = hex::decode("7d2e3761e43119ac714a7acafc9338f1ad3a0f54ea50b1045bc21b29f2cacb670cd4496dfcf91a15d763695cbbdb6c821331462607a339c1c06e3a190edd7e6d").unwrap();
        let expected_pubkey =
            hex::decode("02413f8920f46341bab577b955e047bdaed67f972000626d464b4f55af24b2b0bc")
                .unwrap();
        let enr = text.parse::<DefaultEnr>().unwrap();

        assert_eq!(enr.ip4(), Some(Ipv4Addr::new(14, 161, 38, 107)));
        assert_eq!(enr.id(), Some(String::from("v4")));
        assert_eq!(enr.udp4(), Some(30503));
        assert_eq!(enr.tcp4(), Some(30503));
        assert_eq!(enr.seq(), 40);
        assert_eq!(enr.signature(), &signature[..]);
        assert_eq!(enr.public_key().encode().to_vec(), expected_pubkey);

        assert!(enr.verify());
    }

    #[cfg(feature = "k256")]
    #[test]
    fn test_read_enr_base64url_decoding_enforce_no_pad_no_extra_trailingbits() {
        let test_data = [
            ("padded", "Invalid base64 encoding: InvalidPadding", "enr:-IS4QHCYrYZbAKWCBRlAy5zzaDZXJBGkcnh4MHcBFZntXNFrdvJjX04jRzjzCBOonrkTfj499SZuOh8R33Ls8RRcy5wBgmlkgnY0gmlwhH8AAAGJc2VjcDI1NmsxoQPKY0yuDUmstAHYpMa2_oxVtw0RW_QAdpzBQA8yWM0xOIN1ZHCCdl8="),
            ("extra trailing bits", "Invalid base64 encoding: InvalidLastSymbol(178, 57)", "enr:-IS4QHCYrYZbAKWCBRlAy5zzaDZXJBGkcnh4MHcBFZntXNFrdvJjX04jRzjzCBOonrkTfj499SZuOh8R33Ls8RRcy5wBgmlkgnY0gmlwhH8AAAGJc2VjcDI1NmsxoQPKY0yuDUmstAHYpMa2_oxVtw0RW_QAdpzBQA8yWM0xOIN1ZHCCdl9"),
        ];
        for (test_name, err, text) in test_data {
            assert_eq!(text.parse::<DefaultEnr>().unwrap_err(), err, "{test_name}",);
        }
    }

    #[cfg(feature = "k256")]
    #[test]
    fn test_read_enr_no_prefix() {
        let text = "-Iu4QM-YJF2RRpMcZkFiWzMf2kRd1A5F1GIekPa4Sfi_v0DCLTDBfOMTMMWJhhawr1YLUPb5008CpnBKrgjY3sstjfgCgmlkgnY0gmlwhH8AAAGJc2VjcDI1NmsxoQP8u1uyQFyJYuQUTyA1raXKhSw1HhhxNUQ2VE52LNHWMIN0Y3CCIyiDdWRwgiMo";
        text.parse::<DefaultEnr>().unwrap();
    }

    #[cfg(feature = "k256")]
    #[test]
    fn test_read_enr_prefix() {
        let text = "enr:-Iu4QM-YJF2RRpMcZkFiWzMf2kRd1A5F1GIekPa4Sfi_v0DCLTDBfOMTMMWJhhawr1YLUPb5008CpnBKrgjY3sstjfgCgmlkgnY0gmlwhH8AAAGJc2VjcDI1NmsxoQP8u1uyQFyJYuQUTyA1raXKhSw1HhhxNUQ2VE52LNHWMIN0Y3CCIyiDdWRwgiMo";
        text.parse::<DefaultEnr>().unwrap();
    }

    #[cfg(feature = "k256")]
    #[test]
    fn test_read_enr_reject_too_large_record() {
        // 300-byte rlp encoded content, record creation should succeed.
        let text = concat!("enr:-QEpuEDaLyrPP4gxBI9YL7QE9U1tZig_Nt8rue8bRIuYv_IMziFc8OEt3LQMwkwt6da-Z0Y8BaqkDalZbBq647UtV2ei",
                           "AYJpZIJ2NIJpcIR_AAABiXNlY3AyNTZrMaEDymNMrg1JrLQB2KTGtv6MVbcNEVv0AHacwUAPMljNMTiDdWRwgnZferiieHh4",
                           "eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4",
                           "eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4",
                           "eHh4eHh4eHh4eHh4eHh4");
        let mut record = text.parse::<DefaultEnr>().unwrap();
        // Ensures the size check when creating a record from string is
        // consistent with the internal ones, such as when updating a record
        // field.
        let key_data =
            hex::decode("b71c71a67e1177ad4e901695e1b4b9ee17ae16c6668d313eac2f96dbcda3f291")
                .unwrap();
        let key = k256::ecdsa::SigningKey::from_slice(&key_data).unwrap();
        assert!(record.set_udp4(record.udp4().unwrap(), &key).is_ok());

        // 301-byte rlp encoded content, record creation should fail.
        let text = concat!("enr:-QEquEBxABglcZbIGKJ8RHDCp2Ft59tdf61RhV3XXf2BKTlKE2XwzNfihH-46hKkANsXaGRwH8Dp7a3lTrKiv2FMMaFY",
                           "AYJpZIJ2NIJpcIR_AAABiXNlY3AyNTZrMaEDymNMrg1JrLQB2KTGtv6MVbcNEVv0AHacwUAPMljNMTiDdWRwgnZferijeHh4",
                           "eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4",
                           "eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4",
                           "eHh4eHh4eHh4eHh4eHh4eA");
        assert_eq!(
            text.parse::<DefaultEnr>().unwrap_err(),
            "enr exceeds max size"
        );
    }

    #[cfg(feature = "k256")]
    #[test]
    fn test_read_enr_rlp_decoding_reject_extra_data() {
        // Valid record
        let record_hex1 = concat!(
            "f884b8407098ad865b00a582051940cb9cf36836572411a47278783077011599",
            "ed5cd16b76f2635f4e234738f30813a89eb9137e3e3df5266e3a1f11df72ecf1",
            "145ccb9c01826964827634826970847f00000189736563703235366b31a103ca",
            "634cae0d49acb401d8a4c6b6fe8c55b70d115bf400769cc1400f3258cd313883",
            "75647082765f"
        );
        // Invalid record
        // Replaces prefix "f884" with "f883", items payload length in bytes: 0x84 -> 0x83
        let record_hex2 = concat!(
            "f883b8407098ad865b00a582051940cb9cf36836572411a47278783077011599",
            "ed5cd16b76f2635f4e234738f30813a89eb9137e3e3df5266e3a1f11df72ecf1",
            "145ccb9c01826964827634826970847f00000189736563703235366b31a103ca",
            "634cae0d49acb401d8a4c6b6fe8c55b70d115bf400769cc1400f3258cd313883",
            "75647082765f"
        );
        // Invalid record
        // Appends one byte 252 (0xfc), 1-byte extra data
        let record_hex3 = concat!(
            "f884b8407098ad865b00a582051940cb9cf36836572411a47278783077011599",
            "ed5cd16b76f2635f4e234738f30813a89eb9137e3e3df5266e3a1f11df72ecf1",
            "145ccb9c01826964827634826970847f00000189736563703235366b31a103ca",
            "634cae0d49acb401d8a4c6b6fe8c55b70d115bf400769cc1400f3258cd313883",
            "75647082765ffc"
        );

        let valid_record = hex::decode(record_hex1).unwrap();
        let expected_pubkey =
            hex::decode("03ca634cae0d49acb401d8a4c6b6fe8c55b70d115bf400769cc1400f3258cd3138")
                .unwrap();

        let enr = rlp::decode::<DefaultEnr>(&valid_record).unwrap();
        let pubkey = enr.public_key().encode();
        assert_eq!(pubkey.to_vec(), expected_pubkey);
        assert!(enr.verify());

        let invalid_record = hex::decode(record_hex2).unwrap();
        rlp::decode::<DefaultEnr>(&invalid_record).expect_err("should reject extra data");

        let invalid_record = hex::decode(record_hex3).unwrap();
        rlp::decode::<DefaultEnr>(&invalid_record).expect_err("should reject extra data");
    }

    /// Tests that RLP integers decoding rejects any item with leading zeroes.
    #[cfg(feature = "k256")]
    #[test]
    fn test_rlp_integer_decoding() {
        // Uses the example node from the ENR spec.
        //
        // We first replace "seq" 0x01 with 0x0001 for a leading zero byte,
        // and then construct the RLP.
        //
        // ```
        // seq = bytes.fromhex('0001')  # replaces 0x01
        // rlp_data = encode(
        //     [
        //         0x7098ad865b00a582051940cb9cf36836572411a47278783077011599ed5cd16b76f2635f4e234738f30813a89eb9137e3e3df5266e3a1f11df72ecf1145ccb9c,
        //         seq, 'id', 'v4', 'ip', 0x7f000001, 'secp256k1', bytes.fromhex(
        //         '03ca634cae0d49acb401d8a4c6b6fe8c55b70d115bf400769cc1400f3258cd3138'), 'udp', 0x765f])
        // textual_form = "enr:" + urlsafe_b64encode(rlp_data).decode('utf-8').rstrip('=')
        // print(textual_form)
        // ```
        let text = "enr:-Ia4QHCYrYZbAKWCBRlAy5zzaDZXJBGkcnh4MHcBFZntXNFrdvJjX04jRzjzCBOonrkTfj499SZuOh8R33Ls8RRcy5yCAAGCaWSCdjSCaXCEfwAAAYlzZWNwMjU2azGhA8pjTK4NSay0Adikxrb-jFW3DRFb9AB2nMFADzJYzTE4g3VkcIJ2Xw";
        assert_eq!(
            text.parse::<DefaultEnr>().unwrap_err(),
            "Invalid ENR: RlpInvalidIndirection"
        );
    }

    #[cfg(feature = "rust-secp256k1")]
    #[test]
    fn test_encode_decode_secp256k1() {
        let mut rng = secp256k1::rand::thread_rng();
        let key = secp256k1::SecretKey::new(&mut rng);
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let tcp = 3000;

        let enr = {
            let mut builder = EnrBuilder::new("v4");
            builder.ip4(ip);
            builder.tcp4(tcp);
            builder.build(&key).unwrap()
        };

        let encoded_enr = rlp::encode(&enr);

        let decoded_enr = rlp::decode::<Enr<secp256k1::SecretKey>>(&encoded_enr).unwrap();

        assert_eq!(decoded_enr.id(), Some("v4".into()));
        assert_eq!(decoded_enr.ip4(), Some(ip));
        assert_eq!(decoded_enr.tcp4(), Some(tcp));
        // Must compare encoding as the public key itself can be different
        assert_eq!(decoded_enr.public_key().encode(), key.public().encode());
        assert!(decoded_enr.verify());
    }

    #[cfg(feature = "rust-secp256k1")]
    #[test]
    fn test_secp256k1_sign_ecdsa_with_mock_noncedata() {
        // Uses the example record from the ENR spec.
        //
        // The feature "rust-secp256k1" creates ECDSA signatures with additional random data.
        // Under the unit testing environment, the mock value `MOCK_ECDSA_NONCE_ADDITIONAL_DATA`
        // is always used.
        //
        // The expected ENR textual form `expected_enr_base64` is constructed by a Python script:
        // ```
        // key = SigningKey.from_secret_exponent(
        //     0xb71c71a67e1177ad4e901695e1b4b9ee17ae16c6668d313eac2f96dbcda3f291, curve=SECP256k1)
        //
        // # Builds content RLP
        // rlp_data = encode([1, 'id', 'v4', 'ip', 0x7f000001, 'secp256k1', bytes.fromhex(
        //     '03ca634cae0d49acb401d8a4c6b6fe8c55b70d115bf400769cc1400f3258cd3138'), 'udp', 0x765f])
        // rlp_data_hash = keccak(rlp_data)
        //
        // # Signs the content RLP **with** the additional data.
        // additional_data = bytes.fromhex(
        //     'baaaaaadbaaaaaadbaaaaaadbaaaaaadbaaaaaadbaaaaaadbaaaaaadbaaaaaad')
        // content_signature = key.sign_digest_deterministic(rlp_data_hash, hashfunc=sha256,
        //                                                   sigencode=sigencode_string_canonize,
        //                                                   extra_entropy=additional_data)
        // rlp_with_signature = encode(
        //     [content_signature, 1, 'id', 'v4', 'ip', 0x7f000001, 'secp256k1', bytes.fromhex(
        //         '03ca634cae0d49acb401d8a4c6b6fe8c55b70d115bf400769cc1400f3258cd3138'), 'udp', 0x765f])
        // textual_form = "enr:" + urlsafe_b64encode(rlp_with_signature).decode('utf-8').rstrip('=')
        // ```
        let expected_enr_base64 = "enr:-IS4QLJYdRwxdy-AbzWC6wL9ooB6O6uvCvJsJ36rbJztiAs1JzPY0__YkgFzZwNUuNhm1BDN6c4-UVRCJP9bXNCmoDYBgmlkgnY0gmlwhH8AAAGJc2VjcDI1NmsxoQPKY0yuDUmstAHYpMa2_oxVtw0RW_QAdpzBQA8yWM0xOIN1ZHCCdl8";

        let key_data =
            hex::decode("b71c71a67e1177ad4e901695e1b4b9ee17ae16c6668d313eac2f96dbcda3f291")
                .unwrap();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let udp = 30303;

        let key = secp256k1::SecretKey::from_slice(&key_data).unwrap();
        let enr = EnrBuilder::new("v4").ip4(ip).udp4(udp).build(&key).unwrap();
        let enr_base64 = enr.to_base64();
        assert_eq!(enr_base64, expected_enr_base64);

        let enr = enr_base64.parse::<Enr<secp256k1::SecretKey>>().unwrap();
        assert!(enr.verify());
    }

    #[cfg(feature = "k256")]
    #[test]
    fn test_encode_decode_k256() {
        let key = k256::ecdsa::SigningKey::random(&mut rand::rngs::OsRng);
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let tcp = 3000;

        let enr = {
            let mut builder = EnrBuilder::new("v4");
            builder.ip(ip.into());
            builder.tcp4(tcp);
            builder.build(&key).unwrap()
        };

        let encoded_enr = rlp::encode(&enr);

        let decoded_enr = rlp::decode::<Enr<k256::ecdsa::SigningKey>>(&encoded_enr).unwrap();

        assert_eq!(decoded_enr.id(), Some("v4".into()));
        assert_eq!(decoded_enr.ip4(), Some(ip));
        assert_eq!(decoded_enr.tcp4(), Some(tcp));
        // Must compare encoding as the public key itself can be different
        assert_eq!(decoded_enr.public_key().encode(), key.public().encode());
        decoded_enr.public_key().encode_uncompressed();
        assert!(decoded_enr.verify());
    }

    #[cfg(all(feature = "ed25519", feature = "k256"))]
    #[test]
    fn test_encode_decode_ed25519() {
        let mut rng = rand::thread_rng();
        let key = ed25519_dalek::SigningKey::generate(&mut rng);
        let ip = Ipv4Addr::new(10, 0, 0, 1);
        let tcp = 30303;

        let enr = {
            let mut builder = EnrBuilder::new("v4");
            builder.ip4(ip);
            builder.tcp4(tcp);
            builder.build(&key).unwrap()
        };

        let encoded_enr = rlp::encode(&enr);
        let decoded_enr = rlp::decode::<Enr<CombinedKey>>(&encoded_enr).unwrap();

        assert_eq!(decoded_enr.id(), Some("v4".into()));
        assert_eq!(decoded_enr.ip4(), Some(ip));
        assert_eq!(decoded_enr.tcp4(), Some(tcp));
        assert_eq!(decoded_enr.public_key().encode(), key.public().encode());
        assert!(decoded_enr.verify());
    }

    #[test]
    fn test_add_key() {
        let mut rng = rand::thread_rng();
        let key = k256::ecdsa::SigningKey::random(&mut rng);
        let ip = Ipv4Addr::new(10, 0, 0, 1);
        let tcp = 30303;

        let mut enr = {
            let mut builder = EnrBuilder::new("v4");
            builder.ip(ip.into());
            builder.tcp4(tcp);
            builder.build(&key).unwrap()
        };

        enr.insert("random", &Vec::new(), &key).unwrap();
        assert!(enr.verify());
    }

    #[test]
    fn test_set_ip() {
        let mut rng = rand::thread_rng();
        let key = k256::ecdsa::SigningKey::random(&mut rng);
        let tcp = 30303;
        let ip = Ipv4Addr::new(10, 0, 0, 1);

        let mut enr = {
            let mut builder = EnrBuilder::new("v4");
            builder.tcp4(tcp);
            builder.build(&key).unwrap()
        };

        assert!(enr.set_ip(ip.into(), &key).is_ok());
        assert_eq!(enr.id(), Some("v4".into()));
        assert_eq!(enr.ip4(), Some(ip));
        assert_eq!(enr.tcp4(), Some(tcp));
        assert!(enr.verify());

        // Compare the encoding as the key itself can be different
        assert_eq!(enr.public_key().encode(), key.public().encode());
    }

    #[test]
    fn ip_mutation_static_node_id() {
        let mut rng = rand::thread_rng();
        let key = k256::ecdsa::SigningKey::random(&mut rng);
        let tcp = 30303;
        let udp = 30304;
        let ip = Ipv4Addr::new(10, 0, 0, 1);

        let mut enr = {
            let mut builder = EnrBuilder::new("v4");
            builder.ip(ip.into());
            builder.tcp4(tcp);
            builder.udp4(udp);
            builder.build(&key).unwrap()
        };

        let node_id = enr.node_id();

        enr.set_udp_socket("192.168.0.1:800".parse::<SocketAddr>().unwrap(), &key)
            .unwrap();
        assert_eq!(node_id, enr.node_id());
        assert_eq!(
            enr.udp4_socket(),
            "192.168.0.1:800".parse::<SocketAddrV4>().unwrap().into()
        );
    }

    #[cfg(all(feature = "ed25519", feature = "k256"))]
    #[test]
    fn combined_key_can_decode_all() {
        // generate a random secp256k1 key
        let key = k256::ecdsa::SigningKey::random(&mut rand::thread_rng());
        let ip = Ipv4Addr::new(192, 168, 0, 1);
        let enr_secp256k1 = EnrBuilder::new("v4")
            .ip(ip.into())
            .tcp4(8000)
            .build(&key)
            .unwrap();

        // encode to base64
        let base64_string_secp256k1 = enr_secp256k1.to_base64();

        // generate a random ed25519 key
        let key = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
        let enr_ed25519 = EnrBuilder::new("v4")
            .ip(ip.into())
            .tcp4(8000)
            .build(&key)
            .unwrap();

        // encode to base64
        let base64_string_ed25519 = enr_ed25519.to_base64();

        // decode base64 strings of varying key types
        // decode the secp256k1 with default Enr
        let _decoded_enr_secp256k1: DefaultEnr = base64_string_secp256k1.parse().unwrap();
        // decode ed25519 ENRs
        let _decoded_enr_ed25519: Enr<ed25519_dalek::SigningKey> =
            base64_string_ed25519.parse().unwrap();

        // use the combined key to be able to decode either
        let _decoded_enr: Enr<CombinedKey> = base64_string_secp256k1
            .parse()
            .expect("Can decode both secp");
        let _decoded_enr: Enr<CombinedKey> = base64_string_ed25519.parse().unwrap();
    }

    #[test]
    fn test_remove_insert() {
        let mut rng = rand::thread_rng();
        let key = k256::ecdsa::SigningKey::random(&mut rng);
        let tcp = 30303;
        let mut topics = Vec::new();
        let mut s = RlpStream::new();
        s.begin_list(2);
        s.append(&"lighthouse");
        s.append(&"eth_syncing");
        topics.extend_from_slice(&s.out().freeze());

        let mut enr = {
            let mut builder = EnrBuilder::new("v4");
            builder.tcp4(tcp);
            builder.build(&key).unwrap()
        };

        assert_eq!(enr.tcp4(), Some(tcp));
        assert_eq!(enr.get("topics"), None);

        let topics: &[u8] = &topics;

        let (removed, inserted) = enr
            .remove_insert(
                vec![b"tcp"].iter(),
                vec![(b"topics", topics)].into_iter(),
                &key,
            )
            .unwrap();

        assert_eq!(
            removed[0],
            Some(rlp::encode(&tcp.to_be_bytes().to_vec()).freeze())
        );
        assert_eq!(inserted[0], None);

        assert_eq!(enr.tcp4(), None);
        assert_eq!(enr.get("topics"), Some(topics));

        // Compare the encoding as the key itself can be different
        assert_eq!(enr.public_key().encode(), key.public().encode());
    }

    /// | n     | `rlp::encode(n.to_be_bytes())` | `rlp::encode::<u16>(n)` |
    /// | ----- | ------------------------------ | ----------------------- |
    /// | 0     | 0x820000                       | 0x80
    /// | 30    | 0x82001e                       | 0x1e
    /// | 255   | 0x8200ff                       | 0x81ff
    /// | 30303 | 0x82765f                       | 0x82765f
    const LOW_INT_PORTS: [u16; 4] = [0, 30, 255, 30303];

    #[test]
    fn test_low_integer_build() {
        let key = k256::ecdsa::SigningKey::random(&mut rand::thread_rng());

        for tcp in LOW_INT_PORTS {
            let enr = {
                let mut builder = EnrBuilder::new("v4");
                builder.tcp4(tcp);
                builder.build(&key).unwrap()
            };

            assert_tcp4(&enr, tcp);
        }
    }

    #[test]
    fn test_low_integer_set() {
        let key = k256::ecdsa::SigningKey::random(&mut rand::thread_rng());

        for tcp in LOW_INT_PORTS {
            let mut enr = EnrBuilder::new("v4").build(&key).unwrap();
            enr.set_tcp4(tcp, &key).unwrap();
            assert_tcp4(&enr, tcp);
        }
    }

    #[test]
    fn test_low_integer_set_socket() {
        let key = k256::ecdsa::SigningKey::random(&mut rand::thread_rng());
        let ipv4 = Ipv4Addr::new(127, 0, 0, 1);

        for tcp in LOW_INT_PORTS {
            let mut enr = EnrBuilder::new("v4").build(&key).unwrap();
            enr.set_socket(SocketAddr::V4(SocketAddrV4::new(ipv4, tcp)), &key, true)
                .unwrap();
            assert_tcp4(&enr, tcp);
        }
    }

    #[test]
    fn test_low_integer_insert() {
        let key = k256::ecdsa::SigningKey::random(&mut rand::thread_rng());

        for tcp in LOW_INT_PORTS {
            let mut enr = EnrBuilder::new("v4").build(&key).unwrap();

            let res = enr.insert(b"tcp", &tcp.to_be_bytes().as_ref(), &key);
            if u8::try_from(tcp).is_ok() {
                assert_eq!(res.unwrap_err().to_string(), "invalid rlp data");
            } else {
                res.unwrap(); // integers above 255 are encoded correctly
                assert_tcp4(&enr, tcp);
            }
        }
    }

    #[test]
    fn test_low_integer_remove_insert() {
        let key = k256::ecdsa::SigningKey::random(&mut rand::thread_rng());

        for tcp in LOW_INT_PORTS {
            let mut enr = EnrBuilder::new("v4").build(&key).unwrap();

            let res = enr.remove_insert(
                vec![b"none"].iter(),
                vec![(b"tcp".as_slice(), tcp.to_be_bytes().as_slice())].into_iter(),
                &key,
            );
            if u8::try_from(tcp).is_ok() {
                assert_eq!(res.unwrap_err().to_string(), "invalid rlp data");
            } else {
                res.unwrap(); // integers above 255 are encoded correctly
                assert_tcp4(&enr, tcp);
            }
        }
    }

    #[test]
    fn test_low_integer_bad_enr() {
        let vectors = vec![
            (0, "enr:-Hy4QDMsoimQl2Qb9CuIWlNjyt0C0DmZC4QpAsJzgUHowOq2Nph9UbAtZ_qS_8fl6SU-eSWrswHiLCoMUGQfjhl_GW0BgmlkgnY0iXNlY3AyNTZrMaECMoYV0PAXMueQz19FHpBO0jGBoLYCWhfSxGf5kQgk9KqDdGNwggAA"),
            (30, "enr:-Hy4QCCgTB9tAEJL1DFwTTtwd79xxQx2hvi5RX9vWvcdKqbpS3SDzHHBivpOgxE40HGt6P0NtCE5QKzOQ5fzBwepDfMBgmlkgnY0iXNlY3AyNTZrMaECMoYV0PAXMueQz19FHpBO0jGBoLYCWhfSxGf5kQgk9KqDdGNwggAe"),
            (255, "enr:-Hy4QOrU9C35gZyJigIi-u19sRP42eEjVEhzO-LnKXKM5VlDMZ45vnOIa3bqm15ap8pmLjq5kmRPzjuA0RUdzSsieqcBgmlkgnY0iXNlY3AyNTZrMaECMoYV0PAXMueQz19FHpBO0jGBoLYCWhfSxGf5kQgk9KqDdGNwggD_"),
            (30303, "enr:-Hy4QF_mn4BuM6hY4CuLH8xDQd7U8kVZe9fyNgRB1vjdToGWQsQhetRvsByoJCWGQ6kf2aiWC0le24lkp0IPIJkLSTUBgmlkgnY0iXNlY3AyNTZrMaECMoYV0PAXMueQz19FHpBO0jGBoLYCWhfSxGf5kQgk9KqDdGNwgnZf"),
        ];

        for (tcp, enr_str) in vectors {
            let res = DefaultEnr::from_str(enr_str);
            if u8::try_from(tcp).is_ok() {
                assert_eq!(
                    res.unwrap_err().to_string(),
                    "Invalid ENR: RlpInvalidIndirection"
                );
            } else {
                assert_tcp4(&res.unwrap(), tcp);
            }
        }
    }

    fn assert_tcp4(enr: &DefaultEnr, tcp: u16) {
        assert!(enr.verify());
        assert_eq!(enr.get_raw_rlp("tcp").unwrap(), rlp::encode(&tcp).to_vec());
        assert_eq!(enr.tcp4(), Some(tcp));
    }
}
