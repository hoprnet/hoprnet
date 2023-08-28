use std::time::Duration;
use async_trait::async_trait;
use core_types::acknowledgement::{AcknowledgedTicket, Acknowledgement};
use core_types::channels::ChannelEntry;
use utils_types::primitives::{Address, Balance};
use crate::generic::PeerQuality;

pub enum Decision {
    OpenChannel(Address, Balance),
    CloseChannel(Address)
}

pub struct NodeStateSnapshot {
    pub peers_with_quality: Vec<(Address, f64)>,
    pub opened_channels: Vec<ChannelEntry>,
    pub native_balance: Balance,
    pub token_balance: Balance
}

#[async_trait]
pub trait Strategy {
    async fn on_channel_closed(&self, channel: &ChannelEntry) -> Vec<Decision>;

    async fn on_tick(&self, snapshot: NodeStateSnapshot, elapsed: Duration) -> Vec<Decision>;

    async fn on_acknowledged_ticket(&self, ticket: &AcknowledgedTicket) -> Vec<Decision>;
}