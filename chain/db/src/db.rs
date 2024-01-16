use std::collections::HashMap;

use async_trait::async_trait;
use hopr_crypto::types::{HalfKeyChallenge, Hash, OffchainPublicKey};
use hopr_internal_types::channels::ChannelDirection;
use hopr_internal_types::{
    account::AccountEntry,
    acknowledgement::{AcknowledgedTicket, AcknowledgedTicketStatus, PendingAcknowledgement, UnacknowledgedTicket},
    channels::{generate_channel_id, ChannelEntry, ChannelStatus, Ticket},
};
use hopr_primitive_types::{
    primitives::{Address, Balance, BalanceType, EthereumChallenge, Snapshot, U256},
    traits::BinarySerializable,
};
use log::{debug, error, info};
use utils_db::errors::DbError;
use utils_db::{
    constants::*,
    db::{Batch, DB},
    traits::AsyncKVStorage,
};

use crate::{errors::Result, traits::HoprCoreEthereumDbActions};

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

#[derive(Debug, Clone)]
pub struct CoreEthereumDb<T>
where
    T: AsyncKVStorage<Key = Box<[u8]>, Value = Box<[u8]>> + Clone,
{
    pub db: DB<T>,
    cached_unrealized_value: HashMap<Hash, Balance>,
    pub me: Address,
}

impl<T: AsyncKVStorage<Key = Box<[u8]>, Value = Box<[u8]>> + Clone + Send + Sync> CoreEthereumDb<T> {
    pub fn new(db: DB<T>, me: Address) -> Self {
        Self {
            db,
            cached_unrealized_value: HashMap::new(),
            me,
        }
    }

    pub async fn init_cache(&mut self) -> Result<()> {
        // let channels = self.get_channels().await?;
        // info!("Cleaning up invalid tickets from {} tracked channels...", channels.len());
        // for channel in channels.iter() {
        //     self.cleanup_invalid_channel_tickets(channel).await?
        // }

        let mut cached_channel: HashMap<Hash, (u32, u64)> = HashMap::new(); // channel_id: (channel_epoch, ticket_index)
        debug!("Fetching all tickets to calculate the unrealized value in tracked channels...");

        // FIXME: Currently a node does not have a way of reconciling unacknowledged
        // tickets with the sender. Therefore, the use of unack tickets could make a
        // channel inoperable. Re-enable the use of unacknowledged tickets in this
        // calculation once a reconciliation mechanism has been implemented
        let tickets = self.get_acknowledged_tickets(None).await?;
        info!("Calculating unrealized balance for {} tickets...", tickets.len());

        for ack_ticket in tickets.into_iter() {
            let ticket = ack_ticket.ticket;
            // get the corresponding channel info from the cached_channel, or from the db
            let (channel_epoch, ticket_index) = {
                if let Some((current_channel_epoch, current_ticket_index)) =
                    cached_channel.get(&ticket.channel_id).copied()
                {
                    // from the cached_channel
                    (current_channel_epoch, current_ticket_index)
                } else {
                    // read from db
                    let (channel_epoch, ticket_index) = {
                        if let Some(channel_entry) = self.get_channel(&ticket.channel_id).await? {
                            // update the cached value
                            (
                                channel_entry.channel_epoch.as_u32(),
                                channel_entry.ticket_index.as_u64(),
                            )
                        } else {
                            (0u32, 0u64)
                        }
                    };
                    // update the cached value
                    cached_channel.insert(ticket.channel_id, (channel_epoch, ticket_index));
                    (channel_epoch, ticket_index)
                }
            };

            if ticket.channel_epoch == channel_epoch && ticket.index >= ticket_index {
                // only calculate unrealized balance if ticket is issued of the current channel_epoch and index larger than or equal to the ticket_index in the channel
                // do nothing to tickets that is issued to channel_epoch larger than the current channel_epoch
                // TODO: for other tickets (of previous channel epoch; or of the current channel epoch but index smaller than ticket_index in the channel), remove it from the db
                let unrealized_balance = self
                    .cached_unrealized_value
                    .get(&ticket.channel_id)
                    .copied()
                    .unwrap_or(Balance::zero(BalanceType::HOPR))
                    .add(&ticket.amount);

                self.cached_unrealized_value
                    .insert(ticket.channel_id, unrealized_balance);
            }
        }

        Ok(())
    }
}

#[async_trait] // not placing the `Send` trait limitations on the trait
impl<T: AsyncKVStorage<Key = Box<[u8]>, Value = Box<[u8]>> + Clone + Send + Sync> HoprCoreEthereumDbActions
    for CoreEthereumDb<T>
{
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

    // combine the two function above to increase the current ticket index of a channel
    async fn increase_current_ticket_index(&mut self, channel_id: &Hash) -> Result<()> {
        let prefixed_key = utils_db::db::Key::new_with_prefix(channel_id, TICKET_INDEX_PREFIX)?;
        let current_index = self
            .db
            .get_or_none::<U256>(prefixed_key.clone())
            .await?
            .unwrap_or(U256::zero());
        let _evicted = self.db.set(prefixed_key, &current_index.addn(1_u32)).await?;
        // Ignoring evicted value
        Ok(())
    }

    // ensure the current ticket index is not smaller than the given value. If it's samller, set to the given value
    async fn ensure_current_ticket_index_gte(&mut self, channel_id: &Hash, index: U256) -> Result<()> {
        let prefixed_key = utils_db::db::Key::new_with_prefix(channel_id, TICKET_INDEX_PREFIX)?;
        let current_index = self
            .db
            .get_or_none::<U256>(prefixed_key.clone())
            .await?
            .unwrap_or(U256::zero());
        // compare the current_index with index, if current_index is smaller than index, set to index
        if current_index < index {
            let _evicted = self.db.set(prefixed_key, &index).await?;
            // Ignoring evicted value
            // flush the db after setter
            self.db.flush().await?;
        }
        Ok(())
    }

    async fn get_tickets(&self, maybe_signer: Option<Address>) -> Result<Vec<Ticket>> {
        let mut acked_tickets = self
            .db
            .get_more::<AcknowledgedTicket>(
                Vec::from(ACKNOWLEDGED_TICKETS_PREFIX.as_bytes()).into_boxed_slice(),
                ACKNOWLEDGED_TICKETS_KEY_LENGTH as u32,
                Box::new(move |v: &AcknowledgedTicket| maybe_signer.map(|s| v.signer.eq(&s)).unwrap_or(true)),
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
                Box::new(move |v: &PendingAcknowledgement| match v {
                    PendingAcknowledgement::WaitingAsSender => false,
                    PendingAcknowledgement::WaitingAsRelayer(unack) => match maybe_signer {
                        None => true,
                        Some(signer) => unack.signer.eq(&signer),
                    },
                }),
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

    async fn get_unrealized_balance(&self, channel: &Hash) -> Result<Balance> {
        let channel_key = utils_db::db::Key::new_with_prefix(channel, CHANNEL_PREFIX)?;
        let channel_balance = self
            .db
            .get_or_none::<ChannelEntry>(channel_key)
            .await?
            .map(|c| c.balance);

        Ok(if let Some(balance) = channel_balance {
            if let Some(unrealized_balance) = self.cached_unrealized_value.get(channel) {
                debug!("channel {channel} has unrealized balance {unrealized_balance} to be subtracted from balance {balance}");
                balance.sub(unrealized_balance)
            } else {
                debug!("channel {channel} has no unrealized balance to be subtracted from balance {balance}");
                balance
            }
        } else {
            debug!("channel {channel} has no unrealized balance because it does not exist");
            Balance::zero(BalanceType::HOPR)
        })
    }

    async fn get_channel_epoch(&self, channel: &Hash) -> Result<Option<U256>> {
        let channel_key = utils_db::db::Key::new_with_prefix(channel, CHANNEL_PREFIX)?;
        Ok(self
            .db
            .get_or_none::<ChannelEntry>(channel_key)
            .await?
            .map(|c| c.channel_epoch))
    }

    async fn cleanup_invalid_channel_tickets(&mut self, channel: &ChannelEntry) -> Result<()> {
        // Get all ack tickets in the given channel with channel epoch less than the one on the channel
        let channel = *channel;
        let channel2 = channel;
        let ack_tickets = self
            .db
            .get_more::<AcknowledgedTicket>(
                Vec::from(ACKNOWLEDGED_TICKETS_PREFIX.as_bytes()).into_boxed_slice(),
                ACKNOWLEDGED_TICKETS_KEY_LENGTH as u32,
                Box::new(move |ack: &AcknowledgedTicket| {
                    channel.get_id() == ack.ticket.channel_id
                        && channel.channel_epoch.as_u32() > ack.ticket.channel_epoch
                }),
            )
            .await?
            .into_iter()
            .map(|ack| (ack.ticket.amount, get_acknowledged_ticket_key(&ack)));

        // Get all unack tickets in the given channel with channel epoch less than the one on the channel
        let unack_tickets = self
            .db
            .get_more::<PendingAcknowledgement>(
                Vec::from(PENDING_ACKNOWLEDGEMENTS_PREFIX.as_bytes()).into_boxed_slice(),
                HalfKeyChallenge::SIZE as u32,
                Box::new(move |v: &PendingAcknowledgement| match v {
                    PendingAcknowledgement::WaitingAsSender => false,
                    PendingAcknowledgement::WaitingAsRelayer(unack) => {
                        channel2.get_id() == unack.ticket.channel_id
                            && channel2.channel_epoch.as_u32() > unack.ticket.channel_epoch
                    }
                }),
            )
            .await?
            .into_iter()
            .filter_map(|pa| match pa {
                PendingAcknowledgement::WaitingAsSender => None,
                PendingAcknowledgement::WaitingAsRelayer(unack) => Some((
                    unack.ticket.amount,
                    utils_db::db::Key::new_with_prefix(&unack.own_key.to_challenge(), PENDING_ACKNOWLEDGEMENTS_PREFIX),
                )),
            });

        let count_key = utils_db::db::Key::new_from_str(NEGLECTED_TICKETS_COUNT)?;
        let value_key = utils_db::db::Key::new_from_str(NEGLECTED_TICKETS_VALUE)?;

        let mut count = self.get_neglected_tickets_count().await?;
        let mut balance = self.get_neglected_tickets_value().await?;

        let mut batch_ops = Batch::default();

        // All invalid tickets will be marked as neglected
        for (amount, maybe_key) in ack_tickets.chain(unack_tickets) {
            batch_ops.del(maybe_key?);
            balance = balance.add(&amount);
            count += 1;
        }

        batch_ops.put(count_key, count);
        batch_ops.put(value_key, balance);
        self.db.batch(batch_ops, true).await
    }

    async fn mark_rejected(&mut self, ticket: &Ticket) -> Result<()> {
        let count = self.get_rejected_tickets_count().await?;
        let count_key = utils_db::db::Key::new_from_str(REJECTED_TICKETS_COUNT)?;

        let balance = self.get_rejected_tickets_value().await?;
        let value_key = utils_db::db::Key::new_from_str(REJECTED_TICKETS_VALUE)?;

        let mut batch_ops = utils_db::db::Batch::default();
        batch_ops.put(count_key, count + 1);
        batch_ops.put(value_key, balance.add(&ticket.amount));

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
        self.db.set(key, &pending_acknowledgment).await?;

        // FIXME: Currently a node does not have a way of reconciling unacknowledged
        // tickets with the sender. Therefore, the use of unack tickets could make a
        // channel inoperable. Re-enable the use of unacknowledged tickets in this
        // calculation once a reconciliation mechanism has been implemented

        // if let PendingAcknowledgement::WaitingAsRelayer(v) = pending_acknowledgment {
        //     let current_unrealized_value = self
        //         .cached_unrealized_value
        //         .get(&v.ticket.channel_id)
        //         .map(|v| v.clone())
        //         .unwrap_or(Balance::zero(BalanceType::HOPR));

        //     self.cached_unrealized_value
        //         .insert(v.ticket.channel_id, current_unrealized_value.add(&v.ticket.amount));
        // }

        self.db.flush().await?;

        Ok(())
    }

    async fn replace_unack_with_ack(
        &mut self,
        half_key_challenge: &HalfKeyChallenge,
        acked_ticket: AcknowledgedTicket,
    ) -> Result<()> {
        let unack_key = utils_db::db::Key::new_with_prefix(half_key_challenge, PENDING_ACKNOWLEDGEMENTS_PREFIX)?;
        let ack_key = get_acknowledged_ticket_key(&acked_ticket)?;

        // FIXME: Currently a node does not have a way of reconciling unacknowledged
        // tickets with the sender. Therefore, the use of unack tickets could make a
        // channel inoperable. Re-enable the use of unacknowledged tickets in this
        // calculation once a reconciliation mechanism has been implemented

        let current_unrealized_value = self
            .cached_unrealized_value
            .get(&acked_ticket.ticket.channel_id)
            .copied()
            .unwrap_or(Balance::zero(BalanceType::HOPR));

        self.cached_unrealized_value.insert(
            acked_ticket.ticket.channel_id,
            current_unrealized_value.add(&acked_ticket.ticket.amount),
        );

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
                Box::new(move |ack: &AcknowledgedTicket| {
                    filter.map(|f| f.get_id() == ack.ticket.channel_id).unwrap_or(true)
                }),
            )
            .await?;

        tickets.sort();

        Ok(tickets)
    }

    // core and core-ethereum part
    async fn get_acknowledged_tickets_count(&self, filter: Option<ChannelEntry>) -> Result<usize> {
        let tickets = self
            .db
            .get_more::<AcknowledgedTicket>(
                Vec::from(ACKNOWLEDGED_TICKETS_PREFIX.as_bytes()).into_boxed_slice(),
                ACKNOWLEDGED_TICKETS_KEY_LENGTH as u32,
                Box::new(move |ack: &AcknowledgedTicket| {
                    filter.map(|f| f.get_id() == ack.ticket.channel_id).unwrap_or(true)
                }),
            )
            .await?;

        Ok(tickets.len())
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
        assert!(index_start < index_end, "aggregation indices must be valid");

        let channel_key = utils_db::db::Key::new_with_prefix(channel_id, CHANNEL_PREFIX)?;
        let channel = self
            .db
            .get_or_none::<ChannelEntry>(channel_key)
            .await?
            .expect("must aggregate tickets on an existing channel");

        // Perform sanity checks on the arguments
        assert_eq!(
            ChannelDirection::Incoming,
            channel.direction(&self.me).expect("must be own channel"),
            "aggregation request can happen on incoming channels only"
        );
        assert_ne!(
            ChannelStatus::Closed,
            channel.status,
            "must not aggregate tickets on a closed channel"
        );
        assert_eq!(channel.channel_epoch.as_u32(), epoch, "channel epoch must be valid");
        assert!(
            index_start <= channel.ticket_index.as_u64(),
            "aggregation start index must be less or equal to te current channel ticket index"
        );

        let channel_balance = channel.balance;

        // We store only winning tickets
        let mut tickets = self
            .get_acknowledged_tickets_range(channel_id, epoch, index_start, index_end)
            .await?;

        // We start aggregating from the first ticket that is not in `BeingRedeemed` state, after the last one that is `BeingRedeemed`, for the given range
        let last_redeemed_idx = tickets
            .iter()
            .enumerate()
            .rfind(|(_, t)| t.status.is_being_redeemed())
            .map(|(idx, _)| idx);

        // If no `BeingRedeemed` tickets are in that range, take the entire range
        let agg_start = last_redeemed_idx.map(|idx| idx + 1).unwrap_or(0);

        // Check if there's nothing to aggregate in the given range (= last ticket is `BeingRedeemed`)
        if agg_start > tickets.len() {
            debug!("no tickets to aggregate in {channel_id} ({epoch}) range {index_start}-{index_end}");
            return Ok(vec![]);
        }

        let mut agg_tickets = Vec::new();
        let mut batch_ops = Batch::default();

        // Set all the tickets to `BeingAggregated`
        let mut total_value = Balance::zero(BalanceType::HOPR);
        for ticket in &mut tickets[agg_start..] {
            // Check if we did not exceed the total balance on the channel
            total_value = total_value.add(&ticket.ticket.amount);
            if total_value.gt(&channel_balance) {
                debug!("no more tickets to aggregate in {channel_id} ({epoch}) range {index_start}-{index_end}: channel balance exceeded {channel_balance}");
                break;
            }

            ticket.status = AcknowledgedTicketStatus::BeingAggregated {
                start: index_start,
                end: index_end,
            };

            batch_ops.put(
                to_acknowledged_ticket_key(
                    &ticket.ticket.channel_id,
                    ticket.ticket.channel_epoch,
                    ticket.ticket.index,
                )?,
                ticket.clone(),
            );

            // Collect just the tickets to be aggregated
            agg_tickets.push(ticket.clone());
        }

        self.db.batch(batch_ops, true).await?;

        debug!(
            "prepared {} tickets to aggregate in {channel_id} ({epoch}) range {index_start}-{index_end}",
            agg_tickets.len()
        );
        Ok(agg_tickets)
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
                Box::new(|_| true),
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
            batch.del(get_acknowledged_ticket_key(acked_ticket)?);
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
                Box::new(move |pending: &PendingAcknowledgement| match pending {
                    PendingAcknowledgement::WaitingAsSender => false,
                    PendingAcknowledgement::WaitingAsRelayer(unack) => {
                        filter.map(|f| f.get_id() == unack.ticket.channel_id).unwrap_or(true)
                    }
                }),
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
        let key = get_acknowledged_ticket_key(ticket)?;
        if self.db.contains(key.clone()).await {
            self.db.set(key, ticket).await.map(|_| ())
        } else {
            Err(DbError::NotFound)
        }
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
        //log::debug!("DB: get_channel_to dest: {}", dest);
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
        //log::debug!("DB: update_channel_and_snapshot channel_id: {}", channel_id);
        let channel_key = utils_db::db::Key::new_with_prefix(channel_id, CHANNEL_PREFIX)?;
        let snapshot_key = utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?;

        let unrealized_balance = {
            if let Some(previous_channel_entry) = self.get_channel(channel_id).await? {
                if previous_channel_entry.channel_epoch == channel.channel_epoch {
                    // if funding, current > previous, resulting balance will be 0 and not subtracted
                    // if redeeming, current < previous, resulting balance > 0 and can be subtracted from the cached one
                    let updated_balance_difference = previous_channel_entry.balance.sub(&channel.balance);

                    self.cached_unrealized_value
                        .get(channel_id)
                        .map(|v| v.sub(&updated_balance_difference))
                        .unwrap_or(Balance::zero(BalanceType::HOPR))
                } else {
                    Balance::zero(BalanceType::HOPR)
                }
            } else {
                Balance::zero(BalanceType::HOPR)
            }
        };
        self.cached_unrealized_value.insert(*channel_id, unrealized_balance);

        let mut batch_ops = utils_db::db::Batch::default();

        batch_ops.put(channel_key, channel);
        batch_ops.put(snapshot_key, snapshot);

        self.db.batch(batch_ops, true).await
    }

    // core-ethereum only part
    async fn mark_acknowledged_tickets_neglected(&mut self, channel: ChannelEntry) -> Result<()> {
        let acknowledged_tickets: Vec<AcknowledgedTicket> = self
            .get_acknowledged_tickets(Some(channel))
            .await?
            .into_iter()
            .filter(|ack| U256::from(ack.ticket.channel_epoch) <= channel.channel_epoch)
            .collect();

        let count_key = utils_db::db::Key::new_from_str(NEGLECTED_TICKETS_COUNT)?;
        let value_key = utils_db::db::Key::new_from_str(NEGLECTED_TICKETS_VALUE)?;

        let neglected_ticket_count = self.db.get_or_none::<usize>(count_key.clone()).await?.unwrap_or(0);
        let mut neglected_ticket_value = self
            .db
            .get_or_none::<Balance>(value_key.clone())
            .await?
            .unwrap_or(Balance::zero(BalanceType::HOPR));

        let mut batch_ops = utils_db::db::Batch::default();
        for acked_ticket in acknowledged_tickets.iter() {
            batch_ops.del(get_acknowledged_ticket_key(acked_ticket)?);
            neglected_ticket_value = neglected_ticket_value.add(&acked_ticket.ticket.amount);
        }

        if !acknowledged_tickets.is_empty() {
            batch_ops.put(count_key, neglected_ticket_count + acknowledged_tickets.len());
            batch_ops.put(value_key, neglected_ticket_value);
        }

        self.db.batch(batch_ops, true).await
    }

    async fn get_latest_block_number(&self) -> Result<Option<u32>> {
        let key = utils_db::db::Key::new_from_str(LATEST_BLOCK_NUMBER_KEY)?;
        self.db.get_or_none::<u32>(key).await
    }

    async fn update_latest_block_number(&mut self, number: u32) -> Result<()> {
        //log::debug!("DB: update_latest_block_number to {}", number);
        let key = utils_db::db::Key::new_from_str(LATEST_BLOCK_NUMBER_KEY)?;
        let _ = self.db.set(key, &number).await?;
        Ok(())
    }

    async fn get_latest_confirmed_snapshot(&self) -> Result<Option<Snapshot>> {
        let key = utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?;
        self.db.get_or_none::<Snapshot>(key).await
    }

    async fn get_channel(&self, channel: &Hash) -> Result<Option<ChannelEntry>> {
        //log::debug!("DB: get_channel {}", channel);
        let key = utils_db::db::Key::new_with_prefix(channel, CHANNEL_PREFIX)?;
        self.db.get_or_none::<ChannelEntry>(key).await
    }

    async fn get_channels(&self) -> Result<Vec<ChannelEntry>> {
        self.db
            .get_more::<ChannelEntry>(
                Box::from(CHANNEL_PREFIX.as_bytes()),
                Hash::SIZE as u32,
                Box::new(|_| true),
            )
            .await
    }

    async fn get_channels_open(&self) -> Result<Vec<ChannelEntry>> {
        Ok(self
            .db
            .get_more::<ChannelEntry>(
                Box::from(CHANNEL_PREFIX.as_bytes()),
                Hash::SIZE as u32,
                Box::new(|_| true),
            )
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
            .get_more::<AccountEntry>(
                Box::from(ACCOUNT_PREFIX.as_bytes()),
                Address::SIZE as u32,
                Box::new(|_| true),
            )
            .await
    }

    async fn get_public_node_accounts(&self) -> Result<Vec<AccountEntry>> {
        self.db
            .get_more::<AccountEntry>(
                Box::from(ACCOUNT_PREFIX.as_bytes()),
                Address::SIZE as u32,
                Box::new(|x| x.contains_routing_info()),
            )
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
        let key = utils_db::db::Key::new_from_str(NEGLECTED_TICKETS_COUNT)?;

        Ok(self.db.get_or_none::<usize>(key).await?.unwrap_or(0))
    }

    async fn get_neglected_tickets_value(&self) -> Result<Balance> {
        let key = utils_db::db::Key::new_from_str(NEGLECTED_TICKETS_VALUE)?;

        Ok(self
            .db
            .get_or_none::<Balance>(key)
            .await?
            .unwrap_or(Balance::zero(BalanceType::HOPR)))
    }

    async fn get_losing_tickets_count(&self) -> Result<usize> {
        let key = utils_db::db::Key::new_from_str(LOSING_TICKET_COUNT)?;

        Ok(self.db.get_or_none::<usize>(key).await?.unwrap_or(0))
    }

    async fn mark_redeemed(&mut self, acked_ticket: &AcknowledgedTicket) -> Result<()> {
        debug!("start marking {acked_ticket} as redeemed");

        let mut ops = utils_db::db::Batch::default();

        let key = get_acknowledged_ticket_key(acked_ticket)?;
        ops.del(key);

        let key = utils_db::db::Key::new_from_str(REDEEMED_TICKETS_COUNT)?;
        let count = self.db.get_or_none::<usize>(key.clone()).await?.unwrap_or(0) + 1;

        ops.put(key, count);

        let key = utils_db::db::Key::new_from_str(REDEEMED_TICKETS_VALUE)?;
        let balance = self
            .db
            .get_or_none::<Balance>(key.clone())
            .await?
            .unwrap_or(Balance::zero(BalanceType::HOPR));

        let new_redeemed_balance = balance.add(&acked_ticket.ticket.amount);
        ops.put(key, new_redeemed_balance);
        self.db.batch(ops, true).await?;

        debug!("stopped marking {acked_ticket} as redeemed");

        Ok(())
    }

    async fn mark_losing_acked_ticket(&mut self, acked_ticket: &AcknowledgedTicket) -> Result<()> {
        debug!("marking {acked_ticket} as losing",);

        let mut ops = utils_db::db::Batch::default();

        let key = utils_db::db::Key::new_from_str(LOSING_TICKET_COUNT)?;
        let count = self.db.get_or_none::<usize>(key.clone()).await?.unwrap_or(0);
        ops.put(key, count + 1);

        let key = get_acknowledged_ticket_key(acked_ticket)?;
        ops.del(key);

        if let Some(counterparty) = self.get_channel(&acked_ticket.ticket.channel_id).await?.map(|c| {
            if c.source == self.me {
                c.destination
            } else {
                c.source
            }
        }) {
            let key = utils_db::db::Key::new_with_prefix(&counterparty, LOSING_TICKET_COUNT)?;
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
            );

            return Err(DbError::GenericError(format!(
                "channel {} not found",
                acked_ticket.ticket.channel_id
            )));
        }

        let current_unrealized_value = self
            .cached_unrealized_value
            .get(&acked_ticket.ticket.channel_id)
            .map(|v| v.sub(&acked_ticket.ticket.amount))
            .unwrap_or(Balance::zero(BalanceType::HOPR));

        self.cached_unrealized_value
            .insert(acked_ticket.ticket.channel_id, current_unrealized_value);

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
        //log::debug!("DB: get_channel_x src: {} & dest: {}", src, dest);
        let key = utils_db::db::Key::new_with_prefix(&generate_channel_id(src, dest), CHANNEL_PREFIX)?;
        self.db.get_or_none(key).await
    }

    async fn get_channels_from(&self, address: &Address) -> Result<Vec<ChannelEntry>> {
        Ok(self
            .db
            .get_more::<ChannelEntry>(
                Box::from(CHANNEL_PREFIX.as_bytes()),
                Hash::SIZE as u32,
                Box::new(|_| true),
            )
            .await?
            .into_iter()
            .filter(move |x| x.source.eq(address))
            .collect())
    }

    async fn get_outgoing_channels(&self) -> Result<Vec<ChannelEntry>> {
        Ok(self
            .db
            .get_more::<ChannelEntry>(
                Box::from(CHANNEL_PREFIX.as_bytes()),
                Hash::SIZE as u32,
                Box::new(|_| true),
            )
            .await?
            .into_iter()
            .filter(move |x| x.source.eq(&self.me))
            .collect())
    }

    async fn get_channels_to(&self, address: &Address) -> Result<Vec<ChannelEntry>> {
        Ok(self
            .db
            .get_more::<ChannelEntry>(
                Box::from(CHANNEL_PREFIX.as_bytes()),
                Hash::SIZE as u32,
                Box::new(|_| true),
            )
            .await?
            .into_iter()
            .filter(move |x| x.destination.eq(address))
            .collect())
    }

    async fn get_incoming_channels(&self) -> Result<Vec<ChannelEntry>> {
        Ok(self
            .db
            .get_more::<ChannelEntry>(
                Box::from(CHANNEL_PREFIX.as_bytes()),
                Hash::SIZE as u32,
                Box::new(|_| true),
            )
            .await?
            .into_iter()
            .filter(move |x| x.destination.eq(&self.me))
            .collect())
    }

    async fn get_hopr_balance(&self) -> Result<Balance> {
        //log::debug!("DB: get_hopr_balance");
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
        //log::debug!("DB: get_ticket_price");
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
            snapshot,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;
    use hopr_crypto::{
        keypairs::{ChainKeypair, Keypair},
        types::{Challenge, CurvePoint, HalfKey, Response},
    };
    use hopr_internal_types::channels::ChannelEntry;
    use hopr_primitive_types::{
        primitives::{Address, EthereumChallenge},
        traits::BinarySerializable,
    };
    use lazy_static::lazy_static;
    use std::str::FromStr;
    use utils_db::{db::serialize_to_bytes, CurrentDbShim};

    const ALICE: [u8; 32] = hex!("37eafd5038311f90fc08d13ff9ee16c6709be666e7d96808ba9a786c18f868a8");
    const BOB: [u8; 32] = hex!("d39a926980d6fa96a9eba8f8058b2beb774bc11866a386e9ddf9dc1152557c26");

    lazy_static! {
        static ref ALICE_KEYPAIR: ChainKeypair = ChainKeypair::from_secret(&ALICE).unwrap();
        static ref BOB_KEYPAIR: ChainKeypair = ChainKeypair::from_secret(&BOB).unwrap();
    }

    const PRICE_PER_PACKET: u128 = 10000000000000000_u128;
    const PATH_POS: u64 = 5_u64;

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
        let price_per_packet: U256 = PRICE_PER_PACKET.into(); // 0.01 HOPR

        Ticket::new(
            counterparty,
            &Balance::new(
                price_per_packet.divide_f64(win_prob).unwrap() * U256::from(PATH_POS),
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
    fn test_chain_db_iterable_type_ethereumchallenge_must_have_fixed_key_length() {
        let challenge = vec![10u8; EthereumChallenge::SIZE];
        let eth_challenge = EthereumChallenge::new(challenge.as_slice());

        let serialized = serialize_to_bytes(&eth_challenge);

        assert!(serialized.is_ok());
        assert_eq!(serialized.unwrap().len(), EthereumChallenge::SIZE)
    }

    #[test]
    fn test_chain_db_iterable_type_channelentry_must_have_fixed_key_length() {
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
        let mut db = CoreEthereumDb::new(DB::new(CurrentDbShim::new_in_memory().await), Address::random());

        assert_eq!(db.get_ticket_price().await, Ok(None));

        assert!(db.set_ticket_price(&U256::from(100u64)).await.is_ok());

        assert_eq!(db.get_ticket_price().await, Ok(Some(U256::from(100u64))));
    }

    #[async_std::test]
    async fn test_set_network_registry() {
        let mut db = CoreEthereumDb::new(DB::new(CurrentDbShim::new_in_memory().await), Address::random());

        assert_eq!(db.is_network_registry_enabled().await, Ok(true));

        assert!(db.set_network_registry(false, &Snapshot::default()).await.is_ok());

        assert_eq!(db.is_network_registry_enabled().await, Ok(false));
    }

    #[async_std::test]
    async fn test_allowed_to_access_network() {
        let mut db = CoreEthereumDb::new(DB::new(CurrentDbShim::new_in_memory().await), Address::random());

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
    async fn test_set_mfa() {
        let mut db = CoreEthereumDb::new(DB::new(CurrentDbShim::new_in_memory().await), Address::random());

        let test_address = Address::from_str("0xa6416794a09d1c8c4c6110f83f42cf6f1ed9c416").unwrap();

        db.set_mfa_protected_and_update_snapshot(Some(test_address), &Snapshot::default())
            .await
            .unwrap();

        assert_eq!(db.is_mfa_protected().await.unwrap(), Some(test_address));
    }

    async fn create_acknowledged_tickets(
        db: &mut CoreEthereumDb<CurrentDbShim>,
        tickets_to_generate: u64,
        channel_epoch: u32,
        start_index: u64,
    ) -> Vec<AcknowledgedTicket> {
        let mut hk1_seed: [u8; 32] = hex!("3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa");
        let mut hk2_seed: [u8; 32] = hex!("4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b");

        let mut acked_tickets: Vec<AcknowledgedTicket> = vec![];

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

        acked_tickets
    }

    #[async_std::test]
    async fn test_mark_mark_acknowledged_tickets_neglected() {
        let mut db = CoreEthereumDb::new(DB::new(CurrentDbShim::new_in_memory().await), Address::random());

        let start_index = 23u64;
        let tickets_to_generate = 3u64;
        let channel_epoch = 29u32;

        // set channel to current epoch
        let mut channel = ChannelEntry::new(
            ALICE_KEYPAIR.public().to_address(),
            BOB_KEYPAIR.public().to_address(),
            Balance::zero(BalanceType::HOPR),
            U256::zero(),
            ChannelStatus::Open,
            channel_epoch.into(),
            0_u32.into(),
        );

        db.update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        // create acked tickets of channel_epoch
        let acked_tickets = create_acknowledged_tickets(&mut db, tickets_to_generate, channel_epoch, start_index).await;

        // assert channel id
        let channel_id = generate_channel_id(&ALICE_KEYPAIR.public().to_address(), &BOB_KEYPAIR.public().to_address());
        assert_eq!(channel_id, acked_tickets[0].ticket.channel_id);
        assert_eq!(channel_id, channel.get_id());

        // check acked ticket count
        let acked_tickets_count = db.get_acknowledged_tickets_count(Some(channel.clone())).await.unwrap();

        assert_eq!(acked_tickets_count, tickets_to_generate as usize);

        // bump channel to next epoch
        channel.channel_epoch = (channel_epoch + 1).into();
        db.update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        // mark_acknowledged_tickets_neglected
        assert!(db.mark_acknowledged_tickets_neglected(channel).await.is_ok());

        // acked tickets should reduce to zero
        let acked_tickets_count_after_mark = db.get_acknowledged_tickets_count(None).await.unwrap();
        assert_eq!(acked_tickets_count_after_mark, 0usize);
    }

    #[async_std::test]
    async fn test_aggregatable_acknowledged_tickets() {
        let mut db = CoreEthereumDb::new(DB::new(CurrentDbShim::new_in_memory().await), Address::random());

        let start_index = 23u64;
        let tickets_to_generate = 3u64;

        let channel_epoch = 29u32;

        let acked_tickets = create_acknowledged_tickets(&mut db, tickets_to_generate, channel_epoch, start_index).await;

        let acked_tickets_from_db = db.get_acknowledged_tickets(None).await.unwrap();
        let acked_tickets_count = db.get_acknowledged_tickets_count(None).await.unwrap();
        let tickets_from_db = db.get_tickets(Some(ALICE_KEYPAIR.public().to_address())).await.unwrap();

        for i in 0usize..tickets_to_generate as usize {
            assert_eq!(acked_tickets[i], acked_tickets_from_db[i]);
            assert_eq!(tickets_from_db[i], acked_tickets[i].ticket);
        }

        assert_eq!(acked_tickets_count, acked_tickets.len());
        assert_eq!(acked_tickets_count, acked_tickets_from_db.len());
        assert_eq!(acked_tickets_count, tickets_from_db.len());

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

    async fn generate_ack_tickets(db: &mut DB<CurrentDbShim>, amount: u32) -> (Vec<AcknowledgedTicket>, ChannelEntry) {
        let mut challenge_seed: [u8; 32] = hex!("c04824c574e562b3b96725c8aa6e5b0426a3900cd9efbe48ddf7e754a552abdf");
        let domain_separator = Hash::default();

        let mut total_balance = Balance::zero(BalanceType::HOPR);
        let mut acked_tickets = Vec::new();
        for i in 0..amount {
            let challenge =
                Challenge::from(CurvePoint::from_exponent(&challenge_seed).unwrap()).to_ethereum_challenge();

            let ticket = mock_ticket(
                &ALICE_KEYPAIR,
                &(&*BOB_KEYPAIR).into(),
                Some(domain_separator),
                Some(i.into()),
                Some(1_u64.into()),
                Some(1_u64.into()),
                Some(challenge),
            );

            challenge_seed = Hash::create(&[&challenge_seed]).into();

            total_balance = total_balance.add(&ticket.amount);

            let ack = AcknowledgedTicket::new(
                ticket,
                Response::from_bytes(&challenge_seed.clone()).unwrap(),
                (&*ALICE_KEYPAIR).into(),
                &BOB_KEYPAIR,
                &domain_separator,
            )
            .unwrap();

            acked_tickets.push(ack);
        }

        let channel = ChannelEntry::new(
            ALICE_KEYPAIR.public().to_address(),
            BOB_KEYPAIR.public().to_address(),
            total_balance,
            amount.into(),
            ChannelStatus::Open,
            1_u32.into(),
            0_u32.into(),
        );

        let channel_key = utils_db::db::Key::new_with_prefix(&channel.get_id(), CHANNEL_PREFIX).unwrap();
        db.set(channel_key, &channel).await.unwrap();

        (acked_tickets, channel)
    }

    #[async_std::test]
    async fn test_should_prepare_all_acknowledged_tickets() {
        let mut inner_db = DB::new(CurrentDbShim::new_in_memory().await);

        let amount_tickets = 29;
        let (ack_tickets, channel) = generate_ack_tickets(&mut inner_db, amount_tickets).await;

        // Store ack tickets
        for ack in ack_tickets.iter() {
            inner_db
                .set(get_acknowledged_ticket_key(ack).unwrap(), ack)
                .await
                .unwrap();
        }

        let mut db = CoreEthereumDb::new(inner_db, BOB_KEYPAIR.public().to_address());
        let stored_acked_tickets = db
            .prepare_aggregatable_tickets(&channel.get_id(), 1u32, 0u64, u64::MAX)
            .await
            .unwrap();

        assert_eq!(stored_acked_tickets.len(), amount_tickets as usize);

        assert!(stored_acked_tickets
            .iter()
            .all(|acked_ticket| AcknowledgedTicketStatus::BeingAggregated {
                start: 0u64,
                end: u64::MAX,
            } == acked_ticket.status));
    }

    #[async_std::test]
    async fn test_should_prepare_acknowledged_tickets_skip_redeemed() {
        let mut inner_db = DB::new(CurrentDbShim::new_in_memory().await);

        let amount_tickets = 29;
        let (mut ack_tickets, channel) = generate_ack_tickets(&mut inner_db, amount_tickets).await;

        // Add some being redeemed tickets
        ack_tickets[12].status = AcknowledgedTicketStatus::BeingRedeemed {
            tx_hash: Hash::default(),
        };

        // Store ack tickets
        for ack in ack_tickets.iter() {
            inner_db
                .set(get_acknowledged_ticket_key(ack).unwrap(), ack)
                .await
                .unwrap();
        }

        let mut db = CoreEthereumDb::new(inner_db, BOB_KEYPAIR.public().to_address());
        let stored_acked_tickets = db
            .prepare_aggregatable_tickets(&channel.get_id(), 1u32, 0u64, u64::MAX)
            .await
            .unwrap();

        assert_eq!(stored_acked_tickets.len(), 16);

        assert!(stored_acked_tickets
            .iter()
            .all(|acked_ticket| AcknowledgedTicketStatus::BeingAggregated {
                start: 0u64,
                end: u64::MAX,
            } == acked_ticket.status));
    }

    #[async_std::test]
    async fn test_should_prepare_acknowledged_tickets_after_last_redeemed() {
        let mut inner_db = DB::new(CurrentDbShim::new_in_memory().await);

        let amount_tickets = 29;
        let (mut ack_tickets, channel) = generate_ack_tickets(&mut inner_db, amount_tickets).await;

        // Add some being redeemed tickets
        ack_tickets[12].status = AcknowledgedTicketStatus::BeingRedeemed {
            tx_hash: Hash::default(),
        };
        ack_tickets[17].status = AcknowledgedTicketStatus::BeingRedeemed {
            tx_hash: Hash::default(),
        };

        // Store ack tickets
        for ack in ack_tickets.iter() {
            inner_db
                .set(get_acknowledged_ticket_key(ack).unwrap(), ack)
                .await
                .unwrap();
        }

        let mut db = CoreEthereumDb::new(inner_db, BOB_KEYPAIR.public().to_address());
        let stored_acked_tickets = db
            .prepare_aggregatable_tickets(&channel.get_id(), 1u32, 0u64, u64::MAX)
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

    #[async_std::test]
    async fn test_should_not_prepare_when_last_being_redeemed() {
        let mut inner_db = DB::new(CurrentDbShim::new_in_memory().await);

        let amount_tickets = 29;
        let (mut ack_tickets, channel) = generate_ack_tickets(&mut inner_db, amount_tickets).await;

        // Add some being redeemed tickets
        ack_tickets[28].status = AcknowledgedTicketStatus::BeingRedeemed {
            tx_hash: Hash::default(),
        };

        // Store ack tickets
        for ack in ack_tickets.iter() {
            inner_db
                .set(get_acknowledged_ticket_key(ack).unwrap(), ack)
                .await
                .unwrap();
        }

        let mut db = CoreEthereumDb::new(inner_db, BOB_KEYPAIR.public().to_address());
        let stored_acked_tickets = db
            .prepare_aggregatable_tickets(&channel.get_id(), 1u32, 0u64, u64::MAX)
            .await
            .unwrap();

        assert!(stored_acked_tickets.is_empty());
    }

    #[async_std::test]
    async fn test_db_should_have_0_unrealized_balance_non_existing_channels() {
        let db = CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
            BOB_KEYPAIR.public().to_address(),
        );

        let channel = generate_channel_id(&ALICE_KEYPAIR.public().to_address(), &BOB_KEYPAIR.public().to_address());

        let unrealized_balance = db.get_unrealized_balance(&channel).await;
        assert_eq!(unrealized_balance, Ok(Balance::zero(BalanceType::HOPR)));
    }

    #[async_std::test]
    async fn test_db_should_contain_unrealized_balance_for_the_tickets_present() {
        let inner_db = DB::new(CurrentDbShim::new_in_memory().await);
        let mut db = CoreEthereumDb::new(inner_db, BOB_KEYPAIR.public().to_address());

        let tickets_to_generate = 2u64;
        let channel_epoch = 7u32;
        let start_index = 17u64;

        let expected_balance = Balance::new(U256::from(1_000_000_000_000_000_000u128), BalanceType::HOPR);

        let channel = ChannelEntry::new(
            ALICE_KEYPAIR.public().to_address(),
            BOB_KEYPAIR.public().to_address(),
            expected_balance,
            start_index.into(),
            ChannelStatus::Open,
            channel_epoch.into(),
            U256::from(1000u128),
        );

        db.update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        let acked_tickets = create_acknowledged_tickets(&mut db, tickets_to_generate, channel_epoch, start_index).await;

        let unrealized_balance = db.get_unrealized_balance(&acked_tickets[0].ticket.channel_id).await;

        let ticket_balance = acked_tickets
            .iter()
            .fold(Balance::zero(BalanceType::HOPR), |acc, n| acc.add(&n.ticket.amount));

        assert_eq!(unrealized_balance, Ok(expected_balance.sub(&ticket_balance)));
    }

    #[async_std::test]
    async fn test_db_should_reset_channel_balance_for_newly_opened_channels() {
        let inner_db = DB::new(CurrentDbShim::new_in_memory().await);
        let mut db = CoreEthereumDb::new(inner_db, BOB_KEYPAIR.public().to_address());

        let channel_epoch = 7u32;
        let start_index = 17u64;

        let expected_balance = Balance::new(U256::from(1_000_000_000_000_000_000u128), BalanceType::HOPR);

        let channel = ChannelEntry::new(
            ALICE_KEYPAIR.public().to_address(),
            BOB_KEYPAIR.public().to_address(),
            expected_balance,
            start_index.into(),
            ChannelStatus::Open,
            channel_epoch.into(),
            U256::from(1000u128),
        );

        db.update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        let unrealized_balance = db.get_unrealized_balance(&channel.get_id()).await;
        assert_eq!(unrealized_balance, Ok(expected_balance));
    }

    #[async_std::test]
    async fn test_db_should_reset_unrealized_channel_balance_for_reopened_channels_to_channel_balance() {
        let inner_db = DB::new(CurrentDbShim::new_in_memory().await);
        let mut db = CoreEthereumDb::new(inner_db, BOB_KEYPAIR.public().to_address());

        let channel_epoch = 7u32;
        let start_index = 17u64;

        let expected_balance = Balance::new(U256::from(1_000_000_000_000_000_000u128), BalanceType::HOPR);

        let channel = ChannelEntry::new(
            ALICE_KEYPAIR.public().to_address(),
            BOB_KEYPAIR.public().to_address(),
            expected_balance,
            start_index.into(),
            ChannelStatus::Open,
            channel_epoch.into(),
            U256::from(1000u128),
        );

        db.update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        // let _acked_tickets = create_acknowledged_tickets(&mut db, tickets_to_generate, channel_epoch, start_index).await;

        let newer_channel = ChannelEntry::new(
            ALICE_KEYPAIR.public().to_address(),
            BOB_KEYPAIR.public().to_address(),
            expected_balance,
            start_index.into(),
            ChannelStatus::Open,
            (channel_epoch + 1).into(),
            U256::from(1000u128),
        );

        db.update_channel_and_snapshot(&newer_channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();
        db.cleanup_invalid_channel_tickets(&newer_channel)
            .await
            .expect("Cleanup does not trigger failure");

        let unrealized_balance = db.get_unrealized_balance(&newer_channel.get_id()).await;
        assert_eq!(unrealized_balance, Ok(newer_channel.balance));
    }

    #[async_std::test]
    async fn test_db_should_move_the_outstanding_unrealized_value_to_unrealized_channel_balance_on_channel_update_with_the_same_channel_epoch_on_redeem(
    ) {
        let mut db = CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
            BOB_KEYPAIR.public().to_address(),
        );

        let tickets_to_generate = 2u64;
        let channel_epoch = 7u32;
        let start_index = 17u64;

        let expected_balance = Balance::new(U256::from(1_000_000_000_000_000_000u128), BalanceType::HOPR);

        let channel = ChannelEntry::new(
            ALICE_KEYPAIR.public().to_address(),
            BOB_KEYPAIR.public().to_address(),
            expected_balance,
            start_index.into(),
            ChannelStatus::Open,
            channel_epoch.into(),
            U256::from(1000u128),
        );

        db.update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        assert_eq!(
            db.get_unrealized_balance(&channel.get_id()).await.unwrap(),
            channel.balance
        );

        let mut acked_tickets =
            create_acknowledged_tickets(&mut db, tickets_to_generate, channel_epoch, start_index).await;

        // simulate the first ticket redeem, which would decrease the actual channel balance,
        // but keep the channel epoch the same as before
        let post_redeem_balance = expected_balance.sub(&acked_tickets[1].ticket.amount);

        let post_redeem_channel = ChannelEntry::new(
            ALICE_KEYPAIR.public().to_address(),
            BOB_KEYPAIR.public().to_address(),
            post_redeem_balance,
            start_index.into(),
            ChannelStatus::Open,
            channel_epoch.into(),
            U256::from(1000u128),
        );

        // redeem the ticket...
        acked_tickets.pop();

        db.update_channel_and_snapshot(
            &post_redeem_channel.get_id(),
            &post_redeem_channel,
            &Snapshot::default(),
        )
        .await
        .unwrap();

        let unrealized_balance = db.get_unrealized_balance(&channel.get_id()).await;
        let ticket_balance = acked_tickets
            .iter()
            .fold(Balance::zero(BalanceType::HOPR), |acc, n| acc.add(&n.ticket.amount));

        assert_eq!(unrealized_balance, Ok(post_redeem_balance.sub(&ticket_balance)));
    }

    #[async_std::test]
    async fn test_db_should_not_update_the_unrealized_balance_on_redeem() {
        let inner_db = DB::new(CurrentDbShim::new_in_memory().await);
        let mut db = CoreEthereumDb::new(inner_db, BOB_KEYPAIR.public().to_address());

        let tickets_to_generate = 2u64;
        let channel_epoch = 7u32;
        let start_index = 17u64;

        let channel_balance = Balance::new(U256::from(1_000_000_000_000_000_000u128), BalanceType::HOPR);

        let channel = ChannelEntry::new(
            ALICE_KEYPAIR.public().to_address(),
            BOB_KEYPAIR.public().to_address(),
            channel_balance,
            start_index.into(),
            ChannelStatus::Open,
            channel_epoch.into(),
            U256::from(1000u128),
        );

        db.update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        let mut acked_tickets =
            create_acknowledged_tickets(&mut db, tickets_to_generate, channel_epoch, start_index).await;

        let last_acked_ticket = acked_tickets.pop();

        assert!(last_acked_ticket.is_some());
        let last_acked_ticket = last_acked_ticket.unwrap();

        assert!(db.mark_redeemed(&last_acked_ticket).await.is_ok());

        let unrealized_balance = db.get_unrealized_balance(&acked_tickets[0].ticket.channel_id).await;
        let ticket_balance_without_redeemed = acked_tickets
            .iter()
            .fold(Balance::zero(BalanceType::HOPR), |acc, n| acc.add(&n.ticket.amount));

        assert_eq!(
            unrealized_balance,
            Ok(channel_balance
                .sub(&ticket_balance_without_redeemed)
                .sub(&last_acked_ticket.ticket.amount))
        );
    }

    #[async_std::test]
    async fn test_db_should_decrease_unrealized_balance_on_losing_ticket() {
        let inner_db = DB::new(CurrentDbShim::new_in_memory().await);
        let mut db = CoreEthereumDb::new(inner_db, BOB_KEYPAIR.public().to_address());

        let tickets_to_generate = 2u64;
        let channel_epoch = 7u32;
        let start_index = 17u64;

        let expected_balance = Balance::new(U256::from(1_000_000_000_000_000_000u128), BalanceType::HOPR);

        let channel = ChannelEntry::new(
            ALICE_KEYPAIR.public().to_address(),
            BOB_KEYPAIR.public().to_address(),
            expected_balance,
            start_index.into(),
            ChannelStatus::Open,
            channel_epoch.into(),
            U256::from(1000u128),
        );

        db.update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        let mut acked_tickets =
            create_acknowledged_tickets(&mut db, tickets_to_generate, channel_epoch, start_index).await;

        let last_acked_ticket = acked_tickets.pop();

        assert!(last_acked_ticket.is_some());

        assert!(db.mark_losing_acked_ticket(&last_acked_ticket.unwrap()).await.is_ok());

        let unrealized_balance = db.get_unrealized_balance(&acked_tickets[0].ticket.channel_id).await;
        assert!(unrealized_balance.is_ok());

        let ticket_balance = acked_tickets
            .iter()
            .fold(Balance::zero(BalanceType::HOPR), |acc, n| acc.add(&n.ticket.amount));

        assert_eq!(unrealized_balance.unwrap(), expected_balance.sub(&ticket_balance));
    }

    #[async_std::test]
    async fn test_db_should_initialize_catch_when_explicitly_triggered() {
        let _ = env_logger::builder().is_test(true).try_init();

        let mut inner_db = DB::new(CurrentDbShim::new_in_memory().await);
        // generate_ack_tickets only creates tickets of the current epoch but with smaller ticket indexes
        let (tickets, channel) = generate_ack_tickets(&mut inner_db, 1).await;

        // Store ack tickets
        for ack in tickets.iter() {
            inner_db
                .set(get_acknowledged_ticket_key(ack).unwrap(), ack)
                .await
                .unwrap();
        }

        let mut db = CoreEthereumDb::new(inner_db, BOB_KEYPAIR.public().to_address());

        let unrealized_balance = db.get_unrealized_balance(&channel.get_id()).await;
        assert_eq!(unrealized_balance, Ok(channel.balance));

        db.init_cache()
            .await
            .expect("should initialize cache without any issues");

        let unrealized_balance = db.get_unrealized_balance(&channel.get_id()).await;
        assert_eq!(unrealized_balance, Ok(channel.balance)); //
    }

    #[async_std::test]
    async fn test_db_should_initialize_catch_when_restarting_with_old_database_with_tickets_from_various_epoch_and_indexes(
    ) {
        let _ = env_logger::builder().is_test(true).try_init();

        let tickets_to_generate_per_epoch = 5u64;
        let start_index = 17u64;
        let current_channel_epoch = 7u32;
        let current_channel_ticket_index = 20u64;
        let current_channel_total_balance = Balance::new_from_str("1000000000000000000", BalanceType::HOPR); // 1 HOPR

        let inner_db = DB::new(CurrentDbShim::new_in_memory().await);
        let mut db = CoreEthereumDb::new(inner_db, BOB_KEYPAIR.public().to_address());
        let _tickets_from_previous_epoch = create_acknowledged_tickets(
            &mut db,
            tickets_to_generate_per_epoch,
            current_channel_epoch - 1,
            start_index,
        )
        .await;
        let tickets_from_current_epoch = create_acknowledged_tickets(
            &mut db,
            tickets_to_generate_per_epoch,
            current_channel_epoch,
            start_index,
        )
        .await;
        let _tickets_from_next_epoch = create_acknowledged_tickets(
            &mut db,
            tickets_to_generate_per_epoch,
            current_channel_epoch + 1,
            start_index,
        )
        .await;
        let ticket_balance = tickets_from_current_epoch[0].ticket.amount;

        let channel = ChannelEntry::new(
            ALICE_KEYPAIR.public().to_address(),
            BOB_KEYPAIR.public().to_address(),
            current_channel_total_balance,
            current_channel_ticket_index.into(),
            ChannelStatus::Open,
            current_channel_epoch.into(),
            0_u32.into(),
        );

        db.update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        let unrealized_balance = db.get_unrealized_balance(&channel.get_id()).await;
        assert_eq!(unrealized_balance, Ok(current_channel_total_balance));

        db.init_cache()
            .await
            .expect("should initialize cache without any issues");

        let unrealized_balance = db.get_unrealized_balance(&channel.get_id()).await;
        // Among all the 15 (3 epoch * 5 tickets3 epoch * 5 tickets) tickets, only 2 (start_index + tickets_to_generate_per_epoch - current_channel_ticket_index) tickets from the current epoch
        let cumulated_ticket_balance = ticket_balance.mul(&Balance::new(2_u32.into(), BalanceType::HOPR));
        assert_eq!(
            unrealized_balance,
            Ok(current_channel_total_balance.sub(&cumulated_ticket_balance))
        );
    }
}
