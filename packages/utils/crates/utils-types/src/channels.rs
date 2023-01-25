use utils_proc_macros::wasm_bindgen_if;
use serde::{Serialize, Deserialize};

/// Describes status of the channel
#[wasm_bindgen_if]
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChannelStatus {
    Closed = 0,
    WaitingForCommitment = 1,
    Open = 2,
    PendingToClose = 3
}