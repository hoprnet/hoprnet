use async_trait::async_trait;

use core_crypto::iterated_hash::Intermediate;
use core_crypto::{
    iterated_hash::IteratedHash,
    types::{Hash, PublicKey},
};
use core_types::acknowledgement::AcknowledgedTicket;
use core_types::channels::ChannelStatus;
use core_types::{
    account::AccountEntry,
    channels::{generate_channel_id, ChannelEntry, Ticket},
};
use utils_db::{
    constants::*,
    db::{serialize_to_bytes, DB},
    traits::AsyncKVStorage,
};
use utils_types::primitives::{Address, Balance, BalanceType, EthereumChallenge, Snapshot, U256};

use crate::errors::Result;
use crate::traits::HoprCoreEthereumDbActions;

fn to_commitment_key(channel: &Hash, iteration: usize) -> Result<utils_db::db::Key> {
    let mut channel = serialize_to_bytes(channel)?;
    channel.extend_from_slice(&iteration.to_be_bytes());

    utils_db::db::Key::new_bytes_with_prefix(&channel, COMMITMENT_PREFIX)
}

fn to_acknowledged_ticket_key(challenge: &EthereumChallenge, epoch: &U256) -> Result<utils_db::db::Key> {
    let mut ack_key = serialize_to_bytes(challenge)?;
    let mut channel_epoch = serialize_to_bytes(epoch)?;
    ack_key.append(&mut channel_epoch);

    utils_db::db::Key::new_bytes_with_prefix(&ack_key, ACKNOWLEDGED_TICKETS_PREFIX)
}

pub struct CoreEthereumDb<T>
where
    T: AsyncKVStorage<Key = Box<[u8]>, Value = Box<[u8]>>,
{
    pub db: DB<T>,
    pub me: PublicKey,
}

impl<T: AsyncKVStorage<Key = Box<[u8]>, Value = Box<[u8]>>> CoreEthereumDb<T> {
    pub fn new(db: DB<T>, public_key: PublicKey) -> Self {
        Self { db, me: public_key }
    }
}


#[async_trait(? Send)] // not placing the `Send` trait limitations on the trait
impl<T: AsyncKVStorage<Key = Box<[u8]>, Value = Box<[u8]>>> HoprCoreEthereumDbActions for CoreEthereumDb<T> {
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

    async fn delete_acknowledged_tickets_from(&mut self, channel: ChannelEntry) -> Result<()> {
        let acknowledged_tickets = self.get_acknowledged_tickets(Some(channel)).await?;

        let key = utils_db::db::Key::new_from_str(NEGLECTED_TICKET_COUNT)?;
        let neglected_ticket_count = match self.db.get_or_none::<usize>(key.clone()).await? {
            Some(x) => x,
            None => 0,
        };

        let mut batch_ops = utils_db::db::Batch::new();
        for ticket in acknowledged_tickets.iter() {
            batch_ops.del(to_acknowledged_ticket_key(
                &ticket.ticket.challenge,
                &ticket.ticket.channel_epoch,
            )?);
        }

        if acknowledged_tickets.len() > 0 {
            batch_ops.put(key, neglected_ticket_count + acknowledged_tickets.len())
        }

        self.db.batch(batch_ops, true).await
    }

    async fn delete_acknowledged_ticket(&mut self, ticket: &AcknowledgedTicket) -> Result<()> {
        let ack_key = to_acknowledged_ticket_key(&ticket.ticket.challenge, &ticket.ticket.channel_epoch)?;
        let _ = self.db.remove::<AcknowledgedTicket>(ack_key).await?;
        Ok(())
    }

    async fn store_hash_intermediaries(&mut self, channel: &Hash, iterated_hash: &IteratedHash) -> Result<()> {
        let mut batch_ops = utils_db::db::Batch::new();

        for intermediate in iterated_hash.intermediates.iter() {
            batch_ops.put(to_commitment_key(&channel, intermediate.iteration)?, intermediate);
        }

        self.db.batch(batch_ops, true).await
    }

    async fn get_commitment(&self, channel: &Hash, iteration: usize) -> Result<Option<Hash>> {
        self.db
            .get_or_none::<Intermediate>(to_commitment_key(channel, iteration)?)
            .await
            .map(|opt| opt.map(|i| Hash::new(&i.intermediate)))
    }

    async fn get_current_commitment(&self, channel: &Hash) -> Result<Option<Hash>> {
        let key = utils_db::db::Key::new_with_prefix(channel, CURRENT_COMMITMENT_PREFIX)?;
        self.db.get_or_none::<Hash>(key).await
    }

    async fn set_current_commitment(&mut self, channel: &Hash, commitment: &Hash) -> Result<()> {
        let key = utils_db::db::Key::new_with_prefix(channel, CURRENT_COMMITMENT_PREFIX)?;
        let _ = self.db.set(key, commitment).await?;
        Ok(())
    }

    async fn get_latest_block_number(&self) -> Result<u32> {
        let key = utils_db::db::Key::new_from_str(LATEST_BLOCK_NUMBER_KEY)?;
        self.db
            .get_or_none::<u32>(key)
            .await
            .map(|v| v.unwrap_or(0))
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

    async fn get_channels(&self) -> Result<Vec<ChannelEntry>> {
        self.db
            .get_more::<ChannelEntry>(Box::from(CHANNEL_PREFIX.as_bytes()), Hash::size(), &|_| true)
            .await
    }

    async fn get_channels_open(&self) -> Result<Vec<ChannelEntry>> {
        Ok(self
            .db
            .get_more::<ChannelEntry>(Box::from(CHANNEL_PREFIX.as_bytes()), Hash::size(), &|_| true)
            .await?
            .into_iter()
            .filter(|x| x.status == ChannelStatus::Open)
            .collect())
    }

    async fn update_channel_and_snapshot(
        &mut self,
        channel_id: &Hash,
        channel: &ChannelEntry,
        snapshot: &Snapshot,
    ) -> Result<()> {
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

    async fn update_account_and_snapshot(&mut self, account: &AccountEntry, snapshot: &Snapshot) -> Result<()> {
        let address_key = utils_db::db::Key::new_with_prefix(&account.get_address(), ACCOUNT_PREFIX)?;
        let snapshot_key = utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?;

        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.put(address_key, account);
        batch_ops.put(snapshot_key, snapshot);

        self.db.batch(batch_ops, true).await
    }

    async fn get_accounts(&self) -> Result<Vec<AccountEntry>> {
        self.db
            .get_more::<AccountEntry>(Box::from(ACCOUNT_PREFIX.as_bytes()), Address::size(), &|_| true)
            .await
    }

    async fn get_redeemed_tickets_value(&self) -> Result<Balance> {
        let key = utils_db::db::Key::new_from_str(REDEEMED_TICKETS_VALUE)?;

        self.db
            .get_or_none::<Balance>(key)
            .await
            .map(|v| v.unwrap_or(Balance::zero(BalanceType::HOPR)))
    }

    async fn get_redeemed_tickets_count(&self) -> Result<usize> {
        let key = utils_db::db::Key::new_from_str(REDEEMED_TICKETS_COUNT)?;

        self.db
            .get_or_none::<usize>(key)
            .await
            .map(|v| v.unwrap_or(0))
    }

    async fn get_neglected_tickets_count(&self) -> Result<usize> {
        let key = utils_db::db::Key::new_from_str(NEGLECTED_TICKET_COUNT)?;

        self.db
            .get_or_none::<usize>(key)
            .await
            .map(|v| v.unwrap_or(0))
    }

    async fn get_pending_tickets_count(&self) -> Result<usize> {
        let key = utils_db::db::Key::new_from_str(PENDING_TICKETS_COUNT)?;

        self.db
            .get_or_none::<usize>(key)
            .await
            .map(|v| v.unwrap_or(0))
    }

    async fn get_losing_tickets_count(&self) -> Result<usize> {
        let key = utils_db::db::Key::new_from_str(LOSING_TICKET_COUNT)?;

        self.db
            .get_or_none::<usize>(key)
            .await
            .map(|v| v.unwrap_or(0))
    }

    async fn get_pending_balance_to(&self, counterparty: &Address) -> Result<Balance> {
        let key = utils_db::db::Key::new_with_prefix(counterparty, PENDING_TICKETS_COUNT)?;

        self.db
            .get_or_none::<Balance>(key)
            .await
            .map(|v| v.unwrap_or(Balance::zero(BalanceType::HOPR)))
    }

    async fn mark_pending(&mut self, ticket: &Ticket) -> Result<()> {
        let prefixed_key = utils_db::db::Key::new_with_prefix(&ticket.counterparty, PENDING_TICKETS_COUNT)?;
        let balance = self
            .db
            .get::<Balance>(prefixed_key.clone())
            .await
            .unwrap_or(Balance::zero(ticket.amount.balance_type()));

        let _result = self
            .db
            .set(prefixed_key, &balance.add(&ticket.amount))
            .await?;
        Ok(())
    }

    async fn resolve_pending(&mut self, ticket: &Ticket, snapshot: &Snapshot) -> Result<()> {
        let key = utils_db::db::Key::new_with_prefix(&ticket.counterparty, PENDING_TICKETS_COUNT)?;
        let balance = self
            .db
            .get_or_none(key.clone())
            .await?
            .unwrap_or(Balance::zero(BalanceType::HOPR));

        let mut batch_ops = utils_db::db::Batch::new();
        // NOTE: This operation does not make sense, does it mean to zero out? Why not store zero then?
        batch_ops.put(key.clone(), &balance.sub(&balance));
        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            &snapshot,
        );

        self.db.batch(batch_ops, true).await
    }

    async fn mark_redeemed(&mut self, ticket: &AcknowledgedTicket) -> Result<()> {
        let key = utils_db::db::Key::new_from_str(REDEEMED_TICKETS_COUNT)?;
        let count = self
            .db
            .get_or_none::<usize>(key.clone())
            .await?
            .unwrap_or(0);
        let _ = self.db.set(key, &(count + 1)).await?;

        let key = to_acknowledged_ticket_key(&ticket.ticket.challenge, &ticket.ticket.channel_epoch)?;
        let _ = self.db.remove::<AcknowledgedTicket>(key).await?;

        let key = utils_db::db::Key::new_from_str(REDEEMED_TICKETS_VALUE)?;
        let balance = self
            .db
            .get_or_none::<Balance>(key.clone())
            .await?
            .unwrap_or(Balance::zero(BalanceType::HOPR))
            .add(&ticket.ticket.amount);
        let _ = self.db.set(key, &balance).await?;

        let key = utils_db::db::Key::new_with_prefix(&ticket.ticket.counterparty, PENDING_TICKETS_COUNT)?;
        let balance = self
            .db
            .get_or_none::<Balance>(key.clone())
            .await?
            .unwrap_or(Balance::zero(BalanceType::HOPR))
            .sub(&ticket.ticket.amount);
        let _ = self.db.set(key, &balance).await?;

        Ok(())
    }

    async fn mark_losing_acked_ticket(&mut self, ticket: &AcknowledgedTicket) -> Result<()> {
        let key = utils_db::db::Key::new_from_str(LOSING_TICKET_COUNT)?;
        let count = self
            .db
            .get_or_none::<usize>(key.clone())
            .await?
            .unwrap_or(0);
        let _ = self.db.set(key, &(count + 1)).await?;

        let key = to_acknowledged_ticket_key(&ticket.ticket.challenge, &ticket.ticket.channel_epoch)?;
        let _ = self.db.remove::<AcknowledgedTicket>(key).await?;

        let key = utils_db::db::Key::new_with_prefix(&ticket.ticket.counterparty, PENDING_TICKETS_COUNT)?;
        let balance = self
            .db
            .get_or_none::<Balance>(key.clone())
            .await?
            .unwrap_or(Balance::zero(BalanceType::HOPR))
            .sub(&ticket.ticket.amount);
        let _ = self.db.set(key, &balance).await?;

        Ok(())
    }

    async fn get_rejected_tickets_value(&self) -> Result<Balance> {
        let key = utils_db::db::Key::new_from_str(REJECTED_TICKETS_VALUE)?;

        self.db
            .get_or_none::<Balance>(key)
            .await
            .map(|v| v.unwrap_or(Balance::zero(BalanceType::HOPR)))
    }

    async fn get_rejected_tickets_count(&self) -> Result<usize> {
        let key = utils_db::db::Key::new_from_str(REJECTED_TICKETS_COUNT)?;

        self.db
            .get_or_none::<usize>(key)
            .await
            .map(|v| v.unwrap_or(0))
    }

    async fn get_channel_x(&self, src: &PublicKey, dest: &PublicKey) -> Result<Option<ChannelEntry>> {
        let key = utils_db::db::Key::new_with_prefix(&generate_channel_id(&src.to_address(), &dest.to_address()), "")?;

        self.db.get_or_none(key).await
    }

    async fn get_channel_to(&self, dest: &PublicKey) -> Result<Option<ChannelEntry>> {
        let key =
            utils_db::db::Key::new_with_prefix(&generate_channel_id(&self.me.to_address(), &dest.to_address()), "")?;

        self.db.get_or_none(key).await
    }

    async fn get_channel_from(&self, src: &PublicKey) -> Result<Option<ChannelEntry>> {
        let key =
            utils_db::db::Key::new_with_prefix(&generate_channel_id(&src.to_address(), &self.me.to_address()), "")?;

        self.db.get_or_none(key).await
    }

    async fn get_channels_from(&self, address: Address) -> Result<Vec<ChannelEntry>> {
        Ok(self
            .db
            .get_more::<ChannelEntry>(Box::from(CHANNEL_PREFIX.as_bytes()), Hash::size(), &|_| true)
            .await?
            .into_iter()
            .filter(move |x| x.source.to_address() == address)
            .collect())
    }

    async fn get_channels_to(&self, address: Address) -> Result<Vec<ChannelEntry>> {
        Ok(self
            .db
            .get_more::<ChannelEntry>(Box::from(CHANNEL_PREFIX.as_bytes()), Hash::size(), &|_| true)
            .await?
            .into_iter()
            .filter(move |x| x.destination.to_address() == address)
            .collect())
    }

    async fn get_hopr_balance(&self) -> Result<Balance> {
        let key = utils_db::db::Key::new_from_str(HOPR_BALANCE_KEY)?;

        self.db
            .get_or_none::<Balance>(key)
            .await
            .map(|v| v.unwrap_or(Balance::zero(BalanceType::HOPR)))
    }

    async fn set_hopr_balance(&mut self, balance: &Balance) -> Result<()> {
        let key = utils_db::db::Key::new_from_str(HOPR_BALANCE_KEY)?;

        let _ = self
            .db
            .set::<Balance>(key, balance)
            .await
            .map(|v| v.unwrap_or(Balance::zero(BalanceType::HOPR)))?;

        Ok(())
    }

    async fn add_hopr_balance(&mut self, balance: &Balance, snapshot: &Snapshot) -> Result<()> {
        let key = utils_db::db::Key::new_from_str(HOPR_BALANCE_KEY)?;

        let current_balance = self
            .db
            .get_or_none::<Balance>(key.clone())
            .await?
            .unwrap_or(Balance::zero(BalanceType::HOPR));

        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.put(key, &current_balance.add(&balance));
        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            &snapshot,
        );

        self.db.batch(batch_ops, true).await
    }

    async fn sub_hopr_balance(&mut self, balance: &Balance, snapshot: &Snapshot) -> Result<()> {
        let key = utils_db::db::Key::new_from_str(HOPR_BALANCE_KEY)?;

        let current_balance = self
            .db
            .get_or_none::<Balance>(key.clone())
            .await?
            .unwrap_or(Balance::zero(BalanceType::HOPR));

        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.put(key, &current_balance.sub(&balance));
        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            &snapshot,
        );

        self.db.batch(batch_ops, true).await
    }

    async fn is_network_registry_enabled(&self) -> Result<bool> {
        let key = utils_db::db::Key::new_from_str(NETWORK_REGISTRY_ENABLED_PREFIX)?;
        Ok(self
            .db
            .get_or_none::<bool>(key.clone())
            .await?
            .unwrap_or(false))
    }

    async fn set_network_registry(&mut self, enabled: bool, snapshot: &Snapshot) -> Result<()> {
        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.put(
            utils_db::db::Key::new_from_str(NETWORK_REGISTRY_ENABLED_PREFIX)?,
            &enabled,
        );
        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            &snapshot,
        );

        self.db.batch(batch_ops, true).await
    }

    async fn add_to_network_registry(
        &mut self,
        public_key: &PublicKey,
        account: &Address,
        snapshot: &Snapshot,
    ) -> Result<()> {
        let mut public_keys = self.find_hopr_node_using_account_in_network_registry(&account).await?;

        for pk in public_keys.iter() {
            if public_key == pk {
                let _ = self
                    .db
                    .set(
                        utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
                        snapshot,
                    )
                    .await?;
                return Ok(());
            }
        }

        public_keys.push(public_key.clone());

        let mut batch_ops = utils_db::db::Batch::new();
        // node public key to address (N->1)
        let curve_point: core_crypto::types::CurvePoint = public_key.into();
        batch_ops.put(
            utils_db::db::Key::new_with_prefix(&curve_point, NETWORK_REGISTRY_HOPR_NODE_PREFIX)?,
            account,
        );
        // address to node public keys (1->M)
        batch_ops.put(
            utils_db::db::Key::new_with_prefix(account, NETWORK_REGISTRY_ADDRESS_PUBLIC_KEY_PREFIX)?,
            &public_keys,
        );
        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            snapshot,
        );

        self.db.batch(batch_ops, true).await
    }

    async fn remove_from_network_registry(
        &mut self,
        public_key: &PublicKey,
        account: &Address,
        snapshot: &Snapshot,
    ) -> Result<()> {
        let registered_nodes = self
            .find_hopr_node_using_account_in_network_registry(account)
            .await?
            .into_iter()
            .filter(|pk| pk != public_key)
            .collect::<Vec<_>>();

        let curve_point: core_crypto::types::CurvePoint = public_key.into();

        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.del(utils_db::db::Key::new_with_prefix(
            &curve_point,
            NETWORK_REGISTRY_HOPR_NODE_PREFIX,
        )?);
        batch_ops.put(
            utils_db::db::Key::new_with_prefix(account, NETWORK_REGISTRY_ADDRESS_PUBLIC_KEY_PREFIX)?,
            &registered_nodes,
        );
        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            &snapshot,
        );

        self.db.batch(batch_ops, true).await
    }

    async fn get_account_from_network_registry(&self, public_key: &PublicKey) -> Result<Option<Address>> {
        let curve_point: core_crypto::types::CurvePoint = public_key.into();
        let key = utils_db::db::Key::new_with_prefix(&curve_point, NETWORK_REGISTRY_HOPR_NODE_PREFIX)?;

        self.db.get_or_none::<Address>(key).await
    }

    async fn find_hopr_node_using_account_in_network_registry(&self, account: &Address) -> Result<Vec<PublicKey>> {
        // NOTE: behavioral change, this method does not panic, when no results are found,
        // its returns an empty Vec instead

        let key = utils_db::db::Key::new_with_prefix(account, NETWORK_REGISTRY_ADDRESS_PUBLIC_KEY_PREFIX)?;
        self.db
            .get_or_none::<Vec<PublicKey>>(key)
            .await
            .map(|v| v.unwrap_or(Vec::new()))
    }

    async fn is_eligible(&self, account: &Address) -> Result<bool> {
        let key = utils_db::db::Key::new_with_prefix(account, NETWORK_REGISTRY_ADDRESS_ELIGIBLE_PREFIX)?;

        self.db
            .get_or_none::<bool>(key)
            .await
            .map(|v| v.unwrap_or(false))
    }

    async fn set_eligible(&mut self, account: &Address, eligible: bool, snapshot: &Snapshot) -> Result<()> {
        let key = utils_db::db::Key::new_with_prefix(account, NETWORK_REGISTRY_ADDRESS_ELIGIBLE_PREFIX)?;

        let mut batch_ops = utils_db::db::Batch::new();

        if eligible {
            batch_ops.put(key, &[0u8; 0]);
        } else {
            batch_ops.del(key);
        }

        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            snapshot,
        );
        self.db.batch(batch_ops, true).await
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use super::{CoreEthereumDb, HoprCoreEthereumDbActions, PublicKey, DB};
    use wasm_bindgen::prelude::*;
    use core_crypto::types::Hash;
    use core_types::account::AccountEntry;
    use core_types::acknowledgement::AcknowledgedTicket;
    use core_types::channels::{ChannelEntry, Ticket};
    use utils_db::leveldb;
    use utils_types::primitives::{Address, Balance, Snapshot};

    macro_rules! to_iterable {
        ($obj:ident,$x:ty) => (
            #[wasm_bindgen]
            pub struct $obj {
                v: Vec<$x>
            }

            impl $obj {
                pub fn from(value: Vec<$x>) -> Self {
                    Self {
                        v: value
                    }
                }
            }

            #[wasm_bindgen]
            impl $obj {
                #[wasm_bindgen]
                pub fn next(&mut self) -> Option<$x> {
                    if self.v.len() > 0 {
                        Some(self.v.remove(0))
                    } else {
                        None
                    }
                }
            }
        )
    }

    to_iterable!(WasmVecAcknowledgedTicket, AcknowledgedTicket);
    to_iterable!(WasmVecChannelEntry, ChannelEntry);
    to_iterable!(WasmVecAccountEntry, AccountEntry);
    to_iterable!(WasmVecPublicKey, PublicKey);

    #[wasm_bindgen]
    pub struct CoreEthereumDatabase
    {
        db: CoreEthereumDb<leveldb::LevelDbShim>,
    }

    #[wasm_bindgen]
    impl CoreEthereumDatabase {
        #[wasm_bindgen(constructor)]
        pub fn new(db: utils_db::leveldb::LevelDb, public_key: PublicKey) -> Self {
            Self {
                db: CoreEthereumDb::<leveldb::LevelDbShim>::new(
                    DB::<leveldb::LevelDbShim>::new(leveldb::LevelDbShim::new(db)), public_key)
            }
        }
    }

    #[wasm_bindgen]
    impl CoreEthereumDatabase  {
        #[wasm_bindgen]
        pub async fn get_acknowledged_tickets(&self, filter: Option<ChannelEntry>) -> Result<WasmVecAcknowledgedTicket,JsValue>
        {
            utils_misc::ok_or_jserr!(self.db.get_acknowledged_tickets(filter).await)
                .map(|v| WasmVecAcknowledgedTicket::from(v))
        }

        #[wasm_bindgen]
        pub async fn delete_acknowledged_tickets_from(&mut self, source: ChannelEntry) -> Result<(),JsValue> {
            utils_misc::ok_or_jserr!(self.db.delete_acknowledged_tickets_from(source).await)
        }

        #[wasm_bindgen]
        pub async fn delete_acknowledged_ticket(&mut self, ticket: &AcknowledgedTicket) -> Result<(),JsValue> {
            utils_misc::ok_or_jserr!(self.db.delete_acknowledged_ticket(ticket).await)
        }

        #[wasm_bindgen]
        pub async fn get_commitment(&self, channel: &Hash, iteration: usize) -> Result<Option<Hash>,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_commitment(channel, iteration).await)
        }

        #[wasm_bindgen]
        pub async fn get_current_commitment(&self, channel: &Hash) -> Result<Option<Hash>,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_current_commitment(channel).await)
        }

        #[wasm_bindgen]
        pub async fn set_current_commitment(&mut self, channel: &Hash, commitment: &Hash) -> Result<(),JsValue> {
            utils_misc::ok_or_jserr!(self.db.set_current_commitment(channel, commitment).await)
        }

        #[wasm_bindgen]
        pub async fn get_latest_block_number(&self) -> Result<u32,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_latest_block_number().await)
        }

        #[wasm_bindgen]
        pub async fn update_latest_block_number(&mut self, number: u32) -> Result<(),JsValue> {
            utils_misc::ok_or_jserr!(self.db.update_latest_block_number(number).await)
        }

        #[wasm_bindgen]
        pub async fn get_latest_confirmed_snapshot(&self) -> Result<Option<Snapshot>,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_latest_confirmed_snapshot().await)
        }

        #[wasm_bindgen]
        pub async fn get_channel(&self, channel: &Hash) -> Result<Option<ChannelEntry>,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_channel(channel).await)
        }

        #[wasm_bindgen]
        pub async fn get_channels(&self) -> Result<WasmVecChannelEntry,JsValue>
        {
            utils_misc::ok_or_jserr!(self.db.get_channels().await)
                .map(|v| WasmVecChannelEntry::from(v))
        }

        pub async fn get_channels_open(&self) -> Result<WasmVecChannelEntry,JsValue>
        {
            utils_misc::ok_or_jserr!(self.db.get_channels_open().await)
                .map(|v| WasmVecChannelEntry::from(v))
        }

        #[wasm_bindgen]
        pub async fn update_channel_and_snapshot(&mut self, channel_id: &Hash, channel: &ChannelEntry, snapshot: &Snapshot) -> Result<(),JsValue> {
            utils_misc::ok_or_jserr!(self.db.update_channel_and_snapshot(channel_id, channel, snapshot).await)
        }

        #[wasm_bindgen]
        pub async fn get_account(&self, address: &Address) -> Result<Option<AccountEntry>,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_account(address).await)
        }

        #[wasm_bindgen]
        pub async fn update_account_and_snapshot(&mut self, account: &AccountEntry, snapshot: &Snapshot) -> Result<(),JsValue> {
            utils_misc::ok_or_jserr!(self.db.update_account_and_snapshot(account, snapshot).await)
        }

        #[wasm_bindgen]
        pub async fn get_accounts(&self) -> Result<WasmVecAccountEntry,JsValue>
        {
            utils_misc::ok_or_jserr!(self.db.get_accounts().await)
                .map(|v| WasmVecAccountEntry::from(v))
        }

        #[wasm_bindgen]
        pub async fn get_redeemed_tickets_value(&self) -> Result<Balance,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_redeemed_tickets_value().await)
        }

        #[wasm_bindgen]
        pub async fn get_redeemed_tickets_count(&self) -> Result<usize,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_redeemed_tickets_count().await)
        }

        #[wasm_bindgen]
        pub async fn get_neglected_tickets_count(&self) -> Result<usize,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_neglected_tickets_count().await)
        }

        #[wasm_bindgen]
        pub async fn get_pending_tickets_count(&self) -> Result<usize,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_pending_tickets_count().await)
        }

        #[wasm_bindgen]
        pub async fn get_losing_tickets_count(&self) -> Result<usize,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_losing_tickets_count().await)
        }

        #[wasm_bindgen]
        pub async fn get_pending_balance_to(&self, counterparty: &Address) -> Result<Balance,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_pending_balance_to(counterparty).await)
        }

        #[wasm_bindgen]
        pub async fn mark_pending(&mut self, ticket: &Ticket) -> Result<(),JsValue> {
            utils_misc::ok_or_jserr!(self.db.mark_pending(ticket).await)
        }

        #[wasm_bindgen]
        pub async fn resolve_pending(&mut self, ticket: &Ticket, snapshot: &Snapshot) -> Result<(),JsValue> {
            utils_misc::ok_or_jserr!(self.db.resolve_pending(ticket, snapshot).await)
        }

        #[wasm_bindgen]
        pub async fn mark_redeemed(&mut self, ticket: &AcknowledgedTicket) -> Result<(),JsValue> {
            utils_misc::ok_or_jserr!(self.db.mark_redeemed(ticket).await)
        }

        #[wasm_bindgen]
        pub async fn mark_losing_acked_ticket(&mut self, ticket: &AcknowledgedTicket) -> Result<(),JsValue> {
            utils_misc::ok_or_jserr!(self.db.mark_losing_acked_ticket(ticket).await)
        }

        #[wasm_bindgen]
        pub async fn get_rejected_tickets_value(&self) -> Result<Balance,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_rejected_tickets_value().await)
        }

        #[wasm_bindgen]
        pub async fn get_rejected_tickets_count(&self) -> Result<usize,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_rejected_tickets_count().await)
        }

        #[wasm_bindgen]
        pub async fn get_channel_x(&self, src: &PublicKey, dest: &PublicKey) -> Result<Option<ChannelEntry>,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_channel_x(src, dest).await)
        }

        #[wasm_bindgen]
        pub async fn get_channel_to(&self, dest: &PublicKey) -> Result<Option<ChannelEntry>,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_channel_to(dest).await)
        }

        #[wasm_bindgen]
        pub async fn get_channel_from(&self, src: &PublicKey) -> Result<Option<ChannelEntry>,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_channel_from(src).await)
        }

        #[wasm_bindgen]
        pub async fn get_channels_from(&self, address: Address) -> Result<WasmVecChannelEntry,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_channels_from(address).await)
                .map(|v| WasmVecChannelEntry::from(v))
        }

        #[wasm_bindgen]
        pub async fn get_channels_to(&self, address: Address) -> Result<WasmVecChannelEntry,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_channels_to(address).await)
                .map(|v| WasmVecChannelEntry::from(v))
        }

        #[wasm_bindgen]
        pub async fn get_hopr_balance(&self) -> Result<Balance,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_hopr_balance().await)
        }

        #[wasm_bindgen]
        pub async fn set_hopr_balance(&mut self, balance: &Balance) -> Result<(),JsValue> {
            utils_misc::ok_or_jserr!(self.db.set_hopr_balance(balance).await)
        }

        #[wasm_bindgen]
        pub async fn add_hopr_balance(&mut self, balance: &Balance, snapshot: &Snapshot) -> Result<(),JsValue> {
            utils_misc::ok_or_jserr!(self.db.add_hopr_balance(balance, snapshot).await)
        }

        #[wasm_bindgen]
        pub async fn sub_hopr_balance(&mut self, balance: &Balance, snapshot: &Snapshot) -> Result<(),JsValue> {
            utils_misc::ok_or_jserr!(self.db.sub_hopr_balance(balance, snapshot).await)
        }

        #[wasm_bindgen]
        pub async fn is_network_registry_enabled(&self) -> Result<bool,JsValue> {
            utils_misc::ok_or_jserr!(self.db.is_network_registry_enabled().await)
        }

        #[wasm_bindgen]
        pub async fn set_network_registry(&mut self, enabled: bool, snapshot: &Snapshot) -> Result<(),JsValue> {
            utils_misc::ok_or_jserr!(self.db.set_network_registry(enabled, snapshot).await)
        }

        #[wasm_bindgen]
        pub async fn add_to_network_registry(&mut self, public_key: &PublicKey, account: &Address, snapshot: &Snapshot) -> Result<(),JsValue> {
            utils_misc::ok_or_jserr!(self.db.add_to_network_registry(public_key, account, snapshot).await)
        }

        #[wasm_bindgen]
        pub async fn remove_from_network_registry(&mut self, public_key: &PublicKey, account: &Address, snapshot: &Snapshot) -> Result<(),JsValue> {
            utils_misc::ok_or_jserr!(self.db.remove_from_network_registry(public_key, account, snapshot).await)
        }

        #[wasm_bindgen]
        pub async fn get_account_from_network_registry(&self, public_key: &PublicKey) -> Result<Option<Address>,JsValue> {
            utils_misc::ok_or_jserr!(self.db.get_account_from_network_registry(public_key).await)
        }

        #[wasm_bindgen]
        pub async fn find_hopr_node_using_account_in_network_registry(&self, account: &Address) -> Result<WasmVecPublicKey,JsValue> {
            utils_misc::ok_or_jserr!(self.db.find_hopr_node_using_account_in_network_registry(account).await)
                .map(|v| WasmVecPublicKey::from(v))
        }

        #[wasm_bindgen]
        pub async fn is_eligible(&self, account: &Address) -> Result<bool,JsValue> {
            utils_misc::ok_or_jserr!(self.db.is_eligible(account).await)
        }

        #[wasm_bindgen]
        pub async fn set_eligible(&mut self, account: &Address, eligible: bool, snapshot: &Snapshot) -> Result<(),JsValue> {
            utils_misc::ok_or_jserr!(self.db.set_eligible(account, eligible, snapshot).await)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_types::channels::ChannelEntry;
    use utils_db::db::serialize_to_bytes;
    use utils_types::primitives::EthereumChallenge;
    use utils_types::traits::BinarySerializable;

    #[test]
    fn test_core_ethereum_db_iterable_type_ethereumchallenge_must_have_fixed_key_length() {
        let challenge = vec![10u8; EthereumChallenge::SIZE];
        let eth_challenge = EthereumChallenge::new(challenge.as_slice());

        let serialized = serialize_to_bytes(&eth_challenge);

        assert!(serialized.is_ok());
        assert_eq!(serialized.unwrap().len(), EthereumChallenge::SIZE)
    }

    #[test]
    fn test_core_ethereum_db_iterable_type_channelentry_must_have_fixed_key_length() {
        let channel_entry = ChannelEntry::new(
            PublicKey::random(),
            PublicKey::random(),
            Balance::zero(BalanceType::HOPR),
            Hash::default(),
            U256::from(0u64),
            U256::from(0u64),
            ChannelStatus::Open,
            U256::from(0u64),
            U256::from(0u64),
        );

        let serialized = serialize_to_bytes(&channel_entry);

        assert!(serialized.is_ok());
        assert_eq!(serialized.unwrap().len(), ChannelEntry::SIZE)
    }
}
