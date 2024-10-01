use async_trait::async_trait;
use ethers::contract::EthLogDecode;
use ethers::types::TxHash;
use std::cmp::Ordering;
use std::fmt::Formatter;
use std::ops::{Add, Sub};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::{debug, error, info, trace, warn};

use bindings::{
    hopr_announcements::HoprAnnouncementsEvents, hopr_channels::HoprChannelsEvents,
    hopr_network_registry::HoprNetworkRegistryEvents, hopr_node_management_module::HoprNodeManagementModuleEvents,
    hopr_node_safe_registry::HoprNodeSafeRegistryEvents, hopr_ticket_price_oracle::HoprTicketPriceOracleEvents,
    hopr_token::HoprTokenEvents,
};
use chain_rpc::{BlockWithLogs, Log};
use chain_types::chain_events::{ChainEventType, NetworkRegistryStatus, SignificantChainEvent};
use chain_types::ContractAddresses;
use hopr_crypto_types::keypairs::ChainKeypair;
use hopr_crypto_types::prelude::{Hash, Keypair};
use hopr_crypto_types::types::OffchainSignature;
use hopr_db_sql::api::info::DomainSeparator;
use hopr_db_sql::api::tickets::TicketSelector;
use hopr_db_sql::errors::DbSqlError;
use hopr_db_sql::{HoprDbAllOperations, OpenTransaction};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

use crate::errors::{CoreEthereumIndexerError, Result};

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
                    Err(DbSqlError::MissingAccount) => Err(CoreEthereumIndexerError::AnnounceBeforeKeyBinding),
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
                    Err(DbSqlError::MissingAccount) => {
                        return Err(CoreEthereumIndexerError::RevocationBeforeKeyBinding)
                    }
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
                let maybe_channel = self
                    .db
                    .begin_channel_update(tx.into(), &balance_decreased.channel_id.into())
                    .await?;

                if let Some(channel_edits) = maybe_channel {
                    let new_balance = Balance::new(balance_decreased.new_balance, BalanceType::HOPR);
                    let diff = channel_edits.entry().balance.sub(&new_balance);

                    let updated_channel = self
                        .db
                        .finish_channel_update(tx.into(), channel_edits.change_balance(new_balance))
                        .await?;

                    Ok(Some(ChainEventType::ChannelBalanceDecreased(updated_channel, diff)))
                } else {
                    Err(CoreEthereumIndexerError::ChannelDoesNotExist)
                }
            }
            HoprChannelsEvents::ChannelBalanceIncreasedFilter(balance_increased) => {
                let maybe_channel = self
                    .db
                    .begin_channel_update(tx.into(), &balance_increased.channel_id.into())
                    .await?;

                if let Some(channel_edits) = maybe_channel {
                    let new_balance = Balance::new(balance_increased.new_balance, BalanceType::HOPR);
                    let diff = new_balance.sub(&channel_edits.entry().balance);

                    let updated_channel = self
                        .db
                        .finish_channel_update(tx.into(), channel_edits.change_balance(new_balance))
                        .await?;

                    Ok(Some(ChainEventType::ChannelBalanceIncreased(updated_channel, diff)))
                } else {
                    Err(CoreEthereumIndexerError::ChannelDoesNotExist)
                }
            }
            HoprChannelsEvents::ChannelClosedFilter(channel_closed) => {
                let maybe_channel = self
                    .db
                    .begin_channel_update(tx.into(), &channel_closed.channel_id.into())
                    .await?;

                trace!(
                    "on_channel_closed_event - channel_id: {:?} - channel known: {:?}",
                    channel_closed.channel_id,
                    maybe_channel.is_some()
                );

                if let Some(channel_edits) = maybe_channel {
                    // Incoming channel, so once closed. All unredeemed tickets just became invalid
                    if channel_edits.entry().destination == self.chain_key.public().to_address() {
                        self.db.mark_tickets_neglected(channel_edits.entry().into()).await?;
                    }

                    // set all channel fields like we do on-chain on close
                    let channel = channel_edits
                        .change_status(ChannelStatus::Closed)
                        .change_balance(BalanceType::HOPR.zero())
                        .change_ticket_index(0);

                    let updated_channel = self.db.finish_channel_update(tx.into(), channel).await?;

                    if updated_channel.source == self.chain_key.public().to_address()
                        || updated_channel.destination == self.chain_key.public().to_address()
                    {
                        // Reset the current_ticket_index to zero
                        self.db
                            .reset_outgoing_ticket_index(channel_closed.channel_id.into())
                            .await?;
                    }

                    Ok(Some(ChainEventType::ChannelClosed(updated_channel)))
                } else {
                    Err(CoreEthereumIndexerError::ChannelDoesNotExist)
                }
            }
            HoprChannelsEvents::ChannelOpenedFilter(channel_opened) => {
                let source: Address = channel_opened.source.into();
                let destination: Address = channel_opened.destination.into();
                let channel_id = generate_channel_id(&source, &destination);

                let maybe_channel = self.db.begin_channel_update(tx.into(), &channel_id).await?;

                let channel = if let Some(channel_edits) = maybe_channel {
                    // Check that we're not receiving the Open event without the channel being Close prior
                    if channel_edits.entry().status != ChannelStatus::Closed {
                        return Err(CoreEthereumIndexerError::ProcessError(format!(
                            "trying to re-open channel {} which is not closed",
                            channel_edits.entry().get_id()
                        )));
                    }

                    trace!(
                        "on_channel_reopened_event - source: {source} - destination: {destination} - channel_id: {channel_id}"
                    );

                    let current_epoch = channel_edits.entry().channel_epoch;

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
                    self.db
                        .finish_channel_update(
                            tx.into(),
                            channel_edits
                                .change_ticket_index(0_u32)
                                .change_epoch(current_epoch.add(1))
                                .change_status(ChannelStatus::Open),
                        )
                        .await?
                } else {
                    trace!(
                        "on_channel_opened_event - source: {source} - destination: {destination} - channel_id: {channel_id}"
                    );

                    let new_channel = ChannelEntry::new(
                        source,
                        destination,
                        BalanceType::HOPR.zero(),
                        0_u32.into(),
                        ChannelStatus::Open,
                        1_u32.into(),
                    );

                    self.db.upsert_channel(tx.into(), new_channel).await?;
                    new_channel
                };

                Ok(Some(ChainEventType::ChannelOpened(channel)))
            }
            HoprChannelsEvents::TicketRedeemedFilter(ticket_redeemed) => {
                let maybe_channel = self
                    .db
                    .begin_channel_update(tx.into(), &ticket_redeemed.channel_id.into())
                    .await?;

                if let Some(channel_edits) = maybe_channel {
                    let ack_ticket = match channel_edits.entry().direction(&self.chain_key.public().to_address()) {
                        // For channels where destination is us, it means that our ticket
                        // has been redeemed, so mark it in the DB as redeemed
                        Some(ChannelDirection::Incoming) => {
                            // Filter all BeingRedeemed tickets in this channel and its current epoch
                            let mut matching_tickets = self
                                .db
                                .get_tickets(
                                    TicketSelector::from(channel_edits.entry())
                                        .with_state(AcknowledgedTicketStatus::BeingRedeemed),
                                )
                                .await?
                                .into_iter()
                                .filter(|ticket| {
                                    // The ticket that has been redeemed at this point has: index + index_offset - 1 == new_ticket_index - 1
                                    // Since unaggregated tickets have index_offset = 1, for the unagg case this leads to: index == new_ticket_index - 1
                                    let ticket_idx = ticket.verified_ticket().index;
                                    let ticket_off = ticket.verified_ticket().index_offset as u64;

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
                                        "could not find acknowledged 'BeingRedeemed' ticket with idx {} in {}",
                                        ticket_redeemed.new_ticket_index - 1,
                                        channel_edits.entry()
                                    );
                                    // This is not an error, because the ticket might've become neglected before
                                    // the ticket redemption could finish
                                    None
                                }
                                Ordering::Greater => {
                                    error!(
                                        "found {} tickets matching 'BeingRedeemed' index {} in {}",
                                        matching_tickets.len(),
                                        ticket_redeemed.new_ticket_index - 1,
                                        channel_edits.entry()
                                    );
                                    return Err(CoreEthereumIndexerError::ProcessError(format!(
                                        "multiple tickets matching idx {} found in {}",
                                        ticket_redeemed.new_ticket_index - 1,
                                        channel_edits.entry()
                                    )));
                                }
                            }
                        }
                        // For the channel where the source is us, it means a ticket that we
                        // issue has been redeemed.
                        // So we just need to be sure our outgoing ticket
                        // index value in the cache is at least the index of the redeemed ticket
                        Some(ChannelDirection::Outgoing) => {
                            // We need to ensure the outgoing ticket index is at least this new value
                            debug!("observed redeem event on an outgoing {}", channel_edits.entry());
                            self.db
                                .compare_and_set_outgoing_ticket_index(
                                    channel_edits.entry().get_id(),
                                    ticket_redeemed.new_ticket_index,
                                )
                                .await?;
                            None
                        }
                        // For channel where neither source nor destination is us, we don't care
                        None => {
                            // Not our redeem event
                            debug!("observed redeem event on a foreign {}", channel_edits.entry());
                            None
                        }
                    };

                    // Update the ticket index on the Channel entry and get the updated model
                    let channel = self
                        .db
                        .finish_channel_update(
                            tx.into(),
                            channel_edits.change_ticket_index(ticket_redeemed.new_ticket_index),
                        )
                        .await?;

                    // Neglect all the tickets in this channel
                    // which have a lower ticket index than `ticket_redeemed.new_ticket_index`
                    self.db
                        .mark_tickets_neglected(
                            TicketSelector::from(&channel).with_index_lt(ticket_redeemed.new_ticket_index),
                        )
                        .await?;

                    Ok(Some(ChainEventType::TicketRedeemed(channel, ack_ticket)))
                } else {
                    error!("observed ticket redeem on a channel that we don't have in the DB");
                    Err(CoreEthereumIndexerError::ChannelDoesNotExist)
                }
            }
            HoprChannelsEvents::OutgoingChannelClosureInitiatedFilter(closure_initiated) => {
                let maybe_channel = self
                    .db
                    .begin_channel_update(tx.into(), &closure_initiated.channel_id.into())
                    .await?;

                if let Some(channel_edits) = maybe_channel {
                    let new_status = ChannelStatus::PendingToClose(
                        SystemTime::UNIX_EPOCH.add(Duration::from_secs(closure_initiated.closure_time as u64)),
                    );

                    let channel = self
                        .db
                        .finish_channel_update(tx.into(), channel_edits.change_status(new_status))
                        .await?;
                    Ok(Some(ChainEventType::ChannelClosureInitiated(channel)))
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

    // TODO: uncomment this once ticket winning probability oracle is implemented
    /*async fn on_ticket_winning_probability_oracle_event(
        &self,
        tx: &OpenTransaction,
        event: HoprTicketWinningProbabilityOracleEvents,
    ) -> Result<Option<ChainEventType>> {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_INDEXER_LOG_COUNTERS.increment(&["win_prob_oracle"]);

        match event {
            HoprTicketWinningProbabilityOracleEvents::TicketMinWinningProbability(update) => {
                trace!(
                    "on_ticket_minimum_win_prob_updated - old: {:?} - new: {:?}",
                    update.0.to_string(),
                    update.1.to_string()
                );

                self.db
                    .set_minimum_incoming_ticket_win_prob(Some(tx), win_prob_to_f64(update.1))
                    .await?;

                info!("minimum ticket winning probability has been set to {}", update.1);
            }
            _ => {
                // ignore other events
            }
        }
        Ok(None)
    }*/

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
    async fn process_log_event(&self, tx: &OpenTransaction, slog: SerializableLog) -> Result<Option<ChainEventType>> {
        trace!("processing events in {slog}");
        let log = Log::from(slog);

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
        } else if log.address.eq(&self.addresses.win_prob_oracle) {
            // TODO: add win prob oracle implementation
            warn!("winning probability oracle not yet implemented");
            Ok(None)
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
            self.addresses.announcements,
            self.addresses.channels,
            self.addresses.module_implementation,
            self.addresses.network_registry,
            self.addresses.price_oracle,
            self.addresses.win_prob_oracle,
            self.addresses.safe_registry,
            self.addresses.token,
        ]
    }

    fn contract_address_topics(&self, contract: Address) -> Vec<TxHash> {
        if contract.eq(&self.addresses.announcements) {
            crate::constants::topics::announcement()
        } else if contract.eq(&self.addresses.channels) {
            crate::constants::topics::channel()
        } else if contract.eq(&self.addresses.module_implementation) {
            crate::constants::topics::module_implementation()
        } else if contract.eq(&self.addresses.network_registry) {
            crate::constants::topics::network_registry()
        } else if contract.eq(&self.addresses.price_oracle) {
            crate::constants::topics::ticket_price_oracle()
        } else if contract.eq(&self.addresses.win_prob_oracle) {
            crate::constants::topics::winning_prob_oracle()
        } else if contract.eq(&self.addresses.safe_registry) {
            crate::constants::topics::node_safe_registry()
        } else if contract.eq(&self.addresses.token) {
            crate::constants::topics::token()
        } else {
            vec![]
        }
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

                    // Process all logs in the block
                    for log in block_with_logs.logs {
                        let tx_hash = Hash::from_hex(log.tx_hash.as_str())?;

                        // If a significant chain event can be extracted from the log, push it
                        if let Some(event_type) = myself.process_log_event(tx, log).await? {
                            let significant_event = SignificantChainEvent { tx_hash, event_type };
                            debug!("indexer got {significant_event}");
                            ret.push(significant_event);
                        }
                    }

                    Ok(ret)
                })
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::time::SystemTime;

    use super::ContractEventHandlers;
    use anyhow::Context;
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
    use hopr_db_sql::accounts::{ChainOrPacketKey, HoprDbAccountOperations};
    use hopr_db_sql::api::{info::DomainSeparator, tickets::HoprDbTicketOperations};
    use hopr_db_sql::channels::HoprDbChannelOperations;
    use hopr_db_sql::db::HoprDb;
    use hopr_db_sql::info::HoprDbInfoOperations;
    use hopr_db_sql::prelude::HoprDbResolverOperations;
    use hopr_db_sql::registry::HoprDbRegistryOperations;
    use hopr_db_sql::{HoprDbAllOperations, HoprDbGeneralModelOperations};
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use multiaddr::Multiaddr;
    use primitive_types::H256;

    lazy_static::lazy_static! {
        static ref SELF_PRIV_KEY: OffchainKeypair = OffchainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).expect("lazy static keypair should be constructible");
        static ref COUNTERPARTY_CHAIN_KEY: ChainKeypair = ChainKeypair::random();
        static ref COUNTERPARTY_CHAIN_ADDRESS: Address = COUNTERPARTY_CHAIN_KEY.public().to_address();
        static ref SELF_CHAIN_KEY: ChainKeypair = ChainKeypair::random();
        static ref SELF_CHAIN_ADDRESS: Address = SELF_CHAIN_KEY.public().to_address();
        static ref STAKE_ADDRESS: Address = "4331eaa9542b6b034c43090d9ec1c2198758dbc3".parse().expect("lazy static address should be constructible");
        static ref CHANNELS_ADDR: Address = "bab20aea98368220baa4e3b7f151273ee71df93b".parse().expect("lazy static address should be constructible"); // just a dummy
        static ref TOKEN_ADDR: Address = "47d1677e018e79dcdd8a9c554466cb1556fa5007".parse().expect("lazy static address should be constructible"); // just a dummy
        static ref NETWORK_REGISTRY_ADDR: Address = "a469d0225f884fb989cbad4fe289f6fd2fb98051".parse().expect("lazy static address should be constructible"); // just a dummy
        static ref NODE_SAFE_REGISTRY_ADDR: Address = "0dcd1bf9a1b36ce34237eeafef220932846bcd82".parse().expect("lazy static address should be constructible"); // just a dummy
        static ref ANNOUNCEMENTS_ADDR: Address = "11db4791bf45ef31a10ea4a1b5cb90f46cc72c7e".parse().expect("lazy static address should be constructible"); // just a dummy
        static ref SAFE_MANAGEMENT_MODULE_ADDR: Address = "9b91245a65ad469163a86e32b2281af7a25f38ce".parse().expect("lazy static address should be constructible"); // just a dummy
        static ref SAFE_INSTANCE_ADDR: Address = "b93d7fdd605fb64fdcc87f21590f950170719d47".parse().expect("lazy static address should be constructible"); // just a dummy
        static ref TICKET_PRICE_ORACLE_ADDR: Address = "11db4391bf45ef31a10ea4a1b5cb90f46cc72c7e".parse().expect("lazy static address should be constructible"); // just a dummy
        static ref WIN_PROB_ORACLE_ADDR: Address = "00db4391bf45ef31a10ea4a1b5cb90f46cc64c7e".parse().expect("lazy static address should be constructible"); // just a dummy
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
                win_prob_oracle: *WIN_PROB_ORACLE_ADDR,
                stake_factory: Default::default(),
            }),
            chain_key: SELF_CHAIN_KEY.clone(),
            safe_address: SELF_CHAIN_KEY.public().to_address(),
            db,
        }
    }

    fn test_log() -> SerializableLog {
        SerializableLog {
            address: Default::default(),
            topics: vec![],
            data: Default::default(),
            block_hash: format!("{:#x}", H256::zero()),
            block_number: 0,
            tx_hash: format!("{:#x}", H256::zero()),
            tx_index: 0,
            log_index: 0,
            removed: false,
            ..Default::default()
        }
    }

    #[async_std::test]
    async fn announce_keybinding() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        let keybinding = KeyBinding::new(*SELF_CHAIN_ADDRESS, &SELF_PRIV_KEY);

        let keybinding_log = SerializableLog {
            address: handlers.addresses.announcements.to_hex(),
            topics: vec![format!("{:#x}", KeyBindingFilter::signature())],
            data: encode(&[
                Token::FixedBytes(keybinding.signature.as_ref().to_vec()),
                Token::FixedBytes(keybinding.packet_key.as_ref().to_vec()),
                Token::Address(EthereumAddress::from_slice(
                    &SELF_CHAIN_KEY.public().to_address().as_ref(),
                )),
            ])
            .into(),
            ..test_log()
        };

        let account_entry = AccountEntry::new(*SELF_PRIV_KEY.public(), *SELF_CHAIN_ADDRESS, AccountType::NotAnnounced);

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, keybinding_log.into()).await }))
            .await?;

        assert!(event_type.is_none(), "keybinding does not have a chain event type");

        assert_eq!(
            db.get_account(None, ChainOrPacketKey::ChainKey(*SELF_CHAIN_ADDRESS))
                .await?
                .context("a value should be present")?,
            account_entry
        );
        Ok(())
    }

    #[async_std::test]
    async fn announce_address_announcement() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        // Assume that there is a keybinding
        let account_entry = AccountEntry::new(*SELF_PRIV_KEY.public(), *SELF_CHAIN_ADDRESS, AccountType::NotAnnounced);
        db.insert_account(None, account_entry.clone()).await?;

        let test_multiaddr_empty: Multiaddr = "".parse()?;

        let address_announcement_empty_log = SerializableLog {
            address: handlers.addresses.announcements.to_hex(),
            topics: vec![format!("{:#x}", AddressAnnouncementFilter::signature())],
            data: encode(&[
                Token::Address(EthereumAddress::from_slice(&SELF_CHAIN_ADDRESS.as_ref())),
                Token::String(test_multiaddr_empty.to_string()),
            ])
            .into(),
            ..test_log()
        };

        let handlers_clone = handlers.clone();
        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    handlers_clone
                        .process_log_event(tx, address_announcement_empty_log.into())
                        .await
                })
            })
            .await?;

        assert!(
            event_type.is_none(),
            "announcement of empty multiaddresses must pass through"
        );

        assert_eq!(
            db.get_account(None, ChainOrPacketKey::ChainKey(*SELF_CHAIN_ADDRESS))
                .await?
                .context("a value should be present")?,
            account_entry
        );

        let test_multiaddr: Multiaddr = "/ip4/1.2.3.4/tcp/56".parse()?;

        let address_announcement_log = SerializableLog {
            address: handlers.addresses.announcements.to_hex(),
            block_number: 1,
            topics: vec![format!("{:#x}", AddressAnnouncementFilter::signature())],
            data: encode(&[
                Token::Address(EthereumAddress::from_slice(&SELF_CHAIN_ADDRESS.as_ref())),
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
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    handlers_clone
                        .process_log_event(tx, address_announcement_log.into())
                        .await
                })
            })
            .await?;

        assert!(
            matches!(event_type, Some(ChainEventType::Announcement { multiaddresses,.. }) if multiaddresses == vec![test_multiaddr]),
            "must return the latest announce multiaddress"
        );

        assert_eq!(
            db.get_account(None, ChainOrPacketKey::ChainKey(*SELF_CHAIN_ADDRESS))
                .await?
                .context("a value should be present")?,
            announced_account_entry
        );

        assert_eq!(
            Some(*SELF_CHAIN_ADDRESS),
            db.resolve_chain_key(SELF_PRIV_KEY.public()).await?,
            "must resolve correct chain key"
        );

        assert_eq!(
            Some(*SELF_PRIV_KEY.public()),
            db.resolve_packet_key(&SELF_CHAIN_ADDRESS).await?,
            "must resolve correct packet key"
        );

        let test_multiaddr_dns: Multiaddr = "/dns4/useful.domain/tcp/56".parse()?;

        let address_announcement_dns_log = SerializableLog {
            address: handlers.addresses.announcements.to_hex(),
            block_number: 2,
            topics: vec![format!("{:#x}", AddressAnnouncementFilter::signature())],
            data: encode(&[
                Token::Address(EthereumAddress::from_slice(&SELF_CHAIN_ADDRESS.as_ref())),
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
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    handlers
                        .process_log_event(tx, address_announcement_dns_log.into())
                        .await
                })
            })
            .await?;

        assert!(
            matches!(event_type, Some(ChainEventType::Announcement { multiaddresses,.. }) if multiaddresses == vec![test_multiaddr_dns]),
            "must return the latest announce multiaddress"
        );

        assert_eq!(
            db.get_account(None, ChainOrPacketKey::ChainKey(*SELF_CHAIN_ADDRESS))
                .await?
                .context("a value should be present")?,
            announced_dns_account_entry
        );

        assert_eq!(
            Some(*SELF_CHAIN_ADDRESS),
            db.resolve_chain_key(SELF_PRIV_KEY.public()).await?,
            "must resolve correct chain key"
        );

        assert_eq!(
            Some(*SELF_PRIV_KEY.public()),
            db.resolve_packet_key(&SELF_CHAIN_ADDRESS).await?,
            "must resolve correct packet key"
        );
        Ok(())
    }

    #[async_std::test]
    async fn announce_revoke() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let handlers = init_handlers(db.clone());

        let test_multiaddr: Multiaddr = "/ip4/1.2.3.4/tcp/56".parse()?;

        // Assume that there is a keybinding and an address announcement
        let announced_account_entry = AccountEntry::new(
            *SELF_PRIV_KEY.public(),
            *SELF_CHAIN_ADDRESS,
            AccountType::Announced {
                multiaddr: test_multiaddr,
                updated_block: 0,
            },
        );
        db.insert_account(None, announced_account_entry).await?;

        let revoke_announcement_log = SerializableLog {
            address: handlers.addresses.announcements.to_hex(),
            topics: vec![format!("{:#x}", RevokeAnnouncementFilter::signature())],
            data: encode(&[Token::Address(EthereumAddress::from_slice(
                &SELF_CHAIN_ADDRESS.as_ref(),
            ))])
            .into(),
            ..test_log()
        };

        let account_entry = AccountEntry::new(*SELF_PRIV_KEY.public(), *SELF_CHAIN_ADDRESS, AccountType::NotAnnounced);

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, revoke_announcement_log.into()).await }))
            .await?;

        assert!(
            event_type.is_none(),
            "revoke announcement does not have chain event type"
        );

        assert_eq!(
            db.get_account(None, ChainOrPacketKey::ChainKey(*SELF_CHAIN_ADDRESS))
                .await?
                .context("a value should be present")?,
            account_entry
        );
        Ok(())
    }

    #[async_std::test]
    async fn on_token_transfer_to() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        let value = U256::max_value();

        let transferred_log = SerializableLog {
            address: handlers.addresses.token.to_hex(),
            topics: vec![
                format!("{:#x}", TransferFilter::signature()),
                format!("{:#x}", H256::from_slice(&Address::default().to_bytes32())),
                format!("{:#x}", H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32())),
            ],
            data: encode(&[Token::Uint(value)]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, transferred_log.into()).await }))
            .await?;

        assert!(event_type.is_none(), "token transfer does not have chain event type");

        assert_eq!(
            db.get_safe_hopr_balance(None).await?,
            Balance::new(value, BalanceType::HOPR)
        );

        Ok(())
    }

    #[async_std::test]
    async fn on_token_transfer_from() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        let value = U256::max_value();

        db.set_safe_hopr_balance(None, BalanceType::HOPR.balance(value)).await?;

        let transferred_log = SerializableLog {
            address: handlers.addresses.token.to_hex(),
            topics: vec![
                format!("{:#x}", TransferFilter::signature()),
                format!("{:#x}", H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32())),
                format!("{:#x}", H256::from_slice(&Address::default().to_bytes32())),
            ],
            data: encode(&[Token::Uint(value)]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, transferred_log.into()).await }))
            .await?;

        assert!(event_type.is_none(), "token transfer does not have chain event type");

        assert_eq!(db.get_safe_hopr_balance(None).await?, BalanceType::HOPR.zero());

        Ok(())
    }

    #[async_std::test]
    async fn on_token_approval_correct() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        let approval_log = SerializableLog {
            address: handlers.addresses.token.to_hex(),
            topics: vec![
                format!("{:#x}", ApprovalFilter::signature()),
                format!("{:#x}", H256::from_slice(&handlers.safe_address.to_bytes32())),
                format!("{:#x}", H256::from_slice(&handlers.addresses.channels.to_bytes32())),
            ],
            data: encode(&[Token::Uint(EthU256::from(1000u64))]).into(),
            ..test_log()
        };

        assert_eq!(
            db.get_safe_hopr_allowance(None).await?,
            Balance::new(U256::from(0u64), BalanceType::HOPR)
        );

        let approval_log_clone = approval_log.clone();
        let handlers_clone = handlers.clone();
        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move { handlers_clone.process_log_event(tx, approval_log_clone.into()).await })
            })
            .await?;

        assert!(event_type.is_none(), "token approval does not have chain event type");

        assert_eq!(
            db.get_safe_hopr_allowance(None).await?,
            Balance::new(U256::from(1000u64), BalanceType::HOPR)
        );

        // reduce allowance manually to verify a second time
        let _ = db
            .set_safe_hopr_allowance(None, Balance::new(U256::from(10u64), BalanceType::HOPR))
            .await;
        assert_eq!(
            db.get_safe_hopr_allowance(None).await?,
            Balance::new(U256::from(10u64), BalanceType::HOPR)
        );

        let handlers_clone = handlers.clone();
        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers_clone.process_log_event(tx, approval_log.into()).await }))
            .await?;

        assert!(event_type.is_none(), "token approval does not have chain event type");

        assert_eq!(
            db.get_safe_hopr_allowance(None).await?,
            Balance::new(U256::from(1000u64), BalanceType::HOPR)
        );
        Ok(())
    }

    #[async_std::test]
    async fn on_network_registry_event_registered() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        let registered_log = SerializableLog {
            address: handlers.addresses.network_registry.to_hex(),
            topics: vec![
                format!("{:#x}", RegisteredFilter::signature()),
                format!("{:#x}", H256::from_slice(&STAKE_ADDRESS.to_bytes32())),
                format!("{:#x}", H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32())),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        assert!(!db.is_allowed_in_network_registry(None, *SELF_CHAIN_ADDRESS).await?);

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, registered_log.into()).await }))
            .await?;

        assert!(
            matches!(event_type, Some(ChainEventType::NetworkRegistryUpdate(a, s)) if a == *SELF_CHAIN_ADDRESS && s == NetworkRegistryStatus::Allowed),
            "must return correct NR update"
        );

        assert!(
            db.is_allowed_in_network_registry(None, *SELF_CHAIN_ADDRESS).await?,
            "must be allowed in NR"
        );
        Ok(())
    }

    #[async_std::test]
    async fn on_network_registry_event_registered_by_manager() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        let registered_log = SerializableLog {
            address: handlers.addresses.network_registry.to_hex(),
            topics: vec![
                format!("{:#x}", RegisteredByManagerFilter::signature()),
                format!("{:#x}", H256::from_slice(&STAKE_ADDRESS.to_bytes32())),
                format!("{:#x}", H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32())),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        assert!(!db.is_allowed_in_network_registry(None, *SELF_CHAIN_ADDRESS).await?);

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, registered_log.into()).await }))
            .await?;

        assert!(
            matches!(event_type, Some(ChainEventType::NetworkRegistryUpdate(a, s)) if a == *SELF_CHAIN_ADDRESS && s == NetworkRegistryStatus::Allowed),
            "must return correct NR update"
        );

        assert!(
            db.is_allowed_in_network_registry(None, *SELF_CHAIN_ADDRESS).await?,
            "must be allowed in NR"
        );
        Ok(())
    }

    #[async_std::test]
    async fn on_network_registry_event_deregistered() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        db.set_access_in_network_registry(None, *SELF_CHAIN_ADDRESS, true)
            .await?;

        let registered_log = SerializableLog {
            address: handlers.addresses.network_registry.to_hex(),
            topics: vec![
                format!("{:#x}", DeregisteredFilter::signature()),
                format!("{:#x}", H256::from_slice(&STAKE_ADDRESS.to_bytes32())),
                format!("{:#x}", H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32())),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        assert!(db.is_allowed_in_network_registry(None, *SELF_CHAIN_ADDRESS).await?);

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, registered_log.into()).await }))
            .await?;

        assert!(
            matches!(event_type, Some(ChainEventType::NetworkRegistryUpdate(a, s)) if a == *SELF_CHAIN_ADDRESS && s == NetworkRegistryStatus::Denied),
            "must return correct NR update"
        );

        assert!(
            !db.is_allowed_in_network_registry(None, *SELF_CHAIN_ADDRESS).await?,
            "must not be allowed in NR"
        );
        Ok(())
    }

    #[async_std::test]
    async fn on_network_registry_event_deregistered_by_manager() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        db.set_access_in_network_registry(None, *SELF_CHAIN_ADDRESS, true)
            .await?;

        let registered_log = SerializableLog {
            address: handlers.addresses.network_registry.to_hex(),
            topics: vec![
                format!("{:#x}", DeregisteredByManagerFilter::signature()),
                format!("{:#x}", H256::from_slice(&STAKE_ADDRESS.to_bytes32())),
                format!("{:#x}", H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32())),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        assert!(db.is_allowed_in_network_registry(None, *SELF_CHAIN_ADDRESS).await?);

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, registered_log.into()).await }))
            .await?;

        assert!(
            matches!(event_type, Some(ChainEventType::NetworkRegistryUpdate(a, s)) if a == *SELF_CHAIN_ADDRESS && s == NetworkRegistryStatus::Denied),
            "must return correct NR update"
        );

        assert!(
            !db.is_allowed_in_network_registry(None, *SELF_CHAIN_ADDRESS).await?,
            "must not be allowed in NR"
        );
        Ok(())
    }

    #[async_std::test]
    async fn on_network_registry_event_enabled() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        let nr_enabled = SerializableLog {
            address: handlers.addresses.network_registry.to_hex(),
            topics: vec![
                format!("{:#x}", NetworkRegistryStatusUpdatedFilter::signature()),
                format!("{:#x}", H256::from_low_u64_be(1)),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, nr_enabled.into()).await }))
            .await?;

        assert!(event_type.is_none(), "there's no chain event type for nr disable");

        assert!(db.get_indexer_data(None).await?.nr_enabled);
        Ok(())
    }

    #[async_std::test]
    async fn on_network_registry_event_disabled() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        db.set_network_registry_enabled(None, true).await?;

        let nr_disabled = SerializableLog {
            address: handlers.addresses.network_registry.to_hex(),
            topics: vec![
                format!("{:#x}", NetworkRegistryStatusUpdatedFilter::signature()),
                format!("{:#x}", H256::from_low_u64_be(0)),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, nr_disabled.into()).await }))
            .await?;

        assert!(event_type.is_none(), "there's no chain event type for nr enable");

        assert!(!db.get_indexer_data(None).await?.nr_enabled);
        Ok(())
    }

    #[async_std::test]
    async fn on_network_registry_set_eligible() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        let set_eligible = SerializableLog {
            address: handlers.addresses.network_registry.to_hex(),
            topics: vec![
                format!("{:#x}", EligibilityUpdatedFilter::signature()),
                format!("{:#x}", H256::from_slice(&STAKE_ADDRESS.to_bytes32())),
                format!("{:#x}", H256::from_low_u64_be(1)),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, set_eligible.into()).await }))
            .await?;

        assert!(
            event_type.is_none(),
            "there's no chain event type for setting nr eligibility"
        );

        assert!(db.is_safe_eligible(None, *STAKE_ADDRESS).await?);

        Ok(())
    }

    #[async_std::test]
    async fn on_network_registry_set_not_eligible() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        db.set_safe_eligibility(None, *STAKE_ADDRESS, false).await?;

        let set_eligible = SerializableLog {
            address: handlers.addresses.network_registry.to_hex(),
            topics: vec![
                format!("{:#x}", EligibilityUpdatedFilter::signature()),
                format!("{:#x}", H256::from_slice(&STAKE_ADDRESS.to_bytes32())),
                format!("{:#x}", H256::from_low_u64_be(0)),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, set_eligible.into()).await }))
            .await?;

        assert!(
            event_type.is_none(),
            "there's no chain event type for unsetting nr eligibility"
        );

        assert!(!db.is_safe_eligible(None, *STAKE_ADDRESS).await?);

        Ok(())
    }

    #[async_std::test]
    async fn on_channel_event_balance_increased() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        let channel = ChannelEntry::new(
            *SELF_CHAIN_ADDRESS,
            *COUNTERPARTY_CHAIN_ADDRESS,
            Balance::new(U256::zero(), BalanceType::HOPR),
            U256::zero(),
            ChannelStatus::Open,
            U256::one(),
        );

        db.upsert_channel(None, channel).await?;

        let solidity_balance = BalanceType::HOPR.balance(U256::from((1u128 << 96) - 1));
        let diff = solidity_balance - channel.balance;

        let balance_increased_log = SerializableLog {
            address: handlers.addresses.channels.to_hex(),
            topics: vec![
                format!("{:#x}", ChannelBalanceIncreasedFilter::signature()),
                format!("{:#x}", H256::from_slice(channel.get_id().as_ref())),
            ],
            data: Vec::from(solidity_balance.amount().to_be_bytes()).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, balance_increased_log.into()).await }))
            .await?;

        let channel = db
            .get_channel_by_id(None, &channel.get_id())
            .await?
            .context("a value should be present")?;

        assert!(
            matches!(event_type, Some(ChainEventType::ChannelBalanceIncreased(c, b)) if c == channel && b == diff),
            "must return updated channel entry and balance diff"
        );

        assert_eq!(solidity_balance, channel.balance, "balance must be updated");
        Ok(())
    }

    #[async_std::test]
    async fn on_channel_event_domain_separator_updated() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        let separator = Hash::from(hopr_crypto_random::random_bytes());

        let channels_dst_updated = SerializableLog {
            address: handlers.addresses.channels.to_hex(),
            topics: vec![
                format!("{:#x}", DomainSeparatorUpdatedFilter::signature()),
                format!("{:#x}", H256::from_slice(separator.as_ref())),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        assert!(db.get_indexer_data(None).await?.channels_dst.is_none());

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, channels_dst_updated.into()).await }))
            .await?;

        assert!(
            event_type.is_none(),
            "there's no chain event type for channel dst update"
        );

        assert_eq!(
            separator,
            db.get_indexer_data(None)
                .await?
                .channels_dst
                .context("a value should be present")?,
            "separator must be updated"
        );
        Ok(())
    }

    #[async_std::test]
    async fn on_channel_event_balance_decreased() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        let channel = ChannelEntry::new(
            *SELF_CHAIN_ADDRESS,
            *COUNTERPARTY_CHAIN_ADDRESS,
            Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
            U256::zero(),
            ChannelStatus::Open,
            U256::one(),
        );

        db.upsert_channel(None, channel).await?;

        let solidity_balance = U256::from((1u128 << 96) - 2);
        let diff = channel.balance - solidity_balance;

        let balance_decreased_log = SerializableLog {
            address: handlers.addresses.channels.to_hex(),
            topics: vec![
                format!("{:#x}", ChannelBalanceDecreasedFilter::signature()),
                format!("{:#x}", H256::from_slice(channel.get_id().as_ref())),
            ],
            data: Vec::from(solidity_balance.to_be_bytes()).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, balance_decreased_log.into()).await }))
            .await?;

        let channel = db
            .get_channel_by_id(None, &channel.get_id())
            .await?
            .context("a value should be present")?;

        assert!(
            matches!(event_type, Some(ChainEventType::ChannelBalanceDecreased(c, b)) if c == channel && b == diff),
            "must return updated channel entry and balance diff"
        );

        assert_eq!(solidity_balance, channel.balance.amount(), "balance must be updated");
        Ok(())
    }

    #[async_std::test]
    async fn on_channel_closed() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

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

        db.upsert_channel(None, channel).await?;

        let channel_closed_log = SerializableLog {
            address: handlers.addresses.channels.to_hex(),
            topics: vec![
                format!("{:#x}", ChannelClosedFilter::signature()),
                format!("{:#x}", H256::from_slice(channel.get_id().as_ref())),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, channel_closed_log.into()).await }))
            .await?;

        let closed_channel = db
            .get_channel_by_id(None, &channel.get_id())
            .await?
            .context("a value should be present")?;

        assert!(
            matches!(event_type, Some(ChainEventType::ChannelClosed(c)) if c == closed_channel),
            "must return the updated channel entry"
        );

        assert_eq!(closed_channel.status, ChannelStatus::Closed);
        assert_eq!(closed_channel.ticket_index, 0u64.into());
        assert_eq!(
            0,
            db.get_outgoing_ticket_index(closed_channel.get_id())
                .await?
                .load(Ordering::Relaxed)
        );

        assert!(closed_channel.balance.amount().eq(&U256::zero()));
        Ok(())
    }

    #[async_std::test]
    async fn on_channel_opened() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        let channel_id = generate_channel_id(&SELF_CHAIN_ADDRESS, &COUNTERPARTY_CHAIN_ADDRESS);

        let channel_opened_log = SerializableLog {
            address: handlers.addresses.channels.to_hex(),
            topics: vec![
                format!("{:#x}", ChannelOpenedFilter::signature()),
                format!("{:#x}", H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32())),
                format!("{:#x}", H256::from_slice(&COUNTERPARTY_CHAIN_ADDRESS.to_bytes32())),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, channel_opened_log.into()).await }))
            .await?;

        let channel = db
            .get_channel_by_id(None, &channel_id)
            .await?
            .context("a value should be present")?;

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
                .await?
                .load(Ordering::Relaxed)
        );
        Ok(())
    }

    #[async_std::test]
    async fn on_channel_reopened() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        let channel = ChannelEntry::new(
            *SELF_CHAIN_ADDRESS,
            *COUNTERPARTY_CHAIN_ADDRESS,
            Balance::zero(BalanceType::HOPR),
            U256::zero(),
            ChannelStatus::Closed,
            3.into(),
        );

        db.upsert_channel(None, channel).await?;

        let channel_opened_log = SerializableLog {
            address: handlers.addresses.channels.to_hex(),
            topics: vec![
                format!("{:#x}", ChannelOpenedFilter::signature()),
                format!("{:#x}", H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32())),
                format!("{:#x}", H256::from_slice(&COUNTERPARTY_CHAIN_ADDRESS.to_bytes32())),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, channel_opened_log.into()).await }))
            .await?;

        let channel = db
            .get_channel_by_id(None, &channel.get_id())
            .await?
            .context("a value should be present")?;

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
                .await?
                .load(Ordering::Relaxed)
        );
        Ok(())
    }

    #[async_std::test]
    async fn on_channel_should_not_reopen_when_not_closed() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        let channel = ChannelEntry::new(
            *SELF_CHAIN_ADDRESS,
            *COUNTERPARTY_CHAIN_ADDRESS,
            Balance::zero(BalanceType::HOPR),
            U256::zero(),
            ChannelStatus::Open,
            3.into(),
        );

        db.upsert_channel(None, channel).await?;

        let channel_opened_log = SerializableLog {
            address: handlers.addresses.channels.to_hex(),
            topics: vec![
                format!("{:#x}", ChannelOpenedFilter::signature()),
                format!("{:#x}", H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32())),
                format!("{:#x}", H256::from_slice(&COUNTERPARTY_CHAIN_ADDRESS.to_bytes32())),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        db.begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, channel_opened_log.into()).await }))
            .await
            .expect_err("should not re-open channel that is not Closed");
        Ok(())
    }

    fn mock_acknowledged_ticket(
        signer: &ChainKeypair,
        destination: &ChainKeypair,
        index: u64,
    ) -> anyhow::Result<AcknowledgedTicket> {
        let price_per_packet: U256 = 20_u32.into();
        let ticket_win_prob = 1.0f64;

        let channel_id = generate_channel_id(&signer.into(), &destination.into());

        let channel_epoch = 1u64;
        let domain_separator = Hash::default();

        let response = Response::try_from(
            Hash::create(&[channel_id.as_ref(), &channel_epoch.to_be_bytes(), &index.to_be_bytes()]).as_ref(),
        )?;

        Ok(TicketBuilder::default()
            .direction(&signer.into(), &destination.into())
            .amount(price_per_packet.div_f64(ticket_win_prob)?)
            .index(index)
            .index_offset(1)
            .win_prob(ticket_win_prob)
            .channel_epoch(1)
            .challenge(response.to_challenge().into())
            .build_signed(signer, &domain_separator)?
            .into_acknowledged(response))
    }

    #[async_std::test]
    async fn on_channel_ticket_redeemed_incoming_channel() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Hash::default())
            .await?;

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

        let mut ticket = mock_acknowledged_ticket(&COUNTERPARTY_CHAIN_KEY, &SELF_CHAIN_KEY, ticket_index.as_u64())?;
        ticket.status = AcknowledgedTicketStatus::BeingRedeemed;

        let ticket_value = ticket.verified_ticket().amount;

        db.upsert_channel(None, channel).await?;
        db.upsert_ticket(None, ticket.clone()).await?;

        let ticket_redeemed_log = SerializableLog {
            address: handlers.addresses.channels.to_hex(),
            topics: vec![
                format!("{:#x}", TicketRedeemedFilter::signature()),
                format!("{:#x}", H256::from_slice(channel.get_id().as_ref())),
            ],
            data: Vec::from(next_ticket_index.to_be_bytes()).into(),
            ..test_log()
        };

        let outgoing_ticket_index_before = db
            .get_outgoing_ticket_index(channel.get_id())
            .await?
            .load(Ordering::Relaxed);

        let stats = db.get_ticket_statistics(Some(channel.get_id())).await?;
        assert_eq!(
            BalanceType::HOPR.zero(),
            stats.redeemed_value,
            "there should not be any redeemed value"
        );
        assert_eq!(
            BalanceType::HOPR.zero(),
            stats.neglected_value,
            "there should not be any neglected value"
        );

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, ticket_redeemed_log.into()).await }))
            .await?;

        let channel = db
            .get_channel_by_id(None, &channel.get_id())
            .await?
            .context("a value should be present")?;

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
            .await?
            .load(Ordering::Relaxed);

        assert_eq!(
            outgoing_ticket_index_before, outgoing_ticket_index_after,
            "outgoing ticket index must not change"
        );

        let tickets = db.get_tickets((&channel).into()).await?;
        assert!(tickets.is_empty(), "there should not be any tickets left");

        let stats = db.get_ticket_statistics(Some(channel.get_id())).await?;
        assert_eq!(
            ticket_value, stats.redeemed_value,
            "there should be redeemed value worth 1 ticket"
        );
        assert_eq!(
            BalanceType::HOPR.zero(),
            stats.neglected_value,
            "there should not be any neglected ticket"
        );
        Ok(())
    }

    #[async_std::test]
    async fn on_channel_ticket_redeemed_incoming_channel_neglect_left_over_tickets() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Hash::default())
            .await?;

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

        let mut ticket = mock_acknowledged_ticket(&COUNTERPARTY_CHAIN_KEY, &SELF_CHAIN_KEY, ticket_index.as_u64())?;
        ticket.status = AcknowledgedTicketStatus::BeingRedeemed;

        let ticket_value = ticket.verified_ticket().amount;

        db.upsert_channel(None, channel).await?;
        db.upsert_ticket(None, ticket.clone()).await?;

        let old_ticket = mock_acknowledged_ticket(&COUNTERPARTY_CHAIN_KEY, &SELF_CHAIN_KEY, ticket_index.as_u64() - 1)?;
        db.upsert_ticket(None, old_ticket.clone()).await?;

        let ticket_redeemed_log = SerializableLog {
            address: handlers.addresses.channels.to_hex(),
            topics: vec![
                format!("{:#x}", TicketRedeemedFilter::signature()),
                format!("{:#x}", H256::from_slice(&channel.get_id().as_ref())),
            ],
            data: Vec::from(next_ticket_index.to_be_bytes()).into(),
            ..test_log()
        };

        let outgoing_ticket_index_before = db
            .get_outgoing_ticket_index(channel.get_id())
            .await?
            .load(Ordering::Relaxed);

        let stats = db.get_ticket_statistics(Some(channel.get_id())).await?;
        assert_eq!(
            BalanceType::HOPR.zero(),
            stats.redeemed_value,
            "there should not be any redeemed value"
        );
        assert_eq!(
            BalanceType::HOPR.zero(),
            stats.neglected_value,
            "there should not be any neglected value"
        );

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, ticket_redeemed_log.into()).await }))
            .await?;

        let channel = db
            .get_channel_by_id(None, &channel.get_id())
            .await?
            .context("a value should be present")?;

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
            .await?
            .load(Ordering::Relaxed);

        assert_eq!(
            outgoing_ticket_index_before, outgoing_ticket_index_after,
            "outgoing ticket index must not change"
        );

        let tickets = db.get_tickets((&channel).into()).await?;
        assert!(tickets.is_empty(), "there should not be any tickets left");

        let stats = db.get_ticket_statistics(Some(channel.get_id())).await?;
        assert_eq!(
            ticket_value, stats.redeemed_value,
            "there should be redeemed value worth 1 ticket"
        );
        assert_eq!(
            ticket_value, stats.neglected_value,
            "there should neglected value worth 1 ticket"
        );
        Ok(())
    }

    #[async_std::test]
    async fn on_channel_ticket_redeemed_outgoing_channel() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Hash::default())
            .await?;

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

        db.upsert_channel(None, channel).await?;

        let ticket_redeemed_log = SerializableLog {
            address: handlers.addresses.channels.to_hex(),
            topics: vec![
                format!("{:#x}", TicketRedeemedFilter::signature()),
                format!("{:#x}", H256::from_slice(channel.get_id().as_ref())),
            ],
            data: Vec::from(next_ticket_index.to_be_bytes()).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, ticket_redeemed_log.into()).await }))
            .await?;

        let channel = db
            .get_channel_by_id(None, &channel.get_id())
            .await?
            .context("a value should be present")?;

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
            .await?
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
        Ok(())
    }

    #[async_std::test]
    async fn on_channel_ticket_redeemed_on_incoming_channel_with_non_existent_ticket_should_pass() -> anyhow::Result<()>
    {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Hash::default())
            .await?;

        let handlers = init_handlers(db.clone());

        let channel = ChannelEntry::new(
            *COUNTERPARTY_CHAIN_ADDRESS,
            *SELF_CHAIN_ADDRESS,
            Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
            U256::zero(),
            ChannelStatus::Open,
            U256::one(),
        );

        db.upsert_channel(None, channel).await?;

        let next_ticket_index = U256::from((1u128 << 48) - 1);

        let ticket_redeemed_log = SerializableLog {
            address: handlers.addresses.channels.to_hex(),
            topics: vec![
                format!("{:#x}", TicketRedeemedFilter::signature()),
                format!("{:#x}", H256::from_slice(channel.get_id().as_ref())),
            ],
            data: Vec::from(next_ticket_index.to_be_bytes()).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, ticket_redeemed_log.into()).await }))
            .await?;

        let channel = db
            .get_channel_by_id(None, &channel.get_id())
            .await?
            .context("a value should be present")?;

        assert!(
            matches!(event_type, Some(ChainEventType::TicketRedeemed(c, None)) if c == channel),
            "must return updated channel entry and no ticket"
        );

        assert_eq!(
            channel.ticket_index, next_ticket_index,
            "channel entry must contain next ticket index"
        );
        Ok(())
    }

    #[async_std::test]
    async fn on_channel_ticket_redeemed_on_foreign_channel_should_pass() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        let channel = ChannelEntry::new(
            Address::from(hopr_crypto_random::random_bytes()),
            Address::from(hopr_crypto_random::random_bytes()),
            Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
            U256::zero(),
            ChannelStatus::Open,
            U256::one(),
        );

        db.upsert_channel(None, channel).await?;

        let next_ticket_index = U256::from((1u128 << 48) - 1);

        let ticket_redeemed_log = SerializableLog {
            address: handlers.addresses.channels.to_hex(),
            topics: vec![
                format!("{:#x}", TicketRedeemedFilter::signature()),
                format!("{:#x}", H256::from_slice(channel.get_id().as_ref())),
            ],
            data: Vec::from(next_ticket_index.to_be_bytes()).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, ticket_redeemed_log.into()).await }))
            .await?;

        let channel = db
            .get_channel_by_id(None, &channel.get_id())
            .await?
            .context("a value should be present")?;

        assert!(
            matches!(event_type, Some(ChainEventType::TicketRedeemed(c, None)) if c == channel),
            "must return updated channel entry and no ticket"
        );

        assert_eq!(
            channel.ticket_index, next_ticket_index,
            "channel entry must contain next ticket index"
        );
        Ok(())
    }

    #[async_std::test]
    async fn on_channel_closure_initiated() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        let channel = ChannelEntry::new(
            *SELF_CHAIN_ADDRESS,
            *COUNTERPARTY_CHAIN_ADDRESS,
            Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
            U256::zero(),
            ChannelStatus::Open,
            U256::one(),
        );

        db.upsert_channel(None, channel).await?;

        let timestamp = SystemTime::now();

        let closure_initiated_log = SerializableLog {
            address: handlers.addresses.channels.to_hex(),
            topics: vec![
                format!("{:#x}", OutgoingChannelClosureInitiatedFilter::signature()),
                format!("{:#x}", H256::from_slice(channel.get_id().as_ref())),
            ],
            data: Vec::from(U256::from(timestamp.as_unix_timestamp().as_secs()).to_be_bytes()).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, closure_initiated_log.into()).await }))
            .await?;

        let channel = db
            .get_channel_by_id(None, &channel.get_id())
            .await?
            .context("a value should be present")?;

        assert!(
            matches!(event_type, Some(ChainEventType::ChannelClosureInitiated(c)) if c == channel),
            "must return updated channel entry"
        );

        assert_eq!(
            channel.status,
            ChannelStatus::PendingToClose(timestamp),
            "channel status must match"
        );
        Ok(())
    }

    #[async_std::test]
    async fn on_node_safe_registry_registered() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        let safe_registered_log = SerializableLog {
            address: handlers.addresses.safe_registry.to_hex(),
            topics: vec![
                format!("{:#x}", RegisteredNodeSafeFilter::signature()),
                format!("{:#x}", H256::from_slice(&SAFE_INSTANCE_ADDR.to_bytes32())),
                format!("{:#x}", H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32())),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, safe_registered_log.into()).await }))
            .await?;

        assert!(matches!(event_type, Some(ChainEventType::NodeSafeRegistered(addr)) if addr == *SAFE_INSTANCE_ADDR));

        // Nothing to check in the DB here, since we do not track this
        Ok(())
    }

    #[async_std::test]
    async fn on_node_safe_registry_deregistered() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        // Nothing to write to the DB here, since we do not track this

        let safe_registered_log = SerializableLog {
            address: handlers.addresses.safe_registry.to_hex(),
            topics: vec![
                format!("{:#x}", DergisteredNodeSafeFilter::signature()),
                format!("{:#x}", H256::from_slice(&SAFE_INSTANCE_ADDR.to_bytes32())),
                format!("{:#x}", H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32())),
            ],
            data: encode(&[]).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, safe_registered_log.into()).await }))
            .await?;

        assert!(
            event_type.is_none(),
            "there's no associated chain event type with safe deregistration"
        );

        // Nothing to check in the DB here, since we do not track this
        Ok(())
    }

    #[async_std::test]
    async fn ticket_price_update() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let handlers = init_handlers(db.clone());

        let price_change_log = SerializableLog {
            address: handlers.addresses.price_oracle.to_hex(),
            topics: vec![format!("{:#x}", TicketPriceUpdatedFilter::signature())],
            data: encode(&[Token::Uint(EthU256::from(1u64)), Token::Uint(EthU256::from(123u64))]).into(),
            ..test_log()
        };

        assert_eq!(db.get_indexer_data(None).await?.ticket_price, None);

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, price_change_log.into()).await }))
            .await?;

        assert!(
            event_type.is_none(),
            "there's no associated chain event type with price oracle"
        );

        assert_eq!(
            db.get_indexer_data(None).await?.ticket_price.map(|p| p.amount()),
            Some(U256::from(123u64))
        );
        Ok(())
    }
}
