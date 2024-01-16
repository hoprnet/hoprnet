use crate::errors::{CoreTypesError, Result};
use bindings::hopr_channels::RedeemTicketCall;
use enum_iterator::{all, Sequence};
use ethers::contract::EthCall;
use hex_literal::hex;
use hopr_crypto_types::prelude::*;
use hopr_primitive_types::primitives::{Address, Balance, BalanceType, EthereumChallenge, U256};
use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize, Serialize,
};
use serde_repr::*;
use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
};

use hopr_primitive_types::traits::{BinarySerializable, ToHex};

/// Size-optimized encoding of the ticket, used for both,
/// network transfer and in the smart contract.
const ENCODED_TICKET_LENGTH: usize = 64;

pub type EncodedWinProb = [u8; 7];

/// Describes status of a channel
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize_repr, Deserialize_repr, Sequence)]
pub enum ChannelStatus {
    #[default]
    Closed = 0,
    Open = 1,
    PendingToClose = 2,
}

impl ChannelStatus {
    pub fn from_byte(byte: u8) -> Option<Self> {
        all::<ChannelStatus>().find(|v| v.to_byte() == byte)
    }

    pub fn to_byte(&self) -> u8 {
        *self as u8
    }
}

impl Display for ChannelStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelStatus::Closed => write!(f, "Closed"),
            ChannelStatus::Open => write!(f, "Open"),
            ChannelStatus::PendingToClose => write!(f, "PendingToClose"),
        }
    }
}

/// Describes a direction of node's own channel.
/// The direction of a channel that is not own is undefined.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelDirection {
    /// The other party is initiator of the channel.
    Incoming = 0,
    /// Our own node is the initiator of the channel.
    Outgoing = 1,
}

impl Display for ChannelDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelDirection::Incoming => write!(f, "incoming"),
            ChannelDirection::Outgoing => write!(f, "outgoing"),
        }
    }
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
        write!(
            f,
            "{} channel {} ({}): {} -> {}",
            self.status,
            self.get_id(),
            self.closure_time,
            self.source,
            self.destination,
        )
    }
}

impl BinarySerializable for ChannelEntry {
    const SIZE: usize = Address::SIZE + Address::SIZE + Balance::SIZE + U256::SIZE + 1 + U256::SIZE + U256::SIZE;

    fn from_bytes(data: &[u8]) -> hopr_primitive_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            let mut b = data.to_vec();
            let source = Address::from_bytes(b.drain(0..Address::SIZE).as_ref())?;
            let destination = Address::from_bytes(b.drain(0..Address::SIZE).as_ref())?;
            let balance = Balance::deserialize(b.drain(0..Balance::SIZE).as_ref(), BalanceType::HOPR)?;
            let ticket_index = U256::from_bytes(b.drain(0..U256::SIZE).as_ref())?;
            let status = ChannelStatus::from_byte(b.drain(0..1).as_ref()[0])
                .ok_or(hopr_primitive_types::errors::GeneralError::ParseError)?;
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
        ret.extend_from_slice(self.source.to_bytes().as_ref());
        ret.extend_from_slice(self.destination.to_bytes().as_ref());
        ret.extend_from_slice(self.balance.serialize_value().as_ref());
        ret.extend_from_slice(self.ticket_index.to_bytes().as_ref());
        ret.push(self.status as u8);
        ret.extend_from_slice(self.channel_epoch.to_bytes().as_ref());
        ret.extend_from_slice(self.closure_time.to_bytes().as_ref());
        ret.into_boxed_slice()
    }
}

pub fn generate_channel_id(source: &Address, destination: &Address) -> Hash {
    Hash::create(&[&source.to_bytes(), &destination.to_bytes()])
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

/// Contains the overall description of a ticket with a signature
#[derive(Clone, Eq, PartialEq)]
pub struct Ticket {
    pub channel_id: Hash,
    pub amount: Balance,
    pub index: u64,
    pub index_offset: u32,
    pub encoded_win_prob: EncodedWinProb,
    pub channel_epoch: u32,
    pub challenge: EthereumChallenge,
    pub signature: Option<Signature>,
}

impl PartialOrd for Ticket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ticket {
    fn cmp(&self, other: &Self) -> Ordering {
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
        serializer.serialize_bytes(self.to_bytes().as_ref())
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
            encoded_win_prob: f64_to_win_prob(1.0f64).expect("failed creating 100% winning probability"),
            channel_epoch: 1u32,
            challenge: EthereumChallenge::default(),
            signature: None,
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
            .field("signature", &self.signature.as_ref().map(|s| s.to_hex()))
            .finish()
    }
}

impl Ticket {
    /// Creates a new Ticket given the raw Challenge and signs it using the given chain keypair.
    pub fn new(
        counterparty: &Address,
        amount: &Balance,
        index: U256,
        index_offset: U256,
        win_prob: f64,
        channel_epoch: U256,
        challenge: EthereumChallenge,
        signing_key: &ChainKeypair,
        domain_separator: &Hash,
    ) -> Result<Ticket> {
        let own_address = signing_key.public().to_address();

        Ticket::check_value_boundaries(
            &own_address,
            counterparty,
            amount,
            index,
            index_offset,
            win_prob,
            channel_epoch,
        )?;

        let channel_id = generate_channel_id(&own_address, counterparty);

        let mut ret = Ticket {
            channel_id,
            amount: amount.to_owned(),
            index: index.as_u64(),
            index_offset: index_offset.as_u32(),
            channel_epoch: channel_epoch.as_u32(),
            challenge,
            encoded_win_prob: f64_to_win_prob(win_prob).expect("error encoding winning probability"),
            signature: None,
        };
        ret.sign(signing_key, domain_separator);

        Ok(ret)
    }

    /// Creates a ticket with signature attached.
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
        domain_separator: &Hash,
    ) -> Result<Ticket> {
        Ticket::check_value_boundaries(
            own_address,
            counterparty,
            amount,
            index,
            index_offset,
            win_prob_to_f64(&encoded_win_prob),
            channel_epoch,
        )?;

        let channel_id = generate_channel_id(own_address, counterparty);

        let ret = Ticket {
            channel_id,
            amount: amount.to_owned(),
            index: index.as_u64(),
            index_offset: index_offset.as_u32(),
            channel_epoch: channel_epoch.as_u32(),
            challenge,
            encoded_win_prob,
            signature: Some(signature),
        };

        ret.verify(own_address, domain_separator)
            .map_err(|_| CoreTypesError::InvalidInputData("Invalid signature".into()))?;

        Ok(ret)
    }

    /// Creates a ticket *without* signature and *without* a challenge set.
    /// This sets a default value as challenge.
    pub fn new_partial(
        own_address: &Address,
        counterparty: &Address,
        amount: &Balance,
        index: U256,
        index_offset: U256,
        win_prob: f64,
        channel_epoch: U256,
    ) -> Result<Ticket> {
        Ticket::check_value_boundaries(
            own_address,
            counterparty,
            amount,
            index,
            index_offset,
            win_prob,
            channel_epoch,
        )?;

        let channel_id = generate_channel_id(own_address, counterparty);

        Ok(Ticket {
            channel_id,
            amount: amount.to_owned(),
            index: index.as_u64(),
            index_offset: index_offset.as_u32(),
            channel_epoch: channel_epoch.as_u32(),
            challenge: EthereumChallenge::default(),
            encoded_win_prob: f64_to_win_prob(win_prob).expect("error encoding winning probability"),
            signature: None,
        })
    }

    /// Tickets 2.0 come with meaningful boundaries to fit into 2 EVM slots.
    /// This method checks whether they are met and prevents from unintended
    /// usage.
    fn check_value_boundaries(
        own_address: &Address,
        counterparty: &Address,
        amount: &Balance,
        index: U256,
        index_offset: U256,
        win_prob: f64,
        channel_epoch: U256,
    ) -> Result<()> {
        if own_address.eq(counterparty) {
            return Err(CoreTypesError::InvalidInputData(
                "Source and destination must be different".into(),
            ));
        }

        if amount.balance_type().ne(&BalanceType::HOPR) {
            return Err(CoreTypesError::InvalidInputData(
                "Tickets can only have HOPR balance".into(),
            ));
        }

        if amount.value().ge(&10u128.pow(25).into()) {
            return Err(CoreTypesError::InvalidInputData(
                "Tickets may not have more than 1% of total supply".into(),
            ));
        }

        if index.gt(&(1u64 << 48).into()) {
            return Err(CoreTypesError::InvalidInputData(
                "Cannot hold ticket indices larger than 2^48".into(),
            ));
        }

        if index_offset.gt(&(1u64 << 32).into()) {
            return Err(CoreTypesError::InvalidInputData(
                "Cannot hold ticket index offsets larger than 2^32".into(),
            ));
        }

        if channel_epoch.gt(&(1u64 << 24).into()) {
            return Err(CoreTypesError::InvalidInputData(
                "Cannot hold channel epoch larger than 2^24".into(),
            ));
        }

        if win_prob < 0.0 {
            return Err(CoreTypesError::InvalidInputData(
                "Cannot use negative winning ptobability".into(),
            ));
        }

        if win_prob > 1.0 {
            return Err(CoreTypesError::InvalidInputData(
                "Cannot use winning ptobabilities larger than 100%".into(),
            ));
        }

        Ok(())
    }

    /// Add the challenge property and signs the finished ticket afterwards
    pub fn set_challenge(&mut self, challenge: EthereumChallenge, signing_key: &ChainKeypair, domain_separator: &Hash) {
        self.challenge = challenge;
        self.sign(signing_key, domain_separator);
    }

    /// Encode winning probability such that it can get used in
    /// the smart contract
    pub fn win_prob(&self) -> f64 {
        win_prob_to_f64(&self.encoded_win_prob)
    }

    /// Serializes the ticket with or without signature
    ///
    /// Signing requires hashing which requires serialization without signature.
    /// Transferring ticket requires serialization with signature attached.
    fn to_bytes_internal(&self, with_signature: bool) -> Result<Vec<u8>> {
        let mut ret = Vec::<u8>::with_capacity(if with_signature {
            Self::SIZE
        } else {
            Self::SIZE - Signature::SIZE
        });

        ret.extend_from_slice(&self.channel_id.to_bytes());
        ret.extend_from_slice(&self.amount.serialize_value()[20..32]);
        ret.extend_from_slice(&self.index.to_be_bytes()[2..8]);
        ret.extend_from_slice(&self.index_offset.to_be_bytes());
        ret.extend_from_slice(&self.channel_epoch.to_be_bytes()[1..4]);
        ret.extend_from_slice(&self.encoded_win_prob);
        ret.extend_from_slice(&self.challenge.to_bytes());

        if with_signature {
            if let Some(ref signature) = self.signature {
                ret.extend_from_slice(&signature.to_bytes());
            } else {
                return Err(CoreTypesError::ParseError(
                    "Tried to serialize with a non-existing signature".into(),
                ));
            }
        }

        Ok(ret)
    }

    /// Computes Ethereum signature hash of the ticket,
    /// must be equal to on-chain computation
    pub fn get_hash(&self, domain_separator: &Hash) -> Hash {
        let ticket_hash = Hash::create(&[&self.to_bytes_internal(false).unwrap()]); // cannot fail
        let hash_struct = Hash::create(&[&RedeemTicketCall::selector(), &[0u8; 28], &ticket_hash.to_bytes()]);
        Hash::create(&[&hex!("1901"), &domain_separator.to_bytes(), &hash_struct.to_bytes()])
    }

    /// Signs the ticket using the given private key.
    pub fn sign(&mut self, signing_key: &ChainKeypair, domain_separator: &Hash) {
        self.signature = Some(Signature::sign_hash(
            &self.get_hash(domain_separator).to_bytes(),
            signing_key,
        ));
    }

    /// Convenience method for creating a zero-hop ticket
    pub fn new_zero_hop(destination: &Address, private_key: &ChainKeypair, domain_separator: &Hash) -> Result<Self> {
        Self::new(
            destination,
            &Balance::new(0u32.into(), BalanceType::HOPR),
            U256::zero(),
            U256::zero(),
            0.0,
            U256::zero(),
            EthereumChallenge::default(),
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

    pub fn get_expected_payout(&self) -> U256 {
        let mut win_prob = [0u8; 8];
        win_prob[1..].copy_from_slice(&self.encoded_win_prob);

        // Add + 1 to project interval [0x00ffffffffffff, 0x00000000000000] to [0x00000000000001, 0x01000000000000]
        // Add + 1 to "round to next integer"
        let win_prob = (u64::from_be_bytes(win_prob) >> 4) + 1 + 1;

        (*self.amount.value() * U256::from(win_prob)) >> U256::from(52u64)
    }

    /// Recovers the signer public key from the embedded ticket signature.
    /// This is possible due this specific instantiation of the ECDSA over the secp256k1 curve.
    pub fn recover_signer(&self, domain_separator: &Hash) -> hopr_crypto_types::errors::Result<PublicKey> {
        PublicKey::from_signature_hash(
            &self.get_hash(domain_separator).to_bytes(),
            self.signature.as_ref().expect("ticket not signed"),
        )
    }

    /// Verifies the signature of this ticket.
    /// The operation can fail if a public key cannot be recovered from the ticket signature.
    pub fn verify(&self, address: &Address, domain_separator: &Hash) -> hopr_crypto_types::errors::Result<()> {
        let recovered = self.recover_signer(domain_separator)?;
        recovered
            .to_address()
            .eq(address)
            .then_some(())
            .ok_or(CryptoError::SignatureVerification)
    }

    pub fn is_aggregated(&self) -> bool {
        // Aggregated tickets have always an index offset > 1
        self.index_offset > 1
    }
}

impl BinarySerializable for Ticket {
    const SIZE: usize = ENCODED_TICKET_LENGTH + EthereumChallenge::SIZE + Signature::SIZE;

    /// Tickets get sent next to packets, hence they need to be as small as possible.
    /// Transmitting tickets to the next downstream share the same binary representation
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
                signature: Some(signature),
            })
        } else {
            // TODO: make Error a generic
            Err(hopr_primitive_types::errors::GeneralError::ParseError)
        }
    }

    /// Serializes the ticket to be transmitted to the next downstream node or handled by the
    /// smart contract
    fn to_bytes(&self) -> Box<[u8]> {
        self.to_bytes_internal(true)
            .expect("ticket not signed")
            .into_boxed_slice()
    }
}

/// Decodes [0x00000000000000, 0xffffffffffffff] to [0.0f64, 1.0f64]
pub fn win_prob_to_f64(encoded_win_prob: &EncodedWinProb) -> f64 {
    if encoded_win_prob.eq(&hex!("00000000000000")) {
        return 0.0;
    }

    if encoded_win_prob.eq(&hex!("ffffffffffffff")) {
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
        return Ok(hex!("00000000000000"));
    }

    if win_prob == 1.0 {
        return Ok(hex!("ffffffffffffff"));
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
    use crate::channels::{f64_to_win_prob, ChannelEntry, ChannelStatus, Ticket};
    use hex_literal::hex;
    use hopr_crypto_types::{
        keypairs::{ChainKeypair, Keypair},
        types::Hash,
    };
    use hopr_primitive_types::{
        primitives::{Address, Balance, BalanceType, EthereumChallenge, U256},
        traits::BinarySerializable,
    };

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).unwrap();
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).unwrap();

        static ref ADDRESS_1: Address = Address::from_bytes(&hex!("3829b806aea42200c623c4d6b9311670577480ed")).unwrap();
        static ref ADDRESS_2: Address = Address::from_bytes(&hex!("1a34729c69e95d6e11c3a9b9be3ea0c62c6dc5b1")).unwrap();
    }

    #[test]
    pub fn channel_entry_test() {
        let ce1 = ChannelEntry::new(
            *ADDRESS_1,
            *ADDRESS_2,
            Balance::new(10u64.into(), BalanceType::HOPR),
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
        let cs2 = ChannelStatus::from_byte(cs1.to_byte()).unwrap();

        assert!(ChannelStatus::from_byte(231).is_none());
        assert_eq!(cs1, cs2, "channel status does not match");
    }

    #[test]
    pub fn channel_entry_closure_time() {
        let mut ce = ChannelEntry::new(
            *ADDRESS_1,
            *ADDRESS_2,
            Balance::new(10u64.into(), BalanceType::HOPR),
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
            &BOB.public().to_address(),
            &Balance::new(U256::one(), BalanceType::HOPR),
            U256::zero(),
            U256::one(),
            1.0,
            U256::one(),
            EthereumChallenge::default(),
            &ALICE,
            &Hash::default(),
        )
        .unwrap();

        assert_ne!(*initial_ticket.get_hash(&Hash::default()).to_bytes(), [0u8; Hash::SIZE]);

        assert_eq!(initial_ticket, Ticket::from_bytes(&initial_ticket.to_bytes()).unwrap());
    }

    #[test]
    pub fn test_ticket_serialize_deserialize_serde() {
        let initial_ticket = super::Ticket::new(
            &BOB.public().to_address(),
            &Balance::new(U256::one(), BalanceType::HOPR),
            U256::zero(),
            U256::one(),
            1.0,
            U256::one(),
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
            &Balance::new(U256::one(), BalanceType::HOPR),
            U256::zero(),
            U256::one(),
            1.0,
            U256::one(),
            EthereumChallenge::default(),
            &ALICE,
            &Hash::default(),
        )
        .unwrap();

        assert_ne!(*initial_ticket.get_hash(&Hash::default()).to_bytes(), [0u8; Hash::SIZE]);

        assert!(initial_ticket
            .verify(&ALICE.public().to_address(), &Hash::default())
            .is_ok());
    }

    #[test]
    pub fn test_ticket_expected_payout() {
        let mut ticket = Ticket::new_partial(
            &ALICE.public().to_address(),
            &BOB.public().to_address(),
            &Balance::new(U256::one(), BalanceType::HOPR),
            U256::zero(),
            U256::one(),
            1.0,
            U256::one(),
        )
        .unwrap();

        assert_eq!(U256::one(), ticket.get_expected_payout());

        ticket.encoded_win_prob = f64_to_win_prob(0.0).unwrap();
        assert_eq!(U256::zero(), ticket.get_expected_payout());

        ticket.amount = Balance::new(100000000000u64.into(), BalanceType::HOPR);
        ticket.encoded_win_prob = f64_to_win_prob(0.00000000001f64).unwrap();

        assert_eq!(U256::one(), ticket.get_expected_payout());
    }

    #[test]
    pub fn test_path_position() {
        let mut ticket = Ticket::new_partial(
            &ALICE.public().to_address(),
            &BOB.public().to_address(),
            &Balance::new(U256::one(), BalanceType::HOPR),
            U256::zero(),
            U256::one(),
            1.0,
            U256::one(),
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
            &Balance::new(256u64.into(), BalanceType::HOPR),
            U256::zero(),
            U256::one(),
            1.0,
            U256::one(),
        )
        .unwrap();

        assert!(ticket.get_path_position(U256::from(1u64)).is_err());
    }

    #[test]
    pub fn test_zero_hop() {
        let zero_hop_ticket = Ticket::new_zero_hop(&BOB.public().to_address(), &ALICE, &Hash::default()).unwrap();
        assert!(zero_hop_ticket
            .verify(&ALICE.public().to_address(), &Hash::default())
            .is_ok());
    }
}
