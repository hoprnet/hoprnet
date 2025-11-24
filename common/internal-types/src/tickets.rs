use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
};

use hex_literal::hex;
use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
use tracing::{debug, error, instrument};

use crate::{
    errors,
    errors::CoreTypesError,
    prelude::{ChannelId, CoreTypesError::InvalidInputData, generate_channel_id},
};

/// Size-optimized encoding of the ticket, used for both,
/// network transfer and in the smart contract.
const ENCODED_TICKET_LENGTH: usize = 60;

/// Custom float to integer encoding used in the integer-only
/// Ethereum Virtual Machine (EVM). Chosen to be easily
/// convertible to IEEE754 double-precision and vice versa
const ENCODED_WIN_PROB_LENGTH: usize = 7;

/// Define the selector for the redeemTicketCall to avoid importing
/// the entire hopr-bindings crate for one single constant.
/// This value should be updated with the function interface changes.
// pub const REDEEM_CALL_SELECTOR: [u8; 4] = [252, 183, 121, 111];
pub const REDEEM_CALL_SELECTOR: [u8; 4] = [101, 227, 250, 114];

/// Winning probability encoded in 7-byte representation
pub type EncodedWinProb = [u8; ENCODED_WIN_PROB_LENGTH];

/// Represents a ticket winning probability.
///
/// It holds the modified IEEE-754 but behaves like a reduced precision float.
/// It intentionally does not implement `Ord` or `Eq`, as
/// it can be only [approximately compared](WinningProbability::approx_cmp).
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WinningProbability(#[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] EncodedWinProb);

impl WinningProbability {
    /// 100% winning probability
    pub const ALWAYS: Self = Self([0xff; ENCODED_WIN_PROB_LENGTH]);
    // This value can no longer be represented with the winning probability encoding
    // and is equal to 0
    pub const EPSILON: f64 = 0.00000001;
    /// 0% winning probability.
    pub const NEVER: Self = Self([0u8; ENCODED_WIN_PROB_LENGTH]);

    /// Converts winning probability to an unsigned integer (luck).
    pub fn as_luck(&self) -> u64 {
        let mut tmp = [0u8; 8];
        tmp[1..].copy_from_slice(&self.0);
        u64::from_be_bytes(tmp)
    }

    /// Convenience function to convert to internal probability representation.
    pub fn as_encoded(&self) -> EncodedWinProb {
        self.0
    }

    /// Convert probability to a float.
    pub fn as_f64(&self) -> f64 {
        if self.0.eq(&Self::NEVER.0) {
            return 0.0;
        }

        if self.0.eq(&Self::ALWAYS.0) {
            return 1.0;
        }

        let mut tmp = [0u8; 8];
        tmp[1..].copy_from_slice(&self.0);

        let tmp = u64::from_be_bytes(tmp);

        // project interval [0x0fffffffffffff, 0x0000000000000f] to [0x00000000000010, 0x10000000000000]
        let significand: u64 = tmp + 1;

        f64::from_bits((1023u64 << 52) | (significand >> 4)) - 1.0
    }

    /// Tries to get probability from a float.
    pub fn try_from_f64(win_prob: f64) -> errors::Result<Self> {
        // Also makes sure the input value is not NaN or infinite.
        if !(0.0..=1.0).contains(&win_prob) {
            return Err(InvalidInputData("winning probability must be in [0.0, 1.0]".into()));
        }

        if f64_approx_eq(0.0, win_prob, Self::EPSILON) {
            return Ok(Self::NEVER);
        }

        if f64_approx_eq(1.0, win_prob, Self::EPSILON) {
            return Ok(Self::ALWAYS);
        }

        let tmp: u64 = (win_prob + 1.0).to_bits();

        // // clear sign and exponent
        let significand: u64 = tmp & 0x000fffffffffffffu64;

        // project interval [0x10000000000000, 0x00000000000010] to [0x0000000000000f, 0x0fffffffffffff]
        let encoded = ((significand - 1) << 4) | 0x000000000000000fu64;

        let mut res = [0u8; 7];
        res.copy_from_slice(&encoded.to_be_bytes()[1..]);

        Ok(Self(res))
    }

    /// Performs approximate comparison up to [`Self::EPSILON`].
    pub fn approx_cmp(&self, other: &Self) -> Ordering {
        let a = self.as_f64();
        let b = other.as_f64();
        if !f64_approx_eq(a, b, Self::EPSILON) {
            a.partial_cmp(&b).expect("finite non-NaN f64 comparison cannot fail")
        } else {
            Ordering::Equal
        }
    }

    /// Performs approximate equality comparison up to [`Self::EPSILON`].
    pub fn approx_eq(&self, other: &Self) -> bool {
        self.approx_cmp(other) == Ordering::Equal
    }

    /// Gets the minimum of two winning probabilities.
    pub fn min(&self, other: &Self) -> Self {
        if self.approx_cmp(other) == Ordering::Less {
            *self
        } else {
            *other
        }
    }

    /// Gets the maximum of two winning probabilities.
    pub fn max(&self, other: &Self) -> Self {
        if self.approx_cmp(other) == Ordering::Greater {
            *self
        } else {
            *other
        }
    }
}

impl Default for WinningProbability {
    fn default() -> Self {
        Self::ALWAYS
    }
}

impl Display for WinningProbability {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.8}", self.as_f64())
    }
}

impl From<EncodedWinProb> for WinningProbability {
    fn from(value: EncodedWinProb) -> Self {
        Self(value)
    }
}

impl<'a> From<&'a EncodedWinProb> for WinningProbability {
    fn from(value: &'a EncodedWinProb) -> Self {
        Self(*value)
    }
}

impl From<WinningProbability> for EncodedWinProb {
    fn from(value: WinningProbability) -> Self {
        value.0
    }
}

impl From<u64> for WinningProbability {
    fn from(value: u64) -> Self {
        let mut ret = Self::default();
        ret.0.copy_from_slice(&value.to_be_bytes()[1..]);
        ret
    }
}

impl TryFrom<f64> for WinningProbability {
    type Error = CoreTypesError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::try_from_f64(value)
    }
}

impl From<WinningProbability> for f64 {
    fn from(value: WinningProbability) -> Self {
        value.as_f64()
    }
}

impl PartialEq<f64> for WinningProbability {
    fn eq(&self, other: &f64) -> bool {
        f64_approx_eq(self.as_f64(), *other, Self::EPSILON)
    }
}

impl PartialEq<WinningProbability> for f64 {
    fn eq(&self, other: &WinningProbability) -> bool {
        f64_approx_eq(*self, other.as_f64(), WinningProbability::EPSILON)
    }
}

impl AsRef<[u8]> for WinningProbability {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<'a> TryFrom<&'a [u8]> for WinningProbability {
    type Error = GeneralError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        value
            .try_into()
            .map(Self)
            .map_err(|_| GeneralError::ParseError("WinningProbability".into()))
    }
}

impl BytesRepresentable for WinningProbability {
    const SIZE: usize = ENCODED_WIN_PROB_LENGTH;
}

/// Helper function checks if the given ticket values belong to a winning ticket.
///
/// This function is inexpensive to compute.
pub(crate) fn check_ticket_win(
    ticket_hash: &Hash,
    ticket_signature: &Signature,
    win_prob: &WinningProbability,
    response: &Response,
    vrf_params: &VrfParameters,
) -> bool {
    // Computed winning probability
    let mut computed_ticket_luck = [0u8; 8];
    computed_ticket_luck[1..].copy_from_slice(
        &Hash::create(&[
            ticket_hash.as_ref(),
            &vrf_params.get_v_encoded_point().as_bytes()[1..], // skip prefix
            response.as_ref(),
            ticket_signature.as_ref(),
        ])
        .as_ref()[0..7],
    );

    u64::from_be_bytes(computed_ticket_luck) <= win_prob.as_luck()
}

/// A ticket is uniquely identified by its channel id, ticket index and epoch.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TicketId {
    pub id: ChannelId,
    pub epoch: u32,
    pub index: u64,
}

impl Display for TicketId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ticket #{}, epoch {} in channel {}", self.index, self.epoch, self.id)
    }
}

impl From<&Ticket> for TicketId {
    fn from(value: &Ticket) -> Self {
        Self {
            id: value.channel_id,
            epoch: value.channel_epoch,
            index: value.index,
        }
    }
}

/// Builder for the [`Ticket`] and [`VerifiedTicket`].
///
/// A new builder is created via [`TicketBuilder::default`] or [`TicketBuilder::zero_hop`].
///
/// Input validation is performed upon calling [`TicketBuilder::build`], [`TicketBuilder::build_signed`]
/// and [`TicketBuilder::build_verified`].
#[derive(Debug, Clone, smart_default::SmartDefault)]
pub struct TicketBuilder {
    channel_id: Option<Hash>,
    amount: Option<U256>,
    balance: Option<HoprBalance>,
    #[default = 0]
    index: u64,
    #[default = 1]
    channel_epoch: u32,
    win_prob: WinningProbability,
    challenge: Option<EthereumChallenge>,
    signature: Option<Signature>,
}

impl TicketBuilder {
    /// Initializes the builder for a zero-hop ticket.
    #[must_use]
    pub fn zero_hop() -> Self {
        Self {
            index: 0,
            amount: Some(U256::zero()),
            win_prob: WinningProbability::NEVER,
            channel_epoch: 0,
            ..Default::default()
        }
    }

    /// Initializes the builder for a ticket with the given [`TicketId`].
    #[must_use]
    pub fn from_id(ticket_id: &TicketId) -> Self {
        Self::default()
            .channel_id(ticket_id.id)
            .index(ticket_id.index)
            .channel_epoch(ticket_id.epoch)
    }

    /// Sets channel id based on the `source` and `destination`.
    /// This, [TicketBuilder::channel_id] or [TicketBuilder::addresses] must be set.
    #[must_use]
    pub fn direction(mut self, source: &Address, destination: &Address) -> Self {
        self.channel_id = Some(generate_channel_id(source, destination));
        self
    }

    /// Sets channel id based on the `source` and `destination`.
    /// This, [TicketBuilder::channel_id] or [TicketBuilder::direction] must be set.
    #[must_use]
    pub fn addresses<T: Into<Address>, U: Into<Address>>(mut self, source: T, destination: U) -> Self {
        self.channel_id = Some(generate_channel_id(&source.into(), &destination.into()));
        self
    }

    /// Sets the channel id.
    /// This, [TicketBuilder::addresses] or [TicketBuilder::direction] must be set.
    #[must_use]
    pub fn channel_id(mut self, channel_id: Hash) -> Self {
        self.channel_id = Some(channel_id);
        self
    }

    /// Sets the ticket amount.
    /// This or [TicketBuilder::balance] must be set and be less or equal to 10^25.
    #[must_use]
    pub fn amount<T: Into<U256>>(mut self, amount: T) -> Self {
        self.amount = Some(amount.into());
        self.balance = None;
        self
    }

    /// Sets the ticket amount as HOPR balance.
    /// This or [TicketBuilder::amount] must be set and be less or equal to 10^25.
    #[must_use]
    pub fn balance(mut self, balance: HoprBalance) -> Self {
        self.balance = Some(balance);
        self.amount = None;
        self
    }

    /// Sets the ticket index.
    /// Must be less or equal to 2^48.
    /// Defaults to 0.
    #[must_use]
    pub fn index(mut self, index: u64) -> Self {
        self.index = index;
        self
    }

    /// Sets the channel epoch.
    /// Must be less or equal to 2^24.
    /// Defaults to 1.
    #[must_use]
    pub fn channel_epoch(mut self, channel_epoch: u32) -> Self {
        self.channel_epoch = channel_epoch;
        self
    }

    /// Sets the ticket winning probability.
    /// Defaults to 1.0
    #[must_use]
    pub fn win_prob(mut self, win_prob: WinningProbability) -> Self {
        self.win_prob = win_prob;
        self
    }

    /// Sets the [`Challenge`] for the Proof of Relay, converting it to [`EthereumChallenge`] first.
    ///
    /// Either this method or [`Ticket::eth_challenge`] must be called.
    #[must_use]
    pub fn challenge(mut self, challenge: Challenge) -> Self {
        self.challenge = Some(challenge.to_ethereum_challenge());
        self
    }

    /// Sets the [`EthereumChallenge`] for the Proof of Relay.
    /// Either this method or [`Ticket::challenge`] must be called.
    pub fn eth_challenge(mut self, challenge: EthereumChallenge) -> Self {
        self.challenge = Some(challenge);
        self
    }

    /// Set the signature of this ticket.
    /// Defaults to `None`.
    #[must_use]
    pub fn signature(mut self, signature: Signature) -> Self {
        self.signature = Some(signature);
        self
    }

    /// Verifies all inputs and builds the [Ticket].
    /// This **does not** perform signature verification if a [signature](TicketBuilder::signature)
    /// was set.
    pub fn build(self) -> errors::Result<Ticket> {
        let amount = match (self.amount, self.balance) {
            (Some(amount), None) if amount.lt(&10_u128.pow(25).into()) => HoprBalance::from(amount),
            (None, Some(balance)) if balance.amount().lt(&10_u128.pow(25).into()) => balance,
            (None, None) => return Err(InvalidInputData("missing ticket amount".into())),
            (Some(_), Some(_)) => {
                return Err(InvalidInputData(
                    "either amount or balance must be set but not both".into(),
                ));
            }
            _ => {
                return Err(InvalidInputData(
                    "tickets may not have more than 1% of total supply".into(),
                ));
            }
        };

        if self.index > (1_u64 << 48) {
            return Err(InvalidInputData("cannot hold ticket indices larger than 2^48".into()));
        }

        if self.channel_epoch > (1_u32 << 24) {
            return Err(InvalidInputData("cannot hold channel epoch larger than 2^24".into()));
        }

        Ok(Ticket {
            channel_id: self.channel_id.ok_or(InvalidInputData("missing channel id".into()))?,
            amount,
            index: self.index,
            encoded_win_prob: self.win_prob.into(),
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

    /// Validates all inputs and builds the [VerifiedTicket] by **assuming** the previously
    /// set [signature](TicketBuilder::signature) is valid and belongs to the given ticket `hash`.
    /// It does **not** check whether `hash` matches the input data nor that the signature verifies
    /// the given hash.
    pub fn build_verified(self, hash: Hash) -> errors::Result<VerifiedTicket> {
        if let Some(signature) = self.signature {
            let issuer = signature.recover_from_hash(&hash)?.to_address();
            Ok(VerifiedTicket(self.build()?, hash, issuer))
        } else {
            Err(InvalidInputData("signature is missing".into()))
        }
    }
}

impl From<&Ticket> for TicketBuilder {
    fn from(value: &Ticket) -> Self {
        Self {
            channel_id: Some(value.channel_id),
            amount: None,
            balance: Some(value.amount),
            index: value.index,
            channel_epoch: value.channel_epoch,
            win_prob: value.encoded_win_prob.into(),
            challenge: Some(value.challenge),
            signature: None,
        }
    }
}

impl From<Ticket> for TicketBuilder {
    fn from(value: Ticket) -> Self {
        Self::from(&value)
    }
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// Contains the overall description of a ticket with a signature.
///
/// This structure is not considered [verified](VerifiedTicket), unless
/// the [`Ticket::verify`] or [`Ticket::sign`] methods are called.
///
/// # Ticket state machine
/// See the entire state machine describing the relations of different ticket types below:
/// ```mermaid
/// flowchart TB
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
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ticket {
    /// Channel ID.
    ///
    /// See [`generate_channel_id`] for how this value is generated.
    pub channel_id: ChannelId,
    /// Amount of HOPR tokens this ticket is worth.
    ///
    /// Always between 0 and 2^92.
    pub amount: HoprBalance, // 92 bits
    /// Ticket index.
    ///
    /// Always between 0 and 2^48.
    pub index: u64, // 48 bits
    /// Encoded winning probability represented via 56-bit number.
    pub encoded_win_prob: EncodedWinProb, // 56 bits
    /// Epoch of the channel this ticket belongs to.
    ///
    /// Always between 0 and 2^24.
    pub channel_epoch: u32, // 24 bits
    /// Represent the Proof of Relay challenge encoded as an Ethereum address.
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
        TicketId::from(self).cmp(&TicketId::from(other))
    }
}

impl Display for Ticket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ticket #{}, amount {}, epoch {} in channel {}",
            self.index, self.amount, self.channel_epoch, self.channel_id
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
        let hash_struct = Hash::create(&[&REDEEM_CALL_SELECTOR, &[0u8; 28], ticket_hash.as_ref()]);
        Hash::create(&[&hex!("1901"), domain_separator.as_ref(), hash_struct.as_ref()])
    }

    /// Signs the ticket using the given private key, turning this ticket into [VerifiedTicket].
    /// If a signature was already present, it will be replaced.
    pub fn sign(mut self, signing_key: &ChainKeypair, domain_separator: &Hash) -> VerifiedTicket {
        let ticket_hash = self.get_hash(domain_separator);
        self.signature = Some(Signature::sign_hash(&ticket_hash, signing_key));
        VerifiedTicket(self, ticket_hash, signing_key.public().to_address())
    }

    /// Verifies the signature of this ticket, turning this ticket into `VerifiedTicket`.
    /// If the verification fails, `Self` is returned in the error.
    ///
    /// This is done by recovering the signer from the signature and verifying that it matches
    /// the given `issuer` argument. This is possible due this specific instantiation of the ECDSA
    /// over the secp256k1 curve.
    /// The operation can fail if a public key cannot be recovered from the ticket signature.
    #[instrument(level = "trace", skip_all, err)]
    pub fn verify(self, issuer: &Address, domain_separator: &Hash) -> Result<VerifiedTicket, Box<Ticket>> {
        let ticket_hash = self.get_hash(domain_separator);

        if let Some(signature) = &self.signature {
            match signature.recover_from_hash(&ticket_hash) {
                Ok(pk) if pk.to_address().eq(issuer) => Ok(VerifiedTicket(self, ticket_hash, *issuer)),
                Err(e) => {
                    error!("failed to verify ticket signature: {e}");
                    Err(self.into())
                }
                _ => Err(self.into()),
            }
        } else {
            Err(self.into())
        }
    }

    /// Returns the decoded winning probability of the ticket
    #[inline]
    pub fn win_prob(&self) -> WinningProbability {
        WinningProbability(self.encoded_win_prob)
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
            let mut offset = 0;

            // TODO: not necessary to the ChannelId over the wire, only the counterparty is sufficient
            let channel_id = Hash::try_from(&value[offset..offset + Hash::SIZE])?;
            offset += Hash::SIZE;

            let mut amount = [0u8; 32];
            amount[20..32].copy_from_slice(&value[offset..offset + 12]);
            offset += 12;

            let mut index = [0u8; 8];
            index[2..8].copy_from_slice(&value[offset..offset + 6]);
            offset += 6;

            let mut channel_epoch = [0u8; 4];
            channel_epoch[1..4].copy_from_slice(&value[offset..offset + 3]);
            offset += 3;

            let win_prob = WinningProbability::try_from(&value[offset..offset + WinningProbability::SIZE])?;
            offset += WinningProbability::SIZE;

            debug_assert_eq!(offset, ENCODED_TICKET_LENGTH);

            let challenge = EthereumChallenge::try_from(&value[offset..offset + EthereumChallenge::SIZE])?;
            offset += EthereumChallenge::SIZE;

            let signature = Signature::try_from(&value[offset..offset + Signature::SIZE])?;

            // Validate the boundaries of the parsed values
            TicketBuilder::default()
                .channel_id(channel_id)
                .amount(U256::from_big_endian(&amount))
                .index(u64::from_be_bytes(index))
                .channel_epoch(u32::from_be_bytes(channel_epoch))
                .win_prob(win_prob)
                .eth_challenge(challenge)
                .signature(signature)
                .build()
                .map_err(|e| GeneralError::ParseError(format!("ticket build failed: {e}")))
        } else {
            Err(GeneralError::ParseError("Ticket".into()))
        }
    }
}

const TICKET_SIZE: usize = ENCODED_TICKET_LENGTH + EthereumChallenge::SIZE + Signature::SIZE;

impl BytesEncodable<TICKET_SIZE> for Ticket {}

/// Holds a ticket that has been already verified.
/// This structure guarantees that [`Ticket::get_hash()`] of [`VerifiedTicket::verified_ticket()`]
/// is always equal to [`VerifiedTicket::verified_hash`]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VerifiedTicket(Ticket, Hash, Address);

impl VerifiedTicket {
    /// Returns the verified encoded winning probability of the ticket
    #[inline]
    pub fn win_prob(&self) -> WinningProbability {
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
    /// where the input is the concatenation of ticket's hash, VRF's encoded `v` value,
    /// PoR response and the ticket's signature.
    ///
    /// ## Winning probability
    /// Each ticket specifies a probability, given as an integer in
    /// `[0, 2^56-1]` where 0 -> 0% and 2^56 - 1 -> 100% win
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
                &self.0.win_prob(),
                response,
                &vrf_params,
            )
        } else {
            error!("cannot derive vrf parameters for {self}");
            false
        }
    }

    /// Ticket with already verified signature.
    #[inline]
    pub fn verified_ticket(&self) -> &Ticket {
        &self.0
    }

    /// Fixed ticket hash that is guaranteed to be equal to
    /// [`Ticket::get_hash`] of [`VerifiedTicket::verified_ticket`].
    #[inline]
    pub fn verified_hash(&self) -> &Hash {
        &self.1
    }

    /// Verified issuer of the ticket.
    /// The returned address is guaranteed to be equal to the signer
    /// recovered from the [`VerifiedTicket::verified_ticket`]'s signature.
    #[inline]
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
    #[inline]
    pub fn leak(self) -> Ticket {
        self.0
    }

    /// Creates a new unacknowledged ticket from the [VerifiedTicket],
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

/// Represents a [`VerifiedTicket`] with an unknown other part of the [`HalfKey`].
/// Once the other [`HalfKey`] is known (forming a [`Response`]),
/// it can be [acknowledged](UnacknowledgedTicket::acknowledge).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
        debug!(ticket = %self.ticket, response = response.to_hex(), "acknowledging ticket using response");

        if self.ticket.verified_ticket().challenge == response.to_challenge()?.to_ethereum_challenge() {
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
    strum::Display,
    strum::EnumString,
    num_enum::IntoPrimitive,
    num_enum::TryFromPrimitive,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[strum(serialize_all = "PascalCase")]
pub enum AcknowledgedTicketStatus {
    /// The ticket is available for redeeming or aggregating
    #[default]
    Untouched = 0,
    /// Ticket is currently being redeemed in and ongoing redemption process
    BeingRedeemed = 1,
}

/// Contains acknowledgment information and the respective ticket
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AcknowledgedTicket {
    #[cfg_attr(feature = "serde", serde(default))]
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

    /// Transforms this ticket into [`RedeemableTicket`] that can be redeemed on-chain
    /// or transformed into [`TransferableWinningTicket`] that can be sent for aggregation.
    ///
    /// The `chain_keypair` must not be of the ticket's issuer.
    /// This ticket MUST be winning, otherwise the function fails with [`CoreTypesError::TicketNotWinning`].
    pub fn into_redeemable(
        self,
        chain_keypair: &ChainKeypair,
        domain_separator: &Hash,
    ) -> errors::Result<RedeemableTicket> {
        // This function must be called by the ticket recipient and not the issuer
        if chain_keypair.public().to_address().eq(self.ticket.verified_issuer()) {
            return Err(errors::CoreTypesError::LoopbackTicket);
        }

        let vrf_params = derive_vrf_parameters(self.ticket.verified_hash(), chain_keypair, domain_separator.as_ref())?;

        if !check_ticket_win(
            self.ticket.verified_hash(),
            self.ticket.verified_signature(),
            &self.ticket.win_prob(),
            &self.response,
            &vrf_params,
        ) {
            return Err(CoreTypesError::TicketNotWinning);
        }

        Ok(RedeemableTicket {
            ticket: self.ticket,
            response: self.response,
            vrf_params,
            channel_dst: *domain_separator,
        })
    }

    /// Shorthand for transforming this ticket into [TransferableWinningTicket].
    /// See [`AcknowledgedTicket::into_redeemable`] for details.
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

/// Represents a winning ticket that can be successfully redeemed on-chain.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    /// Gets the [`TicketId`] of the ticket.
    #[inline]
    pub fn ticket_id(&self) -> TicketId {
        TicketId::from(self.ticket.verified_ticket())
    }
}

impl PartialEq for RedeemableTicket {
    fn eq(&self, other: &Self) -> bool {
        self.ticket == other.ticket && self.channel_dst == other.channel_dst && self.response == other.response
    }
}

impl Eq for RedeemableTicket {}

impl PartialOrd for RedeemableTicket {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RedeemableTicket {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ticket.cmp(&other.ticket)
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
/// The [`TransferableWinningTicket`] can be easily retrieved from [`RedeemableTicket`], which strips
/// information about verification.
/// [`TransferableWinningTicket`] can be attempted to be converted back to [`RedeemableTicket`] only
/// when verified via [`TransferableWinningTicket::into_redeemable`] again.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransferableWinningTicket {
    pub ticket: Ticket,
    pub response: Response,
    pub vrf_params: VrfParameters,
    pub signer: Address,
}

impl TransferableWinningTicket {
    /// Attempts to transform this ticket back into a [`RedeemableTicket`].
    ///
    /// Verifies that the `signer` matches the `expected_issuer` and that the
    /// ticket has a valid signature from the `signer`.
    /// Then it verifies if the ticket is winning and therefore if it can be successfully
    /// redeemed on-chain.
    pub fn into_redeemable(
        self,
        expected_issuer: &Address,
        domain_separator: &Hash,
    ) -> errors::Result<RedeemableTicket> {
        if !self.signer.eq(expected_issuer) {
            return Err(InvalidInputData("invalid ticket issuer".into()));
        }

        let verified_ticket = self
            .ticket
            .verify(&self.signer, domain_separator)
            .map_err(|_| CoreTypesError::CryptoError(CryptoError::SignatureVerification))?;

        if check_ticket_win(
            verified_ticket.verified_hash(),
            verified_ticket.verified_signature(),
            &verified_ticket.verified_ticket().win_prob(),
            &self.response,
            &self.vrf_params,
        ) {
            Ok(RedeemableTicket {
                ticket: verified_ticket,
                response: self.response,
                vrf_params: self.vrf_params,
                channel_dst: *domain_separator,
            })
        } else {
            Err(InvalidInputData("ticket is not a win".into()))
        }
    }
}

impl PartialEq for TransferableWinningTicket {
    fn eq(&self, other: &Self) -> bool {
        self.ticket == other.ticket && self.signer == other.signer && self.response == other.response
    }
}

impl PartialOrd<Self> for TransferableWinningTicket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.ticket.cmp(&other.ticket))
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
pub mod tests {
    use hex_literal::hex;
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::{
        keypairs::{ChainKeypair, Keypair},
        types::{HalfKey, Hash, Response},
    };
    use hopr_primitive_types::{
        prelude::UnitaryFloatOps,
        primitives::{Address, EthereumChallenge, U256},
    };

    use super::*;

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).expect("lazy static keypair should be constructible");
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).expect("lazy static keypair should be constructible");
    }

    #[cfg(feature = "serde")]
    const BINCODE_CONFIGURATION: bincode::config::Configuration = bincode::config::standard()
        .with_little_endian()
        .with_variable_int_encoding();

    #[test]
    pub fn test_win_prob_to_f64() -> anyhow::Result<()> {
        assert_eq!(0.0f64, WinningProbability::NEVER.as_f64());

        assert_eq!(1.0f64, WinningProbability::ALWAYS.as_f64());

        let mut test_bit_string = [0xffu8; 7];
        test_bit_string[0] = 0x7f;
        assert_eq!(0.5f64, WinningProbability::from(&test_bit_string).as_f64());

        test_bit_string[0] = 0x3f;
        assert_eq!(0.25f64, WinningProbability::from(&test_bit_string).as_f64());

        test_bit_string[0] = 0x1f;
        assert_eq!(0.125f64, WinningProbability::from(&test_bit_string).as_f64());

        Ok(())
    }

    #[test]
    pub fn test_f64_to_win_prob() -> anyhow::Result<()> {
        assert_eq!([0u8; 7], WinningProbability::try_from(0.0f64)?.as_encoded());

        let mut test_bit_string = [0xffu8; 7];
        assert_eq!(test_bit_string, WinningProbability::try_from(1.0f64)?.as_encoded());

        test_bit_string[0] = 0x7f;
        assert_eq!(test_bit_string, WinningProbability::try_from(0.5f64)?.as_encoded());

        test_bit_string[0] = 0x3f;
        assert_eq!(test_bit_string, WinningProbability::try_from(0.25f64)?.as_encoded());

        test_bit_string[0] = 0x1f;
        assert_eq!(test_bit_string, WinningProbability::try_from(0.125f64)?.as_encoded());

        Ok(())
    }

    #[test]
    pub fn test_win_prob_approx_eq() -> anyhow::Result<()> {
        let wp_0 = WinningProbability(hex!("0020C49BBFFFFF"));
        let wp_1 = WinningProbability(hex!("0020C49BA5E34F"));

        assert_ne!(wp_0.as_ref(), wp_1.as_ref());
        assert_eq!(wp_0, wp_1.as_f64());

        Ok(())
    }

    #[test]
    pub fn test_win_prob_back_and_forth() -> anyhow::Result<()> {
        for float in [0.1f64, 0.002f64, 0.00001f64, 0.7311111f64, 1.0f64, 0.0f64] {
            assert!((float - WinningProbability::try_from_f64(float)?.as_f64()).abs() < f64::EPSILON);
        }

        Ok(())
    }

    #[test]
    pub fn test_win_prob_must_be_correctly_ordered() {
        let increment = WinningProbability::EPSILON * 100.0; // Testing the entire range would take too long
        let mut prev = WinningProbability::NEVER;
        while let Ok(next) = WinningProbability::try_from_f64(prev.as_f64() + increment) {
            assert!(prev.approx_cmp(&next).is_lt());
            prev = next;
        }
    }

    #[test]
    pub fn test_win_prob_epsilon_must_be_never() -> anyhow::Result<()> {
        assert!(WinningProbability::NEVER.approx_eq(&WinningProbability::try_from_f64(WinningProbability::EPSILON)?));
        Ok(())
    }

    #[test]
    pub fn test_win_prob_bounds_must_be_eq() -> anyhow::Result<()> {
        let bound = 0.1 + WinningProbability::EPSILON;
        let other = 0.1;
        assert!(WinningProbability::try_from_f64(bound)?.approx_eq(&WinningProbability::try_from_f64(other)?));
        Ok(())
    }

    #[test]
    pub fn test_ticket_builder_zero_hop() -> anyhow::Result<()> {
        let ticket = TicketBuilder::zero_hop()
            .direction(&ALICE.public().to_address(), &BOB.public().to_address())
            .eth_challenge(Default::default())
            .build()?;
        assert_eq!(0, ticket.index);
        assert_eq!(0.0, ticket.win_prob().as_f64());
        assert_eq!(0, ticket.channel_epoch);
        assert_eq!(
            generate_channel_id(&ALICE.public().to_address(), &BOB.public().to_address()),
            ticket.channel_id
        );
        Ok(())
    }

    #[test]
    pub fn test_ticket_serialize_deserialize() -> anyhow::Result<()> {
        let initial_ticket = TicketBuilder::default()
            .direction(&ALICE.public().to_address(), &BOB.public().to_address())
            .balance(1.into())
            .index(0)
            .win_prob(1.0.try_into()?)
            .channel_epoch(1)
            .eth_challenge(Default::default())
            .build_signed(&ALICE, &Default::default())?;

        assert_ne!(initial_ticket.verified_hash().as_ref(), [0u8; Hash::SIZE]);

        let ticket_bytes: [u8; Ticket::SIZE] = initial_ticket.verified_ticket().clone().into();
        assert_eq!(
            initial_ticket.verified_ticket(),
            &Ticket::try_from(ticket_bytes.as_ref())?
        );
        Ok(())
    }

    #[test]
    #[cfg(feature = "serde")]
    pub fn test_ticket_serialize_deserialize_serde() -> anyhow::Result<()> {
        let initial_ticket = TicketBuilder::default()
            .direction(&ALICE.public().to_address(), &BOB.public().to_address())
            .balance(1.into())
            .index(0)
            .win_prob(1.0.try_into()?)
            .channel_epoch(1)
            .eth_challenge(Default::default())
            .build_signed(&ALICE, &Default::default())?;

        assert_eq!(
            initial_ticket,
            bincode::serde::decode_from_slice(
                &bincode::serde::encode_to_vec(&initial_ticket, BINCODE_CONFIGURATION)?,
                BINCODE_CONFIGURATION
            )
            .map(|v| v.0)?
        );
        Ok(())
    }

    #[test]
    pub fn test_ticket_sign_verify() -> anyhow::Result<()> {
        let initial_ticket = TicketBuilder::default()
            .direction(&ALICE.public().to_address(), &BOB.public().to_address())
            .balance(1.into())
            .index(0)
            .win_prob(1.0.try_into()?)
            .channel_epoch(1)
            .eth_challenge(Default::default())
            .build_signed(&ALICE, &Default::default())?;

        assert_ne!(initial_ticket.verified_hash().as_ref(), [0u8; Hash::SIZE]);

        let ticket = initial_ticket.leak();
        assert!(ticket.verify(&ALICE.public().to_address(), &Default::default()).is_ok());
        Ok(())
    }

    #[test]
    pub fn test_zero_hop() -> anyhow::Result<()> {
        let ticket = TicketBuilder::zero_hop()
            .direction(&ALICE.public().to_address(), &BOB.public().to_address())
            .eth_challenge(Default::default())
            .build_signed(&ALICE, &Default::default())?;

        assert!(
            ticket
                .leak()
                .verify(&ALICE.public().to_address(), &Hash::default())
                .is_ok()
        );
        Ok(())
    }

    fn mock_ticket(
        pk: &ChainKeypair,
        counterparty: &Address,
        domain_separator: Option<Hash>,
        challenge: Option<EthereumChallenge>,
    ) -> anyhow::Result<VerifiedTicket> {
        let win_prob = 1.0f64; // 100 %
        let price_per_packet: U256 = 10000000000000000u128.into(); // 0.01 HOPR
        let path_pos = 5u64;

        Ok(TicketBuilder::default()
            .direction(&pk.public().to_address(), counterparty)
            .amount(price_per_packet.div_f64(win_prob)? * U256::from(path_pos))
            .index(0)
            .win_prob(1.0.try_into()?)
            .channel_epoch(4)
            .eth_challenge(challenge.unwrap_or_default())
            .build_signed(pk, &domain_separator.unwrap_or_default())?)
    }

    #[test]
    fn test_unacknowledged_ticket_challenge_response() -> anyhow::Result<()> {
        let hk1 = HalfKey::try_from(hex!("3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa").as_ref())?;

        let hk2 = HalfKey::try_from(hex!("4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b").as_ref())?;

        let challenge = Response::from_half_keys(&hk1, &hk2)?.to_challenge()?;

        let dst = Hash::default();
        let ack = mock_ticket(
            &ALICE,
            &BOB.public().to_address(),
            Some(dst),
            Some(challenge.to_ethereum_challenge()),
        )?
        .into_unacknowledged(hk1)
        .acknowledge(&hk2)?;

        assert!(ack.is_winning(&BOB, &dst), "ticket must be winning");
        Ok(())
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_acknowledged_ticket_serde() -> anyhow::Result<()> {
        let response =
            Response::try_from(hex!("876a41ee5fb2d27ac14d8e8d552692149627c2f52330ba066f9e549aef762f73").as_ref())?;

        let dst = Hash::default();

        let ticket = mock_ticket(
            &ALICE,
            &BOB.public().to_address(),
            Some(dst),
            Some(response.to_challenge()?.to_ethereum_challenge()),
        )?;

        let acked_ticket = ticket.into_acknowledged(response);

        let mut deserialized_ticket = bincode::serde::decode_from_slice(
            &bincode::serde::encode_to_vec(&acked_ticket, BINCODE_CONFIGURATION)?,
            BINCODE_CONFIGURATION,
        )
        .map(|v| v.0)?;
        assert_eq!(acked_ticket, deserialized_ticket);

        assert!(deserialized_ticket.is_winning(&BOB, &dst));

        deserialized_ticket.status = AcknowledgedTicketStatus::BeingRedeemed;

        assert_eq!(
            deserialized_ticket,
            bincode::serde::decode_from_slice(
                &bincode::serde::encode_to_vec(&deserialized_ticket, BINCODE_CONFIGURATION)?,
                BINCODE_CONFIGURATION,
            )
            .map(|v| v.0)?
        );
        Ok(())
    }

    #[test]
    fn test_ticket_entire_ticket_transfer_flow() -> anyhow::Result<()> {
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();
        let resp = Response::from_half_keys(&hk1, &hk2)?;

        let verified = TicketBuilder::default()
            .direction(&ALICE.public().to_address(), &BOB.public().to_address())
            .balance(1.into())
            .index(0)
            .win_prob(1.0.try_into()?)
            .channel_epoch(1)
            .challenge(resp.to_challenge()?)
            .build_signed(&ALICE, &Default::default())?;

        let unack = verified.into_unacknowledged(hk1);
        let acknowledged = unack.acknowledge(&hk2).expect("should acknowledge");

        let redeemable_1 = acknowledged.clone().into_redeemable(&BOB, &Hash::default())?;

        let transferable = acknowledged.into_transferable(&BOB, &Hash::default())?;

        let redeemable_2 = transferable.into_redeemable(&ALICE.public().to_address(), &Hash::default())?;

        assert_eq!(redeemable_1, redeemable_2);
        assert_eq!(redeemable_1.vrf_params.V, redeemable_2.vrf_params.V);
        Ok(())
    }
}
