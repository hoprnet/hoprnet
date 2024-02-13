use crate::errors::{CoreTypesError, Result};
use bindings::hopr_channels::RedeemTicketCall;
use ethers::contract::EthCall;
use hex_literal::hex;
use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize, Serialize,
};
use serde_repr::*;
use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
};

/// Size-optimized encoding of the ticket, used for both,
/// network transfer and in the smart contract.
const ENCODED_TICKET_LENGTH: usize = 64;

/// Custom float to integer encoding used in the integer-only
/// Ethereum Virtual Machine (EVM). Chosen to be easily
/// convertible to IEEE754 double-precision and vice versa
const ENCODED_WIN_PROB_LENGTH: usize = 7;

pub type EncodedWinProb = [u8; ENCODED_WIN_PROB_LENGTH];

/// Encodes 100% winning probability
const ALWAYS_WINNING: EncodedWinProb = hex!("ffffffffffffff");

/// Encodes 0% winning probabilitiy
const NEVER_WINNING: EncodedWinProb = hex!("00000000000000");

/// Describes status of a channel
#[repr(u8)]
#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, Serialize_repr, Deserialize_repr, strum::Display, strum::EnumString,
)]
#[strum(serialize_all = "PascalCase")]
pub enum ChannelStatus {
    #[default]
    Closed = 0,
    Open = 1,
    PendingToClose = 2,
}

impl TryFrom<u8> for ChannelStatus {
    type Error = GeneralError;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Closed),
            1 => Ok(Self::Open),
            2 => Ok(Self::PendingToClose),
            _ => Err(GeneralError::ParseError),
        }
    }
}

/// Describes a direction of node's own channel.
/// The direction of a channel that is not own is undefined.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum ChannelDirection {
    /// The other party is initiator of the channel.
    Incoming = 0,
    /// Our own node is the initiator of the channel.
    Outgoing = 1,
}

/// Overall description of a channel
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChannelEntry {
    pub source: Address,
    pub destination: Address,
    pub balance: Balance,
    pub ticket_index: U256,
    pub status: ChannelStatus,
    pub channel_epoch: U256,
    pub closure_time: U256,
    id: Hash,
}

impl ChannelEntry {
    pub fn new(
        source: Address,
        destination: Address,
        balance: Balance,
        ticket_index: U256,
        status: ChannelStatus,
        channel_epoch: U256,
        closure_time: U256,
    ) -> Self {
        assert_eq!(BalanceType::HOPR, balance.balance_type(), "invalid balance currency");
        ChannelEntry {
            source,
            destination,
            balance,
            ticket_index,
            status,
            channel_epoch,
            closure_time,
            id: generate_channel_id(&source, &destination),
        }
    }

    /// Generates the channel ID using the source and destination address
    pub fn get_id(&self) -> Hash {
        self.id
    }

    /// Checks if the closure time of this channel has passed.
    /// Also returns `false` if the channel closure has not been initiated.
    pub fn closure_time_passed(&self, current_timestamp_ms: u64) -> bool {
        self.remaining_closure_time(current_timestamp_ms)
            .map(|remaining| remaining == 0)
            .unwrap_or(false)
    }

    /// Calculates the remaining channel closure grace period in seconds.
    /// Returns `None` if the channel closure has not been initiated yet.
    pub fn remaining_closure_time(&self, current_timestamp_ms: u64) -> Option<u64> {
        assert!(current_timestamp_ms > 0, "invalid timestamp");
        // round clock ms to seconds
        let now_seconds = current_timestamp_ms / 1000_u64;

        self.closure_time
            .gt(&0_u64.into())
            .then(|| self.closure_time.as_u64().saturating_sub(now_seconds))
    }
}

impl ChannelEntry {
    /// Determines the channel direction given the self address.
    /// Returns `None` if neither source nor destination are equal to `me`.
    pub fn direction(&self, me: &Address) -> Option<ChannelDirection> {
        if self.source.eq(me) {
            Some(ChannelDirection::Outgoing)
        } else if self.destination.eq(me) {
            Some(ChannelDirection::Incoming)
        } else {
            None
        }
    }

    /// Determines the channel's direction and counterparty relative to `me`.
    /// Returns `None` if neither source nor destination are equal to `me`.
    pub fn orientation(&self, me: &Address) -> Option<(ChannelDirection, Address)> {
        if self.source.eq(me) {
            Some((ChannelDirection::Outgoing, self.destination))
        } else if self.destination.eq(me) {
            Some((ChannelDirection::Incoming, self.source))
        } else {
            None
        }
    }
}

impl Display for ChannelEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} channel {}", self.status, self.get_id(),)
    }
}

impl BinarySerializable for ChannelEntry {
    const SIZE: usize = Address::SIZE + Address::SIZE + U256::SIZE + U256::SIZE + 1 + U256::SIZE + U256::SIZE;

    fn from_bytes(data: &[u8]) -> hopr_primitive_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            let mut b = data.to_vec();
            let source = Address::from_bytes(b.drain(0..Address::SIZE).as_ref())?;
            let destination = Address::from_bytes(b.drain(0..Address::SIZE).as_ref())?;
            let balance = Balance::new(U256::from_bytes(b.drain(0..U256::SIZE).as_ref())?, BalanceType::HOPR);
            let ticket_index = U256::from_bytes(b.drain(0..U256::SIZE).as_ref())?;
            let status = ChannelStatus::try_from(b.drain(0..1).as_ref()[0])?;
            let channel_epoch = U256::from_bytes(b.drain(0..U256::SIZE).as_ref())?;
            let closure_time = U256::from_bytes(b.drain(0..U256::SIZE).as_ref())?;
            Ok(Self::new(
                source,
                destination,
                balance,
                ticket_index,
                status,
                channel_epoch,
                closure_time,
            ))
        } else {
            Err(hopr_primitive_types::errors::GeneralError::ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::<u8>::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.source.as_slice());
        ret.extend_from_slice(&self.destination.as_slice());
        ret.extend_from_slice(self.balance.amount().to_bytes().as_ref());
        ret.extend_from_slice(self.ticket_index.to_bytes().as_ref());
        ret.push(self.status as u8);
        ret.extend_from_slice(self.channel_epoch.to_bytes().as_ref());
        ret.extend_from_slice(self.closure_time.to_bytes().as_ref());
        ret.into_boxed_slice()
    }
}

/// Computes the identifier for a payment channel.
///
/// Internally this is done by computing `hash(source, destination)`.
/// This ensures that channel_id for a channel `source -> destination`
/// has a different channel_id than `destination -> source`.
///
/// ```rust
/// # use hex_literal::hex;
/// # use hopr_crypto_types::prelude::*;
/// # use hopr_internal_types::prelude::*;
/// # use hopr_primitive_types::prelude::*;
///
/// let ALICE = Address::from_bytes(&hex!("0f8c01194e2ea6298690e035653cd4ce2032ca71")).unwrap();
/// let BOB = Address::from_bytes(&hex!("79befcde8a692a2b6d9d99de05346db535124bc9")).unwrap();
///
/// let ALICE_BOB = generate_channel_id(&ALICE, &BOB);
/// let BOB_ALICE = generate_channel_id(&BOB, &ALICE);
///
/// assert_ne!(ALICE_BOB, BOB_ALICE);
/// ```
pub fn generate_channel_id(source: &Address, destination: &Address) -> Hash {
    Hash::create(&[&source.as_slice(), &destination.as_slice()])
}

/// Enumerates possible changes on a channel entry update
#[derive(Clone, Copy, Debug)]
pub enum ChannelChange {
    /// Channel status has changed
    Status { left: ChannelStatus, right: ChannelStatus },

    /// Channel balance has changed
    CurrentBalance { left: Balance, right: Balance },

    /// Channel epoch has changed
    Epoch { left: u32, right: u32 },

    /// Ticket index has changed
    TicketIndex { left: u64, right: u64 },
}

impl Display for ChannelChange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelChange::Status { left, right } => {
                write!(f, "Status: {left} -> {right}")
            }

            ChannelChange::CurrentBalance { left, right } => {
                write!(f, "Balance: {left} -> {right}")
            }

            ChannelChange::Epoch { left, right } => {
                write!(f, "Epoch: {left} -> {right}")
            }

            ChannelChange::TicketIndex { left, right } => {
                write!(f, "TicketIndex: {left} -> {right}")
            }
        }
    }
}

impl ChannelChange {
    /// Compares the two given channels and returns a vector of `ChannelChange`s
    /// Both channels must have the same ID (source,destination and direction) to be comparable using this function.
    /// The function panics if `left` and `right` do not have equal ids.
    /// Note that only some fields are tracked for changes, and therefore an empty vector returned
    /// does not imply the two `ChannelEntry` instances are equal.
    pub fn diff_channels(left: &ChannelEntry, right: &ChannelEntry) -> Vec<Self> {
        assert_eq!(left.id, right.id, "must have equal ids"); // misuse
        let mut ret = Vec::with_capacity(4);
        if left.status != right.status {
            ret.push(ChannelChange::Status {
                left: left.status,
                right: right.status,
            });
        }

        if left.balance != right.balance {
            ret.push(ChannelChange::CurrentBalance {
                left: left.balance,
                right: right.balance,
            });
        }

        if left.channel_epoch != right.channel_epoch {
            ret.push(ChannelChange::Epoch {
                left: left.channel_epoch.as_u32(),
                right: right.channel_epoch.as_u32(),
            });
        }

        if left.ticket_index != right.ticket_index {
            ret.push(ChannelChange::TicketIndex {
                left: left.ticket_index.as_u64(),
                right: right.ticket_index.as_u64(),
            })
        }

        ret
    }
}

/// Checks the validity of a ticket. Returns `Ok(())` if all values
/// are in the boundaries allowed by the smart contract and if the
/// signature is valid.
///
/// However, it does not check if the ticket is a win.
pub fn validate_ticket(ticket: &Ticket, destination: &Address, domain_separator: &Hash) -> Result<()> {
    // @TODO fail if win_prob > 50%, see
    // https://github.com/hoprnet/hoprnet/issues/5985

    if ticket.channel_id == generate_channel_id(destination, destination) {
        return Err(CoreTypesError::InvalidInputData(
            "Cannot produce loopback tickets".into(),
        ));
    }

    if ticket.amount.balance_type() != BalanceType::HOPR {
        return Err(CoreTypesError::InvalidInputData(
            "Tickets can only have HOPR balance".into(),
        ));
    }

    if ticket.amount.amount() >= 10u128.pow(25).into() {
        return Err(CoreTypesError::InvalidInputData(
            "Tickets may not have more than 1% of total supply".into(),
        ));
    }

    if ticket.index >= 1u64 << 48 {
        return Err(CoreTypesError::InvalidInputData(
            "Cannot hold ticket indices larger than 2^48".into(),
        ));
    }

    // Not checked by Rust because data is represented in larger structs
    if ticket.index + ticket.index_offset as u64 >= 1u64 << 48 {
        return Err(CoreTypesError::InvalidInputData(
            "Ticket index + ticket index offset exceed smart contract boundaries".into(),
        ));
    }

    if ticket.channel_epoch >= 1u32 << 24 {
        return Err(CoreTypesError::InvalidInputData(
            "Cannot hold channel epoch larger than 2^24".into(),
        ));
    }

    // Check signature part
    let recovered_address = Ticket::recover_issuer_address(ticket, domain_separator)?;

    let computed_channel_id = generate_channel_id(&recovered_address, destination);

    if ticket.channel_id != computed_channel_id {
        return Err(CoreTypesError::InvalidInputData("Invalid ticket signature".into()));
    }

    Ok(())
}

/// Implements the probabilistic payment scheme that is used to
/// claim incentives within the smart contract.
///
/// ```rust
/// # use hex_literal::hex;
/// # use hopr_crypto_types::prelude::*;
/// # use hopr_internal_types::prelude::*;
/// # use hopr_primitive_types::prelude::*;
/// # use std::str::FromStr;
///
/// let ALICE = ChainKeypair::from_secret(&hex!("bec16a36a1e4b9035afcebd4e87a3d6e2e515a6178658d897ab7da8549f46921")).unwrap();
/// let BOB = ChainKeypair::from_secret(&hex!("763a128abb7be6327173763392c25b1c1bd1cf6f41e1ef025485ad72eb2e1c77")).unwrap();
///
/// // prevents tickets issued for one smart contract issued being replayed
/// // on differen smart contract or on a different EVM-compatible blockchain
/// let DOMAIN_SEPARATOR: Hash = hex!("02b2631423a4a6f5cc01657d184ffd8ad3e875af9460dffc506a5133c64d949b").into();
///
/// // randomly sampled, leads to a winning ticket
/// let por_response: Response = hex!("e7c980ada1cb92530e3b2c7cfb82aa372cef1aa94dcc57de5add7d17573025c9").into();
///
/// let mut ticket = Ticket::new_unsigned(
///     &ALICE.public().to_address(),
///     &BOB.public().to_address(),
///     &Balance::from_str("1 HOPR").unwrap(),
///     1u64.into(),
///     1u64.into(),
///     0.5f64, // 50% win probability
///     1u64.into(),
///     &por_response.to_challenge().to_ethereum_challenge(),
/// );
///
/// // unsigned tickets are not valid
/// assert!(validate_ticket(&ticket, &BOB.public().to_address(), &DOMAIN_SEPARATOR).is_err());
///
/// ticket.sign(&ALICE, &DOMAIN_SEPARATOR);
///
/// assert!(validate_ticket(&ticket, &BOB.public().to_address(), &DOMAIN_SEPARATOR).is_ok());
///
/// let ticket_hash = ticket.get_hash(&DOMAIN_SEPARATOR);
///
/// let vrf_values = Ticket::get_vrf_values(&ticket_hash, &BOB, &DOMAIN_SEPARATOR).unwrap();
///
/// assert!(Ticket::is_winning(&ticket, &vrf_values, &por_response, &DOMAIN_SEPARATOR).unwrap());
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct Ticket {
    /// identifier of the channel in which the ticket is valid
    ///
    /// Encoded using 32 bytes on-chain
    pub channel_id: Hash,
    /// tokens that are released to the redeeming party if the
    /// ticket turns out to be win and is redeemed on-chain
    ///
    /// Encoded using 92 bytes on-chain
    pub amount: Balance,
    /// serial number, makes each ticket in the channel unique
    ///
    /// Encoded using 8 bytes on-chain
    pub index: u64,
    /// allows aggregation of tickets, set to values > 1 for
    /// aggregated tickets
    ///
    /// Encoded using 4 bytes on-chain
    pub index_offset: u32,
    /// raw encoded ticket winning probability, reaches from
    /// `0x00000000000000` (0%) to `0xffffffffffffff` (100%)
    ///
    /// Encoded using 7 bytes on-chain
    pub encoded_win_prob: EncodedWinProb,
    /// make each channel incarnation unique, closing -> opening
    /// the payment channel increases channel epoch
    ///
    /// Encoded using 3 bytes on-chain
    pub channel_epoch: u32,
    /// part of a challenge-response scheme to be fulfilled
    /// in order to claim the tickets' incentives
    ///
    /// Encoded using 32 bytes on-chain
    pub challenge: EthereumChallenge,
    /// digital signature produced by the issuer of the ticket,
    /// necessary to prove origin of the ticket
    ///
    /// Encoded using 64 bytes on-chain
    pub signature: Signature,
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

// Use compact serialization for ticket as they are used very often
impl Serialize for Ticket {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.to_bytes_internal(true))
    }
}

struct TicketVisitor {}

impl<'de> Visitor<'de> for TicketVisitor {
    type Value = Ticket;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_fmt(format_args!("a byte-array with {} elements", Ticket::SIZE))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ticket::from_bytes(v).map_err(|e| de::Error::custom(e.to_string()))
    }
}

// Use compact deserialization for tickets as they are used very often
impl<'de> Deserialize<'de> for Ticket {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(TicketVisitor {})
    }
}

impl Default for Ticket {
    fn default() -> Self {
        Self {
            channel_id: Hash::default(),
            amount: Balance::new(U256::zero(), BalanceType::HOPR),
            index: 0u64,
            index_offset: 1u32,
            encoded_win_prob: f64_to_win_prob(0.5f64).expect("failed creating 50% winning probability"),
            channel_epoch: 1u32,
            challenge: EthereumChallenge::default(),
            signature: Signature::new(&[0u8; Signature::SIZE], 27),
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

impl std::fmt::Debug for Ticket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ticket")
            .field("channel_id", &self.channel_id)
            .field("amount", &self.amount)
            .field("index", &self.index)
            .field("index_offset", &self.index_offset)
            .field("win_prob", &format!("{}%", (&self.win_prob() * 100.0)))
            .field("channel_epoch", &self.channel_epoch)
            .field("challenge", &self.challenge)
            .field("signature", &self.signature.to_hex())
            .finish()
    }
}

impl Ticket {
    /// Creates an unsigned ticket. Note that this ticket is
    /// not a valid ticket.
    pub fn new_unsigned(
        source: &Address,
        destination: &Address,
        amount: &Balance,
        index: U256,
        index_offset: U256,
        win_prob: f64,
        channel_epoch: U256,
        challenge: &EthereumChallenge,
    ) -> Self {
        let channel_id = generate_channel_id(&source, destination);

        Self {
            channel_id,
            amount: amount.to_owned(),
            index: index.as_u64(),
            index_offset: index_offset.as_u32(),
            channel_epoch: channel_epoch.as_u32(),
            challenge: challenge.to_owned(),
            encoded_win_prob: f64_to_win_prob(win_prob).expect("error encoding winning probability"),
            signature: Signature::new(&[0u8; Signature::SIZE], 0),
        }
    }
    /// Creates a signed ticket using the given signing key. This
    /// method does not check the validity of the given values.
    ///
    /// Use `validate_ticket()` to check the tickets' validity.
    #[allow(clippy::too_many_arguments)] // TODO: Refactor to use less inputs
    pub fn new(
        counterparty: &Address,
        amount: &Balance,
        index: U256,
        index_offset: U256,
        win_prob: f64,
        channel_epoch: U256,
        challenge: &EthereumChallenge,
        signing_key: &ChainKeypair,
        domain_separator: &Hash,
    ) -> Ticket {
        let own_address = signing_key.public().to_address();

        let mut ret = Ticket::new_unsigned(
            &own_address,
            counterparty,
            amount,
            index,
            index_offset,
            win_prob,
            channel_epoch,
            challenge,
        );

        ret.sign(signing_key, domain_separator);

        ret
    }

    /// Creates a ticket with signature attached. Usually used for
    /// deserialization.
    #[allow(clippy::too_many_arguments)] // TODO: Refactor the function to take either less arguments or is more straigtforward
    pub fn new_with_signature(
        own_address: &Address,
        counterparty: &Address,
        amount: &Balance,
        index: U256,
        index_offset: U256,
        encoded_win_prob: EncodedWinProb,
        channel_epoch: U256,
        challenge: EthereumChallenge,
        signature: Signature,
    ) -> Self {
        let channel_id = generate_channel_id(own_address, counterparty);

        Self {
            channel_id,
            amount: amount.to_owned(),
            index: index.as_u64(),
            index_offset: index_offset.as_u32(),
            channel_epoch: channel_epoch.as_u32(),
            challenge,
            encoded_win_prob,
            signature,
        }
    }

    /// Encode winning probability such that it can get used in
    /// the smart contract
    pub fn win_prob(&self) -> f64 {
        win_prob_to_f64(&self.encoded_win_prob)
    }

    /// Serializes the ticket with or without signature
    ///
    /// Use cases:
    /// - Signing: computing the ticket hash requires serialization
    ///            without a signature
    /// - Transport: when sending the ticket over the wire or storing
    ///            it in the database, serialization is done with the
    ///            signature attacked
    fn to_bytes_internal(&self, with_signature: bool) -> Vec<u8> {
        let mut ret = Vec::<u8>::with_capacity(if with_signature {
            Self::SIZE
        } else {
            Self::SIZE - Signature::SIZE
        });

        ret.extend_from_slice(&self.channel_id.as_slice());
        ret.extend_from_slice(&self.amount.amount().to_bytes()[20..32]);
        ret.extend_from_slice(&self.index.to_be_bytes()[2..8]);
        ret.extend_from_slice(&self.index_offset.to_be_bytes());
        ret.extend_from_slice(&self.channel_epoch.to_be_bytes()[1..4]);
        ret.extend_from_slice(&self.encoded_win_prob);
        ret.extend_from_slice(&self.challenge.as_slice());

        if with_signature {
            ret.extend_from_slice(&self.signature.to_bytes());
        }

        ret
    }

    /// Computes Ethereum signature hash of the ticket, yields the
    /// same value as the on-chain implementation.
    pub fn get_hash(&self, domain_separator: &Hash) -> Hash {
        let ticket_hash = Hash::create(&[&self.to_bytes_internal(false)]); // cannot fail
        let hash_struct = Hash::create(&[&RedeemTicketCall::selector(), &[0u8; 28], ticket_hash.as_slice()]);
        Hash::create(&[&hex!("1901"), domain_separator.as_slice(), hash_struct.as_slice()])
    }

    /// Signs the ticket using the given private key.
    pub fn sign(&mut self, signing_key: &ChainKeypair, domain_separator: &Hash) {
        self.signature = Signature::sign_hash(self.get_hash(domain_separator).as_slice(), signing_key);
    }

    /// Computes the VRF values to check or prove that the ticket
    /// is a win. Called by the ticket recipient.
    ///
    /// ```rust
    /// # use hex_literal::hex;
    /// # use hopr_crypto_types::prelude::*;
    /// # use hopr_internal_types::prelude::*;
    /// # use std::str::FromStr;
    ///
    /// let ALICE = ChainKeypair::from_secret(&hex!("bec16a36a1e4b9035afcebd4e87a3d6e2e515a6178658d897ab7da8549f46921")).unwrap();
    /// let DOMAIN_SEPARATOR: Hash = hex!("6cbc3c380cdb774b842190a2ccea4fedb6bc474a47d303da09f2206bde942b0c").into();
    ///
    /// // random example
    /// let ticket_hash: Hash = hex!("94510e10dabb4cd3264ef0d9080add1f07bd8b823281023c542058b31ca37baa").into();
    ///
    /// assert!(Ticket::get_vrf_values(&ticket_hash, &ALICE, &DOMAIN_SEPARATOR).is_ok());
    /// ```
    pub fn get_vrf_values(
        ticket_hash: &Hash,
        chain_key: &ChainKeypair,
        domain_separator: &Hash,
    ) -> Result<VrfParameters> {
        derive_vrf_parameters(&ticket_hash.into(), chain_key, domain_separator.as_slice()).map_err(|e| e.into())
    }

    /// Computes the value which is used to determine
    /// if a ticket is win.
    ///
    /// Each ticket specifies a probability, given as an integer in
    /// [0, 2**56 - 1] where 0 -> 0% and 2*56 - 1 -> 100% win
    /// probability. If the ticket's luck value is greater than
    /// the stated probability, it is considered a winning ticket.
    ///
    /// Requires access to the private key to compute the VRF values.
    pub fn get_luck(
        ticket_hash: &Hash,
        signature: &Signature,
        vrf_params: &VrfParameters,
        response: &Response,
    ) -> Result<[u8; 7]> {
        let mut luck = [0u8; 7];

        luck.copy_from_slice(
            &Hash::create(&[
                ticket_hash.as_slice(),
                &vrf_params.get_decompressed_v()?.to_bytes()[1..], // skip prefix
                response.as_slice(),
                &signature.to_bytes(),
            ])
            .as_slice()[0..7],
        );

        // clone bytes
        Ok(luck)
    }

    /// Checks if this ticket is considered a win.
    ///
    /// Computes the ticket's luck value and compares it against the
    /// ticket's probability. If luck <= probability, the ticket is
    /// considered a win.
    ///
    /// Requires access to the private key to compute the VRF values.
    pub fn is_winning(
        ticket: &Ticket,
        vrf_params: &VrfParameters,
        response: &Response,
        domain_separator: &Hash,
    ) -> Result<bool> {
        let mut signed_ticket_luck = [0u8; 8];
        signed_ticket_luck[1..].copy_from_slice(&ticket.encoded_win_prob);

        let mut computed_ticket_luck = [0u8; 8];
        computed_ticket_luck[1..].copy_from_slice(&Ticket::get_luck(
            &ticket.get_hash(domain_separator),
            &ticket.signature,
            vrf_params,
            response,
        )?);
        Ok(u64::from_be_bytes(computed_ticket_luck) <= u64::from_be_bytes(signed_ticket_luck))
    }

    /// Convenience method for creating a zero-hop ticket. Zero-hop
    /// tickets do not lead to a payout and act as junk traffic between
    /// nodes.
    pub fn new_zero_hop(destination: &Address, private_key: &ChainKeypair, domain_separator: &Hash) -> Self {
        Self::new(
            destination,
            &Balance::new(0_u32, BalanceType::HOPR),
            U256::zero(),
            U256::zero(),
            0.0,
            U256::zero(),
            &EthereumChallenge::default(),
            private_key,
            domain_separator,
        )
    }

    /// Based on the price of this ticket, determines the path position (hop number) this ticket
    /// relates to.
    ///
    /// Does not support path lengths greater than 255
    pub fn get_path_position(&self, price_per_packet: U256) -> Result<u8> {
        (self.get_expected_payout() / price_per_packet)
            .as_u64()
            .try_into() // convert to u8
            .map_err(|_| {
                CoreTypesError::ArithmeticError(format!(
                    "Cannot convert {} to u8",
                    price_per_packet / self.get_expected_payout()
                ))
            })
    }

    /// Determines the amount of tokens the ticket is worth
    /// if it were a win and redeemed on-chain.
    pub fn get_expected_payout(&self) -> U256 {
        let mut win_prob = [0u8; 8];
        win_prob[1..].copy_from_slice(&self.encoded_win_prob);

        // Add + 1 to project interval [0x00ffffffffffff, 0x00000000000000] to [0x00000000000001, 0x01000000000000]
        // Add + 1 to "round to next integer"
        let win_prob = (u64::from_be_bytes(win_prob) >> 4) + 1 + 1;

        (self.amount.amount() * U256::from(win_prob)) >> U256::from(52u64)
    }

    /// Recovers Ethereum address of the signer from the embedded ticket signature.
    ///
    /// This is possible due this specific instantiation of the ECDSA over the secp256k1 curve.
    pub fn recover_issuer_address(ticket: &Ticket, domain_separator: &Hash) -> Result<Address> {
        Ok(
            PublicKey::from_signature_hash(&ticket.get_hash(domain_separator).as_slice(), &ticket.signature)?
                .to_address(),
        )
    }

    /// Determines if the ticket has been aggregated, i.e. is the result
    /// of combining multiple tickets.
    ///
    /// Aggregated tickets will always have a ticket interval length > 1
    /// since aggregation adds subsequent intervals.
    /// Aggregated tickets will always have 100% winning probability
    /// since they are the combination of multiple winning tickets.
    pub fn is_aggregated(&self) -> bool {
        // Aggregated tickets have always an index offset > 1
        self.index_offset > 1 && self.encoded_win_prob == ALWAYS_WINNING
    }
}

impl BinarySerializable for Ticket {
    const SIZE: usize = ENCODED_TICKET_LENGTH + EthereumChallenge::SIZE + Signature::SIZE;

    /// Tickets get sent next to packets, so they need to be as small as possible.
    /// Transmitting tickets to the next downstream shares the same binary representation
    /// as used in the smart contract.
    fn from_bytes(data: &[u8]) -> hopr_primitive_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            // TODO: not necessary to transmit over the wire
            let channel_id = Hash::from_bytes(&data[0..32])?;
            let mut amount = [0u8; 32];
            amount[20..32].copy_from_slice(&data[Hash::SIZE..Hash::SIZE + 12]);

            let mut index = [0u8; 8];
            index[2..8].copy_from_slice(&data[Hash::SIZE + 12..Hash::SIZE + 12 + 6]);

            let mut index_offset = [0u8; 4];
            index_offset.copy_from_slice(&data[Hash::SIZE + 12 + 6..Hash::SIZE + 12 + 6 + 4]);

            let mut channel_epoch = [0u8; 4];
            channel_epoch[1..4].copy_from_slice(&data[Hash::SIZE + 12 + 6 + 4..Hash::SIZE + 12 + 6 + 4 + 3]);

            let mut encoded_win_prob = [0u8; 7];
            encoded_win_prob.copy_from_slice(&data[Hash::SIZE + 12 + 6 + 4 + 3..Hash::SIZE + 12 + 6 + 4 + 3 + 7]);

            let challenge = EthereumChallenge::from_bytes(
                &data[ENCODED_TICKET_LENGTH..ENCODED_TICKET_LENGTH + EthereumChallenge::SIZE],
            )?;

            let signature = Signature::from_bytes(
                &data[ENCODED_TICKET_LENGTH + EthereumChallenge::SIZE
                    ..ENCODED_TICKET_LENGTH + EthereumChallenge::SIZE + Signature::SIZE],
            )?;

            Ok(Self {
                channel_id,
                amount: Balance::new(U256::from_bytes(&amount)?, BalanceType::HOPR),
                index: u64::from_be_bytes(index),
                index_offset: u32::from_be_bytes(index_offset),
                encoded_win_prob,
                channel_epoch: u32::from_be_bytes(channel_epoch),
                challenge,
                signature,
            })
        } else {
            // TODO: make Error a generic
            Err(hopr_primitive_types::errors::GeneralError::ParseError)
        }
    }

    /// Serializes the ticket to be transmitted to the next downstream node or handled by the
    /// smart contract
    fn to_bytes(&self) -> Box<[u8]> {
        self.to_bytes_internal(true).into_boxed_slice()
    }
}

/// Decodes [0x00000000000000, 0xffffffffffffff] to [0.0f64, 1.0f64]
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
pub fn f64_to_win_prob(win_prob: f64) -> Result<EncodedWinProb> {
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

#[cfg(test)]
pub mod tests {
    use crate::channels::{f64_to_win_prob, generate_channel_id, ChannelEntry, ChannelStatus, Ticket};
    use hex_literal::hex;
    use hopr_crypto_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use std::str::FromStr;

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).unwrap();
        static ref ALICE_ADDR: Address = ALICE.public().to_address();
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).unwrap();
        static ref BOB_ADDR: Address = BOB.public().to_address();
    }

    #[test]
    pub fn test_generate_id() {
        let from = Address::from_str("0xa460f2e47c641b64535f5f4beeb9ac6f36f9d27c").unwrap();
        let to = Address::from_str("0xb8b75fef7efdf4530cf1688c933d94e4e519ccd1").unwrap();
        let id = generate_channel_id(&from, &to).to_string();
        assert_eq!("0x1a410210ce7265f3070bf0e8885705dce452efcfbd90a5467525d136fcefc64a", id);
    }

    #[test]
    pub fn channel_entry_test() {
        let ce1 = ChannelEntry::new(
            *ALICE_ADDR,
            *BOB_ADDR,
            Balance::new(10_u64, BalanceType::HOPR),
            23u64.into(),
            ChannelStatus::PendingToClose,
            3u64.into(),
            4u64.into(),
        );

        let ce2 = ChannelEntry::from_bytes(&ce1.to_bytes()).unwrap();
        assert_eq!(ce1, ce2, "deserialized channel entry does not match");
    }

    #[test]
    pub fn channel_status_test() {
        let cs1 = ChannelStatus::Open;
        let cs2 = ChannelStatus::try_from(cs1 as u8).unwrap();

        assert!(ChannelStatus::try_from(231_u8).is_err());
        assert_eq!(cs1, cs2, "channel status does not match");
    }

    #[test]
    pub fn channel_entry_closure_time() {
        let mut ce = ChannelEntry::new(
            *ALICE_ADDR,
            *BOB_ADDR,
            Balance::new(10_u64, BalanceType::HOPR),
            23u64.into(),
            ChannelStatus::Open,
            3u64.into(),
            0u64.into(),
        );

        assert!(!ce.closure_time_passed(10));
        assert!(ce.remaining_closure_time(10).is_none());

        ce.closure_time = 12_u32.into();

        assert!(!ce.closure_time_passed(10000));
        assert_eq!(2, ce.remaining_closure_time(10000).expect("must have closure time"));

        assert!(ce.closure_time_passed(14000));
        assert_eq!(0, ce.remaining_closure_time(14000).expect("must have closure time"));
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
            &BOB_ADDR,
            &Balance::new(U256::one(), BalanceType::HOPR),
            U256::zero(),
            U256::one(),
            0.5,
            U256::one(),
            &EthereumChallenge::default(),
            &ALICE,
            &Hash::default(),
        );

        assert!(super::validate_ticket(&initial_ticket, &BOB_ADDR, &Hash::default()).is_ok());

        assert_ne!(*initial_ticket.get_hash(&Hash::default()).as_slice(), [0u8; Hash::SIZE]);

        assert_eq!(initial_ticket, Ticket::from_bytes(&initial_ticket.to_bytes()).unwrap());
    }

    #[test]
    pub fn test_ticket_serialize_deserialize_serde() {
        let initial_ticket = super::Ticket::new(
            &BOB_ADDR,
            &Balance::new(U256::one(), BalanceType::HOPR),
            U256::zero(),
            U256::one(),
            0.5,
            U256::one(),
            &EthereumChallenge::default(),
            &ALICE,
            &Hash::default(),
        );

        assert!(super::validate_ticket(&initial_ticket, &BOB_ADDR, &Hash::default()).is_ok());

        assert_eq!(
            initial_ticket,
            bincode::deserialize(&bincode::serialize(&initial_ticket).unwrap()).unwrap()
        );
    }

    #[test]
    pub fn test_ticket_sign_verify() {
        let initial_ticket = super::Ticket::new(
            &BOB_ADDR,
            &Balance::new(U256::one(), BalanceType::HOPR),
            U256::zero(),
            U256::one(),
            0.5,
            U256::one(),
            &EthereumChallenge::default(),
            &ALICE,
            &Hash::default(),
        );

        assert!(super::validate_ticket(&initial_ticket, &BOB_ADDR, &Hash::default()).is_ok());

        assert_ne!(*initial_ticket.get_hash(&Hash::default()).as_slice(), [0u8; Hash::SIZE]);
    }

    #[test]
    pub fn test_ticket_properties_bad_examples() {
        let bad_index = super::Ticket::new(
            &BOB_ADDR,
            &Balance::new(U256::one(), BalanceType::HOPR),
            ((1u64 << 48) + 1).into(),
            U256::one(),
            0.5,
            U256::one(),
            &EthereumChallenge::default(),
            &ALICE,
            &Hash::default(),
        );

        assert!(super::validate_ticket(&bad_index, &ALICE_ADDR, &Hash::default()).is_err());

        let bad_index_index_offset_sum = super::Ticket::new(
            &BOB_ADDR,
            &Balance::new(U256::one(), BalanceType::HOPR),
            ((1u64 << 48) - 1).into(),
            U256::one(),
            0.5,
            U256::one(),
            &EthereumChallenge::default(),
            &ALICE,
            &Hash::default(),
        );

        assert!(super::validate_ticket(&bad_index_index_offset_sum, &ALICE_ADDR, &Hash::default()).is_err());

        let bad_channel_epoch = super::Ticket::new(
            &BOB_ADDR,
            &Balance::new(U256::one(), BalanceType::HOPR),
            ((1u64 << 48) - 1).into(),
            U256::one(),
            0.5,
            ((1u32 << 24) + 1).into(),
            &EthereumChallenge::default(),
            &ALICE,
            &Hash::default(),
        );

        assert!(super::validate_ticket(&bad_channel_epoch, &ALICE_ADDR, &Hash::default()).is_err());

        // invalid signature
        let bad_signature: Signature = Signature::from_bytes(&[0u8; 64]).unwrap();

        let bad_signature_ticket = Ticket::new_with_signature(
            &ALICE_ADDR,
            &BOB_ADDR,
            &Balance::new(U256::one(), BalanceType::HOPR),
            U256::one(),
            U256::one(),
            [0x7fu8; 7], // 50%
            U256::one(),
            EthereumChallenge::default(),
            bad_signature,
        );

        assert!(super::validate_ticket(&bad_signature_ticket, &ALICE_ADDR, &Hash::default()).is_err());
    }

    #[test]
    pub fn test_ticket_expected_payout() {
        let mut ticket = Ticket::new(
            &BOB_ADDR,
            &Balance::new(U256::one(), BalanceType::HOPR),
            U256::zero(),
            U256::one(),
            1.0f64,
            U256::one(),
            &EthereumChallenge::default(),
            &*ALICE,
            &Hash::default(),
        );

        assert_eq!(U256::one(), ticket.get_expected_payout());

        ticket.encoded_win_prob = f64_to_win_prob(0.0).unwrap();
        assert_eq!(U256::zero(), ticket.get_expected_payout());

        ticket.amount = Balance::new(100000000000_u64, BalanceType::HOPR);
        ticket.encoded_win_prob = f64_to_win_prob(0.00000000001f64).unwrap();

        assert_eq!(U256::one(), ticket.get_expected_payout());
    }

    #[test]
    pub fn test_path_position() {
        let mut ticket = Ticket::new(
            &BOB_ADDR,
            &Balance::new(U256::one(), BalanceType::HOPR),
            U256::zero(),
            U256::one(),
            1.0f64,
            U256::one(),
            &EthereumChallenge::default(),
            &*ALICE,
            &Hash::default(),
        );

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
        let ticket = Ticket::new(
            &BOB_ADDR,
            &Balance::new(256u64, BalanceType::HOPR),
            U256::zero(),
            U256::one(),
            1.0f64,
            U256::one(),
            &EthereumChallenge::default(),
            &*ALICE,
            &Hash::default(),
        );

        assert!(ticket.get_path_position(U256::from(1u64)).is_err());
    }

    #[test]
    pub fn test_zero_hop_ticket() {
        let zero_hop_ticket = Ticket::new_zero_hop(&BOB_ADDR, &ALICE, &Hash::default());
        assert!(super::validate_ticket(&zero_hop_ticket, &BOB_ADDR, &Hash::default()).is_ok());
    }

    #[test]
    pub fn test_zero_hop_loopback_ticket() {
        let zero_hop_ticket = Ticket::new_zero_hop(&ALICE_ADDR, &ALICE, &Hash::default());
        assert!(super::validate_ticket(&zero_hop_ticket, &ALICE_ADDR, &Hash::default()).is_err());
    }
}
