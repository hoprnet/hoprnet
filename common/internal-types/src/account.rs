use std::fmt::{Display, Formatter};

use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
use multiaddr::Multiaddr;

/// Type of the node account.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccountEntry {
    pub public_key: OffchainPublicKey,
    pub chain_addr: Address,
    pub entry_type: AccountType,
    pub published_at: u32,
}

impl AccountEntry {
    /// Gets the block number of the announcement if this peer ID has been announced.
    pub fn updated_at(&self) -> Option<u32> {
        match &self.entry_type {
            AccountType::NotAnnounced => None,
            AccountType::Announced { updated_block, .. } => Some(*updated_block),
        }
    }

    /// Returns the computed key ID for this account.
    // TODO: change this to use assigned ID from the SC in the next version
    pub fn key_id(&self) -> KeyIdent<4> {
        let id_hash = Hash::create(&[
            self.public_key.as_ref(),
            self.chain_addr.as_ref(),
            &self.published_at.to_be_bytes(),
        ]);

        u32::from_be_bytes(
            id_hash.as_ref()[0..std::mem::size_of::<u32>()]
                .try_into()
                .expect("4 byte must fit into u32"),
        )
        .into()
    }

    /// Is the node announced?
    pub fn has_announced(&self) -> bool {
        matches!(self.entry_type, AccountType::Announced { .. })
    }

    /// If the node has announced, did it announce with routing information?
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

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use hopr_crypto_types::types::OffchainPublicKey;
    use hopr_primitive_types::prelude::*;
    use multiaddr::Multiaddr;

    use crate::account::{
        AccountEntry,
        AccountType::{Announced, NotAnnounced},
    };

    const PRIVATE_KEY: [u8; 32] = hex!("c14b8faa0a9b8a5fa4453664996f23a7e7de606d42297d723fc4a794f375e260");
    const CHAIN_ADDR: [u8; 20] = hex!("2cDD13ddB0346E0F620C8E5826Da5d7230341c6E");

    #[test]
    fn test_account_entry_non_routable() -> anyhow::Result<()> {
        let public_key = OffchainPublicKey::from_privkey(&PRIVATE_KEY)?;
        let chain_addr = Address::try_from(CHAIN_ADDR.as_ref())?;

        let ae1 = AccountEntry {
            public_key,
            chain_addr,
            published_at: 1,
            entry_type: Announced {
                multiaddr: "/p2p/16Uiu2HAm3rUQdpCz53tK1MVUUq9NdMAU6mFgtcXrf71Ltw6AStzk".parse::<Multiaddr>()?,
                updated_block: 1,
            },
        };

        assert!(ae1.has_announced());
        assert_eq!(1, ae1.updated_at().expect("should be present"));
        assert!(!ae1.contains_routing_info());

        Ok(())
    }

    #[test]
    fn test_account_entry_routable() -> anyhow::Result<()> {
        let public_key = OffchainPublicKey::from_privkey(&PRIVATE_KEY)?;
        let chain_addr = Address::try_from(CHAIN_ADDR.as_ref())?;

        let ae1 = AccountEntry {
            public_key,
            chain_addr,
            published_at: 1,
            entry_type: Announced {
                multiaddr: "/ip4/34.65.237.196/tcp/9091/p2p/16Uiu2HAm3rUQdpCz53tK1MVUUq9NdMAU6mFgtcXrf71Ltw6AStzk"
                    .parse::<Multiaddr>()?,
                updated_block: 1,
            },
        };

        assert!(ae1.has_announced());
        assert_eq!(1, ae1.updated_at().expect("should be present"));
        assert!(ae1.contains_routing_info());
        assert_eq!("0x4e1ddc66", ae1.key_id().to_hex());

        Ok(())
    }

    #[test]
    fn test_account_entry_not_announced() -> anyhow::Result<()> {
        let public_key = OffchainPublicKey::from_privkey(&PRIVATE_KEY)?;
        let chain_addr = Address::try_from(CHAIN_ADDR.as_ref())?;

        let ae1 = AccountEntry {
            public_key,
            chain_addr,
            published_at: 0,
            entry_type: NotAnnounced,
        };

        assert!(!ae1.has_announced());
        assert!(ae1.updated_at().is_none());
        assert!(!ae1.contains_routing_info());

        Ok(())
    }
}
