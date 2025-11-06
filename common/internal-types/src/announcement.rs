use std::fmt::{Display, Formatter};

use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
use multiaddr::Multiaddr;
use tracing::debug;

/// Holds the signed binding of the chain key and the packet key.
///
/// The signature is done via the offchain key to bind it with the on-chain key. The structure
/// then makes it on-chain, making it effectively cross-signed with both keys (offchain and onchain).
/// This is used to attest on-chain that node owns the corresponding packet key and links it with
/// the chain key.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyBinding {
    pub chain_key: Address,
    pub packet_key: OffchainPublicKey,
    pub signature: OffchainSignature,
}

impl KeyBinding {
    const SIGNING_SIZE: usize = 16 + Address::SIZE + OffchainPublicKey::SIZE;

    fn prepare_for_signing(chain_key: &Address, packet_key: &OffchainPublicKey) -> [u8; Self::SIGNING_SIZE] {
        let mut to_sign = [0u8; Self::SIGNING_SIZE];
        to_sign[0..16].copy_from_slice(b"HOPR_KEY_BINDING");
        to_sign[16..36].copy_from_slice(chain_key.as_ref());
        to_sign[36..].copy_from_slice(packet_key.as_ref());
        to_sign
    }

    /// Create and sign new key binding of the given chain key and packet key.
    pub fn new(chain_key: Address, packet_key: &OffchainKeypair) -> Self {
        let to_sign = Self::prepare_for_signing(&chain_key, packet_key.public());
        Self {
            chain_key,
            packet_key: *packet_key.public(),
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
    ) -> crate::errors::Result<Self> {
        let to_verify = Self::prepare_for_signing(&chain_key, &packet_key);
        signature
            .verify_message(&to_verify, &packet_key)
            .then_some(Self {
                chain_key,
                packet_key,
                signature,
            })
            .ok_or(CryptoError::SignatureVerification.into())
    }
}

impl Display for KeyBinding {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "keybinding {} <-> {}", self.chain_key, self.packet_key)
    }
}

/// Decapsulates the multiaddress (= strips the /p2p/<peer_id> suffix).
/// If it is already decapsulated, the function is an identity.
pub fn decapsulate_multiaddress(multiaddr: Multiaddr) -> Multiaddr {
    multiaddr
        .into_iter()
        .take_while(|p| !matches!(p, multiaddr::Protocol::P2p(_)))
        .collect()
}

/// Structure containing data used for an on-chain announcement.
/// That is the decapsulated multiaddress (with the /p2p/{peer_id} suffix removed) and
/// optional `KeyBinding` (an announcement can be done with key bindings or without)
///
/// NOTE: This currently supports only announcing of a single multiaddress
#[derive(Clone, Debug, PartialEq)]
pub struct AnnouncementData {
    multiaddress: Option<Multiaddr>,
    pub key_binding: KeyBinding,
}

impl AnnouncementData {
    /// Constructs structure from multiaddress and optionally also `KeyBinding`.
    /// The multiaddress must not be empty. It should be the external address of the node.
    /// It may contain a trailing PeerId (encapsulated multiaddr) or come without. If the
    /// peerId is present, it must match with the keybinding.
    pub fn new(multiaddress: Option<Multiaddr>, key_binding: KeyBinding) -> Result<Self, GeneralError> {
        if let Some(ma) = &multiaddress {
            if ma.is_empty() {
                debug!("Received empty multiaddr");
                return Err(GeneralError::InvalidInput);
            }

            match ma.clone().with_p2p(key_binding.packet_key.into()) {
                Ok(mut valid_ma) => {
                    // Now decapsulate again, because we store decapsulated multiaddress only (without the
                    // /p2p/<peer_id> suffix)
                    valid_ma.pop();
                    Ok(Self {
                        multiaddress: Some(valid_ma),
                        key_binding,
                    })
                }
                Err(invalid_ma) => Err(GeneralError::NonSpecificError(format!(
                    "{invalid_ma} does not match the keybinding {} peer id",
                    key_binding.packet_key.to_peerid_str()
                ))),
            }
        } else {
            Ok(Self {
                multiaddress: None,
                key_binding,
            })
        }
    }

    /// Returns the multiaddress associated with this announcement.
    /// Note that the returned multiaddress is *always* decapsulated (= without the /p2p/<peer_id> suffix)
    pub fn multiaddress(&self) -> &Option<Multiaddr> {
        &self.multiaddress
    }
}

impl Display for AnnouncementData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ma = self.multiaddress.as_ref().map(ToString::to_string).unwrap_or_default(); // "" if None

        write!(f, "announcement of {ma} with {}", self.key_binding)
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use hopr_crypto_types::keypairs::{Keypair, OffchainKeypair};
    use hopr_primitive_types::primitives::Address;
    use multiaddr::Multiaddr;

    use crate::{
        announcement::{AnnouncementData, KeyBinding},
        prelude::decapsulate_multiaddress,
    };

    lazy_static::lazy_static! {
        static ref KEY_PAIR: OffchainKeypair = OffchainKeypair::from_secret(&hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d")).expect("lazy static keypair should be constructible");
        static ref CHAIN_ADDR: Address = Address::try_from(hex!("78392d47e3522219e2802e7d6c45ee84b5d5c185").as_ref()).expect("lazy static address should be constructible");
        static ref SECOND_KEY_PAIR: OffchainKeypair = OffchainKeypair::from_secret(&hex!("c24bd833704dd2abdae3933fcc9962c2ac404f84132224c474147382d4db2299")).expect("lazy static keypair should be constructible");
    }

    #[test]
    fn test_key_binding() -> anyhow::Result<()> {
        let kb_1 = KeyBinding::new(*CHAIN_ADDR, &KEY_PAIR);
        let kb_2 = KeyBinding::from_parts(kb_1.chain_key, kb_1.packet_key, kb_1.signature.clone())?;

        assert_eq!(kb_1, kb_2, "must be equal");

        Ok(())
    }

    #[test]
    fn test_announcement() -> anyhow::Result<()> {
        let key_binding = KeyBinding::new(*CHAIN_ADDR, &KEY_PAIR);
        let peer_id = KEY_PAIR.public().to_peerid_str();

        for (ma_str, decapsulated_ma_str) in vec![
            (
                format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}"),
                "/ip4/127.0.0.1/tcp/10000".to_string(),
            ),
            (
                format!("/ip6/::1/tcp/10000/p2p/{peer_id}"),
                "/ip6/::1/tcp/10000".to_string(),
            ),
            (
                format!("/dns4/hoprnet.org/tcp/10000/p2p/{peer_id}"),
                "/dns4/hoprnet.org/tcp/10000".to_string(),
            ),
            (
                format!("/dns6/hoprnet.org/tcp/10000/p2p/{peer_id}"),
                "/dns6/hoprnet.org/tcp/10000".to_string(),
            ),
            (
                format!("/ip4/127.0.0.1/udp/10000/quic/p2p/{peer_id}"),
                "/ip4/127.0.0.1/udp/10000/quic".to_string(),
            ),
        ] {
            let maddr: Multiaddr = ma_str.parse()?;

            let ad = AnnouncementData::new(Some(maddr), key_binding.clone())?;
            assert_eq!(
                decapsulated_ma_str,
                ad.multiaddress().as_ref().map(ToString::to_string).unwrap_or_default()
            );
            assert_eq!(key_binding.clone(), ad.key_binding);
        }

        Ok(())
    }

    #[test]
    fn test_announcement_no_multiaddress() -> anyhow::Result<()> {
        let key_binding = KeyBinding::new(*CHAIN_ADDR, &KEY_PAIR);

        let ad = AnnouncementData::new(None, key_binding.clone())?;

        assert!(ad.multiaddress().is_none());
        assert_eq!(
            "".to_string(),
            ad.multiaddress().as_ref().map(ToString::to_string).unwrap_or_default()
        );

        Ok(())
    }

    #[test]
    fn test_announcement_decapsulated_ma() -> anyhow::Result<()> {
        let key_binding = KeyBinding::new(*CHAIN_ADDR, &KEY_PAIR);
        let maddr: Multiaddr = "/ip4/127.0.0.1/tcp/10000".to_string().parse()?;

        let ad = AnnouncementData::new(Some(maddr), key_binding.clone())?;
        assert_eq!(
            "/ip4/127.0.0.1/tcp/10000",
            ad.multiaddress().as_ref().map(ToString::to_string).unwrap_or_default()
        );
        assert_eq!(key_binding, ad.key_binding);

        Ok(())
    }

    #[test]
    fn test_announcement_wrong_peerid() -> anyhow::Result<()> {
        let key_binding = KeyBinding::new(*CHAIN_ADDR, &KEY_PAIR);
        let peer_id = SECOND_KEY_PAIR.public().to_peerid_str();
        let maddr: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse()?;

        assert!(AnnouncementData::new(Some(maddr), key_binding.clone()).is_err());

        Ok(())
    }

    #[test]
    fn test_decapsulate_multiaddr() -> anyhow::Result<()> {
        let maddr_1: Multiaddr = "/ip4/127.0.0.1/tcp/10000".parse()?;
        let maddr_2 = maddr_1
            .clone()
            .with_p2p(OffchainKeypair::random().public().into())
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(maddr_1, decapsulate_multiaddress(maddr_2), "multiaddresses must match");
        assert_eq!(
            maddr_1,
            decapsulate_multiaddress(maddr_1.clone()),
            "decapsulation must be idempotent"
        );

        Ok(())
    }
}
