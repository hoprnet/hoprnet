use async_trait::async_trait;
use core_crypto::types::{HalfKeyChallenge, Hash, OffchainPublicKey};
use core_types::{
    account::AccountEntry,
    acknowledgement::{AcknowledgedTicket, PendingAcknowledgement, UnacknowledgedTicket},
    channels::{generate_channel_id, ChannelEntry, ChannelStatus, Ticket},
};
use utils_db::db::Batch;
use utils_db::{
    constants::*,
    db::{serialize_to_bytes, DB},
    traits::AsyncKVStorage,
};
use utils_log::debug;
use utils_types::{
    primitives::{Address, AuthorizationToken, Balance, BalanceType, EthereumChallenge, Snapshot, U256},
    traits::BinarySerializable,
};

use crate::errors::Result;
use crate::traits::HoprCoreEthereumDbActions;

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
    pub me: Address,
}

impl<T: AsyncKVStorage<Key = Box<[u8]>, Value = Box<[u8]>>> CoreEthereumDb<T> {
    pub fn new(db: DB<T>, me: Address) -> Self {
        Self { db, me }
    }
}

#[async_trait(? Send)] // not placing the `Send` trait limitations on the trait
impl<T: AsyncKVStorage<Key = Box<[u8]>, Value = Box<[u8]>>> HoprCoreEthereumDbActions for CoreEthereumDb<T> {
    // core only part
    async fn get_current_ticket_index(&self, channel_id: &Hash) -> Result<Option<U256>> {
        let prefixed_key = utils_db::db::Key::new_with_prefix(channel_id, TICKET_INDEX_PREFIX)?;
        self.db.get_or_none::<U256>(prefixed_key).await
    }

    async fn set_current_ticket_index(&mut self, channel_id: &Hash, index: U256) -> Result<()> {
        let prefixed_key = utils_db::db::Key::new_with_prefix(channel_id, TICKET_INDEX_PREFIX)?;
        let _evicted = self.db.set(prefixed_key, &index).await?;
        // Ignoring evicted value
        Ok(())
    }

    async fn get_tickets(&self, maybe_signer: Option<Address>) -> Result<Vec<Ticket>> {
        let mut tickets = self
            .db
            .get_more::<AcknowledgedTicket>(
                Vec::from(ACKNOWLEDGED_TICKETS_PREFIX.as_bytes()).into_boxed_slice(),
                EthereumChallenge::SIZE as u32,
                &|v: &AcknowledgedTicket| maybe_signer.map(|s| v.signer.eq(&s)).unwrap_or(true),
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
                HalfKeyChallenge::SIZE as u32,
                &move |v: &PendingAcknowledgement| match v {
                    PendingAcknowledgement::WaitingAsSender => false,
                    PendingAcknowledgement::WaitingAsRelayer(unack) => match maybe_signer {
                        None => true,
                        Some(signer) => unack.signer.eq(&signer),
                    },
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

    async fn mark_rejected(&mut self, ticket: &Ticket) -> Result<()> {
        let count = self.get_rejected_tickets_count().await?;
        let count_key = utils_db::db::Key::new_from_str(REJECTED_TICKETS_COUNT)?;
        self.db.set(count_key, &(count + 1)).await?;

        let balance = self.get_rejected_tickets_value().await?;
        let value_key = utils_db::db::Key::new_from_str(REJECTED_TICKETS_VALUE)?;
        let _result = self.db.set(value_key, &balance.add(&ticket.amount)).await?;

        Ok(())
    }

    async fn check_and_set_packet_tag(&mut self, tag: &[u8]) -> Result<bool> {
        let key = utils_db::db::Key::new_bytes_with_prefix(tag, PACKET_TAG_PREFIX)?;

        let has_packet_tag = self.db.contains(key.clone()).await;
        if !has_packet_tag {
            let empty: [u8; 0] = [];
            self.db.set(key, &empty).await?;
        }

        //debug!("packet tag check: {}, set to: {}", has_packet_tag, hex::encode(tag));

        Ok(has_packet_tag)
    }

    async fn get_pending_acknowledgement(
        &self,
        half_key_challenge: &HalfKeyChallenge,
    ) -> Result<Option<PendingAcknowledgement>> {
        let key = utils_db::db::Key::new_with_prefix(half_key_challenge, PENDING_ACKNOWLEDGEMENTS_PREFIX)?;
        self.db.get_or_none::<PendingAcknowledgement>(key).await
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
        let ack_key =
            to_acknowledged_ticket_key(&ack_ticket.ticket.challenge, &ack_ticket.ticket.channel_epoch.into())?;

        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.del(unack_key);
        batch_ops.put(ack_key, ack_ticket);

        self.db.batch(batch_ops, true).await
    }

    // core and core-ethereum part
    async fn get_acknowledged_tickets(&self, filter: Option<ChannelEntry>) -> Result<Vec<AcknowledgedTicket>> {
        let mut tickets = self
            .db
            .get_more::<AcknowledgedTicket>(
                Vec::from(ACKNOWLEDGED_TICKETS_PREFIX.as_bytes()).into_boxed_slice(),
                EthereumChallenge::SIZE as u32,
                &|ack: &AcknowledgedTicket| match &filter {
                    Some(f) => {
                        f.destination.eq(&self.me)
                            && f.channel_epoch.eq(&ack.ticket.channel_epoch.into())
                            && f.source.eq(&ack.signer)
                    }
                    None => true,
                },
            )
            .await?;

        tickets.sort_by(|a, b| a.ticket.index.cmp(&b.ticket.index));

        Ok(tickets)
    }

    async fn get_unacknowledged_tickets(&self, filter: Option<ChannelEntry>) -> Result<Vec<UnacknowledgedTicket>> {
        Ok(self
            .db
            .get_more::<PendingAcknowledgement>(
                Vec::from(PENDING_ACKNOWLEDGEMENTS_PREFIX.as_bytes()).into_boxed_slice(),
                EthereumChallenge::SIZE as u32,
                &|pending: &PendingAcknowledgement| match pending {
                    PendingAcknowledgement::WaitingAsSender => false,
                    PendingAcknowledgement::WaitingAsRelayer(unack) => match &filter {
                        Some(f) => {
                            f.destination.eq(&self.me)
                                && f.channel_epoch.eq(&unack.ticket.channel_epoch.into())
                                && f.source.eq(&unack.signer)
                        }
                        None => true,
                    },
                },
            )
            .await?
            .into_iter()
            .filter_map(|a| match a {
                PendingAcknowledgement::WaitingAsSender => None,
                PendingAcknowledgement::WaitingAsRelayer(unack) => Some(unack),
            })
            .collect::<Vec<UnacknowledgedTicket>>())
    }

    async fn mark_pending(&mut self, counterparty: &Address, ticket: &Ticket) -> Result<()> {
        let prefixed_key = utils_db::db::Key::new_with_prefix(counterparty, PENDING_TICKETS_COUNT)?;
        let balance = self
            .db
            .get_or_none::<Balance>(prefixed_key.clone())
            .await?
            .unwrap_or(Balance::zero(ticket.amount.balance_type()));

        let _result = self.db.set(prefixed_key, &balance.add(&ticket.amount)).await?;
        Ok(())
    }

    async fn get_pending_balance_to(&self, counterparty: &Address) -> Result<Balance> {
        let key = utils_db::db::Key::new_with_prefix(counterparty, PENDING_TICKETS_COUNT)?;

        self.db
            .get_or_none::<Balance>(key)
            .await
            .map(|v| v.unwrap_or(Balance::zero(BalanceType::HOPR)))
    }

    async fn get_packet_key(&self, chain_key: &Address) -> Result<Option<OffchainPublicKey>> {
        let key = utils_db::db::Key::new_with_prefix(chain_key, CHAIN_KEY_PREFIX)?;
        self.db.get_or_none(key).await
    }

    async fn get_chain_key(&self, packet_key: &OffchainPublicKey) -> Result<Option<Address>> {
        let key = utils_db::db::Key::new_with_prefix(&Hash::create(&[&packet_key.to_bytes()]), PACKET_KEY_PREFIX)?;
        self.db.get_or_none(key).await
    }

    async fn link_chain_and_packet_keys(
        &mut self,
        chain_key: &Address,
        packet_key: &OffchainPublicKey,
        snapshot: &Snapshot,
    ) -> Result<()> {
        let mut batch = Batch::new();
        let ck_key = utils_db::db::Key::new_with_prefix(chain_key, CHAIN_KEY_PREFIX)?;
        let pk_key = utils_db::db::Key::new_with_prefix(&Hash::create(&[&packet_key.to_bytes()]), PACKET_KEY_PREFIX)?;

        batch.put(ck_key, packet_key);
        batch.put(pk_key, chain_key);
        batch.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            snapshot,
        );

        self.db.batch(batch, true).await
    }

    async fn get_channel_to(&self, dest: &Address) -> Result<Option<ChannelEntry>> {
        //utils_log::debug!("DB: get_channel_to dest: {}", dest);
        let key = utils_db::db::Key::new_with_prefix(&generate_channel_id(&self.me, dest), CHANNEL_PREFIX)?;

        self.db.get_or_none(key).await
    }

    async fn get_channel_from(&self, src: &Address) -> Result<Option<ChannelEntry>> {
        let key = utils_db::db::Key::new_with_prefix(&generate_channel_id(src, &self.me), CHANNEL_PREFIX)?;

        self.db.get_or_none(key).await
    }

    async fn update_channel_and_snapshot(
        &mut self,
        channel_id: &Hash,
        channel: &ChannelEntry,
        snapshot: &Snapshot,
    ) -> Result<()> {
        //utils_log::debug!("DB: update_channel_and_snapshot channel_id: {}", channel_id);
        let channel_key = utils_db::db::Key::new_with_prefix(channel_id, CHANNEL_PREFIX)?;
        let snapshot_key = utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?;

        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.put(channel_key, channel);
        batch_ops.put(snapshot_key, snapshot);

        self.db.batch(batch_ops, true).await
    }

    // core-ethereum only part
    async fn delete_acknowledged_tickets_from(&mut self, channel: ChannelEntry) -> Result<()> {
        let acknowledged_tickets = self.get_acknowledged_tickets(Some(channel)).await?;

        let key = utils_db::db::Key::new_from_str(NEGLECTED_TICKET_COUNT)?;
        let neglected_ticket_count = self.db.get_or_none::<usize>(key.clone()).await?.unwrap_or(0);

        let mut batch_ops = utils_db::db::Batch::new();
        for ticket in acknowledged_tickets.iter() {
            batch_ops.del(to_acknowledged_ticket_key(
                &ticket.ticket.challenge,
                &ticket.ticket.channel_epoch.into(),
            )?);
        }

        if !acknowledged_tickets.is_empty() {
            batch_ops.put(key, neglected_ticket_count + acknowledged_tickets.len())
        }

        self.db.batch(batch_ops, true).await
    }

    async fn get_latest_block_number(&self) -> Result<u32> {
        let key = utils_db::db::Key::new_from_str(LATEST_BLOCK_NUMBER_KEY)?;
        self.db.get_or_none::<u32>(key).await.map(|v| v.unwrap_or(0))
    }

    async fn update_latest_block_number(&mut self, number: u32) -> Result<()> {
        //utils_log::debug!("DB: update_latest_block_number to {}", number);
        let key = utils_db::db::Key::new_from_str(LATEST_BLOCK_NUMBER_KEY)?;
        let _ = self.db.set(key, &number).await?;
        Ok(())
    }

    async fn get_latest_confirmed_snapshot(&self) -> Result<Option<Snapshot>> {
        let key = utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?;
        self.db.get_or_none::<Snapshot>(key).await
    }

    async fn get_channel(&self, channel: &Hash) -> Result<Option<ChannelEntry>> {
        //utils_log::debug!("DB: get_channel {}", channel);
        let key = utils_db::db::Key::new_with_prefix(channel, CHANNEL_PREFIX)?;
        self.db.get_or_none::<ChannelEntry>(key).await
    }

    async fn get_channels(&self) -> Result<Vec<ChannelEntry>> {
        self.db
            .get_more::<ChannelEntry>(Box::from(CHANNEL_PREFIX.as_bytes()), Hash::SIZE as u32, &|_| true)
            .await
    }

    async fn get_channels_open(&self) -> Result<Vec<ChannelEntry>> {
        Ok(self
            .db
            .get_more::<ChannelEntry>(Box::from(CHANNEL_PREFIX.as_bytes()), Hash::SIZE as u32, &|_| true)
            .await?
            .into_iter()
            .filter(|x| x.status == ChannelStatus::Open)
            .collect())
    }

    async fn get_account(&self, address: &Address) -> Result<Option<AccountEntry>> {
        let key = utils_db::db::Key::new_with_prefix(address, ACCOUNT_PREFIX)?;
        self.db.get_or_none::<AccountEntry>(key).await
    }

    async fn update_account_and_snapshot(&mut self, account: &AccountEntry, snapshot: &Snapshot) -> Result<()> {
        let address_key = utils_db::db::Key::new_with_prefix(&account.chain_addr, ACCOUNT_PREFIX)?;
        let snapshot_key = utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?;

        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.put(address_key, account);
        batch_ops.put(snapshot_key, snapshot);

        self.db.batch(batch_ops, true).await
    }

    async fn get_accounts(&self) -> Result<Vec<AccountEntry>> {
        self.db
            .get_more::<AccountEntry>(Box::from(ACCOUNT_PREFIX.as_bytes()), Address::SIZE as u32, &|_| true)
            .await
    }

    async fn get_public_node_accounts(&self) -> Result<Vec<AccountEntry>> {
        self.db
            .get_more::<AccountEntry>(Box::from(ACCOUNT_PREFIX.as_bytes()), Address::SIZE as u32, &|x| {
                x.contains_routing_info()
            })
            .await
    }

    async fn get_redeemed_tickets_value(&self) -> Result<Balance> {
        let key = utils_db::db::Key::new_from_str(REDEEMED_TICKETS_VALUE)?;

        Ok(self
            .db
            .get_or_none::<Balance>(key)
            .await?
            .unwrap_or(Balance::zero(BalanceType::HOPR)))
    }

    async fn get_redeemed_tickets_count(&self) -> Result<usize> {
        let key = utils_db::db::Key::new_from_str(REDEEMED_TICKETS_COUNT)?;

        Ok(self.db.get_or_none::<usize>(key).await?.unwrap_or(0))
    }

    async fn get_neglected_tickets_count(&self) -> Result<usize> {
        let key = utils_db::db::Key::new_from_str(NEGLECTED_TICKET_COUNT)?;

        Ok(self.db.get_or_none::<usize>(key).await?.unwrap_or(0))
    }

    async fn get_pending_tickets_count(&self) -> Result<usize> {
        let key = utils_db::db::Key::new_from_str(PENDING_TICKETS_COUNT)?;

        Ok(self.db.get_or_none::<usize>(key).await?.unwrap_or(0))
    }

    async fn get_losing_tickets_count(&self) -> Result<usize> {
        let key = utils_db::db::Key::new_from_str(LOSING_TICKET_COUNT)?;

        Ok(self.db.get_or_none::<usize>(key).await?.unwrap_or(0))
    }

    async fn resolve_pending(&mut self, address: &Address, balance: &Balance, snapshot: &Snapshot) -> Result<()> {
        let key = utils_db::db::Key::new_with_prefix(address, PENDING_TICKETS_COUNT)?;
        let current_balance = self
            .db
            .get_or_none(key.clone())
            .await?
            .unwrap_or(Balance::zero(BalanceType::HOPR));

        let mut batch_ops = utils_db::db::Batch::new();
        // TODO: NOTE: was there a bug in the original implementation in TS? val.sub(val)?
        batch_ops.put(key.clone(), current_balance.sub(balance));
        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            snapshot,
        );

        self.db.batch(batch_ops, true).await
    }

    async fn mark_redeemed(&mut self, counterparty: &Address, ticket: &AcknowledgedTicket) -> Result<()> {
        debug!(
            "marking ticket #{} in channel with {} as redeemed",
            ticket.ticket.index, counterparty
        );

        let mut ops = utils_db::db::Batch::new();

        let key = utils_db::db::Key::new_from_str(REDEEMED_TICKETS_COUNT)?;
        let count = self.db.get_or_none::<usize>(key.clone()).await?.unwrap_or(0);
        ops.put(key, count + 1);

        let key = to_acknowledged_ticket_key(&ticket.ticket.challenge, &ticket.ticket.channel_epoch.into())?;
        ops.del(key);

        let key = utils_db::db::Key::new_from_str(REDEEMED_TICKETS_VALUE)?;
        let balance = self
            .db
            .get_or_none::<Balance>(key.clone())
            .await?
            .unwrap_or(Balance::zero(BalanceType::HOPR));

        let new_redeemed_balance = balance.add(&ticket.ticket.amount);
        ops.put(key, new_redeemed_balance);

        let key = utils_db::db::Key::new_with_prefix(counterparty, PENDING_TICKETS_COUNT)?;
        let pending_balance = self
            .db
            .get_or_none::<Balance>(key.clone())
            .await?
            .unwrap_or(Balance::zero(BalanceType::HOPR));

        let new_pending_balance = pending_balance.sub(&ticket.ticket.amount);
        ops.put(key, new_pending_balance);

        self.db.batch(ops, true).await
    }

    async fn mark_losing_acked_ticket(&mut self, counterparty: &Address, ticket: &AcknowledgedTicket) -> Result<()> {
        debug!(
            "marking ticket #{} in channel with {} as losing",
            ticket.ticket.index, counterparty
        );

        let mut ops = utils_db::db::Batch::new();

        let key = utils_db::db::Key::new_from_str(LOSING_TICKET_COUNT)?;
        let count = self.db.get_or_none::<usize>(key.clone()).await?.unwrap_or(0);
        ops.put(key, count + 1);

        let key = to_acknowledged_ticket_key(&ticket.ticket.challenge, &ticket.ticket.channel_epoch.into())?;
        ops.del(key);

        let key = utils_db::db::Key::new_with_prefix(counterparty, PENDING_TICKETS_COUNT)?;
        let balance = self
            .db
            .get_or_none::<Balance>(key.clone())
            .await?
            .unwrap_or(Balance::zero(BalanceType::HOPR));
        ops.put(key, balance.sub(&ticket.ticket.amount));

        self.db.batch(ops, true).await
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

        self.db.get_or_none::<usize>(key).await.map(|v| v.unwrap_or(0))
    }

    async fn get_channel_x(&self, src: &Address, dest: &Address) -> Result<Option<ChannelEntry>> {
        //utils_log::debug!("DB: get_channel_x src: {} & dest: {}", src, dest);
        let key = utils_db::db::Key::new_with_prefix(&generate_channel_id(src, dest), CHANNEL_PREFIX)?;
        self.db.get_or_none(key).await
    }

    async fn get_channels_from(&self, address: &Address) -> Result<Vec<ChannelEntry>> {
        Ok(self
            .db
            .get_more::<ChannelEntry>(Box::from(CHANNEL_PREFIX.as_bytes()), Hash::SIZE as u32, &|_| true)
            .await?
            .into_iter()
            .filter(move |x| x.source.eq(address))
            .collect())
    }

    async fn get_channels_to(&self, address: &Address) -> Result<Vec<ChannelEntry>> {
        Ok(self
            .db
            .get_more::<ChannelEntry>(Box::from(CHANNEL_PREFIX.as_bytes()), Hash::SIZE as u32, &|_| true)
            .await?
            .into_iter()
            .filter(move |x| x.destination.eq(address))
            .collect())
    }

    async fn get_hopr_balance(&self) -> Result<Balance> {
        //utils_log::debug!("DB: get_hopr_balance");
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

    async fn get_ticket_price(&self) -> Result<Option<U256>> {
        //utils_log::debug!("DB: get_ticket_price");
        let key = utils_db::db::Key::new_from_str(TICKET_PRICE_KEY)?;

        self.db.get_or_none::<U256>(key).await
    }

    async fn set_ticket_price(&mut self, ticket_price: &U256) -> Result<()> {
        let key = utils_db::db::Key::new_from_str(TICKET_PRICE_KEY)?;

        let _ = self
            .db
            .set::<U256>(key, ticket_price)
            .await
            .map(|v| v.unwrap_or(U256::zero()))?;

        Ok(())
    }

    async fn get_node_safe_registry_domain_separator(&self) -> Result<Option<Hash>> {
        let key = utils_db::db::Key::new_from_str(NODE_SAFE_REGISTRY_DOMAIN_SEPARATOR_KEY)?;
        self.db.get_or_none::<Hash>(key).await
    }

    async fn set_node_safe_registry_domain_separator(
        &mut self,
        node_safe_registry_domain_separator: &Hash,
        snapshot: &Snapshot,
    ) -> Result<()> {
        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.put(
            utils_db::db::Key::new_from_str(NODE_SAFE_REGISTRY_DOMAIN_SEPARATOR_KEY)?,
            node_safe_registry_domain_separator,
        );
        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            snapshot,
        );

        self.db.batch(batch_ops, true).await
    }

    async fn get_channels_domain_separator(&self) -> Result<Option<Hash>> {
        let key = utils_db::db::Key::new_from_str(CHANNELS_DOMAIN_SEPARATOR_KEY)?;
        self.db.get_or_none::<Hash>(key).await
    }

    async fn set_channels_domain_separator(
        &mut self,
        channels_domain_separator: &Hash,
        snapshot: &Snapshot,
    ) -> Result<()> {
        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.put(
            utils_db::db::Key::new_from_str(CHANNELS_DOMAIN_SEPARATOR_KEY)?,
            channels_domain_separator,
        );
        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            snapshot,
        );

        self.db.batch(batch_ops, true).await
    }

    async fn get_channels_ledger_domain_separator(&self) -> Result<Option<Hash>> {
        let key = utils_db::db::Key::new_from_str(CHANNELS_LEDGER_DOMAIN_SEPARATOR_KEY)?;
        self.db.get_or_none::<Hash>(key).await
    }

    async fn set_channels_ledger_domain_separator(
        &mut self,
        channels_ledger_domain_separator: &Hash,
        snapshot: &Snapshot,
    ) -> Result<()> {
        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.put(
            utils_db::db::Key::new_from_str(CHANNELS_LEDGER_DOMAIN_SEPARATOR_KEY)?,
            channels_ledger_domain_separator,
        );
        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            snapshot,
        );

        self.db.batch(batch_ops, true).await
    }

    async fn add_hopr_balance(&mut self, balance: &Balance, snapshot: &Snapshot) -> Result<()> {
        let key = utils_db::db::Key::new_from_str(HOPR_BALANCE_KEY)?;

        let current_balance = self
            .db
            .get_or_none::<Balance>(key.clone())
            .await?
            .unwrap_or(Balance::zero(BalanceType::HOPR));

        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.put(key, current_balance.add(balance));
        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            snapshot,
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
        batch_ops.put(key, current_balance.sub(balance));
        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            snapshot,
        );

        self.db.batch(batch_ops, true).await
    }

    /// Get the staking safe address
    async fn get_staking_safe_address(&self) -> Result<Option<Address>> {
        let key = utils_db::db::Key::new_from_str(STAKING_SAFE_ADDRESS_KEY)?;
        self.db.get_or_none::<Address>(key).await
    }

    /// Sets the staking safe address
    ///
    /// - `safe_address`: safe address that holds tokens for the node
    async fn set_staking_safe_address(&mut self, safe_address: &Address) -> Result<()> {
        let safe_address_key = utils_db::db::Key::new_from_str(STAKING_SAFE_ADDRESS_KEY)?;

        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.put(safe_address_key, safe_address);

        self.db.batch(batch_ops, true).await
    }

    /// Get the staking module address
    async fn get_staking_module_address(&self) -> Result<Option<Address>> {
        let key = utils_db::db::Key::new_from_str(STAKING_MODULE_ADDRESS_KEY)?;
        self.db.get_or_none::<Address>(key).await
    }

    /// Sets the staking module address
    ///
    /// - `module_address`: module address that stores permissions
    async fn set_staking_module_address(&mut self, module_address: &Address) -> Result<()> {
        let module_address_key = utils_db::db::Key::new_from_str(STAKING_MODULE_ADDRESS_KEY)?;

        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.put(module_address_key, module_address);

        self.db.batch(batch_ops, true).await
    }

    /// Get the allowance for HoprChannels contract to transfer tokens on behalf of staking safe address
    async fn get_staking_safe_allowance(&self) -> Result<Balance> {
        let key = utils_db::db::Key::new_from_str(STAKING_SAFE_ALLOWANCE_KEY)?;

        self.db
            .get_or_none::<Balance>(key)
            .await
            .map(|v| v.unwrap_or(Balance::zero(BalanceType::HOPR)))
    }

    /// Sets the allowance for HoprChannels contract to transfer tokens on behalf of staking safe address
    async fn set_staking_safe_allowance(&mut self, allowance: &Balance, snapshot: &Snapshot) -> Result<()> {
        let key = utils_db::db::Key::new_from_str(STAKING_SAFE_ALLOWANCE_KEY)?;

        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.put(key, allowance);
        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            snapshot,
        );

        self.db.batch(batch_ops, true).await
    }

    /// Checks whether network registry is enabled. Default: true
    async fn is_network_registry_enabled(&self) -> Result<bool> {
        let key = utils_db::db::Key::new_from_str(NETWORK_REGISTRY_ENABLED_PREFIX)?;
        Ok(self.db.get_or_none::<bool>(key.clone()).await?.unwrap_or(true))
    }

    async fn set_network_registry(&mut self, enabled: bool, snapshot: &Snapshot) -> Result<()> {
        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.put(
            utils_db::db::Key::new_from_str(NETWORK_REGISTRY_ENABLED_PREFIX)?,
            enabled,
        );
        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            snapshot,
        );

        self.db.batch(batch_ops, true).await
    }

    async fn is_allowed_to_access_network(&self, node: &Address) -> Result<bool> {
        let key = utils_db::db::Key::new_with_prefix(node, NETWORK_REGISTRY_ALLOWED_PREFIX)?;

        Ok(self.db.contains(key).await)
    }

    async fn set_allowed_to_access_network(
        &mut self,
        node: &Address,
        allowed: bool,
        snapshot: &Snapshot,
    ) -> Result<()> {
        let key = utils_db::db::Key::new_with_prefix(node, NETWORK_REGISTRY_ALLOWED_PREFIX)?;

        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            &snapshot,
        );

        if allowed {
            batch_ops.put(key, ());
        } else {
            batch_ops.del(key)
        }

        self.db.batch(batch_ops, true).await
    }

    async fn get_from_network_registry(&self, stake_account: &Address) -> Result<Vec<Address>> {
        let key = utils_db::db::Key::new_with_prefix(stake_account, NETWORK_REGISTRY_ADDRESS_CHAIN_KEY_PREFIX)?;

        Ok(self.db.get_or_none::<Vec<Address>>(key).await?.unwrap_or(vec![]))
    }

    /// Removes a node from the network registry
    ///
    /// - `address`: the address to remove
    /// - `stake_account`: the stake account from which the address should be removed
    /// - `snapshot`: latest chain snapshot
    async fn add_to_network_registry(
        &mut self,
        stake_account: &Address,
        node_address: &Address,
        snapshot: &Snapshot,
    ) -> Result<()> {
        let mut registered_nodes = self.get_from_network_registry(stake_account).await?;

        let already_included = registered_nodes.contains(node_address);

        if !already_included {
            registered_nodes.push(node_address.to_owned());
        }

        let mut batch_ops = utils_db::db::Batch::new();

        let eligible = self.is_eligible(stake_account).await?;

        if eligible {
            batch_ops.put(
                utils_db::db::Key::new_with_prefix(node_address, NETWORK_REGISTRY_ALLOWED_PREFIX)?,
                (),
            )
        }

        batch_ops.put(
            utils_db::db::Key::new_with_prefix(stake_account, NETWORK_REGISTRY_ADDRESS_CHAIN_KEY_PREFIX)?,
            &registered_nodes,
        );
        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            snapshot,
        );

        self.db.batch(batch_ops, true).await
    }

    async fn remove_from_network_registry(
        &mut self,
        stake_account: &Address,
        node_address: &Address,
        snapshot: &Snapshot,
    ) -> Result<()> {
        let registered_nodes: Vec<Address> = self
            .get_from_network_registry(stake_account)
            .await?
            .into_iter()
            .filter(|registered_node| registered_node.ne(&node_address))
            .collect();

        let mut batch_ops = utils_db::db::Batch::new();

        batch_ops.del(utils_db::db::Key::new_with_prefix(
            node_address,
            NETWORK_REGISTRY_ALLOWED_PREFIX,
        )?);
        batch_ops.put(
            utils_db::db::Key::new_with_prefix(stake_account, NETWORK_REGISTRY_ADDRESS_CHAIN_KEY_PREFIX)?,
            &registered_nodes,
        );
        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            snapshot,
        );

        self.db.batch(batch_ops, true).await
    }

    /// Checks if an stake account is eligible to register nodes
    ///
    /// - `stake_account`: the account to check
    async fn is_eligible(&self, stake_account: &Address) -> Result<bool> {
        let key = utils_db::db::Key::new_with_prefix(stake_account, NETWORK_REGISTRY_ADDRESS_ELIGIBLE_PREFIX)?;

        Ok(self.db.contains(key).await)
    }

    /// Sets the eligibility status of an account
    ///
    /// - `stake_account`: the account whose eligibility to be set
    /// - `eligible`: true if `stake_account` is now eligible
    /// - `snapshot`: latest chain snapshot
    async fn set_eligible(
        &mut self,
        stake_account: &Address,
        eligible: bool,
        snapshot: &Snapshot,
    ) -> Result<Vec<Address>> {
        let key = utils_db::db::Key::new_with_prefix(stake_account, NETWORK_REGISTRY_ADDRESS_ELIGIBLE_PREFIX)?;

        let mut batch_ops = utils_db::db::Batch::new();

        let registered_nodes = self.get_from_network_registry(stake_account).await?;

        if eligible {
            for registered_node in registered_nodes.iter() {
                batch_ops.put(
                    utils_db::db::Key::new_with_prefix(registered_node, NETWORK_REGISTRY_ALLOWED_PREFIX)?,
                    (),
                )
            }

            batch_ops.put(key, ());
        } else {
            for registered_node in registered_nodes.iter() {
                batch_ops.del(utils_db::db::Key::new_with_prefix(
                    registered_node,
                    NETWORK_REGISTRY_ALLOWED_PREFIX,
                )?)
            }

            batch_ops.del(key);
        }

        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            snapshot,
        );
        self.db.batch(batch_ops, true).await?;

        Ok(registered_nodes)
    }

    async fn store_authorization(&mut self, token: AuthorizationToken) -> Result<()> {
        let tid = Hash::create(&[token.id().as_bytes()]);
        let key = utils_db::db::Key::new_with_prefix(&tid, API_AUTHORIZATION_TOKEN_KEY_PREFIX)?;
        let _ = self.db.set(key, &token).await?;
        Ok(())
    }

    async fn is_mfa_protected(&self) -> Result<Option<Address>> {
        let key = utils_db::db::Key::new_from_str(MFA_MODULE_PREFIX)?;
        self.db.get_or_none::<Address>(key).await
    }

    async fn set_mfa_protected_and_update_snapshot(
        &mut self,
        maybe_mfa_address: Option<Address>,
        snapshot: &Snapshot,
    ) -> Result<()> {
        let mfa_key = utils_db::db::Key::new_from_str(MFA_MODULE_PREFIX)?;
        let snapshot_key = utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?;

        match maybe_mfa_address {
            Some(mfa_address) => {
                let mut batch_ops = utils_db::db::Batch::new();
                batch_ops.put(mfa_key, mfa_address);
                batch_ops.put(snapshot_key, snapshot);

                self.db.batch(batch_ops, true).await
            }
            None => {
                let mut batch_ops = utils_db::db::Batch::new();
                batch_ops.del(mfa_key);
                batch_ops.put(snapshot_key, snapshot);

                self.db.batch(batch_ops, true).await
            }
        }
    }

    async fn retrieve_authorization(&self, id: String) -> Result<Option<AuthorizationToken>> {
        let tid = Hash::create(&[id.as_bytes()]);
        let key = utils_db::db::Key::new_with_prefix(&tid, API_AUTHORIZATION_TOKEN_KEY_PREFIX)?;
        self.db.get_or_none::<AuthorizationToken>(key).await
    }

    async fn delete_authorization(&mut self, id: String) -> Result<()> {
        let tid = Hash::create(&[id.as_bytes()]);
        let key = utils_db::db::Key::new_with_prefix(&tid, API_AUTHORIZATION_TOKEN_KEY_PREFIX)?;
        let _ = self.db.remove::<AuthorizationToken>(key).await?;
        Ok(())
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use super::{CoreEthereumDb, HoprCoreEthereumDbActions, DB};
    use async_lock::RwLock;
    use core_crypto::types::{Hash, OffchainPublicKey};
    use core_types::{
        account::AccountEntry,
        acknowledgement::wasm::AcknowledgedTicket,
        channels::{wasm::Ticket, ChannelEntry},
    };
    use js_sys::Uint8Array;
    use std::sync::Arc;
    use utils_db::leveldb;
    use utils_types::primitives::{Address, AuthorizationToken, Balance, Snapshot, U256};
    use wasm_bindgen::prelude::*;

    macro_rules! to_iterable {
        ($obj:ident,$x:ty) => {
            #[wasm_bindgen]
            pub struct $obj {
                v: Vec<$x>,
            }

            impl $obj {
                pub fn from(value: Vec<$x>) -> Self {
                    Self { v: value }
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

                #[wasm_bindgen]
                pub fn at(&self, index: usize) -> Option<$x> {
                    if index < self.v.len() {
                        Some(self.v[index].clone())
                    } else {
                        None
                    }
                }

                #[wasm_bindgen]
                pub fn len(&self) -> usize {
                    self.v.len()
                }
            }
        };
    }

    to_iterable!(WasmVecAcknowledgedTicket, AcknowledgedTicket);
    to_iterable!(WasmVecChannelEntry, ChannelEntry);
    to_iterable!(WasmVecAccountEntry, AccountEntry);
    to_iterable!(WasmVecAddress, Address);

    #[derive(Clone)]
    #[wasm_bindgen]
    pub struct Database {
        core_ethereum_db: Arc<RwLock<CoreEthereumDb<leveldb::wasm::LevelDbShim>>>,
    }

    impl Database {
        pub fn as_ref_counted(&self) -> Arc<RwLock<CoreEthereumDb<leveldb::wasm::LevelDbShim>>> {
            self.core_ethereum_db.clone()
        }
    }

    #[wasm_bindgen]
    impl Database {
        #[wasm_bindgen(constructor)]
        pub fn new(db: leveldb::wasm::LevelDb, me_addr: Address) -> Self {
            Self {
                core_ethereum_db: Arc::new(RwLock::new(CoreEthereumDb::new(
                    DB::new(leveldb::wasm::LevelDbShim::new(db)),
                    me_addr,
                ))),
            }
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }
    }

    /*
    -- Un-comment these when hunting for deadlocks --

    macro_rules! check_lock_read {
        { $($rest:tt)* } => {
            let r = {
                utils_misc::console_log!("{} >>> READ ", stdext::function_name!());
                $($rest)*
            };
            utils_misc::console_log!("{} <<< READ ", stdext::function_name!());
            r
        };
    }

    macro_rules! check_lock_write {
        { $($rest:tt)* } => {
            let r = {
                utils_misc::console_log!("{} >>> WRITE ", stdext::function_name!());
                $($rest)*
            };
            utils_misc::console_log!("{} <<< WRITE ", stdext::function_name!());
            r
        };
    }*/

    #[wasm_bindgen]
    impl Database {
        #[wasm_bindgen]
        pub async fn get_acknowledged_tickets(
            &self,
            filter: Option<ChannelEntry>,
        ) -> Result<WasmVecAcknowledgedTicket, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db
                .get_acknowledged_tickets(filter)
                .await
                .map(|x| { x.into_iter().map(AcknowledgedTicket::from).collect::<Vec<_>>() })
                .map(WasmVecAcknowledgedTicket::from))
            //}
        }

        #[wasm_bindgen]
        pub async fn delete_acknowledged_tickets_from(&self, source: ChannelEntry) -> Result<(), JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_write! {
            let mut db = data.write().await;
            utils_misc::ok_or_jserr!(db.delete_acknowledged_tickets_from(source).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_latest_block_number(&self) -> Result<u32, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_latest_block_number().await)
            //}
        }

        #[wasm_bindgen]
        pub async fn update_latest_block_number(&self, number: u32) -> Result<(), JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_write! {
            let mut db = data.write().await;
            utils_misc::ok_or_jserr!(db.update_latest_block_number(number).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_latest_confirmed_snapshot(&self) -> Result<Option<Snapshot>, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_latest_confirmed_snapshot().await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_channel(&self, channel: &Hash) -> Result<Option<ChannelEntry>, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_channel(channel).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_channels(&self) -> Result<WasmVecChannelEntry, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_channels().await).map(WasmVecChannelEntry::from)
            //}
        }

        pub async fn get_channels_open(&self) -> Result<WasmVecChannelEntry, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_channels_open().await).map(WasmVecChannelEntry::from)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_account(&self, address: &Address) -> Result<Option<AccountEntry>, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_account(address).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_accounts(&self) -> Result<WasmVecAccountEntry, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_accounts().await).map(WasmVecAccountEntry::from)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_public_node_accounts(&self) -> Result<WasmVecAccountEntry, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_public_node_accounts().await).map(WasmVecAccountEntry::from)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_redeemed_tickets_value(&self) -> Result<Balance, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_redeemed_tickets_value().await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_redeemed_tickets_count(&self) -> Result<usize, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_redeemed_tickets_count().await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_neglected_tickets_count(&self) -> Result<usize, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_neglected_tickets_count().await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_pending_tickets_count(&self) -> Result<usize, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_pending_tickets_count().await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_losing_tickets_count(&self) -> Result<usize, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_losing_tickets_count().await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_pending_balance_to(&self, counterparty: &Address) -> Result<Balance, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_pending_balance_to(counterparty).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_packet_key(&self, chain_key: &Address) -> Result<Option<OffchainPublicKey>, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_packet_key(chain_key).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_chain_key(&self, packet_key: &OffchainPublicKey) -> Result<Option<Address>, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_chain_key(packet_key).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn link_chain_and_packet_keys(
            &self,
            chain_key: &Address,
            packet_key: &OffchainPublicKey,
            snapshot: &Snapshot,
        ) -> Result<(), JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_write! {
            let mut db = data.write().await;
            utils_misc::ok_or_jserr!(db.link_chain_and_packet_keys(chain_key, packet_key, snapshot).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn mark_pending(&self, counterparty: &Address, ticket: &Ticket) -> Result<(), JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_write! {
            let mut db = data.write().await;
            utils_misc::ok_or_jserr!(db.mark_pending(counterparty, &ticket.into()).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn resolve_pending(
            &self,
            address: &Address,
            balance: &Balance,
            snapshot: &Snapshot,
        ) -> Result<(), JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_write! {
            let mut db = data.write().await;
            utils_misc::ok_or_jserr!(db.resolve_pending(address, balance, snapshot).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn mark_redeemed(
            &self,
            counterparty: &Address,
            acked_ticket: &AcknowledgedTicket,
        ) -> Result<(), JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_write! {
            let mut db = data.write().await;
            utils_misc::ok_or_jserr!(db.mark_redeemed(counterparty, &acked_ticket.into()).await)
            //}
        }

        /// NOTE: needed only for testing
        #[wasm_bindgen]
        pub async fn mark_rejected(&self, ticket: &Ticket) -> Result<(), JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_write! {
            let mut db = data.write().await;
            utils_misc::ok_or_jserr!(db.mark_rejected(&ticket.into()).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn mark_losing_acked_ticket(
            &self,
            counterparty: &Address,
            ticket: &AcknowledgedTicket,
        ) -> Result<(), JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_write! {
            let mut db = data.write().await;
            utils_misc::ok_or_jserr!(db.mark_losing_acked_ticket(counterparty, &ticket.into()).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_rejected_tickets_value(&self) -> Result<Balance, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_rejected_tickets_value().await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_rejected_tickets_count(&self) -> Result<usize, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_rejected_tickets_count().await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_channel_x(&self, src: &Address, dest: &Address) -> Result<Option<ChannelEntry>, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_channel_x(src, dest).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_channel_to(&self, dest: &Address) -> Result<Option<ChannelEntry>, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_channel_to(dest).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_channel_from(&self, src: &Address) -> Result<Option<ChannelEntry>, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_channel_from(src).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_channels_from(&self, address: &Address) -> Result<WasmVecChannelEntry, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_channels_from(address).await).map(WasmVecChannelEntry::from)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_channels_to(&self, address: &Address) -> Result<WasmVecChannelEntry, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_channels_to(address).await).map(WasmVecChannelEntry::from)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_hopr_balance(&self) -> Result<Balance, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_hopr_balance().await)
            //}
        }

        #[wasm_bindgen]
        pub async fn set_hopr_balance(&self, balance: &Balance) -> Result<(), JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_write! {
            let mut db = data.write().await;
            utils_misc::ok_or_jserr!(db.set_hopr_balance(balance).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_ticket_price(&self) -> Result<Option<U256>, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_ticket_price().await)
            //}
        }

        #[wasm_bindgen]
        pub async fn set_ticket_price(&self, ticket_price: &U256) -> Result<(), JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_write! {
            let mut db = data.write().await;
            utils_misc::ok_or_jserr!(db.set_ticket_price(ticket_price).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_staking_module_address(&self) -> Result<Option<Address>, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_staking_module_address().await)
            //}
        }
        #[wasm_bindgen]
        pub async fn set_staking_module_address(&self, module_address: &Address) -> Result<(), JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_write! {
            let mut db = data.write().await;
            utils_misc::ok_or_jserr!(db.set_staking_module_address(module_address).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_staking_safe_address(&self) -> Result<Option<Address>, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_staking_safe_address().await)
            //}
        }
        #[wasm_bindgen]
        pub async fn set_staking_safe_address(&self, safe_address: &Address) -> Result<(), JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_write! {
            let mut db = data.write().await;
            utils_misc::ok_or_jserr!(db.set_staking_safe_address(safe_address).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn get_staking_safe_allowance(&self) -> Result<Balance, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.get_staking_safe_allowance().await)
            //}
        }

        #[wasm_bindgen]
        pub async fn set_staking_safe_allowance(&self, balance: &Balance, snapshot: &Snapshot) -> Result<(), JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_write! {
            let mut db = data.write().await;
            utils_misc::ok_or_jserr!(db.set_staking_safe_allowance(balance, snapshot).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn check_and_set_packet_tag(&self, tag: &Uint8Array) -> Result<bool, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_write! {
            let mut db = data.write().await;
            utils_misc::ok_or_jserr!(db.check_and_set_packet_tag(&tag.to_vec()).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn store_authorization(&self, token: AuthorizationToken) -> Result<(), JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_write! {
            let mut db = data.write().await;
            utils_misc::ok_or_jserr!(db.store_authorization(token).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn retrieve_authorization(&self, id: String) -> Result<Option<AuthorizationToken>, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.retrieve_authorization(id).await)
            //}
        }

        #[wasm_bindgen]
        pub async fn delete_authorization(&self, id: String) -> Result<(), JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_write! {
            let mut db = data.write().await;
            utils_misc::ok_or_jserr!(db.delete_authorization(id).await)
            //}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_types::channels::ChannelEntry;
    use std::{
        str::FromStr,
        sync::{Arc, Mutex},
    };
    use utils_db::db::serialize_to_bytes;
    use utils_db::leveldb::rusty::RustyLevelDbShim;
    use utils_types::primitives::{Address, EthereumChallenge};
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
            Address::random(),
            Address::random(),
            Balance::zero(BalanceType::HOPR),
            U256::from(0u64),
            ChannelStatus::Open,
            U256::from(1u64),
            U256::from(0u64),
        );

        let serialized = serialize_to_bytes(&channel_entry);

        assert!(serialized.is_ok());
        assert_eq!(serialized.unwrap().len(), ChannelEntry::SIZE)
    }

    #[async_std::test]
    async fn test_set_ticket_price() {
        let level_db = Arc::new(Mutex::new(
            rusty_leveldb::DB::open("test", rusty_leveldb::in_memory()).unwrap(),
        ));

        let mut db = CoreEthereumDb::new(DB::new(RustyLevelDbShim::new(level_db)), Address::random());

        assert_eq!(db.get_ticket_price().await, Ok(None));

        assert!(db.set_ticket_price(&U256::from(100u64)).await.is_ok());

        assert_eq!(db.get_ticket_price().await, Ok(Some(U256::from(100u64))));
    }

    #[async_std::test]
    async fn test_set_network_registry() {
        let level_db = Arc::new(Mutex::new(
            rusty_leveldb::DB::open("test", rusty_leveldb::in_memory()).unwrap(),
        ));

        let mut db = CoreEthereumDb::new(DB::new(RustyLevelDbShim::new(level_db)), Address::random());

        assert_eq!(db.is_network_registry_enabled().await, Ok(true));

        assert!(db.set_network_registry(false, &Snapshot::default()).await.is_ok());

        assert_eq!(db.is_network_registry_enabled().await, Ok(false));
    }

    #[async_std::test]
    async fn test_allowed_to_access_network() {
        let level_db = Arc::new(Mutex::new(
            rusty_leveldb::DB::open("test", rusty_leveldb::in_memory()).unwrap(),
        ));

        let mut db = CoreEthereumDb::new(DB::new(RustyLevelDbShim::new(level_db)), Address::random());

        let test_address = Address::from_str("0xa6416794a09d1c8c4c6110f83f42cf6f1ed9c416").unwrap();

        assert_eq!(db.is_allowed_to_access_network(&test_address).await.unwrap(), false);

        db.set_allowed_to_access_network(&test_address, true, &Snapshot::default())
            .await
            .unwrap();

        assert_eq!(db.is_allowed_to_access_network(&test_address).await.unwrap(), true);
    }

    #[async_std::test]
    async fn test_add_to_network_registry() {
        let level_db = Arc::new(Mutex::new(
            rusty_leveldb::DB::open("test", rusty_leveldb::in_memory()).unwrap(),
        ));

        let mut db = CoreEthereumDb::new(DB::new(RustyLevelDbShim::new(level_db)), Address::random());

        let test_address = Address::from_str("0xa6416794a09d1c8c4c6110f83f42cf6f1ed9c416").unwrap();
        let test_stake_account = Address::from_str("0xf2a867525fc8a16055d0dea371f0360288795c61").unwrap();

        db.add_to_network_registry(&test_stake_account, &test_address, &Snapshot::default())
            .await
            .unwrap();

        assert_eq!(
            db.get_from_network_registry(&test_stake_account).await.unwrap(),
            vec![test_address]
        );

        assert!(!db.is_allowed_to_access_network(&test_address).await.unwrap());

        db.set_eligible(&test_stake_account, true, &Snapshot::default())
            .await
            .unwrap();

        db.add_to_network_registry(&test_stake_account, &test_address, &Snapshot::default())
            .await
            .unwrap();

        assert_eq!(
            db.get_from_network_registry(&test_stake_account).await.unwrap(),
            vec![test_address]
        );

        assert!(db.is_allowed_to_access_network(&test_address).await.unwrap());
    }

    #[async_std::test]
    async fn test_remove_from_network_registry() {
        let level_db = Arc::new(Mutex::new(
            rusty_leveldb::DB::open("test", rusty_leveldb::in_memory()).unwrap(),
        ));

        let mut db = CoreEthereumDb::new(DB::new(RustyLevelDbShim::new(level_db)), Address::random());

        let test_address = Address::from_str("0xa6416794a09d1c8c4c6110f83f42cf6f1ed9c416").unwrap();
        let test_stake_account = Address::from_str("0xf2a867525fc8a16055d0dea371f0360288795c61").unwrap();

        db.add_to_network_registry(&test_stake_account, &test_address, &Snapshot::default())
            .await
            .unwrap();

        assert_eq!(
            db.get_from_network_registry(&test_stake_account).await.unwrap(),
            vec![test_address]
        );

        db.remove_from_network_registry(&test_stake_account, &test_address, &Snapshot::default())
            .await
            .unwrap();

        assert_eq!(db.get_from_network_registry(&test_stake_account).await.unwrap(), vec![]);

        assert!(!db.is_allowed_to_access_network(&test_address).await.unwrap());

        db.add_to_network_registry(&test_stake_account, &test_address, &Snapshot::default())
            .await
            .unwrap();

        db.set_eligible(&test_stake_account, true, &Snapshot::default())
            .await
            .unwrap();

        db.remove_from_network_registry(&test_stake_account, &test_address, &Snapshot::default())
            .await
            .unwrap();

        assert_eq!(db.get_from_network_registry(&test_stake_account).await.unwrap(), vec![]);

        assert!(!db.is_allowed_to_access_network(&test_address).await.unwrap());
    }

    #[async_std::test]
    async fn test_token_storage() {
        let level_db = Arc::new(Mutex::new(
            rusty_leveldb::DB::open("test", rusty_leveldb::in_memory()).unwrap(),
        ));
        let mut db = CoreEthereumDb::new(DB::new(RustyLevelDbShim::new(level_db)), Address::random());

        let token_id = "test";

        let token = AuthorizationToken::new(token_id.to_string(), &[0xffu8; 100]);
        db.store_authorization(token.clone()).await.unwrap();

        let token_2 = db
            .retrieve_authorization(token_id.to_string())
            .await
            .unwrap()
            .expect("db should contain a token");
        assert_eq!(token, token_2, "retrieved token should be equal to the stored one");

        db.delete_authorization(token_id.to_string())
            .await
            .expect("db should remove token");

        let nonexistent = db.retrieve_authorization(token_id.to_string()).await.unwrap();
        assert!(nonexistent.is_none(), "token should be removed from the db");
    }

    #[async_std::test]
    async fn test_set_mfa() {
        let level_db = Arc::new(Mutex::new(
            rusty_leveldb::DB::open("test", rusty_leveldb::in_memory()).unwrap(),
        ));
        let mut db = CoreEthereumDb::new(DB::new(RustyLevelDbShim::new(level_db)), Address::random());

        let test_address = Address::from_str("0xa6416794a09d1c8c4c6110f83f42cf6f1ed9c416").unwrap();

        db.set_mfa_protected_and_update_snapshot(Some(test_address), &Snapshot::default())
            .await
            .unwrap();

        assert_eq!(db.is_mfa_protected().await.unwrap(), Some(test_address));
    }
}
