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
    Announced(Multiaddr),
}

impl Display for AccountType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::NotAnnounced => {
                write!(f, "not announced")
            }
            Self::Announced(multiaddr) => write!(f, "announced as {multiaddr}"),
        }
    }
}

/// Represents a node announcement entry on the block chain.
/// This contains node's public key and optional announcement information (multiaddress).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccountEntry {
    pub public_key: OffchainPublicKey,
    pub chain_addr: Address,
    pub entry_type: AccountType,
    pub safe_address: Option<Address>,
    pub key_id: KeyIdent<4>,
}

impl AccountEntry {
    /// Is the node announced?
    pub fn has_announced(&self) -> bool {
        matches!(self.entry_type, AccountType::Announced { .. })
    }

    /// If the node has announced, did it announce with routing information?
    pub fn contains_routing_info(&self) -> bool {
        match &self.entry_type {
            AccountType::NotAnnounced => false,
            AccountType::Announced(multiaddr) => {
                multiaddr.protocol_stack().any(|p| p == "ip4" || p == "dns4" || p == "ip6" || p == "dns6")
                    && multiaddr.protocol_stack().any(|p| p == "tcp" || p == "udp")
            }
        }
    }

    pub fn get_multiaddr(&self) -> Option<Multiaddr> {
        match &self.entry_type {
            AccountType::NotAnnounced => None,
            AccountType::Announced(multiaddr) => Some(multiaddr.clone()),
        }
    }

    /// Used to either set a PRN or revoke being a PRN
    ///
    /// Examples:
    /// - a node transitions from being a PRN to an edge node
    /// - a node becomes a PRN
    /// - the IP of a PRN has changed, e.g., due to relocation
    pub fn update(&mut self, new_entry_type: AccountType) {
        self.entry_type = new_entry_type;
    }
}

impl Display for AccountEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "account {} ({}:{}) (safe: {:?}) {}",
            self.key_id,
            self.chain_addr,
            self.public_key.to_peerid_str(),
            self.safe_address,
            self.entry_type
        )
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
            key_id: 1.into(),
            entry_type: Announced("/p2p/16Uiu2HAm3rUQdpCz53tK1MVUUq9NdMAU6mFgtcXrf71Ltw6AStzk".parse()?),
            safe_address: None,
        };

        assert!(ae1.has_announced());
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
            key_id: 1.into(),
            entry_type: Announced("/ip4/34.65.237.196/tcp/9091/p2p/16Uiu2HAm3rUQdpCz53tK1MVUUq9NdMAU6mFgtcXrf71Ltw6AStzk".parse()?),
            safe_address: None,
        };

        assert!(ae1.has_announced());
        assert!(ae1.contains_routing_info());

        let ae1 = AccountEntry {
            public_key,
            chain_addr,
            key_id: 1.into(),
            entry_type: Announced("/ip4/34.65.237.196/udp/9091/p2p/16Uiu2HAm3rUQdpCz53tK1MVUUq9NdMAU6mFgtcXrf71Ltw6AStzk".parse()?),
            safe_address: None,
        };

        assert!(ae1.has_announced());
        assert!(ae1.contains_routing_info());

        Ok(())
    }

    #[test]
    fn test_account_entry_not_announced() -> anyhow::Result<()> {
        let public_key = OffchainPublicKey::from_privkey(&PRIVATE_KEY)?;
        let chain_addr = Address::try_from(CHAIN_ADDR.as_ref())?;

        let ae1 = AccountEntry {
            public_key,
            chain_addr,
            key_id: 0.into(),
            entry_type: NotAnnounced,
            safe_address: None,
        };

        assert!(!ae1.has_announced());
        assert!(!ae1.contains_routing_info());

        Ok(())
    }
}
