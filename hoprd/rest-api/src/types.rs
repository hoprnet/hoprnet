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
use hopr_db_api::prelude::HoprDbResolverOperations;
use hopr_lib::{Address, GeneralError};

use crate::ApiErrorStatus;

#[derive(Debug, Clone, Copy, Eq, Hash, Ord, Serialize, Deserialize, PartialEq, PartialOrd, ToSchema)]
/// Unified type for PeerId and Address
///
/// This enum can be used to represent either a PeerId or an Address.
/// It is used in the API to accept both types of input.
pub enum PeerOrAddress {
    #[schema(value_type = String)]
    PeerId(PeerId),
    #[schema(value_type = String)]
    Address(Address),
}

impl From<PeerId> for PeerOrAddress {
    fn from(peer_id: PeerId) -> Self {
        Self::PeerId(peer_id)
    }
}

impl From<Address> for PeerOrAddress {
    fn from(address: Address) -> Self {
        Self::Address(address)
    }
}
impl FromStr for PeerOrAddress {
    type Err = GeneralError;

    fn from_str(value: &str) -> Result<PeerOrAddress, GeneralError> {
        if value.starts_with("0x") {
            Address::from_str(value)
                .map(PeerOrAddress::from)
                .map_err(|_e| GeneralError::ParseError("PeerOrAddress".into()))
        } else {
            PeerId::from_str(value)
                .map(PeerOrAddress::from)
                .map_err(|_e| GeneralError::ParseError("PeerOrAddress".into()))
        }
    }
}

impl Display for PeerOrAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerOrAddress::PeerId(peer_id) => write!(f, "{}", peer_id),
            PeerOrAddress::Address(address) => write!(f, "{}", address),
        }
    }
}

pub struct HoprIdentifier {
    pub peer_id: PeerId,
    pub address: Address,
}

impl HoprIdentifier {
    pub async fn new_with<T: HoprDbResolverOperations>(
        peer_or_address: PeerOrAddress,
        resolver: &T,
    ) -> Result<Self, ApiErrorStatus> {
        match peer_or_address {
            PeerOrAddress::PeerId(peer_id) => {
                let offchain_key = match OffchainPublicKey::try_from(peer_id) {
                    Ok(key) => key,
                    Err(_) => return Err(ApiErrorStatus::InvalidInput),
                };

                match resolver.resolve_chain_key(&offchain_key).await {
                    Ok(Some(address)) => Ok(HoprIdentifier { peer_id, address }),
                    Ok(None) => Err(ApiErrorStatus::PeerNotFound),
                    Err(_) => Err(ApiErrorStatus::PeerNotFound),
                }
            }
            PeerOrAddress::Address(address) => match resolver.resolve_packet_key(&address).await {
                Ok(Some(offchain_key)) => Ok(HoprIdentifier {
                    peer_id: PeerId::from(offchain_key),
                    address,
                }),
                Ok(None) => Err(ApiErrorStatus::PeerNotFound),
                Err(_) => Err(ApiErrorStatus::PeerNotFound),
            },
        }
    }
}
