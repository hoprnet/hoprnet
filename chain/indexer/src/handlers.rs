use crate::errors::{CoreEthereumIndexerError, Result};
use async_lock::RwLock;
use async_trait::async_trait;
use bindings::{
    hopr_announcements::HoprAnnouncementsEvents, hopr_channels::HoprChannelsEvents,
    hopr_network_registry::HoprNetworkRegistryEvents, hopr_node_management_module::HoprNodeManagementModuleEvents,
    hopr_node_safe_registry::HoprNodeSafeRegistryEvents, hopr_ticket_price_oracle::HoprTicketPriceOracleEvents,
    hopr_token::HoprTokenEvents,
};
use chain_db::traits::HoprCoreEthereumDbActions;
use chain_types::chain_events::{ChainEventType, NetworkRegistryStatus};
use chain_types::ContractAddresses;
use ethers::{contract::EthLogDecode, core::abi::RawLog};
use hopr_crypto_types::types::OffchainSignature;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use log::{error, info, trace, warn};
use multiaddr::Multiaddr;
use std::ops::{Add, Sub};
use std::time::{Duration, SystemTime};
use std::{str::FromStr, sync::Arc};

/// Event handling object for on-chain operations
///
/// Once an on-chain operation is recorded by the [chain_indexer::block::Indexer], it is pre-processed
/// and passed on to this object that handles event specific actions for each on-chain operation.
///
#[derive(Debug, Clone)]
pub struct ContractEventHandlers<U: HoprCoreEthereumDbActions> {
    /// channels, announcements, network_registry, token: contract addresses
    /// whose event we process
    addresses: ContractAddresses,
    /// Safe on-chain address which we are monitoring
    safe_address: Address,
    /// own address, aka message sender
    chain_key: Address,
    /// callbacks to inform other modules
    db: Arc<RwLock<U>>,
}

impl<U: HoprCoreEthereumDbActions> ContractEventHandlers<U> {
    pub fn new(addresses: ContractAddresses, safe_address: Address, chain_key: Address, db: Arc<RwLock<U>>) -> Self {
        Self {
            addresses,
            safe_address,
            chain_key,
            db,
        }
    }

    async fn on_announcement_event(
        &self,
        db: &mut U,
        event: HoprAnnouncementsEvents,
        block_number: u32,
        snapshot: &Snapshot,
    ) -> Result<Option<ChainEventType>> {
        match event {
            HoprAnnouncementsEvents::AddressAnnouncementFilter(address_announcement) => {
                let maybe_account = db.get_account(&address_announcement.node.into()).await?;

                trace!(
                    "on_announcement_event - multiaddr: {:?} - node: {:?}",
                    &address_announcement.base_multiaddr,
                    &address_announcement.node.to_string()
                );
                // safeguard against empty multiaddrs, skip
                if address_announcement.base_multiaddr.is_empty() {
                    return Err(CoreEthereumIndexerError::AnnounceEmptyMultiaddr);
                }

                if let Some(mut account) = maybe_account {
                    let new_entry_type = AccountType::Announced {
                        multiaddr: Multiaddr::from_str(&address_announcement.base_multiaddr)?,
                        updated_block: block_number,
                    };

                    account.update(new_entry_type);
                    db.update_account_and_snapshot(&account, snapshot).await?;

                    if let Some(ma) = account.get_multiaddr() {
                        return Ok(Some(ChainEventType::Announcement {
                            peer: account.public_key.into(),
                            address: account.chain_addr,
                            multiaddresses: vec![ma],
                        }));
                    }
                } else {
                    return Err(CoreEthereumIndexerError::AnnounceBeforeKeyBinding);
                }
            }
            HoprAnnouncementsEvents::KeyBindingFilter(key_binding) => {
                if db.get_account(&key_binding.chain_key.into()).await?.is_some() {
                    return Err(CoreEthereumIndexerError::UnsupportedKeyRebinding);
                }

                match KeyBinding::from_parts(
                    key_binding.chain_key.into(),
                    key_binding.ed_25519_pub_key.try_into()?,
                    OffchainSignature::try_from((key_binding.ed_25519_sig_0, key_binding.ed_25519_sig_1))?,
                ) {
                    Ok(binding) => {
                        let updated_account = AccountEntry::new(
                            key_binding.ed_25519_pub_key.try_into()?,
                            key_binding.chain_key.into(),
                            AccountType::NotAnnounced,
                        );

                        db.link_chain_and_packet_keys(&binding.chain_key, &binding.packet_key, snapshot)
                            .await?;

                        db.update_account_and_snapshot(&updated_account, snapshot).await?;
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
                let maybe_account = db.get_account(&revocation.node.into()).await?;

                if let Some(mut account) = maybe_account {
                    account.update(AccountType::NotAnnounced);
                    db.update_account_and_snapshot(&account, snapshot).await?;
                } else {
                    return Err(CoreEthereumIndexerError::RevocationBeforeKeyBinding);
                }
            }
        };

        Ok(None)
    }

    async fn on_channel_event(
        &self,
        db: &mut U,
        event: HoprChannelsEvents,
        snapshot: &Snapshot,
    ) -> Result<Option<ChainEventType>> {
        match event {
            HoprChannelsEvents::ChannelBalanceDecreasedFilter(balance_decreased) => {
                let maybe_channel = db.get_channel(&balance_decreased.channel_id.into()).await?;

                if let Some(mut channel) = maybe_channel {
                    let new_balance = Balance::new(balance_decreased.new_balance, BalanceType::HOPR);
                    let diff = channel.balance.sub(&new_balance);
                    channel.balance = new_balance;

                    db.update_channel_and_snapshot(&balance_decreased.channel_id.into(), &channel, snapshot)
                        .await?;

                    Ok(Some(ChainEventType::ChannelBalanceDecreased(channel, diff)))
                } else {
                    Err(CoreEthereumIndexerError::ChannelDoesNotExist)
                }
            }
            HoprChannelsEvents::ChannelBalanceIncreasedFilter(balance_increased) => {
                let maybe_channel = db.get_channel(&balance_increased.channel_id.into()).await?;

                if let Some(mut channel) = maybe_channel {
                    let new_balance = Balance::new(balance_increased.new_balance, BalanceType::HOPR);
                    let diff = new_balance.sub(&channel.balance);
                    channel.balance = new_balance;

                    db.update_channel_and_snapshot(&balance_increased.channel_id.into(), &channel, snapshot)
                        .await?;

                    Ok(Some(ChainEventType::ChannelBalanceIncreased(channel, diff)))
                } else {
                    Err(CoreEthereumIndexerError::ChannelDoesNotExist)
                }
            }
            HoprChannelsEvents::ChannelClosedFilter(channel_closed) => {
                let maybe_channel = db.get_channel(&channel_closed.channel_id.into()).await?;

                trace!(
                    "on_channel_closed_event - channel_id: {:?} - channel known: {:?}",
                    channel_closed.channel_id,
                    maybe_channel.is_some()
                );

                if let Some(mut channel) = maybe_channel {
                    // set all channel fields like we do on-chain on close
                    channel.status = ChannelStatus::Closed;
                    channel.balance = Balance::new(U256::zero(), BalanceType::HOPR);
                    channel.ticket_index = 0u64.into();

                    // Incoming channel, so once closed. All unredeemed tickets just became invalid
                    if channel.destination.eq(&self.chain_key) {
                        db.mark_acknowledged_tickets_neglected(channel).await?;
                    }

                    db.update_channel_and_snapshot(&channel_closed.channel_id.into(), &channel, snapshot)
                        .await?;

                    if channel.source.eq(&self.chain_key) || channel.destination.eq(&self.chain_key) {
                        // Reset the current_ticket_index to zero
                        db.set_current_ticket_index(&channel_closed.channel_id.into(), U256::zero())
                            .await?;
                    }
                    Ok(Some(ChainEventType::ChannelClosed(channel)))
                } else {
                    Err(CoreEthereumIndexerError::ChannelDoesNotExist)
                }
            }
            HoprChannelsEvents::ChannelOpenedFilter(channel_opened) => {
                let source: Address = channel_opened.source.into();
                let destination: Address = channel_opened.destination.into();

                let channel_id = generate_channel_id(&source, &destination);

                let maybe_channel = db.get_channel(&channel_id).await?;
                let is_reopen = maybe_channel.is_some();
                trace!(
                    "on_channel_opened_event - source: {source} - destination: {destination} - channel_id: {channel_id}, channel known: {is_reopen}"
                );

                let channel = maybe_channel
                    .map(|mut channel| {
                        // set all channel fields like we do on-chain on close
                        channel.status = ChannelStatus::Open;
                        channel.ticket_index = 0u64.into();
                        channel.channel_epoch += U256::from(1u64);
                        channel
                    })
                    .unwrap_or(ChannelEntry::new(
                        source,
                        destination,
                        Balance::new(0_u64, BalanceType::HOPR),
                        0u64.into(),
                        ChannelStatus::Open,
                        1u64.into(),
                    ));

                db.update_channel_and_snapshot(&channel_id, &channel, snapshot).await?;

                if source.eq(&self.chain_key) || destination.eq(&self.chain_key) {
                    db.set_current_ticket_index(&channel_id, U256::zero()).await?;

                    // Cleanup tickets from previous epochs on channel re-opening
                    if is_reopen {
                        db.cleanup_invalid_channel_tickets(&channel).await?;
                    }
                }

                Ok(Some(ChainEventType::ChannelOpened(channel)))
            }
            HoprChannelsEvents::TicketRedeemedFilter(ticket_redeemed) => {
                let maybe_channel = db.get_channel(&ticket_redeemed.channel_id.into()).await?;

                if let Some(mut channel) = maybe_channel {
                    channel.ticket_index = ticket_redeemed.new_ticket_index.into();

                    db.update_channel_and_snapshot(&ticket_redeemed.channel_id.into(), &channel, snapshot)
                        .await?;

                    // compare the ticket index from the redeemed ticket with the current_ticket_index. Ensure that the current_ticket_index is not smaller than the value from redeemed ticket.
                    db.ensure_current_ticket_index_gte(&ticket_redeemed.channel_id.into(), channel.ticket_index)
                        .await?;

                    if channel.source.eq(&self.chain_key) || channel.destination.eq(&self.chain_key) {
                        // For channels that destination is us, it means that our ticket
                        // has been redeemed, so mark it in the DB as redeemed
                        let ack_ticket = if channel.destination.eq(&self.chain_key) {
                            // The ticket that has been redeemed at this point has: index + index_offset - 1 == new_ticket_index - 1
                            // Since unaggregated tickets have index_offset = 1, for the unagg case this leads to: index == new_ticket_index - 1
                            if let Some(ticket) = db
                                .get_acknowledged_tickets(Some(channel))
                                .await? // TODO: optimize this DB query and iteration
                                .into_iter()
                                .find(|ticket| {
                                    ticket.ticket.index + ticket.ticket.index_offset as u64
                                        == ticket_redeemed.new_ticket_index
                                })
                            {
                                db.mark_redeemed(&ticket).await?;
                                Some(ticket)
                            } else {
                                error!(
                                    "could not find acknowledged ticket with idx {} in {channel}",
                                    ticket_redeemed.new_ticket_index - 1
                                );
                                None
                            }
                        } else {
                            None
                        };

                        Ok(Some(ChainEventType::TicketRedeemed(channel, ack_ticket)))
                    } else {
                        Ok(Some(ChainEventType::TicketRedeemed(channel, None)))
                    }
                } else {
                    Err(CoreEthereumIndexerError::ChannelDoesNotExist)
                }
            }
            HoprChannelsEvents::OutgoingChannelClosureInitiatedFilter(closure_initiated) => {
                let maybe_channel = db.get_channel(&closure_initiated.channel_id.into()).await?;

                if let Some(mut channel) = maybe_channel {
                    channel.status = ChannelStatus::PendingToClose(
                        SystemTime::UNIX_EPOCH.add(Duration::from_secs(closure_initiated.closure_time as u64)),
                    );

                    db.update_channel_and_snapshot(&closure_initiated.channel_id.into(), &channel, snapshot)
                        .await?;

                    Ok(Some(ChainEventType::ChannelClosureInitiated(channel)))
                } else {
                    Err(CoreEthereumIndexerError::ChannelDoesNotExist)
                }
            }
            HoprChannelsEvents::DomainSeparatorUpdatedFilter(domain_separator_updated) => {
                db.set_channels_domain_separator(&domain_separator_updated.domain_separator.into(), snapshot)
                    .await?;

                Ok(None)
            }
            HoprChannelsEvents::LedgerDomainSeparatorUpdatedFilter(ledger_domain_separator_updated) => {
                db.set_channels_ledger_domain_separator(
                    &ledger_domain_separator_updated.ledger_domain_separator.into(),
                    snapshot,
                )
                .await?;

                Ok(None)
            }
        }
    }

    async fn on_token_event(
        &self,
        db: &mut U,
        event: HoprTokenEvents,
        snapshot: &Snapshot,
    ) -> Result<Option<ChainEventType>>
    where
        U: HoprCoreEthereumDbActions,
    {
        match event {
            HoprTokenEvents::TransferFilter(transfered) => {
                let from: Address = transfered.from.into();
                let to: Address = transfered.to.into();

                trace!(
                    "on_token_transfer_event - address_to_monitor: {:?} - to: {to} - from: {from}",
                    &self.safe_address,
                );

                if to.ne(&self.safe_address) && from.ne(&self.safe_address) {
                    return Ok(None);
                } else if to.eq(&self.safe_address) {
                    db.add_hopr_balance(&Balance::new(transfered.value, BalanceType::HOPR), snapshot)
                        .await?;
                } else if from.eq(&self.safe_address) {
                    db.sub_hopr_balance(&Balance::new(transfered.value, BalanceType::HOPR), snapshot)
                        .await?;
                }
            }
            HoprTokenEvents::ApprovalFilter(approved) => {
                let owner: Address = approved.owner.into();
                let spender: Address = approved.spender.into();

                trace!(
                    "on_token_approval_event - address_to_monitor: {:?} - owner: {owner} - spender: {spender}, allowance: {:?}",
                    &self.safe_address, approved.value
                );

                // if approval is for tokens on Safe contract to be spend by HoprChannels
                if owner.eq(&self.safe_address) && spender.eq(&self.addresses.channels) {
                    db.set_staking_safe_allowance(&Balance::new(approved.value, BalanceType::HOPR), snapshot)
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
        db: &mut U,
        event: HoprNetworkRegistryEvents,
        snapshot: &Snapshot,
    ) -> Result<Option<ChainEventType>>
    where
        U: HoprCoreEthereumDbActions,
    {
        match event {
            HoprNetworkRegistryEvents::DeregisteredByManagerFilter(deregistered) => {
                let node_address = &deregistered.node_address.into();
                db.set_allowed_to_access_network(node_address, false, snapshot).await?;
                return Ok(Some(ChainEventType::NetworkRegistryUpdate(
                    *node_address,
                    NetworkRegistryStatus::Denied,
                )));
            }
            HoprNetworkRegistryEvents::DeregisteredFilter(deregistered) => {
                let node_address = &deregistered.node_address.into();
                db.set_allowed_to_access_network(node_address, false, snapshot).await?;
                return Ok(Some(ChainEventType::NetworkRegistryUpdate(
                    *node_address,
                    NetworkRegistryStatus::Denied,
                )));
            }
            HoprNetworkRegistryEvents::RegisteredByManagerFilter(registered) => {
                let node_address = &registered.node_address.into();
                db.set_allowed_to_access_network(node_address, true, snapshot).await?;
                return Ok(Some(ChainEventType::NetworkRegistryUpdate(
                    *node_address,
                    NetworkRegistryStatus::Allowed,
                )));
            }
            HoprNetworkRegistryEvents::RegisteredFilter(registered) => {
                let node_address = &registered.node_address.into();
                db.set_allowed_to_access_network(node_address, true, snapshot).await?;
                return Ok(Some(ChainEventType::NetworkRegistryUpdate(
                    *node_address,
                    NetworkRegistryStatus::Allowed,
                )));
            }
            HoprNetworkRegistryEvents::EligibilityUpdatedFilter(eligibility_updated) => {
                let account: Address = eligibility_updated.staking_account.into();
                db.set_eligible(&account, eligibility_updated.eligibility, snapshot)
                    .await?;
            }
            HoprNetworkRegistryEvents::NetworkRegistryStatusUpdatedFilter(enabled) => {
                db.set_network_registry(enabled.is_enabled, snapshot).await?;
            }
            HoprNetworkRegistryEvents::RequirementUpdatedFilter(_) => {
                // TODO: implement this
            }
            _ => {
                // don't care. Not important to HOPR
            }
        };

        Ok(None)
    }

    async fn on_node_safe_registry_event(
        &self,
        db: &mut U,
        event: HoprNodeSafeRegistryEvents,
        snapshot: &Snapshot,
    ) -> Result<Option<ChainEventType>>
    where
        U: HoprCoreEthereumDbActions,
    {
        match event {
            HoprNodeSafeRegistryEvents::RegisteredNodeSafeFilter(registered) => {
                if self.chain_key.eq(&registered.node_address.into()) {
                    db.set_mfa_protected_and_update_snapshot(Some(registered.safe_address.into()), snapshot)
                        .await?;

                    return Ok(Some(ChainEventType::NodeSafeRegistered(registered.safe_address.into())));
                }
            }
            HoprNodeSafeRegistryEvents::DergisteredNodeSafeFilter(deregistered) => {
                if self.chain_key.eq(&deregistered.node_address.into()) {
                    if db.is_mfa_protected().await?.is_some() {
                        db.set_mfa_protected_and_update_snapshot(None, snapshot).await?;
                    } else {
                        return Err(CoreEthereumIndexerError::MFAModuleDoesNotExist);
                    }
                }
            }
            HoprNodeSafeRegistryEvents::DomainSeparatorUpdatedFilter(domain_separator_updated) => {
                db.set_node_safe_registry_domain_separator(
                    &(domain_separator_updated.domain_separator).into(),
                    snapshot,
                )
                .await?;
            }
        }

        Ok(None)
    }

    async fn on_node_management_module_event(
        &self,
        _db: &mut U,
        _event: HoprNodeManagementModuleEvents,
        _snapshot: &Snapshot,
    ) -> Result<Option<ChainEventType>>
    where
        U: HoprCoreEthereumDbActions,
    {
        // match event {
        //     _ => {
        //         // don't care at the moment
        //     }
        // }

        Ok(None)
    }

    async fn on_ticket_price_oracle_event(
        &self,
        db: &mut U,
        event: HoprTicketPriceOracleEvents,
        _snapshot: &Snapshot,
    ) -> Result<Option<ChainEventType>>
    where
        U: HoprCoreEthereumDbActions,
    {
        match event {
            HoprTicketPriceOracleEvents::TicketPriceUpdatedFilter(update) => {
                trace!(
                    "on_ticket_price_updated - old: {:?} - new: {:?}",
                    update.0.to_string(),
                    update.1.to_string()
                );

                info!("ticket price has been set to {}", update.1);
                db.set_ticket_price(&update.1).await?;
            }
            HoprTicketPriceOracleEvents::OwnershipTransferredFilter(_event) => {
                // ignore ownership transfer event
            }
        }
        Ok(None)
    }
}

#[async_trait]
impl<U: HoprCoreEthereumDbActions + Send + Sync> crate::traits::ChainLogHandler for ContractEventHandlers<U> {
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

    async fn on_event(
        &self,
        address: Address,
        block_number: u32,
        log: RawLog,
        snapshot: Snapshot,
    ) -> Result<Option<ChainEventType>> {
        trace!(
            "on_event - address: {:?} - received log: {:?}",
            address.to_string(),
            log
        );

        // NOTE: RW LOCK is unnecessary as it will block all other parts of the code
        // but it must be done in order to make sure that operations happen in a
        // single transaction.
        let mut db = self.db.write().await;

        if address.eq(&self.addresses.announcements) {
            let event = HoprAnnouncementsEvents::decode_log(&log)?;
            self.on_announcement_event(&mut db, event, block_number, &snapshot)
                .await
        } else if address.eq(&self.addresses.channels) {
            let event = HoprChannelsEvents::decode_log(&log)?;
            self.on_channel_event(&mut db, event, &snapshot).await
        } else if address.eq(&self.addresses.network_registry) {
            let event = HoprNetworkRegistryEvents::decode_log(&log)?;
            self.on_network_registry_event(&mut db, event, &snapshot).await
        } else if address.eq(&self.addresses.token) {
            let event = HoprTokenEvents::decode_log(&log)?;
            self.on_token_event(&mut db, event, &snapshot).await
        } else if address.eq(&self.addresses.safe_registry) {
            let event = HoprNodeSafeRegistryEvents::decode_log(&log)?;
            self.on_node_safe_registry_event(&mut db, event, &snapshot).await
        } else if address.eq(&self.addresses.module_implementation) {
            let event = HoprNodeManagementModuleEvents::decode_log(&log)?;
            self.on_node_management_module_event(&mut db, event, &snapshot).await
        } else if address.eq(&self.addresses.price_oracle) {
            let event = HoprTicketPriceOracleEvents::decode_log(&log)?;
            self.on_ticket_price_oracle_event(&mut db, event, &snapshot).await
        } else {
            error!(
                "on_event error - unknown contract address: {:?} - received log: {:?}",
                address.to_string(),
                log
            );

            Err(CoreEthereumIndexerError::UnknownContract(address))
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::traits::ChainLogHandler;

    use super::ContractEventHandlers;
    use async_lock::RwLock;
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
    use chain_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
    use chain_types::ContractAddresses;
    use ethers::contract::EthEvent;
    use ethers::{
        abi::{encode, Address as EthereumAddress, RawLog, Token},
        types::U256 as EthU256,
    };
    use hex_literal::hex;
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use multiaddr::Multiaddr;
    use primitive_types::H256;
    use utils_db::{db::DB, CurrentDbShim};

    lazy_static::lazy_static! {
        static ref SELF_PRIV_KEY: OffchainKeypair = OffchainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).unwrap();
        static ref COUNTERPARTY_CHAIN_ADDRESS: Address = Address::from_bytes(&hex!("f1a73ef496c45e260924a9279d2d9752ae378812")).unwrap();
        static ref SELF_CHAIN_ADDRESS: Address = Address::from_bytes(&hex!("2e505638d318598334c0a2c2e887e0ff1a23ec6a")).unwrap();
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

    async fn create_db() -> Arc<RwLock<CoreEthereumDb<CurrentDbShim>>> {
        Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
            Address::random(),
        )))
    }

    fn init_handlers<U: HoprCoreEthereumDbActions>(db: Arc<RwLock<U>>) -> ContractEventHandlers<U> {
        ContractEventHandlers {
            addresses: ContractAddresses {
                channels: *CHANNELS_ADDR,
                token: *TOKEN_ADDR,
                network_registry: *NETWORK_REGISTRY_ADDR,
                network_registry_proxy: Default::default(),
                safe_registry: *NODE_SAFE_REGISTRY_ADDR,
                announcements: *ANNOUNCEMENTS_ADDR,
                module_implementation: *SAFE_MANAGEMENT_MODULE_ADDR,
                price_oracle: *TICKET_PRICE_ORACLE_ADDR,
                stake_factory: Default::default(),
            },
            chain_key: *SELF_CHAIN_ADDRESS,
            safe_address: *SELF_CHAIN_ADDRESS,
            db,
        }
    }

    #[async_std::test]
    async fn announce_keybinding() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let keybinding = KeyBinding::new(*SELF_CHAIN_ADDRESS, &SELF_PRIV_KEY);

        let keybinding_log = RawLog {
            topics: vec![KeyBindingFilter::signature()],
            data: encode(&[
                Token::FixedBytes(Vec::from(keybinding.signature.to_bytes())),
                Token::FixedBytes(Vec::from(keybinding.packet_key.to_bytes())),
                Token::Address(EthereumAddress::from_slice(&SELF_CHAIN_ADDRESS.to_bytes())),
            ]),
        };

        let account_entry = AccountEntry::new(*SELF_PRIV_KEY.public(), *SELF_CHAIN_ADDRESS, AccountType::NotAnnounced);

        handlers
            .on_event(
                handlers.addresses.announcements,
                0u32,
                keybinding_log,
                Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            db.read().await.get_account(&SELF_CHAIN_ADDRESS).await.unwrap().unwrap(),
            account_entry
        );
    }

    #[async_std::test]
    async fn announce_address_announcement() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        // Assume that there is a keybinding
        let account_entry = AccountEntry::new(*SELF_PRIV_KEY.public(), *SELF_CHAIN_ADDRESS, AccountType::NotAnnounced);

        db.write()
            .await
            .update_account_and_snapshot(&account_entry, &Snapshot::default())
            .await
            .unwrap();

        let test_multiaddr_empty: Multiaddr = "".parse().unwrap();

        let address_announcement_empty_log = RawLog {
            topics: vec![AddressAnnouncementFilter::signature()],
            data: encode(&[
                Token::Address(EthereumAddress::from_slice(&SELF_CHAIN_ADDRESS.to_bytes())),
                Token::String(test_multiaddr_empty.to_string()),
            ]),
        };

        let _error = handlers
            .on_event(
                handlers.addresses.announcements,
                0u32,
                address_announcement_empty_log,
                Snapshot::default(),
            )
            .await
            .unwrap_err();

        assert_eq!(
            db.read().await.get_account(&SELF_CHAIN_ADDRESS).await.unwrap().unwrap(),
            account_entry
        );

        let test_multiaddr: Multiaddr = "/ip4/1.2.3.4/tcp/56".parse().unwrap();

        let address_announcement_log = RawLog {
            topics: vec![AddressAnnouncementFilter::signature()],
            data: encode(&[
                Token::Address(EthereumAddress::from_slice(&SELF_CHAIN_ADDRESS.to_bytes())),
                Token::String(test_multiaddr.to_string()),
            ]),
        };

        let announced_account_entry = AccountEntry::new(
            *SELF_PRIV_KEY.public(),
            *SELF_CHAIN_ADDRESS,
            AccountType::Announced {
                multiaddr: test_multiaddr,
                updated_block: 0,
            },
        );

        handlers
            .on_event(
                handlers.addresses.announcements,
                0u32,
                address_announcement_log,
                Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            db.read().await.get_account(&SELF_CHAIN_ADDRESS).await.unwrap().unwrap(),
            announced_account_entry
        );

        let test_multiaddr_dns: Multiaddr = "/dns4/useful.domain/tcp/56".parse().unwrap();

        let address_announcement_dns_log = RawLog {
            topics: vec![AddressAnnouncementFilter::signature()],
            data: encode(&[
                Token::Address(EthereumAddress::from_slice(&SELF_CHAIN_ADDRESS.to_bytes())),
                Token::String(test_multiaddr_dns.to_string()),
            ]),
        };

        let announced_dns_account_entry = AccountEntry::new(
            *SELF_PRIV_KEY.public(),
            *SELF_CHAIN_ADDRESS,
            AccountType::Announced {
                multiaddr: test_multiaddr_dns,
                updated_block: 0,
            },
        );

        handlers
            .on_event(
                handlers.addresses.announcements,
                0u32,
                address_announcement_dns_log,
                Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            db.read().await.get_account(&SELF_CHAIN_ADDRESS).await.unwrap().unwrap(),
            announced_dns_account_entry
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

        db.write()
            .await
            .update_account_and_snapshot(&announced_account_entry, &Snapshot::default())
            .await
            .unwrap();

        let revoke_announcement_log = RawLog {
            topics: vec![RevokeAnnouncementFilter::signature()],
            data: encode(&[Token::Address(EthereumAddress::from_slice(
                &SELF_CHAIN_ADDRESS.to_bytes(),
            ))]),
        };

        let account_entry = AccountEntry::new(*SELF_PRIV_KEY.public(), *SELF_CHAIN_ADDRESS, AccountType::NotAnnounced);

        handlers
            .on_event(
                handlers.addresses.announcements,
                0u32,
                revoke_announcement_log,
                Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            db.read().await.get_account(&SELF_CHAIN_ADDRESS).await.unwrap().unwrap(),
            account_entry
        );
    }

    #[async_std::test]
    async fn on_token_transfer_to() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let value = U256::max_value();

        let transferred_log = RawLog {
            topics: vec![
                TransferFilter::signature(),
                H256::from_slice(&Address::default().to_bytes32()),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[Token::Uint(EthU256::from_big_endian(&value.to_bytes()))]),
        };

        handlers
            .on_event(handlers.addresses.token, 0u32, transferred_log, Snapshot::default())
            .await
            .unwrap();

        let db = db.read().await;

        assert_eq!(
            db.get_hopr_balance().await.unwrap(),
            Balance::new(value, BalanceType::HOPR)
        )
    }

    #[async_std::test]
    async fn on_token_transfer_from() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let value = U256::max_value();

        db.write()
            .await
            .set_hopr_balance(&Balance::new(value, BalanceType::HOPR))
            .await
            .unwrap();

        let transferred_log = RawLog {
            topics: vec![
                TransferFilter::signature(),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
                H256::from_slice(&Address::default().to_bytes32()),
            ],
            data: encode(&[Token::Uint(EthU256::from_big_endian(&value.to_bytes()))]),
        };

        handlers
            .on_event(handlers.addresses.token, 0u32, transferred_log, Snapshot::default())
            .await
            .unwrap();

        let db = db.read().await;
        assert_eq!(
            db.get_hopr_balance().await.unwrap(),
            Balance::new(U256::zero(), BalanceType::HOPR)
        )
    }

    #[async_std::test]
    async fn on_token_approval_correct() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let log = RawLog {
            topics: vec![
                ApprovalFilter::signature(),
                H256::from_slice(&handlers.safe_address.to_bytes32()),
                H256::from_slice(&handlers.addresses.channels.to_bytes32()),
            ],
            data: encode(&[Token::Uint(EthU256::from(1000u64))]),
        };

        assert_eq!(
            db.read().await.get_staking_safe_allowance().await.unwrap(),
            Balance::new(U256::from(0u64), BalanceType::HOPR)
        );

        handlers
            .on_event(handlers.addresses.token, 0u32, log.clone(), Snapshot::default())
            .await
            .unwrap();

        assert_eq!(
            db.read().await.get_staking_safe_allowance().await.unwrap(),
            Balance::new(U256::from(1000u64), BalanceType::HOPR)
        );

        // reduce allowance manually to verify a second time
        let _ = db
            .write()
            .await
            .set_staking_safe_allowance(
                &Balance::new(U256::from(10u64), BalanceType::HOPR),
                &Snapshot::default(),
            )
            .await;
        assert_eq!(
            db.read().await.get_staking_safe_allowance().await.unwrap(),
            Balance::new(U256::from(10u64), BalanceType::HOPR)
        );

        handlers
            .on_event(handlers.addresses.token, 0u32, log, Snapshot::default())
            .await
            .unwrap();

        assert_eq!(
            db.read().await.get_staking_safe_allowance().await.unwrap(),
            Balance::new(U256::from(1000u64), BalanceType::HOPR)
        );
    }

    #[async_std::test]
    async fn on_network_registry_event_registered() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let registered_log = RawLog {
            topics: vec![
                RegisteredFilter::signature(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[]),
        };

        assert!(!db
            .read()
            .await
            .is_allowed_to_access_network(&SELF_CHAIN_ADDRESS)
            .await
            .unwrap());

        handlers
            .on_event(
                handlers.addresses.network_registry,
                0u32,
                registered_log,
                Snapshot::default(),
            )
            .await
            .unwrap();

        assert!(db
            .read()
            .await
            .is_allowed_to_access_network(&SELF_CHAIN_ADDRESS)
            .await
            .unwrap());
    }

    #[async_std::test]
    async fn on_network_registry_event_registered_by_manager() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let registered_log = RawLog {
            topics: vec![
                RegisteredByManagerFilter::signature(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[]),
        };

        assert!(!db
            .read()
            .await
            .is_allowed_to_access_network(&SELF_CHAIN_ADDRESS)
            .await
            .unwrap());

        handlers
            .on_event(
                handlers.addresses.network_registry,
                0u32,
                registered_log,
                Snapshot::default(),
            )
            .await
            .unwrap();

        assert!(db
            .read()
            .await
            .is_allowed_to_access_network(&SELF_CHAIN_ADDRESS)
            .await
            .unwrap());
    }

    #[async_std::test]
    async fn on_network_registry_event_deregistered() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        db.write()
            .await
            .set_allowed_to_access_network(&SELF_CHAIN_ADDRESS, true, &Snapshot::default())
            .await
            .unwrap();

        let registered_log = RawLog {
            topics: vec![
                DeregisteredFilter::signature(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[]),
        };

        assert!(db
            .read()
            .await
            .is_allowed_to_access_network(&SELF_CHAIN_ADDRESS)
            .await
            .unwrap());

        handlers
            .on_event(
                handlers.addresses.network_registry,
                0u32,
                registered_log,
                Snapshot::default(),
            )
            .await
            .unwrap();

        assert!(!db
            .read()
            .await
            .is_allowed_to_access_network(&SELF_CHAIN_ADDRESS)
            .await
            .unwrap());
    }

    #[async_std::test]
    async fn on_network_registry_event_deregistered_by_manager() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        db.write()
            .await
            .set_allowed_to_access_network(&SELF_CHAIN_ADDRESS, true, &Snapshot::default())
            .await
            .unwrap();

        let registered_log = RawLog {
            topics: vec![
                DeregisteredByManagerFilter::signature(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[]),
        };

        assert!(db
            .read()
            .await
            .is_allowed_to_access_network(&SELF_CHAIN_ADDRESS)
            .await
            .unwrap());

        handlers
            .on_event(
                handlers.addresses.network_registry,
                0u32,
                registered_log,
                Snapshot::default(),
            )
            .await
            .unwrap();

        assert!(!db
            .read()
            .await
            .is_allowed_to_access_network(&SELF_CHAIN_ADDRESS)
            .await
            .unwrap());
    }

    #[async_std::test]
    async fn on_network_registry_event_enabled() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let nr_enabled = RawLog {
            topics: vec![
                NetworkRegistryStatusUpdatedFilter::signature(),
                H256::from_low_u64_be(1),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                handlers.addresses.network_registry,
                0u32,
                nr_enabled,
                Snapshot::default(),
            )
            .await
            .unwrap();

        assert!(db.read().await.is_network_registry_enabled().await.unwrap());
    }

    #[async_std::test]
    async fn on_network_registry_event_disabled() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        db.write()
            .await
            .set_network_registry(true, &Snapshot::default())
            .await
            .unwrap();

        let nr_disabled = RawLog {
            topics: vec![
                NetworkRegistryStatusUpdatedFilter::signature(),
                H256::from_low_u64_be(0),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                handlers.addresses.network_registry,
                0u32,
                nr_disabled,
                Snapshot::default(),
            )
            .await
            .unwrap();

        assert!(!db.read().await.is_network_registry_enabled().await.unwrap());
    }

    #[async_std::test]
    async fn on_network_registry_set_eligible() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let set_eligible = RawLog {
            topics: vec![
                EligibilityUpdatedFilter::signature(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()),
                H256::from_low_u64_be(1),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                handlers.addresses.network_registry,
                0u32,
                set_eligible,
                Snapshot::default(),
            )
            .await
            .unwrap();

        assert!(db.read().await.is_eligible(&STAKE_ADDRESS).await.unwrap());
    }

    #[async_std::test]
    async fn on_network_registry_set_not_eligible() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        db.write()
            .await
            .set_eligible(&STAKE_ADDRESS, false, &Snapshot::default())
            .await
            .unwrap();

        let set_eligible = RawLog {
            topics: vec![
                EligibilityUpdatedFilter::signature(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()),
                H256::from_low_u64_be(0),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                handlers.addresses.network_registry,
                0u32,
                set_eligible,
                Snapshot::default(),
            )
            .await
            .unwrap();

        assert!(!db.read().await.is_eligible(&STAKE_ADDRESS).await.unwrap());
    }

    #[async_std::test]
    async fn on_channel_event_balance_increased() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let channel_id = generate_channel_id(&SELF_CHAIN_ADDRESS, &COUNTERPARTY_CHAIN_ADDRESS);

        db.write()
            .await
            .update_channel_and_snapshot(
                &channel_id,
                &ChannelEntry::new(
                    *SELF_CHAIN_ADDRESS,
                    *COUNTERPARTY_CHAIN_ADDRESS,
                    Balance::new(U256::zero(), BalanceType::HOPR),
                    U256::zero(),
                    ChannelStatus::Open,
                    U256::one(),
                ),
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let solidity_balance = U256::from((1u128 << 96) - 1);

        let balance_increased_log = RawLog {
            topics: vec![
                ChannelBalanceIncreasedFilter::signature(),
                H256::from_slice(&channel_id.to_bytes()),
            ],
            data: Vec::from(solidity_balance.to_bytes()),
        };

        handlers
            .on_event(
                handlers.addresses.channels,
                0u32,
                balance_increased_log,
                Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            db.read()
                .await
                .get_channel(&channel_id)
                .await
                .unwrap()
                .unwrap()
                .balance
                .amount(),
            solidity_balance
        );
    }

    #[async_std::test]
    async fn on_channel_event_domain_separator_updated() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let separator = Hash::default();

        let log = RawLog {
            topics: vec![
                DomainSeparatorUpdatedFilter::signature(),
                H256::from_slice(&separator.to_bytes()),
            ],
            data: encode(&[]),
        };

        assert!(db.read().await.get_channels_domain_separator().await.unwrap().is_none());

        handlers
            .on_event(handlers.addresses.channels, 0u32, log, Snapshot::default())
            .await
            .unwrap();

        assert_eq!(
            db.read().await.get_channels_domain_separator().await.unwrap().unwrap(),
            separator
        );
    }

    #[async_std::test]
    async fn on_channel_event_balance_decreased() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let channel_id = generate_channel_id(&SELF_CHAIN_ADDRESS, &COUNTERPARTY_CHAIN_ADDRESS);

        db.write()
            .await
            .update_channel_and_snapshot(
                &channel_id,
                &ChannelEntry::new(
                    *SELF_CHAIN_ADDRESS,
                    *COUNTERPARTY_CHAIN_ADDRESS,
                    Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
                    U256::zero(),
                    ChannelStatus::Open,
                    U256::one(),
                ),
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let solidity_balance = U256::from((1u128 << 96) - 1);

        let balance_increased_log = RawLog {
            topics: vec![
                ChannelBalanceDecreasedFilter::signature(),
                H256::from_slice(&channel_id.to_bytes()),
            ],
            data: Vec::from(solidity_balance.to_bytes()),
        };

        handlers
            .on_event(
                handlers.addresses.channels,
                0u32,
                balance_increased_log,
                Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            db.read()
                .await
                .get_channel(&channel_id)
                .await
                .unwrap()
                .unwrap()
                .balance
                .amount(),
            solidity_balance
        );
    }

    #[async_std::test]
    async fn on_channel_closed() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let channel_id = generate_channel_id(&SELF_CHAIN_ADDRESS, &COUNTERPARTY_CHAIN_ADDRESS);
        let starting_balance = Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR);

        db.write()
            .await
            .update_channel_and_snapshot(
                &channel_id,
                &ChannelEntry::new(
                    *SELF_CHAIN_ADDRESS,
                    *COUNTERPARTY_CHAIN_ADDRESS,
                    starting_balance,
                    U256::zero(),
                    ChannelStatus::Open,
                    U256::one(),
                ),
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let channel_closed_log = RawLog {
            topics: vec![
                ChannelClosedFilter::signature(),
                H256::from_slice(&channel_id.to_bytes()),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                handlers.addresses.channels,
                0u32,
                channel_closed_log,
                Snapshot::default(),
            )
            .await
            .unwrap();

        let closed_channel = db.read().await.get_channel(&channel_id).await.unwrap().unwrap();
        let current_ticket_index = db
            .read()
            .await
            .get_current_ticket_index(&channel_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(closed_channel.status, ChannelStatus::Closed);
        assert_eq!(closed_channel.ticket_index, 0u64.into());

        assert!(closed_channel.balance.amount().eq(&U256::zero()));
        assert!(current_ticket_index.eq(&U256::zero()));
    }

    #[async_std::test]
    async fn on_channel_opened() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let channel_id = generate_channel_id(&SELF_CHAIN_ADDRESS, &COUNTERPARTY_CHAIN_ADDRESS);

        let channel_opened_log = RawLog {
            topics: vec![
                ChannelOpenedFilter::signature(),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
                H256::from_slice(&COUNTERPARTY_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                handlers.addresses.channels,
                0u32,
                channel_opened_log,
                Snapshot::default(),
            )
            .await
            .unwrap();

        let channel = db.read().await.get_channel(&channel_id).await.unwrap().unwrap();

        assert_eq!(channel.status, ChannelStatus::Open);
        assert_eq!(channel.channel_epoch, 1u64.into());
        assert_eq!(channel.ticket_index, 0u64.into());

        let current_ticket_index = db
            .read()
            .await
            .get_current_ticket_index(&channel_id)
            .await
            .unwrap()
            .unwrap();
        assert!(current_ticket_index.eq(&U256::zero()));
    }

    #[async_std::test]
    async fn on_channel_reopened() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let channel_id = generate_channel_id(&SELF_CHAIN_ADDRESS, &COUNTERPARTY_CHAIN_ADDRESS);

        db.write()
            .await
            .update_channel_and_snapshot(
                &channel_id,
                &ChannelEntry::new(
                    *SELF_CHAIN_ADDRESS,
                    *COUNTERPARTY_CHAIN_ADDRESS,
                    Balance::zero(BalanceType::HOPR),
                    U256::zero(),
                    ChannelStatus::Open,
                    3u64.into(),
                ),
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let channel_opened_log = RawLog {
            topics: vec![
                ChannelOpenedFilter::signature(),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
                H256::from_slice(&COUNTERPARTY_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                handlers.addresses.channels,
                0u32,
                channel_opened_log,
                Snapshot::default(),
            )
            .await
            .unwrap();

        let channel = db.read().await.get_channel(&channel_id).await.unwrap().unwrap();

        assert_eq!(channel.status, ChannelStatus::Open);
        assert_eq!(channel.channel_epoch, 4u64.into());
        assert_eq!(channel.ticket_index, 0u64.into());

        // after the channel epoch is bumped, the ticket index gets reset to zero
        let current_ticket_index = db
            .read()
            .await
            .get_current_ticket_index(&channel_id)
            .await
            .unwrap()
            .unwrap();
        assert!(current_ticket_index.eq(&U256::zero()));
    }

    #[async_std::test]
    async fn on_channel_ticket_redeemed() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let channel_id = generate_channel_id(&SELF_CHAIN_ADDRESS, &COUNTERPARTY_CHAIN_ADDRESS);

        db.write()
            .await
            .update_channel_and_snapshot(
                &channel_id,
                &ChannelEntry::new(
                    *SELF_CHAIN_ADDRESS,
                    *COUNTERPARTY_CHAIN_ADDRESS,
                    Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
                    U256::zero(),
                    ChannelStatus::Open,
                    U256::one(),
                ),
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let ticket_index = U256::from((1u128 << 48) - 1);

        let ticket_redeemed_log = RawLog {
            topics: vec![
                TicketRedeemedFilter::signature(),
                H256::from_slice(&channel_id.to_bytes()),
            ],
            data: Vec::from(ticket_index.to_bytes()),
        };

        handlers
            .on_event(
                handlers.addresses.channels,
                0u32,
                ticket_redeemed_log,
                Snapshot::default(),
            )
            .await
            .unwrap();

        let channel = db.read().await.get_channel(&channel_id).await.unwrap().unwrap();

        assert_eq!(channel.ticket_index, ticket_index);

        // check the current_ticket_index is not smaller than the new ticket index
        let current_ticket_index = db
            .read()
            .await
            .get_current_ticket_index(&channel_id)
            .await
            .unwrap()
            .unwrap();
        assert!(current_ticket_index.ge(&ticket_index));
    }

    #[async_std::test]
    async fn on_channel_closure_initiated() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let channel_id = generate_channel_id(&SELF_CHAIN_ADDRESS, &COUNTERPARTY_CHAIN_ADDRESS);

        db.write()
            .await
            .update_channel_and_snapshot(
                &channel_id,
                &ChannelEntry::new(
                    *SELF_CHAIN_ADDRESS,
                    *COUNTERPARTY_CHAIN_ADDRESS,
                    Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
                    U256::zero(),
                    ChannelStatus::Open,
                    U256::one(),
                ),
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let timestamp = SystemTime::now();
        //let timestamp = U256::from((1u64 << 32) - 1);

        let closure_initiated_log = RawLog {
            topics: vec![
                OutgoingChannelClosureInitiatedFilter::signature(),
                H256::from_slice(&channel_id.to_bytes()),
            ],
            data: Vec::from(U256::from(timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs()).to_bytes()),
        };

        handlers
            .on_event(
                handlers.addresses.channels,
                0u32,
                closure_initiated_log,
                Snapshot::default(),
            )
            .await
            .unwrap();

        // TODO: check for Vec<ChainEventType> content here

        let channel = db.read().await.get_channel(&channel_id).await.unwrap().unwrap();

        assert_eq!(channel.status, ChannelStatus::PendingToClose(timestamp));
    }

    #[async_std::test]
    async fn on_node_safe_registry_registered() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let safe_registered_log = RawLog {
            topics: vec![
                RegisteredNodeSafeFilter::signature(),
                H256::from_slice(&SAFE_INSTANCE_ADDR.to_bytes32()),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                handlers.addresses.safe_registry,
                0u32,
                safe_registered_log,
                Snapshot::default(),
            )
            .await
            .unwrap();

        // TODO: check for Vec<ChainEventType> content here

        assert_eq!(
            db.read().await.is_mfa_protected().await.unwrap(),
            Some(*SAFE_INSTANCE_ADDR)
        );
    }

    #[async_std::test]
    async fn on_node_safe_registry_deregistered() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        db.write()
            .await
            .set_mfa_protected_and_update_snapshot(Some(*SAFE_INSTANCE_ADDR), &Snapshot::default())
            .await
            .unwrap();

        let safe_registered_log = RawLog {
            topics: vec![
                DergisteredNodeSafeFilter::signature(),
                H256::from_slice(&SAFE_INSTANCE_ADDR.to_bytes32()),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                handlers.addresses.safe_registry,
                0u32,
                safe_registered_log,
                Snapshot::default(),
            )
            .await
            .unwrap();

        // TODO: check for Vec<ChainEventType> content here

        assert_eq!(db.read().await.is_mfa_protected().await.unwrap(), None);
    }

    #[async_std::test]
    async fn ticket_price_update() {
        let db = create_db().await;

        let handlers = init_handlers(db.clone());

        let log = RawLog {
            topics: vec![TicketPriceUpdatedFilter::signature()],
            data: encode(&[Token::Uint(EthU256::from(1u64)), Token::Uint(EthU256::from(123u64))]),
        };

        assert_eq!(db.read().await.get_ticket_price().await.unwrap(), None);

        handlers
            .on_event(handlers.addresses.price_oracle, 0u32, log, Snapshot::default())
            .await
            .unwrap();

        // TODO: check for Vec<ChainEventType> content here

        assert_eq!(
            db.read().await.get_ticket_price().await.unwrap(),
            Some(U256::from(123u64))
        );
    }
}
