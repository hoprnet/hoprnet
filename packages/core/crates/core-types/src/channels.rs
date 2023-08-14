use core_crypto::errors::CryptoError::SignatureVerification;
use core_crypto::keypairs::ChainKeypair;
use core_crypto::types::{Hash, PublicKey, Response, Signature};
use enum_iterator::{all, Sequence};
use ethnum::u256;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::fmt::{Display, Formatter};
use std::ops::{Div, Mul, Sub};
use utils_types::errors::{GeneralError::ParseError, Result};
use utils_types::primitives::{Address, Balance, BalanceType, EthereumChallenge, U256};

#[cfg(all(feature = "wasm", not(test)))]
use utils_misc::time::wasm::current_timestamp;

#[cfg(any(not(feature = "wasm"), test))]
use utils_misc::time::native::current_timestamp;

use utils_types::traits::{BinarySerializable, ToHex};

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

    /// Generates the ticket ID using the source and destination address
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
    pub fn get_id(&self) -> Hash {
        generate_channel_id(&self.source, &self.destination)
    }

    /// Checks if the closure time of this channel has passed.
    pub fn closure_time_passed(&self) -> Option<bool> {
        let now_seconds = current_timestamp() / 1000;
        (!self.closure_time.eq(&U256::zero())).then(|| self.closure_time.value().lt(&u256::from(now_seconds)))
    }

    /// Calculates the remaining channel closure grace period.
    pub fn remaining_closure_time(&self) -> Option<u64> {
        let now_seconds = u256::from(current_timestamp() / 1000);
        (!self.closure_time.eq(&U256::zero())).then(|| {
            if now_seconds.ge(self.closure_time.value()) {
                now_seconds.sub(self.closure_time.value()).as_u64()
            } else {
                0
            }
        })
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
    const SIZE: usize =
        Address::SIZE + Address::SIZE + Balance::SIZE + U256::SIZE + U256::SIZE + 1 + U256::SIZE + U256::SIZE;

    fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() == Self::SIZE {
            let mut b = data.to_vec();
            let source = Address::from_bytes(b.drain(0..Address::SIZE).as_ref())?;
            let destination = Address::from_bytes(b.drain(0..Address::SIZE).as_ref())?;
            let balance = Balance::deserialize(b.drain(0..Balance::SIZE).as_ref(), BalanceType::HOPR)?;
            let ticket_index = U256::from_bytes(b.drain(0..U256::SIZE).as_ref())?;
            let status = ChannelStatus::from_byte(b.drain(0..1).as_ref()[0]).ok_or(ParseError)?;
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
            Err(ParseError)
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
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct Ticket {
    pub counterparty: Address,
    pub challenge: EthereumChallenge,
    pub index: U256,
    pub amount: Balance,
    pub win_prob: U256,
    pub channel_epoch: U256,
    pub signature: Option<Signature>,
}

impl Default for Ticket {
    fn default() -> Self {
        Self {
            counterparty: Address::default(),
            challenge: EthereumChallenge::default(),
            index: U256::zero(),
            amount: Balance::new(U256::zero(), BalanceType::HOPR),
            win_prob: U256::max(),
            channel_epoch: U256::zero(),
            signature: None,
        }
    }
}

impl std::fmt::Display for Ticket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ticket")
            .field("counterparty", &self.counterparty)
            .field("challenge", &self.challenge)
            .field("index", &self.index)
            .field("amount", &self.amount)
            .field("channel_epoch", &self.channel_epoch)
            .field("signature", &self.signature.as_ref().map(|s| s.to_hex()))
            .finish()
    }
}

/// Prefix message with "\x19Ethereum Signed Message:\n {length} {message}" and returns its hash
/// Keccak256 is used as the underlying digest.
pub fn ethereum_signed_hash<T: AsRef<[u8]>>(message: T) -> Hash {
    const PREFIX: &str = "\x19Ethereum Signed Message:\n";

    let message = message.as_ref();
    let len = message.len();
    let len_string = len.to_string();

    let mut eth_message = Vec::with_capacity(PREFIX.len() + len_string.len() + len);
    eth_message.extend_from_slice(PREFIX.as_bytes());
    eth_message.extend_from_slice(len_string.as_bytes());
    eth_message.extend_from_slice(message);

    Hash::create(&[&eth_message])
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Ticket {
    fn serialize_unsigned_aux(
        counterparty: &Address,
        challenge: &EthereumChallenge,
        amount: &Balance,
        win_prob: &U256,
        index: &U256,
        channel_epoch: &U256,
    ) -> Vec<u8> {
        assert_eq!(BalanceType::HOPR, amount.balance_type(), "invalid balance currency");
        let mut ret = Vec::<u8>::with_capacity(Self::SIZE);
        ret.extend_from_slice(&counterparty.to_bytes());
        ret.extend_from_slice(&challenge.to_bytes());
        ret.extend_from_slice(&amount.serialize_value());
        ret.extend_from_slice(&win_prob.to_bytes());
        ret.extend_from_slice(&index.to_bytes());
        ret.extend_from_slice(&channel_epoch.to_bytes());
        ret
    }

    /// Creates a new Ticket given the raw Challenge and signs it using the given key.
    pub fn new(
        counterparty: Address,
        index: U256,
        amount: Balance,
        win_prob: U256,
        channel_epoch: U256,
        signing_key: &ChainKeypair,
    ) -> Self {
        let mut ret = Self {
            counterparty,
            challenge: EthereumChallenge::default(),
            index,
            amount,
            win_prob,
            channel_epoch,
            signature: None,
        };
        ret.sign(signing_key);
        ret
    }

    pub fn set_challenge(&mut self, challenge: EthereumChallenge, signing_key: &ChainKeypair) {
        self.challenge = challenge;
        self.sign(signing_key);
    }

    /// Signs the ticket using the given private key.
    pub fn sign(&mut self, signing_key: &ChainKeypair) {
        self.signature = Some(Signature::sign_message(&self.get_hash().to_bytes(), signing_key));
    }

    /// Convenience method for creating a zero-hop ticket
    pub fn new_zero_hop(destination: Address, private_key: &ChainKeypair) -> Self {
        Self::new(
            destination,
            U256::zero(),
            Balance::new(0u32.into(), BalanceType::HOPR),
            U256::zero(),
            U256::zero(),
            private_key,
        )
    }

    /// Serializes the ticket except the signature
    pub fn serialize_unsigned(&self) -> Box<[u8]> {
        Self::serialize_unsigned_aux(
            &self.counterparty,
            &self.challenge,
            &self.amount,
            &self.win_prob,
            &self.index,
            &self.channel_epoch,
        )
        .into_boxed_slice()
    }

    /// Computes Ethereum signature hash of the ticket
    pub fn get_hash(&self) -> Hash {
        ethereum_signed_hash(Hash::create(&[&self.serialize_unsigned()]).to_bytes())
    }

    /// Computes a candidate check value to verify if this ticket is winning
    pub fn get_luck(&self, preimage: &Hash, channel_response: &Response) -> U256 {
        U256::from_bytes(
            &Hash::create(&[
                &self.get_hash().to_bytes(),
                &preimage.to_bytes(),
                &channel_response.to_bytes(),
            ])
            .to_bytes(),
        )
        .unwrap()
    }

    /// Decides whether a ticket is a win or not.
    /// Note that this mimics the on-chain logic.
    /// Purpose of the function is to check the validity of ticket before we submit it to the blockchain.
    pub fn is_winning(&self, preimage: &Hash, channel_response: &Response, win_prob: U256) -> bool {
        let luck = self.get_luck(preimage, channel_response);
        luck.value().le(win_prob.value())
    }

    /// Based on the price of this ticket, determines the path position (hop number) this ticket
    /// relates to.
    pub fn get_path_position(&self, price_per_packet: U256, inverse_ticket_win_prob: U256) -> u8 {
        let base_unit = price_per_packet.value().mul(inverse_ticket_win_prob.value());
        self.amount.value().value().div(base_unit).as_u8()
    }
}

impl BinarySerializable for Ticket {
    const SIZE: usize =
        Address::SIZE + EthereumChallenge::SIZE + U256::SIZE + Balance::SIZE + 2 * U256::SIZE + Signature::SIZE;

    fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() == Self::SIZE {
            let mut b = data.to_vec();
            let counterparty = Address::from_bytes(b.drain(0..Address::SIZE).as_ref())?;
            let challenge = EthereumChallenge::from_bytes(b.drain(0..EthereumChallenge::SIZE).as_ref())?;
            let amount = Balance::deserialize(b.drain(0..Balance::SIZE).as_ref(), BalanceType::HOPR)?;
            let win_prob = U256::from_bytes(b.drain(0..U256::SIZE).as_ref())?;
            let index = U256::from_bytes(b.drain(0..U256::SIZE).as_ref())?;
            let channel_epoch = U256::from_bytes(b.drain(0..U256::SIZE).as_ref())?;
            let signature = Signature::from_bytes(b.drain(0..Signature::SIZE).as_ref())?;

            Ok(Self {
                counterparty,
                challenge,
                index,
                amount,
                win_prob,
                channel_epoch,
                signature: Some(signature),
            })
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut unsigned = self.serialize_unsigned().into_vec();
        unsigned.extend_from_slice(&self.signature.as_ref().expect("ticket not signed").to_bytes());
        unsigned.into_boxed_slice()
    }
}

impl Ticket {
    /// Recovers the signer public key from the embedded ticket signature.
    /// This is possible due this specific instantiation of the ECDSA over the secp256k1 curve.
    pub fn recover_signer(&self) -> core_crypto::errors::Result<PublicKey> {
        PublicKey::from_signature(
            &self.get_hash().to_bytes(),
            self.signature.as_ref().expect("ticket not signed"),
        )
    }

    /// Verifies the signature of this ticket.
    /// The operation can fail if a public key cannot be recovered from the ticket signature.
    pub fn verify(&self, address: &Address) -> core_crypto::errors::Result<()> {
        let recovered = self.recover_signer()?;
        recovered
            .to_address()
            .eq(address)
            .then_some(())
            .ok_or(SignatureVerification)
    }
}

#[cfg(test)]
pub mod tests {
    use core_crypto::keypairs::{ChainKeypair, Keypair};
    use core_crypto::types::{Hash, PublicKey};
    use ethnum::u256;
    use hex_literal::hex;
    use utils_types::primitives::{Address, Balance, BalanceType, U256};
    use utils_types::traits::BinarySerializable;

    use crate::channels::{ethereum_signed_hash, ChannelEntry, ChannelStatus, Ticket};

    const ADDRESS_1: [u8; 20] = hex!("3829b806aea42200c623c4d6b9311670577480ed");
    const ADDRESS_2: [u8; 20] = hex!("1a34729c69e95d6e11c3a9b9be3ea0c62c6dc5b1");

    const SGN_PRIVATE_KEY: [u8; 32] = hex!("e17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8");

    #[test]
    pub fn ethereum_signed_hash_test() {
        let hash = Hash::create(&[&hex!("deadbeef")]);
        let expected = hex!("3c4fb46e8b00d86ff9ff3a2fcd21c99564d0c8797c2eb05e5eb22b8102e283a5");

        let res = ethereum_signed_hash(&hash.to_bytes());
        assert_eq!(&expected, res.to_bytes().as_ref());
    }

    #[test]
    pub fn channel_entry_test() {
        let ce1 = ChannelEntry::new(
            Address::from_bytes(&ADDRESS_1).unwrap(),
            Address::from_bytes(&ADDRESS_2).unwrap(),
            Balance::new(u256::from(10u8).into(), BalanceType::HOPR),
            U256::new("0"),
            ChannelStatus::PendingToClose,
            U256::new("3"),
            U256::new("4"),
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
    pub fn ticket_test() {
        let inverse_win_prob = u256::new(1u128); // 100 %
        let price_per_packet = u256::new(10000000000000000u128); // 0.01 HOPR
        let path_pos = 5u8;

        let kp = ChainKeypair::from_secret(&SGN_PRIVATE_KEY).unwrap();

        let ticket1 = Ticket::new(
            Address::new(&[0u8; Address::SIZE]),
            U256::new("1"),
            Balance::new(
                (inverse_win_prob * price_per_packet * path_pos as u128).into(),
                BalanceType::HOPR,
            ),
            U256::from_inverse_probability(inverse_win_prob.into()).unwrap(),
            U256::new("4"),
            &kp,
        );

        let ticket2 = Ticket::from_bytes(&ticket1.to_bytes()).unwrap();

        assert_eq!(ticket1, ticket2, "deserialized ticket does not match");

        let pub_key = PublicKey::from_privkey(&SGN_PRIVATE_KEY).unwrap();
        assert!(
            ticket1.verify(&pub_key.to_address()).is_ok(),
            "failed to verify signed ticket 1"
        );
        assert!(
            ticket2.verify(&pub_key.to_address()).is_ok(),
            "failed to verify signed ticket 2"
        );

        assert_eq!(
            ticket1.get_path_position(price_per_packet.into(), inverse_win_prob.into()),
            path_pos,
            "invalid path pos"
        );
        assert_eq!(
            ticket2.get_path_position(price_per_packet.into(), inverse_win_prob.into()),
            path_pos,
            "invalid path pos"
        );
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use core_crypto::types::{Hash, PublicKey, Signature};
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::{Address, Balance, EthereumChallenge, U256};
    use utils_types::traits::{BinarySerializable, ToHex};
    use wasm_bindgen::prelude::wasm_bindgen;

    use crate::channels::{ChannelEntry, ChannelStatus, Ticket};

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
    pub fn ethereum_signed_hash(message: &Hash) -> Hash {
        super::ethereum_signed_hash(message.to_bytes())
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
    impl Ticket {
        #[wasm_bindgen(constructor)]
        pub fn _new(
            counterparty: Address,
            challenge: EthereumChallenge,
            index: U256,
            amount: Balance,
            win_prob: U256,
            channel_epoch: U256,
            signature: Signature,
        ) -> Self {
            Ticket {
                counterparty,
                challenge,
                index,
                amount,
                win_prob,
                channel_epoch,
                signature: Some(signature),
            }
        }

        #[wasm_bindgen(js_name = "recover_signer")]
        pub fn _recover_signer(&self) -> JsResult<PublicKey> {
            ok_or_jserr!(self.recover_signer())
        }

        #[wasm_bindgen(js_name = "verify")]
        pub fn _verify(&self, address: &Address) -> JsResult<bool> {
            ok_or_jserr!(self.verify(address).map(|_| true))
        }

        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(bytes: &[u8]) -> JsResult<Ticket> {
            ok_or_jserr!(Self::from_bytes(bytes))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "to_hex")]
        pub fn _to_hex(&self) -> String {
            self.to_hex()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &Ticket) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        #[wasm_bindgen(js_name = "to_string")]
        pub fn _to_string(&self) -> String {
            self.to_string()
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }
}
