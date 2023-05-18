use serde::{Serialize, Deserialize};

use crate::account::AccountType::{Announced, NotAnnounced};
use core_crypto::types::PublicKey;
use multiaddr::Multiaddr;
use std::fmt::{Display, Formatter};
use utils_types::errors::GeneralError::ParseError;
use utils_types::primitives::Address;
use utils_types::traits::{BinarySerializable, PeerIdLike};

/// Type of the node account.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    /// Node is not announced.
    NotAnnounced,
    /// Node is announced with a multi-address
    Announced { multiaddr: Multiaddr, updated_block: u32 },
}

/// Represents a node announcement entry on the block chain.
/// This contains node's public key and optional announcement information (multiaddress, block number).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct AccountEntry {
    pub public_key: PublicKey,
    entry_type: AccountType,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl AccountEntry {
    /// Gets public key as an address
    pub fn get_address(&self) -> Address {
        self.public_key.to_address()
    }

    /// Gets public key as a PeerId string
    pub fn get_peer_id_str(&self) -> String {
        self.public_key.to_peerid_str()
    }

    /// Gets multiaddress as string if this peer ID has been announced.
    pub fn get_multiaddress_str(&self) -> Option<String> {
        match &self.entry_type {
            NotAnnounced => None,
            Announced { multiaddr, .. } => Some(multiaddr.to_string()),
        }
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
                multiaddr
                    .protocol_stack()
                    .find(|p| p == &"ip4" || p == &"ip6")
                    .is_some()
                    && multiaddr.protocol_stack().find(|p| p == &"tcp").is_some()
            }
        }
    }
}

impl AccountEntry {
    const MAX_MULTI_ADDR_LENGTH: usize = 200;
    const MA_LENGTH_PREFIX: usize = std::mem::size_of::<u32>();

    pub fn new(public_key: PublicKey, entry_type: AccountType) -> Self {
        Self { public_key, entry_type }
    }
}

impl Display for AccountEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AccountEntry {}:", self.public_key.to_peerid_str())?;
        write!(f, " PublicKey: {}", self.public_key.to_hex(true))?;
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

impl BinarySerializable<'_> for AccountEntry {
    const SIZE: usize = PublicKey::SIZE_UNCOMPRESSED + Self::MA_LENGTH_PREFIX + Self::MAX_MULTI_ADDR_LENGTH + 4;

    fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            let mut buf = data.to_vec();
            let public_key = PublicKey::from_bytes(buf.drain(..PublicKey::SIZE_UNCOMPRESSED).as_ref())?;
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
            Ok(Self { public_key, entry_type })
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.public_key.to_bytes(false));

        match &self.entry_type {
            NotAnnounced => {
                ret.extend_from_slice(&(0 as u32).to_be_bytes());
                ret.extend_from_slice(&[0u8; Self::MAX_MULTI_ADDR_LENGTH]);
                ret.extend_from_slice(&(0 as u32).to_be_bytes());
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

#[cfg(test)]
mod test {
    use crate::account::AccountEntry;
    use crate::account::AccountType::{Announced, NotAnnounced};
    use core_crypto::types::PublicKey;
    use hex_literal::hex;
    use multiaddr::Multiaddr;
    use utils_types::traits::BinarySerializable;

    const PRIVATE_KEY: [u8; 32] = hex!("c14b8faa0a9b8a5fa4453664996f23a7e7de606d42297d723fc4a794f375e260");

    #[test]
    fn test_account_entry_non_routable() {
        let pub_key = PublicKey::from_privkey(&PRIVATE_KEY).unwrap();

        let ae1 = AccountEntry::new(
            pub_key.clone(),
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
        let pub_key = PublicKey::from_privkey(&PRIVATE_KEY).unwrap();

        let ae1 = AccountEntry::new(
            pub_key.clone(),
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
        let pub_key = PublicKey::from_privkey(&PRIVATE_KEY).unwrap();

        let ae1 = AccountEntry::new(pub_key.clone(), NotAnnounced);

        assert!(!ae1.has_announced());
        assert!(ae1.updated_at().is_none());
        assert!(!ae1.contains_routing_info());

        let ae2 = AccountEntry::from_bytes(&ae1.to_bytes()).unwrap();
        assert_eq!(ae1, ae2);
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::account::AccountEntry;
    use crate::account::AccountType::{Announced, NotAnnounced};
    use core_crypto::types::PublicKey;
    use multiaddr::Multiaddr;
    use std::str::FromStr;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::traits::BinarySerializable;
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    impl AccountEntry {
        #[wasm_bindgen(constructor)]
        pub fn _new(
            public_key: PublicKey,
            multiaddr: Option<String>,
            updated_at: Option<u32>,
        ) -> JsResult<AccountEntry> {
            if (multiaddr.is_some() && updated_at.is_some()) || (multiaddr.is_none() && updated_at.is_none()) {
                Ok(Self {
                    public_key,
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

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }
}
