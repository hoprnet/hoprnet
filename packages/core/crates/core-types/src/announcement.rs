use core_crypto::errors::CryptoError;
use core_crypto::keypairs::{Keypair, OffchainKeypair};
use core_crypto::types::{OffchainPublicKey, OffchainSignature};
use multiaddr::{Multiaddr, Protocol};
use std::fmt::{Display, Formatter};
use utils_types::errors::GeneralError;
use utils_types::errors::GeneralError::{InvalidInput, NonSpecificError};
use utils_types::primitives::Address;
use utils_types::traits::{BinarySerializable, PeerIdLike, ToHex};

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
    /// The multiaddress must not be empty and must end with `/p2p/<peer id>` to be considered valid.
    /// Also if `key_binding` is specified, the public key must match the peer id of the multiaddress.
    pub fn new(multiaddress: Multiaddr, key_binding: Option<KeyBinding>) -> Result<Self, GeneralError> {
        let mut decapsulated = multiaddress;
        match decapsulated.pop().ok_or(InvalidInput)? {
            Protocol::P2p(peer) => {
                if key_binding
                    .clone()
                    .is_some_and(|kb| !peer.eq(&kb.packet_key.to_peerid()))
                {
                    return Err(NonSpecificError(format!(
                        "decapsulated peer id {peer} does not match the key binding"
                    )));
                }
            }
            _ => return Err(InvalidInput),
        };

        Ok(Self {
            multiaddress: decapsulated,
            key_binding,
        })
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
    use multiaddr::Multiaddr;
    use utils_types::primitives::Address;
    use utils_types::traits::PeerIdLike;

    #[test]
    fn test_key_binding() {
        let kp = OffchainKeypair::random();

        let kb_1 = KeyBinding::new(Address::default(), &kp);
        let kb_2 = KeyBinding::from_parts(kb_1.chain_key.clone(), kb_1.packet_key.clone(), kb_1.signature.clone())
            .expect("should verify correctly");

        assert_eq!(kb_1, kb_2, "must be equal");
    }

    #[test]
    fn test_announcement_data() {
        let keypair = OffchainKeypair::random();
        let key_binding = KeyBinding::new(Address::default(), &keypair);
        let peer_id = keypair.public().to_peerid();

        let maddr: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse().unwrap();
        let ad = AnnouncementData::new(maddr, Some(key_binding.clone()))
            .expect("construction of announcement data should work");
        assert_eq!("/ip4/127.0.0.1/tcp/10000", ad.to_multiaddress_str());

        let maddr: Multiaddr = format!("/ip4/147.75.83.83/udp/4001/quic/p2p/{peer_id}")
            .parse()
            .unwrap();
        let ad = AnnouncementData::new(maddr, Some(key_binding.clone()))
            .expect("construction of announcement data should work with multiple protocols");
        assert_eq!("/ip4/147.75.83.83/udp/4001/quic", ad.to_multiaddress_str());

        let maddr: Multiaddr = "/ip4/127.0.0.1/udt/sctp/5678".parse().unwrap();
        AnnouncementData::new(maddr, Some(key_binding.clone())).expect_err("should not work without p2p protocol");

        let peer_id = OffchainKeypair::random().public().to_peerid();
        let maddr: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse().unwrap();
        AnnouncementData::new(maddr, Some(key_binding.clone())).expect_err("should not work with different peer ID");

        let maddr: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse().unwrap();
        AnnouncementData::new(maddr, None)
            .expect("should not work with different peer ID if no key binding was specified");
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::announcement::{AnnouncementData, KeyBinding};
    use core_crypto::types::{OffchainPublicKey, OffchainSignature};
    use multiaddr::Multiaddr;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::Address;
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    impl KeyBinding {
        #[wasm_bindgen(js_name = "from_parts")]
        pub fn _from_parts(
            chain_key: &Address,
            packet_key: &OffchainPublicKey,
            signature: &OffchainSignature,
        ) -> JsResult<KeyBinding> {
            ok_or_jserr!(Self::from_parts(
                chain_key.clone(),
                packet_key.clone(),
                signature.clone()
            ))
        }
    }

    #[wasm_bindgen]
    impl AnnouncementData {
        #[wasm_bindgen(constructor)]
        pub fn _new(multiaddress: String, key_binding: Option<KeyBinding>) -> JsResult<AnnouncementData> {
            let maddr = ok_or_jserr!(Multiaddr::try_from(multiaddress))?;
            ok_or_jserr!(Self::new(maddr, key_binding))
        }

        #[wasm_bindgen(js_name = "to_string")]
        pub fn _to_string(&self) -> String {
            self.to_string()
        }
    }
}
