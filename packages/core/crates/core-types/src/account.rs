use serde::{Deserialize, Serialize};

use crate::account::AccountType::{Announced, NotAnnounced};
use core_crypto::keypairs::{Keypair, OffchainKeypair};
use core_crypto::types::{OffchainPublicKey, OffchainSignature};
use multiaddr::Multiaddr;
use std::fmt::{Display, Formatter};
use utils_types::errors::GeneralError::ParseError;
use utils_types::primitives::Address;
use utils_types::traits::{BinarySerializable, PeerIdLike, ToHex};

/// Type of the node account.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    /// Node is not announced.
    NotAnnounced,
    /// Node is announced with a multi-address
    Announced { multiaddr: Multiaddr, updated_block: u32 },
}

impl Display for AccountType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            NotAnnounced => {
                write!(f, "Not announced")
            }
            Announced {
                multiaddr,
                updated_block,
            } => f
                .debug_struct("AccountType")
                .field("MultiAddr", multiaddr)
                .field("UpdatedAt", updated_block)
                .finish(),
        }
    }
}

/// Represents a node announcement entry on the block chain.
/// This contains node's public key and optional announcement information (multiaddress, block number).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct AccountEntry {
    pub public_key: OffchainPublicKey,
    pub chain_addr: Address,
    entry_type: AccountType,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl AccountEntry {
    /// Gets multiaddress as string if this peer ID has been announced.
    pub fn get_multiaddr_str(&self) -> Option<String> {
        self.get_multiaddr().map(|m| m.to_string())
    }

    /// Gets the block number of the announcement if this peer ID has been announced.
    pub fn updated_at(&self) -> Option<u32> {
        match &self.entry_type {
            NotAnnounced => None,
            Announced { updated_block, .. } => Some(*updated_block),
        }
    }

    /// Is the node announced?
    pub fn has_announced(&self) -> bool {
        match &self.entry_type {
            NotAnnounced => false,
            Announced { .. } => true,
        }
    }

    /// If the node has announced, did it announce with routing information ?
    pub fn contains_routing_info(&self) -> bool {
        match &self.entry_type {
            NotAnnounced => false,
            Announced { multiaddr, .. } => {
                multiaddr.protocol_stack().any(|p| p == "ip4" || p == "ip6")
                    && multiaddr.protocol_stack().any(|p| p == "tcp")
            }
        }
    }
}

impl AccountEntry {
    const MAX_MULTI_ADDR_LENGTH: usize = 200;
    const MA_LENGTH_PREFIX: usize = std::mem::size_of::<u32>();

    pub fn new(public_key: OffchainPublicKey, address: Address, entry_type: AccountType) -> Self {
        Self {
            public_key,
            chain_addr: address,
            entry_type,
        }
    }

    pub fn get_multiaddr(&self) -> Option<Multiaddr> {
        match &self.entry_type {
            NotAnnounced => None,
            Announced { multiaddr, .. } => Some(multiaddr.clone()),
        }
    }

    /// Used to either set a PRN or revoke being a PRN
    ///
    /// Examples:
    /// - a node transitions from being a PRN to an edge node
    /// - a node becomes a PRN
    /// - the IP of a PRN has changed, e.g. due to relocation
    pub fn update(&mut self, new_entry_type: AccountType) {
        self.entry_type = new_entry_type;
    }
}

impl Display for AccountEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AccountEntry {}:", self.public_key.to_peerid_str())?;
        write!(f, " PublicKey: {}", self.public_key.to_hex())?;
        match &self.entry_type {
            NotAnnounced => {
                write!(f, " Multiaddr: not announced")?;
                write!(f, " UpdatedAt: not announced")?;
                write!(f, " RoutingInfo: false")?;
            }
            Announced {
                multiaddr,
                updated_block,
            } => {
                write!(f, " Multiaddr: {}", multiaddr)?;
                write!(f, " UpdatedAt: {}", updated_block)?;
                write!(f, " RoutingInfo: {}", self.contains_routing_info())?;
            }
        }
        Ok(())
    }
}

impl BinarySerializable for AccountEntry {
    const SIZE: usize =
        OffchainPublicKey::SIZE + Address::SIZE + Self::MA_LENGTH_PREFIX + Self::MAX_MULTI_ADDR_LENGTH + 4;

    fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            let mut buf = data.to_vec();
            let public_key = OffchainPublicKey::from_bytes(buf.drain(..OffchainPublicKey::SIZE).as_ref())?;
            let chain_addr = Address::from_bytes(buf.drain(..Address::SIZE).as_ref())?;
            let ma_len = u32::from_be_bytes(buf.drain(..Self::MA_LENGTH_PREFIX).as_ref().try_into().unwrap()) as usize;
            let entry_type = if ma_len > 0 {
                let multiaddr =
                    Multiaddr::try_from(buf.drain(..ma_len).collect::<Vec<u8>>()).map_err(|_| ParseError)?;
                buf.drain(..Self::MAX_MULTI_ADDR_LENGTH - ma_len);
                Announced {
                    multiaddr,
                    updated_block: u32::from_be_bytes(
                        buf.drain(..std::mem::size_of::<u32>()).as_ref().try_into().unwrap(),
                    ),
                }
            } else {
                NotAnnounced
            };
            Ok(Self {
                public_key,
                chain_addr,
                entry_type,
            })
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.public_key.to_bytes());
        ret.extend_from_slice(&self.chain_addr.to_bytes());

        match &self.entry_type {
            NotAnnounced => {
                ret.extend_from_slice(&(0_u32).to_be_bytes());
                ret.extend_from_slice(&[0u8; Self::MAX_MULTI_ADDR_LENGTH]);
                ret.extend_from_slice(&(0_u32).to_be_bytes());
            }
            Announced {
                multiaddr,
                updated_block,
            } => {
                let ma_bytes = multiaddr.to_vec();
                assert!(ma_bytes.len() <= Self::MAX_MULTI_ADDR_LENGTH, "multi address too long");
                ret.extend_from_slice(&(ma_bytes.len() as u32).to_be_bytes());
                ret.extend_from_slice(&ma_bytes);
                ret.extend((0..Self::MAX_MULTI_ADDR_LENGTH - ma_bytes.len()).map(|_| 0u8));
                ret.extend_from_slice(&updated_block.to_be_bytes());
            }
        }

        ret.into_boxed_slice()
    }
}

#[derive(Clone, Debug)]
pub struct AccountSignature {
    pub signature: OffchainSignature,
    pub pub_key: OffchainPublicKey,
    pub chain_key: Address,
}

impl AccountSignature {
    pub fn new(signing_key: &OffchainKeypair, chain_key: &Address) -> Self {
        Self {
            signature: OffchainSignature::sign_message(
                format!(
                    "HoprAnnouncements: cross-sign off-chain identity {} on-chain identity {}",
                    signing_key.public().to_peerid(),
                    chain_key
                )
                .as_bytes(),
                signing_key,
            ),
            pub_key: signing_key.public().to_owned(),
            chain_key: chain_key.to_owned(),
        }
    }

    pub fn verify(&self) -> bool {
        self.signature.verify_message(
            format!(
                "HoprAnnouncements: cross-sign off-chain identity {} on-chain identity {}",
                self.pub_key.to_peerid(),
                self.chain_key
            )
            .as_bytes(),
            &self.pub_key,
        )
    }
}

#[cfg(test)]
mod test {
    use super::AccountSignature;
    use crate::account::AccountEntry;
    use crate::account::AccountType::{Announced, NotAnnounced};
    use core_crypto::{
        keypairs::{Keypair, OffchainKeypair},
        types::OffchainPublicKey,
    };
    use hex_literal::hex;
    use multiaddr::Multiaddr;
    use utils_types::{primitives::Address, traits::BinarySerializable};

    const PRIVATE_KEY: [u8; 32] = hex!("c14b8faa0a9b8a5fa4453664996f23a7e7de606d42297d723fc4a794f375e260");
    const CHAIN_ADDR: [u8; 20] = hex!("2cDD13ddB0346E0F620C8E5826Da5d7230341c6E");

    #[test]
    fn test_account_entry_non_routable() {
        let pub_key = OffchainPublicKey::from_privkey(&PRIVATE_KEY).unwrap();
        let chain_addr = Address::from_bytes(&CHAIN_ADDR).unwrap();

        let ae1 = AccountEntry::new(
            pub_key,
            chain_addr,
            Announced {
                multiaddr: "/p2p/16Uiu2HAm3rUQdpCz53tK1MVUUq9NdMAU6mFgtcXrf71Ltw6AStzk"
                    .parse::<Multiaddr>()
                    .unwrap(),
                updated_block: 1,
            },
        );

        assert!(ae1.has_announced());
        assert_eq!(1, ae1.updated_at().unwrap());
        assert!(!ae1.contains_routing_info());

        let ae2 = AccountEntry::from_bytes(&ae1.to_bytes()).unwrap();
        assert_eq!(ae1, ae2);
    }

    #[test]
    fn test_account_entry_routable() {
        let pub_key = OffchainPublicKey::from_privkey(&PRIVATE_KEY).unwrap();
        let chain_addr = Address::from_bytes(&CHAIN_ADDR).unwrap();

        let ae1 = AccountEntry::new(
            pub_key,
            chain_addr,
            Announced {
                multiaddr: "/ip4/34.65.237.196/tcp/9091/p2p/16Uiu2HAm3rUQdpCz53tK1MVUUq9NdMAU6mFgtcXrf71Ltw6AStzk"
                    .parse::<Multiaddr>()
                    .unwrap(),
                updated_block: 1,
            },
        );

        assert!(ae1.has_announced());
        assert_eq!(1, ae1.updated_at().unwrap());
        assert!(ae1.contains_routing_info());

        let ae2 = AccountEntry::from_bytes(&ae1.to_bytes()).unwrap();
        assert_eq!(ae1, ae2);
    }

    #[test]
    fn test_account_entry_not_announced() {
        let pub_key = OffchainPublicKey::from_privkey(&PRIVATE_KEY).unwrap();
        let chain_addr = Address::from_bytes(&CHAIN_ADDR).unwrap();

        let ae1 = AccountEntry::new(pub_key, chain_addr, NotAnnounced);

        assert!(!ae1.has_announced());
        assert!(ae1.updated_at().is_none());
        assert!(!ae1.contains_routing_info());

        let ae2 = AccountEntry::from_bytes(&ae1.to_bytes()).unwrap();
        assert_eq!(ae1, ae2);
    }

    #[test]
    fn test_account_signature_workflow() {
        let keypair = OffchainKeypair::from_secret(&PRIVATE_KEY).unwrap();
        let chain_addr = Address::from_bytes(&CHAIN_ADDR).unwrap();

        let sig = AccountSignature::new(&keypair, &chain_addr);
        assert!(sig.verify());
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::account::AccountEntry;
    use crate::account::AccountType::{Announced, NotAnnounced};
    use core_crypto::types::OffchainPublicKey;
    use multiaddr::Multiaddr;
    use std::str::FromStr;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::{primitives::Address, traits::BinarySerializable};
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    impl AccountEntry {
        #[wasm_bindgen(constructor)]
        pub fn _new(
            public_key: OffchainPublicKey,
            chain_addr: Address,
            multiaddr: Option<String>,
            updated_at: Option<u32>,
        ) -> JsResult<AccountEntry> {
            if (multiaddr.is_some() && updated_at.is_some()) || (multiaddr.is_none() && updated_at.is_none()) {
                Ok(Self {
                    public_key,
                    chain_addr,
                    entry_type: match multiaddr {
                        None => NotAnnounced,
                        Some(multiaddr) => Announced {
                            multiaddr: ok_or_jserr!(Multiaddr::from_str(multiaddr.as_str()))?,
                            updated_block: updated_at.unwrap(),
                        },
                    },
                })
            } else {
                Err("must either specify both: multiaddr and updated_at or none".into())
            }
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<AccountEntry> {
            ok_or_jserr!(Self::from_bytes(data))
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &AccountEntry) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }
}
