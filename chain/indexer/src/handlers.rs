use std::{
    cmp::Ordering,
    fmt::{Debug, Formatter},
    ops::Add,
    sync::Arc,
    time::{Duration, SystemTime},
};

use alloy::{primitives::B256, sol_types::SolEventInterface};
use async_trait::async_trait;
use hopr_bindings::{
    hoprannouncements::HoprAnnouncements::HoprAnnouncementsEvents, hoprchannels::HoprChannels::HoprChannelsEvents,
    hoprnetworkregistry::HoprNetworkRegistry::HoprNetworkRegistryEvents,
    hoprnodemanagementmodule::HoprNodeManagementModule::HoprNodeManagementModuleEvents,
    hoprnodesaferegistry::HoprNodeSafeRegistry::HoprNodeSafeRegistryEvents,
    hoprticketpriceoracle::HoprTicketPriceOracle::HoprTicketPriceOracleEvents, hoprtoken::HoprToken::HoprTokenEvents,
    hoprwinningprobabilityoracle::HoprWinningProbabilityOracle::HoprWinningProbabilityOracleEvents,
};
use hopr_chain_rpc::{HoprIndexerRpcOperations, Log};
use hopr_chain_types::{
    ContractAddresses,
    chain_events::{ChainEventType, NetworkRegistryStatus, SignificantChainEvent},
};
use hopr_crypto_types::{
    keypairs::ChainKeypair,
    prelude::{Hash, Keypair},
    types::OffchainSignature,
};
use hopr_db_sql::{
    HoprDbAllOperations, OpenTransaction,
    api::{info::DomainSeparator, tickets::TicketSelector},
    errors::DbSqlError,
    prelude::TicketMarker,
};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use tracing::{debug, error, info, trace, warn};

use crate::errors::{CoreEthereumIndexerError, Result};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_INDEXER_LOG_COUNTERS: hopr_metrics::MultiCounter =
        hopr_metrics::MultiCounter::new(
            "hopr_indexer_contract_log_count",
            "Counts of different HOPR contract logs processed by the Indexer",
            &["contract"]
    ).unwrap();
}

/// Event handling an object for on-chain operations
///
/// Once an on-chain operation is recorded by the [crate::block::Indexer], it is pre-processed
/// and passed on to this object that handles event-specific actions for each on-chain operation.
#[derive(Clone)]
pub struct ContractEventHandlers<T, Db> {
    /// channels, announcements, network_registry, token: contract addresses
    /// whose event we process
    addresses: Arc<ContractAddresses>,
    /// Safe on-chain address which we are monitoring
    safe_address: Address,
    /// own address, aka message sender
    chain_key: ChainKeypair, // TODO: store only address here once Ticket have TryFrom DB
    /// callbacks to inform other modules
    db: Db,
    /// rpc operations to interact with the chain
    rpc_operations: T,
}

impl<T, Db> Debug for ContractEventHandlers<T, Db> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContractEventHandler")
            .field("addresses", &self.addresses)
            .field("safe_address", &self.safe_address)
            .field("chain_key", &self.chain_key)
            .finish_non_exhaustive()
    }
}

impl<T, Db> ContractEventHandlers<T, Db>
where
    T: HoprIndexerRpcOperations + Clone + Send + 'static,
    Db: HoprDbAllOperations + Clone,
{
    /// Creates a new instance of contract event handlers with RPC operations support.
    ///
    /// This constructor initializes the event handlers with all necessary dependencies
    /// for processing blockchain events and making direct RPC calls for fresh state data.
    ///
    /// # Type Parameters
    /// * `T` - Type implementing `HoprIndexerRpcOperations` for blockchain queries
    ///
    /// # Arguments
    /// * `addresses` - Contract addresses configuration
    /// * `safe_address` - The node's safe address for filtering relevant events
    /// * `chain_key` - Cryptographic key for chain operations
    /// * `db` - Database connection for persistent storage
    /// * `rpc_operations` - RPC interface for direct blockchain queries
    ///
    /// # Returns
    /// * `Self` - New instance of `ContractEventHandlers`
    pub fn new(
        addresses: ContractAddresses,
        safe_address: Address,
        chain_key: ChainKeypair,
        db: Db,
        rpc_operations: T,
    ) -> Self {
        Self {
            addresses: Arc::new(addresses),
            safe_address,
            chain_key,
            db,
            rpc_operations,
        }
    }

    async fn on_announcement_event(
        &self,
        tx: &OpenTransaction,
        event: HoprAnnouncementsEvents,
        block_number: u32,
        _is_synced: bool,
    ) -> Result<Option<ChainEventType>> {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_INDEXER_LOG_COUNTERS.increment(&["announcements"]);

        match event {
            HoprAnnouncementsEvents::AddressAnnouncement(address_announcement) => {
                debug!(
                    multiaddress = &address_announcement.baseMultiaddr,
                    address = &address_announcement.node.to_string(),
                    "on_announcement_event: AddressAnnouncement",
                );
                // safeguard against empty multiaddrs, skip
                if address_announcement.baseMultiaddr.is_empty() {
                    warn!(
                        address = %address_announcement.node,
                        "encountered empty multiaddress announcement",
                    );
                    return Ok(None);
                }
                let node_address: Address = address_announcement.node.into();

                return match self
                    .db
                    .insert_announcement(
                        Some(tx),
                        node_address,
                        address_announcement.baseMultiaddr.parse()?,
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
            HoprAnnouncementsEvents::KeyBinding(key_binding) => {
                debug!(
                    address = %key_binding.chain_key,
                    public_key = %key_binding.ed25519_pub_key,
                    "on_announcement_event: KeyBinding",
                );
                match KeyBinding::from_parts(
                    key_binding.chain_key.into(),
                    key_binding.ed25519_pub_key.0.try_into()?,
                    OffchainSignature::try_from((key_binding.ed25519_sig_0.0, key_binding.ed25519_sig_1.0))?,
                ) {
                    Ok(binding) => {
                        match self
                            .db
                            .insert_account(
                                Some(tx),
                                AccountEntry {
                                    public_key: binding.packet_key,
                                    chain_addr: binding.chain_key,
                                    entry_type: AccountType::NotAnnounced,
                                    published_at: block_number,
                                },
                            )
                            .await
                        {
                            Ok(_) => (),
                            Err(err) => {
                                // We handle these errors gracefully and don't want the indexer to crash,
                                // because anybody could write faulty entries into the announcement contract.
                                error!(%err, "failed to store announcement key binding")
                            }
                        }
                    }
                    Err(e) => {
                        warn!(
                            address = ?key_binding.chain_key,
                            error = %e,
                            "Filtering announcement with invalid signature",

                        )
                    }
                }
            }
            HoprAnnouncementsEvents::RevokeAnnouncement(revocation) => {
                let node_address: Address = revocation.node.into();
                match self.db.delete_all_announcements(Some(tx), node_address).await {
                    Err(DbSqlError::MissingAccount) => {
                        return Err(CoreEthereumIndexerError::RevocationBeforeKeyBinding);
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
        is_synced: bool,
    ) -> Result<Option<ChainEventType>> {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_INDEXER_LOG_COUNTERS.increment(&["channels"]);

        match event {
            HoprChannelsEvents::ChannelBalanceDecreased(balance_decreased) => {
                let channel_id = balance_decreased.channelId.0.into();

                let maybe_channel = match self.db.begin_channel_update(tx.into(), &channel_id).await {
                    Ok(channel) => channel,
                    Err(DbSqlError::CorruptedChannelEntry(_)) => {
                        error!(%channel_id, "failed to begin channel update on on_channel_balance_decreased_event: corrupted channel entry");
                        return Ok(None);
                    }
                    Err(e) => {
                        error!(%channel_id, %e, "failed to begin channel update on on_channel_balance_decreased_event");
                        return Err(e.into());
                    }
                };

                trace!(
                    %channel_id,
                    is_channel = maybe_channel.is_some(),
                    "on_channel_balance_decreased_event",
                );

                if let Some(channel_edits) = maybe_channel {
                    let new_balance = HoprBalance::from_be_bytes(balance_decreased.newBalance.to_be_bytes::<12>());
                    let diff = channel_edits.entry().balance - new_balance;

                    let updated_channel = self
                        .db
                        .finish_channel_update(tx.into(), channel_edits.change_balance(new_balance))
                        .await?
                        .ok_or(CoreEthereumIndexerError::ProcessError(format!(
                            "channel balance decreased event for channel {channel_id} did not return an updated \
                             channel"
                        )))?;

                    if is_synced
                        && (updated_channel.source == self.chain_key.public().to_address()
                            || updated_channel.destination == self.chain_key.public().to_address())
                    {
                        info!("updating safe balance from chain after channel balance decreased event");
                        match self.rpc_operations.get_hopr_balance(self.safe_address).await {
                            Ok(balance) => {
                                self.db.set_safe_hopr_balance(Some(tx), balance).await?;
                            }
                            Err(error) => {
                                error!(%error, "error getting safe balance from chain after channel balance decreased event");
                            }
                        }
                    }

                    Ok(Some(ChainEventType::ChannelBalanceDecreased(updated_channel, diff)))
                } else {
                    error!(%channel_id, "observed balance decreased event for a channel that does not exist");
                    self.db.insert_corrupted_channel(tx.into(), channel_id).await?;
                    Err(CoreEthereumIndexerError::ChannelDoesNotExist)
                }
            }
            HoprChannelsEvents::ChannelBalanceIncreased(balance_increased) => {
                let channel_id = balance_increased.channelId.0.into();

                let maybe_channel = match self.db.begin_channel_update(tx.into(), &channel_id).await {
                    Ok(channel) => channel,
                    Err(DbSqlError::CorruptedChannelEntry(_)) => {
                        error!(%channel_id, "failed to begin channel update on on_channel_balance_increased_event: corrupted channel entry");
                        return Ok(None);
                    }
                    Err(e) => {
                        error!(%channel_id, %e, "failed to begin channel update on on_channel_balance_increased_event");
                        return Err(e.into());
                    }
                };

                trace!(
                    channel_id = %channel_id,
                    is_channel = maybe_channel.is_some(),
                    "on_channel_balance_increased_event",
                );

                if let Some(channel_edits) = maybe_channel {
                    let new_balance = HoprBalance::from_be_bytes(balance_increased.newBalance.to_be_bytes::<12>());
                    let diff = new_balance - channel_edits.entry().balance;

                    let updated_channel = self
                        .db
                        .finish_channel_update(tx.into(), channel_edits.change_balance(new_balance))
                        .await?
                        .ok_or(CoreEthereumIndexerError::ProcessError(format!(
                            "channel balance increased event for channel {channel_id} did not return an updated \
                             channel"
                        )))?;

                    if updated_channel.source == self.chain_key.public().to_address() && is_synced {
                        info!("updating safe balance from chain after channel balance increased event");
                        match self.rpc_operations.get_hopr_balance(self.safe_address).await {
                            Ok(balance) => {
                                self.db.set_safe_hopr_balance(Some(tx), balance).await?;
                            }
                            Err(error) => {
                                error!(%error, "error getting safe balance from chain after channel balance increased event");
                            }
                        }
                        info!("updating safe allowance from chain after channel balance increased event");
                        match self
                            .rpc_operations
                            .get_hopr_allowance(self.safe_address, self.addresses.channels)
                            .await
                        {
                            Ok(allowance) => {
                                self.db.set_safe_hopr_allowance(Some(tx), allowance).await?;
                            }
                            Err(error) => {
                                error!(%error, "error getting safe allowance from chain after channel balance increased event");
                            }
                        }
                    }

                    Ok(Some(ChainEventType::ChannelBalanceIncreased(updated_channel, diff)))
                } else {
                    error!(%channel_id, "observed balance increased event for a channel that does not exist");
                    self.db.insert_corrupted_channel(tx.into(), channel_id).await?;
                    Err(CoreEthereumIndexerError::ChannelDoesNotExist)
                }
            }
            HoprChannelsEvents::ChannelClosed(channel_closed) => {
                let channel_id = Hash::from(channel_closed.channelId.0);

                let maybe_channel = match self.db.begin_channel_update(tx.into(), &channel_id).await {
                    Ok(channel) => channel,
                    Err(DbSqlError::CorruptedChannelEntry(_)) => {
                        error!(%channel_id, "failed to begin channel update on on_channel_closed_event: corrupted channel entry");
                        return Ok(None);
                    }
                    Err(e) => {
                        error!(%channel_id, %e, "failed to begin channel update on on_channel_closed_event");
                        return Err(e.into());
                    }
                };

                trace!(
                    channel_id = %channel_id,
                    is_channel = maybe_channel.is_some(),
                    "on_channel_closed_event",
                );

                if let Some(channel_edits) = maybe_channel {
                    let channel_id = channel_edits.entry().get_id();
                    let orientation = channel_edits.entry().orientation(&self.chain_key.public().to_address());

                    // If the channel is our own (incoming or outgoing) reset its fields
                    // and change its state to Closed.
                    let updated_channel = if let Some((direction, _)) = orientation {
                        // Set all channel fields like we do on-chain on close
                        let channel_edits = channel_edits
                            .change_status(ChannelStatus::Closed)
                            .change_balance(HoprBalance::zero())
                            .change_ticket_index(0);

                        let updated_channel = self.db.finish_channel_update(tx.into(), channel_edits).await?.ok_or(
                            CoreEthereumIndexerError::ProcessError(format!(
                                "channel closed event for channel {channel_id} did not return an updated channel",
                            )),
                        )?;

                        // Perform additional tasks based on the channel's direction
                        match direction {
                            ChannelDirection::Incoming => {
                                // On incoming channel, mark all unredeemed tickets as neglected
                                self.db
                                    .mark_tickets_as(updated_channel.into(), TicketMarker::Neglected)
                                    .await?;
                            }
                            ChannelDirection::Outgoing => {
                                // On outgoing channels, reset the current_ticket_index to zero
                                self.db.reset_outgoing_ticket_index(channel_id).await?;
                            }
                        }
                        updated_channel
                    } else {
                        // Closed channels that are not our own, we be safely removed
                        // from the database
                        let updated_channel = self
                            .db
                            .finish_channel_update(tx.into(), channel_edits.delete())
                            .await?
                            .ok_or(CoreEthereumIndexerError::ProcessError(format!(
                                "channel closed event for channel {} did not return an updated channel",
                                Hash::from(channel_closed.channelId.0)
                            )))?;

                        debug!(channel_id = %channel_id, "foreign closed closed channel was deleted");
                        updated_channel
                    };

                    Ok(Some(ChainEventType::ChannelClosed(updated_channel)))
                } else {
                    error!(%channel_id, "observed closure finalization event for a channel that does not exist.");
                    self.db.insert_corrupted_channel(tx.into(), channel_id).await?;
                    Err(CoreEthereumIndexerError::ChannelDoesNotExist)
                }
            }
            HoprChannelsEvents::ChannelOpened(channel_opened) => {
                let source: Address = channel_opened.source.into();
                let destination: Address = channel_opened.destination.into();
                let channel_id = generate_channel_id(&source, &destination);

                let maybe_channel = match self.db.begin_channel_update(tx.into(), &channel_id).await {
                    Ok(channel) => channel,
                    Err(DbSqlError::CorruptedChannelEntry(_)) => {
                        error!(%source, %destination, %channel_id, "failed to begin channel update on on_channel_opened_event: corrupted channel entry");
                        return Ok(None);
                    }
                    Err(e) => {
                        error!(%source, %destination, %channel_id, %e, "failed to begin channel update on on_channel_opened_event");
                        return Err(e.into());
                    }
                };

                let channel = if let Some(channel_edits) = maybe_channel {
                    // Check that we're not receiving the Open event without the channel being Close prior, or that the
                    // channel is not yet corrupted

                    if channel_edits.entry().status != ChannelStatus::Closed {
                        warn!(%source, %destination, %channel_id, "received Open event for a channel that is not Closed, marking it as corrupted");

                        self.db
                            .finish_channel_update(tx.into(), channel_edits.set_corrupted())
                            .await?;

                        return Ok(None);
                    }

                    trace!(%source, %destination, %channel_id, "on_channel_reopened_event");

                    let current_epoch = channel_edits.entry().channel_epoch;

                    // cleanup tickets from previous epochs on channel re-opening
                    if source == self.chain_key.public().to_address()
                        || destination == self.chain_key.public().to_address()
                    {
                        self.db
                            .mark_tickets_as(TicketSelector::new(channel_id, current_epoch), TicketMarker::Neglected)
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
                        .ok_or(CoreEthereumIndexerError::ProcessError(format!(
                            "channel opened event for channel {channel_id} did not return an updated channel",
                        )))?
                } else {
                    trace!(%source, %destination, %channel_id, "on_channel_opened_event");

                    let new_channel = ChannelEntry::new(
                        source,
                        destination,
                        0_u32.into(),
                        0_u32.into(),
                        ChannelStatus::Open,
                        1_u32.into(),
                    );

                    self.db.upsert_channel(tx.into(), new_channel).await?;
                    new_channel
                };

                Ok(Some(ChainEventType::ChannelOpened(channel)))
            }
            HoprChannelsEvents::TicketRedeemed(ticket_redeemed) => {
                let channel_id = ticket_redeemed.channelId.0.into();

                let maybe_channel = match self.db.begin_channel_update(tx.into(), &channel_id).await {
                    Ok(channel) => channel,
                    Err(DbSqlError::CorruptedChannelEntry(_)) => {
                        error!(%channel_id, "failed to begin channel update on on_ticket_redeemed_event: corrupted channel entry");
                        return Ok(None);
                    }
                    Err(e) => {
                        error!(%channel_id, %e, "failed to begin channel update on on_ticket_redeemed_event");
                        return Err(e.into());
                    }
                };

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
                                    // The ticket that has been redeemed at this point has: index + index_offset - 1 ==
                                    // new_ticket_index - 1 Since unaggregated
                                    // tickets have index_offset = 1, for the unagg case this leads to: index ==
                                    // new_ticket_index - 1
                                    let ticket_idx = ticket.verified_ticket().index;
                                    let ticket_off = ticket.verified_ticket().index_offset as u64;

                                    ticket_idx + ticket_off == ticket_redeemed.newTicketIndex.to::<u64>()
                                })
                                .collect::<Vec<_>>();

                            match matching_tickets.len().cmp(&1) {
                                Ordering::Equal => {
                                    let ack_ticket = matching_tickets.pop().unwrap();

                                    self.db
                                        .mark_tickets_as((&ack_ticket).into(), TicketMarker::Redeemed)
                                        .await?;
                                    info!(%ack_ticket, "ticket marked as redeemed");
                                    Some(ack_ticket)
                                }
                                Ordering::Less => {
                                    error!(
                                        idx = %ticket_redeemed.newTicketIndex.to::<u64>() - 1,
                                        entry = %channel_edits.entry(),
                                        "could not find acknowledged 'BeingRedeemed' ticket",
                                    );
                                    // This is not an error, because the ticket might've become neglected before
                                    // the ticket redemption could finish
                                    None
                                }
                                Ordering::Greater => {
                                    error!(
                                        count = matching_tickets.len(),
                                        index = %ticket_redeemed.newTicketIndex.to::<u64>() - 1,
                                        entry = %channel_edits.entry(),
                                        "found tickets matching 'BeingRedeemed'",
                                    );

                                    let entry_str = channel_edits.entry().to_string();

                                    self.db
                                        .finish_channel_update(tx.into(), channel_edits.set_corrupted())
                                        .await?;

                                    return Err(CoreEthereumIndexerError::ProcessError(format!(
                                        "multiple tickets matching idx {} found in {}",
                                        ticket_redeemed.newTicketIndex.to::<u64>() - 1,
                                        entry_str
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
                            debug!(channel = %channel_edits.entry(), "observed redeem event on an outgoing channel");
                            self.db
                                .compare_and_set_outgoing_ticket_index(
                                    channel_edits.entry().get_id(),
                                    ticket_redeemed.newTicketIndex.to::<u64>(),
                                )
                                .await?;
                            None
                        }
                        // For a channel where neither source nor destination is us, we don't care
                        None => {
                            // Not our redeem event
                            debug!(channel = %channel_edits.entry(), "observed redeem event on a foreign channel");
                            None
                        }
                    };

                    // Update the ticket index on the Channel entry and get the updated model
                    let channel = self
                        .db
                        .finish_channel_update(
                            tx.into(),
                            channel_edits.change_ticket_index(U256::from_be_bytes(
                                ticket_redeemed.newTicketIndex.to_be_bytes::<6>(),
                            )),
                        )
                        .await?
                        .ok_or(CoreEthereumIndexerError::ProcessError(format!(
                            "ticket redeemed event for channel {channel_id} did not return an updated channel",
                        )))?;

                    // Neglect all the tickets in this channel
                    // which have a lower ticket index than `ticket_redeemed.new_ticket_index`
                    self.db
                        .mark_tickets_as(
                            TicketSelector::from(&channel)
                                .with_index_range(..ticket_redeemed.newTicketIndex.to::<u64>()),
                            TicketMarker::Neglected,
                        )
                        .await?;

                    Ok(Some(ChainEventType::TicketRedeemed(channel, ack_ticket)))
                } else {
                    error!(%channel_id, "observed ticket redeem on a channel that we don't have in the DB");
                    self.db.insert_corrupted_channel(tx.into(), channel_id).await?;
                    Err(CoreEthereumIndexerError::ChannelDoesNotExist)
                }
            }
            HoprChannelsEvents::OutgoingChannelClosureInitiated(closure_initiated) => {
                let channel_id = closure_initiated.channelId.0.into();
                let maybe_channel = match self.db.begin_channel_update(tx.into(), &channel_id).await {
                    Ok(channel) => channel,
                    Err(DbSqlError::CorruptedChannelEntry(_)) => {
                        error!(%channel_id, "failed to begin channel update on on_outgoing_channel_closure_initiated_event: corrupted channel entry");
                        return Ok(None);
                    }
                    Err(e) => {
                        error!(%channel_id, %e, "failed to begin channel update on on_outgoing_channel_closure_initiated_event");
                        return Err(e.into());
                    }
                };

                let closure_time: u32 = closure_initiated.closureTime;
                if let Some(channel_edits) = maybe_channel {
                    let new_status = ChannelStatus::PendingToClose(
                        SystemTime::UNIX_EPOCH.add(Duration::from_secs(closure_time.into())),
                    );

                    let channel = self
                        .db
                        .finish_channel_update(tx.into(), channel_edits.change_status(new_status))
                        .await?
                        .ok_or(CoreEthereumIndexerError::ProcessError(format!(
                            "channel closure initiation event for channel {} did not return an updated channel",
                            Hash::from(closure_initiated.channelId.0)
                        )))?;

                    Ok(Some(ChainEventType::ChannelClosureInitiated(channel)))
                } else {
                    error!(%channel_id, "observed channel closure initiation on a channel that we don't have in the DB");
                    self.db.insert_corrupted_channel(tx.into(), channel_id).await?;
                    Err(CoreEthereumIndexerError::ChannelDoesNotExist)
                }
            }
            HoprChannelsEvents::DomainSeparatorUpdated(domain_separator_updated) => {
                self.db
                    .set_domain_separator(
                        Some(tx),
                        DomainSeparator::Channel,
                        domain_separator_updated.domainSeparator.0.into(),
                    )
                    .await?;

                Ok(None)
            }
            HoprChannelsEvents::LedgerDomainSeparatorUpdated(ledger_domain_separator_updated) => {
                self.db
                    .set_domain_separator(
                        Some(tx),
                        DomainSeparator::Ledger,
                        ledger_domain_separator_updated.ledgerDomainSeparator.0.into(),
                    )
                    .await?;

                Ok(None)
            }
        }
    }

    async fn on_token_event(
        &self,
        tx: &OpenTransaction,
        event: HoprTokenEvents,
        is_synced: bool,
    ) -> Result<Option<ChainEventType>> {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_INDEXER_LOG_COUNTERS.increment(&["token"]);

        match event {
            HoprTokenEvents::Transfer(transferred) => {
                let from: Address = transferred.from.into();
                let to: Address = transferred.to.into();

                trace!(
                    safe_address = %&self.safe_address, %from, %to,
                    "on_token_transfer_event"
                );

                if to.ne(&self.safe_address) && from.ne(&self.safe_address) {
                    error!(
                    safe_address = %&self.safe_address, %from, %to,
                        "filter misconfiguration: transfer event not involving the safe");
                    return Ok(None);
                }

                if is_synced {
                    info!("updating safe balance from chain after transfer event");
                    match self.rpc_operations.get_hopr_balance(self.safe_address).await {
                        Ok(balance) => {
                            self.db.set_safe_hopr_balance(Some(tx), balance).await?;
                        }
                        Err(error) => {
                            error!(%error, "error getting safe balance from chain after transfer event");
                        }
                    }
                    info!("updating safe allowance from chain after transfer event");
                    match self
                        .rpc_operations
                        .get_hopr_allowance(self.safe_address, self.addresses.channels)
                        .await
                    {
                        Ok(allowance) => {
                            self.db.set_safe_hopr_allowance(Some(tx), allowance).await?;
                        }
                        Err(error) => {
                            error!(%error, "error getting safe allowance from chain after transfer event");
                        }
                    }
                }
            }
            HoprTokenEvents::Approval(approved) => {
                let owner: Address = approved.owner.into();
                let spender: Address = approved.spender.into();

                trace!(
                    address = %&self.safe_address, %owner, %spender, allowance = %approved.value,
                    "on_token_approval_event",

                );

                if owner.ne(&self.safe_address) || spender.ne(&self.addresses.channels) {
                    error!(
                        address = %&self.safe_address, %owner, %spender, allowance = %approved.value,
                        "filter misconfiguration: approval event not involving the safe and channels contract");
                    return Ok(None);
                }

                // if approval is for tokens on Safe contract to be spent by HoprChannels
                if is_synced {
                    info!("updating safe allowance from chain after approval event");
                    match self
                        .rpc_operations
                        .get_hopr_allowance(self.safe_address, self.addresses.channels)
                        .await
                    {
                        Ok(allowance) => {
                            self.db.set_safe_hopr_allowance(Some(tx), allowance).await?;
                        }
                        Err(error) => {
                            error!(%error, "error getting safe allowance from chain after approval event");
                        }
                    }
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
        _is_synced: bool,
    ) -> Result<Option<ChainEventType>> {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_INDEXER_LOG_COUNTERS.increment(&["network_registry"]);

        match event {
            HoprNetworkRegistryEvents::DeregisteredByManager(deregistered) => {
                debug!(
                    node_address = %deregistered.nodeAddress,
                    "on_network_registry_deregistered_by_manager_event",
                );
                let node_address: Address = deregistered.nodeAddress.into();
                self.db
                    .set_access_in_network_registry(Some(tx), node_address, false)
                    .await?;

                return Ok(Some(ChainEventType::NetworkRegistryUpdate(
                    node_address,
                    NetworkRegistryStatus::Denied,
                )));
            }
            HoprNetworkRegistryEvents::Deregistered(deregistered) => {
                debug!(
                    node_address = %deregistered.nodeAddress,
                    "on_network_registry_deregistered_event",
                );
                let node_address: Address = deregistered.nodeAddress.into();
                self.db
                    .set_access_in_network_registry(Some(tx), node_address, false)
                    .await?;

                return Ok(Some(ChainEventType::NetworkRegistryUpdate(
                    node_address,
                    NetworkRegistryStatus::Denied,
                )));
            }
            HoprNetworkRegistryEvents::RegisteredByManager(registered) => {
                let node_address: Address = registered.nodeAddress.into();
                debug!(
                    %node_address,
                    "Node registered by manager, adding to the network registry",
                );
                self.db
                    .set_access_in_network_registry(Some(tx), node_address, true)
                    .await?;

                if node_address == self.chain_key.public().to_address() {
                    info!(
                        "This node has been added to the registry, node activation process continues on: http://hub.hoprnet.org/."
                    );
                }

                return Ok(Some(ChainEventType::NetworkRegistryUpdate(
                    node_address,
                    NetworkRegistryStatus::Allowed,
                )));
            }
            HoprNetworkRegistryEvents::Registered(registered) => {
                let node_address: Address = registered.nodeAddress.into();
                debug!(
                    %node_address,
                    "Node registered, adding to the network registry",
                );
                self.db
                    .set_access_in_network_registry(Some(tx), node_address, true)
                    .await?;

                if node_address == self.chain_key.public().to_address() {
                    info!(
                        "This node has been added to the registry, node can now continue the node activation process on: http://hub.hoprnet.org/."
                    );
                }

                return Ok(Some(ChainEventType::NetworkRegistryUpdate(
                    node_address,
                    NetworkRegistryStatus::Allowed,
                )));
            }
            HoprNetworkRegistryEvents::EligibilityUpdated(eligibility_updated) => {
                debug!(
                    staking_account = %eligibility_updated.stakingAccount,
                    eligibility = %eligibility_updated.eligibility,
                    "Node eligibility updated, updating the safe eligibility",
                );
                let account: Address = eligibility_updated.stakingAccount.into();
                self.db
                    .set_safe_eligibility(Some(tx), account, eligibility_updated.eligibility)
                    .await?;
            }
            HoprNetworkRegistryEvents::NetworkRegistryStatusUpdated(enabled) => {
                debug!(enabled = enabled.isEnabled, "on_network_registry_status_updated_event",);
                self.db
                    .set_network_registry_enabled(Some(tx), enabled.isEnabled)
                    .await?;
            }
            _ => {
                debug!("on_network_registry_event - not implemented for event: {:?}", event);
            } // Not important to at the moment
        };

        Ok(None)
    }

    async fn on_node_safe_registry_event(
        &self,
        tx: &OpenTransaction,
        event: HoprNodeSafeRegistryEvents,
        _is_synced: bool,
    ) -> Result<Option<ChainEventType>> {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_INDEXER_LOG_COUNTERS.increment(&["safe_registry"]);

        match event {
            HoprNodeSafeRegistryEvents::RegisteredNodeSafe(registered) => {
                if self.chain_key.public().to_address() == registered.nodeAddress.into() {
                    info!(safe_address = %registered.safeAddress, "Node safe registered", );
                    // NOTE: we don't store this state in the DB
                    return Ok(Some(ChainEventType::NodeSafeRegistered(registered.safeAddress.into())));
                }
            }
            HoprNodeSafeRegistryEvents::DergisteredNodeSafe(deregistered) => {
                if self.chain_key.public().to_address() == deregistered.nodeAddress.into() {
                    info!("Node safe unregistered");
                    // NOTE: we don't store this state in the DB
                }
            }
            HoprNodeSafeRegistryEvents::DomainSeparatorUpdated(domain_separator_updated) => {
                self.db
                    .set_domain_separator(
                        Some(tx),
                        DomainSeparator::SafeRegistry,
                        domain_separator_updated.domainSeparator.0.into(),
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
        _is_synced: bool,
    ) -> Result<Option<ChainEventType>> {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_INDEXER_LOG_COUNTERS.increment(&["node_management_module"]);

        // Don't care at the moment
        Ok(None)
    }

    async fn on_ticket_winning_probability_oracle_event(
        &self,
        tx: &OpenTransaction,
        event: HoprWinningProbabilityOracleEvents,
        _is_synced: bool,
    ) -> Result<Option<ChainEventType>> {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_INDEXER_LOG_COUNTERS.increment(&["win_prob_oracle"]);

        match event {
            HoprWinningProbabilityOracleEvents::WinProbUpdated(update) => {
                let old_minimum_win_prob: WinningProbability = update.oldWinProb.to_be_bytes().into();
                let new_minimum_win_prob: WinningProbability = update.newWinProb.to_be_bytes().into();

                trace!(
                    %old_minimum_win_prob,
                    %new_minimum_win_prob,
                    "on_ticket_minimum_win_prob_updated",
                );

                self.db
                    .set_minimum_incoming_ticket_win_prob(Some(tx), new_minimum_win_prob)
                    .await?;

                info!(
                    %old_minimum_win_prob,
                    %new_minimum_win_prob,
                    "minimum ticket winning probability updated"
                );

                // If the old minimum was less strict, we need to mark of all the
                // tickets below the new higher minimum as rejected
                if old_minimum_win_prob.approx_cmp(&new_minimum_win_prob).is_lt() {
                    let mut selector: Option<TicketSelector> = None;
                    for channel in self.db.get_incoming_channels(tx.into()).await? {
                        selector = selector
                            .map(|s| s.also_on_channel(channel.get_id(), channel.channel_epoch))
                            .or_else(|| Some(TicketSelector::from(channel)));
                    }
                    // Reject unredeemed tickets on all the channels with win prob lower than the new one
                    if let Some(selector) = selector {
                        let num_rejected = self
                            .db
                            .mark_tickets_as(
                                selector.with_winning_probability(..new_minimum_win_prob),
                                TicketMarker::Rejected,
                            )
                            .await?;
                        info!(
                            count = num_rejected,
                            "unredeemed tickets were rejected, because the minimum winning probability has been \
                             increased"
                        );
                    }
                }
            }
            _ => {
                // Ignore other events
            }
        }
        Ok(None)
    }

    async fn on_ticket_price_oracle_event(
        &self,
        tx: &OpenTransaction,
        event: HoprTicketPriceOracleEvents,
        _is_synced: bool,
    ) -> Result<Option<ChainEventType>> {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_INDEXER_LOG_COUNTERS.increment(&["price_oracle"]);

        match event {
            HoprTicketPriceOracleEvents::TicketPriceUpdated(update) => {
                trace!(
                    old = update._0.to_string(),
                    new = update._1.to_string(),
                    "on_ticket_price_updated",
                );

                self.db
                    .update_ticket_price(Some(tx), HoprBalance::from_be_bytes(update._1.to_be_bytes::<32>()))
                    .await?;

                info!(price = %update._1, "ticket price updated");
            }
            HoprTicketPriceOracleEvents::OwnershipTransferred(_event) => {
                // ignore ownership transfer event
            }
        }
        Ok(None)
    }

    #[tracing::instrument(level = "debug", skip(self, slog), fields(log=%slog))]
    async fn process_log_event(
        &self,
        tx: &OpenTransaction,
        slog: SerializableLog,
        is_synced: bool,
    ) -> Result<Option<ChainEventType>> {
        trace!(log = %slog, "log content");

        let log = Log::from(slog.clone());

        let primitive_log = alloy::primitives::Log::new(
            slog.address.into(),
            slog.topics.iter().map(|h| B256::from_slice(h.as_ref())).collect(),
            slog.data.clone().into(),
        )
        .ok_or_else(|| {
            CoreEthereumIndexerError::ProcessError(format!("failed to convert log to primitive log: {slog:?}"))
        })?;

        if log.address.eq(&self.addresses.announcements) {
            let bn = log.block_number as u32;
            let event = HoprAnnouncementsEvents::decode_log(&primitive_log)?;
            self.on_announcement_event(tx, event.data, bn, is_synced).await
        } else if log.address.eq(&self.addresses.channels) {
            let event = HoprChannelsEvents::decode_log(&primitive_log)?;
            match self.on_channel_event(tx, event.data, is_synced).await {
                Ok(res) => Ok(res),
                Err(CoreEthereumIndexerError::ChannelDoesNotExist) => {
                    // This is not an error, just a log that we don't have the channel in the DB
                    debug!(
                        ?log,
                        "channel didn't exist in the db. Created a corrupted channel entry and ignored event"
                    );
                    Ok(None)
                }
                Err(e) => Err(e),
            }
        } else if log.address.eq(&self.addresses.network_registry) {
            let event = HoprNetworkRegistryEvents::decode_log(&primitive_log)?;
            self.on_network_registry_event(tx, event.data, is_synced).await
        } else if log.address.eq(&self.addresses.token) {
            let event = HoprTokenEvents::decode_log(&primitive_log)?;
            self.on_token_event(tx, event.data, is_synced).await
        } else if log.address.eq(&self.addresses.safe_registry) {
            let event = HoprNodeSafeRegistryEvents::decode_log(&primitive_log)?;
            self.on_node_safe_registry_event(tx, event.data, is_synced).await
        } else if log.address.eq(&self.addresses.module_implementation) {
            let event = HoprNodeManagementModuleEvents::decode_log(&primitive_log)?;
            self.on_node_management_module_event(tx, event.data, is_synced).await
        } else if log.address.eq(&self.addresses.price_oracle) {
            let event = HoprTicketPriceOracleEvents::decode_log(&primitive_log)?;
            self.on_ticket_price_oracle_event(tx, event.data, is_synced).await
        } else if log.address.eq(&self.addresses.win_prob_oracle) {
            let event = HoprWinningProbabilityOracleEvents::decode_log(&primitive_log)?;
            self.on_ticket_winning_probability_oracle_event(tx, event.data, is_synced)
                .await
        } else {
            #[cfg(all(feature = "prometheus", not(test)))]
            METRIC_INDEXER_LOG_COUNTERS.increment(&["unknown"]);

            error!(
                address = %log.address, log = ?log,
                "on_event error - unknown contract address, received log"
            );
            return Err(CoreEthereumIndexerError::UnknownContract(log.address));
        }
    }
}

#[async_trait]
impl<T, Db> crate::traits::ChainLogHandler for ContractEventHandlers<T, Db>
where
    T: HoprIndexerRpcOperations + Clone + Send + Sync + 'static,
    Db: HoprDbAllOperations + Clone + Debug + Send + Sync + 'static,
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

    fn contract_addresses_map(&self) -> Arc<ContractAddresses> {
        self.addresses.clone()
    }

    fn safe_address(&self) -> Address {
        self.safe_address
    }

    fn contract_address_topics(&self, contract: Address) -> Vec<B256> {
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
        } else {
            panic!("use of unsupported contract address: {contract}");
        }
    }

    async fn collect_log_event(&self, slog: SerializableLog, is_synced: bool) -> Result<Option<SignificantChainEvent>> {
        let myself = self.clone();
        self.db
            .begin_transaction()
            .await?
            .perform(move |tx| {
                let log = slog.clone();
                let tx_hash = Hash::from(log.tx_hash);
                let log_id = log.log_index;
                let block_id = log.block_number;

                Box::pin(async move {
                    match myself.process_log_event(tx, log, is_synced).await {
                        // If a significant chain event can be extracted from the log
                        Ok(Some(event_type)) => {
                            let significant_event = SignificantChainEvent { tx_hash, event_type };
                            debug!(block_id, %tx_hash, log_id, ?significant_event, "indexer got significant_event");
                            Ok(Some(significant_event))
                        }
                        Ok(None) => {
                            debug!(block_id, %tx_hash, log_id, "no significant event in log");
                            Ok(None)
                        }
                        Err(error) => {
                            error!(block_id, %tx_hash, log_id, %error, "error processing log in tx");
                            Err(error)
                        }
                    }
                })
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, atomic::Ordering},
        time::SystemTime,
    };

    use alloy::{
        dyn_abi::DynSolValue,
        primitives::{Address as AlloyAddress, U256},
        sol_types::{SolEvent, SolValue},
    };
    use anyhow::{Context, anyhow};
    use hex_literal::hex;
    use hopr_chain_rpc::HoprIndexerRpcOperations;
    use hopr_chain_types::{
        ContractAddresses,
        chain_events::{ChainEventType, NetworkRegistryStatus},
    };
    use hopr_crypto_types::prelude::*;
    use hopr_db_sql::{
        HoprDbAllOperations, HoprDbGeneralModelOperations,
        accounts::{ChainOrPacketKey, HoprDbAccountOperations},
        api::{info::DomainSeparator, tickets::HoprDbTicketOperations},
        channels::HoprDbChannelOperations,
        db::HoprDb,
        info::HoprDbInfoOperations,
        prelude::HoprDbResolverOperations,
        registry::HoprDbRegistryOperations,
    };
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use multiaddr::Multiaddr;
    use primitive_types::H256;

    use super::ContractEventHandlers;

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

    mockall::mock! {
        pub(crate) IndexerRpcOperations {}

        #[async_trait::async_trait]
        impl HoprIndexerRpcOperations for IndexerRpcOperations {
            async fn block_number(&self) -> hopr_chain_rpc::errors::Result<u64>;

        async fn get_hopr_allowance(&self, owner: Address, spender: Address) -> hopr_chain_rpc::errors::Result<HoprBalance>;

        async fn get_xdai_balance(&self, address: Address) -> hopr_chain_rpc::errors::Result<XDaiBalance>;

        async fn get_hopr_balance(&self, address: Address) -> hopr_chain_rpc::errors::Result<HoprBalance>;

        fn try_stream_logs<'a>(
            &'a self,
            start_block_number: u64,
            filters: hopr_chain_rpc::FilterSet,
            is_synced: bool,
        ) -> hopr_chain_rpc::errors::Result<std::pin::Pin<Box<dyn futures::Stream<Item=hopr_chain_rpc::BlockWithLogs> + Send + 'a> > >;
        }
    }

    #[derive(Clone)]
    pub struct ClonableMockOperations {
        pub inner: Arc<MockIndexerRpcOperations>,
    }

    #[async_trait::async_trait]
    impl HoprIndexerRpcOperations for ClonableMockOperations {
        async fn block_number(&self) -> hopr_chain_rpc::errors::Result<u64> {
            self.inner.block_number().await
        }

        async fn get_hopr_allowance(
            &self,
            owner: Address,
            spender: Address,
        ) -> hopr_chain_rpc::errors::Result<HoprBalance> {
            self.inner.get_hopr_allowance(owner, spender).await
        }

        async fn get_xdai_balance(&self, address: Address) -> hopr_chain_rpc::errors::Result<XDaiBalance> {
            self.inner.get_xdai_balance(address).await
        }

        async fn get_hopr_balance(&self, address: Address) -> hopr_chain_rpc::errors::Result<HoprBalance> {
            self.inner.get_hopr_balance(address).await
        }

        fn try_stream_logs<'a>(
            &'a self,
            start_block_number: u64,
            filters: hopr_chain_rpc::FilterSet,
            is_synced: bool,
        ) -> hopr_chain_rpc::errors::Result<
            std::pin::Pin<Box<dyn futures::Stream<Item = hopr_chain_rpc::BlockWithLogs> + Send + 'a>>,
        > {
            self.inner.try_stream_logs(start_block_number, filters, is_synced)
        }
    }

    fn init_handlers<T: Clone, Db: HoprDbAllOperations + Clone>(
        rpc_operations: T,
        db: Db,
    ) -> ContractEventHandlers<T, Db> {
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
            safe_address: *STAKE_ADDRESS,
            db,
            rpc_operations,
        }
    }

    fn test_log() -> SerializableLog {
        SerializableLog { ..Default::default() }
    }

    #[tokio::test]
    async fn announce_keybinding() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            inner: Arc::new(rpc_operations),
        };

        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let keybinding = KeyBinding::new(*SELF_CHAIN_ADDRESS, &SELF_PRIV_KEY);

        let keybinding_log = SerializableLog {
            address: handlers.addresses.announcements,
            topics: vec![
                hopr_bindings::hoprannouncementsevents::HoprAnnouncementsEvents::KeyBinding::SIGNATURE_HASH.into(),
            ],
            data: DynSolValue::Tuple(vec![
                DynSolValue::Bytes(keybinding.signature.as_ref().to_vec()),
                DynSolValue::Bytes(keybinding.packet_key.as_ref().to_vec()),
                DynSolValue::FixedBytes(AlloyAddress::from_slice(SELF_CHAIN_ADDRESS.as_ref()).into_word(), 32),
            ])
            .abi_encode_packed(),
            ..test_log()
        };

        let account_entry = AccountEntry {
            public_key: *SELF_PRIV_KEY.public(),
            chain_addr: *SELF_CHAIN_ADDRESS,
            entry_type: AccountType::NotAnnounced,
            published_at: 0,
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, keybinding_log, true).await }))
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

    #[tokio::test]
    async fn announce_address_announcement() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        // Assume that there is a keybinding
        let account_entry = AccountEntry {
            public_key: *SELF_PRIV_KEY.public(),
            chain_addr: *SELF_CHAIN_ADDRESS,
            entry_type: AccountType::NotAnnounced,
            published_at: 1,
        };
        db.insert_account(None, account_entry.clone()).await?;

        let test_multiaddr_empty: Multiaddr = "".parse()?;

        let address_announcement_empty_log_encoded_data = DynSolValue::Tuple(vec![
            DynSolValue::Address(AlloyAddress::from_slice(SELF_CHAIN_ADDRESS.as_ref())),
            DynSolValue::String(test_multiaddr_empty.to_string()),
        ])
        .abi_encode();

        let address_announcement_empty_log = SerializableLog {
            address: handlers.addresses.announcements,
            topics: vec![
                hopr_bindings::hoprannouncementsevents::HoprAnnouncementsEvents::AddressAnnouncement::SIGNATURE_HASH
                    .into(),
            ],
            data: address_announcement_empty_log_encoded_data[32..].into(),
            ..test_log()
        };

        let handlers_clone = handlers.clone();
        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    handlers_clone
                        .process_log_event(tx, address_announcement_empty_log, true)
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

        let address_announcement_log_encoded_data = DynSolValue::Tuple(vec![
            DynSolValue::Address(AlloyAddress::from_slice(SELF_CHAIN_ADDRESS.as_ref())),
            DynSolValue::String(test_multiaddr.to_string()),
        ])
        .abi_encode();

        let address_announcement_log = SerializableLog {
            address: handlers.addresses.announcements,
            block_number: 1,
            topics: vec![
                hopr_bindings::hoprannouncementsevents::HoprAnnouncementsEvents::AddressAnnouncement::SIGNATURE_HASH
                    .into(),
            ],
            data: address_announcement_log_encoded_data[32..].into(),
            ..test_log()
        };

        let announced_account_entry = AccountEntry {
            public_key: *SELF_PRIV_KEY.public(),
            chain_addr: *SELF_CHAIN_ADDRESS,
            entry_type: AccountType::Announced {
                multiaddr: test_multiaddr.clone(),
                updated_block: 1,
            },
            published_at: 1,
        };

        let handlers_clone = handlers.clone();
        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    handlers_clone
                        .process_log_event(tx, address_announcement_log, true)
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

        let address_announcement_dns_log_encoded_data = DynSolValue::Tuple(vec![
            DynSolValue::Address(AlloyAddress::from_slice(SELF_CHAIN_ADDRESS.as_ref())),
            DynSolValue::String(test_multiaddr_dns.to_string()),
        ])
        .abi_encode();

        let address_announcement_dns_log = SerializableLog {
            address: handlers.addresses.announcements,
            block_number: 2,
            topics: vec![
                hopr_bindings::hoprannouncementsevents::HoprAnnouncementsEvents::AddressAnnouncement::SIGNATURE_HASH
                    .into(),
            ],
            data: address_announcement_dns_log_encoded_data[32..].into(),
            ..test_log()
        };

        let announced_dns_account_entry = AccountEntry {
            public_key: *SELF_PRIV_KEY.public(),
            chain_addr: *SELF_CHAIN_ADDRESS,
            entry_type: AccountType::Announced {
                multiaddr: test_multiaddr_dns.clone(),
                updated_block: 2,
            },
            published_at: 1,
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move { handlers.process_log_event(tx, address_announcement_dns_log, true).await })
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

    #[tokio::test]
    async fn announce_revoke() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let test_multiaddr: Multiaddr = "/ip4/1.2.3.4/tcp/56".parse()?;

        // Assume that there is a keybinding and an address announcement
        let announced_account_entry = AccountEntry {
            public_key: *SELF_PRIV_KEY.public(),
            chain_addr: *SELF_CHAIN_ADDRESS,
            entry_type: AccountType::Announced {
                multiaddr: test_multiaddr,
                updated_block: 0,
            },
            published_at: 1,
        };

        db.insert_account(None, announced_account_entry).await?;

        let encoded_data = (AlloyAddress::from_slice(SELF_CHAIN_ADDRESS.as_ref()),).abi_encode();

        let revoke_announcement_log = SerializableLog {
            address: handlers.addresses.announcements,
            topics: vec![
                hopr_bindings::hoprannouncementsevents::HoprAnnouncementsEvents::RevokeAnnouncement::SIGNATURE_HASH
                    .into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        let account_entry = AccountEntry {
            public_key: *SELF_PRIV_KEY.public(),
            chain_addr: *SELF_CHAIN_ADDRESS,
            entry_type: AccountType::NotAnnounced,
            published_at: 1,
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, revoke_announcement_log, true).await }))
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

    #[tokio::test]
    async fn on_token_transfer_to() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let value = U256::MAX;
        let target_hopr_balance = HoprBalance::from(primitive_types::U256::from_big_endian(
            value.to_be_bytes_vec().as_slice(),
        ));

        let mut rpc_operations = MockIndexerRpcOperations::new();
        rpc_operations
            .expect_get_hopr_balance()
            .times(1)
            .return_once(move |_| Ok(target_hopr_balance));
        rpc_operations
            .expect_get_hopr_allowance()
            .times(1)
            .returning(move |_, _| Ok(HoprBalance::from(primitive_types::U256::from(1000u64))));
        let clonable_rpc_operations = ClonableMockOperations {
            inner: Arc::new(rpc_operations),
        };

        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let encoded_data = (value).abi_encode();

        let transferred_log = SerializableLog {
            address: handlers.addresses.token,
            topics: vec![
                hopr_bindings::hoprtoken::HoprToken::Transfer::SIGNATURE_HASH.into(),
                H256::from_slice(&Address::default().to_bytes32()).into(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, transferred_log, true).await }))
            .await?;

        assert!(event_type.is_none(), "token transfer does not have chain event type");

        assert_eq!(db.get_safe_hopr_balance(None).await?, target_hopr_balance);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn on_token_transfer_from() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let mut rpc_operations = MockIndexerRpcOperations::new();
        rpc_operations
            .expect_get_hopr_balance()
            .times(1)
            .return_once(|_| Ok(HoprBalance::zero()));
        rpc_operations
            .expect_get_hopr_allowance()
            .times(1)
            .returning(move |_, _| Ok(HoprBalance::from(primitive_types::U256::from(1000u64))));
        let clonable_rpc_operations = ClonableMockOperations {
            inner: Arc::new(rpc_operations),
        };

        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let value = U256::MAX;

        let encoded_data = (value).abi_encode();

        db.set_safe_hopr_balance(
            None,
            HoprBalance::from(primitive_types::U256::from_big_endian(
                value.to_be_bytes_vec().as_slice(),
            )),
        )
        .await?;

        let transferred_log = SerializableLog {
            address: handlers.addresses.token,
            topics: vec![
                hopr_bindings::hoprtoken::HoprToken::Transfer::SIGNATURE_HASH.into(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()).into(),
                H256::from_slice(&Address::default().to_bytes32()).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, transferred_log, true).await }))
            .await?;

        assert!(event_type.is_none(), "token transfer does not have chain event type");

        assert_eq!(db.get_safe_hopr_balance(None).await?, HoprBalance::zero());

        Ok(())
    }

    #[tokio::test]
    async fn on_token_approval_correct() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let target_allowance = HoprBalance::from(primitive_types::U256::from(1000u64));
        let mut rpc_operations = MockIndexerRpcOperations::new();
        rpc_operations
            .expect_get_hopr_allowance()
            .times(2)
            .returning(move |_, _| Ok(target_allowance));
        let clonable_rpc_operations = ClonableMockOperations {
            inner: Arc::new(rpc_operations),
        };

        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let encoded_data = (U256::from(1000u64)).abi_encode();

        let approval_log = SerializableLog {
            address: handlers.addresses.token,
            topics: vec![
                hopr_bindings::hoprtoken::HoprToken::Approval::SIGNATURE_HASH.into(),
                H256::from_slice(&handlers.safe_address.to_bytes32()).into(),
                H256::from_slice(&handlers.addresses.channels.to_bytes32()).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        // before any operation the allowance should be 0
        assert_eq!(db.get_safe_hopr_allowance(None).await?, HoprBalance::zero());

        let approval_log_clone = approval_log.clone();
        let handlers_clone = handlers.clone();
        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers_clone.process_log_event(tx, approval_log_clone, true).await }))
            .await?;

        assert!(event_type.is_none(), "token approval does not have chain event type");

        // after processing the allowance should be 0
        assert_eq!(db.get_safe_hopr_allowance(None).await?, target_allowance.clone());

        // reduce allowance manually to verify a second time
        let _ = db
            .set_safe_hopr_allowance(None, HoprBalance::from(primitive_types::U256::from(10u64)))
            .await;
        assert_eq!(
            db.get_safe_hopr_allowance(None).await?,
            HoprBalance::from(primitive_types::U256::from(10u64))
        );

        let handlers_clone = handlers.clone();
        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers_clone.process_log_event(tx, approval_log, true).await }))
            .await?;

        assert!(event_type.is_none(), "token approval does not have chain event type");

        assert_eq!(db.get_safe_hopr_allowance(None).await?, target_allowance);
        Ok(())
    }

    #[tokio::test]
    async fn on_network_registry_event_registered() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let encoded_data = ().abi_encode();

        let registered_log = SerializableLog {
            address: handlers.addresses.network_registry,
            topics: vec![
                hopr_bindings::hoprnetworkregistry::HoprNetworkRegistry::Registered::SIGNATURE_HASH.into(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()).into(),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()).into(),
            ],
            data: encoded_data,
            // data: encode(&[]).into(),
            ..test_log()
        };

        assert!(
            !db.is_allowed_in_network_registry(None, &SELF_CHAIN_ADDRESS.as_ref())
                .await?
        );

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, registered_log, false).await }))
            .await?;

        assert!(
            matches!(event_type, Some(ChainEventType::NetworkRegistryUpdate(a, s)) if a == *SELF_CHAIN_ADDRESS && s == NetworkRegistryStatus::Allowed),
            "must return correct NR update"
        );

        assert!(
            db.is_allowed_in_network_registry(None, &SELF_CHAIN_ADDRESS.as_ref())
                .await?,
            "must be allowed in NR"
        );
        Ok(())
    }

    #[tokio::test]
    async fn on_network_registry_event_registered_by_manager() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let registered_log = SerializableLog {
            address: handlers.addresses.network_registry,
            topics: vec![
                hopr_bindings::hoprnetworkregistry::HoprNetworkRegistry::RegisteredByManager::SIGNATURE_HASH.into(),
                // RegisteredByManagerFilter::signature().into(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()).into(),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()).into(),
            ],
            data: ().abi_encode(),
            // data: encode(&[]).into(),
            ..test_log()
        };

        assert!(
            !db.is_allowed_in_network_registry(None, &SELF_CHAIN_ADDRESS.as_ref())
                .await?
        );

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, registered_log, false).await }))
            .await?;

        assert!(
            matches!(event_type, Some(ChainEventType::NetworkRegistryUpdate(a, s)) if a == *SELF_CHAIN_ADDRESS && s == NetworkRegistryStatus::Allowed),
            "must return correct NR update"
        );

        assert!(
            db.is_allowed_in_network_registry(None, &SELF_CHAIN_ADDRESS.as_ref())
                .await?,
            "must be allowed in NR"
        );
        Ok(())
    }

    #[tokio::test]
    async fn on_network_registry_event_deregistered() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        db.set_access_in_network_registry(None, *SELF_CHAIN_ADDRESS, true)
            .await?;

        let encoded_data = ().abi_encode();

        let registered_log = SerializableLog {
            address: handlers.addresses.network_registry,
            topics: vec![
                hopr_bindings::hoprnetworkregistry::HoprNetworkRegistry::Deregistered::SIGNATURE_HASH.into(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()).into(),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        assert!(
            db.is_allowed_in_network_registry(None, &SELF_CHAIN_ADDRESS.as_ref())
                .await?
        );

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, registered_log, false).await }))
            .await?;

        assert!(
            matches!(event_type, Some(ChainEventType::NetworkRegistryUpdate(a, s)) if a == *SELF_CHAIN_ADDRESS && s == NetworkRegistryStatus::Denied),
            "must return correct NR update"
        );

        assert!(
            !db.is_allowed_in_network_registry(None, &SELF_CHAIN_ADDRESS.as_ref())
                .await?,
            "must not be allowed in NR"
        );
        Ok(())
    }

    #[tokio::test]
    async fn on_network_registry_event_deregistered_by_manager() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        db.set_access_in_network_registry(None, *SELF_CHAIN_ADDRESS, true)
            .await?;

        let encoded_data = ().abi_encode();

        let registered_log = SerializableLog {
            address: handlers.addresses.network_registry,
            topics: vec![
                hopr_bindings::hoprnetworkregistry::HoprNetworkRegistry::DeregisteredByManager::SIGNATURE_HASH.into(),
                // DeregisteredByManagerFilter::signature().into(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()).into(),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        assert!(
            db.is_allowed_in_network_registry(None, &SELF_CHAIN_ADDRESS.as_ref())
                .await?
        );

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, registered_log, false).await }))
            .await?;

        assert!(
            matches!(event_type, Some(ChainEventType::NetworkRegistryUpdate(a, s)) if a == *SELF_CHAIN_ADDRESS && s == NetworkRegistryStatus::Denied),
            "must return correct NR update"
        );

        assert!(
            !db.is_allowed_in_network_registry(None, &SELF_CHAIN_ADDRESS.as_ref())
                .await?,
            "must not be allowed in NR"
        );
        Ok(())
    }

    #[tokio::test]
    async fn on_network_registry_event_enabled() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let encoded_data = ().abi_encode();

        let nr_enabled = SerializableLog {
            address: handlers.addresses.network_registry,
            topics: vec![
                hopr_bindings::hoprnetworkregistry::HoprNetworkRegistry::NetworkRegistryStatusUpdated::SIGNATURE_HASH
                    .into(),
                // NetworkRegistryStatusUpdatedFilter::signature().into(),
                H256::from_low_u64_be(1).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, nr_enabled, false).await }))
            .await?;

        assert!(event_type.is_none(), "there's no chain event type for nr disable");

        assert!(db.get_indexer_data(None).await?.nr_enabled);
        Ok(())
    }

    #[tokio::test]
    async fn on_network_registry_event_disabled() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        db.set_network_registry_enabled(None, true).await?;

        let encoded_data = ().abi_encode();

        let nr_disabled = SerializableLog {
            address: handlers.addresses.network_registry,
            topics: vec![
                hopr_bindings::hoprnetworkregistry::HoprNetworkRegistry::NetworkRegistryStatusUpdated::SIGNATURE_HASH
                    .into(),
                // NetworkRegistryStatusUpdatedFilter::signature().into(),
                H256::from_low_u64_be(0).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, nr_disabled, false).await }))
            .await?;

        assert!(event_type.is_none(), "there's no chain event type for nr enable");

        assert!(!db.get_indexer_data(None).await?.nr_enabled);
        Ok(())
    }

    #[tokio::test]
    async fn on_network_registry_set_eligible() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let encoded_data = ().abi_encode();

        let set_eligible = SerializableLog {
            address: handlers.addresses.network_registry,
            topics: vec![
                hopr_bindings::hoprnetworkregistry::HoprNetworkRegistry::EligibilityUpdated::SIGNATURE_HASH.into(),
                // EligibilityUpdatedFilter::signature().into(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()).into(),
                H256::from_low_u64_be(1).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, set_eligible, false).await }))
            .await?;

        assert!(
            event_type.is_none(),
            "there's no chain event type for setting nr eligibility"
        );

        assert!(db.is_safe_eligible(None, *STAKE_ADDRESS).await?);

        Ok(())
    }

    #[tokio::test]
    async fn on_network_registry_set_not_eligible() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        db.set_safe_eligibility(None, *STAKE_ADDRESS, false).await?;

        let encoded_data = ().abi_encode();

        let set_eligible = SerializableLog {
            address: handlers.addresses.network_registry,
            topics: vec![
                hopr_bindings::hoprnetworkregistry::HoprNetworkRegistry::EligibilityUpdated::SIGNATURE_HASH.into(),
                // EligibilityUpdatedFilter::signature().into(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()).into(),
                H256::from_low_u64_be(0).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, set_eligible, true).await }))
            .await?;

        assert!(
            event_type.is_none(),
            "there's no chain event type for unsetting nr eligibility"
        );

        assert!(!db.is_safe_eligible(None, *STAKE_ADDRESS).await?);

        Ok(())
    }

    #[tokio::test]
    async fn on_channel_event_balance_increased() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let value = U256::MAX;
        let target_hopr_balance = HoprBalance::from(primitive_types::U256::from_big_endian(
            value.to_be_bytes_vec().as_slice(),
        ));

        let mut rpc_operations = MockIndexerRpcOperations::new();
        rpc_operations
            .expect_get_hopr_balance()
            .times(1)
            .return_once(move |_| Ok(target_hopr_balance));
        rpc_operations
            .expect_get_hopr_allowance()
            .times(1)
            .returning(move |_, _| Ok(HoprBalance::from(primitive_types::U256::from(1000u64))));
        let clonable_rpc_operations = ClonableMockOperations {
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let channel = ChannelEntry::new(
            *SELF_CHAIN_ADDRESS,
            *COUNTERPARTY_CHAIN_ADDRESS,
            0.into(),
            primitive_types::U256::zero(),
            ChannelStatus::Open,
            primitive_types::U256::one(),
        );

        db.upsert_channel(None, channel).await?;

        let solidity_balance: HoprBalance = primitive_types::U256::from((1u128 << 96) - 1).into();
        let diff = solidity_balance - channel.balance;

        let encoded_data = (solidity_balance.amount().to_be_bytes()).abi_encode();

        let balance_increased_log = SerializableLog {
            address: handlers.addresses.channels,
            topics: vec![
                hopr_bindings::hoprchannels::HoprChannels::ChannelBalanceIncreased::SIGNATURE_HASH.into(),
                // ChannelBalanceIncreasedFilter::signature().into(),
                H256::from_slice(channel.get_id().as_ref()).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, balance_increased_log, true).await }))
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

    #[tokio::test]
    async fn on_channel_event_domain_separator_updated() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let separator = Hash::from(hopr_crypto_random::random_bytes());

        let encoded_data = ().abi_encode();

        let channels_dst_updated = SerializableLog {
            address: handlers.addresses.channels,
            topics: vec![
                hopr_bindings::hoprchannels::HoprChannels::DomainSeparatorUpdated::SIGNATURE_HASH.into(),
                // DomainSeparatorUpdatedFilter::signature().into(),
                H256::from_slice(separator.as_ref()).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        assert!(db.get_indexer_data(None).await?.channels_dst.is_none());

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, channels_dst_updated, true).await }))
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

    #[tokio::test]
    async fn on_channel_event_balance_decreased() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;

        let value = U256::MAX;
        let target_hopr_balance = HoprBalance::from(primitive_types::U256::from_big_endian(
            value.to_be_bytes_vec().as_slice(),
        ));

        let mut rpc_operations = MockIndexerRpcOperations::new();
        rpc_operations
            .expect_get_hopr_balance()
            .times(1)
            .return_once(move |_| Ok(target_hopr_balance));
        let clonable_rpc_operations = ClonableMockOperations {
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let channel = ChannelEntry::new(
            *SELF_CHAIN_ADDRESS,
            *COUNTERPARTY_CHAIN_ADDRESS,
            HoprBalance::from(primitive_types::U256::from((1u128 << 96) - 1)),
            primitive_types::U256::zero(),
            ChannelStatus::Open,
            primitive_types::U256::one(),
        );

        db.upsert_channel(None, channel).await?;

        let solidity_balance: HoprBalance = primitive_types::U256::from((1u128 << 96) - 2).into();
        let diff = channel.balance - solidity_balance;

        // let encoded_data = (solidity_balance).abi_encode();
        let encoded_data = DynSolValue::Tuple(vec![DynSolValue::Uint(
            U256::from_be_slice(&solidity_balance.amount().to_be_bytes()),
            256,
        )])
        .abi_encode();

        let balance_decreased_log = SerializableLog {
            address: handlers.addresses.channels,
            topics: vec![
                hopr_bindings::hoprchannels::HoprChannels::ChannelBalanceDecreased::SIGNATURE_HASH.into(),
                // ChannelBalanceDecreasedFilter::signature().into(),
                H256::from_slice(channel.get_id().as_ref()).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, balance_decreased_log, true).await }))
            .await?;

        let channel = db
            .get_channel_by_id(None, &channel.get_id())
            .await?
            .context("a value should be present")?;

        assert!(
            matches!(event_type, Some(ChainEventType::ChannelBalanceDecreased(c, b)) if c == channel && b == diff),
            "must return updated channel entry and balance diff"
        );

        assert_eq!(solidity_balance, channel.balance, "balance must be updated");
        Ok(())
    }

    #[tokio::test]
    async fn on_channel_closed() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let starting_balance = HoprBalance::from(primitive_types::U256::from((1u128 << 96) - 1));

        let channel = ChannelEntry::new(
            *SELF_CHAIN_ADDRESS,
            *COUNTERPARTY_CHAIN_ADDRESS,
            starting_balance,
            primitive_types::U256::zero(),
            ChannelStatus::Open,
            primitive_types::U256::one(),
        );

        db.upsert_channel(None, channel).await?;

        let encoded_data = ().abi_encode();

        let channel_closed_log = SerializableLog {
            address: handlers.addresses.channels,
            topics: vec![
                hopr_bindings::hoprchannels::HoprChannels::ChannelClosed::SIGNATURE_HASH.into(),
                // ChannelClosedFilter::signature().into(),
                H256::from_slice(channel.get_id().as_ref()).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, channel_closed_log, true).await }))
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

        assert!(closed_channel.balance.amount().eq(&primitive_types::U256::zero()));
        Ok(())
    }

    #[tokio::test]
    async fn on_foreign_channel_closed() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let starting_balance = HoprBalance::from(primitive_types::U256::from((1u128 << 96) - 1));

        let channel = ChannelEntry::new(
            Address::new(&hex!("B7397C218766eBe6A1A634df523A1a7e412e67eA")),
            Address::new(&hex!("D4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1")),
            starting_balance,
            primitive_types::U256::zero(),
            ChannelStatus::Open,
            primitive_types::U256::one(),
        );

        db.upsert_channel(None, channel).await?;

        let encoded_data = ().abi_encode();

        let channel_closed_log = SerializableLog {
            address: handlers.addresses.channels,
            topics: vec![
                hopr_bindings::hoprchannels::HoprChannels::ChannelClosed::SIGNATURE_HASH.into(),
                // ChannelClosedFilter::signature().into(),
                H256::from_slice(channel.get_id().as_ref()).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, channel_closed_log, true).await }))
            .await?;

        let closed_channel = db.get_channel_by_id(None, &channel.get_id()).await?;

        assert_eq!(None, closed_channel, "foreign channel must be deleted");

        assert!(
            matches!(event_type, Some(ChainEventType::ChannelClosed(c)) if c.get_id() == channel.get_id()),
            "must return the closed channel entry"
        );

        Ok(())
    }

    #[tokio::test]
    async fn on_channel_opened() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let channel_id = generate_channel_id(&SELF_CHAIN_ADDRESS, &COUNTERPARTY_CHAIN_ADDRESS);

        let encoded_data = ().abi_encode();

        let channel_opened_log = SerializableLog {
            address: handlers.addresses.channels,
            topics: vec![
                hopr_bindings::hoprchannels::HoprChannels::ChannelOpened::SIGNATURE_HASH.into(),
                // ChannelOpenedFilter::signature().into(),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()).into(),
                H256::from_slice(&COUNTERPARTY_CHAIN_ADDRESS.to_bytes32()).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, channel_opened_log, true).await }))
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

    #[tokio::test]
    async fn on_channel_reopened() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let channel = ChannelEntry::new(
            *SELF_CHAIN_ADDRESS,
            *COUNTERPARTY_CHAIN_ADDRESS,
            HoprBalance::zero(),
            primitive_types::U256::zero(),
            ChannelStatus::Closed,
            3.into(),
        );

        db.upsert_channel(None, channel).await?;

        let encoded_data = ().abi_encode();

        let channel_opened_log = SerializableLog {
            address: handlers.addresses.channels,
            topics: vec![
                hopr_bindings::hoprchannels::HoprChannels::ChannelOpened::SIGNATURE_HASH.into(),
                // ChannelOpenedFilter::signature().into(),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()).into(),
                H256::from_slice(&COUNTERPARTY_CHAIN_ADDRESS.to_bytes32()).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, channel_opened_log, true).await }))
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

    #[tokio::test]
    async fn on_channel_should_not_reopen_when_not_closed() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let channel = ChannelEntry::new(
            *SELF_CHAIN_ADDRESS,
            *COUNTERPARTY_CHAIN_ADDRESS,
            0.into(),
            primitive_types::U256::zero(),
            ChannelStatus::Open,
            3.into(),
        );

        db.upsert_channel(None, channel).await?;

        let encoded_data = ().abi_encode();

        let channel_opened_log = SerializableLog {
            address: handlers.addresses.channels,
            topics: vec![
                hopr_bindings::hoprchannels::HoprChannels::ChannelOpened::SIGNATURE_HASH.into(),
                // ChannelOpenedFilter::signature().into(),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()).into(),
                H256::from_slice(&COUNTERPARTY_CHAIN_ADDRESS.to_bytes32()).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        db.begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, channel_opened_log, true).await }))
            .await
            .context("Channel should stay open, with corrupted flag set")?;

        assert!(
            db.get_channel_by_id(None, &channel.get_id()).await?.is_none(),
            "channel should not be returned as marked as corrupted",
        );

        db.get_corrupted_channel_by_id(None, &channel.get_id())
            .await?
            .context("a value should be present")?;

        Ok(())
    }

    #[tokio::test]
    async fn corrupted_channel_editing_should_not_panic() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let channel = ChannelEntry::new(
            *SELF_CHAIN_ADDRESS,
            *COUNTERPARTY_CHAIN_ADDRESS,
            0.into(),
            primitive_types::U256::zero(),
            ChannelStatus::Open,
            3.into(),
        );

        db.upsert_channel(None, channel).await?;

        // set the channel as corrupted
        let editor = match db.begin_channel_update(None, &channel.get_id()).await? {
            Some(editor) => editor,
            None => return Err(anyhow::anyhow!("Failed to begin channel update")),
        };

        db.finish_channel_update(None, editor.set_corrupted()).await?;

        db.get_corrupted_channel_by_id(None, &channel.get_id())
            .await?
            .context("channel should be set a corrupted at this point")?;

        let encoded_data = ().abi_encode();
        let handlers_clone = handlers.clone();
        let channel_opened_log = SerializableLog {
            address: handlers_clone.addresses.channels,
            topics: vec![
                hopr_bindings::hoprchannels::HoprChannels::ChannelOpened::SIGNATURE_HASH.into(),
                // ChannelOpenedFilter::signature().into(),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()).into(),
                H256::from_slice(&COUNTERPARTY_CHAIN_ADDRESS.to_bytes32()).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        db.begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers_clone.process_log_event(tx, channel_opened_log, true).await }))
            .await
            .context("Channel should stay open without panicking")?;

        // Attempt to increase balance
        let solidity_balance: HoprBalance = primitive_types::U256::from((1u128 << 96) - 1).into();

        let encoded_data = (solidity_balance.amount().to_be_bytes()).abi_encode();

        let balance_increased_log = SerializableLog {
            address: handlers.addresses.channels,
            topics: vec![
                hopr_bindings::hoprchannels::HoprChannels::ChannelBalanceIncreased::SIGNATURE_HASH.into(),
                // ChannelBalanceIncreasedFilter::signature().into(),
                H256::from_slice(channel.get_id().as_ref()).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        db.begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, balance_increased_log, true).await }))
            .await?;

        // Check that the channel is still marked as corrupted
        db.get_corrupted_channel_by_id(None, &channel.get_id())
            .await?
            .context("channel should still be set a corrupted")?;

        Ok(())
    }

    #[tokio::test]
    async fn event_for_non_existing_channel_should_create_dummy_channel() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let channel_id = generate_channel_id(&SELF_CHAIN_ADDRESS, &COUNTERPARTY_CHAIN_ADDRESS);

        // Attempt to increase balance
        let solidity_balance: HoprBalance = primitive_types::U256::from((1u128 << 96) - 1).into();

        let encoded_data = (solidity_balance.amount().to_be_bytes()).abi_encode();

        let balance_increased_log = SerializableLog {
            address: handlers.addresses.channels,
            topics: vec![
                hopr_bindings::hoprchannels::HoprChannels::ChannelBalanceIncreased::SIGNATURE_HASH.into(),
                // ChannelBalanceIncreasedFilter::signature().into(),
                H256::from_slice(channel_id.as_ref()).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        db.begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, balance_increased_log, true).await }))
            .await?;

        // Check that the dummy corrupted channel was created
        db.get_corrupted_channel_by_id(None, &channel_id)
            .await?
            .context("channel should be set a corrupted")?;

        Ok(())
    }

    const PRICE_PER_PACKET: u32 = 20_u32;

    fn mock_acknowledged_ticket(
        signer: &ChainKeypair,
        destination: &ChainKeypair,
        index: u64,
        win_prob: f64,
    ) -> anyhow::Result<AcknowledgedTicket> {
        let channel_id = generate_channel_id(&signer.into(), &destination.into());

        let channel_epoch = 1u64;
        let domain_separator = Hash::default();

        let response = Response::try_from(
            Hash::create(&[channel_id.as_ref(), &channel_epoch.to_be_bytes(), &index.to_be_bytes()]).as_ref(),
        )?;

        Ok(TicketBuilder::default()
            .direction(&signer.into(), &destination.into())
            .amount(primitive_types::U256::from(PRICE_PER_PACKET).div_f64(win_prob)?)
            .index(index)
            .index_offset(1)
            .win_prob(win_prob.try_into()?)
            .channel_epoch(1)
            .challenge(response.to_challenge().into())
            .build_signed(signer, &domain_separator)?
            .into_acknowledged(response))
    }

    #[tokio::test]
    async fn on_channel_ticket_redeemed_incoming_channel() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Hash::default())
            .await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let channel = ChannelEntry::new(
            *COUNTERPARTY_CHAIN_ADDRESS,
            *SELF_CHAIN_ADDRESS,
            HoprBalance::from(primitive_types::U256::from((1u128 << 96) - 1)),
            primitive_types::U256::zero(),
            ChannelStatus::Open,
            primitive_types::U256::one(),
        );

        let ticket_index = primitive_types::U256::from((1u128 << 48) - 2);
        let next_ticket_index = ticket_index + 1;

        let mut ticket =
            mock_acknowledged_ticket(&COUNTERPARTY_CHAIN_KEY, &SELF_CHAIN_KEY, ticket_index.as_u64(), 1.0)?;
        ticket.status = AcknowledgedTicketStatus::BeingRedeemed;

        let ticket_value = ticket.verified_ticket().amount;

        db.upsert_channel(None, channel).await?;
        db.upsert_ticket(None, ticket.clone()).await?;

        let ticket_redeemed_log = SerializableLog {
            address: handlers.addresses.channels,
            topics: vec![
                hopr_bindings::hoprchannels::HoprChannels::TicketRedeemed::SIGNATURE_HASH.into(),
                // TicketRedeemedFilter::signature().into(),
                H256::from_slice(channel.get_id().as_ref()).into(),
            ],
            data: DynSolValue::Tuple(vec![DynSolValue::Uint(
                U256::from_be_bytes(next_ticket_index.to_be_bytes()),
                48,
            )])
            .abi_encode(),
            ..test_log()
        };

        let outgoing_ticket_index_before = db
            .get_outgoing_ticket_index(channel.get_id())
            .await?
            .load(Ordering::Relaxed);

        let stats = db.get_ticket_statistics(Some(channel.get_id())).await?;
        assert_eq!(
            HoprBalance::zero(),
            stats.redeemed_value,
            "there should not be any redeemed value"
        );
        assert_eq!(
            HoprBalance::zero(),
            stats.neglected_value,
            "there should not be any neglected value"
        );

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, ticket_redeemed_log, true).await }))
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
            HoprBalance::zero(),
            stats.neglected_value,
            "there should not be any neglected ticket"
        );
        Ok(())
    }

    #[tokio::test]
    async fn on_channel_ticket_redeemed_incoming_channel_neglect_left_over_tickets() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Hash::default())
            .await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let channel = ChannelEntry::new(
            *COUNTERPARTY_CHAIN_ADDRESS,
            *SELF_CHAIN_ADDRESS,
            primitive_types::U256::from((1u128 << 96) - 1).into(),
            primitive_types::U256::zero(),
            ChannelStatus::Open,
            primitive_types::U256::one(),
        );

        let ticket_index = primitive_types::U256::from((1u128 << 48) - 2);
        let next_ticket_index = ticket_index + 1;

        let mut ticket =
            mock_acknowledged_ticket(&COUNTERPARTY_CHAIN_KEY, &SELF_CHAIN_KEY, ticket_index.as_u64(), 1.0)?;
        ticket.status = AcknowledgedTicketStatus::BeingRedeemed;

        let ticket_value = ticket.verified_ticket().amount;

        db.upsert_channel(None, channel).await?;
        db.upsert_ticket(None, ticket.clone()).await?;

        let old_ticket =
            mock_acknowledged_ticket(&COUNTERPARTY_CHAIN_KEY, &SELF_CHAIN_KEY, ticket_index.as_u64() - 1, 1.0)?;
        db.upsert_ticket(None, old_ticket.clone()).await?;

        let ticket_redeemed_log = SerializableLog {
            address: handlers.addresses.channels,
            topics: vec![
                hopr_bindings::hoprchannels::HoprChannels::TicketRedeemed::SIGNATURE_HASH.into(),
                // TicketRedeemedFilter::signature().into(),
                H256::from_slice(channel.get_id().as_ref()).into(),
            ],
            data: Vec::from(next_ticket_index.to_be_bytes()),
            ..test_log()
        };

        let outgoing_ticket_index_before = db
            .get_outgoing_ticket_index(channel.get_id())
            .await?
            .load(Ordering::Relaxed);

        let stats = db.get_ticket_statistics(Some(channel.get_id())).await?;
        assert_eq!(
            HoprBalance::zero(),
            stats.redeemed_value,
            "there should not be any redeemed value"
        );
        assert_eq!(
            HoprBalance::zero(),
            stats.neglected_value,
            "there should not be any neglected value"
        );

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, ticket_redeemed_log, true).await }))
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

    #[tokio::test]
    async fn on_channel_ticket_redeemed_outgoing_channel() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Hash::default())
            .await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let channel = ChannelEntry::new(
            *SELF_CHAIN_ADDRESS,
            *COUNTERPARTY_CHAIN_ADDRESS,
            primitive_types::U256::from((1u128 << 96) - 1).into(),
            primitive_types::U256::zero(),
            ChannelStatus::Open,
            primitive_types::U256::one(),
        );

        let ticket_index = primitive_types::U256::from((1u128 << 48) - 2);
        let next_ticket_index = ticket_index + 1;

        db.upsert_channel(None, channel).await?;

        let ticket_redeemed_log = SerializableLog {
            address: handlers.addresses.channels,
            topics: vec![
                hopr_bindings::hoprchannels::HoprChannels::TicketRedeemed::SIGNATURE_HASH.into(),
                // TicketRedeemedFilter::signature().into(),
                H256::from_slice(channel.get_id().as_ref()).into(),
            ],
            data: Vec::from(next_ticket_index.to_be_bytes()),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, ticket_redeemed_log, true).await }))
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

    #[tokio::test]
    async fn on_channel_ticket_redeemed_on_incoming_channel_with_non_existent_ticket_should_pass() -> anyhow::Result<()>
    {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Hash::default())
            .await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let channel = ChannelEntry::new(
            *COUNTERPARTY_CHAIN_ADDRESS,
            *SELF_CHAIN_ADDRESS,
            primitive_types::U256::from((1u128 << 96) - 1).into(),
            primitive_types::U256::zero(),
            ChannelStatus::Open,
            primitive_types::U256::one(),
        );

        db.upsert_channel(None, channel).await?;

        let next_ticket_index = primitive_types::U256::from((1u128 << 48) - 1);

        let ticket_redeemed_log = SerializableLog {
            address: handlers.addresses.channels,
            topics: vec![
                hopr_bindings::hoprchannels::HoprChannels::TicketRedeemed::SIGNATURE_HASH.into(),
                // TicketRedeemedFilter::signature().into(),
                H256::from_slice(channel.get_id().as_ref()).into(),
            ],
            data: Vec::from(next_ticket_index.to_be_bytes()),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, ticket_redeemed_log, true).await }))
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

    #[tokio::test]
    async fn on_channel_ticket_redeemed_on_foreign_channel_should_pass() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let channel = ChannelEntry::new(
            Address::from(hopr_crypto_random::random_bytes()),
            Address::from(hopr_crypto_random::random_bytes()),
            primitive_types::U256::from((1u128 << 96) - 1).into(),
            primitive_types::U256::zero(),
            ChannelStatus::Open,
            primitive_types::U256::one(),
        );

        db.upsert_channel(None, channel).await?;

        let next_ticket_index = primitive_types::U256::from((1u128 << 48) - 1);

        let ticket_redeemed_log = SerializableLog {
            address: handlers.addresses.channels,
            topics: vec![
                hopr_bindings::hoprchannels::HoprChannels::TicketRedeemed::SIGNATURE_HASH.into(),
                // TicketRedeemedFilter::signature().into(),
                H256::from_slice(channel.get_id().as_ref()).into(),
            ],
            data: Vec::from(next_ticket_index.to_be_bytes()),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, ticket_redeemed_log, true).await }))
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

    #[tokio::test]
    async fn on_channel_closure_initiated() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let channel = ChannelEntry::new(
            *SELF_CHAIN_ADDRESS,
            *COUNTERPARTY_CHAIN_ADDRESS,
            primitive_types::U256::from((1u128 << 96) - 1).into(),
            primitive_types::U256::zero(),
            ChannelStatus::Open,
            primitive_types::U256::one(),
        );

        db.upsert_channel(None, channel).await?;

        let timestamp = SystemTime::now();

        let encoded_data = (U256::from(timestamp.as_unix_timestamp().as_secs())).abi_encode();

        let closure_initiated_log = SerializableLog {
            address: handlers.addresses.channels,
            topics: vec![
                hopr_bindings::hoprchannels::HoprChannels::OutgoingChannelClosureInitiated::SIGNATURE_HASH.into(),
                // OutgoingChannelClosureInitiatedFilter::signature().into(),
                H256::from_slice(channel.get_id().as_ref()).into(),
            ],
            data: encoded_data,
            // data: Vec::from(U256::from(timestamp.as_unix_timestamp().as_secs()).to_be_bytes()).into(),
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, closure_initiated_log, true).await }))
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

    #[tokio::test]
    async fn on_node_safe_registry_registered() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let encoded_data = ().abi_encode();

        let safe_registered_log = SerializableLog {
            address: handlers.addresses.safe_registry,
            topics: vec![
                hopr_bindings::hoprnodesaferegistry::HoprNodeSafeRegistry::RegisteredNodeSafe::SIGNATURE_HASH.into(),
                // RegisteredNodeSafeFilter::signature().into(),
                H256::from_slice(&SAFE_INSTANCE_ADDR.to_bytes32()).into(),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, safe_registered_log, true).await }))
            .await?;

        assert!(matches!(event_type, Some(ChainEventType::NodeSafeRegistered(addr)) if addr == *SAFE_INSTANCE_ADDR));

        // Nothing to check in the DB here, since we do not track this
        Ok(())
    }

    #[tokio::test]
    async fn on_node_safe_registry_deregistered() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        // Nothing to write to the DB here, since we do not track this

        let encoded_data = ().abi_encode();

        let safe_registered_log = SerializableLog {
            address: handlers.addresses.safe_registry,
            topics: vec![
                hopr_bindings::hoprnodesaferegistry::HoprNodeSafeRegistry::DergisteredNodeSafe::SIGNATURE_HASH.into(),
                // DergisteredNodeSafeFilter::signature().into(),
                H256::from_slice(&SAFE_INSTANCE_ADDR.to_bytes32()).into(),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()).into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, safe_registered_log, true).await }))
            .await?;

        assert!(
            event_type.is_none(),
            "there's no associated chain event type with safe deregistration"
        );

        // Nothing to check in the DB here, since we do not track this
        Ok(())
    }

    #[tokio::test]
    async fn ticket_price_update() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let encoded_data = (U256::from(1u64), U256::from(123u64)).abi_encode();

        let price_change_log = SerializableLog {
            address: handlers.addresses.price_oracle,
            topics: vec![
                hopr_bindings::hoprticketpriceoracle::HoprTicketPriceOracle::TicketPriceUpdated::SIGNATURE_HASH.into(),
                // TicketPriceUpdatedFilter::signature().into()
            ],
            data: encoded_data,
            // data: encode(&[Token::Uint(EthU256::from(1u64)), Token::Uint(EthU256::from(123u64))]).into(),
            ..test_log()
        };

        assert_eq!(db.get_indexer_data(None).await?.ticket_price, None);

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, price_change_log, true).await }))
            .await?;

        assert!(
            event_type.is_none(),
            "there's no associated chain event type with price oracle"
        );

        assert_eq!(
            db.get_indexer_data(None).await?.ticket_price.map(|p| p.amount()),
            Some(primitive_types::U256::from(123u64))
        );
        Ok(())
    }

    #[tokio::test]
    async fn minimum_win_prob_update() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let encoded_data = (
            U256::from_be_slice(WinningProbability::ALWAYS.as_ref()),
            U256::from_be_slice(WinningProbability::try_from_f64(0.5)?.as_ref()),
        )
            .abi_encode();

        let win_prob_change_log = SerializableLog {
            address: handlers.addresses.win_prob_oracle,
            topics: vec![
                hopr_bindings::hoprwinningprobabilityoracle::HoprWinningProbabilityOracle::WinProbUpdated::SIGNATURE_HASH.into()],
            data: encoded_data,
            ..test_log()
        };

        assert_eq!(
            db.get_indexer_data(None).await?.minimum_incoming_ticket_winning_prob,
            1.0
        );

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, win_prob_change_log, true).await }))
            .await?;

        assert!(
            event_type.is_none(),
            "there's no associated chain event type with winning probability change"
        );

        assert_eq!(
            db.get_indexer_data(None).await?.minimum_incoming_ticket_winning_prob,
            0.5
        );
        Ok(())
    }

    #[tokio::test]
    async fn lowering_minimum_win_prob_update_should_reject_non_satisfying_unredeemed_tickets() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(SELF_CHAIN_KEY.clone()).await?;
        db.set_minimum_incoming_ticket_win_prob(None, 0.1.try_into()?).await?;

        let new_minimum = 0.5;
        let ticket_win_probs = [0.1, 1.0, 0.3, 0.2];

        let channel_1 = ChannelEntry::new(
            *COUNTERPARTY_CHAIN_ADDRESS,
            *SELF_CHAIN_ADDRESS,
            primitive_types::U256::from((1u128 << 96) - 1).into(),
            3_u32.into(),
            ChannelStatus::Open,
            primitive_types::U256::one(),
        );

        db.upsert_channel(None, channel_1).await?;

        let ticket = mock_acknowledged_ticket(&COUNTERPARTY_CHAIN_KEY, &SELF_CHAIN_KEY, 1, ticket_win_probs[0])?;
        db.upsert_ticket(None, ticket).await?;

        let ticket = mock_acknowledged_ticket(&COUNTERPARTY_CHAIN_KEY, &SELF_CHAIN_KEY, 2, ticket_win_probs[1])?;
        db.upsert_ticket(None, ticket).await?;

        let tickets = db.get_tickets((&channel_1).into()).await?;
        assert_eq!(tickets.len(), 2);

        // ---

        let other_counterparty = ChainKeypair::random();
        let channel_2 = ChannelEntry::new(
            other_counterparty.public().to_address(),
            *SELF_CHAIN_ADDRESS,
            primitive_types::U256::from((1u128 << 96) - 1).into(),
            3_u32.into(),
            ChannelStatus::Open,
            primitive_types::U256::one(),
        );

        db.upsert_channel(None, channel_2).await?;

        let ticket = mock_acknowledged_ticket(&other_counterparty, &SELF_CHAIN_KEY, 1, ticket_win_probs[2])?;
        db.upsert_ticket(None, ticket).await?;

        let ticket = mock_acknowledged_ticket(&other_counterparty, &SELF_CHAIN_KEY, 2, ticket_win_probs[3])?;
        db.upsert_ticket(None, ticket).await?;

        let tickets = db.get_tickets((&channel_2).into()).await?;
        assert_eq!(tickets.len(), 2);

        let stats = db.get_ticket_statistics(None).await?;
        assert_eq!(HoprBalance::zero(), stats.rejected_value);

        let rpc_operations = MockIndexerRpcOperations::new();
        // ==> set mock expectations here
        let clonable_rpc_operations = ClonableMockOperations {
            //
            inner: Arc::new(rpc_operations),
        };
        let handlers = init_handlers(clonable_rpc_operations, db.clone());

        let encoded_data = (
            U256::from_be_slice(WinningProbability::try_from(0.1)?.as_ref()),
            U256::from_be_slice(WinningProbability::try_from(new_minimum)?.as_ref()),
        )
            .abi_encode();

        let win_prob_change_log = SerializableLog {
            address: handlers.addresses.win_prob_oracle,
            topics: vec![
                hopr_bindings::hoprwinningprobabilityoracle::HoprWinningProbabilityOracle::WinProbUpdated::SIGNATURE_HASH.into(),
            ],
            data: encoded_data,
            ..test_log()
        };

        let event_type = db
            .begin_transaction()
            .await?
            .perform(|tx| Box::pin(async move { handlers.process_log_event(tx, win_prob_change_log, true).await }))
            .await?;

        assert!(
            event_type.is_none(),
            "there's no associated chain event type with winning probability change"
        );

        assert_eq!(
            db.get_indexer_data(None).await?.minimum_incoming_ticket_winning_prob,
            new_minimum
        );

        let tickets = db.get_tickets((&channel_1).into()).await?;
        assert_eq!(tickets.len(), 1);

        let tickets = db.get_tickets((&channel_2).into()).await?;
        assert_eq!(tickets.len(), 0);

        let stats = db.get_ticket_statistics(None).await?;
        let rejected_value: primitive_types::U256 = ticket_win_probs
            .iter()
            .filter(|p| **p < new_minimum)
            .map(|p| {
                primitive_types::U256::from(PRICE_PER_PACKET)
                    .div_f64(*p)
                    .expect("must divide")
            })
            .reduce(|a, b| a + b)
            .ok_or(anyhow!("must sum"))?;

        assert_eq!(HoprBalance::from(rejected_value), stats.rejected_value);

        Ok(())
    }
}
