use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use core_crypto::iterated_hash::Intermediate;
use core_crypto::{iterated_hash::IteratedHash, types::{HalfKeyChallenge, Hash, PublicKey}};
use core_types::acknowledgement::{AcknowledgedTicket};
use core_types::{channels::{ChannelEntry, Ticket}, account::AccountEntry};
use utils_types::primitives::{Address, Balance, BalanceType, Snapshot, U256};
use utils_db::{constants::*, db::{DB, serialize_to_bytes}, traits::BinaryAsyncKVStorage};

use crate::errors::Result;
use crate::traits::HoprCoreEthereumDbActions;

pub struct CoreEthereumDb<T>
    where
        T: BinaryAsyncKVStorage,
{
    db: DB<T>,
    me: PublicKey,
}

fn to_commitment_key(channel: &Hash, iteration: usize) -> Result<utils_db::db::Key> {
    let mut channel = serialize_to_bytes(channel)?;
    channel.extend_from_slice(&iteration.to_be_bytes());
    utils_db::db::Key::new_bytes_with_prefix(channel.into_boxed_slice(), COMMITMENT_PREFIX)
}

#[async_trait(? Send)] // not placing the `Send` trait limitations on the trait
impl<T: BinaryAsyncKVStorage> HoprCoreEthereumDbActions for CoreEthereumDb<T> {
    async fn get_acknowledged_tickets(&self, filter: ChannelEntry) -> Result<Vec<AcknowledgedTicket>> {
        todo!()
    }

    async fn delete_acknowledged_tickets_from(&mut self, channel: ChannelEntry) -> Result<()> {
        todo!()
    }

    async fn delete_acknowledged_ticket(&mut self, ticket: AcknowledgedTicket) -> Result<()> {
        let mut ack_key = serialize_to_bytes(&ticket.ticket.challenge)?;
        let mut channel_epoch = serialize_to_bytes(&ticket.ticket.channel_epoch)?;
        ack_key.append(&mut channel_epoch);

        let ack_key = utils_db::db::Key::new_bytes_with_prefix(ack_key.into_boxed_slice(), ACKNOWLEDGED_TICKETS_PREFIX)?;

        let _ = self.db.remove::<AcknowledgedTicket>(ack_key).await?;
        Ok(())
    }

    async fn store_hash_intermediaries(&mut self, channel: Hash, iterated_hash: IteratedHash) -> Result<()> {
        let mut batch_ops = utils_db::db::Batch::new();

        for intermediate in iterated_hash.intermediates.iter() {
            batch_ops.put(to_commitment_key(&channel, intermediate.iteration)?, intermediate)
        }

        self.db.batch(batch_ops, true).await
    }

    async fn get_commitment(&self, channel: Hash, iteration: usize) -> Result<Option<Hash>> {
        self.db.get_or_none::<Hash>(to_commitment_key(&channel, iteration)?).await
    }

    async fn get_current_commitment(&self, channel: Hash) -> Result<Option<Hash>> {
        let key = utils_db::db::Key::new_with_prefix(&channel, CURRENT_COMMITMENT_PREFIX)?;
        self.db.get_or_none::<Hash>(key).await
    }

    async fn set_current_commitment(&mut self, channel: Hash, commitment: Hash) -> Result<()> {
        let key = utils_db::db::Key::new_with_prefix(&channel, CURRENT_COMMITMENT_PREFIX)?;
        let _ = self.db.set(key, &commitment).await?;
        Ok(())
    }

    async fn get_latest_block_number(&self) -> Result<u32> {
        let key = utils_db::db::Key::new_from_str(LATEST_BLOCK_NUMBER_KEY)?;
        self.db.get_or_none::<u32>(key).await.map(|v| v.unwrap_or(0))
    }

    async fn update_latest_block_number(&mut self, number: u32) -> Result<()> {
        let key = utils_db::db::Key::new_from_str(LATEST_BLOCK_NUMBER_KEY)?;
        let _ = self.db.set(key, &number).await?;
        Ok(())
    }

    async fn get_latest_confirmed_snapshot(&self) -> Result<Option<Snapshot>> {
        let key = utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?;
        self.db.get_or_none::<Snapshot>(key).await
    }

    async fn get_channel(&self, channel: &Hash) -> Result<Option<ChannelEntry>> {
        let key = utils_db::db::Key::new_with_prefix(channel, CHANNEL_PREFIX)?;
        self.db.get_or_none::<ChannelEntry>(key).await
    }

    async fn get_channels(&self, filter: ChannelEntry) -> Result<Vec<AcknowledgedTicket>> {
        Ok(Vec::new())
    }

    async fn update_channel_and_snapshot(&mut self, channel_id: &Hash, channel: ChannelEntry, snapshot: Snapshot) -> Result<()> {
        let channel_key = utils_db::db::Key::new_with_prefix(channel_id, CHANNEL_PREFIX)?;
        let snapshot_key = utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?;

        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.put(channel_key, channel);
        batch_ops.put(snapshot_key, snapshot);

        self.db.batch(batch_ops, true).await
    }

    async fn get_account(&self, address: &Address) -> Result<Option<AccountEntry>> {
        let key = utils_db::db::Key::new_with_prefix(address, ACCOUNT_PREFIX)?;
        self.db.get_or_none::<AccountEntry>(key).await
    }

    async fn update_account_and_snapshot(&mut self, account: AccountEntry, snapshot: Snapshot) -> Result<()> {
        let address_key = utils_db::db::Key::new_with_prefix(&account.get_address(), ACCOUNT_PREFIX)?;
        let snapshot_key = utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?;

        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.put(address_key, account);
        batch_ops.put(snapshot_key, snapshot);

        self.db.batch(batch_ops, true).await
    }

    async fn get_accounts(&self, address: Address) -> Result<Vec<AccountEntry>> {
        self.db.get_more::<AccountEntry>(Box::from(ACCOUNT_PREFIX.as_bytes()),
                                         Address::size(),
                                         &|_| true).await
    }

    async fn get_redeemed_tickets_value(&self) -> Result<Balance> {
        let key = utils_db::db::Key::new_from_str(REDEEMED_TICKETS_VALUE)?;

        self.db.get_or_none::<Balance>(key).await
            .map(|v| v.unwrap_or(Balance::new(0u32.into(), BalanceType::HOPR)))
    }

    async fn get_redeemed_tickets_count(&self) -> Result<usize> {
        let key = utils_db::db::Key::new_from_str(REDEEMED_TICKETS_COUNT)?;

        self.db.get_or_none::<usize>(key).await.map(|v| v.unwrap_or(0))
    }

    async fn get_neglected_tickets_count(&self) -> Result<usize> {
        let key = utils_db::db::Key::new_from_str(NEGLECTED_TICKET_COUNT)?;

        self.db.get_or_none::<usize>(key).await.map(|v| v.unwrap_or(0))
    }

    async fn get_pending_tickets_count(&self) -> Result<usize> {
        let key = utils_db::db::Key::new_from_str(PENDING_TICKETS_COUNT)?;

        self.db.get_or_none::<usize>(key).await.map(|v| v.unwrap_or(0))
    }

    async fn get_losing_tickets_count(&self) -> Result<usize> {
        let key = utils_db::db::Key::new_from_str(LOSING_TICKET_COUNT)?;

        self.db.get_or_none::<usize>(key).await.map(|v| v.unwrap_or(0))
    }

    async fn get_pendings_balance_to(&self, counterparty: &Address) -> Result<Balance> {
        let key = utils_db::db::Key::new_with_prefix(counterparty, PENDING_TICKETS_COUNT)?;

        self.db.get_or_none::<Balance>(key).await
            .map(|v| v.unwrap_or(Balance::new(0u32.into(), BalanceType::HOPR)))
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

    async fn resolve_pending(&mut self, ticket: &Ticket, snapshot: Snapshot) -> Result<()> {
        todo!()
    }

    async fn mark_redeemeed(&mut self, ticket: &AcknowledgedTicket) -> Result<()> {
        todo!()
    }

    async fn mark_losing_acked_ticket(&mut self, ticket: &AcknowledgedTicket) -> Result<()> {
        todo!()
    }

    async fn get_rejected_tickets_value(&self) -> Result<Balance> {
        let key = utils_db::db::Key::new_from_str(REJECTED_TICKETS_VALUE)?;

        self.db.get_or_none::<Balance>(key).await
            .map(|v| v.unwrap_or(Balance::new(0u32.into(), BalanceType::HOPR)))
    }

    async fn get_rejected_tickets_count(&self) -> Result<usize> {
        let key = utils_db::db::Key::new_from_str(REJECTED_TICKETS_COUNT)?;

        self.db.get_or_none::<usize>(key).await
            .map(|v| v.unwrap_or(0))
    }

    async fn get_channel_x(&self, src: &PublicKey, dest: &PublicKey) -> Result<Option<ChannelEntry>> {
        todo!()
    }

    async fn get_channel_to(&self, dest: &PublicKey) -> Result<Option<ChannelEntry>> {
        todo!()
    }

    async fn get_channel_from(&self, src: &PublicKey) -> Result<Option<ChannelEntry>> {
        todo!()
    }

    async fn get_channels_from(&self, address: Address) -> Result<Vec<ChannelEntry>> {
        todo!()
    }

    async fn get_channels_to(&self, address: Address) -> Result<Vec<ChannelEntry>> {
        todo!()
    }

    async fn get_hopr_balance(&self, src: &PublicKey) -> Result<Balance> {
        let key = utils_db::db::Key::new_from_str(HOPR_BALANCE_KEY)?;

        self.db.get_or_none::<Balance>(key).await
            .map(|v| v.unwrap_or(Balance::new(0u32.into(), BalanceType::HOPR)))
    }

    async fn set_hopr_balance(&mut self, balance: &Balance) -> Result<()> {
        let key = utils_db::db::Key::new_from_str(HOPR_BALANCE_KEY)?;

        let _ = self.db.set::<Balance>(key, balance).await
            .map(|v| v.unwrap_or(Balance::new(0u32.into(), BalanceType::HOPR)))?;

        Ok(())
    }

    async fn add_hopr_balance(&mut self, balance: Balance, snapshot: Snapshot) -> Result<()> {
        todo!()
    }

    async fn sub_hopr_balance(&mut self, balance: Balance, snapshot: Snapshot) -> Result<()> {
        todo!()
    }

    async fn is_network_registry_enabled(&self, snapshot: Snapshot) -> Result<bool> {
        todo!()
    }

    async fn set_network_registry(&mut self, enabled: bool, snapshot: Snapshot) -> Result<()> {
        todo!()
    }

    async fn add_to_network_registry(&mut self, public_key: &PublicKey, account: Address, snapshot: Snapshot) -> Result<()> {
        todo!()
    }

    async fn remove_from_network_registry(&mut self, public_key: &PublicKey, account: Address, snapshot: Snapshot) -> Result<()> {
        todo!()
    }

    async fn get_account_from_network_registry(&self, public_key: &PublicKey) -> Result<Option<Address>> {
        // TODO: FIX THIS, pubkey is not binary serializable
        // let key = utils_db::db::Key::new_with_prefix(public_key, NETWORK_REGISTRY_HOPR_NODE_PREFIX)?;
        let key = utils_db::db::Key::new_from_str(NETWORK_REGISTRY_HOPR_NODE_PREFIX)?;

        self.db.get_or_none::<Address>(key).await
    }

    async fn find_hopr_node_using_account_in_network_registry(&self, account: Address) -> Result<Vec<PublicKey>> {
        todo!()
    }

    async fn is_eligible(&self, account: &Address) -> Result<bool> {
        let key = utils_db::db::Key::new_with_prefix(account,NETWORK_REGISTRY_ADDRESS_ELIGIBLE_PREFIX)?;

        self.db.get_or_none::<bool>(key).await
            .map(|v| v.unwrap_or(false))
    }

    async fn set_eligible(&mut self, account: &Address, eligible: bool, snapshot: Snapshot) -> Result<()> {
        todo!()
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn build_core_ethereum_db(_db: utils_db::leveldb::LevelDb) -> JsValue {
        // TODO: build core ethereum db
        JsValue::undefined()
    }
}

#[cfg(test)]
mod tests {
    use utils_db::db::serialize_to_bytes;
    use utils_types::primitives::EthereumChallenge;
    use utils_types::traits::BinarySerializable;

    #[test]
    fn test_core_ethereum_db_iterable_type_EhtereumChallenge_must_have_fixed_key_length() {
        let challenge = vec![10u8; EthereumChallenge::SIZE];
        let eth_challenge = EthereumChallenge::new(challenge.as_slice());

        let serialized = serialize_to_bytes(&eth_challenge);

        assert!(serialized.is_ok());
        assert_eq!(serialized.unwrap().len(), EthereumChallenge::SIZE)
    }
}
