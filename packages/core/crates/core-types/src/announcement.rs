use core_crypto::{
    errors::CryptoError,
    keypairs::{Keypair, OffchainKeypair},
    types::{OffchainPublicKey, OffchainSignature},
};
use multiaddr::Multiaddr;
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};
use utils_log::debug;
use utils_types::{
    errors::GeneralError::{self, InvalidInput, NonSpecificError},
    primitives::Address,
    traits::{BinarySerializable, PeerIdLike, ToHex},
};

/// Holds the signed binding of the chain key and the packet key.
/// The signature
/// This is used to attest on-chain that node owns the corresponding packet key and links it with
/// the chain key.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct KeyBinding {
    pub chain_key: Address,
    pub packet_key: OffchainPublicKey,
    pub signature: OffchainSignature,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl KeyBinding {
    fn prepare_for_signing(chain_key: &Address, packet_key: &OffchainPublicKey) -> Box<[u8]> {
        let mut to_sign = Vec::with_capacity(70);
        to_sign.extend_from_slice(b"HOPR_KEY_BINDING");
        to_sign.extend_from_slice(&chain_key.to_bytes());
        to_sign.extend_from_slice(&packet_key.to_bytes());
        to_sign.into_boxed_slice()
    }

    /// Create and sign new key binding of the given chain key and packet key.
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(chain_key: Address, packet_key: &OffchainKeypair) -> Self {
        let to_sign = Self::prepare_for_signing(&chain_key, packet_key.public());
        Self {
            chain_key,
            packet_key: packet_key.public().clone(),
            signature: OffchainSignature::sign_message(&to_sign, packet_key),
        }
    }
}

impl KeyBinding {
    /// Re-construct binding from the chain key and packet key, while also verifying the given signature of the binding.
    /// Fails if the signature is not valid for the given entries.
    pub fn from_parts(
        chain_key: Address,
        packet_key: OffchainPublicKey,
        signature: OffchainSignature,
    ) -> Result<Self, GeneralError> {
        let to_verify = Self::prepare_for_signing(&chain_key, &packet_key);
        signature
            .verify_message(&to_verify, &packet_key)
            .then_some(Self {
                chain_key,
                packet_key,
                signature,
            })
            .ok_or(GeneralError::Other(CryptoError::SignatureVerification.into()))
    }
}

impl Display for KeyBinding {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyBinding")
            .field("ChainKey", &self.chain_key.to_hex())
            .field("PacketKey", &self.packet_key.to_hex())
            .field("Signature", &self.signature.to_hex())
            .finish()
    }
}

/// Structure containing data used for on-chain announcement.
/// That is the decapsulated multiaddress (with the /p2p/<peer id> suffix removed) and
/// optional `KeyBinding` (announcement can be done with key bindings or without)
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct AnnouncementData {
    multiaddress: Multiaddr,
    pub key_binding: Option<KeyBinding>,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl AnnouncementData {
    pub fn to_multiaddress_str(&self) -> String {
        self.multiaddress.to_string()
    }
}

impl AnnouncementData {
    /// Constructs structure from multiaddress and optionally also `KeyBinding`.
    /// The multiaddress must not be empty. It should be the external address of the node.
    /// It may contain a trailing PeerId (encapsulated multiaddr) or come without. If the
    /// peerId is present, it must match with the keybinding.
    pub fn new(multiaddress: &Multiaddr, key_binding: Option<KeyBinding>) -> Result<Self, GeneralError> {
        if let Some(ending) = multiaddress.protocol_stack().last() {
            if let Some(ref binding) = key_binding {
                if ending == "p2p" {
                    let expected: String = format!("/p2p/{}", binding.packet_key.to_peerid_str());
                    // We got a keybinding and we get a multiaddress with trailing PeerId, so
                    // fail if they don't match
                    if multiaddress.ends_with(
                        &Multiaddr::from_str(expected.as_str()).map_err(|e| NonSpecificError(e.to_string()))?,
                    ) {
                        let mut decapsulated = multiaddress.clone();
                        decapsulated.pop();
                        Ok(Self {
                            multiaddress: decapsulated,
                            key_binding,
                        })
                    } else {
                        return Err(NonSpecificError(format!(
                            "Received a multiaddr with a PeerId that doesn't match the keybinding, got {} but expected {}",
                            multiaddress, expected
                        )));
                    }
                } else {
                    Ok(Self {
                        multiaddress: multiaddress.to_owned(),
                        key_binding,
                    })
                }
            } else {
                Ok(Self {
                    multiaddress: multiaddress.to_owned(),
                    key_binding,
                })
            }
        } else {
            debug!("Received empty multiaddr");
            return Err(InvalidInput);
        }
    }
}

impl Display for AnnouncementData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnnouncementData")
            .field("Multiaddress", &self.multiaddress)
            .field("Keybinding", &self.key_binding)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::announcement::{AnnouncementData, KeyBinding};
    use core_crypto::keypairs::{Keypair, OffchainKeypair};
    use hex_literal::hex;
    use multiaddr::Multiaddr;
    use utils_types::{
        primitives::Address,
        traits::{BinarySerializable, PeerIdLike},
    };

    lazy_static::lazy_static! {
        static ref KEY_PAIR: OffchainKeypair = OffchainKeypair::from_secret(&hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d")).unwrap();
        static ref CHAIN_ADDR: Address = Address::from_bytes(&hex!("78392d47e3522219e2802e7d6c45ee84b5d5c185")).unwrap();
        static ref SECOND_KEY_PAIR: OffchainKeypair = OffchainKeypair::from_secret(&hex!("c24bd833704dd2abdae3933fcc9962c2ac404f84132224c474147382d4db2299")).unwrap();
    }

    #[test]
    fn test_key_binding() {
        let kb_1 = KeyBinding::new(*CHAIN_ADDR, &KEY_PAIR);
        let kb_2 = KeyBinding::from_parts(kb_1.chain_key.clone(), kb_1.packet_key.clone(), kb_1.signature.clone())
            .expect("should verify correctly");

        assert_eq!(kb_1, kb_2, "must be equal");
    }

    #[test]
    fn test_announcement() {
        let key_binding = KeyBinding::new(*CHAIN_ADDR, &KEY_PAIR);
        let peer_id = KEY_PAIR.public().to_peerid();

        for (ma_str, decapsulated_ma_str) in vec![
            (
                format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}"),
                format!("/ip4/127.0.0.1/tcp/10000"),
            ),
            (
                format!("/ip6/::1/tcp/10000/p2p/{peer_id}"),
                format!("/ip6/::1/tcp/10000"),
            ),
            (
                format!("/dns4/hoprnet.org/tcp/10000/p2p/{peer_id}"),
                format!("/dns4/hoprnet.org/tcp/10000"),
            ),
            (
                format!("/dns6/hoprnet.org/tcp/10000/p2p/{peer_id}"),
                format!("/dns6/hoprnet.org/tcp/10000"),
            ),
            (
                format!("/ip4/127.0.0.1/udp/10000/quic/p2p/{peer_id}"),
                format!("/ip4/127.0.0.1/udp/10000/quic"),
            ),
        ] {
            let maddr: Multiaddr = ma_str.parse().unwrap();

            let ad = AnnouncementData::new(&maddr, Some(key_binding.clone()))
                .expect("construction of announcement data should work");
            assert_eq!(decapsulated_ma_str, ad.to_multiaddress_str());
            assert_eq!(Some(key_binding.clone()), ad.key_binding);
        }
    }

    #[test]
    fn test_announcement_no_keybinding() {
        let maddr: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000").parse().unwrap();

        let ad = AnnouncementData::new(&maddr, None).expect("construction of announcement data should work");

        assert_eq!(None, ad.key_binding);
    }

    #[test]
    fn test_announcement_decapsulated_ma() {
        let key_binding = KeyBinding::new(*CHAIN_ADDR, &KEY_PAIR);
        let maddr: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000").parse().unwrap();

        let ad = AnnouncementData::new(&maddr, Some(key_binding.clone()))
            .expect("construction of announcement data should work");
        assert_eq!("/ip4/127.0.0.1/tcp/10000", ad.to_multiaddress_str());
        assert_eq!(Some(key_binding), ad.key_binding);
    }

    #[test]
    fn test_announcement_wrong_peerid() {
        let key_binding = KeyBinding::new(*CHAIN_ADDR, &KEY_PAIR);
        let peer_id = SECOND_KEY_PAIR.public().to_peerid();
        let maddr: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse().unwrap();

        assert!(AnnouncementData::new(&maddr, Some(key_binding.clone())).is_err());
    }
}
