use bindings::hopr_channels::RedeemTicketCall;
use ethers::contract::EthCall;
use hex_literal::hex;
use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use tracing::{debug, error};

use crate::errors;
use crate::errors::CoreTypesError;
use crate::prelude::generate_channel_id;
use crate::prelude::CoreTypesError::InvalidInputData;

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

/// Builder for [Ticket] and [VerifiedTicket].
///
/// A new builder is created via [TicketBuilder::default] or [TicketBuilder::zero_hop].
///
/// Input validation is performed upon calling [TicketBuilder::build], [TicketBuilder::build_signed]
/// and [TicketBuilder::build_verified].
#[derive(Debug, Clone, smart_default::SmartDefault)]
pub struct TicketBuilder {
    channel_id: Option<Hash>,
    amount: Option<U256>,
    balance: Option<Balance>,
    #[default = 0]
    index: u64,
    #[default = 1]
    index_offset: u32,
    #[default = 1]
    channel_epoch: u32,
    #[default(Some(1.0))]
    win_prob: Option<f64>,
    win_prob_enc: Option<EncodedWinProb>,
    challenge: Option<EthereumChallenge>,
    signature: Option<Signature>,
}

impl TicketBuilder {
    /// Initializes the builder for a zero hop ticket.
    pub fn zero_hop() -> Self {
        Self {
            index: 0,
            index_offset: 0,
            win_prob: Some(0.0),
            channel_epoch: 0,
            ..Default::default()
        }
    }

    /// Sets channel id based on the `source` and `destination`.
    /// This or [TicketBuilder::channel_id] must be set.
    pub fn direction(mut self, source: &Address, destination: &Address) -> Self {
        self.channel_id = Some(generate_channel_id(source, destination));
        self
    }

    /// Sets the channel id.
    /// This or [TicketBuilder::direction] must be set.
    pub fn channel_id(mut self, channel_id: Hash) -> Self {
        self.channel_id = Some(channel_id);
        self
    }

    /// Sets the ticket amount.
    /// This or [TicketBuilder::balance] must be set and be less or equal to 10^25.
    pub fn amount<T: Into<U256>>(mut self, amount: T) -> Self {
        self.amount = Some(amount.into());
        self.balance = None;
        self
    }

    /// Sets the ticket amount as HOPR balance.
    /// This or [TicketBuilder::amount] must be set and be less or equal to 10^25.
    pub fn balance(mut self, balance: Balance) -> Self {
        self.balance = Some(balance);
        self.amount = None;
        self
    }

    /// Sets the ticket index.
    /// Must be less or equal to 2^48.
    /// Defaults to 0.
    pub fn index(mut self, index: u64) -> Self {
        self.index = index;
        self
    }

    /// Sets the index offset.
    /// Must be greater or equal 1.
    /// Defaults to 1.
    pub fn index_offset(mut self, index_offset: u32) -> Self {
        self.index_offset = index_offset;
        self
    }

    /// Sets the channel epoch.
    /// Must be less or equal to 2^24.
    /// Defaults to 1.
    pub fn channel_epoch(mut self, channel_epoch: u32) -> Self {
        self.channel_epoch = channel_epoch;
        self
    }

    /// Sets the ticket winning probability.
    /// Mutually exclusive with [TicketBuilder::win_prob_encoded].
    /// Defaults to 1.0
    pub fn win_prob(mut self, win_prob: f64) -> Self {
        self.win_prob = Some(win_prob);
        self.win_prob_enc = None;
        self
    }

    /// Sets the encoded ticket winning probability.
    /// Mutually exlusive with [TicketBuilder::win_prob].
    /// Defaults to [ALWAYS_WINNING].
    pub fn win_prob_encoded(mut self, win_prob: EncodedWinProb) -> Self {
        self.win_prob = None;
        self.win_prob_enc = Some(win_prob);
        self
    }

    /// Sets the [EthereumChallenge] for Proof of Relay.
    /// Must be set.
    pub fn challenge(mut self, challenge: EthereumChallenge) -> Self {
        self.challenge = Some(challenge);
        self
    }

    /// Set the signature of this ticket.
    /// Defaults to `None`.
    pub fn signature(mut self, signature: Signature) -> Self {
        self.signature = Some(signature);
        self
    }

    /// Verifies all inputs and builds the [Ticket].
    /// This does not perform signature verification, if a [signature](TicketBuilder::signature)
    /// was set.
    pub fn build(self) -> errors::Result<Ticket> {
        let amount = match (self.amount, self.balance) {
            (Some(amount), None) if amount.lt(&10_u128.pow(25).into()) => BalanceType::HOPR.balance(amount),
            (None, Some(balance))
                if balance.balance_type() == BalanceType::HOPR && balance.amount().lt(&10_u128.pow(25).into()) =>
            {
                balance
            }
            (None, None) => return Err(InvalidInputData("missing ticket amount".into())),
            (Some(_), Some(_)) => {
                return Err(InvalidInputData(
                    "either amount or balance must be set but not both".into(),
                ))
            }
            _ => {
                return Err(InvalidInputData(
                    "tickets may not have more than 1% of total supply".into(),
                ))
            }
        };

        if self.index > (1_u64 << 48) {
            return Err(InvalidInputData("cannot hold ticket indices larger than 2^48".into()));
        }

        if self.channel_epoch > (1_u32 << 24) {
            return Err(InvalidInputData("cannot hold channel epoch larger than 2^24".into()));
        }

        let encoded_win_prob = match (self.win_prob, self.win_prob_enc) {
            (Some(win_prob), None) => f64_to_win_prob(win_prob)?,
            (None, Some(win_prob)) => win_prob,
            (Some(_), Some(_)) => return Err(InvalidInputData("conflicting winning probabilities".into())),
            (None, None) => return Err(InvalidInputData("missing ticket winning probability".into())),
        };

        if self.index_offset < 1 {
            return Err(InvalidInputData(
                "ticket index offset must be greater or equal to 1".into(),
            ));
        }

        Ok(Ticket {
            channel_id: self.channel_id.ok_or(InvalidInputData("missing channel id".into()))?,
            amount,
            index: self.index,
            index_offset: self.index_offset,
            encoded_win_prob,
            channel_epoch: self.channel_epoch,
            challenge: self
                .challenge
                .ok_or(InvalidInputData("missing ticket challenge".into()))?,
            signature: self.signature,
        })
    }

    /// Validates all inputs and builds the [VerifiedTicket] by signing the ticket data
    /// with the given key. Fails if [signature](TicketBuilder::signature) was previously set.
    pub fn build_signed(self, signer: &ChainKeypair, domain_separator: &Hash) -> errors::Result<VerifiedTicket> {
        if self.signature.is_none() {
            Ok(self.build()?.sign(signer, domain_separator))
        } else {
            Err(InvalidInputData("signature already set".into()))
        }
    }

    /// Validates all input and builds the [VerifiedTicket] by **assuming** the previously
    /// set [signature](TicketBuilder::signature) is valid and belongs to the given ticket `hash`.
    /// It does **not** check whether `hash` matches the input data nor that the signature verifies
    /// the given hash.
    pub fn build_verified(self, hash: Hash) -> errors::Result<VerifiedTicket> {
        if let Some(signature) = self.signature {
            let issuer = PublicKey::from_signature_hash(hash.as_ref(), &signature)?.to_address();
            Ok(VerifiedTicket(self.build()?, hash, issuer))
        } else {
            Err(InvalidInputData("signature is missing".into()))
        }
    }
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// Contains the overall description of a ticket with a signature.
///
/// This structure is not considered [verified](VerifiedTicket), unless
/// the [Ticket::verify] or [Ticket::sign] methods are called.
///
/// # Ticket state machine
/// See the entire state machine describing the relations of different ticket types below:
///```mermaid
///flowchart TB
///     A[Ticket] -->|verify| B(VerifiedTicket)
///     B --> |leak| A
///     A --> |sign| B
///     B --> |into_unacknowledged| C(UnacknowledgedTicket)
///     B --> |into_acknowledged| D(AcknowledgedTicket)
///     C --> |acknowledge| D
///     D --> |into_redeemable| E(RedeemableTicket)
///     D --> |into_transferable| F(TransferableWinningTicket)
///     E --> |into_transferable| F
///     F --> |into_redeemable| E
///```
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
    /// If a signature was already present, it will be replaced.
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

    /// Returns the decoded winning probability of the ticket
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
            TicketBuilder::default()
                .channel_id(channel_id)
                .amount(amount)
                .index(u64::from_be_bytes(index))
                .index_offset(u32::from_be_bytes(index_offset))
                .channel_epoch(u32::from_be_bytes(channel_epoch))
                .win_prob_encoded(encoded_win_prob)
                .challenge(challenge)
                .signature(signature)
                .build()
                .map_err(|_| GeneralError::ParseError)
        } else {
            Err(GeneralError::ParseError)
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

    /// Shorthand to acknowledge the ticket if the matching response is already known.
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
            Ok(self.ticket.into_acknowledged(response))
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
    use crate::tickets::AcknowledgedTicket;
    use hex_literal::hex;
    use hopr_crypto_types::{
        keypairs::{ChainKeypair, Keypair},
        types::{Challenge, CurvePoint, HalfKey, Hash, Response},
    };
    use hopr_primitive_types::prelude::UnitaryFloatOps;
    use hopr_primitive_types::primitives::{Address, BalanceType, EthereumChallenge, U256};

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
    pub fn test_ticket_builder_zero_hop() {
        let ticket = TicketBuilder::zero_hop()
            .direction(&ALICE.public().to_address(), &BOB.public().to_address())
            .build()
            .expect("should build ticket");
        assert_eq!(0, ticket.index);
        assert_eq!(0.0, ticket.win_prob());
        assert_eq!(0, ticket.channel_epoch);
        assert_eq!(
            generate_channel_id(&ALICE.public().to_address(), &BOB.public().to_address()),
            ticket.channel_id
        );
    }

    #[test]
    pub fn test_ticket_serialize_deserialize() {
        let initial_ticket = TicketBuilder::default()
            .direction(&ALICE.public().to_address(), &BOB.public().to_address())
            .balance(BalanceType::HOPR.one())
            .index(0)
            .index_offset(1)
            .win_prob(1.0)
            .channel_epoch(1)
            .challenge(Default::default())
            .build_signed(&ALICE, &Default::default())
            .expect("should build ticket");

        assert_ne!(initial_ticket.verified_hash().as_ref(), [0u8; Hash::SIZE]);

        let ticket_bytes: [u8; Ticket::SIZE] = initial_ticket.verified_ticket().clone().into();
        assert_eq!(
            initial_ticket.verified_ticket(),
            &Ticket::try_from(ticket_bytes.as_ref()).unwrap()
        );
    }

    #[test]
    pub fn test_ticket_serialize_deserialize_serde() {
        let initial_ticket = TicketBuilder::default()
            .direction(&ALICE.public().to_address(), &BOB.public().to_address())
            .balance(BalanceType::HOPR.one())
            .index(0)
            .index_offset(1)
            .win_prob(1.0)
            .channel_epoch(1)
            .challenge(Default::default())
            .build_signed(&ALICE, &Default::default())
            .expect("should build ticket");

        assert_eq!(
            initial_ticket,
            bincode::deserialize(&bincode::serialize(&initial_ticket).unwrap()).unwrap()
        );
    }

    #[test]
    pub fn test_ticket_sign_verify() {
        let initial_ticket = TicketBuilder::default()
            .direction(&ALICE.public().to_address(), &BOB.public().to_address())
            .balance(BalanceType::HOPR.one())
            .index(0)
            .index_offset(1)
            .win_prob(1.0)
            .channel_epoch(1)
            .challenge(Default::default())
            .build_signed(&ALICE, &Default::default())
            .expect("should build ticket");

        assert_ne!(initial_ticket.verified_hash().as_ref(), [0u8; Hash::SIZE]);

        let ticket = initial_ticket.leak();
        assert!(ticket.verify(&ALICE.public().to_address(), &Default::default()).is_ok());
    }

    #[test]
    pub fn test_path_position() {
        let builder = TicketBuilder::default()
            .direction(&ALICE.public().to_address(), &BOB.public().to_address())
            .balance(BalanceType::HOPR.one())
            .index(0)
            .index_offset(1)
            .win_prob(1.0)
            .channel_epoch(1)
            .challenge(Default::default());

        let ticket = builder
            .clone()
            .build_signed(&ALICE, &Default::default())
            .expect("should build ticket");

        assert_eq!(1u8, ticket.get_path_position(1_32.into()).unwrap());

        let ticket = builder
            .clone()
            .amount(34_u64)
            .build_signed(&ALICE, &Default::default())
            .expect("should build ticket");

        assert_eq!(2u8, ticket.get_path_position(17_u64.into()).unwrap());

        let ticket = builder
            .clone()
            .amount(30_u64)
            .win_prob(0.2)
            .build_signed(&ALICE, &Default::default())
            .expect("should build ticket");

        assert_eq!(2u8, ticket.get_path_position(3_u64.into()).unwrap());
    }

    #[test]
    pub fn test_path_position_mismatch() {
        let ticket = TicketBuilder::default()
            .direction(&ALICE.public().to_address(), &BOB.public().to_address())
            .balance(BalanceType::HOPR.one())
            .index(0)
            .index_offset(1)
            .win_prob(1.0)
            .channel_epoch(1)
            .challenge(Default::default())
            .build_signed(&ALICE, &Default::default())
            .expect("should build ticket");

        assert!(ticket.get_path_position(1_u64.into()).is_err());
    }

    #[test]
    pub fn test_zero_hop() {
        let ticket = TicketBuilder::zero_hop()
            .direction(&ALICE.public().to_address(), &BOB.public().to_address())
            .build_signed(&ALICE, &Default::default())
            .expect("should build ticket");

        assert!(ticket
            .leak()
            .verify(&ALICE.public().to_address(), &Hash::default())
            .is_ok());
    }

    fn mock_ticket(
        pk: &ChainKeypair,
        counterparty: &Address,
        domain_separator: Option<Hash>,
        challenge: Option<EthereumChallenge>,
    ) -> VerifiedTicket {
        let win_prob = 1.0f64; // 100 %
        let price_per_packet: U256 = 10000000000000000u128.into(); // 0.01 HOPR
        let path_pos = 5u64;

        TicketBuilder::default()
            .direction(&pk.public().to_address(), counterparty)
            .amount(price_per_packet.div_f64(win_prob).unwrap() * U256::from(path_pos))
            .index(0)
            .index_offset(1)
            .win_prob(1.0)
            .channel_epoch(4)
            .challenge(challenge.unwrap_or_default())
            .build_signed(pk, &domain_separator.unwrap_or_default())
            .expect("should build ticket")
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

        let dst = Hash::default();
        let ack = mock_ticket(
            &ALICE,
            &BOB.public().to_address(),
            Some(dst),
            Some(Challenge::from(cp_sum).to_ethereum_challenge()),
        )
        .into_unacknowledged(hk1)
        .acknowledge(&hk2)
        .expect("must be able to acknowledge ticket");

        assert!(ack.is_winning(&BOB, &dst), "ticket must be winning");
    }

    #[test]
    fn test_acknowledged_ticket() {
        let response =
            Response::try_from(hex!("876a41ee5fb2d27ac14d8e8d552692149627c2f52330ba066f9e549aef762f73").as_ref())
                .unwrap();

        let dst = Hash::default();

        let ticket = mock_ticket(
            &ALICE,
            &BOB.public().to_address(),
            Some(dst),
            Some(response.to_challenge().into()),
        );

        let acked_ticket = ticket.into_acknowledged(response);

        let mut deserialized_ticket =
            bincode::deserialize::<AcknowledgedTicket>(&bincode::serialize(&acked_ticket).unwrap()).unwrap();
        assert_eq!(acked_ticket, deserialized_ticket);

        assert!(deserialized_ticket.is_winning(&BOB, &dst));

        deserialized_ticket.status = super::AcknowledgedTicketStatus::BeingAggregated;

        assert_eq!(
            deserialized_ticket,
            bincode::deserialize(&bincode::serialize(&deserialized_ticket).unwrap()).unwrap()
        );
    }

    #[test]
    fn test_ticket_entire_ticket_transfer_flow() {}
}
