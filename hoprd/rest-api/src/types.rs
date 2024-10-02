// Unified type for PeerID and Address
//
// This module provides a unified type for PeerID and Address. This is useful for APIs that accept both PeerID and Address.

use core::result::Result;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_db_sql::prelude::HoprDbResolverOperations;
use hopr_lib::{Address, GeneralError};
use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Copy, Default, Eq, Hash, Ord, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct PeerOrAddress {
    pub peer_id: Option<PeerId>,
    pub address: Option<Address>,
}

impl PeerOrAddress {
    pub fn new(identifier: String) -> Self {
        // string to &[u8]
        if let Ok(peer_id) = PeerId::from_str(&identifier) {
            Self::from(peer_id)
        } else if let Ok(address) = Address::from_str(&identifier) {
            Self::from(address)
        } else {
            Self::default()
        }
    }

    pub async fn fullfill<T: HoprDbResolverOperations>(&mut self, resolver: &T) -> Result<Self, ()> {
        if let Some(peer_id) = self.peer_id {
            let offchain_key = OffchainPublicKey::try_from(peer_id).unwrap();

            if let Ok(Some(address)) = resolver.resolve_chain_key(&offchain_key).await {
                self.address = Some(address);
                return Ok(*self);
            }
        } else if let Some(address) = self.address {
            if let Ok(Some(offchain_key)) = resolver.resolve_packet_key(&address).await {
                self.peer_id = Some(PeerId::from(offchain_key));
                return Ok(*self);
            }
        }

        Err(())
    }
}

impl From<PeerId> for PeerOrAddress {
    fn from(peer_id: PeerId) -> Self {
        Self {
            peer_id: Some(peer_id),
            address: None,
        }
    }
}

impl From<Address> for PeerOrAddress {
    fn from(address: Address) -> Self {
        Self {
            peer_id: None,
            address: Some(address),
        }
    }
}
impl FromStr for PeerOrAddress {
    type Err = GeneralError;

    fn from_str(value: &str) -> Result<PeerOrAddress, hopr_lib::GeneralError> {
        Ok(Self::new(value.to_owned()))
    }
}

impl Debug for PeerOrAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:?}", self.peer_id, self.address)
    }
}

impl Display for PeerOrAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:?}", self.peer_id, self.address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foo_test() {}
}
