use async_trait::async_trait;

use core_crypto::types::{HalfKeyChallenge, Hash, PublicKey};
use core_types::acknowledgement::{AcknowledgedTicket, PendingAcknowledgement};
use core_types::channels::{ChannelEntry, Ticket};
use utils_db::traits::AsyncKVStorage;
use utils_db::{
    constants::*,
    db::{serialize_to_bytes, DB},
    traits::BinaryAsyncKVStorage,
};
use utils_types::primitives::Snapshot;
use utils_types::primitives::{Address, Balance, EthereumChallenge, U256};

use crate::errors::Result;
use crate::traits::HoprCoreDbActions;

pub struct CoreDb<T>
where
    T: AsyncKVStorage<Key = Box<[u8]>, Value = Box<[u8]>>,
{
    pub db: DB<T>,
    pub me: PublicKey,
}

#[async_trait(? Send)] // not placing the `Send` trait limitations on the trait
impl<T: AsyncKVStorage<Key = Box<[u8]>, Value = Box<[u8]>>> HoprCoreDbActions for CoreDb<T> {
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

    async fn get_acknowledged_tickets(&self, filter: Option<ChannelEntry>) -> Result<Vec<AcknowledgedTicket>> {
        self.db
            .get_more::<AcknowledgedTicket>(
                Vec::from(ACKNOWLEDGED_TICKETS_PREFIX.as_bytes()).into_boxed_slice(),
                EthereumChallenge::size(),
                &|ack: &AcknowledgedTicket| {
                    if filter.is_none() {
                        true
                    } else {
                        let f = filter.clone().unwrap();
                        f.destination.eq(&self.me) && ack.ticket.channel_epoch.eq(&f.channel_epoch)
                    }
                },
            )
            .await
    }

    async fn get_tickets(&self, signer: &PublicKey) -> Result<Vec<Ticket>> {
        let mut tickets = self
            .db
            .get_more::<AcknowledgedTicket>(
                Vec::from(ACKNOWLEDGED_TICKETS_PREFIX.as_bytes()).into_boxed_slice(),
                EthereumChallenge::size(),
                &|v: &AcknowledgedTicket| v.signer.eq(signer),
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
                    PendingAcknowledgement::WaitingAsRelayer(unack) => unack.signer.eq(signer),
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
    async fn get_channel_to(&self, dest: &PublicKey) -> Result<ChannelEntry> {
        let from = serialize_to_bytes(&self.me.to_address())?;
        let to = serialize_to_bytes(&dest.to_address())?;

        let key = utils_db::db::Key::new(&Hash::create(&[from.as_ref(), to.as_ref()]))?;

        self.db.get::<ChannelEntry>(key).await
    }

    /// TODO: missing key prefix?
    async fn get_channel_from(&self, origin: &PublicKey) -> Result<ChannelEntry> {
        let to = serialize_to_bytes(&self.me.to_address())?;
        let from = serialize_to_bytes(&origin.to_address())?;

        let key = utils_db::db::Key::new(&Hash::create(&[from.as_ref(), to.as_ref()]))?;

        self.db.get::<ChannelEntry>(key).await
    }

    async fn get_pending_balance_to(&self, counterparty: &Address) -> Result<Balance> {
        let key = utils_db::db::Key::new_with_prefix(counterparty, PENDING_TICKETS_COUNT)?;

        self.db.get::<Balance>(key).await
    }

    async fn check_and_set_packet_tag(&mut self, tag: &[u8]) -> Result<bool> {
        let key = utils_db::db::Key::new_bytes_with_prefix(tag, PACKET_TAG_PREFIX)?;

        let has_packet_tag = self.db.contains(key.clone()).await;
        if !has_packet_tag {
            let empty: [u8; 0] = [];
            self.db.set(key, &empty).await?;
        }

        Ok(has_packet_tag)
    }

    async fn get_pending_acknowledgement(
        &self,
        half_key_challenge: &HalfKeyChallenge,
    ) -> Result<Option<PendingAcknowledgement>> {
        let key = utils_db::db::Key::new_with_prefix(half_key_challenge, PENDING_ACKNOWLEDGEMENTS_PREFIX)?;
        if self.db.contains(key.clone()).await {
            let value = self.db.get::<PendingAcknowledgement>(key).await?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
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

    async fn replace_unack_with_ack(
        &mut self,
        half_key_challenge: &HalfKeyChallenge,
        ack_ticket: AcknowledgedTicket,
    ) -> Result<()> {
        let unack_key = utils_db::db::Key::new_with_prefix(half_key_challenge, PENDING_ACKNOWLEDGEMENTS_PREFIX)?;

        let mut ack_key = serialize_to_bytes(&ack_ticket.ticket.challenge)?;
        let mut channel_epoch = serialize_to_bytes(&ack_ticket.ticket.channel_epoch)?;
        ack_key.append(&mut channel_epoch);

        let ack_key =
            utils_db::db::Key::new_bytes_with_prefix(&ack_key.into_boxed_slice(), ACKNOWLEDGED_TICKETS_PREFIX)?;

        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.del(unack_key);
        batch_ops.put(ack_key, ack_ticket);

        self.db.batch(batch_ops, true).await
    }

    async fn update_channel_and_snapshot(
        &mut self,
        channel_id: &Hash,
        channel: ChannelEntry,
        snapshot: Snapshot,
    ) -> Result<()> {
        let channel_key = utils_db::db::Key::new_with_prefix(channel_id, CHANNEL_PREFIX)?;
        let snapshot_key = utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?;

        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.put(channel_key, channel);
        batch_ops.put(snapshot_key, snapshot);

        self.db.batch(batch_ops, true).await
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
    fn test_core_db_iterable_type_ehtereum_challenge_must_have_fixed_key_length() {
        let challenge = vec![10u8; EthereumChallenge::SIZE];
        let eth_challenge = EthereumChallenge::new(challenge.as_slice());

        let serialized = serialize_to_bytes(&eth_challenge);

        assert!(serialized.is_ok());
        assert_eq!(serialized.unwrap().len(), EthereumChallenge::SIZE)
    }

    #[test]
    fn test_core_db_iterable_type_half_key_challenge_must_have_fixed_key_length() {
        let challenge = vec![10u8; HalfKeyChallenge::SIZE];
        let eth_challenge = HalfKeyChallenge::new(challenge.as_slice());

        let serialized = serialize_to_bytes(&eth_challenge);

        assert!(serialized.is_ok());
        assert_eq!(serialized.unwrap().len(), HalfKeyChallenge::SIZE)
    }
}
