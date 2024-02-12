use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::errors::CoreTypesError;
use crate::{
    acknowledgement::PendingAcknowledgement::{WaitingAsRelayer, WaitingAsSender},
    channels::Ticket,
    errors::Result as CoreTypesResult,
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
            ack_signature: OffchainSignature::sign_message(&ack_key_share.to_bytes(), node_keypair),
            ack_key_share,
            validated: true,
        }
    }

    /// Validates the acknowledgement. Must be called immediately after deserialization or otherwise
    /// any operations with the deserialized acknowledgment will panic.
    pub fn validate(&mut self, sender_node_key: &OffchainPublicKey) -> bool {
        self.validated = self
            .ack_signature
            .verify_message(&self.ack_key_share.to_bytes(), sender_node_key);

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
        ret.extend_from_slice(&self.ack_key_share.to_bytes());
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

pub fn validate_acknowledged_ticket(acked_ticket: &AcknowledgedTicket) -> CoreTypesResult<()> {
    if !acked_ticket
        .ticket
        .challenge
        .eq(&acked_ticket.response.to_challenge().into())
    {
        return Err(CoreTypesError::InvalidInputData(
            "Computed challenged does not match signed challenge".into(),
        ));
    }

    Ok(())
}

/// Contains acknowledgment information and the respective ticket
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcknowledgedTicket {
    #[serde(default)]
    pub status: AcknowledgedTicketStatus,
    /// ticket data, including signature and expected payout
    pub ticket: Ticket,
    /// Proof-Of-Relay response to challenge stated in ticket
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
    /// Creates an acknowledged ticket. Once done, we can check
    /// if it is a win and if so, it can be redeemed on-chain or
    /// be sent to the ticket issuer to have it aggregated.
    pub fn new(ticket: Ticket, response: Response) -> AcknowledgedTicket {
        Self {
            // new tickets are always untouched
            status: AcknowledgedTicketStatus::Untouched,
            ticket,
            response,
        }
    }

    pub fn is_winning_ticket(&self, chain_key: &ChainKeypair, domain_separator: &Hash) -> CoreTypesResult<()> {
        let vrf_params = self.ticket.get_vrf_values(&chain_key, domain_separator)?;
        if self
            .ticket
            .is_winning_ticket(vrf_params, &self.response, domain_separator)
        {
            Ok(())
        } else {
            Err(CoreTypesError::GeneralError(GeneralError::NonSpecificError(
                "Ticket is not a win".into(),
            )))
        }
    }
}

impl Display for AcknowledgedTicket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "acknowledged {} in state '{}'", self.ticket, self.status)
    }
}

/// Returs Ok(()) if the ticket is considered a win by the smart
/// contract and is thus considered valid.
pub fn validate_provable_winning_ticket(
    maybe_winning_ticket: &ProvableWinningTicket,
    destination: &Address,
    domain_separator: &Hash,
) -> CoreTypesResult<()> {
    if maybe_winning_ticket.ticket.challenge != maybe_winning_ticket.response.to_challenge().into() {
        return Err(CoreTypesError::InvalidInputData(
            "Response does not fulfill signed challenge".into(),
        ));
    }

    maybe_winning_ticket
        .vrf_params
        .verify(
            destination,
            &maybe_winning_ticket.ticket.get_hash(domain_separator).into(),
            domain_separator.as_slice(),
        )
        .map_err(|_| CoreTypesError::InvalidInputData("VRF values are invalid".into()))?;

    if !maybe_winning_ticket.ticket.is_winning_ticket(
        &maybe_winning_ticket.vrf_params,
        &maybe_winning_ticket.response,
        domain_separator,
    ) {
        return Err(CoreTypesError::InvalidInputData("Ticket is not a win".into()));
    }

    Ok(())
}

/// Data structure that can only hold winning tickets.
/// Used for aggregating tickets.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvableWinningTicket {
    /// ticket data, including signature and expected payout
    pub ticket: Ticket,
    /// Proof-Of-Relay response to challenge stated in ticket
    pub response: Response,
    pub vrf_params: VrfParameters,
    pub signer: Address,
}

impl ProvableWinningTicket {
    /// Consumes an acknowledged ticket and stores all values that are
    /// necessary to provde that it is a win
    pub fn from_acked_ticket(
        acked_ticket: AcknowledgedTicket,
        chain_key: &ChainKeypair,
        domain_separator: &Hash,
    ) -> CoreTypesResult<Self> {
        let vrf_params = acked_ticket
            .ticket
            .get_vrf_values(chain_key, domain_separator)?
            .to_owned();
        let signer = acked_ticket.ticket.recover_signer(domain_separator)?;

        Ok(Self {
            ticket: acked_ticket.ticket,
            response: acked_ticket.response,
            vrf_params,
            signer: signer.to_address(),
        })
    }
}

impl PartialOrd for ProvableWinningTicket {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProvableWinningTicket {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ticket.cmp(&other.ticket)
    }
}

impl Display for ProvableWinningTicket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "winning ticket: {}", self.ticket)
    }
}

/// over-the-wire format to be deprecated soon
impl BinarySerializable for ProvableWinningTicket {
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
        ret.extend_from_slice(&self.response.to_bytes());
        ret.extend_from_slice(&self.vrf_params.to_bytes());
        ret.extend_from_slice(&self.signer.to_bytes());
        ret.into_boxed_slice()
    }
}

pub fn validate_unacknowledged_ticket(
    _unacked_ticket: &UnacknowledgedTicket,
    _hint: Hash,
) -> hopr_crypto_types::errors::Result<()> {
    // TODO: validate hint

    Ok(())
}
/// Wrapper for an unacknowledged ticket
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnacknowledgedTicket {
    pub ticket: Ticket,
    pub own_key: HalfKey,
}

impl UnacknowledgedTicket {
    pub fn new(ticket: Ticket, own_key: HalfKey) -> Self {
        Self { ticket, own_key }
    }

    pub fn get_challenge(&self) -> HalfKeyChallenge {
        self.own_key.to_challenge()
    }

    /// Turn an unacknowledged ticket into an acknowledged ticket by combining
    /// both Proof-Of-Relay key halves
    pub fn acknowledge(self, acknowledgement: &HalfKey) -> CoreTypesResult<AcknowledgedTicket> {
        let response = Response::from_half_keys(&self.own_key, acknowledgement)?;
        debug!("acknowledging ticket using response {}", response.to_hex());

        Ok(AcknowledgedTicket::new(self.ticket, response))
    }
}

impl BinarySerializable for UnacknowledgedTicket {
    const SIZE: usize = Ticket::SIZE + HalfKey::SIZE;

    fn from_bytes(data: &[u8]) -> hopr_primitive_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            let ticket = Ticket::from_bytes(&data[0..Ticket::SIZE])?;
            let own_key = HalfKey::from_bytes(&data[Ticket::SIZE..Ticket::SIZE + HalfKey::SIZE])?;

            Ok(Self { ticket, own_key })
        } else {
            Err(GeneralError::ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.ticket.to_bytes());
        ret.extend_from_slice(&self.own_key.to_bytes());
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
        acknowledgement::{
            AcknowledgedTicket, Acknowledgement, PendingAcknowledgement, ProvableWinningTicket, UnacknowledgedTicket,
        },
        channels::Ticket,
        prelude::validate_ticket,
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
        static ref ALICE_ADDR: Address = ALICE.public().to_address();
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).unwrap();
        static ref BOB_ADDR: Address = BOB.public().to_address();

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
            0.5f64,
            4u64.into(),
            challenge.unwrap_or_default(),
            pk,
            &domain_separator.unwrap_or_default(),
        )
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
        let unacked_ticket = UnacknowledgedTicket::new(mock_ticket(&ALICE, &BOB_ADDR, None, None), HalfKey::default());

        assert!(super::validate_unacknowledged_ticket(&unacked_ticket, Hash::default()).is_ok());

        assert_eq!(
            unacked_ticket,
            UnacknowledgedTicket::from_bytes(&unacked_ticket.to_bytes()).unwrap()
        );
    }

    #[test]
    fn test_unacknowledged_ticket_sign_verify() {
        let unacked_ticket = UnacknowledgedTicket::new(mock_ticket(&ALICE, &BOB_ADDR, None, None), HalfKey::default());

        assert!(validate_ticket(&unacked_ticket.ticket, &BOB_ADDR, &Hash::default()).is_ok());
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
            &BOB_ADDR,
            None,
            Some(Challenge::from(cp_sum).to_ethereum_challenge()),
        );

        let unacked_ticket = UnacknowledgedTicket::new(ticket, hk1);

        let acked_ticket = unacked_ticket.acknowledge(&hk2).unwrap();

        assert!(super::validate_acknowledged_ticket(&acked_ticket).is_ok());
    }

    #[test]
    fn test_acknowledged_ticket_bad_examples() {
        let response = Response::from_bytes(&hex!(
            "876a41ee5fb2d27ac14d8e8d552692149627c2f52330ba066f9e549aef762f73"
        ))
        .unwrap();

        let ticket = mock_ticket(&ALICE, &BOB_ADDR, None, Some(response.to_challenge().into()));

        assert!(validate_ticket(&ticket, &BOB_ADDR, &Hash::default()).is_ok());

        let acked_ticket = AcknowledgedTicket::new(
            ticket,
            Response::from_bytes(&hex!(
                "2bce05b81349033c920c8bc61242f650e176f20fe518237d791720454b02bfd5"
            ))
            .unwrap(),
        );

        assert!(super::validate_acknowledged_ticket(&acked_ticket).is_err());
    }

    #[test]
    fn test_acknowledged_ticket_serialize_deserialize() {
        let response = Response::from_bytes(&hex!(
            "876a41ee5fb2d27ac14d8e8d552692149627c2f52330ba066f9e549aef762f73"
        ))
        .unwrap();

        let ticket = mock_ticket(&ALICE, &BOB_ADDR, None, Some(response.to_challenge().into()));

        assert!(validate_ticket(&ticket, &BOB_ADDR, &Hash::default()).is_ok());

        let acked_ticket = AcknowledgedTicket::new(ticket, response);

        assert!(super::validate_acknowledged_ticket(&acked_ticket).is_ok());

        let mut deserialized_ticket =
            bincode::deserialize::<AcknowledgedTicket>(&bincode::serialize(&acked_ticket).unwrap()).unwrap();
        assert_eq!(acked_ticket, deserialized_ticket);

        deserialized_ticket.status = super::AcknowledgedTicketStatus::BeingAggregated;

        assert_eq!(
            deserialized_ticket,
            bincode::deserialize(&bincode::serialize(&deserialized_ticket).unwrap()).unwrap()
        );

        assert!(validate_ticket(&deserialized_ticket.ticket, &BOB_ADDR, &Hash::default()).is_ok());
        assert!(super::validate_acknowledged_ticket(&deserialized_ticket).is_ok());
    }

    #[test]
    fn test_provable_winning_ticket() {
        let response = Response::from_bytes(&hex!(
            "c598332ab309e84a40d71a605db03a70876ebfa574191eff1787921ae90f1624"
        ))
        .unwrap();

        let ticket = mock_ticket(&ALICE, &BOB_ADDR, None, Some(response.to_challenge().into()));

        assert!(validate_ticket(&ticket, &BOB_ADDR, &Hash::default()).is_ok());

        let acked_ticket = AcknowledgedTicket::new(ticket, response);

        assert!(super::validate_acknowledged_ticket(&acked_ticket).is_ok());

        let winning_ticket = ProvableWinningTicket::from_acked_ticket(acked_ticket, &BOB, &Hash::default()).unwrap();

        assert!(super::validate_provable_winning_ticket(&winning_ticket, &BOB_ADDR, &Hash::default()).is_ok());
    }

    #[test]
    fn test_provable_winning_ticket_wrong_creator() {
        let response = Response::from_bytes(&hex!(
            "c598332ab309e84a40d71a605db03a70876ebfa574191eff1787921ae90f1624"
        ))
        .unwrap();

        let ticket = mock_ticket(&ALICE, &BOB_ADDR, None, Some(response.to_challenge().into()));

        assert!(validate_ticket(&ticket, &BOB_ADDR, &Hash::default()).is_ok());

        let acked_ticket = AcknowledgedTicket::new(ticket, response);

        assert!(super::validate_acknowledged_ticket(&acked_ticket).is_ok());

        let winning_ticket = ProvableWinningTicket::from_acked_ticket(acked_ticket, &BOB, &Hash::default()).unwrap();

        // Wrong creator address
        assert!(super::validate_provable_winning_ticket(&winning_ticket, &ALICE_ADDR, &Hash::default()).is_err());
    }

    #[test]
    fn test_provable_winning_ticket_losing_ticket() {
        let response = Response::from_bytes(&hex!(
            "2bce05b81349033c920c8bc61242f650e176f20fe518237d791720454b02bfd5"
        ))
        .unwrap();

        let ticket = mock_ticket(&ALICE, &BOB_ADDR, None, Some(response.to_challenge().into()));

        assert!(validate_ticket(&ticket, &BOB_ADDR, &Hash::default()).is_ok());

        let acked_ticket = AcknowledgedTicket::new(ticket, response);

        assert!(super::validate_acknowledged_ticket(&acked_ticket).is_ok());

        let winning_ticket = ProvableWinningTicket::from_acked_ticket(acked_ticket, &BOB, &Hash::default()).unwrap();

        // Probabilistic, ticket is not a win
        assert!(super::validate_provable_winning_ticket(&winning_ticket, &BOB_ADDR, &Hash::default()).is_err());
    }
}
