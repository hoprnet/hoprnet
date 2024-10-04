// Unified type for PeerId and Address
//
// This module provides a unified type for PeerId and Address. This is useful for APIs that accept both PeerId and Address.

use crate::ApiErrorStatus;
use axum::http::StatusCode;
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
    pub fn new(identifier: String) -> Result<Self, GeneralError> {
        if let Ok(peer_id) = PeerId::from_str(&identifier) {
            Ok(Self::from(peer_id))
        } else if let Ok(address) = Address::from_str(&identifier) {
            Ok(Self::from(address))
        } else {
            Err(GeneralError::InvalidInput)
        }
    }

    pub async fn fulfill<T: HoprDbResolverOperations>(
        &mut self,
        resolver: &T,
    ) -> Result<Self, (StatusCode, ApiErrorStatus)> {
        if let Some(peer_id) = self.peer_id {
            let offchain_key = match OffchainPublicKey::try_from(peer_id) {
                Ok(key) => key,
                Err(_) => return Err((StatusCode::NOT_FOUND, ApiErrorStatus::PeerNotFound)),
            };

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

        Err((StatusCode::NOT_FOUND, ApiErrorStatus::PeerNotFound))
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
