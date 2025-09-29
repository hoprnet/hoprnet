use crate::prelude::Hash;

pub struct OpenChannelResult {
    pub tx_hash: Hash,
    pub channel_id: Hash,
}

pub struct CloseChannelResult {
    pub tx_hash: Hash,
    pub status: hopr_internal_types::channels::ChannelStatus,
}

#[async_trait::async_trait]
pub trait HoprChainApi {}
