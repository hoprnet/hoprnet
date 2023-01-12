use ethnum::u256;
use crate::primitives::Balance;

pub enum ChannelStatus {
    Closed = 0,
    WaitingForCommitment = 1,
    Open = 2,
    PendingToClose = 3
}

pub struct ChannelEntry {
    source: Box<[u8]>,
    destination: Box<[u8]>,
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

    impl From<super::ChannelEntry> for ChannelEntry {
        fn from(w: crate::channels::ChannelEntry) -> Self {
            ChannelEntry { w }
        }
    }

    #[wasm_bindgen]
    pub enum ChannelStatus {
        Closed = 0,
        WaitingForCommitment = 1,
        Open = 2,
        PendingToClose = 3
    }

}