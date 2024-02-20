use hopr_crypto_types::types::OffchainPublicKey;
use hopr_primitive_types::prelude::*;
use multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

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
            Self::NotAnnounced => {
                write!(f, "Not announced")
            }
            Self::Announced {
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
pub struct AccountEntry {
    pub public_key: OffchainPublicKey,
    pub chain_addr: Address,
    entry_type: AccountType,
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

    /// Gets the block number of the announcement if this peer ID has been announced.
    pub fn updated_at(&self) -> Option<u32> {
        match &self.entry_type {
            AccountType::NotAnnounced => None,
            AccountType::Announced { updated_block, .. } => Some(*updated_block),
        }
    }

    /// Is the node announced?
    pub fn has_announced(&self) -> bool {
        matches!(self.entry_type, AccountType::Announced { .. })
    }

    /// If the node has announced, did it announce with routing information ?
    pub fn contains_routing_info(&self) -> bool {
        match &self.entry_type {
            AccountType::NotAnnounced => false,
            AccountType::Announced { multiaddr, .. } => {
                multiaddr.protocol_stack().any(|p| p == "ip4" || p == "dns4")
                    && multiaddr.protocol_stack().any(|p| p == "tcp")
            }
        }
    }

    pub fn get_multiaddr(&self) -> Option<Multiaddr> {
        match &self.entry_type {
            AccountType::NotAnnounced => None,
            AccountType::Announced { multiaddr, .. } => Some(multiaddr.clone()),
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
            AccountType::NotAnnounced => {
                write!(f, " Multiaddr: not announced")?;
                write!(f, " UpdatedAt: not announced")?;
                write!(f, " RoutingInfo: false")?;
            }
            AccountType::Announced {
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

    fn from_bytes(data: &[u8]) -> hopr_primitive_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            let mut buf = data.to_vec();
            let public_key = OffchainPublicKey::from_bytes(buf.drain(..OffchainPublicKey::SIZE).as_ref())?;
            let mut chain_addr = [0u8; Address::SIZE];
            chain_addr.copy_from_slice(buf.drain(..Address::SIZE).as_ref());
            let ma_len = u32::from_be_bytes(buf.drain(..Self::MA_LENGTH_PREFIX).as_ref().try_into().unwrap()) as usize;
            let entry_type = if ma_len > 0 {
                let multiaddr = Multiaddr::try_from(buf.drain(..ma_len).collect::<Vec<u8>>())
                    .map_err(|_| GeneralError::ParseError)?;
                buf.drain(..Self::MAX_MULTI_ADDR_LENGTH - ma_len);
                AccountType::Announced {
                    multiaddr,
                    updated_block: u32::from_be_bytes(
                        buf.drain(..std::mem::size_of::<u32>()).as_ref().try_into().unwrap(),
                    ),
                }
            } else {
                AccountType::NotAnnounced
            };
            Ok(Self {
                public_key,
                chain_addr: chain_addr.into(),
                entry_type,
            })
        } else {
            Err(GeneralError::ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.public_key.to_bytes());
        ret.extend_from_slice(self.chain_addr.as_ref());

        match &self.entry_type {
            AccountType::NotAnnounced => {
                ret.extend_from_slice(&(0_u32).to_be_bytes());
                ret.extend_from_slice(&[0u8; Self::MAX_MULTI_ADDR_LENGTH]);
                ret.extend_from_slice(&(0_u32).to_be_bytes());
            }
            AccountType::Announced {
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
    use crate::account::{
        AccountEntry,
        AccountType::{Announced, NotAnnounced},
    };
    use hex_literal::hex;
    use hopr_crypto_types::types::OffchainPublicKey;
    use hopr_primitive_types::prelude::*;
    use multiaddr::Multiaddr;

    lazy_static::lazy_static! {
        static ref PUBLIC_KEY: OffchainPublicKey = OffchainPublicKey::from_privkey(&hex!("c14b8faa0a9b8a5fa4453664996f23a7e7de606d42297d723fc4a794f375e260")).unwrap();
        static ref CHAIN_ADDR: Address = hex!("2cDD13ddB0346E0F620C8E5826Da5d7230341c6E").into();
    }

    #[test]
    fn test_account_entry_non_routable() {
        let ae1 = AccountEntry::new(
            *PUBLIC_KEY,
            *CHAIN_ADDR,
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
        let ae1 = AccountEntry::new(
            *PUBLIC_KEY,
            *CHAIN_ADDR,
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
        let ae1 = AccountEntry::new(*PUBLIC_KEY, *CHAIN_ADDR, NotAnnounced);

        assert!(!ae1.has_announced());
        assert!(ae1.updated_at().is_none());
        assert!(!ae1.contains_routing_info());

        let ae2 = AccountEntry::from_bytes(&ae1.to_bytes()).unwrap();
        assert_eq!(ae1, ae2);
    }
}
