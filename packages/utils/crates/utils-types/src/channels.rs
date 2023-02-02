use ethnum::u256;
use serde_repr::*;
use crate::crypto::{PublicKey, Signature};
use crate::errors::GeneralError;
use crate::errors::GeneralError::ParseError;
use crate::primitives::{Address, Balance, EthereumChallenge, Hash, U256};

/// Describes status of the channel
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub enum ChannelStatus {
    Closed = 0,
    WaitingForCommitment = 1,
    Open = 2,
    PendingToClose = 3
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct AcknowledgedTicket {
    pub ticket: Ticket,
    pub response: Response,
    pub pre_image: Hash,
    pub signer: PublicKey
}

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

// TODO: Update to use secp256k1 private key length constant from core-crypto
pub const RESPONSE_LENGTH: usize = 32;

#[derive(Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Response {
    response: [u8; RESPONSE_LENGTH],
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Response {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(data: &[u8]) -> Self {
        assert_eq!(data.len(), RESPONSE_LENGTH);
        let mut ret = Response {
            response: [0u8; RESPONSE_LENGTH]
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
    pub fn deserialize(data: &[u8]) -> Result<Response, GeneralError> {
        if data.len() == RESPONSE_LENGTH {
            Ok(Response::new(data))
        } else {
            Err(ParseError)
        }
    }
}

#[derive(Clone)]
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

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::wasm_bindgen;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use crate::channels::{AcknowledgedTicket, ChannelEntry, ChannelStatus, Response, Ticket};
    use crate::crypto::{PublicKey, Signature};
    use crate::primitives::{Address, Balance, EthereumChallenge, Hash, U256};

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
        pub fn deserialize_response(data: &[u8]) -> JsResult<Response> {
            ok_or_jserr!(Response::deserialize(data))
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
    }


}