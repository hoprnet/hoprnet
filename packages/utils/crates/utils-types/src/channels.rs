use ethnum::u256;
use crate::primitives::{Balance, PublicKey};

pub enum ChannelStatus {
    Closed = 0,
    WaitingForCommitment = 1,
    Open = 2,
    PendingToClose = 3
}

pub struct ChannelEntry {
    source: PublicKey,
    destination: PublicKey,
    balance: Balance,
    commitment: Box<[u8]>,
    ticket_epoch: u256,
    ticket_index: u256,
    status: ChannelStatus,
    channel_epoch: u256,
    closure_time: u256
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct ChannelEntry {
        w: super::ChannelEntry
    }

    #[wasm_bindgen]
    pub enum ChannelStatus {
        Closed = 0,
        WaitingForCommitment = 1,
        Open = 2,
        PendingToClose = 3
    }

}