use ethnum::u256;
use serde_repr::*;
use crate::errors::GeneralError;
use crate::errors::GeneralError::ParseError;
use crate::primitives::{Address, Balance, EthereumChallenge, Signature};

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

// TODO: Update to use secp256k1 private key length constant
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
    epoch: u256,
    index: u256,
    pub amount: Balance,
    win_prob: u256,
    channel_epoch: u256,
    pub signature: Signature
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Ticket {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter))]
    pub fn epoch(&self) -> String {
        self.epoch.to_string()
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter))]
    pub fn index(&self) -> String {
        self.index.to_string()
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter))]
    pub fn win_probability(&self) -> String {
        self.win_prob.to_string()
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter))]
    pub fn channel_epoch(&self) -> String {
        self.channel_epoch.to_string()
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::wasm_bindgen;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use crate::channels::Response;

    #[wasm_bindgen]
    impl Response {
        pub fn deserialize_response(data: &[u8]) -> JsResult<Response> {
            ok_or_jserr!(Response::deserialize(data))
        }
    }
}