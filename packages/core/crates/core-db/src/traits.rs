use async_trait::async_trait;

use core_crypto::types::{HalfKeyChallenge, Hash, PublicKey};
use core_types::acknowledgement::{PendingAcknowledgement, UnacknowledgedTicket};
use core_types::channels::{ChannelEntry, Ticket};
use utils_types::primitives::{Address, Balance, U256};

use crate::errors::Result;

#[async_trait(? Send)] // not placing the `Send` trait limitations on the trait
pub trait HoprCoreDbActions {
    async fn get_current_ticket_index(&self, channel_id: &Hash) -> Result<Option<U256>>;
    async fn set_current_ticket_index(&mut self, channel_id: &Hash, index: U256) -> Result<()>;

    // TODO: trait with generic argument rather than allocated Box
    async fn get_tickets(&self, signer: PublicKey) -> Result<Vec<Ticket>>;
    async fn mark_pending(&mut self, ticket: &Ticket) -> Result<()>;
    async fn mark_rejected(&mut self, ticket: &Ticket) -> Result<()>;

    async fn get_channel_to(&self, dest: PublicKey) -> Result<ChannelEntry>;
    async fn get_channel_from(&self, origin: PublicKey) -> Result<ChannelEntry>;
    async fn get_pending_balance_to(&self, counterparty: &Address) -> Result<Balance>;

    async fn check_and_set_packet_tag(&mut self, tag: Box<[u8]>) -> Result<bool>;

    async fn get_pending_acknowledgement(&self, half_key_challenge: HalfKeyChallenge) -> Result<Option<PendingAcknowledgement>>;
    async fn store_pending_acknowledgment(
        &mut self,
        half_key_challenge: HalfKeyChallenge,
        pending_acknowledgment: PendingAcknowledgement,
    ) -> Result<()>;
}
