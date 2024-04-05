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

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use lazy_static::lazy_static;
    use hopr_crypto_random::random_bytes;
    use hopr_crypto_types::prelude::{ChainKeypair, Keypair};
    use hopr_crypto_types::types::Hash;
    use hopr_primitive_types::prelude::{BalanceType, BinarySerializable, EthereumChallenge};
    use hopr_primitive_types::primitives::Address;
    use crate::prelude::Ticket;


    #[test]
    fn test_legacy_binary_compatibility_with_2_0_8() {
        let ckp = ChainKeypair::from_secret(&hex!("14d2d952715a51aadbd4cc6bfac9aa9927182040da7b336d37d5bb7247aa7566")).unwrap();
        let dst = hex!("345ae204774ff2b3e8d4cac884dad3d1603b5917");
        let channel_dst = hex!("57dc754bb522f2fe7799e471fd6efd0b6139a2120198f15b92f4a78cb882af35");
        let challenge = hex!("4162339a4204a1cedf43c92049875a19cb09dd20");
        let response = hex!("83c841f72b270440b7c8cd7b4f7d806a84f40ead5b04edccbb9a4c8936b91436");

        let ticket = Ticket::new(
            &Address::new(&dst),
            &BalanceType::HOPR.balance(1000000_u64),
            10.into(),
            2.into(),
            1.0_f64,
            2.into(),
            EthereumChallenge::new(&challenge),
            &ckp,
            &Hash::new(&channel_dst)
        ).unwrap();

        let mut signer = crate::legacy::Address::default();
        signer.addr.copy_from_slice(&ckp.public().to_address().to_bytes());

        let ack_ticket = crate::legacy::AcknowledgedTicket {
            status: crate::legacy::AcknowledgedTicketStatus::BeingAggregated {start: 0, end: 0},
            ticket,
            response: crate::legacy::Response { response },
            vrf_params: Default::default(),
            signer,
        };

        let serialized = cbor4ii::serde::to_vec(Vec::with_capacity(300), &ack_ticket).unwrap();

        let expected = hex!("00");

        //assert_eq!(expected.as_ref(), serialized.as_slice());
    }
}
