use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use core_crypto::types::{HalfKeyChallenge, Hash, PublicKey};
use core_types::acknowledgement::{AcknowledgedTicket,UnacknowledgedTicket};
use core_types::channels::{ChannelEntry, Ticket};
use utils_db::{
    db::{serialize_to_bytes, DB},
    traits::BinaryAsyncKVStorage,
};
use utils_types::primitives::{Address, Balance, U256};

use crate::errors::Result;
use crate::traits::HoprCoreDbActions;

const TICKET_INDEX_PREFIX: &str = "ticketIndex-";
const PENDING_TICKETS_COUNT: &str = "statistics:pending:value-";
const REJECTED_TICKETS_COUNT: &str = "statistics:rejected:count";
const REJECTED_TICKETS_VALUE: &str = "statistics:rejected:value";
const PACKET_TAG_PREFIX: &str = "packets:tag-";
const PENDING_ACKNOWLEDGEMENTS_PREFIX: &str = "tickets:pending-acknowledgement-";
const ACKNOWLEDGED_TICKETS_PREFIX: &str = "tickets:acknowledged-";

#[derive(Serialize, Deserialize)]
pub enum PendingAcknowledgementPrefix {
    Relayer = 0,
    MessageSender = 1,
}

pub struct CoreDb<T>
where
    T: BinaryAsyncKVStorage,
{
    db: DB<T>,
    me: PublicKey,
}

#[async_trait(? Send)] // not placing the `Send` trait limitations on the trait
impl<T: BinaryAsyncKVStorage> HoprCoreDbActions for CoreDb<T> {
    async fn get_current_ticket_index(&self, channel_id: &Hash) -> Result<Option<U256>> {
        let prefixed_key = utils_db::db::Key::new_with_prefix(channel_id, TICKET_INDEX_PREFIX)?;
        if self.db.contains(prefixed_key.clone()).await {
            let value = self.db.get::<U256>(prefixed_key).await?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    async fn set_current_ticket_index(&mut self, channel_id: &Hash, index: U256) -> Result<()> {
        let prefixed_key = utils_db::db::Key::new_with_prefix(channel_id, TICKET_INDEX_PREFIX)?;
        let _evicted = self.db.set(prefixed_key, &index).await?;
        // Ignoring evicted value
        Ok(())
    }

    async fn get_tickets(&self, predicate: Box<dyn Fn(&PublicKey) -> bool>) -> Result<Vec<Ticket>> {
        // acknowledged tickets
        let ack_tickets_stream = self.db.get_more(
            Vec::from(ACKNOWLEDGED_TICKETS_PREFIX.as_bytes()).into_boxed_slice(),
            1,
            Box::new(|v: &AcknowledgedTicket| false)).await?;

        // unacknowledged tickets
        //
        let ack_tickets_stream = self.db.get_more(
            Vec::from(ACKNOWLEDGED_TICKETS_PREFIX.as_bytes()).into_boxed_slice(),
            1,
            Box::new(|v: &UnacknowledgedTicket| false)).await?;

        Ok(Vec::new())
    }

    async fn mark_pending(&mut self, ticket: &Ticket) -> Result<()> {
        let prefixed_key = utils_db::db::Key::new_with_prefix(&ticket.counterparty, PENDING_TICKETS_COUNT)?;
        // TODO: verify that this balance type is correct
        let balance = self
            .db
            .get::<Balance>(prefixed_key.clone())
            .await
            .unwrap_or(Balance::new(U256::from(0u64), ticket.amount.balance_type()));

        let _result = self.db.set(prefixed_key, &balance.add(&ticket.amount)).await?;
        Ok(())
    }

    /// TODO: missing key prefix?
    async fn mark_rejected(&mut self, ticket: &Ticket) -> Result<()> {
        let count_key = utils_db::db::Key::new_from_str(REJECTED_TICKETS_COUNT)?;
        // always store as 2^32 - 1 options
        let count = self.db.get::<u128>(count_key.clone()).await?;
        self.db.set(count_key, &(count + 1)).await?;

        let prefixed_key = utils_db::db::Key::new_from_str(REJECTED_TICKETS_VALUE)?;
        let balance = self
            .db
            .get::<Balance>(prefixed_key.clone())
            .await
            .unwrap_or(Balance::new(U256::from(0u64), ticket.amount.balance_type()));

        let _result = self.db.set(prefixed_key, &balance.add(&ticket.amount)).await?;
        Ok(())
    }

    /// TODO: missing key prefix?
    async fn get_channel_to(&self, dest: PublicKey) -> Result<ChannelEntry> {
        let from = serialize_to_bytes(&self.me.to_address())?;
        let to = serialize_to_bytes(&dest.to_address())?;

        let key = utils_db::db::Key::new(&Hash::create(&[from.as_ref(), to.as_ref()]))?;

        self.db.get::<ChannelEntry>(key).await
    }

    /// TODO: missing key prefix?
    async fn get_channel_from(&self, origin: PublicKey) -> Result<ChannelEntry> {
        let to = serialize_to_bytes(&self.me.to_address())?;
        let from = serialize_to_bytes(&origin.to_address())?;

        let key = utils_db::db::Key::new(&Hash::create(&[from.as_ref(), to.as_ref()]))?;

        self.db.get::<ChannelEntry>(key).await
    }

    async fn get_pending_balance_to(&self, counterparty: &Address) -> Result<Balance> {
        let key = utils_db::db::Key::new_with_prefix(counterparty, PENDING_TICKETS_COUNT)?;

        self.db.get::<Balance>(key).await
    }

    async fn check_and_set_packet_tag(&mut self, tag: Box<[u8]>) -> Result<bool> {
        let key = utils_db::db::Key::new_with_prefix(&tag, PACKET_TAG_PREFIX)?;

        let has_packet_tag = self.db.contains(key.clone()).await;
        if !has_packet_tag {
            let empty: [u8; 0] = [];
            self.db.set(key, &empty).await?;
        }

        Ok(has_packet_tag)
    }

    async fn store_pending_acknowledgment(
        &mut self,
        half_key_challenge: HalfKeyChallenge,
        is_message_sender: bool,
        unack_ticket: Option<UnacknowledgedTicket>,
    ) -> Result<()> {
        let key = utils_db::db::Key::new_with_prefix(&half_key_challenge, PENDING_ACKNOWLEDGEMENTS_PREFIX)?;

        let value = if is_message_sender {
            serialize_to_bytes(&PendingAcknowledgementPrefix::MessageSender)?
        } else {
            let unacked_ticket_se = serialize_to_bytes(&unack_ticket)?;
            let mut out = serialize_to_bytes(&PendingAcknowledgementPrefix::Relayer)?;
            out.extend_from_slice(&unacked_ticket_se);
            out
        };

        let _ = self.db.set(key, &value).await?;

        Ok(())
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
    // use super::*;

    #[test]
    fn test_core_db_is_able_to_set_tickets() {
        assert!(true)
    }
}
