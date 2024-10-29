// Unified type for PeerId and Address
//
// This module provides a unified type for PeerId and Address. This is useful for APIs that accept both PeerId and Address.

use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use utoipa::ToSchema;

use core::result::Result;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_db_sql::prelude::HoprDbResolverOperations;
use hopr_lib::{Address, GeneralError};

use crate::ApiErrorStatus;

#[derive(Clone, Copy, Default, Eq, Hash, Ord, Serialize, Deserialize, PartialEq, PartialOrd, ToSchema)]
pub struct PeerOrAddress {
    pub peer_id: Option<PeerId>,
    pub address: Option<Address>,
}

impl PeerOrAddress {
    pub fn new(identifier: String) -> Result<Self, GeneralError> {
        if let Ok(peer_id) = PeerId::from_str(&identifier) {
            Ok(Self::from(peer_id))
        } else if let Ok(address) = Address::from_str(&identifier) {
            Ok(Self::from(address))
        } else {
            Err(GeneralError::InvalidInput)
        }
    }

    pub async fn fulfill<T: HoprDbResolverOperations>(mut self, resolver: &T) -> Result<Self, ApiErrorStatus> {
        if let Some(peer_id) = self.peer_id {
            let offchain_key = match OffchainPublicKey::try_from(peer_id) {
                Ok(key) => key,
                Err(_) => return Err(ApiErrorStatus::InvalidInput),
            };

            match resolver.resolve_chain_key(&offchain_key).await {
                Ok(Some(address)) => {
                    self.address = Some(address);
                    Ok(self)
                }
                Ok(None) => Err(ApiErrorStatus::PeerNotFound),
                Err(_) => Err(ApiErrorStatus::PeerNotFound),
            }
        } else if let Some(address) = self.address {
            match resolver.resolve_packet_key(&address).await {
                Ok(Some(offchain_key)) => {
                    self.peer_id = Some(PeerId::from(offchain_key));
                    Ok(self)
                }
                Ok(None) => Err(ApiErrorStatus::PeerNotFound),
                Err(_) => Err(ApiErrorStatus::PeerNotFound),
            }
        } else {
            Err(ApiErrorStatus::InvalidInput)
        }
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

    fn from_str(value: &str) -> Result<PeerOrAddress, GeneralError> {
        match Self::new(value.to_owned()) {
            Ok(peer_or_address) => Ok(peer_or_address),
            Err(err) => Err(err),
        }
    }
}

impl Debug for PeerOrAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:?}", self.peer_id, self.address)
    }
}

impl Display for PeerOrAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match (self.peer_id, self.address) {
            (Some(peer_id), Some(address)) => write!(f, "peerId: {}, address: {}", peer_id, address),
            (Some(peer_id), _) => write!(f, "peerId: {}", peer_id),
            (_, Some(address)) => write!(f, "address: {}", address),
            _ => write!(f, "No peerId or address provided"),
        }
    }
}
