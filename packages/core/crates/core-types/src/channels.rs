use crate::errors::{CoreTypesError, Result};
use bindings::hopr_channels::RedeemTicketCall;
use core_crypto::{
    errors::CryptoError::SignatureVerification,
    keypairs::{ChainKeypair, Keypair},
    types::{Hash, PublicKey, Signature},
};
use enum_iterator::{all, Sequence};
use ethers::contract::EthCall;
use hex_literal::hex;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::fmt::{Display, Formatter};
use utils_types::primitives::{Address, Balance, BalanceType, EthereumChallenge, U256};

#[cfg(all(feature = "wasm", not(test)))]
use utils_misc::time::wasm::current_timestamp;

#[cfg(any(not(feature = "wasm"), test))]
use utils_misc::time::native::current_timestamp;

use utils_types::traits::{BinarySerializable, ToHex};

/// Size-optimized encoding of the ticket, used for both,
/// network transfer and in the smart contract.
const ENCODED_TICKET_LENGTH: usize = 64;

pub type EncodedWinProb = [u8; 7];

/// Describes status of a channel
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr, Sequence)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub enum ChannelStatus {
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

/// Overall description of a channel
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct ChannelEntry {
    pub source: Address,
    pub destination: Address,
    pub balance: Balance,
    pub ticket_index: U256,
    pub status: ChannelStatus,
    pub channel_epoch: U256,
    pub closure_time: U256,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl ChannelEntry {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
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
        }
    }

    /// Generates the channel ID using the source and destination address
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
    pub fn get_id(&self) -> Hash {
        generate_channel_id(&self.source, &self.destination)
    }

    /// Checks if the closure time of this channel has passed.
    pub fn closure_time_passed(&self) -> Option<bool> {
        // round clock ms to seconds
        let now_seconds: U256 = U256::from(current_timestamp()) / 1000u64.into();

        if self.closure_time.eq(&U256::zero()) {
            None
        } else {
            Some(self.closure_time < now_seconds)
        }
    }

    /// Calculates the remaining channel closure grace period.
    pub fn remaining_closure_time(&self) -> Option<u64> {
        // round clock ms to seconds
        let now_seconds = U256::from(current_timestamp()) / 1000u64.into();

        if self.closure_time.eq(&U256::zero()) {
            None
        } else if now_seconds >= self.closure_time {
            Some((now_seconds - self.closure_time).as_u64())
        } else {
            Some(0u64)
        }
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(js_name = "to_string"))]
    pub fn _to_string(&self) -> String {
        format!("{}", self)
    }
}

impl std::fmt::Display for ChannelEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ChannelEntry")
            .field("source", &self.source.to_string())
            .field("destination", &self.destination.to_string())
            .field("balance", &format!("{}", self.balance))
            .field("ticket_index", &self.ticket_index.to_string())
            .field("status", &self.status.to_string())
            .field("channel_epoch", &self.channel_epoch.to_string())
            .field("closure_time", &self.closure_time.to_string())
            .finish()
    }
}

impl BinarySerializable for ChannelEntry {
    const SIZE: usize = Address::SIZE + Address::SIZE + Balance::SIZE + U256::SIZE + 1 + U256::SIZE + U256::SIZE;

    fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            let mut b = data.to_vec();
            let source = Address::from_bytes(b.drain(0..Address::SIZE).as_ref())?;
            let destination = Address::from_bytes(b.drain(0..Address::SIZE).as_ref())?;
            let balance = Balance::deserialize(b.drain(0..Balance::SIZE).as_ref(), BalanceType::HOPR)?;
            let ticket_index = U256::from_bytes(b.drain(0..U256::SIZE).as_ref())?;
            let status = ChannelStatus::from_byte(b.drain(0..1).as_ref()[0])
                .ok_or(utils_types::errors::GeneralError::ParseError)?;
            let channel_epoch = U256::from_bytes(b.drain(0..U256::SIZE).as_ref())?;
            let closure_time = U256::from_bytes(b.drain(0..U256::SIZE).as_ref())?;
            Ok(Self {
                source,
                destination,
                balance,
                ticket_index,
                status,
                channel_epoch,
                closure_time,
            })
        } else {
            Err(utils_types::errors::GeneralError::ParseError)
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

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub fn generate_channel_id(source: &Address, destination: &Address) -> Hash {
    Hash::create(&[&source.to_bytes(), &destination.to_bytes()])
}

/// Contains the overall description of a ticket with a signature
#[derive(Clone, PartialEq, Serialize, Deserialize)]
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

        let channel_id = generate_channel_id(&own_address, &counterparty);

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

        let channel_id = generate_channel_id(&own_address, &counterparty);

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

        ret.verify(&own_address, domain_separator)
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

        let channel_id = generate_channel_id(&own_address, &counterparty);

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
        if own_address.eq(&counterparty) {
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
    pub fn new_zero_hop(destination: &Address, private_key: &ChainKeypair, domain_separator: &Hash) -> Self {
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
        .expect("Failed to create zero-hop ticket")
    }

    /// Based on the price of this ticket, determines the path position (hop number) this ticket
    /// relates to.
    ///
    /// Does not support path lengths greater than 255
    pub fn get_path_position(&self, price_per_packet: U256) -> Result<u8> {
        Ok((self.get_expected_payout() / price_per_packet)
            .as_u64()
            .try_into() // convert to u8
            .map_err(|_| {
                CoreTypesError::ArithmeticError(format!(
                    "Cannot convert {} to u8",
                    price_per_packet / self.get_expected_payout()
                ))
            })?)
    }

    pub fn get_expected_payout(&self) -> U256 {
        let mut win_prob = [0u8; 8];
        win_prob[1..].copy_from_slice(&self.encoded_win_prob);

        // Add + 1 to project interval [0x00ffffffffffff, 0x00000000000000] to [0x00000000000001, 0x01000000000000]
        // Add + 1 to "round to next integer"
        let win_prob = (u64::from_be_bytes(win_prob) >> 4) + 1 + 1;

        (*self.amount.value() * win_prob.into()) >> U256::from(52u64)
    }

    /// Recovers the signer public key from the embedded ticket signature.
    /// This is possible due this specific instantiation of the ECDSA over the secp256k1 curve.
    pub fn recover_signer(&self, domain_separator: &Hash) -> core_crypto::errors::Result<PublicKey> {
        PublicKey::from_signature_hash(
            &self.get_hash(domain_separator).to_bytes(),
            self.signature.as_ref().expect("ticket not signed"),
        )
    }

    /// Verifies the signature of this ticket.
    /// The operation can fail if a public key cannot be recovered from the ticket signature.
    pub fn verify(&self, address: &Address, domain_separator: &Hash) -> core_crypto::errors::Result<()> {
        let recovered = self.recover_signer(domain_separator)?;
        recovered
            .to_address()
            .eq(address)
            .then_some(())
            .ok_or(SignatureVerification)
    }
}

impl BinarySerializable for Ticket {
    const SIZE: usize = ENCODED_TICKET_LENGTH + EthereumChallenge::SIZE + Signature::SIZE;

    /// Tickets get sent next to packets, hence they need to be as small as possible.
    /// Transmitting tickets to the next downstream share the same binary representation
    /// as used in the smart contract.
    fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
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
            Err(utils_types::errors::GeneralError::ParseError)
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
    if win_prob > 1.0 || win_prob < 0.0 {
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
    use core_crypto::{
        keypairs::{ChainKeypair, Keypair},
        types::Hash,
    };
    use hex_literal::hex;
    use utils_types::{
        primitives::{Address, Balance, BalanceType, EthereumChallenge, U256},
        traits::BinarySerializable,
    };

    const ADDRESS_1: [u8; 20] = hex!("3829b806aea42200c623c4d6b9311670577480ed");
    const ADDRESS_2: [u8; 20] = hex!("1a34729c69e95d6e11c3a9b9be3ea0c62c6dc5b1");
    const ALICE: [u8; 32] = hex!("e17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8");
    const BOB: [u8; 32] = hex!("07af9653b11d609139597aecd26360fce1f9c23864d0c6ce1035bdb77ea4e27c");

    #[test]
    pub fn channel_entry_test() {
        let ce1 = ChannelEntry::new(
            Address::from_bytes(&ADDRESS_1).unwrap(),
            Address::from_bytes(&ADDRESS_2).unwrap(),
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
        let alice = ChainKeypair::from_secret(&ALICE).unwrap();
        let bob = ChainKeypair::from_secret(&BOB).unwrap();

        let initial_ticket = super::Ticket::new(
            &bob.public().to_address(),
            &Balance::new(U256::one(), BalanceType::HOPR),
            U256::zero(),
            U256::one(),
            1.0,
            U256::one(),
            EthereumChallenge::default(),
            &alice,
            &Hash::default(),
        )
        .unwrap();

        assert_ne!(*initial_ticket.get_hash(&Hash::default()).to_bytes(), [0u8; Hash::SIZE]);

        assert_eq!(initial_ticket, Ticket::from_bytes(&initial_ticket.to_bytes()).unwrap());
    }

    #[test]
    pub fn test_ticket_sign_verify() {
        let alice = ChainKeypair::from_secret(&ALICE).unwrap();
        let bob = ChainKeypair::from_secret(&BOB).unwrap();

        let initial_ticket = super::Ticket::new(
            &bob.public().to_address(),
            &Balance::new(U256::one(), BalanceType::HOPR),
            U256::zero(),
            U256::one(),
            1.0,
            U256::one(),
            EthereumChallenge::default(),
            &alice,
            &Hash::default(),
        )
        .unwrap();

        assert_ne!(*initial_ticket.get_hash(&Hash::default()).to_bytes(), [0u8; Hash::SIZE]);

        assert!(initial_ticket
            .verify(&alice.public().to_address(), &Hash::default())
            .is_ok());
    }

    #[test]
    pub fn test_ticket_expected_payout() {
        let alice = ChainKeypair::from_secret(&ALICE).unwrap();
        let bob = ChainKeypair::from_secret(&BOB).unwrap();

        let mut ticket = Ticket::new_partial(
            &alice.public().to_address(),
            &bob.public().to_address(),
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
        let alice = ChainKeypair::from_secret(&ALICE).unwrap();
        let bob = ChainKeypair::from_secret(&BOB).unwrap();
        let mut ticket = Ticket::new_partial(
            &alice.public().to_address(),
            &bob.public().to_address(),
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
        let alice = ChainKeypair::from_secret(&ALICE).unwrap();
        let bob = ChainKeypair::from_secret(&BOB).unwrap();
        let ticket = Ticket::new_partial(
            &alice.public().to_address(),
            &bob.public().to_address(),
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
        let alice = ChainKeypair::from_secret(&ALICE).unwrap();
        let bob = ChainKeypair::from_secret(&BOB).unwrap();

        let zero_hop_ticket = Ticket::new_zero_hop(&bob.public().to_address(), &alice, &Hash::default());
        assert!(zero_hop_ticket
            .verify(&alice.public().to_address(), &Hash::default())
            .is_ok());
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use core_crypto::{
        keypairs::ChainKeypair,
        types::{Hash, Signature},
    };
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::{
        primitives::{Address, Balance, EthereumChallenge, U256},
        traits::BinarySerializable,
    };
    use wasm_bindgen::prelude::wasm_bindgen;

    use crate::channels::{ChannelEntry, ChannelStatus};

    #[wasm_bindgen]
    pub fn channel_status_to_number(status: ChannelStatus) -> u8 {
        status as u8
    }

    #[wasm_bindgen]
    pub fn number_to_channel_status(number: u8) -> Option<ChannelStatus> {
        ChannelStatus::from_byte(number)
    }

    #[wasm_bindgen]
    pub fn channel_status_to_string(status: ChannelStatus) -> String {
        status.to_string()
    }

    #[wasm_bindgen]
    impl ChannelEntry {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<ChannelEntry> {
            ok_or_jserr!(ChannelEntry::from_bytes(data))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &ChannelEntry) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    pub struct Ticket {
        w: super::Ticket,
    }

    #[wasm_bindgen]
    impl Ticket {
        #[wasm_bindgen(constructor)]
        pub fn new(
            counterparty: &Address,
            balance: &Balance,
            index: U256,
            index_offset: U256,
            win_prob: f64,
            epoch: U256,
            challenge: EthereumChallenge,
            chain_key: &ChainKeypair,
            domain_separator: &Hash,
        ) -> JsResult<Ticket> {
            Ok(Self {
                w: ok_or_jserr!(super::Ticket::new(
                    counterparty,
                    balance,
                    index,
                    index_offset,
                    win_prob,
                    epoch,
                    challenge,
                    chain_key,
                    domain_separator
                ))?,
            })
        }

        #[wasm_bindgen]
        pub fn default() -> Ticket {
            Self {
                w: super::Ticket::default(),
            }
        }

        #[wasm_bindgen(getter)]
        pub fn channel_id(&self) -> Hash {
            self.w.channel_id.clone()
        }

        #[wasm_bindgen(getter)]
        pub fn amount(&self) -> Balance {
            self.w.amount.clone()
        }

        #[wasm_bindgen(getter)]
        pub fn index(&self) -> U256 {
            self.w.index.into()
        }

        #[wasm_bindgen(getter)]
        pub fn index_offset(&self) -> U256 {
            self.w.index_offset.into()
        }

        #[wasm_bindgen(getter)]
        pub fn win_prob(&self) -> f64 {
            self.w.win_prob()
        }

        #[wasm_bindgen(getter)]
        pub fn channel_epoch(&self) -> U256 {
            self.w.channel_epoch.into()
        }

        #[wasm_bindgen(getter)]
        pub fn challenge(&self) -> EthereumChallenge {
            self.w.challenge.clone()
        }

        #[wasm_bindgen(getter)]
        pub fn signature(&self) -> Option<Signature> {
            self.w.signature.clone()
        }

        #[wasm_bindgen]
        pub fn to_string(&self) -> String {
            format!("{:?}", self.w)
        }

        pub fn clone(&self) -> Ticket {
            Self { w: self.w.clone() }
        }
    }

    impl From<super::Ticket> for Ticket {
        fn from(value: super::Ticket) -> Self {
            Self { w: value }
        }
    }

    impl From<&super::Ticket> for Ticket {
        fn from(value: &super::Ticket) -> Self {
            Self { w: value.clone() }
        }
    }

    impl From<Ticket> for super::Ticket {
        fn from(value: Ticket) -> Self {
            value.w
        }
    }

    impl From<&Ticket> for super::Ticket {
        fn from(value: &Ticket) -> Self {
            value.w.clone()
        }
    }
}
