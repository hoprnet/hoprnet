use serde::{Deserialize, Serialize};
use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;

use crate::channels::Ticket;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcknowledgedTicket {
    #[serde(default)]
    pub status: AcknowledgedTicketStatus,
    pub ticket: Ticket,
    pub response: Response,
    pub vrf_params: VrfParameters,
    pub signer: Address,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub struct Address {
    addr: [u8; 20],
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, PartialOrd, Ord, std::hash::Hash)]
pub struct Hash {
    hash: [u8; 32],
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response {
    response: [u8; 32],
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum AcknowledgedTicketStatus {
    /// The ticket is available for redeeming or aggregating
    #[default]
    Untouched,
    /// Ticket is currently being redeemed in and on-going redemption process
    BeingRedeemed { tx_hash: Hash },
    /// Ticket is currently being aggregated in and on-going aggregation process
    BeingAggregated { start: u64, end: u64 },
}

impl From<AcknowledgedTicket> for crate::acknowledgement::AcknowledgedTicket {
    fn from(value: AcknowledgedTicket) -> Self {
        Self {
            status: crate::acknowledgement::AcknowledgedTicketStatus::Untouched,
            ticket: value.ticket,
            response: hopr_crypto_types::types::Response::new(&value.response.response),
            vrf_params: value.vrf_params,
            signer: hopr_primitive_types::primitives::Address::new(&value.signer.addr),
        }
    }
}

impl From<crate::acknowledgement::AcknowledgedTicket> for AcknowledgedTicket {
    fn from(value: crate::acknowledgement::AcknowledgedTicket) -> Self {
        let mut response = Response::default();
        response.response.copy_from_slice(&value.response.to_bytes());

        let mut signer = Address::default();
        signer.addr.copy_from_slice(value.signer.as_ref());

        Self {
            status: AcknowledgedTicketStatus::BeingAggregated {start: 0, end: 0}, // values not  used
            ticket: value.ticket,
            response,
            vrf_params: value.vrf_params,
            signer,
        }
    }
}
