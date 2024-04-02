use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use tracing::debug;

use crate::{
    channels::{generate_channel_id, Ticket},
    errors::{
        CoreTypesError::{InvalidInputData, InvalidTicketRecipient, LoopbackTicket},
        Result as CoreTypesResult,
    },
};

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
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AcknowledgedTicket {
    #[serde(default)]
    pub status: AcknowledgedTicketStatus,
    pub ticket: Ticket,
    pub response: Response,
    pub vrf_params: VrfParameters,
    pub signer: Address,
}

impl PartialEq for AcknowledgedTicket {
    fn eq(&self, other: &Self) -> bool {
        self.status == other.status
            && self.ticket == other.ticket
            && self.response == other.response
            && self.signer == other.signer
    }
}

impl Eq for AcknowledgedTicket {}

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
        ticket: Ticket,
        response: Response,
        signer: Address,
        chain_keypair: &ChainKeypair,
        domain_separator: &Hash,
    ) -> CoreTypesResult<AcknowledgedTicket> {
        if signer.eq(&chain_keypair.into()) {
            return Err(LoopbackTicket);
        }
        if generate_channel_id(&signer, &chain_keypair.into()).ne(&ticket.channel_id) {
            return Err(InvalidTicketRecipient);
        }

        let vrf_params = derive_vrf_parameters(
            &ticket.get_hash(domain_separator).into(),
            chain_keypair,
            domain_separator.as_ref(),
        )?;

        Ok(Self {
            // new tickets are always untouched
            status: AcknowledgedTicketStatus::Untouched,
            ticket,
            response,
            vrf_params,
            signer,
        })
    }

    /// Does a verification of the acknowledged ticket, including:
    /// - ticket signature
    /// - ticket challenge (proof-of-relay)
    /// - VRF values (ticket redemption)
    pub fn verify(
        &self,
        issuer: &Address,
        recipient: &Address,
        domain_separator: &Hash,
    ) -> hopr_crypto_types::errors::Result<()> {
        if self.ticket.verify(issuer, domain_separator).is_err() {
            return Err(CryptoError::SignatureVerification);
        }

        if !self.ticket.challenge.eq(&self.response.to_challenge().into()) {
            return Err(CryptoError::InvalidChallenge);
        }

        if self
            .vrf_params
            .verify(
                recipient,
                &self.ticket.get_hash(domain_separator).into(),
                domain_separator.as_ref(),
            )
            .is_err()
        {
            return Err(CryptoError::InvalidVrfValues);
        }

        Ok(())
    }

    pub fn get_luck(&self, domain_separator: &Hash) -> CoreTypesResult<[u8; 7]> {
        let mut luck = [0u8; 7];

        if let Some(ref signature) = self.ticket.signature {
            luck.copy_from_slice(
                &Hash::create(&[
                    self.ticket.get_hash(domain_separator).as_ref(),
                    &self.vrf_params.v.as_uncompressed().as_bytes()[1..], // skip prefix
                    self.response.as_ref(),
                    &signature.as_ref(),
                ])
                .as_ref()[0..7],
            );
        } else {
            return Err(InvalidInputData(
                "Cannot compute ticket luck from unsigned ticket".into(),
            ));
        }

        // clone bytes
        Ok(luck)
    }

    pub fn is_winning_ticket(&self, domain_separator: &Hash) -> bool {
        let mut signed_ticket_luck = [0u8; 8];
        signed_ticket_luck[1..].copy_from_slice(&self.ticket.encoded_win_prob);

        let mut computed_ticket_luck = [0u8; 8];
        computed_ticket_luck[1..].copy_from_slice(&self.get_luck(domain_separator).expect("unsigned ticket"));

        u64::from_be_bytes(computed_ticket_luck) <= u64::from_be_bytes(signed_ticket_luck)
    }
}

impl Display for AcknowledgedTicket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "acknowledged {} in state '{}'", self.ticket, self.status)
    }
}

/// Wrapper for an unacknowledged ticket
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnacknowledgedTicket {
    pub ticket: Ticket,
    pub own_key: HalfKey,
    pub signer: Address,
}

impl UnacknowledgedTicket {
    pub fn new(ticket: Ticket, own_key: HalfKey, signer: Address) -> Self {
        Self {
            ticket,
            own_key,
            signer,
        }
    }

    pub fn get_challenge(&self) -> HalfKeyChallenge {
        self.own_key.to_challenge()
    }

    /// Verifies if signature on the embedded ticket using the embedded public key.
    pub fn verify_signature(&self, domain_separator: &Hash) -> hopr_crypto_types::errors::Result<()> {
        self.ticket.verify(&self.signer, domain_separator)
    }

    /// Verifies if the challenge on the embedded ticket matches the solution
    /// from the given acknowledgement and the embedded half key.
    pub fn verify_challenge(&self, acknowledgement: &HalfKey) -> hopr_crypto_types::errors::Result<()> {
        if self
            .ticket
            .challenge
            .eq(&self.get_response(acknowledgement)?.to_challenge().into())
        {
            Ok(())
        } else {
            Err(CryptoError::InvalidChallenge)
        }
    }

    pub fn get_response(&self, acknowledgement: &HalfKey) -> hopr_crypto_types::errors::Result<Response> {
        Response::from_half_keys(&self.own_key, acknowledgement)
    }

    /// Turn an unacknowledged ticket into an acknowledged ticket by adding
    /// VRF output (requires private key) and the received acknowledgement of the forwarded packet.
    pub fn acknowledge(
        self,
        acknowledgement: &HalfKey,
        chain_keypair: &ChainKeypair,
        domain_separator: &Hash,
    ) -> CoreTypesResult<AcknowledgedTicket> {
        let response = Response::from_half_keys(&self.own_key, acknowledgement)?;
        debug!("acknowledging ticket using response {}", response.to_hex());

        AcknowledgedTicket::new(self.ticket, response, self.signer, chain_keypair, domain_separator)
    }
}

/// Contains either unacknowledged ticket if we're waiting for the acknowledgement as a relayer
/// or information if we wait for the acknowledgement as a sender.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]

pub enum PendingAcknowledgement {
    /// We're waiting for acknowledgement as a sender
    WaitingAsSender,
    /// We're waiting for the acknowledgement as a relayer with a ticket
    WaitingAsRelayer(UnacknowledgedTicket),
}

#[cfg(test)]
pub mod test {
    use crate::{
        acknowledgement::{AcknowledgedTicket, Acknowledgement, PendingAcknowledgement, UnacknowledgedTicket},
        channels::Ticket,
    };
    use hex_literal::hex;
    use hopr_crypto_types::{
        keypairs::{ChainKeypair, Keypair, OffchainKeypair},
        types::{Challenge, CurvePoint, HalfKey, Hash, OffchainPublicKey, Response},
    };
    use hopr_primitive_types::prelude::UnitaryFloatOps;
    use hopr_primitive_types::{
        primitives::{Address, Balance, BalanceType, EthereumChallenge, U256},
        traits::BinarySerializable,
    };

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
            &Balance::new(
                price_per_packet.div_f64(win_prob).unwrap() * U256::from(path_pos),
                BalanceType::HOPR,
            ),
            U256::zero(),
            U256::one(),
            1.0f64,
            4u64.into(),
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
            ALICE.public().to_address(),
        );

        assert!(unacked_ticket.verify_signature(&Hash::default()).is_ok());
    }

    #[test]
    fn test_unacknowledged_ticket_challenge_response() {
        let hk1 = HalfKey::new(&hex!(
            "3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa"
        ));
        let hk2 = HalfKey::new(&hex!(
            "4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b"
        ));
        let cp1: CurvePoint = hk1.to_challenge().into();
        let cp2: CurvePoint = hk2.to_challenge().into();
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
        let hk1 = HalfKey::new(&hex!(
            "3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa"
        ));
        let hk2 = HalfKey::new(&hex!(
            "4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b"
        ));
        let cp1: CurvePoint = hk1.to_challenge().into();
        let cp2: CurvePoint = hk2.to_challenge().into();
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
        let response = Response::from_bytes(&hex!(
            "876a41ee5fb2d27ac14d8e8d552692149627c2f52330ba066f9e549aef762f73"
        ))
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
