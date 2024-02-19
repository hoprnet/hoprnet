use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use tracing::debug;

use crate::{
    acknowledgement::PendingAcknowledgement::{WaitingAsRelayer, WaitingAsSender},
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
    /// any operations with the deserialized acknowledgment will panic.
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

impl BinarySerializable for Acknowledgement {
    const SIZE: usize = OffchainSignature::SIZE + HalfKey::SIZE;

    fn from_bytes(data: &[u8]) -> hopr_primitive_types::errors::Result<Self> {
        let mut buf = data.to_vec();
        if data.len() == Self::SIZE {
            let ack_signature = OffchainSignature::from_bytes(buf.drain(..OffchainSignature::SIZE).as_ref())?;
            let ack_key_share = HalfKey::from_bytes(buf.drain(..HalfKey::SIZE).as_ref())?;
            Ok(Self {
                ack_signature,
                ack_key_share,
                validated: false,
            })
        } else {
            Err(GeneralError::ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        assert!(self.validated, "acknowledgement not validated");
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.ack_signature.to_bytes());
        ret.extend_from_slice(self.ack_key_share.as_ref());
        ret.into_boxed_slice()
    }
}

/// Status of the acknowledged ticket.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "PascalCase")]
pub enum AcknowledgedTicketStatus {
    /// The ticket is available for redeeming or aggregating
    #[default]
    Untouched,
    /// Ticket is currently being redeemed in and on-going redemption process
    BeingRedeemed,
    /// Ticket is currently being aggregated in and on-going aggregation process
    BeingAggregated,
}

/// Contains acknowledgment information and the respective ticket
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcknowledgedTicket {
    #[serde(default)]
    pub status: AcknowledgedTicketStatus,
    pub ticket: Ticket,
    pub response: Response,
    pub vrf_params: VrfParameters,
    pub signer: Address,
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
                    &self.vrf_params.v.serialize_uncompressed().as_bytes()[1..], // skip prefix
                    self.response.as_ref(),
                    &signature.to_bytes(),
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

impl BinarySerializable for AcknowledgedTicket {
    const SIZE: usize = Ticket::SIZE + Response::SIZE + VrfParameters::SIZE + Address::SIZE;

    fn from_bytes(data: &[u8]) -> hopr_primitive_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            let ticket = Ticket::from_bytes(&data[0..Ticket::SIZE])?;
            let response = Response::from_bytes(&data[Ticket::SIZE..Ticket::SIZE + Response::SIZE])?;
            let vrf_params = VrfParameters::from_bytes(
                &data[Ticket::SIZE + Response::SIZE..Ticket::SIZE + Response::SIZE + VrfParameters::SIZE],
            )?;
            let signer = Address::from_bytes(
                &data[Ticket::SIZE + Response::SIZE + VrfParameters::SIZE
                    ..Ticket::SIZE + Response::SIZE + VrfParameters::SIZE + Address::SIZE],
            )?;

            Ok(Self {
                // BinarySerializable is only used for over-the-wire encoding,
                // so acked tickets are always untouched
                status: AcknowledgedTicketStatus::Untouched,
                ticket,
                response,
                vrf_params,
                signer,
            })
        } else {
            Err(GeneralError::ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.ticket.to_bytes());
        ret.extend_from_slice(self.response.as_ref());
        ret.extend_from_slice(&self.vrf_params.to_bytes());
        ret.extend_from_slice(self.signer.as_ref());
        ret.into_boxed_slice()
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
    /// VRF output (requires private key) and the received acknowledgement
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

impl BinarySerializable for UnacknowledgedTicket {
    const SIZE: usize = Ticket::SIZE + HalfKey::SIZE + Address::SIZE;

    fn from_bytes(data: &[u8]) -> hopr_primitive_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            let ticket = Ticket::from_bytes(&data[0..Ticket::SIZE])?;
            let own_key = HalfKey::from_bytes(&data[Ticket::SIZE..Ticket::SIZE + HalfKey::SIZE])?;
            let signer =
                Address::from_bytes(&data[Ticket::SIZE + HalfKey::SIZE..Ticket::SIZE + HalfKey::SIZE + Address::SIZE])?;
            Ok(Self {
                ticket,
                own_key,
                signer,
            })
        } else {
            Err(GeneralError::ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.ticket.to_bytes());
        ret.extend_from_slice(self.own_key.as_ref());
        ret.extend_from_slice(self.signer.as_ref());
        ret.into_boxed_slice()
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

impl PendingAcknowledgement {
    const SENDER_PREFIX: u8 = 0;
    const RELAYER_PREFIX: u8 = 1;
}

impl BinarySerializable for PendingAcknowledgement {
    const SIZE: usize = 1;

    fn from_bytes(data: &[u8]) -> hopr_primitive_types::errors::Result<Self> {
        if data.len() >= Self::SIZE {
            match data[0] {
                Self::SENDER_PREFIX => Ok(WaitingAsSender),
                Self::RELAYER_PREFIX => Ok(WaitingAsRelayer(UnacknowledgedTicket::from_bytes(&data[1..])?)),
                _ => Err(GeneralError::ParseError),
            }
        } else {
            Err(GeneralError::ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        match &self {
            WaitingAsSender => ret.push(Self::SENDER_PREFIX),
            WaitingAsRelayer(unacknowledged) => {
                ret.push(Self::RELAYER_PREFIX);
                ret.extend_from_slice(&unacknowledged.to_bytes());
            }
        }
        ret.into_boxed_slice()
    }
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
    fn test_pending_ack_sender() {
        assert_eq!(
            PendingAcknowledgement::WaitingAsSender,
            PendingAcknowledgement::from_bytes(&PendingAcknowledgement::WaitingAsSender.to_bytes()).unwrap()
        );
    }

    #[test]
    fn test_acknowledgement() {
        let pk_2 = OffchainKeypair::from_secret(&hex!(
            "4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b"
        ))
        .unwrap();
        let pub_key_2 = OffchainPublicKey::from_privkey(pk_2.secret().as_ref()).unwrap();

        let ack_key = HalfKey::new(&hex!(
            "3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa"
        ));

        let mut ack1 = Acknowledgement::new(ack_key, &pk_2);
        assert!(ack1.validate(&pub_key_2));

        let mut ack2 = Acknowledgement::from_bytes(&ack1.to_bytes()).unwrap();
        assert!(ack2.validate(&pub_key_2));

        assert_eq!(ack1, ack2);
    }

    #[test]
    fn test_unacknowledged_ticket_serialize_deserialize() {
        let unacked_ticket = UnacknowledgedTicket::new(
            mock_ticket(&ALICE, &BOB.public().to_address(), None, None),
            HalfKey::default(),
            ALICE.public().to_address(),
        );

        assert_eq!(
            unacked_ticket,
            UnacknowledgedTicket::from_bytes(&unacked_ticket.to_bytes()).unwrap()
        );
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

    #[test]
    fn test_acknowledged_ticket_serialize_deserialize() {
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

        assert_eq!(
            acked_ticket,
            AcknowledgedTicket::from_bytes(&acked_ticket.to_bytes()).unwrap()
        );
    }
}
