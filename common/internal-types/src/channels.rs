use crate::errors::{CoreTypesError, Result};
use bindings::hopr_channels::RedeemTicketCall;
use ethers::contract::EthCall;
use hex_literal::hex;
use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
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

/// Winning probability encoded in 7-byte representation
pub type EncodedWinProb = [u8; ENCODED_WIN_PROB_LENGTH];

/// Encodes 100% winning probability
const ALWAYS_WINNING: EncodedWinProb = hex!("ffffffffffffff");

/// Encodes 0% winning probability
const NEVER_WINNING: EncodedWinProb = hex!("00000000000000");

/// Describes status of a channel
#[derive(Copy, Clone, Debug, smart_default::SmartDefault, Serialize, Deserialize, strum::Display)]
#[strum(serialize_all = "PascalCase")]
pub enum ChannelStatus {
    /// Channel is closed.
    #[default]
    Closed,
    /// Channel is opened.
    Open,
    /// Channel is pending to be closed.
    /// The timestamp marks the *earliest* possible time when the channel can transition into the `Closed` state.
    #[strum(serialize = "PendingToClose")]
    PendingToClose(SystemTime),
}

// Cannot use #[repr(u8)] due to PendingToClose
impl From<ChannelStatus> for u8 {
    fn from(value: ChannelStatus) -> Self {
        match value {
            ChannelStatus::Closed => 0,
            ChannelStatus::Open => 1,
            ChannelStatus::PendingToClose(_) => 2,
        }
    }
}

// Manual implementation of PartialEq, because we need only precision up to seconds in PendingToClose
impl PartialEq for ChannelStatus {
    fn eq(&self, other: &Self) -> bool {
        // Use pattern matching to avoid recursion
        match (self, other) {
            (Self::Open, Self::Open) => true,
            (Self::Closed, Self::Closed) => true,
            (Self::PendingToClose(ct_1), Self::PendingToClose(ct_2)) => {
                let diff = ct_1.max(ct_2).duration_since(*ct_1.min(ct_2)).unwrap();
                diff.as_secs() == 0
            }
            _ => false,
        }
    }
}
impl Eq for ChannelStatus {}

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
    ) -> Self {
        assert_eq!(BalanceType::HOPR, balance.balance_type(), "invalid balance currency");
        ChannelEntry {
            source,
            destination,
            balance,
            ticket_index,
            status,
            channel_epoch,
            id: generate_channel_id(&source, &destination),
        }
    }

    /// Generates the channel ID using the source and destination address
    pub fn get_id(&self) -> Hash {
        self.id
    }

    /// Checks if the closure time of this channel has passed.
    /// Also returns `false` if the channel closure has not been initiated (it is in `Open` state).
    /// Returns also `true`, if the channel is in `Closed` state.
    pub fn closure_time_passed(&self, current_time: SystemTime) -> bool {
        match self.status {
            ChannelStatus::Open => false,
            ChannelStatus::PendingToClose(closure_time) => closure_time <= current_time,
            ChannelStatus::Closed => true,
        }
    }

    /// Calculates the remaining channel closure grace period.
    /// Returns `None` if the channel closure has not been initiated yet (channel is in `Open` state).
    pub fn remaining_closure_time(&self, current_time: SystemTime) -> Option<Duration> {
        match self.status {
            ChannelStatus::Open => None,
            ChannelStatus::PendingToClose(closure_time) => {
                Some(closure_time.duration_since(current_time).unwrap_or(Duration::ZERO))
            }
            ChannelStatus::Closed => Some(Duration::ZERO),
        }
    }

    /// Returns the earliest time the channel can transition from `PendingToClose` into `Closed`.
    /// If the channel is not in `PendingToClose` state, returns `None`.
    pub fn closure_time_at(&self) -> Option<SystemTime> {
        match self.status {
            ChannelStatus::PendingToClose(ct) => Some(ct),
            _ => None,
        }
    }

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

/// Generates channel ID hash from `source` and `destination` addresses.
pub fn generate_channel_id(source: &Address, destination: &Address) -> Hash {
    Hash::create(&[source.as_ref(), destination.as_ref()])
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ticket {
    pub channel_id: Hash,
    pub amount: Balance,                  // 92 bits
    pub index: u64,                       // 48 bits
    pub index_offset: u32,                // 32 bits
    pub encoded_win_prob: EncodedWinProb, // 56 bits
    pub channel_epoch: u32,               // 24 bits
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

impl Ticket {
    /// Creates a new Ticket given the raw Challenge and signs it using the given chain keypair.
    #[allow(clippy::too_many_arguments)] // TODO: Refactor to use less inputs
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
    ) -> Result<Self> {
        let mut ret = Self::new_partial(
            &signing_key.public().to_address(),
            counterparty,
            amount,
            index,
            index_offset,
            win_prob,
            channel_epoch
        )?;
        ret.set_challenge(challenge, signing_key, domain_separator);

        Ok(ret)
    }

    /// Creates a ticket *without* signature and *without* a challenge set.
    /// This sets a default value as challenge.
    pub fn new_partial(
        own_address: &Address,
        counterparty: &Address,
        amount: Balance,
        index: u64,
        index_offset: u32,
        win_prob: f64,
        channel_epoch: u32,
    ) -> Result<Self> {
        if own_address.eq(counterparty) {
            return Err(CoreTypesError::InvalidInputData(
                "Source and destination must be different".into(),
            ));
        }

        Ticket::check_value_boundaries(
            &amount,
            index,
            win_prob,
            channel_epoch,
        )?;

        let channel_id = generate_channel_id(own_address, counterparty);

        Ok(Self {
            channel_id,
            amount: amount.to_owned(),
            index,
            index_offset,
            channel_epoch,
            challenge: EthereumChallenge::default(),
            encoded_win_prob: f64_to_win_prob(win_prob).expect("error encoding winning probability"),
            signature: None,
        })
    }

    /// Tickets 2.0 come with meaningful boundaries to fit into 2 EVM slots.
    /// This method checks whether they are met and prevents from unintended
    /// usage.
    fn check_value_boundaries(
        amount: &Balance,
        index: u64,
        win_prob: f64,
        channel_epoch: u32,
    ) -> Result<()> {
        if amount.balance_type().ne(&BalanceType::HOPR) {
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

        Ok(())
    }

    /// Add the challenge property and signs the finished ticket afterward
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

    /// Signs the ticket using the given private key.
    pub fn sign(&mut self, signing_key: &ChainKeypair, domain_separator: &Hash) {
        self.signature = Some(Signature::sign_hash(
                self.get_hash(domain_separator).as_ref(),
                signing_key,
            )
        )
    }

    /// Convenience method for creating a zero-hop ticket
    pub fn new_zero_hop(destination: &Address, private_key: &ChainKeypair, domain_separator: &Hash) -> Result<Self> {
        Self::new(
            destination,
            BalanceType::HOPR.zero(),
            0,
            0,
            0.0,
            0,
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

    fn get_expected_payout(&self) -> U256 {
        let mut win_prob = [0u8; 8];
        win_prob[1..].copy_from_slice(&self.encoded_win_prob);

        // Add + 1 to project interval [0x00ffffffffffff, 0x00000000000000] to [0x00000000000001, 0x01000000000000]
        // Add + 1 to "round to next integer"
        let win_prob = (u64::from_be_bytes(win_prob) >> 4) + 1 + 1;

        (self.amount.amount() * U256::from(win_prob)) >> U256::from(52u64)
    }

    /// Computes the VRF values to check or prove that the ticket
    /// is a win. Called by the ticket recipient.
    pub fn get_vrf_values(
        &self,
        chain_key: &ChainKeypair,
        domain_separator: &Hash,
    ) -> Result<VrfParameters> {
        let ticket_hash = self.get_hash(domain_separator);
        derive_vrf_parameters(&ticket_hash.into(), chain_key, domain_separator.as_ref())
            .map_err(|e| e.into())
    }

    /// Verifies the signature of this ticket.
    ///
    /// This is done by recovering the signer from the signature and verifying that it matches
    /// the given `signer` argument. This is possible due this specific instantiation of the ECDSA
    /// over the secp256k1 curve.
    /// The operation can fail if a public key cannot be recovered from the ticket signature.
    pub fn verify(&self, signer: &Address, domain_separator: &Hash) -> hopr_crypto_types::errors::Result<()> {
        PublicKey::from_signature_hash(
            self.get_hash(domain_separator).as_ref(),
            self.signature.as_ref().expect("ticket not signed"),
        )?
        .to_address()
        .eq(signer)
        .then_some(())
        .ok_or(CryptoError::SignatureVerification)
    }

    /// Returns true if this ticket aggregates multiple tickets.
    pub fn is_aggregated(&self) -> bool {
        // Aggregated tickets have always an index offset > 1
        self.index_offset > 1
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

            let amount = BalanceType::HOPR.balance_bytes(amount);
            let index = u64::from_be_bytes(index);
            let channel_epoch = u32::from_be_bytes(channel_epoch);

            // Validate the boundaries of the parsed values
            Ticket::check_value_boundaries(&amount, index, win_prob_to_f64(&encoded_win_prob) , channel_epoch)
                .map_err(|_| GeneralError::InvalidInput)?;

            Ok(Self {
                channel_id,
                amount,
                index,
                index_offset: u32::from_be_bytes(index_offset),
                encoded_win_prob,
                channel_epoch,
                challenge,
                signature: Some(signature),
            })
        } else {
            Err(hopr_primitive_types::errors::GeneralError::ParseError)
        }
    }
}

const TICKET_SIZE: usize = ENCODED_TICKET_LENGTH + EthereumChallenge::SIZE + Signature::SIZE;
impl BytesEncodable<TICKET_SIZE> for Ticket {}

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
    use std::ops::Add;
    use std::str::FromStr;
    use std::time::{Duration, SystemTime};

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).unwrap();
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).unwrap();

        static ref ADDRESS_1: Address = "3829b806aea42200c623c4d6b9311670577480ed".parse().unwrap();
        static ref ADDRESS_2: Address = "1a34729c69e95d6e11c3a9b9be3ea0c62c6dc5b1".parse().unwrap();
    }

    #[test]
    pub fn test_generate_id() {
        let from = Address::from_str("0xa460f2e47c641b64535f5f4beeb9ac6f36f9d27c").unwrap();
        let to = Address::from_str("0xb8b75fef7efdf4530cf1688c933d94e4e519ccd1").unwrap();
        let id = generate_channel_id(&from, &to).to_string();
        assert_eq!("0x1a410210ce7265f3070bf0e8885705dce452efcfbd90a5467525d136fcefc64a", id);
    }

    #[test]
    fn channel_status_names() {
        assert_eq!("Open", ChannelStatus::Open.to_string());
        assert_eq!("Closed", ChannelStatus::Closed.to_string());
        assert_eq!(
            "PendingToClose",
            ChannelStatus::PendingToClose(SystemTime::now()).to_string()
        );
    }

    #[test]
    pub fn channel_entry_closure_time() {
        let mut ce = ChannelEntry::new(
            *ADDRESS_1,
            *ADDRESS_2,
            Balance::new(10_u64, BalanceType::HOPR),
            23u64.into(),
            ChannelStatus::Open,
            3u64.into(),
        );

        assert!(
            !ce.closure_time_passed(SystemTime::now()),
            "opened channel cannot pass closure time"
        );
        assert!(
            ce.remaining_closure_time(SystemTime::now()).is_none(),
            "opened channel cannot have remaining closure time"
        );

        let current_time = SystemTime::now();
        ce.status = ChannelStatus::PendingToClose(current_time.add(Duration::from_secs(60)));

        assert!(
            !ce.closure_time_passed(current_time),
            "must not have passed closure time"
        );
        assert_eq!(
            60,
            ce.remaining_closure_time(current_time)
                .expect("must have closure time")
                .as_secs()
        );

        let current_time = current_time.add(Duration::from_secs(120));

        assert!(ce.closure_time_passed(current_time), "must have passed closure time");
        assert_eq!(
            Duration::ZERO,
            ce.remaining_closure_time(current_time).expect("must have closure time")
        );
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
        let zero_hop_ticket = Ticket::new_zero_hop(&BOB.public().to_address(), &ALICE, &Hash::default()).unwrap();
        assert!(zero_hop_ticket
            .verify(&ALICE.public().to_address(), &Hash::default())
            .is_ok());
    }
}
