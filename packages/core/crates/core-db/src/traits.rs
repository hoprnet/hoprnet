use async_trait::async_trait;

use core_crypto::types::{HalfKeyChallenge, Hash, PublicKey};
use utils_types::primitives::{Address, Balance};

pub type Result<T> = std::result::Result<T, utils_db::errors::DbError>;

// TODO: placeholder for the non-existent types
type Ticket = u128;
type UnacknowledgedTicket = u128;
type ChannelEntry = u128;

// TODO: packet interaction only
#[async_trait]
pub trait HoprCoreDbActions {
    // TODO: update u128 to u256
    async fn get_current_ticket_index(&self, channel_id: &Hash) -> Result<Option<u128>>;
    async fn set_current_ticket_index(&self, channel_id: &Hash, commitment: &Hash) -> Result<()>;

    // TODO: trait with generic argument rather than allocated Box
    async fn get_tickets(&self, predicate: Box<dyn Fn() -> bool>) -> Result<Vec<Ticket>>;
    async fn get_channel_to(&self, dest: PublicKey) -> ChannelEntry;
    async fn get_channel_from(&self, origin: PublicKey) -> ChannelEntry;

    async fn get_pending_balance_to(&self, counterparty: &Address) -> Balance;
    async fn mark_pending(&self, ticket: &Ticket);
    async fn mark_rejected(&self, ticket: &Ticket);

    async fn check_and_set_packet_tag(&self, tag: Box<[u8]>);

    async fn store_pending_acknowledgment(
        &self,
        half_key_challenge: HalfKeyChallenge,
        is_message_sender: bool,
        unack_ticket: Option<UnacknowledgedTicket>,
    );
}
