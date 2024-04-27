use crate::errors::{CoreEthereumIndexerError, Result};
use async_trait::async_trait;
use bindings::{
    hopr_announcements::HoprAnnouncementsEvents, hopr_channels::HoprChannelsEvents,
    hopr_network_registry::HoprNetworkRegistryEvents, hopr_node_management_module::HoprNodeManagementModuleEvents,
    hopr_node_safe_registry::HoprNodeSafeRegistryEvents, hopr_ticket_price_oracle::HoprTicketPriceOracleEvents,
    hopr_token::HoprTokenEvents,
};
use chain_rpc::{BlockWithLogs, Log};
use chain_types::chain_events::{ChainEventType, NetworkRegistryStatus, SignificantChainEvent};
use chain_types::ContractAddresses;
use ethers::contract::EthLogDecode;
use hopr_crypto_types::keypairs::ChainKeypair;
use hopr_crypto_types::prelude::{Hash, Keypair};
use hopr_crypto_types::types::OffchainSignature;
use hopr_db_api::errors::DbError;
use hopr_db_api::info::DomainSeparator;
use hopr_db_api::tickets::TicketSelector;
use hopr_db_api::{HoprDbAllOperations, OpenTransaction};
use hopr_db_entity::channel;
use hopr_db_entity::conversions::channels::ChannelStatusUpdate;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};
use std::cmp::Ordering;
use std::fmt::Formatter;
use std::ops::{Add, Sub};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::{debug, error, info, trace, warn};

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::MultiCounter;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_INDEXER_LOG_COUNTERS: MultiCounter =
        MultiCounter::new(
            "hopr_indexer_contract_log_counters",
            "Counts of different HOPR contract logs processed by the Indexer",
            &["contract"]
    ).unwrap();
}

/// Event handling object for on-chain operations
///
/// Once an on-chain operation is recorded by the [crate::block::Indexer], it is pre-processed
/// and passed on to this object that handles event specific actions for each on-chain operation.
///
#[derive(Clone)]
pub struct ContractEventHandlers<Db: Clone> {
    /// channels, announcements, network_registry, token: contract addresses
    /// whose event we process
    addresses: Arc<ContractAddresses>,
    /// Safe on-chain address which we are monitoring
    safe_address: Address,
    /// own address, aka message sender
    chain_key: ChainKeypair, // TODO: store only address here once Ticket have TryFrom DB
    /// callbacks to inform other modules
    db: Db,
}

impl<Db: Clone> std::fmt::Debug for ContractEventHandlers<Db> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContractEventHandler")
            .field("addresses", &self.addresses)
            .field("safe_address", &self.safe_address)
            .field("chain_key", &self.chain_key)
            .finish_non_exhaustive()
    }
}

async fn channel_model_from_id(tx: &OpenTransaction, channel_id: Hash) -> Result<Option<channel::Model>> {
    Ok(channel::Entity::find()
        .filter(channel::Column::ChannelId.eq(channel_id.to_hex()))
        .one(tx.as_ref())
        .await?)
}

impl<Db> ContractEventHandlers<Db>
where
    Db: HoprDbAllOperations + Clone,
{
    pub fn new(addresses: ContractAddresses, safe_address: Address, chain_key: ChainKeypair, db: Db) -> Self {
        Self {
            addresses: Arc::new(addresses),
            safe_address,
            chain_key,
            db,
        }
    }

    async fn on_announcement_event(
        &self,
        tx: &OpenTransaction,
        event: HoprAnnouncementsEvents,
        block_number: u32,
    ) -> Result<Option<ChainEventType>> {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_INDEXER_LOG_COUNTERS.increment(&["announcements"]);

        match event {
            HoprAnnouncementsEvents::AddressAnnouncementFilter(address_announcement) => {
                trace!(
                    "on_announcement_event - multiaddr: {} - node: {}",
                    &address_announcement.base_multiaddr,
                    &address_announcement.node.to_string()
                );
                // safeguard against empty multiaddrs, skip
                if address_announcement.base_multiaddr.is_empty() {
                    warn!(
                        "encountered empty multiaddress announcement for account {}",
                        address_announcement.node
                    );
                    return Ok(None);
                }
                let node_address: Address = address_announcement.node.into();

                return match self
                    .db
                    .insert_announcement(
                        Some(tx),
                        node_address,
                        address_announcement.base_multiaddr.parse()?,
                        block_number,
                    )
                    .await
                {
                    Ok(account) => Ok(Some(ChainEventType::Announcement {
                        peer: account.public_key.into(),
                        address: account.chain_addr,
                        multiaddresses: vec![account.get_multiaddr().expect("not must contain multiaddr")],
                    })),
                    Err(DbError::MissingAccount) => Err(CoreEthereumIndexerError::AnnounceBeforeKeyBinding),
                    Err(e) => Err(e.into()),
                };
            }
            HoprAnnouncementsEvents::KeyBindingFilter(key_binding) => {
                match KeyBinding::from_parts(
                    key_binding.chain_key.into(),
                    key_binding.ed_25519_pub_key.try_into()?,
                    OffchainSignature::try_from((key_binding.ed_25519_sig_0, key_binding.ed_25519_sig_1))?,
                ) {
                    Ok(binding) => {
                        self.db
                            .insert_account(
                                Some(tx),
                                AccountEntry::new(binding.packet_key, binding.chain_key, AccountType::NotAnnounced),
                            )
                            .await?;
                    }
                    Err(_) => {
                        warn!(
                            "Filtering announcement from {} with invalid signature.",
                            key_binding.chain_key
                        )
                    }
                }
            }
            HoprAnnouncementsEvents::RevokeAnnouncementFilter(revocation) => {
                let node_address: Address = revocation.node.into();
                match self.db.delete_all_announcements(Some(tx), node_address).await {
                    Err(DbError::MissingAccount) => return Err(CoreEthereumIndexerError::RevocationBeforeKeyBinding),
                    Err(e) => return Err(e.into()),
                    _ => {}
                }
            }
        };

        Ok(None)
    }

    async fn on_channel_event(
        &self,
        tx: &OpenTransaction,
        event: HoprChannelsEvents,
    ) -> Result<Option<ChainEventType>> {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_INDEXER_LOG_COUNTERS.increment(&["channels"]);

        match event {
            HoprChannelsEvents::ChannelBalanceDecreasedFilter(balance_decreased) => {
                let maybe_channel = channel_model_from_id(tx, balance_decreased.channel_id.into()).await?;

                if let Some(channel) = maybe_channel {
                    let channel_entry: ChannelEntry = (&channel).try_into()?;
                    let new_balance = Balance::new(balance_decreased.new_balance, BalanceType::HOPR);
                    let diff = channel_entry.balance.sub(&new_balance);

                    let mut updated = channel.into_active_model();
                    updated.balance = Set(new_balance.amount().to_bytes().to_vec());
                    let channel = updated.update(tx.as_ref()).await?;

                    Ok(Some(ChainEventType::ChannelBalanceDecreased(channel.try_into()?, diff)))
                } else {
                    Err(CoreEthereumIndexerError::ChannelDoesNotExist)
                }
            }
            HoprChannelsEvents::ChannelBalanceIncreasedFilter(balance_increased) => {
                let maybe_channel = channel_model_from_id(tx, balance_increased.channel_id.into()).await?;

                if let Some(channel) = maybe_channel {
                    let channel_entry: ChannelEntry = (&channel).try_into()?;
                    let new_balance = Balance::new(balance_increased.new_balance, BalanceType::HOPR);
                    let diff = new_balance.sub(&channel_entry.balance);

                    let mut updated = channel.into_active_model();
                    updated.balance = Set(new_balance.amount().to_bytes().to_vec());
                    let channel = updated.update(tx.as_ref()).await?;

                    Ok(Some(ChainEventType::ChannelBalanceIncreased(channel.try_into()?, diff)))
                } else {
                    Err(CoreEthereumIndexerError::ChannelDoesNotExist)
                }
            }
            HoprChannelsEvents::ChannelClosedFilter(channel_closed) => {
                let maybe_channel = channel_model_from_id(tx, channel_closed.channel_id.into()).await?;

                trace!(
                    "on_channel_closed_event - channel_id: {:?} - channel known: {:?}",
                    channel_closed.channel_id,
                    maybe_channel.is_some()
                );

                if let Some(channel) = maybe_channel {
                    let channel_entry: ChannelEntry = (&channel).try_into()?;

                    // Incoming channel, so once closed. All unredeemed tickets just became invalid
                    if channel_entry.destination == self.chain_key.public().to_address() {
                        self.db.mark_tickets_neglected((&channel_entry).into()).await?;
                    }

                    // set all channel fields like we do on-chain on close
                    let mut active_channel = channel.into_active_model();
                    active_channel.balance = Set(BalanceType::HOPR.zero().amount().to_bytes().to_vec());
                    active_channel.ticket_index = Set(U256::zero().to_bytes().to_vec());
                    active_channel.set_status(ChannelStatus::Closed);
                    let channel = active_channel.update(tx.as_ref()).await?;

                    if channel_entry.source == self.chain_key.public().to_address()
                        || channel_entry.destination == self.chain_key.public().to_address()
                    {
                        // Reset the current_ticket_index to zero
                        self.db
                            .reset_outgoing_ticket_index(channel_closed.channel_id.into())
                            .await?;
                    }

                    Ok(Some(ChainEventType::ChannelClosed(channel.try_into()?)))
                } else {
                    Err(CoreEthereumIndexerError::ChannelDoesNotExist)
                }
            }
            HoprChannelsEvents::ChannelOpenedFilter(channel_opened) => {
                let source: Address = channel_opened.source.into();
                let destination: Address = channel_opened.destination.into();
                let channel_id = generate_channel_id(&source, &destination);

                let maybe_channel = channel_model_from_id(tx, channel_id).await?;
                let new_model = if let Some(channel) = maybe_channel {
                    // Check that we're not receiving the Open event without the channel being Close prior
                    if channel.status != u8::from(ChannelStatus::Closed) as i32 {
                        return Err(CoreEthereumIndexerError::ProcessError(format!(
                            "trying to re-open channel {} which is not closed",
                            channel.channel_id
                        )));
                    }

                    trace!(
                        "on_channel_reopened_event - source: {source} - destination: {destination} - channel_id: {channel_id}"
                    );

                    let current_epoch = U256::from_big_endian(&channel.epoch);

                    // cleanup tickets from previous epochs on channel re-opening
                    if source == self.chain_key.public().to_address()
                        || destination == self.chain_key.public().to_address()
                    {
                        self.db
                            .mark_tickets_neglected(TicketSelector::new(channel_id, current_epoch))
                            .await?;

                        self.db.reset_outgoing_ticket_index(channel_id).await?;
                    }

                    // set all channel fields like we do on-chain on close
                    let mut existing_channel = channel.into_active_model();
                    existing_channel.ticket_index = Set(U256::zero().to_be_bytes().into());
                    existing_channel.epoch = Set(current_epoch.add(1).to_be_bytes().into());
                    existing_channel.set_status(ChannelStatus::Open);
                    existing_channel.update(tx.as_ref()).await?
                } else {
                    trace!(
                        "on_channel_opened_event - source: {source} - destination: {destination} - channel_id: {channel_id}"
                    );

                    channel::ActiveModel {
                        channel_id: Set(channel_id.to_hex()),
                        source: Set(source.to_hex()),
                        destination: Set(destination.to_hex()),
                        balance: Set(U256::zero().to_be_bytes().into()),
                        status: Set(u8::from(ChannelStatus::Open) as i32),
                        epoch: Set(U256::one().to_be_bytes().into()),
                        ticket_index: Set(U256::zero().to_be_bytes().into()),
                        ..Default::default()
                    }
                    .insert(tx.as_ref())
                    .await?
                };

                Ok(Some(ChainEventType::ChannelOpened(new_model.try_into()?)))
            }
            HoprChannelsEvents::TicketRedeemedFilter(ticket_redeemed) => {
                let maybe_channel = channel_model_from_id(tx, ticket_redeemed.channel_id.into()).await?;

                if let Some(channel) = maybe_channel {
                    let channel_entry: ChannelEntry = (&channel).try_into()?;
                    let ack_ticket = match channel_entry.direction(&self.chain_key.public().to_address()) {
                        // For channels where destination is us, it means that our ticket
                        // has been redeemed, so mark it in the DB as redeemed
                        Some(ChannelDirection::Incoming) => {
                            // Filter all BeingRedeemed tickets in this channel and its current epoch
                            let mut matching_tickets = self
                                .db
                                .get_tickets(
                                    None,
                                    TicketSelector::from(&channel_entry)
                                        .with_state(AcknowledgedTicketStatus::BeingRedeemed),
                                )
                                .await?
                                .into_iter()
                                .filter(|ticket| {
                                    // The ticket that has been redeemed at this point has: index + index_offset - 1 == new_ticket_index - 1
                                    // Since unaggregated tickets have index_offset = 1, for the unagg case this leads to: index == new_ticket_index - 1
                                    let ticket_idx = ticket.ticket.index;
                                    let ticket_off = ticket.ticket.index_offset as u64;

                                    ticket_idx + ticket_off == ticket_redeemed.new_ticket_index
                                })
                                .collect::<Vec<_>>();

                            match matching_tickets.len().cmp(&1) {
                                Ordering::Equal => {
                                    let ack_ticket = matching_tickets.pop().unwrap();

                                    self.db.mark_tickets_redeemed((&ack_ticket).into()).await?;
                                    info!("{ack_ticket} has been marked as redeemed");
                                    Some(ack_ticket)
                                }
                                Ordering::Less => {
                                    error!(
                                        "could not find acknowledged 'BeingRedeemed' ticket with idx {} in {channel_entry}",
                                        ticket_redeemed.new_ticket_index - 1
                                    );
                                    // This is not an error, because the ticket might've become neglected before
                                    // the ticket redemption could finish
                                    None
                                }
                                Ordering::Greater => {
                                    error!(
                                        "found {} tickets matching 'BeingRedeemed' index {} in {channel_entry}",
                                        matching_tickets.len(),
                                        ticket_redeemed.new_ticket_index - 1
                                    );
                                    return Err(CoreEthereumIndexerError::ProcessError(format!(
                                        "multiple tickets matching idx {} found in {channel_entry}",
                                        ticket_redeemed.new_ticket_index - 1
                                    )));
                                }
                            }
                        }
                        // For channel where the source is us, it means a ticket that we
                        // issue has been redeemed, so we just need to be sure our outgoing ticket
                        // index value in the cache is at least the index of the redeemed ticket
                        Some(ChannelDirection::Outgoing) => {
                            // We need to ensure the outgoing ticket index is at least this new value
                            debug!("observed redeem event on an outgoing {channel_entry}");
                            self.db
                                .compare_and_set_outgoing_ticket_index(
                                    channel_entry.get_id(),
                                    ticket_redeemed.new_ticket_index,
                                )
                                .await?;
                            None
                        }
                        // For channel where neither source nor destination is us, we don't care
                        None => {
                            // Not our redeem event
                            debug!("observed redeem event on a foreign {channel_entry}");
                            None
                        }
                    };

                    // Update ticket index on the Channel entry and get the updated model
                    let mut active_channel = channel.into_active_model();
                    active_channel.ticket_index = Set(ticket_redeemed.new_ticket_index.to_be_bytes().into());
                    let channel = active_channel.update(tx.as_ref()).await?;

                    Ok(Some(ChainEventType::TicketRedeemed(channel.try_into()?, ack_ticket)))
                } else {
                    error!("observed ticket redeem on a channel that we don't have in the DB");
                    Err(CoreEthereumIndexerError::ChannelDoesNotExist)
                }
            }
            HoprChannelsEvents::OutgoingChannelClosureInitiatedFilter(closure_initiated) => {
                let maybe_channel = channel_model_from_id(tx, closure_initiated.channel_id.into()).await?;

                if let Some(channel) = maybe_channel {
                    let mut channel_entry: ChannelEntry = (&channel).try_into()?;
                    channel_entry.status = ChannelStatus::PendingToClose(
                        SystemTime::UNIX_EPOCH.add(Duration::from_secs(closure_initiated.closure_time as u64)),
                    );

                    let mut active_channel = channel.into_active_model();
                    active_channel.set_status(channel_entry.status);
                    let channel = active_channel.update(tx.as_ref()).await?;

                    Ok(Some(ChainEventType::ChannelClosureInitiated(channel.try_into()?)))
                } else {
                    Err(CoreEthereumIndexerError::ChannelDoesNotExist)
                }
            }
            HoprChannelsEvents::DomainSeparatorUpdatedFilter(domain_separator_updated) => {
                self.db
                    .set_domain_separator(
                        Some(tx),
                        DomainSeparator::Channel,
                        domain_separator_updated.domain_separator.into(),
                    )
                    .await?;

                Ok(None)
            }
            HoprChannelsEvents::LedgerDomainSeparatorUpdatedFilter(ledger_domain_separator_updated) => {
                self.db
                    .set_domain_separator(
                        Some(tx),
                        DomainSeparator::Ledger,
                        ledger_domain_separator_updated.ledger_domain_separator.into(),
                    )
                    .await?;

                Ok(None)
            }
        }
    }

    async fn on_token_event(&self, tx: &OpenTransaction, event: HoprTokenEvents) -> Result<Option<ChainEventType>> {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_INDEXER_LOG_COUNTERS.increment(&["token"]);

        match event {
            HoprTokenEvents::TransferFilter(transferred) => {
                let from: Address = transferred.from.into();
                let to: Address = transferred.to.into();

                trace!(
                    "on_token_transfer_event - address_to_monitor: {:?} - to: {to} - from: {from}",
                    &self.safe_address,
                );

                let mut current_balance = self.db.get_safe_hopr_balance(Some(tx)).await?;
                let transferred_value = transferred.value;

                if to.ne(&self.safe_address) && from.ne(&self.safe_address) {
                    return Ok(None);
                } else if to.eq(&self.safe_address) {
                    // This + is internally defined as saturating add
                    info!("Safe balance {current_balance} increased by {transferred_value}");
                    current_balance = current_balance + transferred_value;
                } else if from.eq(&self.safe_address) {
                    // This - is internally defined as saturating sub
                    info!("Safe balance {current_balance} decreased by {transferred_value}");
                    current_balance = current_balance - transferred_value;
                }

                self.db.set_safe_hopr_balance(Some(tx), current_balance).await?;
            }
            HoprTokenEvents::ApprovalFilter(approved) => {
                let owner: Address = approved.owner.into();
                let spender: Address = approved.spender.into();

                trace!(
                    "on_token_approval_event - address_to_monitor: {:?} - owner: {owner} - spender: {spender}, allowance: {:?}",
                    &self.safe_address, approved.value
                );

                // if approval is for tokens on Safe contract to be spent by HoprChannels
                if owner.eq(&self.safe_address) && spender.eq(&self.addresses.channels) {
                    self.db
                        .set_safe_hopr_allowance(Some(tx), BalanceType::HOPR.balance(approved.value))
                        .await?;
                } else {
                    return Ok(None);
                }
            }
            _ => error!("Implement all the other filters for HoprTokenEvents"),
        }

        Ok(None)
    }

    async fn on_network_registry_event(
        &self,
        tx: &OpenTransaction,
        event: HoprNetworkRegistryEvents,
    ) -> Result<Option<ChainEventType>> {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_INDEXER_LOG_COUNTERS.increment(&["network_registry"]);

        match event {
            HoprNetworkRegistryEvents::DeregisteredByManagerFilter(deregistered) => {
                let node_address: Address = deregistered.node_address.into();
                self.db
                    .set_access_in_network_registry(Some(tx), node_address, false)
                    .await?;

                return Ok(Some(ChainEventType::NetworkRegistryUpdate(
                    node_address,
                    NetworkRegistryStatus::Denied,
                )));
            }
            HoprNetworkRegistryEvents::DeregisteredFilter(deregistered) => {
                let node_address: Address = deregistered.node_address.into();
                self.db
                    .set_access_in_network_registry(Some(tx), node_address, false)
                    .await?;

                return Ok(Some(ChainEventType::NetworkRegistryUpdate(
                    node_address,
                    NetworkRegistryStatus::Denied,
                )));
            }
            HoprNetworkRegistryEvents::RegisteredByManagerFilter(registered) => {
                let node_address: Address = registered.node_address.into();
                self.db
                    .set_access_in_network_registry(Some(tx), node_address, true)
                    .await?;

                if node_address == self.chain_key.public().to_address() {
                    info!("Your node has been added to the registry and you can now continue the node activation process on http://hub.hoprnet.org/.");
                }

                return Ok(Some(ChainEventType::NetworkRegistryUpdate(
                    node_address,
                    NetworkRegistryStatus::Allowed,
                )));
            }
            HoprNetworkRegistryEvents::RegisteredFilter(registered) => {
                let node_address: Address = registered.node_address.into();
                self.db
                    .set_access_in_network_registry(Some(tx), node_address, true)
                    .await?;

                if node_address == self.chain_key.public().to_address() {
                    info!("Your node has been added to the registry and you can now continue the node activation process on http://hub.hoprnet.org/.");
                }

                return Ok(Some(ChainEventType::NetworkRegistryUpdate(
                    node_address,
                    NetworkRegistryStatus::Allowed,
                )));
            }
            HoprNetworkRegistryEvents::EligibilityUpdatedFilter(eligibility_updated) => {
                let account: Address = eligibility_updated.staking_account.into();
                self.db
                    .set_safe_eligibility(Some(tx), account, eligibility_updated.eligibility)
                    .await?;
            }
            HoprNetworkRegistryEvents::NetworkRegistryStatusUpdatedFilter(enabled) => {
                self.db
                    .set_network_registry_enabled(Some(tx), enabled.is_enabled)
                    .await?;
            }
            _ => {} // Not important to at the moment
        };

        Ok(None)
    }

    async fn on_node_safe_registry_event(
        &self,
        tx: &OpenTransaction,
        event: HoprNodeSafeRegistryEvents,
    ) -> Result<Option<ChainEventType>> {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_INDEXER_LOG_COUNTERS.increment(&["safe_registry"]);

        match event {
            HoprNodeSafeRegistryEvents::RegisteredNodeSafeFilter(registered) => {
                if self.chain_key.public().to_address() == registered.node_address.into() {
                    info!("Node safe registered: {}", registered.safe_address);
                    // NOTE: we don't store this state in the DB
                    return Ok(Some(ChainEventType::NodeSafeRegistered(registered.safe_address.into())));
                }
            }
            HoprNodeSafeRegistryEvents::DergisteredNodeSafeFilter(deregistered) => {
                if self.chain_key.public().to_address() == deregistered.node_address.into() {
                    info!("Node safe unregistered");
                    // NOTE: we don't store this state in the DB
                }
            }
            HoprNodeSafeRegistryEvents::DomainSeparatorUpdatedFilter(domain_separator_updated) => {
                self.db
                    .set_domain_separator(
                        Some(tx),
                        DomainSeparator::SafeRegistry,
                        domain_separator_updated.domain_separator.into(),
                    )
                    .await?;
            }
        }

        Ok(None)
    }

    async fn on_node_management_module_event(
        &self,
        _db: &OpenTransaction,
        _event: HoprNodeManagementModuleEvents,
    ) -> Result<Option<ChainEventType>> {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_INDEXER_LOG_COUNTERS.increment(&["node_management_module"]);

        // Don't care at the moment
        Ok(None)
    }

    async fn on_ticket_price_oracle_event(
        &self,
        tx: &OpenTransaction,
        event: HoprTicketPriceOracleEvents,
    ) -> Result<Option<ChainEventType>> {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_INDEXER_LOG_COUNTERS.increment(&["price_oracle"]);

        match event {
            HoprTicketPriceOracleEvents::TicketPriceUpdatedFilter(update) => {
                trace!(
                    "on_ticket_price_updated - old: {:?} - new: {:?}",
                    update.0.to_string(),
                    update.1.to_string()
                );

                self.db
                    .update_ticket_price(Some(tx), BalanceType::HOPR.balance(update.1))
                    .await?;

                info!("ticket price has been set to {}", update.1);
            }
            HoprTicketPriceOracleEvents::OwnershipTransferredFilter(_event) => {
                // ignore ownership transfer event
            }
        }
        Ok(None)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn process_log_event(&self, tx: &OpenTransaction, log: Log) -> Result<Option<ChainEventType>> {
        trace!("processing events in {log}");

        if log.address.eq(&self.addresses.announcements) {
            let bn = log.block_number as u32;
            let event = HoprAnnouncementsEvents::decode_log(&log.into())?;
            self.on_announcement_event(tx, event, bn).await
        } else if log.address.eq(&self.addresses.channels) {
            let event = HoprChannelsEvents::decode_log(&log.into())?;
            self.on_channel_event(tx, event).await
        } else if log.address.eq(&self.addresses.network_registry) {
            let event = HoprNetworkRegistryEvents::decode_log(&log.into())?;
            self.on_network_registry_event(tx, event).await
        } else if log.address.eq(&self.addresses.token) {
            let event = HoprTokenEvents::decode_log(&log.into())?;
            self.on_token_event(tx, event).await
        } else if log.address.eq(&self.addresses.safe_registry) {
            let event = HoprNodeSafeRegistryEvents::decode_log(&log.into())?;
            self.on_node_safe_registry_event(tx, event).await
        } else if log.address.eq(&self.addresses.module_implementation) {
            let event = HoprNodeManagementModuleEvents::decode_log(&log.into())?;
            self.on_node_management_module_event(tx, event).await
        } else if log.address.eq(&self.addresses.price_oracle) {
            let event = HoprTicketPriceOracleEvents::decode_log(&log.into())?;
            self.on_ticket_price_oracle_event(tx, event).await
        } else {
            #[cfg(all(feature = "prometheus", not(test)))]
            METRIC_INDEXER_LOG_COUNTERS.increment(&["unknown"]);

            error!(
                "on_event error - unknown contract address: {} - received log: {log:?}",
                log.address
            );
            return Err(CoreEthereumIndexerError::UnknownContract(log.address));
        }
    }
}

#[async_trait]
impl<Db> crate::traits::ChainLogHandler for ContractEventHandlers<Db>
where
    Db: HoprDbAllOperations + Clone + Send + Sync + 'static,
{
    fn contract_addresses(&self) -> Vec<Address> {
        vec![
            self.addresses.channels,
            self.addresses.token,
            self.addresses.network_registry,
            self.addresses.announcements,
            self.addresses.safe_registry,
            self.addresses.module_implementation,
            self.addresses.price_oracle,
        ]
    }

    async fn collect_block_events(&self, block_with_logs: BlockWithLogs) -> Result<Vec<SignificantChainEvent>> {
        let myself = self.clone();
        self.db
            .begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    // In the worst case, each log contains a single event
                    let mut ret = Vec::with_capacity(block_with_logs.logs.len());

                    let mut log_tx_hashes = Vec::with_capacity(block_with_logs.logs.len());
                    let block_id = block_with_logs.to_string();

                    // Process all logs in the block
                    for log in block_with_logs.logs {
                        let tx_hash = log.tx_hash;

                        // If a significant chain event can be extracted from the log, push it
                        if let Some(event_type) = myself.process_log_event(tx, log).await? {
                            let significant_event = SignificantChainEvent { tx_hash, event_type };
                            debug!("indexer got {significant_event}");
                            ret.push(significant_event);
                        }

                        log_tx_hashes.push(tx_hash);
                    }

                    // Update the hash only if any logs were processed in this block
                    let block_hash = if !log_tx_hashes.is_empty() {
                        debug!("block {block_id} has hashes {:?}", log_tx_hashes);
                        let h = Hash::create(
                            log_tx_hashes
                                .iter()
                                .map(|h| h.as_slice())
                                .collect::<Vec<_>>()
                                .as_slice(),
                        );
                        debug!("block hash of {block_id} is {h}");
                        Some(h)
                    } else {
                        None
                    };

                    // Once we're done with the block, update the DB
                    myself
                        .db
                        .set_last_indexed_block(Some(tx), block_with_logs.block_id as u32, block_hash)
                        .await?;
                    Ok(ret)
                })
            })
            .await
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::time::SystemTime;

    use super::ContractEventHandlers;
    use async_std;
    use bindings::{
        hopr_announcements::{AddressAnnouncementFilter, KeyBindingFilter, RevokeAnnouncementFilter},
        hopr_channels::{
            ChannelBalanceDecreasedFilter, ChannelBalanceIncreasedFilter, ChannelClosedFilter, ChannelOpenedFilter,
            DomainSeparatorUpdatedFilter, OutgoingChannelClosureInitiatedFilter, TicketRedeemedFilter,
        },
        hopr_network_registry::{
            DeregisteredByManagerFilter, DeregisteredFilter, EligibilityUpdatedFilter,
            NetworkRegistryStatusUpdatedFilter, RegisteredByManagerFilter, RegisteredFilter,
        },
        hopr_node_safe_registry::{DergisteredNodeSafeFilter, RegisteredNodeSafeFilter},
        hopr_ticket_price_oracle::TicketPriceUpdatedFilter,
        hopr_token::{ApprovalFilter, TransferFilter},
    };
    use chain_types::chain_events::{ChainEventType, NetworkRegistryStatus};
    use chain_types::ContractAddresses;
    use ethers::contract::EthEvent;
    use ethers::{
        abi::{encode, Address as EthereumAddress, Token},
        types::U256 as EthU256,
    };
    use hex_literal::hex;
    use hopr_crypto_types::prelude::*;
    use hopr_db_api::accounts::{ChainOrPacketKey, HoprDbAccountOperations};
    use hopr_db_api::channels::HoprDbChannelOperations;
    use hopr_db_api::db::HoprDb;
    use hopr_db_api::info::{DomainSeparator, HoprDbInfoOperations};
    use hopr_db_api::prelude::HoprDbResolverOperations;
    use hopr_db_api::registry::HoprDbRegistryOperations;
    use hopr_db_api::tickets::HoprDbTicketOperations;
    use hopr_db_api::{HoprDbAllOperations, HoprDbGeneralModelOperations};
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use multiaddr::Multiaddr;
    use primitive_types::H256;

    lazy_static::lazy_static! {
        static ref SELF_PRIV_KEY: OffchainKeypair = OffchainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).unwrap();
        static ref COUNTERPARTY_CHAIN_KEY: ChainKeypair = ChainKeypair::random();
        static ref COUNTERPARTY_CHAIN_ADDRESS: Address = COUNTERPARTY_CHAIN_KEY.public().to_address();
        static ref SELF_CHAIN_KEY: ChainKeypair = ChainKeypair::random();
        static ref SELF_CHAIN_ADDRESS: Address = SELF_CHAIN_KEY.public().to_address();
        static ref STAKE_ADDRESS: Address = Address::from_bytes(&hex!("4331eaa9542b6b034c43090d9ec1c2198758dbc3")).unwrap();
        static ref CHANNELS_ADDR: Address = Address::from_bytes(&hex!("bab20aea98368220baa4e3b7f151273ee71df93b")).unwrap(); // just a dummy
        static ref TOKEN_ADDR: Address = Address::from_bytes(&hex!("47d1677e018e79dcdd8a9c554466cb1556fa5007")).unwrap(); // just a dummy
        static ref NETWORK_REGISTRY_ADDR: Address = Address::from_bytes(&hex!("a469d0225f884fb989cbad4fe289f6fd2fb98051")).unwrap(); // just a dummy
        static ref NODE_SAFE_REGISTRY_ADDR: Address = Address::from_bytes(&hex!("0dcd1bf9a1b36ce34237eeafef220932846bcd82")).unwrap(); // just a dummy
        static ref ANNOUNCEMENTS_ADDR: Address = Address::from_bytes(&hex!("11db4791bf45ef31a10ea4a1b5cb90f46cc72c7e")).unwrap(); // just a dummy
        static ref SAFE_MANAGEMENT_MODULE_ADDR: Address = Address::from_bytes(&hex!("9b91245a65ad469163a86e32b2281af7a25f38ce")).unwrap(); // just a dummy
        static ref SAFE_INSTANCE_ADDR: Address = Address::from_bytes(&hex!("b93d7fdd605fb64fdcc87f21590f950170719d47")).unwrap(); // just a dummy
        static ref TICKET_PRICE_ORACLE_ADDR: Address = Address::from_bytes(&hex!("11db4391bf45ef31a10ea4a1b5cb90f46cc72c7e")).unwrap(); // just a dummy
    }

    async fn create_db() -> HoprDb {
        HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await
    }

    fn init_handlers<Db: HoprDbAllOperations + Clone>(db: Db) -> ContractEventHandlers<Db> {
        ContractEventHandlers {
            addresses: Arc::new(ContractAddresses {
                channels: *CHANNELS_ADDR,
                token: *TOKEN_ADDR,
                network_registry: *NETWORK_REGISTRY_ADDR,
                network_registry_proxy: Default::default(),
                safe_registry: *NODE_SAFE_REGISTRY_ADDR,
                announcements: *ANNOUNCEMENTS_ADDR,
                module_implementation: *SAFE_MANAGEMENT_MODULE_ADDR,
                price_oracle: *TICKET_PRICE_ORACLE_ADDR,
                stake_factory: Default::default(),
            }),
            chain_key: SELF_CHAIN_KEY.clone(),
            safe_address: SELF_CHAIN_KEY.public().to_address(),
            db,
        }
    }

    fn test_log() -> ethers::prelude::Log {
        ethers::prelude::Log {
            address: Default::default(),
            topics: vec![],
            data: Default::default(),
            block_hash: Some(H256::zero()),
            block_number: Some(0.into()),
            transaction_hash: Some(H256::zero()),
            transaction_index: Some(0.into()),
            log_index: Some(0.into()),
            transaction_log_index: None,
            log_type: None,
            removed: None,
        }
    }

    #[async_std::test]
    async fn announce_keybinding() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let keybinding = KeyBinding::new(*SELF_CHAIN_ADDRESS, &SELF_PRIV_KEY);

        let keybinding_log = ethers::prelude::Log {
            address: handlers.addresses.announcements.into(),
            topics: vec![KeyBindingFilter::signature()],
            data: encode(&[
                Token::FixedBytes(Vec::from(keybinding.signature.to_bytes())),
                Token::FixedBytes(Vec::from(keybinding.packet_key.to_bytes())),
                Token::Address(EthereumAddress::from_slice(
                    &SELF_CHAIN_KEY.public().to_address().to_bytes(),
                )),
            ])
            .into(),
            ..test_log()
        };

        let account_entry = AccountEntry::new(*SELF_PRIV_KEY.public(), *SELF_CHAIN_ADDRESS, AccountType::NotAnnounced);

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, keybinding_log.into()).await }))
            .await
            .unwrap();

        assert!(event_type.is_none(), "keybinding does not have a chain event type");

        assert_eq!(
            db.get_account(None, ChainOrPacketKey::ChainKey(*SELF_CHAIN_ADDRESS))
                .await
                .unwrap()
                .unwrap(),
            account_entry
        );
    }

    #[async_std::test]
    async fn announce_address_announcement() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        // Assume that there is a keybinding
        let account_entry = AccountEntry::new(*SELF_PRIV_KEY.public(), *SELF_CHAIN_ADDRESS, AccountType::NotAnnounced);
        db.insert_account(None, account_entry.clone()).await.unwrap();

        let test_multiaddr_empty: Multiaddr = "".parse().unwrap();

        let address_announcement_empty_log = ethers::prelude::Log {
            address: handlers.addresses.announcements.into(),
            topics: vec![AddressAnnouncementFilter::signature()],
            data: encode(&[
                Token::Address(EthereumAddress::from_slice(&SELF_CHAIN_ADDRESS.to_bytes())),
                Token::String(test_multiaddr_empty.to_string()),
            ])
            .into(),
            ..test_log()
        };

        let handlers_clone = handlers.clone();
        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| {
                Box::pin(async move {
                    handlers_clone
                        .process_log_event(tx, address_announcement_empty_log.into())
                        .await
                })
            })
            .await
            .unwrap();

        assert!(
            event_type.is_none(),
            "announcement of empty multiaddresses must pass through"
        );

        assert_eq!(
            db.get_account(None, ChainOrPacketKey::ChainKey(*SELF_CHAIN_ADDRESS))
                .await
                .unwrap()
                .unwrap(),
            account_entry
        );

        let test_multiaddr: Multiaddr = "/ip4/1.2.3.4/tcp/56".parse().unwrap();

        let address_announcement_log = ethers::prelude::Log {
            address: handlers.addresses.announcements.into(),
            block_number: Some(1.into()),
            topics: vec![AddressAnnouncementFilter::signature()],
            data: encode(&[
                Token::Address(EthereumAddress::from_slice(&SELF_CHAIN_ADDRESS.to_bytes())),
                Token::String(test_multiaddr.to_string()),
            ])
            .into(),
            ..test_log()
        };

        let announced_account_entry = AccountEntry::new(
            *SELF_PRIV_KEY.public(),
            *SELF_CHAIN_ADDRESS,
            AccountType::Announced {
                multiaddr: test_multiaddr.clone(),
                updated_block: 1,
            },
        );

        let handlers_clone = handlers.clone();
        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| {
                Box::pin(async move {
                    handlers_clone
                        .process_log_event(tx, address_announcement_log.into())
                        .await
                })
            })
            .await
            .unwrap();

        assert!(
            matches!(event_type, Some(ChainEventType::Announcement { multiaddresses,.. }) if multiaddresses == vec![test_multiaddr]),
            "must return the latest announce multiaddress"
        );

        assert_eq!(
            db.get_account(None, ChainOrPacketKey::ChainKey(*SELF_CHAIN_ADDRESS))
                .await
                .unwrap()
                .unwrap(),
            announced_account_entry
        );

        assert_eq!(
            Some(*SELF_CHAIN_ADDRESS),
            db.resolve_chain_key(SELF_PRIV_KEY.public()).await.unwrap(),
            "must resolve correct chain key"
        );

        assert_eq!(
            Some(*SELF_PRIV_KEY.public()),
            db.resolve_packet_key(&SELF_CHAIN_ADDRESS).await.unwrap(),
            "must resolve correct packet key"
        );

        let test_multiaddr_dns: Multiaddr = "/dns4/useful.domain/tcp/56".parse().unwrap();

        let address_announcement_dns_log = ethers::prelude::Log {
            address: handlers.addresses.announcements.into(),
            block_number: Some(2.into()),
            topics: vec![AddressAnnouncementFilter::signature()],
            data: encode(&[
                Token::Address(EthereumAddress::from_slice(&SELF_CHAIN_ADDRESS.to_bytes())),
                Token::String(test_multiaddr_dns.to_string()),
            ])
            .into(),
            ..test_log()
        };

        let announced_dns_account_entry = AccountEntry::new(
            *SELF_PRIV_KEY.public(),
            *SELF_CHAIN_ADDRESS,
            AccountType::Announced {
                multiaddr: test_multiaddr_dns.clone(),
                updated_block: 2,
            },
        );

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| {
                Box::pin(async move {
                    handlers
                        .process_log_event(tx, address_announcement_dns_log.into())
                        .await
                })
            })
            .await
            .unwrap();

        assert!(
            matches!(event_type, Some(ChainEventType::Announcement { multiaddresses,.. }) if multiaddresses == vec![test_multiaddr_dns]),
            "must return the latest announce multiaddress"
        );

        assert_eq!(
            db.get_account(None, ChainOrPacketKey::ChainKey(*SELF_CHAIN_ADDRESS))
                .await
                .unwrap()
                .unwrap(),
            announced_dns_account_entry
        );

        assert_eq!(
            Some(*SELF_CHAIN_ADDRESS),
            db.resolve_chain_key(SELF_PRIV_KEY.public()).await.unwrap(),
            "must resolve correct chain key"
        );

        assert_eq!(
            Some(*SELF_PRIV_KEY.public()),
            db.resolve_packet_key(&SELF_CHAIN_ADDRESS).await.unwrap(),
            "must resolve correct packet key"
        );
    }

    #[async_std::test]
    async fn announce_revoke() {
        let db = create_db().await;
        let handlers = init_handlers(db.clone());

        let test_multiaddr: Multiaddr = "/ip4/1.2.3.4/tcp/56".parse().unwrap();

        // Assume that there is a keybinding and an address announcement
        let announced_account_entry = AccountEntry::new(
            *SELF_PRIV_KEY.public(),
            *SELF_CHAIN_ADDRESS,
            AccountType::Announced {
                multiaddr: test_multiaddr,
                updated_block: 0,
            },
        );
        db.insert_account(None, announced_account_entry).await.unwrap();

        let revoke_announcement_log = ethers::prelude::Log {
            address: handlers.addresses.announcements.into(),
            topics: vec![RevokeAnnouncementFilter::signature()],
            data: encode(&[Token::Address(EthereumAddress::from_slice(
                &SELF_CHAIN_ADDRESS.to_bytes(),
            ))])
            .into(),
            ..test_log()
        };

        let account_entry = AccountEntry::new(*SELF_PRIV_KEY.public(), *SELF_CHAIN_ADDRESS, AccountType::NotAnnounced);

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, revoke_announcement_log.into()).await }))
            .await
            .unwrap();

        assert!(
            event_type.is_none(),
            "revoke announcement does not have chain event type"
        );

        assert_eq!(
            db.get_account(None, ChainOrPacketKey::ChainKey(*SELF_CHAIN_ADDRESS))
                .await
                .unwrap()
                .unwrap(),
            account_entry
        );
    }

    #[async_std::test]
    async fn on_token_transfer_to() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let value = U256::max_value();

        let transferred_log = ethers::prelude::Log {
            address: handlers.addresses.token.into(),
            topics: vec![
                TransferFilter::signature(),
                H256::from_slice(&Address::default().to_bytes32()),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[Token::Uint(EthU256::from_big_endian(&value.to_bytes()))]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, transferred_log.into()).await }))
            .await
            .unwrap();

        assert!(event_type.is_none(), "token transfer does not have chain event type");

        assert_eq!(
            db.get_safe_hopr_balance(None).await.unwrap(),
            Balance::new(value, BalanceType::HOPR)
        )
    }

    #[async_std::test]
    async fn on_token_transfer_from() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let value = U256::max_value();

        db.set_safe_hopr_balance(None, BalanceType::HOPR.balance(value))
            .await
            .unwrap();

        let transferred_log = ethers::prelude::Log {
            address: handlers.addresses.token.into(),
            topics: vec![
                TransferFilter::signature(),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
                H256::from_slice(&Address::default().to_bytes32()),
            ],
            data: encode(&[Token::Uint(EthU256::from_big_endian(&value.to_bytes()))]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, transferred_log.into()).await }))
            .await
            .unwrap();

        assert!(event_type.is_none(), "token transfer does not have chain event type");

        assert_eq!(db.get_safe_hopr_balance(None).await.unwrap(), BalanceType::HOPR.zero())
    }

    #[async_std::test]
    async fn on_token_approval_correct() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let approval_log = ethers::prelude::Log {
            address: handlers.addresses.token.into(),
            topics: vec![
                ApprovalFilter::signature(),
                H256::from_slice(&handlers.safe_address.to_bytes32()),
                H256::from_slice(&handlers.addresses.channels.to_bytes32()),
            ],
            data: encode(&[Token::Uint(EthU256::from(1000u64))]).into(),
            ..test_log()
        };

        assert_eq!(
            db.get_safe_hopr_allowance(None).await.unwrap(),
            Balance::new(U256::from(0u64), BalanceType::HOPR)
        );

        let approval_log_clone = approval_log.clone();
        let handlers_clone = handlers.clone();
        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| {
                Box::pin(async move { handlers_clone.process_log_event(tx, approval_log_clone.into()).await })
            })
            .await
            .unwrap();

        assert!(event_type.is_none(), "token approval does not have chain event type");

        assert_eq!(
            db.get_safe_hopr_allowance(None).await.unwrap(),
            Balance::new(U256::from(1000u64), BalanceType::HOPR)
        );

        // reduce allowance manually to verify a second time
        let _ = db
            .set_safe_hopr_allowance(None, Balance::new(U256::from(10u64), BalanceType::HOPR))
            .await;
        assert_eq!(
            db.get_safe_hopr_allowance(None).await.unwrap(),
            Balance::new(U256::from(10u64), BalanceType::HOPR)
        );

        let handlers_clone = handlers.clone();
        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers_clone.process_log_event(tx, approval_log.into()).await }))
            .await
            .unwrap();

        assert!(event_type.is_none(), "token approval does not have chain event type");

        assert_eq!(
            db.get_safe_hopr_allowance(None).await.unwrap(),
            Balance::new(U256::from(1000u64), BalanceType::HOPR)
        );
    }

    #[async_std::test]
    async fn on_network_registry_event_registered() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let registered_log = ethers::prelude::Log {
            address: handlers.addresses.network_registry.into(),
            topics: vec![
                RegisteredFilter::signature(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        assert!(!db
            .is_allowed_in_network_registry(None, *SELF_CHAIN_ADDRESS)
            .await
            .unwrap());

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, registered_log.into()).await }))
            .await
            .unwrap();

        assert!(
            matches!(event_type, Some(ChainEventType::NetworkRegistryUpdate(a, s)) if a == *SELF_CHAIN_ADDRESS && s == NetworkRegistryStatus::Allowed),
            "must return correct NR update"
        );

        assert!(
            db.is_allowed_in_network_registry(None, *SELF_CHAIN_ADDRESS)
                .await
                .unwrap(),
            "must be allowed in NR"
        );
    }

    #[async_std::test]
    async fn on_network_registry_event_registered_by_manager() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let registered_log = ethers::prelude::Log {
            address: handlers.addresses.network_registry.into(),
            topics: vec![
                RegisteredByManagerFilter::signature(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        assert!(!db
            .is_allowed_in_network_registry(None, *SELF_CHAIN_ADDRESS)
            .await
            .unwrap());

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, registered_log.into()).await }))
            .await
            .unwrap();

        assert!(
            matches!(event_type, Some(ChainEventType::NetworkRegistryUpdate(a, s)) if a == *SELF_CHAIN_ADDRESS && s == NetworkRegistryStatus::Allowed),
            "must return correct NR update"
        );

        assert!(
            db.is_allowed_in_network_registry(None, *SELF_CHAIN_ADDRESS)
                .await
                .unwrap(),
            "must be allowed in NR"
        );
    }

    #[async_std::test]
    async fn on_network_registry_event_deregistered() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        db.set_access_in_network_registry(None, *SELF_CHAIN_ADDRESS, true)
            .await
            .unwrap();

        let registered_log = ethers::prelude::Log {
            address: handlers.addresses.network_registry.into(),
            topics: vec![
                DeregisteredFilter::signature(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        assert!(db
            .is_allowed_in_network_registry(None, *SELF_CHAIN_ADDRESS)
            .await
            .unwrap());

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, registered_log.into()).await }))
            .await
            .unwrap();

        assert!(
            matches!(event_type, Some(ChainEventType::NetworkRegistryUpdate(a, s)) if a == *SELF_CHAIN_ADDRESS && s == NetworkRegistryStatus::Denied),
            "must return correct NR update"
        );

        assert!(
            !db.is_allowed_in_network_registry(None, *SELF_CHAIN_ADDRESS)
                .await
                .unwrap(),
            "must not be allowed in NR"
        );
    }

    #[async_std::test]
    async fn on_network_registry_event_deregistered_by_manager() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        db.set_access_in_network_registry(None, *SELF_CHAIN_ADDRESS, true)
            .await
            .unwrap();

        let registered_log = ethers::prelude::Log {
            address: handlers.addresses.network_registry.into(),
            topics: vec![
                DeregisteredByManagerFilter::signature(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        assert!(db
            .is_allowed_in_network_registry(None, *SELF_CHAIN_ADDRESS)
            .await
            .unwrap());

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, registered_log.into()).await }))
            .await
            .unwrap();

        assert!(
            matches!(event_type, Some(ChainEventType::NetworkRegistryUpdate(a, s)) if a == *SELF_CHAIN_ADDRESS && s == NetworkRegistryStatus::Denied),
            "must return correct NR update"
        );

        assert!(
            !db.is_allowed_in_network_registry(None, *SELF_CHAIN_ADDRESS)
                .await
                .unwrap(),
            "must not be allowed in NR"
        );
    }

    #[async_std::test]
    async fn on_network_registry_event_enabled() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let nr_enabled = ethers::prelude::Log {
            address: handlers.addresses.network_registry.into(),
            topics: vec![
                NetworkRegistryStatusUpdatedFilter::signature(),
                H256::from_low_u64_be(1),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, nr_enabled.into()).await }))
            .await
            .unwrap();

        assert!(event_type.is_none(), "there's no chain event type for nr disable");

        assert!(db.get_indexer_data(None).await.unwrap().nr_enabled);
    }

    #[async_std::test]
    async fn on_network_registry_event_disabled() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        db.set_network_registry_enabled(None, true).await.unwrap();

        let nr_disabled = ethers::prelude::Log {
            address: handlers.addresses.network_registry.into(),
            topics: vec![
                NetworkRegistryStatusUpdatedFilter::signature(),
                H256::from_low_u64_be(0),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, nr_disabled.into()).await }))
            .await
            .unwrap();

        assert!(event_type.is_none(), "there's no chain event type for nr enable");

        assert!(!db.get_indexer_data(None).await.unwrap().nr_enabled);
    }

    #[async_std::test]
    async fn on_network_registry_set_eligible() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let set_eligible = ethers::prelude::Log {
            address: handlers.addresses.network_registry.into(),
            topics: vec![
                EligibilityUpdatedFilter::signature(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()),
                H256::from_low_u64_be(1),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, set_eligible.into()).await }))
            .await
            .unwrap();

        assert!(
            event_type.is_none(),
            "there's no chain event type for setting nr eligibility"
        );

        assert!(db.is_safe_eligible(None, *STAKE_ADDRESS).await.unwrap())
    }

    #[async_std::test]
    async fn on_network_registry_set_not_eligible() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        db.set_safe_eligibility(None, *STAKE_ADDRESS, false).await.unwrap();

        let set_eligible = ethers::prelude::Log {
            address: handlers.addresses.network_registry.into(),
            topics: vec![
                EligibilityUpdatedFilter::signature(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()),
                H256::from_low_u64_be(0),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, set_eligible.into()).await }))
            .await
            .unwrap();

        assert!(
            event_type.is_none(),
            "there's no chain event type for unsetting nr eligibility"
        );

        assert!(!db.is_safe_eligible(None, *STAKE_ADDRESS).await.unwrap())
    }

    #[async_std::test]
    async fn on_channel_event_balance_increased() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let channel = ChannelEntry::new(
            *SELF_CHAIN_ADDRESS,
            *COUNTERPARTY_CHAIN_ADDRESS,
            Balance::new(U256::zero(), BalanceType::HOPR),
            U256::zero(),
            ChannelStatus::Open,
            U256::one(),
        );

        db.upsert_channel(None, channel).await.unwrap();

        let solidity_balance = BalanceType::HOPR.balance(U256::from((1u128 << 96) - 1));
        let diff = solidity_balance - channel.balance;

        let balance_increased_log = ethers::prelude::Log {
            address: handlers.addresses.channels.into(),
            topics: vec![
                ChannelBalanceIncreasedFilter::signature(),
                H256::from_slice(&channel.get_id().to_bytes()),
            ],
            data: Vec::from(solidity_balance.amount().to_bytes()).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, balance_increased_log.into()).await }))
            .await
            .unwrap();

        let channel = db.get_channel_by_id(None, &channel.get_id()).await.unwrap().unwrap();

        assert!(
            matches!(event_type, Some(ChainEventType::ChannelBalanceIncreased(c, b)) if c == channel && b == diff),
            "must return updated channel entry and balance diff"
        );

        assert_eq!(solidity_balance, channel.balance, "balance must be updated");
    }

    #[async_std::test]
    async fn on_channel_event_domain_separator_updated() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let separator = Hash::from(hopr_crypto_random::random_bytes());

        let channels_dst_updated = ethers::prelude::Log {
            address: handlers.addresses.channels.into(),
            topics: vec![
                DomainSeparatorUpdatedFilter::signature(),
                H256::from_slice(&separator.to_bytes()),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        assert!(db.get_indexer_data(None).await.unwrap().channels_dst.is_none());

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, channels_dst_updated.into()).await }))
            .await
            .unwrap();

        assert!(
            event_type.is_none(),
            "there's no chain event type for channel dst update"
        );

        assert_eq!(
            separator,
            db.get_indexer_data(None).await.unwrap().channels_dst.unwrap(),
            "separator must be updated"
        );
    }

    #[async_std::test]
    async fn on_channel_event_balance_decreased() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let channel = ChannelEntry::new(
            *SELF_CHAIN_ADDRESS,
            *COUNTERPARTY_CHAIN_ADDRESS,
            Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
            U256::zero(),
            ChannelStatus::Open,
            U256::one(),
        );

        db.upsert_channel(None, channel).await.unwrap();

        let solidity_balance = U256::from((1u128 << 96) - 2);
        let diff = channel.balance - solidity_balance;

        let balance_decreased_log = ethers::prelude::Log {
            address: handlers.addresses.channels.into(),
            topics: vec![
                ChannelBalanceDecreasedFilter::signature(),
                H256::from_slice(&channel.get_id().to_bytes()),
            ],
            data: Vec::from(solidity_balance.to_bytes()).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, balance_decreased_log.into()).await }))
            .await
            .unwrap();

        let channel = db.get_channel_by_id(None, &channel.get_id()).await.unwrap().unwrap();

        assert!(
            matches!(event_type, Some(ChainEventType::ChannelBalanceDecreased(c, b)) if c == channel && b == diff),
            "must return updated channel entry and balance diff"
        );

        assert_eq!(solidity_balance, channel.balance.amount(), "balance must be updated");
    }

    #[async_std::test]
    async fn on_channel_closed() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let starting_balance = Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR);

        let channel = ChannelEntry::new(
            *SELF_CHAIN_ADDRESS,
            *COUNTERPARTY_CHAIN_ADDRESS,
            starting_balance,
            U256::zero(),
            ChannelStatus::Open,
            U256::one(),
        );

        db.upsert_channel(None, channel).await.unwrap();

        let channel_closed_log = ethers::prelude::Log {
            address: handlers.addresses.channels.into(),
            topics: vec![
                ChannelClosedFilter::signature(),
                H256::from_slice(&channel.get_id().to_bytes()),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, channel_closed_log.into()).await }))
            .await
            .unwrap();

        let closed_channel = db.get_channel_by_id(None, &channel.get_id()).await.unwrap().unwrap();

        assert!(
            matches!(event_type, Some(ChainEventType::ChannelClosed(c)) if c == closed_channel),
            "must return the updated channel entry"
        );

        assert_eq!(closed_channel.status, ChannelStatus::Closed);
        assert_eq!(closed_channel.ticket_index, 0u64.into());
        assert_eq!(
            0,
            db.get_outgoing_ticket_index(closed_channel.get_id())
                .await
                .unwrap()
                .load(Ordering::Relaxed)
        );

        assert!(closed_channel.balance.amount().eq(&U256::zero()));
    }

    #[async_std::test]
    async fn on_channel_opened() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let channel_id = generate_channel_id(&SELF_CHAIN_ADDRESS, &COUNTERPARTY_CHAIN_ADDRESS);

        let channel_opened_log = ethers::prelude::Log {
            address: handlers.addresses.channels.into(),
            topics: vec![
                ChannelOpenedFilter::signature(),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
                H256::from_slice(&COUNTERPARTY_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, channel_opened_log.into()).await }))
            .await
            .unwrap();

        let channel = db.get_channel_by_id(None, &channel_id).await.unwrap().unwrap();

        assert!(
            matches!(event_type, Some(ChainEventType::ChannelOpened(c)) if c == channel),
            "must return the updated channel entry"
        );

        assert_eq!(channel.status, ChannelStatus::Open);
        assert_eq!(channel.channel_epoch, 1u64.into());
        assert_eq!(channel.ticket_index, 0u64.into());
        assert_eq!(
            0,
            db.get_outgoing_ticket_index(channel.get_id())
                .await
                .unwrap()
                .load(Ordering::Relaxed)
        );
    }

    #[async_std::test]
    async fn on_channel_reopened() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let channel = ChannelEntry::new(
            *SELF_CHAIN_ADDRESS,
            *COUNTERPARTY_CHAIN_ADDRESS,
            Balance::zero(BalanceType::HOPR),
            U256::zero(),
            ChannelStatus::Closed,
            3.into(),
        );

        db.upsert_channel(None, channel).await.unwrap();

        let channel_opened_log = ethers::prelude::Log {
            address: handlers.addresses.channels.into(),
            topics: vec![
                ChannelOpenedFilter::signature(),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
                H256::from_slice(&COUNTERPARTY_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, channel_opened_log.into()).await }))
            .await
            .unwrap();

        let channel = db.get_channel_by_id(None, &channel.get_id()).await.unwrap().unwrap();

        assert!(
            matches!(event_type, Some(ChainEventType::ChannelOpened(c)) if c == channel),
            "must return the updated channel entry"
        );

        assert_eq!(channel.status, ChannelStatus::Open);
        assert_eq!(channel.channel_epoch, 4u64.into());
        assert_eq!(channel.ticket_index, 0u64.into());

        assert_eq!(
            0,
            db.get_outgoing_ticket_index(channel.get_id())
                .await
                .unwrap()
                .load(Ordering::Relaxed)
        );
    }

    #[async_std::test]
    async fn on_channel_should_not_reopen_when_not_closed() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let channel = ChannelEntry::new(
            *SELF_CHAIN_ADDRESS,
            *COUNTERPARTY_CHAIN_ADDRESS,
            Balance::zero(BalanceType::HOPR),
            U256::zero(),
            ChannelStatus::Open,
            3.into(),
        );

        db.upsert_channel(None, channel).await.unwrap();

        let channel_opened_log = ethers::prelude::Log {
            address: handlers.addresses.channels.into(),
            topics: vec![
                ChannelOpenedFilter::signature(),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
                H256::from_slice(&COUNTERPARTY_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        db.begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, channel_opened_log.into()).await }))
            .await
            .expect_err("should not re-open channel that is not Closed");
    }

    fn mock_acknowledged_ticket(signer: &ChainKeypair, destination: &ChainKeypair, index: u64) -> AcknowledgedTicket {
        let price_per_packet: U256 = 20_u32.into();
        let ticket_win_prob = 1.0f64;

        let channel_id = generate_channel_id(&signer.into(), &destination.into());

        let channel_epoch = 1u64;
        let domain_separator = Hash::default();

        let response = Response::new(
            &Hash::create(&[
                &channel_id.to_bytes(),
                &channel_epoch.to_be_bytes(),
                &index.to_be_bytes(),
            ])
            .to_bytes(),
        );

        let ticket = Ticket::new(
            &destination.into(),
            &Balance::new(price_per_packet.div_f64(ticket_win_prob).unwrap(), BalanceType::HOPR),
            index.into(),
            1u64.into(),
            ticket_win_prob,
            1u64.into(),
            response.to_challenge().into(),
            signer,
            &domain_separator,
        )
        .unwrap();

        AcknowledgedTicket::new(ticket, response, signer.into(), destination, &domain_separator).unwrap()
    }

    #[async_std::test]
    async fn on_channel_ticket_redeemed_incoming_channel() {
        let db = create_db().await;
        db.set_domain_separator(None, DomainSeparator::Channel, Hash::default())
            .await
            .unwrap();

        let handlers = init_handlers(db.clone());

        let channel = ChannelEntry::new(
            *COUNTERPARTY_CHAIN_ADDRESS,
            *SELF_CHAIN_ADDRESS,
            Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
            U256::zero(),
            ChannelStatus::Open,
            U256::one(),
        );

        let ticket_index = U256::from((1u128 << 48) - 1);
        let next_ticket_index = ticket_index + 1;

        let mut ticket = mock_acknowledged_ticket(&COUNTERPARTY_CHAIN_KEY, &SELF_CHAIN_KEY, ticket_index.as_u64());
        ticket.status = AcknowledgedTicketStatus::BeingRedeemed;

        db.upsert_channel(None, channel).await.unwrap();
        db.upsert_ticket(None, ticket.clone()).await.unwrap();

        let ticket_redeemed_log = ethers::prelude::Log {
            address: handlers.addresses.channels.into(),
            topics: vec![
                TicketRedeemedFilter::signature(),
                H256::from_slice(&channel.get_id().to_bytes()),
            ],
            data: Vec::from(next_ticket_index.to_bytes()).into(),
            ..test_log()
        };

        let outgoing_ticket_index_before = db
            .get_outgoing_ticket_index(channel.get_id())
            .await
            .expect("must get ticket index")
            .load(Ordering::Relaxed);

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, ticket_redeemed_log.into()).await }))
            .await
            .unwrap();

        let channel = db.get_channel_by_id(None, &channel.get_id()).await.unwrap().unwrap();

        assert!(
            matches!(event_type, Some(ChainEventType::TicketRedeemed(c, t)) if channel == c && t == Some(ticket)),
            "must return the updated channel entry and the redeemed ticket"
        );

        assert_eq!(
            channel.ticket_index, next_ticket_index,
            "channel entry must contain next ticket index"
        );

        let outgoing_ticket_index_after = db
            .get_outgoing_ticket_index(channel.get_id())
            .await
            .expect("must get ticket index")
            .load(Ordering::Relaxed);

        assert_eq!(
            outgoing_ticket_index_before, outgoing_ticket_index_after,
            "outgoing ticket index must not change"
        );
    }

    #[async_std::test]
    async fn on_channel_ticket_redeemed_outgoing_channel() {
        let db = create_db().await;
        db.set_domain_separator(None, DomainSeparator::Channel, Hash::default())
            .await
            .unwrap();

        let handlers = init_handlers(db.clone());

        let channel = ChannelEntry::new(
            *SELF_CHAIN_ADDRESS,
            *COUNTERPARTY_CHAIN_ADDRESS,
            Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
            U256::zero(),
            ChannelStatus::Open,
            U256::one(),
        );

        let ticket_index = U256::from((1u128 << 48) - 1);
        let next_ticket_index = ticket_index + 1;

        db.upsert_channel(None, channel).await.unwrap();

        let ticket_redeemed_log = ethers::prelude::Log {
            address: handlers.addresses.channels.into(),
            topics: vec![
                TicketRedeemedFilter::signature(),
                H256::from_slice(&channel.get_id().to_bytes()),
            ],
            data: Vec::from(next_ticket_index.to_bytes()).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, ticket_redeemed_log.into()).await }))
            .await
            .unwrap();

        let channel = db.get_channel_by_id(None, &channel.get_id()).await.unwrap().unwrap();

        assert!(
            matches!(event_type, Some(ChainEventType::TicketRedeemed(c, None)) if channel == c),
            "must return update channel entry and no ticket"
        );

        assert_eq!(
            channel.ticket_index, next_ticket_index,
            "channel entry must contain next ticket index"
        );

        let outgoing_ticket_index = db
            .get_outgoing_ticket_index(channel.get_id())
            .await
            .expect("must get ticket index")
            .load(Ordering::Relaxed);

        assert!(
            outgoing_ticket_index >= ticket_index.as_u64(),
            "outgoing idx {outgoing_ticket_index} must be greater or equal to {ticket_index}"
        );
        assert_eq!(
            outgoing_ticket_index,
            next_ticket_index.as_u64(),
            "outgoing ticket index must be equal to next ticket index"
        );
    }

    #[async_std::test]
    async fn on_channel_ticket_redeemed_on_incoming_channel_with_non_existent_ticket_should_pass() {
        let db = create_db().await;
        db.set_domain_separator(None, DomainSeparator::Channel, Hash::default())
            .await
            .unwrap();

        let handlers = init_handlers(db.clone());

        let channel = ChannelEntry::new(
            *COUNTERPARTY_CHAIN_ADDRESS,
            *SELF_CHAIN_ADDRESS,
            Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
            U256::zero(),
            ChannelStatus::Open,
            U256::one(),
        );

        db.upsert_channel(None, channel).await.unwrap();

        let next_ticket_index = U256::from((1u128 << 48) - 1);

        let ticket_redeemed_log = ethers::prelude::Log {
            address: handlers.addresses.channels.into(),
            topics: vec![
                TicketRedeemedFilter::signature(),
                H256::from_slice(&channel.get_id().to_bytes()),
            ],
            data: Vec::from(next_ticket_index.to_bytes()).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, ticket_redeemed_log.into()).await }))
            .await
            .unwrap();

        let channel = db.get_channel_by_id(None, &channel.get_id()).await.unwrap().unwrap();

        assert!(
            matches!(event_type, Some(ChainEventType::TicketRedeemed(c, None)) if c == channel),
            "must return updated channel entry and no ticket"
        );

        assert_eq!(
            channel.ticket_index, next_ticket_index,
            "channel entry must contain next ticket index"
        );
    }

    #[async_std::test]
    async fn on_channel_ticket_redeemed_on_foreign_channel_should_pass() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let channel = ChannelEntry::new(
            Address::from(hopr_crypto_random::random_bytes()),
            Address::from(hopr_crypto_random::random_bytes()),
            Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
            U256::zero(),
            ChannelStatus::Open,
            U256::one(),
        );

        db.upsert_channel(None, channel).await.unwrap();

        let next_ticket_index = U256::from((1u128 << 48) - 1);

        let ticket_redeemed_log = ethers::prelude::Log {
            address: handlers.addresses.channels.into(),
            topics: vec![
                TicketRedeemedFilter::signature(),
                H256::from_slice(&channel.get_id().to_bytes()),
            ],
            data: Vec::from(next_ticket_index.to_bytes()).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, ticket_redeemed_log.into()).await }))
            .await
            .unwrap();

        let channel = db.get_channel_by_id(None, &channel.get_id()).await.unwrap().unwrap();

        assert!(
            matches!(event_type, Some(ChainEventType::TicketRedeemed(c, None)) if c == channel),
            "must return updated channel entry and no ticket"
        );

        assert_eq!(
            channel.ticket_index, next_ticket_index,
            "channel entry must contain next ticket index"
        );
    }

    #[async_std::test]
    async fn on_channel_closure_initiated() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let channel = ChannelEntry::new(
            *SELF_CHAIN_ADDRESS,
            *COUNTERPARTY_CHAIN_ADDRESS,
            Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
            U256::zero(),
            ChannelStatus::Open,
            U256::one(),
        );

        db.upsert_channel(None, channel).await.unwrap();

        let timestamp = SystemTime::now();

        let closure_initiated_log = ethers::prelude::Log {
            address: handlers.addresses.channels.into(),
            topics: vec![
                OutgoingChannelClosureInitiatedFilter::signature(),
                H256::from_slice(&channel.get_id().to_bytes()),
            ],
            data: Vec::from(U256::from(timestamp.as_unix_timestamp().as_secs()).to_bytes()).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, closure_initiated_log.into()).await }))
            .await
            .unwrap();

        let channel = db.get_channel_by_id(None, &channel.get_id()).await.unwrap().unwrap();

        assert!(
            matches!(event_type, Some(ChainEventType::ChannelClosureInitiated(c)) if c == channel),
            "must return updated channel entry"
        );

        assert_eq!(
            channel.status,
            ChannelStatus::PendingToClose(timestamp),
            "channel status must match"
        );
    }

    #[async_std::test]
    async fn on_node_safe_registry_registered() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let safe_registered_log = ethers::prelude::Log {
            address: handlers.addresses.safe_registry.into(),
            topics: vec![
                RegisteredNodeSafeFilter::signature(),
                H256::from_slice(&SAFE_INSTANCE_ADDR.to_bytes32()),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, safe_registered_log.into()).await }))
            .await
            .unwrap();

        assert!(matches!(event_type, Some(ChainEventType::NodeSafeRegistered(addr)) if addr == *SAFE_INSTANCE_ADDR));

        // Nothing to check in the DB here, since we do not track this
    }

    #[async_std::test]
    async fn on_node_safe_registry_deregistered() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        // Nothing to write to the DB here, since we do not track this

        let safe_registered_log = ethers::prelude::Log {
            address: handlers.addresses.safe_registry.into(),
            topics: vec![
                DergisteredNodeSafeFilter::signature(),
                H256::from_slice(&SAFE_INSTANCE_ADDR.to_bytes32()),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, safe_registered_log.into()).await }))
            .await
            .unwrap();

        assert!(
            event_type.is_none(),
            "there's no associated chain event type with safe deregistration"
        );

        // Nothing to check in the DB here, since we do not track this
    }

    #[async_std::test]
    async fn ticket_price_update() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let price_change_log = ethers::prelude::Log {
            address: handlers.addresses.price_oracle.into(),
            topics: vec![TicketPriceUpdatedFilter::signature()],
            data: encode(&[Token::Uint(EthU256::from(1u64)), Token::Uint(EthU256::from(123u64))]).into(),
            ..test_log()
        };

        assert_eq!(db.get_indexer_data(None).await.unwrap().ticket_price, None);

        let event_type = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, price_change_log.into()).await }))
            .await
            .unwrap();

        assert!(
            event_type.is_none(),
            "there's no associated chain event type with price oracle"
        );

        assert_eq!(
            db.get_indexer_data(None)
                .await
                .unwrap()
                .ticket_price
                .map(|p| p.amount()),
            Some(U256::from(123u64))
        );
    }
}
