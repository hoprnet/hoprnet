use async_trait::async_trait;

use core_crypto::types::{HalfKeyChallenge, Hash, PublicKey};
use core_types::acknowledgement::UnacknowledgedTicket;
use core_types::channels::{ChannelEntry, Ticket};
use utils_types::primitives::{Address, Balance, U256};

use crate::errors::Result;

#[async_trait(? Send)] // not placing the `Send` trait limitations on the trait
pub trait HoprCoreDbActions {
    async fn get_current_ticket_index(&self, channel_id: &Hash) -> Result<Option<U256>>;
    async fn set_current_ticket_index(&mut self, channel_id: &Hash, index: U256) -> Result<()>;

    // TODO: trait with generic argument rather than allocated Box
    async fn get_tickets(&self, predicate: Box<dyn Fn() -> bool>) -> Result<Vec<Ticket>>;
    async fn mark_pending(&self, ticket: &Ticket);
    async fn mark_rejected(&self, ticket: &Ticket);

    async fn get_channel_to(&self, dest: PublicKey) -> ChannelEntry;
    async fn get_channel_from(&self, origin: PublicKey) -> ChannelEntry;
    async fn get_pending_balance_to(&self, counterparty: &Address) -> Balance;

    async fn check_and_set_packet_tag(&self, tag: Box<[u8]>) -> bool;

    async fn store_pending_acknowledgment(
        &self,
        half_key_challenge: HalfKeyChallenge,
        is_message_sender: bool,
        unack_ticket: Option<UnacknowledgedTicket>,
    );
}
