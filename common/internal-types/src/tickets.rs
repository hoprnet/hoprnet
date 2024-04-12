use bindings::hopr_channels::RedeemTicketCall;
use ethers::contract::EthCall;
use hex_literal::hex;
use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use tracing::{debug, error};

use crate::errors::CoreTypesError;
use crate::prelude::generate_channel_id;
use crate::prelude::CoreTypesError::InvalidInputData;
use crate::{channels, errors};

/// Size-optimized encoding of the ticket, used for both,
/// network transfer and in the smart contract.
const ENCODED_TICKET_LENGTH: usize = 64;

/// Custom float to integer encoding used in the integer-only
/// Ethereum Virtual Machine (EVM). Chosen to be easily
/// convertible to IEEE754 double-precision and vice versa
const ENCODED_WIN_PROB_LENGTH: usize = 7;

/// Winning probability encoded in 7-byte representation
pub type EncodedWinProb = [u8; ENCODED_WIN_PROB_LENGTH];

/// Encodes 100% winning probability
const ALWAYS_WINNING: EncodedWinProb = hex!("ffffffffffffff");

/// Encodes 0% winning probability
const NEVER_WINNING: EncodedWinProb = hex!("00000000000000");

/// Helper function to checks if the given ticket values belong to a winning ticket.
pub(crate) fn check_ticket_win(
    ticket_hash: &Hash,
    ticket_signature: &Signature,
    win_prob: &EncodedWinProb,
    response: &Response,
    vrf_params: &VrfParameters,
) -> bool {
    // Signed winning probability
    let mut signed_ticket_luck = [0u8; 8];
    signed_ticket_luck[1..].copy_from_slice(win_prob);

    // Computed winning probability
    let mut computed_ticket_luck = [0u8; 8];
    computed_ticket_luck[1..].copy_from_slice(
        &Hash::create(&[
            ticket_hash.as_ref(),
            &vrf_params.v.as_uncompressed().as_bytes()[1..], // skip prefix
            response.as_ref(),
            ticket_signature.as_ref(),
        ])
        .as_ref()[0..7],
    );

    u64::from_be_bytes(computed_ticket_luck) <= u64::from_be_bytes(signed_ticket_luck)
}

#[derive(Debug, Clone, smart_default::SmartDefault)]
pub struct TicketBuilder {
    channel_id: Option<Hash>,
    amount: Option<Balance>,
    index: Option<u64>,
    #[default = 1]
    index_offset: u32,
    epoch: Option<u32>,
    #[default = 1.0]
    win_prob: f64,
    challenge: Option<EthereumChallenge>,
}

impl TicketBuilder {
    pub fn zero_hop() -> Self {
        Self {
            index: Some(0),
            index_offset: 0,
            win_prob: 0.0,
            epoch: Some(0),
            ..Default::default()
        }
    }

    pub fn direction(mut self, source: &Address, destination: &Address) -> Self {
        self.channel_id = Some(generate_channel_id(source, destination));
        self
    }

    pub fn id(mut self, channel_id: Hash) -> Self {
        self.channel_id = Some(channel_id);
        self
    }

    pub fn amount<T: Into<U256>>(mut self, amount: T) -> Self {
        self.amount = Some(BalanceType::HOPR.balance(amount));
        self
    }

    pub fn index(mut self, index: u64) -> Self {
        self.index = Some(index);
        self
    }

    pub fn index_offset(mut self, index_offset: u32) -> Self {
        self.index_offset = index_offset;
        self
    }

    pub fn channel_epoch(mut self, channel_epoch: u32) -> Self {
        self.epoch = Some(channel_epoch);
        self
    }

    pub fn win_prob(mut self, win_prob: f64) -> Self {
        self.win_prob = win_prob;
        self
    }

    pub fn challenge(mut self, challenge: EthereumChallenge) -> Self {
        self.challenge = Some(challenge);
        self
    }

    pub fn build(self) -> errors::Result<Ticket> {
        if self.amount.balance_type().ne(&BalanceType::HOPR) {
            return Err(CoreTypesError::InvalidInputData(
                "Tickets can only have HOPR balance".into(),
            ));
        }

        if amount.amount().ge(&10_u128.pow(25).into()) {
            return Err(CoreTypesError::InvalidInputData(
                "Tickets may not have more than 1% of total supply".into(),
            ));
        }

        if index > (1_u64 << 48) {
            return Err(CoreTypesError::InvalidInputData(
                "Cannot hold ticket indices larger than 2^48".into(),
            ));
        }

        if channel_epoch > (1_u32 << 24) {
            return Err(CoreTypesError::InvalidInputData(
                "Cannot hold channel epoch larger than 2^24".into(),
            ));
        }

        if !(0.0..=1.0).contains(&win_prob) {
            return Err(CoreTypesError::InvalidInputData(
                "Winning probability must be between 0 and 1".into(),
            ));
        }

        Ok(Ticket {
            channel_id: self.channel_id.ok_or(InvalidInputData("missing channel id".into()))?,
            amount: self.amount.ok_or(InvalidInputData("missing amount".into()))?,
            index: self.index.ok_or(InvalidInputData("missing index".into()))?,
            index_offset: self.index_offset,
            encoded_win_prob: self.win_prob,
            channel_epoch: self.epoch.ok_or(InvalidInputData("missing channel epoch".into()))?,
            challenge: self
                .challenge
                .ok_or(InvalidInputData("missing ticket challenge".into()))?,
            signature: None,
        })
    }

    pub fn build_signed(self, signature: Signature) -> errors::Result<Ticket> {
        let mut ret = self.build()?;
        ret.signature = Some(signature);
        Ok(ret)
    }
}

/// Contains the overall description of a ticket with a signature.
///
/// This structure is not considered [verified](VerifiedTicket), unless
/// the [Ticket::verify] or [Ticket::sign] methods are called.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ticket {
    /// Channel ID.
    /// See [generate_channel_id](channels::generate_channel_id) for how this value is generated.
    pub channel_id: Hash,
    /// Amount of HOPR tokens this ticket is worth.
    /// Always between 0 and 2^92.
    pub amount: Balance, // 92 bits
    /// Ticket index.
    /// Always between 0 and 2^48.
    pub index: u64, // 48 bits
    /// Ticket index offset.
    /// Always between 1 and 2^32.
    /// For normal tickets this is always equal to 1, for aggregated this is always > 1.
    pub index_offset: u32, // 32 bits
    /// Encoded winning probability represented via 56-bit number.
    pub encoded_win_prob: EncodedWinProb, // 56 bits
    /// Epoch of the channel this ticket belongs to.
    /// Always between 0 and 2^24.
    pub channel_epoch: u32, // 24 bits
    /// Represent the Proof of Relay challenge encoded as Ethereum address.
    pub challenge: EthereumChallenge,
    /// ECDSA secp256k1 signature of all the above values.
    pub signature: Option<Signature>,
}

impl PartialOrd for Ticket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ticket {
    fn cmp(&self, other: &Self) -> Ordering {
        // Ordering:
        // [channel_id][channel_epoch][ticket_index]
        match self.channel_id.cmp(&other.channel_id) {
            Ordering::Equal => match self.channel_epoch.cmp(&other.channel_epoch) {
                Ordering::Equal => self.index.cmp(&other.index),
                Ordering::Greater => Ordering::Greater,
                Ordering::Less => Ordering::Less,
            },
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
        }
    }
}

impl Display for Ticket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ticket #{}, offset {}, epoch {} in channel {}",
            self.index, self.index_offset, self.channel_epoch, self.channel_id
        )
    }
}

impl Ticket {
    fn encode_without_signature(&self) -> [u8; Self::SIZE - Signature::SIZE] {
        let mut ret = [0u8; Self::SIZE - Signature::SIZE];
        let mut offset = 0;

        ret[offset..offset + Hash::SIZE].copy_from_slice(self.channel_id.as_ref());
        offset += Hash::SIZE;

        // There are only 2^96 HOPR tokens
        ret[offset..offset + 12].copy_from_slice(&self.amount.amount().to_be_bytes()[20..32]);
        offset += 12;

        // Ticket index can go only up to 2^48
        ret[offset..offset + 6].copy_from_slice(&self.index.to_be_bytes()[2..8]);
        offset += 6;

        ret[offset..offset + 4].copy_from_slice(&self.index_offset.to_be_bytes());
        offset += 4;

        // Channel epoch can go only up to 2^24
        ret[offset..offset + 3].copy_from_slice(&self.channel_epoch.to_be_bytes()[1..4]);
        offset += 3;

        ret[offset..offset + ENCODED_WIN_PROB_LENGTH].copy_from_slice(&self.encoded_win_prob);
        offset += ENCODED_WIN_PROB_LENGTH;

        ret[offset..offset + EthereumChallenge::SIZE].copy_from_slice(self.challenge.as_ref());

        ret
    }

    /// Computes Ethereum signature hash of the ticket,
    /// must be equal to on-chain computation
    pub fn get_hash(&self, domain_separator: &Hash) -> Hash {
        let ticket_hash = Hash::create(&[self.encode_without_signature().as_ref()]); // cannot fail
        let hash_struct = Hash::create(&[&RedeemTicketCall::selector(), &[0u8; 28], ticket_hash.as_ref()]);
        Hash::create(&[&hex!("1901"), domain_separator.as_ref(), hash_struct.as_ref()])
    }

    /// Signs the ticket using the given private key, turning this ticket into [VerifiedTicket].
    pub fn sign(mut self, signing_key: &ChainKeypair, domain_separator: &Hash) -> VerifiedTicket {
        let ticket_hash = self.get_hash(domain_separator);
        self.signature = Some(Signature::sign_hash(ticket_hash.as_ref(), signing_key));
        VerifiedTicket(self, ticket_hash, signing_key.public().to_address())
    }

    /// Verifies the signature of this ticket, turning this ticket into `VerifiedTicket`.
    /// If the verification fails, `Self` is returned in the error.
    ///
    /// This is done by recovering the signer from the signature and verifying that it matches
    /// the given `issuer` argument. This is possible due this specific instantiation of the ECDSA
    /// over the secp256k1 curve.
    /// The operation can fail if a public key cannot be recovered from the ticket signature.
    pub fn verify(self, issuer: &Address, domain_separator: &Hash) -> Result<VerifiedTicket, Ticket> {
        let ticket_hash = self.get_hash(domain_separator);

        if let Some(signature) = &self.signature {
            match PublicKey::from_signature_hash(ticket_hash.as_ref(), signature) {
                Ok(pk) if pk.to_address().eq(issuer) => Ok(VerifiedTicket(self, ticket_hash, *issuer)),
                Err(e) => {
                    error!("failed to verify ticket signature: {e}");
                    Err(self)
                }
                _ => Err(self),
            }
        } else {
            Err(self)
        }
    }

    /// Returns true if this ticket aggregates multiple tickets.
    pub fn is_aggregated(&self) -> bool {
        // Aggregated tickets have always an index offset > 1
        self.index_offset > 1
    }

    /// Returns the encoded winning probability of the ticket
    pub fn win_prob(&self) -> f64 {
        win_prob_to_f64(&self.encoded_win_prob)
    }
}

impl From<Ticket> for [u8; TICKET_SIZE] {
    fn from(value: Ticket) -> Self {
        let mut ret = [0u8; TICKET_SIZE];
        ret[0..Ticket::SIZE - Signature::SIZE].copy_from_slice(value.encode_without_signature().as_ref());
        ret[Ticket::SIZE - Signature::SIZE..].copy_from_slice(
            value
                .signature
                .expect("cannot serialize ticket without signature")
                .as_ref(),
        );
        ret
    }
}

impl TryFrom<&[u8]> for Ticket {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        if value.len() == Self::SIZE {
            // TODO: not necessary to transmit over the wire, only the counterparty is sufficient
            let channel_id = Hash::try_from(&value[0..32])?;
            let mut amount = [0u8; 32];
            amount[20..32].copy_from_slice(&value[Hash::SIZE..Hash::SIZE + 12]);

            let mut index = [0u8; 8];
            index[2..8].copy_from_slice(&value[Hash::SIZE + 12..Hash::SIZE + 12 + 6]);

            let mut index_offset = [0u8; 4];
            index_offset.copy_from_slice(&value[Hash::SIZE + 12 + 6..Hash::SIZE + 12 + 6 + 4]);

            let mut channel_epoch = [0u8; 4];
            channel_epoch[1..4].copy_from_slice(&value[Hash::SIZE + 12 + 6 + 4..Hash::SIZE + 12 + 6 + 4 + 3]);

            let mut encoded_win_prob = [0u8; 7];
            encoded_win_prob.copy_from_slice(&value[Hash::SIZE + 12 + 6 + 4 + 3..Hash::SIZE + 12 + 6 + 4 + 3 + 7]);

            let challenge = EthereumChallenge::try_from(
                &value[ENCODED_TICKET_LENGTH..ENCODED_TICKET_LENGTH + EthereumChallenge::SIZE],
            )?;

            let signature = Signature::try_from(
                &value[ENCODED_TICKET_LENGTH + EthereumChallenge::SIZE
                    ..ENCODED_TICKET_LENGTH + EthereumChallenge::SIZE + Signature::SIZE],
            )?;

            // Validate the boundaries of the parsed values
            Ok(TicketBuilder::default()
                .id(channel_id)
                .amount(amount)?
                .index(u64::from_be_bytes(index))?
                .index_offset(u32::from_be_bytes(index_offset))?
                .channel_epoch(u32::from_be_bytes(channel_epoch))?
                .win_prob_encoded(encoded_win_prob)
                .challenge(challenge)
                .build_signed(signature)?)
        } else {
            Err(hopr_primitive_types::errors::GeneralError::ParseError)
        }
    }
}

const TICKET_SIZE: usize = ENCODED_TICKET_LENGTH + EthereumChallenge::SIZE + Signature::SIZE;

impl BytesEncodable<TICKET_SIZE> for Ticket {}

/// Holds a ticket that has been already verified.
/// This structure guarantees that [`Ticket::get_hash()`] of [`VerifiedTicket::verified_ticket()`]
/// is always equal to [`VerifiedTicket::verified_hash`]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedTicket(Ticket, Hash, Address);

impl VerifiedTicket {
    /// Creates a new [VerifiedTicket] by creating a new [Ticket] with
    /// the given raw [EthereumChallenge] and signing it with the given `signing_key`.
    #[allow(clippy::too_many_arguments)] // TODO: Refactor using future Ticket's builder pattern
    pub fn new(
        counterparty: &Address,
        amount: Balance,
        index: u64,
        index_offset: u32,
        win_prob: f64,
        channel_epoch: u32,
        challenge: EthereumChallenge,
        signing_key: &ChainKeypair,
        domain_separator: &Hash,
    ) -> errors::Result<Self> {
        let mut ret = Ticket::new_partial(
            &signing_key.public().to_address(),
            counterparty,
            amount,
            index,
            index_offset,
            win_prob,
            channel_epoch,
        )?;
        ret.challenge = challenge;
        Ok(ret.sign(signing_key, domain_separator))
    }

    #[allow(clippy::too_many_arguments)] // TODO: Refactor using future Ticket's builder pattern
    pub fn new_trusted(
        channel_id: Hash,
        amount: Balance,
        index: u64,
        index_offset: u32,
        win_prob: f64,
        channel_epoch: u32,
        challenge: EthereumChallenge,
        signature: Signature,
        hash: Hash,
    ) -> errors::Result<Self> {
        let mut ret = Ticket::new_partial_with_id(channel_id, amount, index, index_offset, win_prob, channel_epoch)?;
        ret.challenge = challenge;
        ret.signature = Some(signature);

        let issuer = PublicKey::from_signature_hash(hash.as_ref(), &signature)?.to_address();

        Ok(Self(ret, hash, issuer))
    }

    /// Convenience method for creating a zero-hop ticket
    pub fn new_zero_hop(
        destination: &Address,
        challenge: EthereumChallenge,
        private_key: &ChainKeypair,
        domain_separator: &Hash,
    ) -> Self {
        let mut ticket = Ticket::new_zero_hop(&private_key.public().to_address(), destination);
        ticket.challenge = challenge;
        ticket.sign(private_key, domain_separator)
    }

    /// Returns the verified encoded winning probability of the ticket
    pub fn win_prob(&self) -> f64 {
        self.0.win_prob()
    }

    /// Checks if this ticket is considered a win.
    /// Requires access to the private key to compute the VRF values.
    ///
    /// Computes the ticket's luck value and compares it against the
    /// ticket's probability. If luck <= probability, the ticket is
    /// considered a win.
    ///
    /// ## Ticket luck value
    /// This ticket's `luck value` is the first 7 bytes of Keccak256 hash
    /// of the concatenation of ticket's hash, VRF's encoded `v` value,
    /// PoR response and the ticket's signature.
    ///
    /// ## Winning probability
    /// Each ticket specifies a probability, given as an integer in
    /// [0, 2^56 - 1] where 0 -> 0% and 2^56 - 1 -> 100% win
    /// probability. If the ticket's luck value is greater than
    /// the stated probability, it is considered a winning ticket.
    pub fn is_winning(&self, response: &Response, chain_keypair: &ChainKeypair, domain_separator: &Hash) -> bool {
        if let Ok(vrf_params) = derive_vrf_parameters(self.1, chain_keypair, domain_separator.as_ref()) {
            check_ticket_win(
                &self.1,
                self.0
                    .signature
                    .as_ref()
                    .expect("verified ticket have always a signature"),
                &self.0.encoded_win_prob,
                response,
                &vrf_params,
            )
        } else {
            error!("cannot derive vrf parameters for {self}");
            false
        }
    }

    /// Based on the price of this ticket, determines the path position (hop number) this ticket
    /// relates to.
    ///
    /// This is done by first determining the amount of tokens the ticket is worth
    /// if it were a win and redeemed on-chain.
    ///
    /// Does not support path lengths greater than 255.
    pub fn get_path_position(&self, price_per_packet: U256) -> errors::Result<u8> {
        let mut win_prob = [0u8; 8];
        win_prob[1..].copy_from_slice(&self.0.encoded_win_prob);

        // Add + 1 to project interval [0x00ffffffffffff, 0x00000000000000] to [0x00000000000001, 0x01000000000000]
        // Add + 1 to "round to next integer"
        let win_prob = (u64::from_be_bytes(win_prob) >> 4) + 1 + 1;

        let expected_payout = (self.0.amount.amount() * U256::from(win_prob)) >> U256::from(52_u64);

        (expected_payout / price_per_packet)
            .as_u64()
            .try_into() // convert to u8
            .map_err(|_| {
                CoreTypesError::ArithmeticError(format!("Cannot convert {} to u8", price_per_packet / expected_payout))
            })
    }

    /// Ticket with already verified signature.
    pub fn verified_ticket(&self) -> &Ticket {
        &self.0
    }
    /// Fixed ticket hash that is guaranteed to be equal to
    /// [`Ticket::get_hash`] of [`VerifiedTicket::verified_ticket`].
    pub fn verified_hash(&self) -> &Hash {
        &self.1
    }

    /// Verified issuer of the ticket.
    /// The returned address is guaranteed to be equal to the signer
    /// recovered from the [`VerifiedTicket::verified_ticket`]'s signature.
    pub fn verified_issuer(&self) -> &Address {
        &self.2
    }

    /// Shorthand to retrieve reference to the verified ticket signature
    pub fn verified_signature(&self) -> &Signature {
        self.0
            .signature
            .as_ref()
            .expect("verified ticket always has a signature")
    }

    /// Deconstructs self back into the unverified [Ticket].
    pub fn leak(self) -> Ticket {
        self.0
    }

    /// Creates new unacknowledged ticket from the [VerifiedTicket],
    /// given our own part of the PoR challenge.
    pub fn into_unacknowledged(self, own_key: HalfKey) -> UnacknowledgedTicket {
        UnacknowledgedTicket { ticket: self, own_key }
    }

    /// Shorthand to acknowledge the ticket if the response is already known.
    /// This is used upon receiving an aggregated ticket.
    pub fn into_acknowledged(self, response: Response) -> AcknowledgedTicket {
        AcknowledgedTicket {
            status: AcknowledgedTicketStatus::Untouched,
            ticket: self,
            response,
        }
    }
}

impl Display for VerifiedTicket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "verified {}", self.0)
    }
}

impl PartialOrd for VerifiedTicket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VerifiedTicket {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

/// Decodes [0x00000000000000, 0xffffffffffffff] to [0.0f64, 1.0f64]
/// See [ALWAYS_WINNING] and [NEVER_WINNING].
pub fn win_prob_to_f64(encoded_win_prob: &EncodedWinProb) -> f64 {
    if encoded_win_prob.eq(&NEVER_WINNING) {
        return 0.0;
    }

    if encoded_win_prob.eq(&ALWAYS_WINNING) {
        return 1.0;
    }

    let mut tmp = [0u8; 8];
    tmp[1..].copy_from_slice(encoded_win_prob);

    let tmp = u64::from_be_bytes(tmp);

    // project interval [0x0fffffffffffff, 0x0000000000000f] to [0x00000000000010, 0x10000000000000]
    let significand: u64 = tmp + 1;

    f64::from_bits(1023u64 << 52 | significand >> 4) - 1.0
}

/// Encodes [0.0f64, 1.0f64] to [0x00000000000000, 0xffffffffffffff]
/// See [ALWAYS_WINNING] and [NEVER_WINNING].
pub fn f64_to_win_prob(win_prob: f64) -> errors::Result<EncodedWinProb> {
    if !(0.0..=1.0).contains(&win_prob) {
        return Err(CoreTypesError::InvalidInputData(
            "Winning probability must be in [0.0, 1.0]".into(),
        ));
    }

    if win_prob == 0.0 {
        return Ok(NEVER_WINNING);
    }

    if win_prob == 1.0 {
        return Ok(ALWAYS_WINNING);
    }

    let tmp: u64 = (win_prob + 1.0).to_bits();

    // // clear sign and exponent
    let significand: u64 = tmp & 0x000fffffffffffffu64;

    // project interval [0x10000000000000, 0x00000000000010] to [0x0000000000000f, 0x0fffffffffffff]
    let encoded = ((significand - 1) << 4) | 0x000000000000000fu64;

    let mut res = [0u8; 7];
    res.copy_from_slice(&encoded.to_be_bytes()[1..]);

    Ok(res)
}

/// Represents a [VerifiedTicket] with an unknown other part of the [HalfKey].
/// Once the other [HalfKey] is known (forming a [Response]),
/// it can be [acknowledged](UnacknowledgedTicket::acknowledge).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnacknowledgedTicket {
    pub ticket: VerifiedTicket,
    pub(crate) own_key: HalfKey,
}

impl UnacknowledgedTicket {
    /// Convenience method to retrieve a reference to the underlying verified [Ticket].
    #[inline]
    pub fn verified_ticket(&self) -> &Ticket {
        self.ticket.verified_ticket()
    }

    /// Verifies that the given acknowledgement solves this ticket's challenge and then
    /// turns this unacknowledged ticket into an acknowledged ticket by adding
    /// the received acknowledgement of the forwarded packet.
    pub fn acknowledge(self, acknowledgement: &HalfKey) -> crate::errors::Result<AcknowledgedTicket> {
        let response = Response::from_half_keys(&self.own_key, acknowledgement)?;
        debug!("acknowledging ticket using response {}", response.to_hex());

        if self.ticket.verified_ticket().challenge == response.to_challenge().into() {
            Ok(AcknowledgedTicket::new(self.ticket, response))
        } else {
            Err(CryptoError::InvalidChallenge.into())
        }
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
    pub fn new(ticket: VerifiedTicket, response: Response) -> AcknowledgedTicket {
        Self {
            // new tickets are always untouched
            status: AcknowledgedTicketStatus::Untouched,
            ticket,
            response,
        }
    }

    /// Convenience method to retrieve a reference to the underlying verified [Ticket].
    #[inline]
    pub fn verified_ticket(&self) -> &Ticket {
        self.ticket.verified_ticket()
    }

    /// Checks if this acknowledged ticket is winning.
    pub fn is_winning(&self, chain_keypair: &ChainKeypair, domain_separator: &Hash) -> bool {
        self.ticket.is_winning(&self.response, chain_keypair, domain_separator)
    }

    /// Transforms this ticket into [RedeemableTicket] that can be redeemed on-chain
    /// or transformed into [TransferableWinningTicket] that can be sent for aggregation.
    pub fn into_redeemable(
        self,
        chain_keypair: &ChainKeypair,
        domain_separator: &Hash,
    ) -> crate::errors::Result<RedeemableTicket> {
        let vrf_params = derive_vrf_parameters(self.ticket.verified_hash(), chain_keypair, domain_separator.as_ref())?;

        Ok(RedeemableTicket {
            ticket: self.ticket,
            response: self.response,
            vrf_params,
            channel_dst: *domain_separator,
        })
    }

    /// Shorthand for transforming this ticket into [TransferableWinningTicket].
    pub fn into_transferable(
        self,
        chain_keypair: &ChainKeypair,
        domain_separator: &Hash,
    ) -> errors::Result<TransferableWinningTicket> {
        self.into_redeemable(chain_keypair, domain_separator)
            .map(TransferableWinningTicket::from)
    }
}

impl Display for AcknowledgedTicket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "acknowledged {} in state '{}'", self.ticket, self.status)
    }
}

/// Represents a winning ticket that can be successfully redeemed on chain.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedeemableTicket {
    /// Verified ticket that can be redeemed.
    pub ticket: VerifiedTicket,
    /// Solution to the PoR challenge in the ticket.
    pub response: Response,
    /// VRF parameters required for redeeming.
    pub vrf_params: VrfParameters,
    /// Channel domain separator used to compute the VRF parameters.
    pub channel_dst: Hash,
}

impl RedeemableTicket {
    /// Convenience method to retrieve a reference to the underlying verified [Ticket].
    #[inline]
    pub fn verified_ticket(&self) -> &Ticket {
        self.ticket.verified_ticket()
    }
}

impl Display for RedeemableTicket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "redeemable {}", self.ticket)
    }
}

impl From<RedeemableTicket> for AcknowledgedTicket {
    fn from(value: RedeemableTicket) -> Self {
        Self {
            status: AcknowledgedTicketStatus::Untouched,
            ticket: value.ticket,
            response: value.response,
        }
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
    pub fn into_redeemable(
        self,
        expected_issuer: &Address,
        domain_separator: &Hash,
    ) -> errors::Result<RedeemableTicket> {
        if !self.signer.eq(expected_issuer) {
            return Err(crate::errors::CoreTypesError::InvalidInputData(
                "invalid ticket issuer".into(),
            ));
        }

        let verified_ticket = self
            .ticket
            .verify(&self.signer, domain_separator)
            .map_err(|_| CoreTypesError::CryptoError(CryptoError::SignatureVerification.into()))?;

        if check_ticket_win(
            verified_ticket.verified_hash(),
            &verified_ticket.verified_signature(),
            &verified_ticket.verified_ticket().encoded_win_prob,
            &self.response,
            &self.vrf_params,
        ) {
            Ok(RedeemableTicket {
                ticket: verified_ticket,
                response: self.response,
                vrf_params: Default::default(),
                channel_dst: *domain_separator,
            })
        } else {
            Err(crate::errors::CoreTypesError::InvalidInputData(
                "ticket is not a win".into(),
            ))
        }
    }
}

impl PartialOrd<Self> for TransferableWinningTicket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TransferableWinningTicket {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ticket.cmp(&other.ticket)
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
    use super::*;
    use crate::tickets::{AcknowledgedTicket, UnacknowledgedTicket};
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

    #[test]
    pub fn test_win_prob_to_f64() {
        let mut test_bit_string = [0xffu8; 7];

        assert_eq!(0.0f64, super::win_prob_to_f64(&[0u8; 7]));

        assert_eq!(1.0f64, super::win_prob_to_f64(&test_bit_string));

        test_bit_string[0] = 0x7f;
        assert_eq!(0.5f64, super::win_prob_to_f64(&test_bit_string));

        test_bit_string[0] = 0x3f;
        assert_eq!(0.25f64, super::win_prob_to_f64(&test_bit_string));

        test_bit_string[0] = 0x1f;
        assert_eq!(0.125f64, super::win_prob_to_f64(&test_bit_string));
    }

    #[test]
    pub fn test_f64_to_win_prob() {
        let mut test_bit_string = [0xffu8; 7];

        assert_eq!([0u8; 7], super::f64_to_win_prob(0.0f64).unwrap());

        assert_eq!(test_bit_string, super::f64_to_win_prob(1.0f64).unwrap());

        test_bit_string[0] = 0x7f;
        assert_eq!(test_bit_string, super::f64_to_win_prob(0.5f64).unwrap());

        test_bit_string[0] = 0x3f;
        assert_eq!(test_bit_string, super::f64_to_win_prob(0.25f64).unwrap());

        test_bit_string[0] = 0x1f;
        assert_eq!(test_bit_string, super::f64_to_win_prob(0.125f64).unwrap());
    }

    #[test]
    pub fn test_win_prob_back_and_forth() {
        for float in [0.1f64, 0.002f64, 0.00001f64, 0.7311111f64, 1.0f64, 0.0f64] {
            assert!((float - super::win_prob_to_f64(&super::f64_to_win_prob(float).unwrap())).abs() < f64::EPSILON);
        }
    }

    #[test]
    pub fn test_ticket_serialize_deserialize() {
        let initial_ticket = super::Ticket::new(
            &BOB.public().to_address(),
            Balance::new(U256::one(), BalanceType::HOPR),
            0,
            1,
            1.0,
            1,
            EthereumChallenge::default(),
            &ALICE,
            &Hash::default(),
        )
        .unwrap();

        assert_ne!(*initial_ticket.get_hash(&Hash::default()).as_ref(), [0u8; Hash::SIZE]);

        let ticket_bytes: [u8; Ticket::SIZE] = initial_ticket.clone().into();
        assert_eq!(initial_ticket, Ticket::try_from(ticket_bytes.as_ref()).unwrap());
    }

    #[test]
    pub fn test_ticket_serialize_deserialize_serde() {
        let initial_ticket = super::Ticket::new(
            &BOB.public().to_address(),
            Balance::new(U256::one(), BalanceType::HOPR),
            0,
            1,
            1.0,
            1,
            EthereumChallenge::default(),
            &ALICE,
            &Hash::default(),
        )
        .unwrap();

        assert_eq!(
            initial_ticket,
            bincode::deserialize(&bincode::serialize(&initial_ticket).unwrap()).unwrap()
        );
    }

    #[test]
    pub fn test_ticket_sign_verify() {
        let initial_ticket = super::Ticket::new(
            &BOB.public().to_address(),
            Balance::new(U256::one(), BalanceType::HOPR),
            0,
            1,
            1.0,
            1,
            EthereumChallenge::default(),
            &ALICE,
            &Hash::default(),
        )
        .unwrap();

        assert_ne!(*initial_ticket.get_hash(&Hash::default()).as_ref(), [0u8; Hash::SIZE]);

        assert!(initial_ticket
            .verify(&ALICE.public().to_address(), &Hash::default())
            .is_ok());
    }

    #[test]
    pub fn test_ticket_expected_payout() {
        let mut ticket = Ticket::new_partial(
            &ALICE.public().to_address(),
            &BOB.public().to_address(),
            Balance::new(U256::one(), BalanceType::HOPR),
            0,
            1,
            1.0,
            1,
        )
        .unwrap();

        assert_eq!(U256::one(), ticket.get_expected_payout());

        ticket.encoded_win_prob = f64_to_win_prob(0.0).unwrap();
        assert_eq!(U256::zero(), ticket.get_expected_payout());

        ticket.amount = Balance::new(100000000000_u64, BalanceType::HOPR);
        ticket.encoded_win_prob = f64_to_win_prob(0.00000000001f64).unwrap();

        assert_eq!(U256::one(), ticket.get_expected_payout());
    }

    #[test]
    pub fn test_path_position() {
        let mut ticket = Ticket::new_partial(
            &ALICE.public().to_address(),
            &BOB.public().to_address(),
            Balance::new(U256::one(), BalanceType::HOPR),
            0,
            1,
            1.0,
            1,
        )
        .unwrap();

        assert_eq!(1u8, ticket.get_path_position(U256::one()).unwrap());

        ticket.amount = Balance::new(U256::from(34u64), BalanceType::HOPR);

        assert_eq!(2u8, ticket.get_path_position(U256::from(17u64)).unwrap());

        ticket.amount = Balance::new(U256::from(30u64), BalanceType::HOPR);
        ticket.encoded_win_prob = f64_to_win_prob(0.2).unwrap();

        assert_eq!(U256::from(6u64), ticket.get_expected_payout());

        assert_eq!(2u8, ticket.get_path_position(U256::from(3u64)).unwrap());

        ticket.encoded_win_prob = f64_to_win_prob(0.0).unwrap();
        assert_eq!(U256::zero(), ticket.get_expected_payout());
    }

    #[test]
    pub fn test_path_position_bad_examples() {
        let ticket = Ticket::new_partial(
            &ALICE.public().to_address(),
            &BOB.public().to_address(),
            Balance::new(256_u64, BalanceType::HOPR),
            0,
            1,
            1.0,
            1,
        )
        .unwrap();

        assert!(ticket.get_path_position(U256::from(1u64)).is_err());
    }

    #[test]
    pub fn test_zero_hop() {
        let zero_hop_ticket = Ticket::new_zero_hop(
            &BOB.public().to_address(),
            &ALICE.public().to_address(),
            &Hash::default(),
        )
        .unwrap();
        assert!(zero_hop_ticket
            .verify(&ALICE.public().to_address(), &Hash::default())
            .is_ok());
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
