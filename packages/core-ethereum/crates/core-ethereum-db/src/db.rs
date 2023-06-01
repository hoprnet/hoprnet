use async_trait::async_trait;

use core_crypto::iterated_hash::Intermediate;
use core_crypto::{
    iterated_hash::IteratedHash,
    types::{HalfKeyChallenge, Hash, PublicKey},
};
use core_types::acknowledgement::{AcknowledgedTicket, PendingAcknowledgement, UnacknowledgedTicket};
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
use utils_types::primitives::{Address, AuthorizationToken, Balance, BalanceType, EthereumChallenge, Snapshot, U256};

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
    // core only part
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

    async fn get_tickets(&self, signer: Option<PublicKey>) -> Result<Vec<Ticket>> {
        let mut tickets = self
            .db
            .get_more::<AcknowledgedTicket>(
                Vec::from(ACKNOWLEDGED_TICKETS_PREFIX.as_bytes()).into_boxed_slice(),
                EthereumChallenge::size(),
                &|v: &AcknowledgedTicket| signer.clone().map(|s| v.signer.eq(&s)).unwrap_or(true),
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
                    PendingAcknowledgement::WaitingAsRelayer(unack) => signer.clone().map(|s| unack.signer.eq(&s)).unwrap_or(true)
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
        let count_key = utils_db::db::Key::new_from_str(REJECTED_TICKETS_COUNT)?;
        // always store as 2^32 - 1 options
        let count = self.db.get::<u128>(count_key.clone()).await?;
        self.db.set(count_key, &(count + 1)).await?;

        let value_key = utils_db::db::Key::new_from_str(REJECTED_TICKETS_VALUE)?;
        let balance = self
            .db
            .get::<Balance>(value_key.clone())
            .await
            .unwrap_or(Balance::new(U256::from(0u64), ticket.amount.balance_type()));

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

        let ack_key = utils_db::db::Key::new_bytes_with_prefix(&ack_key, ACKNOWLEDGED_TICKETS_PREFIX)?;

        let mut batch_ops = utils_db::db::Batch::new();
        batch_ops.del(unack_key);
        batch_ops.put(ack_key, ack_ticket);

        self.db.batch(batch_ops, true).await
    }

    // core and core-ethereum part
    async fn get_acknowledged_tickets(&self, filter: Option<ChannelEntry>) -> Result<Vec<AcknowledgedTicket>> {
        self.db
            .get_more::<AcknowledgedTicket>(
                Vec::from(ACKNOWLEDGED_TICKETS_PREFIX.as_bytes()).into_boxed_slice(),
                EthereumChallenge::size(),
                &|ack: &AcknowledgedTicket|
                    filter.clone().map(|f| {
                        f.destination.eq(&self.me) && ack.ticket.channel_epoch.eq(&f.channel_epoch)
                    }).unwrap_or(true),
            )
            .await

    }

    async fn get_unacknowledged_tickets(&self, filter: Option<ChannelEntry>) -> Result<Vec<UnacknowledgedTicket>> {
        Ok(self.db
            .get_more::<PendingAcknowledgement>(
                Vec::from(PENDING_ACKNOWLEDGEMENTS_PREFIX.as_bytes()).into_boxed_slice(),
                EthereumChallenge::size(),
                &|pending: &PendingAcknowledgement|
                    match pending {
                        PendingAcknowledgement::WaitingAsSender => false,
                        PendingAcknowledgement::WaitingAsRelayer(unack) => {
                            filter.clone().map(|f| {
                                f.destination.eq(&self.me) && unack.ticket.channel_epoch.eq(&f.channel_epoch)
                            }).unwrap_or(true)
                        }
                    }
            )
            .await?
            .into_iter()
            .filter_map(|a| match a {
                PendingAcknowledgement::WaitingAsSender => None,
                PendingAcknowledgement::WaitingAsRelayer(unack) => Some(unack),
            })
            .collect::<Vec<UnacknowledgedTicket>>())
    }

    async fn mark_pending(&mut self, ticket: &Ticket) -> Result<()> {
        let prefixed_key = utils_db::db::Key::new_with_prefix(&ticket.counterparty, PENDING_TICKETS_COUNT)?;
        let balance = self
            .db
            .get::<Balance>(prefixed_key.clone())
            .await
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

    async fn get_channel_to(&self, dest: &PublicKey) -> Result<Option<ChannelEntry>> {
        let key = utils_db::db::Key::new_with_prefix(
            &generate_channel_id(&self.me.to_address(), &dest.to_address()),
            CHANNEL_PREFIX,
        )?;

        self.db.get_or_none(key).await
    }

    async fn get_channel_from(&self, src: &PublicKey) -> Result<Option<ChannelEntry>> {
        let key = utils_db::db::Key::new_with_prefix(
            &generate_channel_id(&src.to_address(), &self.me.to_address()),
            CHANNEL_PREFIX,
        )?;

        self.db.get_or_none(key).await
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

    // core-ethereum only part
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

    async fn get_public_node_accounts(&self) -> Result<Vec<AccountEntry>> {
        self.db
            .get_more::<AccountEntry>(Box::from(ACCOUNT_PREFIX.as_bytes()), Address::size(), &|x| {
                x.contains_routing_info()
            })
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

    async fn resolve_pending(&mut self, address: &Address, balance: &Balance, snapshot: &Snapshot) -> Result<()> {
        let key = utils_db::db::Key::new_with_prefix(address, PENDING_TICKETS_COUNT)?;
        let current_balance = self
            .db
            .get_or_none(key.clone())
            .await?
            .unwrap_or(Balance::zero(BalanceType::HOPR));

        let mut batch_ops = utils_db::db::Batch::new();
        // TODO: NOTE: was there a bug in the original implementation in TS? val.sub(val)?
        batch_ops.put(key.clone(), &current_balance.sub(&balance));
        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            &snapshot,
        );

        self.db.batch(batch_ops, true).await
    }

    async fn mark_redeemed(&mut self, ticket: &AcknowledgedTicket) -> Result<()> {
        let key = utils_db::db::Key::new_from_str(REDEEMED_TICKETS_COUNT)?;
        let count = self.db.get_or_none::<usize>(key.clone()).await?.unwrap_or(0);
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
        let count = self.db.get_or_none::<usize>(key.clone()).await?.unwrap_or(0);
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

        self.db.get_or_none::<usize>(key).await.map(|v| v.unwrap_or(0))
    }

    async fn get_channel_x(&self, src: &PublicKey, dest: &PublicKey) -> Result<Option<ChannelEntry>> {
        let key = utils_db::db::Key::new_with_prefix(&generate_channel_id(&src.to_address(), &dest.to_address()), "")?;

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
        Ok(self.db.get_or_none::<bool>(key.clone()).await?.unwrap_or(true))
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

        self.db.get_or_none::<bool>(key).await.map(|v| v.unwrap_or(false))
    }

    async fn set_eligible(&mut self, account: &Address, eligible: bool, snapshot: &Snapshot) -> Result<()> {
        let key = utils_db::db::Key::new_with_prefix(account, NETWORK_REGISTRY_ADDRESS_ELIGIBLE_PREFIX)?;

        let mut batch_ops = utils_db::db::Batch::new();

        if eligible {
            batch_ops.put(key, &true);
        } else {
            batch_ops.del(key);
        }

        batch_ops.put(
            utils_db::db::Key::new_from_str(LATEST_CONFIRMED_SNAPSHOT_KEY)?,
            snapshot,
        );
        self.db.batch(batch_ops, true).await
    }

    async fn store_authorization(&mut self, token: AuthorizationToken) -> Result<()> {
        let tid = Hash::create(&[token.id().as_bytes()]);
        let key = utils_db::db::Key::new_with_prefix(&tid, API_AUTHORIZATION_TOKEN_KEY_PREFIX)?;
        let _ = self.db.set(key, &token).await?;
        Ok(())
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
    use super::{CoreEthereumDb, HoprCoreEthereumDbActions, PublicKey, DB};
    use core_crypto::iterated_hash::IteratedHash;
    use core_crypto::types::Hash;
    use core_types::account::AccountEntry;
    use core_types::acknowledgement::AcknowledgedTicket;
    use core_types::channels::{ChannelEntry, Ticket};
    use std::sync::Arc;
    use async_lock::RwLock;
    use utils_db::leveldb;
    use utils_types::primitives::{Address, AuthorizationToken, Balance, Snapshot};
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
    to_iterable!(WasmVecPublicKey, PublicKey);

    #[derive(Clone)]
    #[wasm_bindgen]
    pub struct Database {
        core_ethereum_db: Arc<RwLock<CoreEthereumDb<leveldb::LevelDbShim>>>,
    }

    impl Database {
        pub fn as_ref_counted(&self) -> Arc<RwLock<CoreEthereumDb<leveldb::LevelDbShim>>> {
            self.core_ethereum_db.clone()
        }
    }

    #[wasm_bindgen]
    impl Database {
        #[wasm_bindgen(constructor)]
        pub fn new(db: leveldb::LevelDb, public_key: PublicKey) -> Self {
            Self {
                core_ethereum_db: Arc::new(RwLock::new(CoreEthereumDb::<leveldb::LevelDbShim>::new(
                    DB::<leveldb::LevelDbShim>::new(leveldb::LevelDbShim::new(db)),
                    public_key.clone(),
                ))),
            }
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }
    }

    #[wasm_bindgen]
    impl Database {
        #[wasm_bindgen]
        pub async fn get_acknowledged_tickets(
            &self,
            filter: Option<ChannelEntry>,
        ) -> Result<WasmVecAcknowledgedTicket, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_acknowledged_tickets(filter).await)
                .map(|v| WasmVecAcknowledgedTicket::from(v))
        }

        #[wasm_bindgen]
        pub async fn delete_acknowledged_tickets_from(&mut self, source: ChannelEntry) -> Result<(), JsValue> {
            let mut db = self.core_ethereum_db.write().await;
            utils_misc::ok_or_jserr!(db.delete_acknowledged_tickets_from(source).await)
        }

        #[wasm_bindgen]
        pub async fn store_hash_intermediaries(
            &mut self,
            channel: &Hash,
            iterated_hash: JsValue,
        ) -> Result<(), JsValue> {
            let iterated: IteratedHash = utils_misc::ok_or_jserr!(serde_wasm_bindgen::from_value(iterated_hash))?;
            let mut db = self.core_ethereum_db.write().await;
            utils_misc::ok_or_jserr!(db.store_hash_intermediaries(channel, &iterated).await)
        }

        #[wasm_bindgen]
        pub async fn get_commitment(&self, channel: &Hash, iteration: usize) -> Result<Option<Hash>, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_commitment(channel, iteration).await)
        }

        #[wasm_bindgen]
        pub async fn get_current_commitment(&self, channel: &Hash) -> Result<Option<Hash>, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_current_commitment(channel).await)
        }

        #[wasm_bindgen]
        pub async fn set_current_commitment(&mut self, channel: &Hash, commitment: &Hash) -> Result<(), JsValue> {
            let mut db = self.core_ethereum_db.write().await;
            utils_misc::ok_or_jserr!(db.set_current_commitment(channel, commitment).await)
        }

        #[wasm_bindgen]
        pub async fn get_latest_block_number(&self) -> Result<u32, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_latest_block_number().await)
        }

        #[wasm_bindgen]
        pub async fn update_latest_block_number(&mut self, number: u32) -> Result<(), JsValue> {
            let mut db = self.core_ethereum_db.write().await;
            utils_misc::ok_or_jserr!(db.update_latest_block_number(number).await)
        }

        #[wasm_bindgen]
        pub async fn get_latest_confirmed_snapshot(&self) -> Result<Option<Snapshot>, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_latest_confirmed_snapshot().await)
        }

        #[wasm_bindgen]
        pub async fn get_channel(&self, channel: &Hash) -> Result<Option<ChannelEntry>, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_channel(channel).await)
        }

        #[wasm_bindgen]
        pub async fn get_channels(&self) -> Result<WasmVecChannelEntry, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_channels().await).map(|v| WasmVecChannelEntry::from(v))
        }

        pub async fn get_channels_open(&self) -> Result<WasmVecChannelEntry, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_channels_open().await).map(|v| WasmVecChannelEntry::from(v))
        }

        #[wasm_bindgen]
        pub async fn update_channel_and_snapshot(
            &mut self,
            channel_id: &Hash,
            channel: &ChannelEntry,
            snapshot: &Snapshot,
        ) -> Result<(), JsValue> {
            let mut db = self.core_ethereum_db.write().await;
            utils_misc::ok_or_jserr!(db.update_channel_and_snapshot(channel_id, channel, snapshot).await)
        }

        #[wasm_bindgen]
        pub async fn get_account(&self, address: &Address) -> Result<Option<AccountEntry>, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_account(address).await)
        }

        #[wasm_bindgen]
        pub async fn update_account_and_snapshot(
            &mut self,
            account: &AccountEntry,
            snapshot: &Snapshot,
        ) -> Result<(), JsValue> {
            let mut db = self.core_ethereum_db.write().await;
            utils_misc::ok_or_jserr!(db.update_account_and_snapshot(account, snapshot).await)
        }

        #[wasm_bindgen]
        pub async fn get_accounts(&self) -> Result<WasmVecAccountEntry, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_accounts().await).map(|v| WasmVecAccountEntry::from(v))
        }

        #[wasm_bindgen]
        pub async fn get_public_node_accounts(&self) -> Result<WasmVecAccountEntry, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_public_node_accounts().await).map(|v| WasmVecAccountEntry::from(v))
        }

        #[wasm_bindgen]
        pub async fn get_redeemed_tickets_value(&self) -> Result<Balance, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_redeemed_tickets_value().await)
        }

        #[wasm_bindgen]
        pub async fn get_redeemed_tickets_count(&self) -> Result<usize, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_redeemed_tickets_count().await)
        }

        #[wasm_bindgen]
        pub async fn get_neglected_tickets_count(&self) -> Result<usize, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_neglected_tickets_count().await)
        }

        #[wasm_bindgen]
        pub async fn get_pending_tickets_count(&self) -> Result<usize, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_pending_tickets_count().await)
        }

        #[wasm_bindgen]
        pub async fn get_losing_tickets_count(&self) -> Result<usize, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_losing_tickets_count().await)
        }

        #[wasm_bindgen]
        pub async fn get_pending_balance_to(&self, counterparty: &Address) -> Result<Balance, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_pending_balance_to(counterparty).await)
        }

        #[wasm_bindgen]
        pub async fn mark_pending(&mut self, ticket: &Ticket) -> Result<(), JsValue> {
            let mut db = self.core_ethereum_db.write().await;
            utils_misc::ok_or_jserr!(db.mark_pending(ticket).await)
        }

        #[wasm_bindgen]
        pub async fn resolve_pending(
            &mut self,
            address: &Address,
            balance: &Balance,
            snapshot: &Snapshot,
        ) -> Result<(), JsValue> {
            let mut db = self.core_ethereum_db.write().await;
            utils_misc::ok_or_jserr!(db.resolve_pending(address, balance, snapshot).await)
        }

        #[wasm_bindgen]
        pub async fn mark_redeemed(&mut self, ticket: &AcknowledgedTicket) -> Result<(), JsValue> {
            let mut db = self.core_ethereum_db.write().await;
            utils_misc::ok_or_jserr!(db.mark_redeemed(ticket).await)
        }

        /// NOTE: needed only for testing
        #[wasm_bindgen]
        pub async fn mark_rejected(&mut self, ticket: &Ticket) -> Result<(), JsValue> {
            let mut db = self.core_ethereum_db.write().await;
            utils_misc::ok_or_jserr!(db.mark_rejected(ticket).await)
        }

        #[wasm_bindgen]
        pub async fn mark_losing_acked_ticket(&mut self, ticket: &AcknowledgedTicket) -> Result<(), JsValue> {
            let mut db = self.core_ethereum_db.write().await;
            utils_misc::ok_or_jserr!(db.mark_losing_acked_ticket(ticket).await)
        }

        #[wasm_bindgen]
        pub async fn get_rejected_tickets_value(&self) -> Result<Balance, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_rejected_tickets_value().await)
        }

        #[wasm_bindgen]
        pub async fn get_rejected_tickets_count(&self) -> Result<usize, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_rejected_tickets_count().await)
        }

        #[wasm_bindgen]
        pub async fn get_channel_x(&self, src: &PublicKey, dest: &PublicKey) -> Result<Option<ChannelEntry>, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_channel_x(src, dest).await)
        }

        #[wasm_bindgen]
        pub async fn get_channel_to(&self, dest: &PublicKey) -> Result<Option<ChannelEntry>, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_channel_to(dest).await)
        }

        #[wasm_bindgen]
        pub async fn get_channel_from(&self, src: &PublicKey) -> Result<Option<ChannelEntry>, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_channel_from(src).await)
        }

        #[wasm_bindgen]
        pub async fn get_channels_from(&self, address: Address) -> Result<WasmVecChannelEntry, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_channels_from(address).await).map(|v| WasmVecChannelEntry::from(v))
        }

        #[wasm_bindgen]
        pub async fn get_channels_to(&self, address: Address) -> Result<WasmVecChannelEntry, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_channels_to(address).await).map(|v| WasmVecChannelEntry::from(v))
        }

        #[wasm_bindgen]
        pub async fn get_hopr_balance(&self) -> Result<Balance, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_hopr_balance().await)
        }

        #[wasm_bindgen]
        pub async fn set_hopr_balance(&mut self, balance: &Balance) -> Result<(), JsValue> {
            let mut db = self.core_ethereum_db.write().await;
            utils_misc::ok_or_jserr!(db.set_hopr_balance(balance).await)
        }

        #[wasm_bindgen]
        pub async fn add_hopr_balance(&mut self, balance: &Balance, snapshot: &Snapshot) -> Result<(), JsValue> {
            let mut db = self.core_ethereum_db.write().await;
            utils_misc::ok_or_jserr!(db.add_hopr_balance(balance, snapshot).await)
        }

        #[wasm_bindgen]
        pub async fn sub_hopr_balance(&mut self, balance: &Balance, snapshot: &Snapshot) -> Result<(), JsValue> {
            let mut db = self.core_ethereum_db.write().await;
            utils_misc::ok_or_jserr!(db.sub_hopr_balance(balance, snapshot).await)
        }

        #[wasm_bindgen]
        pub async fn is_network_registry_enabled(&self) -> Result<bool, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.is_network_registry_enabled().await)
        }

        #[wasm_bindgen]
        pub async fn set_network_registry(&mut self, enabled: bool, snapshot: &Snapshot) -> Result<(), JsValue> {
            let mut db = self.core_ethereum_db.write().await;
            utils_misc::ok_or_jserr!(db.set_network_registry(enabled, snapshot).await)
        }

        #[wasm_bindgen]
        pub async fn add_to_network_registry(
            &mut self,
            public_key: &PublicKey,
            account: &Address,
            snapshot: &Snapshot,
        ) -> Result<(), JsValue> {
            let mut db = self.core_ethereum_db.write().await;
            utils_misc::ok_or_jserr!(db.add_to_network_registry(public_key, account, snapshot).await)
        }

        #[wasm_bindgen]
        pub async fn remove_from_network_registry(
            &mut self,
            public_key: &PublicKey,
            account: &Address,
            snapshot: &Snapshot,
        ) -> Result<(), JsValue> {
            let mut db = self.core_ethereum_db.write().await;
            utils_misc::ok_or_jserr!(db.remove_from_network_registry(public_key, account, snapshot).await)
        }

        #[wasm_bindgen]
        pub async fn get_account_from_network_registry(
            &self,
            public_key: &PublicKey,
        ) -> Result<Option<Address>, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.get_account_from_network_registry(public_key).await)
        }

        #[wasm_bindgen]
        pub async fn find_hopr_node_using_account_in_network_registry(
            &self,
            account: &Address,
        ) -> Result<WasmVecPublicKey, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.find_hopr_node_using_account_in_network_registry(account).await)
                .map(|v| WasmVecPublicKey::from(v))
        }

        #[wasm_bindgen]
        pub async fn is_eligible(&self, account: &Address) -> Result<bool, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.is_eligible(account).await)
        }

        #[wasm_bindgen]
        pub async fn set_eligible(
            &mut self,
            account: &Address,
            eligible: bool,
            snapshot: &Snapshot,
        ) -> Result<(), JsValue> {
            let mut db = self.core_ethereum_db.write().await;
            utils_misc::ok_or_jserr!(db.set_eligible(account, eligible, snapshot).await)
        }

        #[wasm_bindgen]
        pub async fn store_authorization(&mut self, token: AuthorizationToken) -> Result<(), JsValue> {
            let mut db = self.core_ethereum_db.write().await;
            utils_misc::ok_or_jserr!(db.store_authorization(token).await)
        }

        #[wasm_bindgen]
        pub async fn retrieve_authorization(&self, id: String) -> Result<Option<AuthorizationToken>, JsValue> {
            let db = self.core_ethereum_db.read().await;
            utils_misc::ok_or_jserr!(db.retrieve_authorization(id).await)
        }

        #[wasm_bindgen]
        pub async fn delete_authorization(&mut self, id: String) -> Result<(), JsValue> {
            let mut db = self.core_ethereum_db.write().await;
            utils_misc::ok_or_jserr!(db.delete_authorization(id).await)
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
