// Unified type for PeerID and Address
//
// This module provides a unified type for PeerID and Address. This is useful for APIs that accept both PeerID and Address.

use core::result::Result;
use hopr_lib::{Address, GeneralError};
use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Copy, Eq, Hash, Ord, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct UnifiedPeerType {
    pub peer_id: PeerId,
    pub address: Address,
}

impl UnifiedPeerType {
    pub fn new(identifier: String) -> Self {
        // string to &[u8]
        if let Ok(peer_id) = PeerId::from_str(&identifier) {
            Self::from(peer_id)
        } else if let Ok(address) = Address::from_str(&identifier) {
            Self::from(address)
        } else {
            panic!("Invalid PeerId or Address")
        }
    }
}

impl From<PeerId> for UnifiedPeerType {
    fn from(peer_id: PeerId) -> Self {
        Self {
            peer_id,
            address: Address::default(),
        }
    }
}

impl From<Address> for UnifiedPeerType {
    fn from(address: Address) -> Self {
        Self {
            peer_id: PeerId::from_bytes(&address.to_bytes32()).unwrap(),
            address,
        }
    }
}
impl FromStr for UnifiedPeerType {
    type Err = GeneralError;

    fn from_str(value: &str) -> Result<UnifiedPeerType, hopr_lib::GeneralError> {
        Ok(Self::new(value.to_owned()))
    }
}

impl Debug for UnifiedPeerType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.peer_id, self.address)
    }
}

impl Display for UnifiedPeerType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.peer_id, self.address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foo_test() {}
}
