use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use tracing::debug;

use crate::{
    channels::Ticket,
    errors::Result as CoreTypesResult,
};
use crate::channels::check_ticket_win;
use crate::prelude::VerifiedTicket;

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

    /// Convenience method to retrieve a reference to the underlying verified [Ticket].
    pub fn verified_ticket(&self) -> &Ticket {
        self.ticket.verified_ticket()
    }

    /// Checks if this acknowledged ticket is winning.
    pub fn is_winning(&self, chain_keypair: &ChainKeypair, domain_separator: &Hash) -> bool {
        self.ticket.is_winning(&self.response, chain_keypair, domain_separator)
    }

    /// Transforms this ticket into [RedeemableTicket] that can be redeemed on-chain
    /// or transformed into [TransferableWinningTicket] that can be sent for aggregation.
    pub fn into_redeemable(self, chain_keypair: &ChainKeypair, domain_separator: &Hash) -> crate::errors::Result<RedeemableTicket> {
        let vrf_params = derive_vrf_parameters(
            self.ticket.verified_hash(),
            chain_keypair,
            domain_separator.as_ref(),
        )?;

        Ok(RedeemableTicket {
            ticket: self.ticket,
            response: self.response,
            vrf_params,
        })
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

/// Represents a winning ticket that can be successfully redeemed on chain.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedeemableTicket {
    pub ticket: VerifiedTicket,
    pub response: Response,
    pub vrf_params: VrfParameters,
}

impl RedeemableTicket {
    /// Convenience method to retrieve a reference to the underlying verified [Ticket].
    pub fn verified_ticket(&self) -> &Ticket {
        self.ticket.verified_ticket()
    }
}

/// Represents a ticket that could be transferred over the wire
/// and independently verified again by the other party.
///
/// The [TransferableWinningTicket] can be easily retrieved from [RedeemableTicket], which strips
/// information about verification.
/// [TransferableWinningTicket] can be attempted to be converted back to [RedeemableTicket] only
/// when verified via [`TransferableWinningTicket::into_redeemable`] again.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferableWinningTicket {
    pub ticket: Ticket,
    pub response: Response,
    pub vrf_params: VrfParameters,
    pub signer: Address,
}

impl TransferableWinningTicket {
    /// Attempts to transform this ticket back into a [RedeemableTicket].
    ///
    /// Verifies that the `signer` matches the `expected_issuer` and that the
    /// ticket has valid signature from the `signer`.
    /// Then it verifies if the ticket is winning and therefore if it can be successfully
    /// redeemed on-chain.
    pub fn into_redeemable(self, expected_issuer: &Address, domain_separator: &Hash) -> crate::errors::Result<RedeemableTicket> {
        if !self.signer.eq(expected_issuer) {
            return Err(crate::errors::CoreTypesError::InvalidInputData("invalid ticket issuer".into()))
        }

        let verified_ticket = self.ticket.verify(&self.signer, domain_separator)?;

        if check_ticket_win(
            verified_ticket.verified_hash(),
            &verified_ticket.verified_signature(),
            &verified_ticket.verified_ticket().encoded_win_prob,
            &self.response,
            &self.vrf_params
        ) {
            Ok(RedeemableTicket {
                ticket: verified_ticket,
                response: self.response,
                vrf_params: Default::default(),
            })
        } else {
            Err(crate::errors::CoreTypesError::InvalidInputData("ticket is not a win".into()))
        }
    }
}

impl From<RedeemableTicket> for TransferableWinningTicket {
    fn from(value: RedeemableTicket) -> Self {
        Self {
            response: value.response,
            vrf_params: value.vrf_params,
            signer: *value.ticket.verified_issuer(),
            ticket: value.ticket.leak(),
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::{
        tickets::{AcknowledgedTicket, UnacknowledgedTicket},
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
