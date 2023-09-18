use async_trait::async_trait;
use core_crypto::types::{HalfKeyChallenge, Hash, OffchainPublicKey};
use core_types::{
    account::AccountEntry,
    acknowledgement::{AcknowledgedTicket, AcknowledgedTicketStatus, PendingAcknowledgement, UnacknowledgedTicket},
    channels::{generate_channel_id, ChannelEntry, ChannelStatus, Ticket},
};
use utils_db::errors::DbError;
use utils_db::{
    constants::*,
    db::{Batch, DB},
    traits::AsyncKVStorage,
};
use utils_log::{debug, error};
use utils_types::{
    primitives::{Address, AuthorizationToken, Balance, BalanceType, EthereumChallenge, Snapshot, U256},
    traits::BinarySerializable,
};

use crate::{
    errors::Result,
    traits::{HoprCoreEthereumDbActions, HoprCoreEthereumTestActions},
};

const ACKNOWLEDGED_TICKETS_KEY_LENGTH: usize = Hash::SIZE + (u32::BITS / 8) as usize + (u64::BITS / 8) as usize;

fn to_acknowledged_ticket_key(channel_id: &Hash, epoch: u32, index: u64) -> Result<utils_db::db::Key> {
    let mut ack_key = Vec::with_capacity(ACKNOWLEDGED_TICKETS_KEY_LENGTH);

    ack_key.extend_from_slice(&channel_id.to_bytes());
    ack_key.extend_from_slice(&epoch.to_be_bytes());
    ack_key.extend_from_slice(&index.to_be_bytes());

    utils_db::db::Key::new_bytes_with_prefix(&ack_key, ACKNOWLEDGED_TICKETS_PREFIX)
}

#[inline]
fn get_acknowledged_ticket_key(ack: &AcknowledgedTicket) -> Result<utils_db::db::Key> {
    to_acknowledged_ticket_key(&ack.ticket.channel_id, ack.ticket.channel_epoch, ack.ticket.index)
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
        let mut acked_tickets = self
            .db
            .get_more::<AcknowledgedTicket>(
                Vec::from(ACKNOWLEDGED_TICKETS_PREFIX.as_bytes()).into_boxed_slice(),
                ACKNOWLEDGED_TICKETS_KEY_LENGTH as u32,
                &|v: &AcknowledgedTicket| maybe_signer.map(|s| v.signer.eq(&s)).unwrap_or(true),
            )
            .await?
            .into_iter()
            .map(|a| a.ticket)
            .collect::<Vec<Ticket>>();
        acked_tickets.sort_by(|l, r| l.index.cmp(&r.index));

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

        acked_tickets.append(&mut unack_tickets);

        Ok(acked_tickets)
    }

    async fn mark_rejected(&mut self, ticket: &Ticket) -> Result<()> {
        let count = self.get_rejected_tickets_count().await?;
        let count_key = utils_db::db::Key::new_from_str(REJECTED_TICKETS_COUNT)?;

        let balance = self.get_rejected_tickets_value().await?;
        let value_key = utils_db::db::Key::new_from_str(REJECTED_TICKETS_VALUE)?;

        let mut batch_ops = utils_db::db::Batch::default();
        batch_ops.put(count_key, &(count + 1));
        batch_ops.put(value_key, &balance.add(&ticket.amount));

        self.db.batch(batch_ops, true).await
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
        acked_ticket: AcknowledgedTicket,
    ) -> Result<()> {
        let unack_key = utils_db::db::Key::new_with_prefix(half_key_challenge, PENDING_ACKNOWLEDGEMENTS_PREFIX)?;
        let ack_key = get_acknowledged_ticket_key(&acked_ticket)?;

        let mut batch_ops = utils_db::db::Batch::default();
        batch_ops.del(unack_key);
        batch_ops.put(ack_key, acked_ticket);

        self.db.batch(batch_ops, true).await
    }

    // core and core-ethereum part
    async fn get_acknowledged_tickets(&self, filter: Option<ChannelEntry>) -> Result<Vec<AcknowledgedTicket>> {
        let mut tickets = self
            .db
            .get_more::<AcknowledgedTicket>(
                Vec::from(ACKNOWLEDGED_TICKETS_PREFIX.as_bytes()).into_boxed_slice(),
                ACKNOWLEDGED_TICKETS_KEY_LENGTH as u32,
                &|ack: &AcknowledgedTicket| filter.map(|f| f.get_id() == ack.ticket.channel_id).unwrap_or(true),
            )
            .await?;

        tickets.sort();

        Ok(tickets)
    }

    /// Fetches all acknowledged tickets in given range.
    /// If any of the tickets is marked as being redeemed, drop this acknowledged ticket
    /// and all previous tickets as aggregation does not make any sense.
    async fn prepare_aggregatable_tickets(
        &mut self,
        channel_id: &Hash,
        epoch: u32,
        index_start: u64,
        index_end: u64,
    ) -> Result<Vec<AcknowledgedTicket>> {
        let mut tickets = self
            .db
            .get_more_range::<AcknowledgedTicket>(
                to_acknowledged_ticket_key(channel_id, epoch, index_start)?.into(),
                to_acknowledged_ticket_key(channel_id, epoch, index_end)?.into(),
                &|_| true,
            )
            .await?;

        tickets.sort();

        let mut batch_ops = utils_db::db::Batch::default();

        let mut done = false;
        while tickets.len() > 0 && !done {
            let tickets_len = tickets.len();
            for (index, ticket) in tickets.iter_mut().enumerate() {
                if let AcknowledgedTicketStatus::BeingRedeemed { tx_hash: _ } = ticket.status {
                    if index + 1 > tickets_len {
                        tickets = vec![];
                        done = true;
                    } else {
                        tickets = tickets.split_at_mut(index + 1).1.to_vec();
                    }
                    batch_ops = utils_db::db::Batch::default();
                    break;
                }
                ticket.status(AcknowledgedTicketStatus::BeingAggregated {
                    start: index_start,
                    end: index_end,
                });
                batch_ops.put(
                    to_acknowledged_ticket_key(
                        &ticket.ticket.channel_id,
                        ticket.ticket.channel_epoch,
                        ticket.ticket.index,
                    )?,
                    ticket,
                );

                if index == tickets_len - 1 {
                    done = true;
                }
            }
        }

        self.db.batch(batch_ops, true).await?;

        Ok(tickets)
    }

    async fn get_acknowledged_tickets_range(
        &self,
        channel_id: &Hash,
        epoch: u32,
        index_start: u64,
        index_end: u64,
    ) -> Result<Vec<AcknowledgedTicket>> {
        let mut tickets = self
            .db
            .get_more_range::<AcknowledgedTicket>(
                to_acknowledged_ticket_key(channel_id, epoch, index_start)?.into(),
                to_acknowledged_ticket_key(channel_id, epoch, index_end)?.into(),
                &|_| true,
            )
            .await?;

        tickets.sort();

        Ok(tickets)
    }

    async fn replace_acked_tickets_by_aggregated_ticket(
        &mut self,
        aggregated_ticket: AcknowledgedTicket,
    ) -> Result<()> {
        let acked_tickets_to_replace = self
            .get_acknowledged_tickets_range(
                &aggregated_ticket.ticket.channel_id,
                aggregated_ticket.ticket.channel_epoch,
                aggregated_ticket.ticket.index,
                aggregated_ticket.ticket.index + aggregated_ticket.ticket.index_offset as u64,
            )
            .await?;

        let mut batch = utils_db::db::Batch::default();

        for acked_ticket in acked_tickets_to_replace.iter() {
            if let AcknowledgedTicketStatus::BeingRedeemed { tx_hash: _ } = acked_ticket.status {
                return Ok(());
            }
            batch.del(get_acknowledged_ticket_key(&acked_ticket)?);
        }

        batch.put(get_acknowledged_ticket_key(&aggregated_ticket)?, aggregated_ticket);

        self.db.batch(batch, true).await?;
        Ok(())
    }

    async fn get_unacknowledged_tickets(&self, filter: Option<ChannelEntry>) -> Result<Vec<UnacknowledgedTicket>> {
        Ok(self
            .db
            .get_more::<PendingAcknowledgement>(
                Vec::from(PENDING_ACKNOWLEDGEMENTS_PREFIX.as_bytes()).into_boxed_slice(),
                EthereumChallenge::SIZE as u32,
                &|pending: &PendingAcknowledgement| match pending {
                    PendingAcknowledgement::WaitingAsSender => false,
                    PendingAcknowledgement::WaitingAsRelayer(unack) => {
                        filter.map(|f| f.get_id() == unack.ticket.channel_id).unwrap_or(true)
                    }
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

    async fn update_acknowledged_ticket(&mut self, ticket: &AcknowledgedTicket) -> Result<()> {
        let key = get_acknowledged_ticket_key(&ticket)?;
        if self.db.contains(key.clone()).await {
            self.db.set(key, ticket).await.map(|_| ())
        } else {
            Err(DbError::NotFound)
        }
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
        let mut batch = Batch::default();
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

        let mut batch_ops = utils_db::db::Batch::default();
        batch_ops.put(channel_key, channel);
        batch_ops.put(snapshot_key, snapshot);

        self.db.batch(batch_ops, true).await
    }

    // core-ethereum only part
    async fn delete_acknowledged_tickets_from(&mut self, channel: ChannelEntry) -> Result<()> {
        let acknowledged_tickets = self.get_acknowledged_tickets(Some(channel)).await?;

        let key = utils_db::db::Key::new_from_str(NEGLECTED_TICKET_COUNT)?;
        let neglected_ticket_count = self.db.get_or_none::<usize>(key.clone()).await?.unwrap_or(0);

        let mut batch_ops = utils_db::db::Batch::default();
        for acked_ticket in acknowledged_tickets.iter() {
            batch_ops.del(get_acknowledged_ticket_key(&acked_ticket)?);
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

        let mut batch_ops = utils_db::db::Batch::default();
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

        let mut batch_ops = utils_db::db::Batch::default();
        // TODO: NOTE: was there a bug in the original implementation in TS? val.sub(val)?
        batch_ops.put(key.clone(), current_balance.sub(balance));
        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            snapshot,
        );

        self.db.batch(batch_ops, true).await
    }

    async fn mark_redeemed(&mut self, acked_ticket: &AcknowledgedTicket) -> Result<()> {
        debug!("marking {} as redeemed", acked_ticket);

        let mut ops = Batch::default();

        let key = utils_db::db::Key::new_from_str(REDEEMED_TICKETS_COUNT)?;
        let count = self.db.get_or_none::<usize>(key.clone()).await?.unwrap_or(0);
        ops.put(key, count + 1);

        let key = get_acknowledged_ticket_key(&acked_ticket)?;
        ops.del(key);

        let key = utils_db::db::Key::new_from_str(REDEEMED_TICKETS_VALUE)?;
        let balance = self
            .db
            .get_or_none::<Balance>(key.clone())
            .await?
            .unwrap_or(Balance::zero(BalanceType::HOPR));

        let new_redeemed_balance = balance.add(&acked_ticket.ticket.amount);
        ops.put(key, new_redeemed_balance);

        if let Some(counterparty) = self.get_channel(&acked_ticket.ticket.channel_id).await?.map(|c| {
            if c.source == self.me {
                c.destination
            } else {
                c.source
            }
        }) {
            let key = utils_db::db::Key::new_with_prefix(&counterparty, PENDING_TICKETS_COUNT)?;
            let pending_balance = self
                .db
                .get_or_none::<Balance>(key.clone())
                .await?
                .unwrap_or(Balance::zero(BalanceType::HOPR));

            let new_pending_balance = pending_balance.sub(&acked_ticket.ticket.amount);
            ops.put(key, new_pending_balance);
        } else {
            error!(
                "could not update redeemed tickets count: unable to find channel with id {}",
                acked_ticket.ticket.channel_id
            )
        }

        self.db.batch(ops, true).await
    }

    async fn mark_losing_acked_ticket(&mut self, acked_ticket: &AcknowledgedTicket) -> Result<()> {
        debug!("marking {acked_ticket} as losing",);

        let mut ops = utils_db::db::Batch::default();

        let key = utils_db::db::Key::new_from_str(LOSING_TICKET_COUNT)?;
        let count = self.db.get_or_none::<usize>(key.clone()).await?.unwrap_or(0);
        ops.put(key, count + 1);

        let key = get_acknowledged_ticket_key(&acked_ticket)?;
        ops.del(key);

        if let Some(counterparty) = self.get_channel(&acked_ticket.ticket.channel_id).await?.map(|c| {
            if c.source == self.me {
                c.destination
            } else {
                c.source
            }
        }) {
            let key = utils_db::db::Key::new_with_prefix(&counterparty, PENDING_TICKETS_COUNT)?;
            let balance = self
                .db
                .get_or_none::<Balance>(key.clone())
                .await?
                .unwrap_or(Balance::zero(BalanceType::HOPR));
            ops.put(key, balance.sub(&acked_ticket.ticket.amount));
        } else {
            error!(
                "could not update losing tickets count: unable to find channel with id {}",
                acked_ticket.ticket.channel_id
            )
        }

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
        let mut batch_ops = utils_db::db::Batch::default();
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
        let mut batch_ops = utils_db::db::Batch::default();
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
        let mut batch_ops = utils_db::db::Batch::default();
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

        let mut batch_ops = utils_db::db::Batch::default();
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

        let mut batch_ops = utils_db::db::Batch::default();
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

        let mut batch_ops = utils_db::db::Batch::default();
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

        let mut batch_ops = utils_db::db::Batch::default();
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

        let mut batch_ops = utils_db::db::Batch::default();
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
        let mut batch_ops = utils_db::db::Batch::default();
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

        let mut batch_ops = utils_db::db::Batch::default();
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

        let mut batch_ops = utils_db::db::Batch::default();

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
                let mut batch_ops = utils_db::db::Batch::default();
                batch_ops.put(mfa_key, mfa_address);
                batch_ops.put(snapshot_key, snapshot);

                self.db.batch(batch_ops, true).await
            }
            None => {
                let mut batch_ops = utils_db::db::Batch::default();
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

/// Only meant for testing!
#[async_trait(? Send)]
impl<T: AsyncKVStorage<Key = Box<[u8]>, Value = Box<[u8]>>> HoprCoreEthereumTestActions for CoreEthereumDb<T> {
    async fn store_acknowledged_tickets(&mut self, acked_tickets: Vec<AcknowledgedTicket>) -> Result<()> {
        let mut batch_ops = utils_db::db::Batch::default();

        for acked_ticket in acked_tickets {
            batch_ops.put(
                to_acknowledged_ticket_key(
                    &acked_ticket.ticket.channel_id,
                    acked_ticket.ticket.channel_epoch,
                    acked_ticket.ticket.index,
                )?
                .into(),
                acked_ticket,
            );
        }

        self.db.batch(batch_ops, true).await
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
    use std::sync::Arc;
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
        core_ethereum_db: Arc<RwLock<CoreEthereumDb<utils_db::rusty::RustyLevelDbShim>>>,
    }

    impl Database {
        pub fn as_ref_counted(&self) -> Arc<RwLock<CoreEthereumDb<utils_db::rusty::RustyLevelDbShim>>> {
            self.core_ethereum_db.clone()
        }
    }

    #[wasm_bindgen]
    impl Database {
        pub fn new_in_memory(me_addr: Address) -> Self {
            Self {
                core_ethereum_db: Arc::new(RwLock::new(CoreEthereumDb::new(
                    DB::new(utils_db::rusty::RustyLevelDbShim::new_in_memory()),
                    me_addr,
                ))),
            }
        }

        #[wasm_bindgen(constructor)]
        pub fn new(path: &str, create_if_missing: bool, me_addr: Address) -> Self {
            Self {
                core_ethereum_db: Arc::new(RwLock::new(CoreEthereumDb::new(
                    DB::new(utils_db::rusty::RustyLevelDbShim::new(path, create_if_missing)),
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
        pub async fn update_acknowledged_ticket(&self, ticket: &AcknowledgedTicket) -> Result<(), JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_write! {
            let mut db = data.write().await;
            utils_misc::ok_or_jserr!(db.update_acknowledged_ticket(&ticket.into()).await)
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
        pub async fn mark_redeemed(&self, acked_ticket: &AcknowledgedTicket) -> Result<(), JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_write! {
            let mut db = data.write().await;
            utils_misc::ok_or_jserr!(db.mark_redeemed(&acked_ticket.into()).await)
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
        pub async fn mark_losing_acked_ticket(&self, ticket: &AcknowledgedTicket) -> Result<(), JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_write! {
            let mut db = data.write().await;
            utils_misc::ok_or_jserr!(db.mark_losing_acked_ticket(&ticket.into()).await)
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
        pub async fn is_allowed_to_access_network(&self, node: &Address) -> Result<bool, JsValue> {
            let data = self.core_ethereum_db.clone();
            //check_lock_read! {
            let db = data.read().await;
            utils_misc::ok_or_jserr!(db.is_allowed_to_access_network(&node.clone()).await)
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
    use core_crypto::{
        keypairs::{ChainKeypair, Keypair},
        types::{Challenge, CurvePoint, HalfKey, Response},
    };
    use core_types::channels::ChannelEntry;
    use hex_literal::hex;
    use lazy_static::lazy_static;
    use std::str::FromStr;
    use utils_db::{db::serialize_to_bytes, rusty::RustyLevelDbShim};
    use utils_types::{
        primitives::{Address, EthereumChallenge},
        traits::BinarySerializable,
    };

    const ALICE: [u8; 32] = hex!("37eafd5038311f90fc08d13ff9ee16c6709be666e7d96808ba9a786c18f868a8");
    const BOB: [u8; 32] = hex!("d39a926980d6fa96a9eba8f8058b2beb774bc11866a386e9ddf9dc1152557c26");

    lazy_static! {
        static ref ALICE_KEYPAIR: ChainKeypair = ChainKeypair::from_secret(&ALICE).unwrap();
        static ref BOB_KEYPAIR: ChainKeypair = ChainKeypair::from_secret(&BOB).unwrap();
    }

    fn mock_ticket(
        pk: &ChainKeypair,
        counterparty: &Address,
        domain_separator: Option<Hash>,
        index: Option<U256>,
        index_offset: Option<U256>,
        channel_epoch: Option<U256>,
        challenge: Option<EthereumChallenge>,
    ) -> Ticket {
        let win_prob = 1.0f64; // 100 %
        let price_per_packet: U256 = 10000000000000000u128.into(); // 0.01 HOPR
        let path_pos = 5u64;

        Ticket::new(
            counterparty,
            &Balance::new(
                price_per_packet.divide_f64(win_prob).unwrap() * path_pos.into(),
                BalanceType::HOPR,
            ),
            index.unwrap_or(U256::one()),
            index_offset.unwrap_or(U256::one()),
            1.0f64,
            channel_epoch.unwrap_or(U256::one()),
            challenge.unwrap_or_default(),
            pk,
            &domain_separator.unwrap_or_default(),
        )
        .unwrap()
    }

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
        let mut db = CoreEthereumDb::new(DB::new(RustyLevelDbShim::new_in_memory()), Address::random());

        assert_eq!(db.get_ticket_price().await, Ok(None));

        assert!(db.set_ticket_price(&U256::from(100u64)).await.is_ok());

        assert_eq!(db.get_ticket_price().await, Ok(Some(U256::from(100u64))));
    }

    #[async_std::test]
    async fn test_set_network_registry() {
        let mut db = CoreEthereumDb::new(DB::new(RustyLevelDbShim::new_in_memory()), Address::random());

        assert_eq!(db.is_network_registry_enabled().await, Ok(true));

        assert!(db.set_network_registry(false, &Snapshot::default()).await.is_ok());

        assert_eq!(db.is_network_registry_enabled().await, Ok(false));
    }

    #[async_std::test]
    async fn test_allowed_to_access_network() {
        let mut db = CoreEthereumDb::new(DB::new(RustyLevelDbShim::new_in_memory()), Address::random());

        let test_address = Address::from_str("0xa6416794a09d1c8c4c6110f83f42cf6f1ed9c416").unwrap();

        assert_eq!(db.is_allowed_to_access_network(&test_address).await.unwrap(), false);

        db.set_allowed_to_access_network(&test_address, true, &Snapshot::default())
            .await
            .unwrap();

        assert_eq!(db.is_allowed_to_access_network(&test_address).await.unwrap(), true);

        db.set_allowed_to_access_network(&test_address, false, &Snapshot::default())
            .await
            .unwrap();

        assert_eq!(db.is_allowed_to_access_network(&test_address).await.unwrap(), false);
    }

    #[async_std::test]
    async fn test_token_storage() {
        let mut db = CoreEthereumDb::new(DB::new(RustyLevelDbShim::new_in_memory()), Address::random());

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
        let mut db = CoreEthereumDb::new(DB::new(RustyLevelDbShim::new_in_memory()), Address::random());

        let test_address = Address::from_str("0xa6416794a09d1c8c4c6110f83f42cf6f1ed9c416").unwrap();

        db.set_mfa_protected_and_update_snapshot(Some(test_address), &Snapshot::default())
            .await
            .unwrap();

        assert_eq!(db.is_mfa_protected().await.unwrap(), Some(test_address));
    }

    #[async_std::test]
    async fn test_aggregatable_acknowledged_tickets() {
        let mut db = CoreEthereumDb::new(DB::new(RustyLevelDbShim::new_in_memory()), Address::random());

        let mut hk1_seed: [u8; 32] = hex!("3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa");
        let mut hk2_seed: [u8; 32] = hex!("4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b");

        let mut acked_tickets: Vec<AcknowledgedTicket> = vec![];

        let start_index = 23u64;
        let tickets_to_generate = 3u64;

        let channel_epoch = 29u32;

        for i in start_index..start_index + tickets_to_generate {
            let cp1: CurvePoint = HalfKey::from_bytes(&hk1_seed).unwrap().to_challenge().into();
            let cp2: CurvePoint = HalfKey::from_bytes(&hk2_seed).unwrap().to_challenge().into();
            let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

            let ticket = mock_ticket(
                &ALICE_KEYPAIR,
                &BOB_KEYPAIR.public().to_address(),
                None,
                Some(i.into()),
                None,
                Some(channel_epoch.into()),
                Some(Challenge::from(cp_sum).to_ethereum_challenge()),
            );

            let unacked_ticket = UnacknowledgedTicket::new(
                ticket,
                HalfKey::from_bytes(&hk1_seed).unwrap(),
                ALICE_KEYPAIR.public().to_address(),
            );

            db.store_pending_acknowledgment(
                HalfKey::from_bytes(&hk1_seed).unwrap().to_challenge(),
                PendingAcknowledgement::WaitingAsRelayer(unacked_ticket),
            )
            .await
            .unwrap();

            let unacked_ticket = match db
                .get_pending_acknowledgement(&HalfKey::from_bytes(&hk1_seed).unwrap().to_challenge())
                .await
                .unwrap()
                .unwrap()
            {
                PendingAcknowledgement::WaitingAsRelayer(unacked) => unacked,
                _ => panic!("must not happen"),
            };

            let acked_ticket = unacked_ticket
                .acknowledge(&HalfKey::from_bytes(&hk2_seed).unwrap(), &BOB_KEYPAIR, &Hash::default())
                .unwrap();

            assert!(acked_ticket
                .verify(
                    &ALICE_KEYPAIR.public().to_address(),
                    &BOB_KEYPAIR.public().to_address(),
                    &Hash::default()
                )
                .is_ok());

            acked_tickets.push(acked_ticket.clone());

            db.replace_unack_with_ack(&HalfKey::from_bytes(&hk1_seed).unwrap().to_challenge(), acked_ticket)
                .await
                .unwrap();

            let new_hk1_seed = Hash::create(&[&hk1_seed.clone()]);
            let new_hk2_seed = Hash::create(&[&hk1_seed.clone()]);

            hk1_seed.copy_from_slice(&new_hk1_seed.to_bytes());
            hk2_seed.copy_from_slice(&new_hk2_seed.to_bytes());
        }

        let acked_tickets_from_db = db.get_acknowledged_tickets(None).await.unwrap();
        let tickets_from_db = db.get_tickets(Some(ALICE_KEYPAIR.public().to_address())).await.unwrap();

        for i in 0usize..tickets_to_generate as usize {
            assert_eq!(acked_tickets[i], acked_tickets_from_db[i]);
            assert_eq!(tickets_from_db[i], acked_tickets[i].ticket);
        }

        let channel_id = generate_channel_id(&ALICE_KEYPAIR.public().to_address(), &BOB_KEYPAIR.public().to_address());

        assert_eq!(channel_id, acked_tickets[0].ticket.channel_id);

        let acked_tickets_range = db
            .get_acknowledged_tickets_range(&channel_id, channel_epoch, 20, start_index + 1)
            .await
            .unwrap();

        // should be a subset
        assert_eq!(acked_tickets_range.len(), 2);

        // now aggregate them
        let aggregated_ticket = mock_ticket(
            &ALICE_KEYPAIR,
            &BOB_KEYPAIR.public().to_address(),
            None,
            Some(start_index.into()),
            Some(2u64.into()),
            Some(channel_epoch.into()),
            Some(acked_tickets[0].ticket.challenge.clone()),
        );

        let aggregated_acked_ticket = AcknowledgedTicket::new(
            aggregated_ticket,
            acked_tickets[0].response.to_owned(),
            acked_tickets[0].signer.to_owned(),
            &BOB_KEYPAIR,
            &Hash::default(),
        )
        .unwrap();

        db.replace_acked_tickets_by_aggregated_ticket(aggregated_acked_ticket)
            .await
            .unwrap();

        let acked_tickets_range = db
            .get_acknowledged_tickets_range(&channel_id, channel_epoch, 20, start_index + 1)
            .await
            .unwrap();

        // only one ticket after replacing with aggregated ticket
        assert_eq!(acked_tickets_range.len(), 1);
    }

    #[async_std::test]
    async fn test_acknowledged_ticket_status() {
        let mut db = CoreEthereumDb::new(DB::new(RustyLevelDbShim::new_in_memory()), Address::random());

        let mut challenge_seed: [u8; 32] = hex!("c04824c574e562b3b96725c8aa6e5b0426a3900cd9efbe48ddf7e754a552abdf");

        let amount_tickets = 29u64;
        let domain_separator = Hash::default();
        let mut acked_tickets = (0..amount_tickets)
            .map(|i| {
                let challenge =
                    Challenge::from(CurvePoint::from_exponent(&challenge_seed).unwrap()).to_ethereum_challenge();

                let ticket = mock_ticket(
                    &ALICE_KEYPAIR,
                    &(&*BOB_KEYPAIR).into(),
                    Some(domain_separator),
                    Some(i.into()),
                    Some(1u64.into()),
                    Some(1u64.into()),
                    Some(challenge),
                );

                challenge_seed = Hash::create(&[&challenge_seed]).into();

                AcknowledgedTicket::new(
                    ticket,
                    Response::from_bytes(&challenge_seed.clone()).unwrap(),
                    (&*ALICE_KEYPAIR).into(),
                    &BOB_KEYPAIR,
                    &domain_separator,
                )
                .unwrap()
            })
            .collect::<Vec<AcknowledgedTicket>>();

        // Add some being redeemed tickets
        acked_tickets[12].status(AcknowledgedTicketStatus::BeingRedeemed {
            tx_hash: Hash::default(),
        });
        acked_tickets[17].status(AcknowledgedTicketStatus::BeingRedeemed {
            tx_hash: Hash::default(),
        });

        db.store_acknowledged_tickets(acked_tickets).await.unwrap();

        let channel_id = generate_channel_id(&(&*ALICE_KEYPAIR).into(), &(&*BOB_KEYPAIR).into());

        let stored_acked_tickets = db
            .prepare_aggregatable_tickets(&channel_id, 1u32, 0u64, u64::MAX)
            .await
            .unwrap();

        assert_eq!(stored_acked_tickets.len(), 11);

        assert!(stored_acked_tickets
            .iter()
            .all(|acked_ticket| AcknowledgedTicketStatus::BeingAggregated {
                start: 0u64,
                end: u64::MAX,
            } == acked_ticket.status));
    }
}
