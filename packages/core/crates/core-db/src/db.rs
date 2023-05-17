use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use core_crypto::types::{HalfKeyChallenge, Hash, PublicKey};
use core_types::acknowledgement::{AcknowledgedTicket, PendingAcknowledgement, UnacknowledgedTicket};
use core_types::channels::{ChannelEntry, Ticket};
use utils_db::{
    db::{serialize_to_bytes, DB},
    traits::BinaryAsyncKVStorage,
};
use utils_types::{
    primitives::{Address, Balance, EthereumChallenge, U256},
    traits::BinarySerializable,
};

use crate::errors::Result;
use crate::traits::HoprCoreDbActions;

const TICKET_INDEX_PREFIX: &str = "ticketIndex-";
const PENDING_TICKETS_COUNT: &str = "statistics:pending:value-";
const REJECTED_TICKETS_COUNT: &str = "statistics:rejected:count";
const REJECTED_TICKETS_VALUE: &str = "statistics:rejected:value";
const PACKET_TAG_PREFIX: &str = "packets:tag-";
const PENDING_ACKNOWLEDGEMENTS_PREFIX: &str = "tickets:pending-acknowledgement-";
const ACKNOWLEDGED_TICKETS_PREFIX: &str = "tickets:acknowledged-";

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

    async fn get_tickets(&self, signer: PublicKey) -> Result<Vec<Ticket>> {
        let mut tickets = self
            .db
            .get_more::<AcknowledgedTicket>(
                Vec::from(ACKNOWLEDGED_TICKETS_PREFIX.as_bytes()).into_boxed_slice(),
                EthereumChallenge::size(),
                &|v: &AcknowledgedTicket| signer == v.signer,
            )
            .await?
            .into_iter()
            .map(|a| a.ticket)
            .collect::<Vec<Ticket>>();
        tickets.sort_by(|l, r| l.index.cmp(&r.index));

        let mut unack_tickets = self
            .db
            .get_more::<PendingAcknowledgement>(
                Vec::from(PENDING_ACKNOWLEDGEMENTS_PREFIX.as_bytes()).into_boxed_slice(),
                HalfKeyChallenge::size(),
                &move |v: &PendingAcknowledgement| match v {
                    PendingAcknowledgement::WaitingAsSender => false,
                    PendingAcknowledgement::WaitingAsRelayer(unack) => signer == unack.signer,
                },
            )
            .await?
            .into_iter()
            .filter_map(|a| match a {
                PendingAcknowledgement::WaitingAsSender => None,
                PendingAcknowledgement::WaitingAsRelayer(unack) => Some(unack.ticket),
            })
            .collect::<Vec<Ticket>>();

        tickets.append(&mut unack_tickets);

        Ok(tickets)
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
        let key = utils_db::db::Key::new_bytes_with_prefix(tag, PACKET_TAG_PREFIX)?;

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
        pending_acknowledgment: PendingAcknowledgement,
    ) -> Result<()> {
        let key = utils_db::db::Key::new_with_prefix(&half_key_challenge, PENDING_ACKNOWLEDGEMENTS_PREFIX)?;

        let _ = self.db.set(key, &pending_acknowledgment).await?;

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
    use core_crypto::types::HalfKeyChallenge;
    use utils_db::db::serialize_to_bytes;
    use utils_types::primitives::EthereumChallenge;
    use utils_types::traits::BinarySerializable;

    #[test]
    fn test_core_db_iterable_type_EhtereumChallenge_must_have_fixed_key_length() {
        let challenge = vec![10u8; EthereumChallenge::SIZE];
        let eth_challenge = EthereumChallenge::new(challenge.as_slice());

        let serialized = serialize_to_bytes(&eth_challenge);

        assert!(serialized.is_ok());
        assert_eq!(serialized.unwrap().len(), EthereumChallenge::SIZE)
    }

    #[test]
    fn test_core_db_iterable_type_HalfKeyChallenge_must_have_fixed_key_length() {
        let challenge = vec![10u8; HalfKeyChallenge::SIZE];
        let eth_challenge = HalfKeyChallenge::new(challenge.as_slice());

        let serialized = serialize_to_bytes(&eth_challenge);

        assert!(serialized.is_ok());
        assert_eq!(serialized.unwrap().len(), HalfKeyChallenge::SIZE)
    }
}
