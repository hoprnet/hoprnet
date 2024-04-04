use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use tracing::debug;

use crate::{
    channels::{Ticket},
    errors::{
        Result as CoreTypesResult,
    },
};
use crate::prelude::VerifiedTicket;

/// Represents packet acknowledgement
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Acknowledgement {
    ack_signature: OffchainSignature,
    pub ack_key_share: HalfKey,
    validated: bool,
}

impl Acknowledgement {
    pub fn new(ack_key_share: HalfKey, node_keypair: &OffchainKeypair) -> Self {
        Self {
            ack_signature: OffchainSignature::sign_message(ack_key_share.as_ref(), node_keypair),
            ack_key_share,
            validated: true,
        }
    }

    /// Validates the acknowledgement. Must be called immediately after deserialization or otherwise
    /// any operations with the deserialized acknowledgement will panic.
    pub fn validate(&mut self, sender_node_key: &OffchainPublicKey) -> bool {
        self.validated = self
            .ack_signature
            .verify_message(self.ack_key_share.as_ref(), sender_node_key);

        self.validated
    }

    /// Obtains the acknowledged challenge out of this acknowledgment.
    pub fn ack_challenge(&self) -> HalfKeyChallenge {
        assert!(self.validated, "acknowledgement not validated");
        self.ack_key_share.to_challenge()
    }
}

/// Status of the acknowledged ticket.
#[repr(u8)]
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    num_enum::IntoPrimitive,
    num_enum::TryFromPrimitive,
)]
#[strum(serialize_all = "PascalCase")]
pub enum AcknowledgedTicketStatus {
    /// The ticket is available for redeeming or aggregating
    #[default]
    Untouched = 0,
    /// Ticket is currently being redeemed in and ongoing redemption process
    BeingRedeemed = 1,
    /// Ticket is currently being aggregated in and ongoing aggregation process
    BeingAggregated = 2,
}

/// Contains acknowledgment information and the respective ticket
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcknowledgedTicket {
    #[serde(default)]
    pub status: AcknowledgedTicketStatus,
    pub ticket: VerifiedTicket,
    pub response: Response,
}

impl PartialOrd for AcknowledgedTicket {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AcknowledgedTicket {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ticket.cmp(&other.ticket)
    }
}

impl AcknowledgedTicket {
    /// Creates an acknowledged ticket out of a plain ticket.
    pub fn new(
        ticket: VerifiedTicket,
        response: Response,
    ) -> AcknowledgedTicket {
        Self {
            // new tickets are always untouched
            status: AcknowledgedTicketStatus::Untouched,
            ticket,
            response,
        }
    }

    pub fn verified_ticket(&self) -> &Ticket {
        self.ticket.verified_ticket()
    }
}

impl Display for AcknowledgedTicket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "acknowledged {} in state '{}'", self.ticket, self.status)
    }
}

/// Wrapper for a **verified** unacknowledged ticket and the half-key for the ticket's challenge.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnacknowledgedTicket {
    pub ticket: VerifiedTicket,
    pub(crate) own_key: HalfKey,
}

impl UnacknowledgedTicket {
    /// Convenience method to retrieve a reference to the underlying verified [Ticket].
    pub fn verified_ticket(&self) -> &Ticket {
        self.ticket.verified_ticket()
    }

    /// Verifies that the given acknowledgement solves this ticket's challenge and then
    /// turns this unacknowledged ticket into an acknowledged ticket by adding
    /// the received acknowledgement of the forwarded packet.
    pub fn acknowledge(
        self,
        acknowledgement: &HalfKey,
    ) -> CoreTypesResult<AcknowledgedTicket> {
        let response = Response::from_half_keys(&self.own_key, acknowledgement)?;
        debug!("acknowledging ticket using response {}", response.to_hex());

        if self.ticket.verified_ticket().challenge == response.to_challenge().into() {
            Ok(AcknowledgedTicket::new(self.ticket, response))
        } else {
            Err(CryptoError::InvalidChallenge.into())
        }
    }
}

/// Represents a winning ticket that can be aggregated and redeemed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedeemableTicket {
    pub ticket: VerifiedTicket,
    pub response: Response,
    pub vrf_params: VrfParameters,
    pub issuer: Address,
}

impl RedeemableTicket {
    pub fn verify_and_check_if_winning(&self, destination: &Address, domain_separator: &Hash) -> CoreTypesResult<bool> {
        todo!()

    }
}

/// Contains either unacknowledged ticket if we're waiting for the acknowledgement as a relayer
/// or information if we wait for the acknowledgement as a sender.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PendingAcknowledgement {
    /// We're waiting for acknowledgement as a sender
    WaitingAsSender,
    /// We're waiting for the acknowledgement as a relayer with a ticket
    WaitingAsRelayer(UnacknowledgedTicket),
}

#[cfg(test)]
pub mod test {
    use crate::{
        acknowledgement::{AcknowledgedTicket, UnacknowledgedTicket},
        channels::Ticket,
    };
    use hex_literal::hex;
    use hopr_crypto_types::{
        keypairs::{ChainKeypair, Keypair},
        types::{Challenge, CurvePoint, HalfKey, Hash, Response},
    };
    use hopr_primitive_types::prelude::UnitaryFloatOps;
    use hopr_primitive_types::primitives::{Address, Balance, BalanceType, EthereumChallenge, U256};

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).unwrap();
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).unwrap();
    }

    fn mock_ticket(
        pk: &ChainKeypair,
        counterparty: &Address,
        domain_separator: Option<Hash>,
        challenge: Option<EthereumChallenge>,
    ) -> Ticket {
        let win_prob = 1.0f64; // 100 %
        let price_per_packet: U256 = 10000000000000000u128.into(); // 0.01 HOPR
        let path_pos = 5u64;

        Ticket::new(
            counterparty,
            Balance::new(
                price_per_packet.div_f64(win_prob).unwrap() * U256::from(path_pos),
                BalanceType::HOPR,
            ),
            0,
            1,
            1.0f64,
            4,
            challenge.unwrap_or_default(),
            pk,
            &domain_separator.unwrap_or_default(),
        )
        .unwrap()
    }

    #[test]
    fn test_unacknowledged_ticket_sign_verify() {
        let unacked_ticket = UnacknowledgedTicket::new(
            mock_ticket(&ALICE, &BOB.public().to_address(), None, None),
            HalfKey::default(),
        );

        assert!(unacked_ticket.verify_signature(&Hash::default()).is_ok());
    }

    #[test]
    fn test_unacknowledged_ticket_challenge_response() {
        let hk1 = HalfKey::try_from(hex!("3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa").as_ref())
            .unwrap();

        let hk2 = HalfKey::try_from(hex!("4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b").as_ref())
            .unwrap();

        let cp1: CurvePoint = hk1.to_challenge().try_into().unwrap();
        let cp2: CurvePoint = hk2.to_challenge().try_into().unwrap();
        let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

        let ticket = mock_ticket(
            &ALICE,
            &BOB.public().to_address(),
            None,
            Some(Challenge::from(cp_sum).to_ethereum_challenge()),
        );

        let unacked_ticket = UnacknowledgedTicket::new(ticket, hk1, ALICE.public().to_address());

        assert!(unacked_ticket.verify_signature(&Hash::default()).is_ok());
        assert!(unacked_ticket.verify_challenge(&hk2).is_ok())
    }

    #[test]
    fn test_unack_transformation() {
        let hk1 = HalfKey::try_from(hex!("3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa").as_ref())
            .unwrap();

        let hk2 = HalfKey::try_from(hex!("4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b").as_ref())
            .unwrap();

        let cp1: CurvePoint = hk1.to_challenge().try_into().unwrap();
        let cp2: CurvePoint = hk2.to_challenge().try_into().unwrap();
        let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

        let ticket = mock_ticket(
            &ALICE,
            &BOB.public().to_address(),
            None,
            Some(Challenge::from(cp_sum).to_ethereum_challenge()),
        );

        let unacked_ticket = UnacknowledgedTicket::new(ticket, hk1, ALICE.public().to_address());

        let acked_ticket = unacked_ticket.acknowledge(&hk2, &BOB, &Hash::default()).unwrap();

        assert!(acked_ticket
            .verify(
                &ALICE.public().to_address(),
                &BOB.public().to_address(),
                &Hash::default()
            )
            .is_ok());
    }

    #[test]
    fn test_acknowledged_ticket() {
        let response =
            Response::try_from(hex!("876a41ee5fb2d27ac14d8e8d552692149627c2f52330ba066f9e549aef762f73").as_ref())
                .unwrap();

        let ticket = mock_ticket(
            &ALICE,
            &BOB.public().to_address(),
            None,
            Some(response.to_challenge().into()),
        );

        let acked_ticket =
            AcknowledgedTicket::new(ticket, response, ALICE.public().to_address(), &BOB, &Hash::default()).unwrap();

        let mut deserialized_ticket =
            bincode::deserialize::<AcknowledgedTicket>(&bincode::serialize(&acked_ticket).unwrap()).unwrap();
        assert_eq!(acked_ticket, deserialized_ticket);

        assert!(deserialized_ticket
            .verify(
                &ALICE.public().to_address(),
                &BOB.public().to_address(),
                &Hash::default()
            )
            .is_ok());

        deserialized_ticket.status = super::AcknowledgedTicketStatus::BeingAggregated;

        assert_eq!(
            deserialized_ticket,
            bincode::deserialize(&bincode::serialize(&deserialized_ticket).unwrap()).unwrap()
        );
    }
}
