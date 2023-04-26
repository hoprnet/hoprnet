use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use core_crypto::types::{HalfKeyChallenge, Hash, PublicKey};
use core_types::acknowledgment::UnacknowledgedTicket;
use core_types::channels::{ChannelEntry, Ticket};
use utils_db::{db::DB, traits::BinaryAsyncKVStorage};
use utils_types::primitives::{Address, Balance, U256};

use crate::traits::HoprCoreDbActions;

const TICKET_INDEX_PREFIX: &str = "ticketIndex-";

fn prefix_object_with<T: Serialize>(object: &T, prefix: &str) -> Box<[u8]> {
    Box::new([])
}

pub struct CoreDb<T>
where
    T: BinaryAsyncKVStorage,
{
    db: DB<T>,
}

#[async_trait(? Send)] // not placing the `Send` trait limitations on the trait
impl<T: BinaryAsyncKVStorage> HoprCoreDbActions for CoreDb<T> {
    async fn get_current_ticket_index(&self, channel_id: &Hash) -> crate::traits::Result<Option<U256>> {
        todo!()
        // let prefixed_key = prefix_object_with(channel_id, TICKET_INDEX_PREFIX);
        // if self.db.contains(&prefixed_key) {
        //     let value= self.db.get::<Box<[u8]>,U256>(&prefixed_key).await?;
        //     Ok(Some(value))
        // } else {
        //     Ok(None)
        // }
    }

    async fn set_current_ticket_index(&self, channel_id: &Hash, commitment: U256) -> crate::traits::Result<()> {
        // return this.db.put(createCurrentTicketIndexKey(channelId), ticketIndex.serialize())
        todo!()
    }

    async fn get_tickets(&self, predicate: Box<dyn Fn() -> bool>) -> crate::traits::Result<Vec<Ticket>> {
        todo!()
    }

    async fn mark_pending(&self, ticket: &Ticket) {
        todo!()
    }

    async fn mark_rejected(&self, ticket: &Ticket) {
        todo!()
    }

    async fn get_channel_to(&self, dest: PublicKey) -> ChannelEntry {
        todo!()
    }

    async fn get_channel_from(&self, origin: PublicKey) -> ChannelEntry {
        todo!()
    }

    async fn get_pending_balance_to(&self, counterparty: &Address) -> Balance {
        todo!()
    }

    async fn check_and_set_packet_tag(&self, tag: Box<[u8]>) -> bool {
        todo!()
    }

    async fn store_pending_acknowledgment(
        &self,
        half_key_challenge: HalfKeyChallenge,
        is_message_sender: bool,
        unack_ticket: Option<UnacknowledgedTicket>,
    ) {
        todo!()
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn build_core_db(_db: utils_db::leveldb::LevelDb) -> JsValue {
        // TODO: build core db
        JsValue::undefined()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_db_is_able_to_set_tickets() {
        assert!(true)
    }
}
