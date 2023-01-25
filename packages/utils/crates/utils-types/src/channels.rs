use serde::{Serialize, Deserialize};
use utils_proc_macros::wasm_bindgen_if;

/// Describes status of the channel
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[wasm_bindgen_if]
pub enum ChannelStatus {
    Closed = 0,
    WaitingForCommitment = 1,
    Open = 2,
    PendingToClose = 3
}