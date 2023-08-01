use std::fmt::{Display, Formatter};
use multiaddr::{Multiaddr, Protocol};
use core_crypto::keypairs::{Keypair, OffchainKeypair};
use core_crypto::types::{OffchainPublicKey, OffchainSignature};
use utils_types::errors::GeneralError;
use utils_types::errors::GeneralError::InvalidInput;
use utils_types::traits::{BinarySerializable, PeerIdLike, ToHex};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct AnnouncementData {
    multiaddress: Multiaddr,
    pub packet_key: OffchainPublicKey,
    pub signature: OffchainSignature
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
impl AnnouncementData {
    pub fn to_multiaddress_str(&self) -> String {
        self.multiaddress.to_string()
    }
}

impl AnnouncementData {
    fn prepare_for_signing(decapsulated: &Multiaddr, packet_key: &OffchainPublicKey) -> Box<[u8]> {
        let mut to_sign = Vec::with_capacity(128);
        to_sign.extend_from_slice(b"HOPR_ONCHAIN_ANNOUNCEMENT");
        to_sign.extend_from_slice(decapsulated.as_ref());
        to_sign.extend_from_slice(&packet_key.to_bytes());
        to_sign.into_boxed_slice()
    }

    pub fn new(multiaddress: Multiaddr, packet_key: &OffchainKeypair) -> Result<Self, GeneralError> {
        let mut decapsulated = multiaddress.clone();
        match decapsulated.pop().ok_or(InvalidInput)? {
            Protocol::P2p(peer) => {
                if !peer.eq(&packet_key.public().to_peerid()) {
                    return Err(InvalidInput)
                }
            }
            _ => return Err(InvalidInput)
        };

        let to_sign = Self::prepare_for_signing(&decapsulated, packet_key.public());
        let signature = OffchainSignature::sign_message(&to_sign, &packet_key);
        Ok(Self {
            multiaddress: decapsulated,
            packet_key: packet_key.public().clone(),
            signature
        })
    }

    pub fn from_parts(multiaddress: Multiaddr, packet_key: OffchainPublicKey, signature: OffchainSignature) -> Result<Self, GeneralError> {
        if multiaddress.protocol_stack().last().ok_or(InvalidInput)? == "p2p" {
            return Err(InvalidInput)
        }

        let to_verify = Self::prepare_for_signing(&multiaddress, &packet_key);
        if !signature.verify_message(&to_verify, &packet_key) {
            return Err(InvalidInput)
        }

        Ok(Self { multiaddress, packet_key, signature })
    }
}

impl Display for AnnouncementData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnnouncementData")
            .field("Multiaddress", &self.multiaddress)
            .field("PacketKey", &self.packet_key.to_hex())
            .field("Signature", &self.signature.to_hex())
        .finish()
    }
}

#[cfg(test)]
mod tests {
    use multiaddr::Multiaddr;
    use core_crypto::keypairs::{Keypair, OffchainKeypair};
    use utils_types::traits::PeerIdLike;
    use crate::announcement::AnnouncementData;

    #[test]
    fn test_announcement_data() {
        let keypair = OffchainKeypair::random();
        let peer_id = keypair.public().to_peerid();

        let maddr: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse().unwrap();
        let ad = AnnouncementData::new(maddr, &keypair)
            .expect("construction of announcement data should work");
        assert_eq!("/ip4/127.0.0.1/tcp/10000", ad.to_multiaddress_str());

        let ad2 = AnnouncementData::from_parts(Multiaddr::try_from(ad.to_multiaddress_str()).unwrap(), keypair.public().clone(), ad.signature.clone())
            .expect("should be reconstructible");

        assert_eq!(ad, ad2, "should be equal when reconstructed");


        let maddr: Multiaddr = format!("/ip4/147.75.83.83/udp/4001/quic/p2p/{peer_id}").parse().unwrap();
        let ad = AnnouncementData::new(maddr, &keypair)
            .expect("construction of announcement data should work with multiple protocols");
        assert_eq!("/ip4/147.75.83.83/udp/4001/quic", ad.to_multiaddress_str());

        let maddr: Multiaddr = "/ip4/127.0.0.1/udt/sctp/5678".parse().unwrap();
        AnnouncementData::new(maddr, &keypair)
            .expect_err("should not work without p2p protocol");

        let peer_id = OffchainKeypair::random().public().to_peerid();
        let maddr: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse().unwrap();
        AnnouncementData::new(maddr, &keypair)
            .expect_err("should not work with different peer ID");

    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use multiaddr::Multiaddr;
    use wasm_bindgen::prelude::wasm_bindgen;
    use core_crypto::keypairs::OffchainKeypair;
    use core_crypto::types::{OffchainPublicKey, OffchainSignature};
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use crate::announcement::AnnouncementData;

    #[wasm_bindgen]
    impl AnnouncementData {
        #[wasm_bindgen(constructor)]
        pub fn _new(multiaddress: String, packet_key: &OffchainKeypair) -> JsResult<AnnouncementData> {
            let maddr = ok_or_jserr!(Multiaddr::try_from(multiaddress))?;
            ok_or_jserr!(Self::new(maddr, packet_key))
        }

        #[wasm_bindgen(js_name = "from_parts")]
        pub fn _from_parts(multiaddress: String, packet_key: &OffchainPublicKey, signature: &OffchainSignature) -> JsResult<AnnouncementData> {
            let maddr = ok_or_jserr!(Multiaddr::try_from(multiaddress))?;
            ok_or_jserr!(Self::from_parts(maddr, packet_key, signature))
        }
    }
}