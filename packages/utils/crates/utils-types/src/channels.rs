use std::ops::{Div, Mul, Sub};
use ethnum::u256;
use serde_repr::*;
use utils_misc::utils::get_time_millis;
use crate::crypto::{Challenge, ethereum_signed_hash, Hash, PublicKey, Signature};
use crate::errors::{Result, GeneralError::ParseError};
use crate::primitives::{Address, Balance, BalanceType, EthereumChallenge, U256};

/// Describes status of a channel
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub enum ChannelStatus {
    Closed = 0,
    WaitingForCommitment = 1,
    Open = 2,
    PendingToClose = 3
}

impl ChannelStatus {
    pub fn from_byte(byte: u8) -> Self {
        unsafe { std::mem::transmute(byte) }
    }

    pub fn to_byte(&self) -> u8 {
        *self as u8
    }

    pub fn to_string(&self) -> String {
        match self {
            ChannelStatus::Closed => "Closed",
            ChannelStatus::WaitingForCommitment => "WaitingForCommitment",
            ChannelStatus::Open => "Open",
            ChannelStatus::PendingToClose => "PendingToClose",
        }.to_string()
    }
}

/// Contains acknowledgment information and the respective ticket
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct AcknowledgedTicket {
    pub ticket: Ticket,
    pub response: Response,
    pub pre_image: Hash,
    pub signer: PublicKey
}

/// Overall description of a channel
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct ChannelEntry {
    pub source: PublicKey,
    pub destination: PublicKey,
    pub balance: Balance,
    pub commitment: Hash,
    pub ticket_epoch: U256,
    pub ticket_index: U256,
    pub status: ChannelStatus,
    pub channel_epoch: U256,
    pub closure_time: U256,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl ChannelEntry {
    pub fn serialize(&self) -> Box<[u8]> {
        let mut ret: Vec<u8> = vec![];
        ret.extend_from_slice(self.source.serialize(false).as_ref());
        ret.extend_from_slice(self.destination.serialize(false).as_ref());
        ret.extend_from_slice(self.balance.serialize_value().as_ref());
        ret.extend_from_slice(self.commitment.serialize().as_ref());
        ret.extend_from_slice(self.ticket_epoch.serialize().as_ref());
        ret.extend_from_slice(self.ticket_index.serialize().as_ref());
        ret.push(self.status as u8);
        ret.extend_from_slice(self.channel_epoch.serialize().as_ref());
        ret.extend_from_slice(self.closure_time.serialize().as_ref());
        ret.into_boxed_slice()
    }

    pub fn get_id(&self) -> Hash {
        generate_channel_id(&self.source.to_address(), &self.destination.to_address())
    }

    pub fn closure_time_passed(&self) -> bool {
        let now_seconds =  get_time_millis() / 1000;
        self.closure_time.value().lt(&u256::from(now_seconds))
    }

    pub fn remaining_closure_time(&self) -> u64 {
        let now_seconds = u256::from(get_time_millis());
        if now_seconds.ge(self.closure_time.value()) {
            now_seconds.sub(self.closure_time.value()).as_u64()
        } else {
            0
        }
    }
}

impl ChannelEntry {
    pub const SIZE: usize = PublicKey::SIZE_UNCOMPRESSED +
                            PublicKey::SIZE_UNCOMPRESSED +
                            Balance::SIZE +
                            Hash::SIZE +
                            U256::SIZE +
                            U256::SIZE +
                            1 +
                            U256::SIZE +
                            U256::SIZE;

    pub fn deserialize(data: &[u8]) -> Result<ChannelEntry> {
        if data.len() == Self::SIZE {
            let mut b = Vec::from(data);
            let source = PublicKey::deserialize(b.drain(0..PublicKey::SIZE_UNCOMPRESSED).as_ref())?;
            let destination = PublicKey::deserialize(b.drain(0..PublicKey::SIZE_UNCOMPRESSED).as_ref())?;
            let balance = Balance::deserialize(b.drain(0..Balance::SIZE).as_ref(), BalanceType::HOPR)?;
            let commitment = Hash::deserialize(b.drain(0..Hash::SIZE).as_ref())?;
            let ticket_epoch = U256::deserialize(b.drain(0..U256::SIZE).as_ref())?;
            let ticket_index = U256::deserialize(b.drain(0..U256::SIZE).as_ref())?;
            let status = ChannelStatus::from_byte(b.pop().unwrap());
            let channel_epoch = U256::deserialize(b.drain(0..U256::SIZE).as_ref())?;
            let closure_time = U256::deserialize(b.drain(0..U256::SIZE).as_ref())?;
            Ok(Self {
                source, destination, balance, commitment, ticket_epoch, ticket_index, status, channel_epoch, closure_time
            })
        } else {
            Err(ParseError)
        }
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub fn generate_channel_id(source: &Address, destination: &Address) -> Hash {
    Hash::create(&[&source.serialize(), &destination.serialize()])
}

/// Contains a response upon ticket acknowledgement
#[derive(Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Response {
    response: [u8; Self::SIZE],
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Response {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(data: &[u8]) -> Self {
        assert_eq!(data.len(), Self::SIZE);
        let mut ret = Response {
            response: [0u8; Self::SIZE]
        };
        ret.response.copy_from_slice(data);
        ret
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.response)
    }

    pub fn serialize(&self) -> Box<[u8]> {
        self.response.into()
    }
}

impl Response {
    /// Size of the serialized response
    pub const SIZE: usize = 32;

    pub fn deserialize(data: &[u8]) -> Result<Response> {
        if data.len() == Self::SIZE {
            Ok(Response::new(data))
        } else {
            Err(ParseError)
        }
    }
}

/// Contains the overall description of a ticket with a signature
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct Ticket {
    pub counterparty: Address,
    pub challenge: EthereumChallenge,
    pub epoch: U256,
    pub index: U256,
    pub amount: Balance,
    pub win_prob: U256,
    pub channel_epoch: U256,
    pub signature: Signature
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Ticket {
    fn serialize_unsigned_aux(counterparty: &Address, challenge: &EthereumChallenge, epoch: &U256, amount: &Balance, win_prob: &U256, index: &U256, channel_epoch: &U256) -> Vec<u8> {
        let mut ret = Vec::<u8>::new();
        ret.extend_from_slice(&counterparty.serialize());
        ret.extend_from_slice(&challenge.serialize());
        ret.extend_from_slice(&epoch.serialize());
        ret.extend_from_slice(&amount.serialize_value());
        ret.extend_from_slice(&win_prob.serialize());
        ret.extend_from_slice(&index.serialize());
        ret.extend_from_slice(&channel_epoch.serialize());
        ret
    }

    pub fn create(counterparty: Address, challenge: Challenge, epoch: U256, index: U256, amount: Balance, win_prob: U256, channel_epoch: U256, signing_key: &[u8]) -> Self {
        let encoded_challenge = challenge.to_ethereum_challenge();
        let hashed_ticket = Hash::create(&[&Self::serialize_unsigned_aux(&counterparty, &encoded_challenge, &epoch, &amount, &win_prob, &index, &channel_epoch)]);
        let msg = ethereum_signed_hash(hashed_ticket.serialize());
        let signature = Signature::sign_message(&msg.serialize(), signing_key);

        Self {
            counterparty, challenge: encoded_challenge, epoch, index, amount, win_prob, channel_epoch, signature
        }
    }

    /// Serializes the ticket except the signature
    pub fn serialize_unsigned(&self) -> Box<[u8]> {
        Self::serialize_unsigned_aux(&self.counterparty, &self.challenge, &self.epoch, &self.amount, &self.win_prob, &self.index, &self.channel_epoch)
            .into_boxed_slice()
    }

    pub fn serialize(&self) -> Box<[u8]> {
        let mut unsigned = Self::serialize_unsigned_aux(&self.counterparty, &self.challenge, &self.epoch, &self.amount, &self.win_prob, &self.index, &self.channel_epoch);
        unsigned.extend_from_slice(&self.signature.serialize());
        unsigned.into_boxed_slice()
    }

    /// Computes Ethereum signature hash of the ticket
    pub fn get_hash(&self) -> Hash {
        ethereum_signed_hash(Hash::create(&[&self.serialize_unsigned()]).serialize())
    }

    /// Recovers the signer public key from the embedded ticket signature.
    /// This is possible due this specific instantiation of the ECDSA over the secp256k1 curve.
    pub fn recover_signer(&self) -> PublicKey {
        PublicKey::from_signature(&self.get_hash().serialize(), &self.signature)
            .expect("invalid signature on ticket, public key not recoverable")
    }

    /// Verifies the signature of this ticket
    pub fn verify(&self, public_key: &PublicKey) -> bool {
        self.recover_signer().eq(public_key)
    }

    /// Computes a candidate check value to verify if this ticket is winning
    pub fn get_luck(&self, preimage: &Hash, channel_response: &Response) -> U256 {
        U256::deserialize(&Hash::create(&[
            &self.get_hash().serialize(),
            &preimage.serialize(),
            &channel_response.serialize()
        ]).serialize())
            .unwrap()
    }

    /// Decides whether a ticket is a win or not.
    /// Note that this mimics the on-chain logic.
    /// Purpose of the function is to check the validity of ticket before we submit it to the blockchain.
    pub fn is_winning(&self, preimage: &Hash, channel_response: &Response, win_prob: &U256) -> bool {
        let luck = self.get_luck(preimage, channel_response);
        luck.value().le(win_prob.value())
    }

    /// Based on the price of this ticket, determines the path position (hop number) this ticket
    /// relates to.
    pub fn get_path_position(&self, price_per_packet: &U256, inverse_ticket_win_prob: &U256) -> u8 {
        let base_unit = price_per_packet.value().mul(inverse_ticket_win_prob.value());
        self.amount.value().div(base_unit).as_u8()
    }
}

impl Ticket {
    pub const SIZE: usize = Address::SIZE + EthereumChallenge::SIZE + 2 * U256::SIZE +
        Balance::SIZE + 2 * U256::SIZE + Signature::SIZE;

    pub fn deserialize(bytes: &[u8]) -> Result<Ticket> {
        if bytes.len() == Self::SIZE {
            let mut b = Vec::from(bytes);
            let counterparty = Address::deserialize(b.drain(0..Address::SIZE).as_ref())?;
            let challenge = EthereumChallenge::deserialize(b.drain(0..EthereumChallenge::SIZE).as_ref())?;
            let epoch = U256::deserialize(b.drain(0..U256::SIZE).as_ref())?;
            let index = U256::deserialize(b.drain(0..U256::SIZE).as_ref())?;
            let amount = Balance::deserialize(b.drain(0..Balance::SIZE).as_ref(), BalanceType::HOPR)?;
            let win_prob = U256::deserialize(b.drain(0..U256::SIZE).as_ref())?;
            let channel_epoch = U256::deserialize(b.drain(0..U256::SIZE).as_ref())?;
            let signature = Signature::deserialize(b.drain(0..Signature::SIZE).as_ref())?;

            Ok(Self {
                counterparty, challenge, epoch, index, amount, win_prob, channel_epoch, signature
            })
        } else {
            Err(ParseError)
        }
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::wasm_bindgen;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use crate::channels::{AcknowledgedTicket, ChannelEntry, ChannelStatus, Response, Ticket};
    use crate::crypto::{Hash, PublicKey, Signature};
    use crate::primitives::{Address, Balance, EthereumChallenge, U256};

    #[wasm_bindgen]
    pub fn channel_status_to_number(status: ChannelStatus) -> u8 {
        status as u8
    }

    #[wasm_bindgen]
    pub fn number_to_channel_status(number: u8) -> ChannelStatus {
        ChannelStatus::from_byte(number)
    }

    #[wasm_bindgen]
    pub fn channel_status_to_string(status: ChannelStatus) -> String {
        status.to_string()
    }

    #[wasm_bindgen]
    impl AcknowledgedTicket {
        #[wasm_bindgen(constructor)]
        pub fn new(
            ticket: Ticket,
            response: Response,
            pre_image: Hash,
            signer: PublicKey
        ) -> Self {
            AcknowledgedTicket {
                ticket, response, pre_image, signer
            }
        }
    }

    #[wasm_bindgen]
    impl ChannelEntry {
        #[wasm_bindgen(constructor)]
        pub fn new(
            source: PublicKey,
            destination: PublicKey,
            balance: Balance,
            commitment: Hash,
            ticket_epoch: U256,
            ticket_index: U256,
            status: ChannelStatus,
            channel_epoch: U256,
            closure_time: U256,
        ) -> Self {
            ChannelEntry {
                source, destination, balance, commitment, ticket_epoch, ticket_index, status,
                channel_epoch, closure_time
            }
        }
    }

    #[wasm_bindgen]
    impl Response {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn deserialize_response(data: &[u8]) -> JsResult<Response> {
            ok_or_jserr!(Response::deserialize(data))
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    impl Ticket {
        #[wasm_bindgen(constructor)]
        pub fn new(
            counterparty: Address,
            challenge: EthereumChallenge,
            epoch: U256,
            index: U256,
            amount: Balance,
            win_prob: U256,
            channel_epoch: U256,
            signature: Signature
        ) -> Self {
            Ticket {
                counterparty, challenge, epoch, index, amount, win_prob, channel_epoch, signature
            }
        }

        #[wasm_bindgen(js_name = "deserialize")]
        pub fn deserialize_bytes(bytes: &[u8]) -> JsResult<Ticket> {
            ok_or_jserr!(Self::deserialize(bytes))
        }
    }
}