use crate::errors::{CoreEthereumIndexerError, Result};
use bindings::{
    hopr_announcements::{
        AddressAnnouncementFilter, HoprAnnouncementsEvents, KeyBindingFilter, RevokeAnnouncementFilter,
    },
    hopr_channels::{
        ChannelBalanceDecreasedFilter, ChannelBalanceIncreasedFilter, ChannelClosedFilter, ChannelOpenedFilter,
        DomainSeparatorUpdatedFilter, HoprChannelsEvents, LedgerDomainSeparatorUpdatedFilter,
        OutgoingChannelClosureInitiatedFilter, TicketRedeemedFilter,
    },
    hopr_network_registry::{
        DeregisteredByManagerFilter, DeregisteredFilter, EligibilityUpdatedFilter, HoprNetworkRegistryEvents,
        NetworkRegistryStatusUpdatedFilter, RegisteredByManagerFilter, RegisteredFilter, RequirementUpdatedFilter,
    },
    hopr_node_management_module::HoprNodeManagementModuleEvents,
    hopr_node_safe_registry::{DergisteredNodeSafeFilter, HoprNodeSafeRegistryEvents, RegisteredNodeSafeFilter},
    hopr_ticket_price_oracle::{HoprTicketPriceOracleEvents, TicketPriceUpdatedFilter},
    hopr_token::{ApprovalFilter, HoprTokenEvents, TransferFilter},
};
use core_crypto::types::{Hash, OffchainSignature};
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::{
    account::{AccountEntry, AccountType},
    announcement::KeyBinding,
    channels::{generate_channel_id, ChannelEntry, ChannelStatus},
};
use ethers::{
    contract::{EthEvent, EthLogDecode},
    core::abi::RawLog,
    types::TxHash,
};
use ethnum::u256;
use multiaddr::Multiaddr;
use serde::Deserialize;
use std::str::FromStr;
use utils_log::{debug, error};
use utils_types::primitives::{Address, Balance, BalanceType, Snapshot, U256};

/// Holds addresses of deployed HOPR contracts
#[derive(Clone, Debug, Deserialize)]
pub struct ContractAddresses {
    /// HoprChannels contract, manages mixnet incentives
    pub channels: Address,
    /// HoprToken contract, the HOPR token
    pub token: Address,
    /// HoprNetworkRegistry contract, manages authorization to
    /// participate in the HOPR network
    pub network_registry: Address,
    /// HoprAnnouncements, announces network information
    pub announcements: Address,
    /// HoprNodeSafeRegistry, mapping from chain_key to Safe instance
    pub node_safe_registry: Address,
    /// NodeManagementModule, permission module for Safe
    pub node_management_module: Address,
    /// TicketPriceOracle, used to set ticket price
    pub ticket_price_oracle: Address,
}

#[cfg(feature = "wasm")]
impl From<&wasm::ContractAddresses> for ContractAddresses {
    fn from(x: &wasm::ContractAddresses) -> Self {
        ContractAddresses {
            channels: Address::from_str(&x.channels).expect("invalid channels address given"),
            token: Address::from_str(&x.token).expect("invalid token address given"),
            network_registry: Address::from_str(&x.network_registry).expect("invalid network_registry address given"),
            announcements: Address::from_str(&x.announcements).expect("invalid announcements address given"),
            node_safe_registry: Address::from_str(&x.node_safe_registry)
                .expect("invalid node_safe_registry address given"),
            node_management_module: Address::from_str(&x.node_management_module)
                .expect("invalid node_management_module address given"),
            ticket_price_oracle: Address::from_str(&x.ticket_price_oracle)
                .expect("invalid ticket_price_oracle address given"),
        }
    }
}

pub trait IndexerCallbacks {
    fn own_channel_updated(&self, channel_entry: &ChannelEntry);

    fn node_not_allowed_to_access_network(&self, address: &Address);

    fn node_allowed_to_access_network(&self, address: &Address);

    fn new_announcement(&self, account_entry: &AccountEntry);
}

pub struct ContractEventHandlers<Cbs> {
    /// channels, announcements, network_registry, token: contract addresses
    /// whose event we process
    addresses: ContractAddresses,
    /// monitor the Hopr Token events, ignore rest
    address_to_monitor: Address,
    /// own address, aka msg.sender
    chain_key: Address,
    /// callbacks to inform other modules
    cbs: Cbs,
}

impl<Cbs> ContractEventHandlers<Cbs>
where
    Cbs: IndexerCallbacks,
{
    pub fn get_channel_topics(&self) -> Vec<TxHash> {
        vec![
            ChannelBalanceDecreasedFilter::signature(),
            ChannelBalanceIncreasedFilter::signature(),
            ChannelClosedFilter::signature(),
            ChannelOpenedFilter::signature(),
            OutgoingChannelClosureInitiatedFilter::signature(),
            TicketRedeemedFilter::signature(),
            DomainSeparatorUpdatedFilter::signature(),
            LedgerDomainSeparatorUpdatedFilter::signature(),
        ]
    }

    pub fn get_token_topics(&self) -> Vec<TxHash> {
        vec![TransferFilter::signature(), ApprovalFilter::signature()]
    }

    pub fn get_network_registry_topics(&self) -> Vec<TxHash> {
        vec![
            DeregisteredByManagerFilter::signature(),
            DeregisteredFilter::signature(),
            EligibilityUpdatedFilter::signature(),
            NetworkRegistryStatusUpdatedFilter::signature(),
            RegisteredByManagerFilter::signature(),
            RegisteredFilter::signature(),
            RequirementUpdatedFilter::signature(),
        ]
    }

    pub fn get_announcement_topics(&self) -> Vec<TxHash> {
        vec![
            AddressAnnouncementFilter::signature(),
            KeyBindingFilter::signature(),
            RevokeAnnouncementFilter::signature(),
        ]
    }

    pub fn get_node_safe_registry_topics(&self) -> Vec<TxHash> {
        vec![
            RegisteredNodeSafeFilter::signature(),
            DergisteredNodeSafeFilter::signature(),
            bindings::hopr_node_safe_registry::DomainSeparatorUpdatedFilter::signature(),
        ]
    }

    pub fn get_ticket_price_oracle_topics(&self) -> Vec<TxHash> {
        vec![TicketPriceUpdatedFilter::signature()]
    }

    async fn on_announcement_event<T>(
        &self,
        db: &mut T,
        log: &RawLog,
        block_number: u32,
        snapshot: &Snapshot,
    ) -> Result<()>
    where
        T: HoprCoreEthereumDbActions,
    {
        match HoprAnnouncementsEvents::decode_log(log)? {
            HoprAnnouncementsEvents::AddressAnnouncementFilter(address_announcement) => {
                let maybe_account = db.get_account(&address_announcement.node.try_into()?).await?;

                debug!(
                    "on_announcement_event - multiaddr: {:?} - node: {:?}",
                    &address_announcement.base_multiaddr,
                    &address_announcement.node.to_string()
                );

                if let Some(mut account) = maybe_account {
                    let new_entry_type = AccountType::Announced {
                        multiaddr: Multiaddr::from_str(&address_announcement.base_multiaddr)?,
                        updated_block: block_number,
                    };

                    account.update(new_entry_type);
                    db.update_account_and_snapshot(&account, snapshot).await?;

                    self.cbs.new_announcement(&account);
                } else {
                    return Err(CoreEthereumIndexerError::AnnounceBeforeKeyBinding);
                }
            }
            HoprAnnouncementsEvents::KeyBindingFilter(key_binding) => {
                if db.get_account(&key_binding.chain_key.try_into()?).await?.is_some() {
                    return Err(CoreEthereumIndexerError::UnsupportedKeyRebinding);
                }

                match KeyBinding::from_parts(
                    key_binding.chain_key.try_into()?,
                    key_binding.ed_25519_pub_key.try_into()?,
                    OffchainSignature::try_from((key_binding.ed_25519_sig_0, key_binding.ed_25519_sig_1))?,
                ) {
                    Ok(binding) => {
                        let updated_account = AccountEntry::new(
                            key_binding.ed_25519_pub_key.try_into()?,
                            key_binding.chain_key.try_into()?,
                            AccountType::NotAnnounced,
                        );

                        db.link_chain_and_packet_keys(&binding.chain_key, &binding.packet_key, snapshot)
                            .await?;

                        db.update_account_and_snapshot(&updated_account, snapshot).await?;
                    }
                    Err(_) => {
                        debug!(
                            "Filtering announcement from {} with invalid signature.",
                            key_binding.chain_key
                        )
                    }
                }
            }
            HoprAnnouncementsEvents::RevokeAnnouncementFilter(revocation) => {
                let maybe_account = db.get_account(&revocation.node.try_into()?).await?;

                if let Some(mut account) = maybe_account {
                    account.update(AccountType::NotAnnounced);
                    db.update_account_and_snapshot(&account, snapshot).await?;
                } else {
                    return Err(CoreEthereumIndexerError::RevocationBeforeKeyBinding);
                }
            }
        };
        Ok(())
    }

    async fn on_channel_event<T>(&self, db: &mut T, log: &RawLog, snapshot: &Snapshot) -> Result<()>
    where
        T: HoprCoreEthereumDbActions,
    {
        match HoprChannelsEvents::decode_log(log)? {
            HoprChannelsEvents::ChannelBalanceDecreasedFilter(balance_decreased) => {
                let maybe_channel = db.get_channel(&balance_decreased.channel_id.try_into()?).await?;

                if let Some(mut channel) = maybe_channel {
                    channel.balance = channel
                        .balance
                        .sub(&Balance::new(balance_decreased.new_balance.into(), BalanceType::HOPR));

                    db.update_channel_and_snapshot(&balance_decreased.channel_id.try_into()?, &channel, snapshot)
                        .await?;

                    if channel.source.eq(&self.chain_key) || channel.destination.eq(&self.chain_key) {
                        self.cbs.own_channel_updated(&channel);
                    }
                } else {
                    return Err(CoreEthereumIndexerError::ChannelDoesNotExist);
                }
            }
            HoprChannelsEvents::ChannelBalanceIncreasedFilter(balance_increased) => {
                let maybe_channel = db.get_channel(&balance_increased.channel_id.try_into()?).await?;

                if let Some(mut channel) = maybe_channel {
                    channel.balance = channel
                        .balance
                        .add(&Balance::new(balance_increased.new_balance.into(), BalanceType::HOPR));

                    db.update_channel_and_snapshot(&balance_increased.channel_id.try_into()?, &channel, snapshot)
                        .await?;

                    if channel.source.eq(&self.chain_key) || channel.destination.eq(&self.chain_key) {
                        self.cbs.own_channel_updated(&channel);
                    }
                } else {
                    return Err(CoreEthereumIndexerError::ChannelDoesNotExist);
                }
            }
            HoprChannelsEvents::ChannelClosedFilter(channel_closed) => {
                let maybe_channel = db.get_channel(&channel_closed.channel_id.try_into()?).await?;

                if let Some(mut channel) = maybe_channel {
                    channel.status = ChannelStatus::Closed;

                    // Incoming channel, so once closed. All unredeemed tickets just became invalid
                    if channel.destination.eq(&self.chain_key) {
                        db.delete_acknowledged_tickets_from(channel).await?;
                    }

                    db.update_channel_and_snapshot(&channel_closed.channel_id.try_into()?, &channel, snapshot)
                        .await?;

                    if channel.source.eq(&self.chain_key) || channel.destination.eq(&self.chain_key) {
                        self.cbs.own_channel_updated(&channel);
                    }
                } else {
                    return Err(CoreEthereumIndexerError::ChannelDoesNotExist);
                }
            }
            HoprChannelsEvents::ChannelOpenedFilter(channel_opened) => {
                let source: Address = channel_opened.source.0.try_into()?;
                let destination: Address = channel_opened.destination.0.try_into()?;

                let channel_id = generate_channel_id(&source, &destination);

                let maybe_channel = db.get_channel(&channel_id).await?;
                debug!(
                    "on_open_channel_event - source: {:?} - destination: {:?} - channel_id: {:?}, channel known: {:?}",
                    source.to_string(),
                    destination.to_string(),
                    channel_id.to_string(),
                    maybe_channel.is_some()
                );

                if let Some(mut channel) = maybe_channel {
                    channel.status = ChannelStatus::Open;

                    db.update_channel_and_snapshot(&channel_id, &channel, snapshot).await?;

                    if source.eq(&self.chain_key) || destination.eq(&self.chain_key) {
                        self.cbs.own_channel_updated(&channel);
                    }
                } else {
                    let new_channel = ChannelEntry::new(
                        source,
                        destination,
                        Balance::new(0u64.into(), utils_types::primitives::BalanceType::HOPR),
                        0u64.into(),
                        ChannelStatus::Open,
                        1u64.into(),
                        0u64.into(),
                    );

                    db.update_channel_and_snapshot(&channel_id, &new_channel, snapshot)
                        .await?;

                    if source.eq(&self.chain_key) || destination.eq(&self.chain_key) {
                        self.cbs.own_channel_updated(&new_channel);
                    }
                }
            }
            HoprChannelsEvents::TicketRedeemedFilter(ticket_redeemed) => {
                let maybe_channel = db.get_channel(&ticket_redeemed.channel_id.try_into()?).await?;

                if let Some(mut channel) = maybe_channel {
                    channel.ticket_index = ticket_redeemed.new_ticket_index.into();

                    db.update_channel_and_snapshot(&ticket_redeemed.channel_id.try_into()?, &channel, snapshot)
                        .await?;

                    if channel.source.eq(&self.chain_key) || channel.destination.eq(&self.chain_key) {
                        self.cbs.own_channel_updated(&channel);
                    }
                } else {
                    return Err(CoreEthereumIndexerError::ChannelDoesNotExist);
                }
            }
            HoprChannelsEvents::OutgoingChannelClosureInitiatedFilter(closure_initiated) => {
                let maybe_channel = db.get_channel(&closure_initiated.channel_id.try_into()?).await?;

                if let Some(mut channel) = maybe_channel {
                    channel.status = ChannelStatus::PendingToClose;
                    channel.closure_time = closure_initiated.closure_time.into();

                    db.update_channel_and_snapshot(&closure_initiated.channel_id.try_into()?, &channel, snapshot)
                        .await?;

                    if channel.source.eq(&self.chain_key) || channel.destination.eq(&self.chain_key) {
                        self.cbs.own_channel_updated(&channel);
                    }
                } else {
                    return Err(CoreEthereumIndexerError::ChannelDoesNotExist);
                }
            }
            HoprChannelsEvents::DomainSeparatorUpdatedFilter(domain_separator_updated) => {
                db.set_channels_domain_separator(&Hash::try_from(domain_separator_updated.domain_separator)?, snapshot)
                    .await?;
            }
            HoprChannelsEvents::LedgerDomainSeparatorUpdatedFilter(ledger_domain_separator_updated) => {
                db.set_channels_ledger_domain_separator(
                    &Hash::try_from(ledger_domain_separator_updated.ledger_domain_separator)?,
                    snapshot,
                )
                .await?;
            }
        }
        Ok(())
    }

    async fn on_token_event<T>(&self, db: &mut T, log: &RawLog, snapshot: &Snapshot) -> Result<()>
    where
        T: HoprCoreEthereumDbActions,
    {
        match HoprTokenEvents::decode_log(log)? {
            HoprTokenEvents::TransferFilter(transfered) => {
                let from: Address = transfered.from.0.try_into()?;
                let to: Address = transfered.to.0.try_into()?;

                let value: U256 = u256::from_be_bytes(transfered.value.into()).into();

                debug!(
                    "on_token_transfer_event - address_to_monitor: {:?} - to: {:?} - from: {:?}",
                    &self.address_to_monitor.to_string(),
                    to.to_string(),
                    from.to_string()
                );

                if to.ne(&self.address_to_monitor) && from.ne(&self.address_to_monitor) {
                    return Ok(());
                } else if to.eq(&self.address_to_monitor) {
                    db.add_hopr_balance(&Balance::new(value, BalanceType::HOPR), snapshot)
                        .await?;
                } else if from.eq(&self.address_to_monitor) {
                    db.sub_hopr_balance(&Balance::new(value, BalanceType::HOPR), snapshot)
                        .await?;
                }
            }
            HoprTokenEvents::ApprovalFilter(approved) => {
                let owner: Address = approved.owner.0.try_into()?;
                let spender: Address = approved.spender.0.try_into()?;

                let allowance: U256 = u256::from_be_bytes(approved.value.into()).into();

                debug!(
                    "on_token_approval_event - address_to_monitor: {:?} - owner: {:?} - spender: {:?}, allowance: {:?}",
                    &self.address_to_monitor.to_string(),
                    owner.to_string(),
                    spender.to_string(),
                    allowance.to_string()
                );

                if owner.eq(&self.address_to_monitor) && spender.eq(&self.addresses.channels) {
                    db.set_staking_safe_allowance(&Balance::new(allowance, BalanceType::HOPR), snapshot)
                        .await?;
                } else {
                    return Ok(());
                }
            }
            _ => debug!("Implement all the other filters for HoprTokenEvents"),
        }

        Ok(())
    }

    async fn on_network_registry_event<T>(&self, db: &mut T, log: &RawLog, snapshot: &Snapshot) -> Result<()>
    where
        T: HoprCoreEthereumDbActions,
    {
        match HoprNetworkRegistryEvents::decode_log(log)? {
            HoprNetworkRegistryEvents::DeregisteredByManagerFilter(deregistered) => {
                let node_address = &deregistered.node_address.0.try_into()?;
                db.remove_from_network_registry(
                    &deregistered.staking_account.0.try_into()?,
                    &deregistered.node_address.0.try_into()?,
                    snapshot,
                )
                .await?;
                db.set_allowed_to_access_network(node_address, false, snapshot).await?;
                self.cbs
                    .node_not_allowed_to_access_network(&deregistered.node_address.0.try_into()?);
            }
            HoprNetworkRegistryEvents::DeregisteredFilter(deregistered) => {
                let node_address = &deregistered.node_address.0.try_into()?;
                db.remove_from_network_registry(
                    &deregistered.staking_account.0.try_into()?,
                    &deregistered.node_address.0.try_into()?,
                    snapshot,
                )
                .await?;
                db.set_allowed_to_access_network(node_address, false, snapshot).await?;
                self.cbs
                    .node_not_allowed_to_access_network(&deregistered.node_address.0.try_into()?);
            }
            HoprNetworkRegistryEvents::RegisteredByManagerFilter(registered) => {
                let node_address = &registered.node_address.0.try_into()?;
                db.add_to_network_registry(
                    &registered.staking_account.0.try_into()?,
                    &registered.node_address.0.try_into()?,
                    snapshot,
                )
                .await?;
                db.set_allowed_to_access_network(node_address, true, snapshot).await?;
                self.cbs.node_allowed_to_access_network(node_address);
            }
            HoprNetworkRegistryEvents::RegisteredFilter(registered) => {
                let node_address = &registered.node_address.0.try_into()?;
                db.add_to_network_registry(
                    &registered.staking_account.0.try_into()?,
                    &registered.node_address.0.try_into()?,
                    snapshot,
                )
                .await?;
                db.set_allowed_to_access_network(node_address, true, snapshot).await?;
                self.cbs
                    .node_allowed_to_access_network(&registered.node_address.0.try_into()?);
            }
            HoprNetworkRegistryEvents::EligibilityUpdatedFilter(eligibility_updated) => {
                let account: Address = eligibility_updated.staking_account.0.try_into()?;
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
        }
        Ok(())
    }

    async fn on_node_safe_registry_event<T>(&self, db: &mut T, log: &RawLog, snapshot: &Snapshot) -> Result<()>
    where
        T: HoprCoreEthereumDbActions,
    {
        match HoprNodeSafeRegistryEvents::decode_log(log)? {
            HoprNodeSafeRegistryEvents::RegisteredNodeSafeFilter(registered) => {
                if self.chain_key.eq(&registered.node_address.0.try_into()?) {
                    db.set_mfa_protected_and_update_snapshot(Some(registered.safe_address.0.try_into()?), snapshot)
                        .await?;
                }
            }
            HoprNodeSafeRegistryEvents::DergisteredNodeSafeFilter(deregistered) => {
                if self.chain_key.eq(&deregistered.node_address.0.try_into()?) {
                    if db.is_mfa_protected().await?.is_some() {
                        db.set_mfa_protected_and_update_snapshot(None, snapshot).await?;
                    } else {
                        return Err(CoreEthereumIndexerError::MFAModuleDoesNotExist);
                    }
                }
            }
            HoprNodeSafeRegistryEvents::DomainSeparatorUpdatedFilter(domain_separator_updated) => {
                db.set_node_safe_registry_domain_separator(
                    &Hash::try_from(domain_separator_updated.domain_separator)?,
                    snapshot,
                )
                .await?;
            }
        }
        Ok(())
    }

    async fn on_node_management_module_event<T>(&self, _db: &mut T, log: &RawLog, _snapshot: &Snapshot) -> Result<()>
    where
        T: HoprCoreEthereumDbActions,
    {
        match HoprNodeManagementModuleEvents::decode_log(log)? {
            _ => {
                // don't care at the moment
            }
        }
        Ok(())
    }

    async fn on_ticket_price_oracle_event<T>(&self, db: &mut T, log: &RawLog, _snapshot: &Snapshot) -> Result<()>
    where
        T: HoprCoreEthereumDbActions,
    {
        match HoprTicketPriceOracleEvents::decode_log(log)? {
            HoprTicketPriceOracleEvents::TicketPriceUpdatedFilter(update) => {
                let old_price: U256 = u256::from_be_bytes(update.0.into()).into();
                let new_price: U256 = u256::from_be_bytes(update.1.into()).into();

                debug!(
                    "on_ticket_price_updated - old: {:?} - new: {:?}",
                    old_price.to_string(),
                    new_price.to_string()
                );

                db.set_ticket_price(&new_price).await?;
            }
            HoprTicketPriceOracleEvents::OwnershipTransferredFilter(_event) => {
                // ignore ownership transfer event
            }
        }
        Ok(())
    }

    pub async fn on_event<T>(
        &self,
        db: &mut T,
        address: &Address,
        block_number: u32,
        log: &RawLog,
        snapshot: &Snapshot,
    ) -> Result<()>
    where
        T: HoprCoreEthereumDbActions,
    {
        debug!(
            "on_event - address: {:?} - received log: {:?}",
            address.to_string(),
            log
        );

        if address.eq(&self.addresses.announcements) {
            return self.on_announcement_event(db, log, block_number, snapshot).await;
        } else if address.eq(&self.addresses.channels) {
            return self.on_channel_event(db, log, snapshot).await;
        } else if address.eq(&self.addresses.network_registry) {
            return self.on_network_registry_event(db, log, snapshot).await;
        } else if address.eq(&self.addresses.token) {
            return self.on_token_event(db, log, snapshot).await;
        } else if address.eq(&self.addresses.node_safe_registry) {
            return self.on_node_safe_registry_event(db, log, snapshot).await;
        } else if address.eq(&self.addresses.node_management_module) {
            return self.on_node_management_module_event(db, log, snapshot).await;
        } else if address.eq(&self.addresses.ticket_price_oracle) {
            return self.on_ticket_price_oracle_event(db, log, snapshot).await;
        } else {
            error!(
                "on_event error - unknown contract address: {:?} - received log: {:?}",
                address.to_string(),
                log
            );

            return Err(CoreEthereumIndexerError::UnknownContract(*address));
        }
    }
}

#[cfg(test)]
pub mod tests {
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
        hopr_token::TransferFilter,
    };
    use core_crypto::{
        keypairs::{Keypair, OffchainKeypair},
        types::Hash,
    };
    use core_ethereum_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
    use core_types::{
        account::{AccountEntry, AccountType},
        announcement::KeyBinding,
        channels::{generate_channel_id, ChannelEntry, ChannelStatus},
    };
    use ethers::{
        abi::{encode, Address as EthereumAddress, RawLog, Token},
        prelude::*,
        types::U256 as EthU256,
    };
    use hex_literal::hex;
    use multiaddr::Multiaddr;
    use primitive_types::H256;
    use std::sync::{Arc, Mutex};
    use utils_db::{db::DB, leveldb::rusty::RustyLevelDbShim};
    use utils_types::{
        primitives::{Address, Balance, BalanceType, Snapshot, U256},
        traits::BinarySerializable,
    };

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

    fn create_mock_db() -> CoreEthereumDb<RustyLevelDbShim> {
        let opt = rusty_leveldb::in_memory();
        let db = rusty_leveldb::DB::open("test", opt).unwrap();

        CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new(Arc::new(Mutex::new(db)))),
            Address::random(),
        )
    }

    struct DummyCallbacks {}

    impl super::IndexerCallbacks for DummyCallbacks {
        fn new_announcement(&self, _account_entry: &AccountEntry) {}

        fn own_channel_updated(&self, _channel_entry: &ChannelEntry) {}

        fn node_not_allowed_to_access_network(&self, _address: &Address) {}

        fn node_allowed_to_access_network(&self, _address: &Address) {}
    }

    fn init_handlers() -> ContractEventHandlers<DummyCallbacks> {
        ContractEventHandlers {
            addresses: super::ContractAddresses {
                channels: *CHANNELS_ADDR,
                token: *TOKEN_ADDR,
                network_registry: *NETWORK_REGISTRY_ADDR,
                node_safe_registry: *NODE_SAFE_REGISTRY_ADDR,
                announcements: *ANNOUNCEMENTS_ADDR,
                node_management_module: *SAFE_MANAGEMENT_MODULE_ADDR,
                ticket_price_oracle: *TICKET_PRICE_ORACLE_ADDR,
            },
            chain_key: *SELF_CHAIN_ADDRESS,
            address_to_monitor: *SELF_CHAIN_ADDRESS,
            cbs: DummyCallbacks {},
        }
    }

    #[async_std::test]
    async fn announce_keybinding() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let keybinding = KeyBinding::new(*SELF_CHAIN_ADDRESS, &SELF_PRIV_KEY);

        let keybinding_log = RawLog {
            topics: vec![KeyBindingFilter::signature()],
            data: encode(&[
                Token::FixedBytes(Vec::from(keybinding.signature.to_bytes())),
                Token::FixedBytes(Vec::from(keybinding.packet_key.to_bytes())),
                Token::Address(EthereumAddress::from_slice(&SELF_CHAIN_ADDRESS.to_bytes())),
            ]),
        };

        let account_entry = AccountEntry::new(
            SELF_PRIV_KEY.public().clone(),
            *SELF_CHAIN_ADDRESS,
            AccountType::NotAnnounced,
        );

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.announcements,
                0u32,
                &keybinding_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            db.get_account(&SELF_CHAIN_ADDRESS).await.unwrap().unwrap(),
            account_entry
        );
    }

    #[async_std::test]
    async fn announce_address_announcement() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        // Assume that there is a keybinding
        let account_entry = AccountEntry::new(
            SELF_PRIV_KEY.public().clone(),
            *SELF_CHAIN_ADDRESS,
            AccountType::NotAnnounced,
        );

        db.update_account_and_snapshot(&account_entry, &Snapshot::default())
            .await
            .unwrap();

        let test_multiaddr: Multiaddr = "/ip4/1.2.3.4/tcp/56".parse().unwrap();

        let address_announcement_log = RawLog {
            topics: vec![AddressAnnouncementFilter::signature()],
            data: encode(&[
                Token::Address(EthereumAddress::from_slice(&SELF_CHAIN_ADDRESS.to_bytes())),
                Token::String(test_multiaddr.to_string()),
            ]),
        };

        let announced_account_entry = AccountEntry::new(
            SELF_PRIV_KEY.public().clone(),
            *SELF_CHAIN_ADDRESS,
            AccountType::Announced {
                multiaddr: test_multiaddr,
                updated_block: 0,
            },
        );

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.announcements,
                0u32,
                &address_announcement_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            db.get_account(&SELF_CHAIN_ADDRESS).await.unwrap().unwrap(),
            announced_account_entry
        );
    }

    #[async_std::test]
    async fn announce_revoke() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let test_multiaddr: Multiaddr = "/ip4/1.2.3.4/tcp/56".parse().unwrap();

        // Assume that there is a keybinding and an address announcement
        let announced_account_entry = AccountEntry::new(
            SELF_PRIV_KEY.public().clone(),
            *SELF_CHAIN_ADDRESS,
            AccountType::Announced {
                multiaddr: test_multiaddr,
                updated_block: 0,
            },
        );

        db.update_account_and_snapshot(&announced_account_entry, &Snapshot::default())
            .await
            .unwrap();

        let revoke_announcement_log = RawLog {
            topics: vec![RevokeAnnouncementFilter::signature()],
            data: encode(&[Token::Address(EthereumAddress::from_slice(
                &SELF_CHAIN_ADDRESS.to_bytes(),
            ))]),
        };

        let account_entry = AccountEntry::new(
            SELF_PRIV_KEY.public().clone(),
            *SELF_CHAIN_ADDRESS,
            AccountType::NotAnnounced,
        );

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.announcements,
                0u32,
                &revoke_announcement_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            db.get_account(&SELF_CHAIN_ADDRESS).await.unwrap().unwrap(),
            account_entry
        );
    }

    #[async_std::test]
    async fn on_token_transfer_to() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let value = U256::max();

        let transferred_log = RawLog {
            topics: vec![
                TransferFilter::signature(),
                H256::from_slice(&Address::default().to_bytes32()),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[Token::Uint(EthU256::from_big_endian(&value.to_bytes()))]),
        };

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.token,
                0u32,
                &transferred_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            db.get_hopr_balance().await.unwrap(),
            Balance::new(value, BalanceType::HOPR)
        )
    }

    #[async_std::test]
    async fn on_token_transfer_from() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let value = U256::max();

        db.set_hopr_balance(&Balance::new(value, BalanceType::HOPR))
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
            .on_event(
                &mut db,
                &handlers.addresses.token,
                0u32,
                &transferred_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            db.get_hopr_balance().await.unwrap(),
            Balance::new(U256::zero(), BalanceType::HOPR)
        )
    }

    #[async_std::test]
    async fn on_network_registry_event_registered() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let registered_log = RawLog {
            topics: vec![
                RegisteredFilter::signature(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.network_registry,
                0u32,
                &registered_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let stored = db.get_from_network_registry(&STAKE_ADDRESS).await.unwrap();

        assert_eq!(stored, vec![*SELF_CHAIN_ADDRESS]);
    }

    #[async_std::test]
    async fn on_network_registry_event_registered_by_manager() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let registered_log = RawLog {
            topics: vec![
                RegisteredByManagerFilter::signature(),
                H256::from_slice(&STAKE_ADDRESS.to_bytes32()),
                H256::from_slice(&SELF_CHAIN_ADDRESS.to_bytes32()),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.network_registry,
                0u32,
                &registered_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let stored = db.get_from_network_registry(&STAKE_ADDRESS).await.unwrap();

        assert_eq!(stored, vec![*SELF_CHAIN_ADDRESS]);
    }

    #[async_std::test]
    async fn on_network_registry_event_deregistered() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        db.add_to_network_registry(&SELF_CHAIN_ADDRESS, &STAKE_ADDRESS, &Snapshot::default())
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

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.network_registry,
                0u32,
                &registered_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let stored = db.get_from_network_registry(&STAKE_ADDRESS).await.unwrap();

        assert_eq!(stored, vec![]);
    }

    #[async_std::test]
    async fn on_network_registry_event_deregistered_by_manager() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        db.add_to_network_registry(&SELF_CHAIN_ADDRESS, &STAKE_ADDRESS, &Snapshot::default())
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

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.network_registry,
                0u32,
                &registered_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let stored = db.get_from_network_registry(&STAKE_ADDRESS).await.unwrap();

        assert_eq!(stored, vec![]);
    }

    #[async_std::test]
    async fn on_network_registry_event_enabled() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let nr_enabled = RawLog {
            topics: vec![
                NetworkRegistryStatusUpdatedFilter::signature(),
                H256::from_low_u64_be(1),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.network_registry,
                0u32,
                &nr_enabled,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert!(db.is_network_registry_enabled().await.unwrap());
    }

    #[async_std::test]
    async fn on_network_registry_event_disabled() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        db.set_network_registry(true, &Snapshot::default()).await.unwrap();

        let nr_disabled = RawLog {
            topics: vec![
                NetworkRegistryStatusUpdatedFilter::signature(),
                H256::from_low_u64_be(0),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.network_registry,
                0u32,
                &nr_disabled,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert!(!db.is_network_registry_enabled().await.unwrap());
    }

    #[async_std::test]
    async fn on_network_registry_set_eligible() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

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
                &mut db,
                &handlers.addresses.network_registry,
                0u32,
                &set_eligible,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert!(db.is_eligible(&STAKE_ADDRESS).await.unwrap());
    }

    #[async_std::test]
    async fn on_network_registry_set_not_eligible() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        db.set_eligible(&STAKE_ADDRESS, false, &Snapshot::default())
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
                &mut db,
                &handlers.addresses.network_registry,
                0u32,
                &set_eligible,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert!(!db.is_eligible(&STAKE_ADDRESS).await.unwrap());
    }

    #[async_std::test]
    async fn on_channel_event_balance_increased() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let channel_id = generate_channel_id(&SELF_CHAIN_ADDRESS, &COUNTERPARTY_CHAIN_ADDRESS);

        db.update_channel_and_snapshot(
            &channel_id,
            &ChannelEntry::new(
                *SELF_CHAIN_ADDRESS,
                *COUNTERPARTY_CHAIN_ADDRESS,
                Balance::new(U256::zero(), BalanceType::HOPR),
                U256::zero(),
                ChannelStatus::Open,
                U256::one(),
                U256::zero(),
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
                &mut db,
                &handlers.addresses.channels,
                0u32,
                &balance_increased_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            *db.get_channel(&channel_id).await.unwrap().unwrap().balance.value(),
            solidity_balance
        );
    }

    #[async_std::test]
    async fn on_channel_event_domain_separator_updated() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let separator = Hash::default();

        let log = RawLog {
            topics: vec![
                DomainSeparatorUpdatedFilter::signature(),
                H256::from_slice(&separator.to_bytes()),
            ],
            data: encode(&[]),
        };

        assert!(db.get_channels_domain_separator().await.unwrap().is_none());

        handlers
            .on_event(&mut db, &handlers.addresses.channels, 0u32, &log, &Snapshot::default())
            .await
            .unwrap();

        assert_eq!(db.get_channels_domain_separator().await.unwrap().unwrap(), separator);
    }

    #[async_std::test]
    async fn on_channel_event_balance_decreased() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let channel_id = generate_channel_id(&SELF_CHAIN_ADDRESS, &COUNTERPARTY_CHAIN_ADDRESS);

        db.update_channel_and_snapshot(
            &channel_id,
            &ChannelEntry::new(
                *SELF_CHAIN_ADDRESS,
                *COUNTERPARTY_CHAIN_ADDRESS,
                Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
                U256::zero(),
                ChannelStatus::Open,
                U256::one(),
                U256::zero(),
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
                &mut db,
                &handlers.addresses.channels,
                0u32,
                &balance_increased_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            *db.get_channel(&channel_id).await.unwrap().unwrap().balance.value(),
            U256::zero()
        );
    }

    #[async_std::test]
    async fn on_channel_closed() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let channel_id = generate_channel_id(&SELF_CHAIN_ADDRESS, &COUNTERPARTY_CHAIN_ADDRESS);

        db.update_channel_and_snapshot(
            &channel_id,
            &ChannelEntry::new(
                *SELF_CHAIN_ADDRESS,
                *COUNTERPARTY_CHAIN_ADDRESS,
                Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
                U256::zero(),
                ChannelStatus::Open,
                U256::one(),
                U256::zero(),
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
                &mut db,
                &handlers.addresses.channels,
                0u32,
                &channel_closed_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            db.get_channel(&channel_id).await.unwrap().unwrap().status,
            ChannelStatus::Closed
        );
    }

    #[async_std::test]
    async fn on_channel_opened() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

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
                &mut db,
                &handlers.addresses.channels,
                0u32,
                &channel_opened_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let channel = db.get_channel(&channel_id).await.unwrap().unwrap();

        assert_eq!(channel.status, ChannelStatus::Open);
    }

    #[async_std::test]
    async fn on_channel_ticket_redeemed() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let channel_id = generate_channel_id(&SELF_CHAIN_ADDRESS, &COUNTERPARTY_CHAIN_ADDRESS);

        db.update_channel_and_snapshot(
            &channel_id,
            &ChannelEntry::new(
                *SELF_CHAIN_ADDRESS,
                *COUNTERPARTY_CHAIN_ADDRESS,
                Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
                U256::zero(),
                ChannelStatus::Open,
                U256::one(),
                U256::zero(),
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
                &mut db,
                &handlers.addresses.channels,
                0u32,
                &ticket_redeemed_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let channel = db.get_channel(&channel_id).await.unwrap().unwrap();

        assert_eq!(channel.ticket_index, ticket_index);
    }

    #[async_std::test]
    async fn on_channel_closure_initiated() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let channel_id = generate_channel_id(&SELF_CHAIN_ADDRESS, &COUNTERPARTY_CHAIN_ADDRESS);

        db.update_channel_and_snapshot(
            &channel_id,
            &ChannelEntry::new(
                *SELF_CHAIN_ADDRESS,
                *COUNTERPARTY_CHAIN_ADDRESS,
                Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
                U256::zero(),
                ChannelStatus::Open,
                U256::one(),
                U256::zero(),
            ),
            &Snapshot::default(),
        )
        .await
        .unwrap();

        let timestamp = U256::from((1u64 << 32) - 1);

        let closure_initiated_log = RawLog {
            topics: vec![
                OutgoingChannelClosureInitiatedFilter::signature(),
                H256::from_slice(&channel_id.to_bytes()),
            ],
            data: Vec::from(timestamp.to_bytes()),
        };

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.channels,
                0u32,
                &closure_initiated_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let channel = db.get_channel(&channel_id).await.unwrap().unwrap();

        assert_eq!(channel.status, ChannelStatus::PendingToClose);
        assert_eq!(channel.closure_time, timestamp);
    }

    #[async_std::test]
    async fn on_node_safe_registry_registered() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

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
                &mut db,
                &handlers.addresses.node_safe_registry,
                0u32,
                &safe_registered_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(db.is_mfa_protected().await.unwrap(), Some(*SAFE_INSTANCE_ADDR));
    }

    #[async_std::test]
    async fn on_node_safe_registry_deregistered() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        db.set_mfa_protected_and_update_snapshot(Some(*SAFE_INSTANCE_ADDR), &Snapshot::default())
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
                &mut db,
                &handlers.addresses.node_safe_registry,
                0u32,
                &safe_registered_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(db.is_mfa_protected().await.unwrap(), None);
    }

    #[async_std::test]
    async fn ticket_price_update() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let log = RawLog {
            topics: vec![TicketPriceUpdatedFilter::signature()],
            data: encode(&[Token::Uint(EthU256::from(1u64)), Token::Uint(EthU256::from(123u64))]),
        };

        assert_eq!(db.get_ticket_price().await.unwrap(), None);

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.ticket_price_oracle,
                0u32,
                &log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(db.get_ticket_price().await.unwrap(), Some(U256::from(123u64)));
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use core_ethereum_db::db::wasm::Database;
    use core_types::{account::AccountEntry, channels::ChannelEntry};
    use ethers::{core::abi::RawLog, types::H256};
    use hex::decode;
    use js_sys::{Array, JsString};
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;
    use utils_log::{debug, error};
    use utils_misc::{ok_or_jserr, utils::wasm::JsResult};
    use utils_types::primitives::{Address, Snapshot};
    use wasm_bindgen::{prelude::*, JsValue};
    use wasm_bindgen_futures;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen]
        pub type IndexerCallbacks;

        #[wasm_bindgen(method, js_name = "ownChannelUpdated")]
        pub fn js_own_channel_updated(this: &IndexerCallbacks, channel_entry: ChannelEntry);

        #[wasm_bindgen(method, js_name = "newAnnouncement")]
        pub fn js_new_announcement(this: &IndexerCallbacks, account_entry: AccountEntry);

        #[wasm_bindgen(method, js_name = "nodeAllowedToAccessNetwork")]
        pub fn js_node_allowed_to_access_network(this: &IndexerCallbacks, address: Address);

        #[wasm_bindgen(method, js_name = "nodeNotAllowedToAccessNetwork")]
        pub fn js_node_not_allowed_to_access_network(this: &IndexerCallbacks, address: Address);

    }

    impl super::IndexerCallbacks for IndexerCallbacks {
        fn new_announcement(&self, account_entry: &AccountEntry) {
            self.js_new_announcement(account_entry.clone())
        }

        fn own_channel_updated(&self, channel_entry: &ChannelEntry) {
            self.js_own_channel_updated(*channel_entry)
        }

        fn node_allowed_to_access_network(&self, address: &Address) {
            self.js_node_allowed_to_access_network(*address)
        }

        fn node_not_allowed_to_access_network(&self, address: &Address) {
            self.js_node_not_allowed_to_access_network(*address)
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct ContractAddresses {
        pub channels: String,
        pub token: String,
        pub network_registry: String,
        pub announcements: String,
        pub node_safe_registry: String,
        pub node_management_module: String,
        pub ticket_price_oracle: String,
    }

    impl From<&super::ContractAddresses> for ContractAddresses {
        fn from(x: &crate::handlers::ContractAddresses) -> Self {
            ContractAddresses {
                channels: x.channels.to_string(),
                token: x.token.to_string(),
                network_registry: x.network_registry.to_string(),
                announcements: x.announcements.to_string(),
                node_safe_registry: x.node_safe_registry.to_string(),
                node_management_module: x.node_management_module.to_string(),
                ticket_price_oracle: x.ticket_price_oracle.to_string(),
            }
        }
    }

    #[wasm_bindgen]
    pub struct Handlers {
        w: super::ContractEventHandlers<IndexerCallbacks>,
    }

    #[wasm_bindgen]
    impl Handlers {
        #[wasm_bindgen]
        pub fn get_token_topics(&self) -> Vec<JsString> {
            self.w
                .get_token_topics()
                .iter()
                .map(|t| JsString::from(format!("0x{}", hex::encode(t.0))))
                .collect::<Vec<_>>()
        }

        #[wasm_bindgen]
        pub fn get_announcement_topics(&self) -> Vec<JsString> {
            self.w
                .get_announcement_topics()
                .iter()
                .map(|t| JsString::from(format!("0x{}", hex::encode(t.0))))
                .collect::<Vec<_>>()
        }

        #[wasm_bindgen]
        pub fn get_channel_topics(&self) -> Vec<JsString> {
            self.w
                .get_channel_topics()
                .iter()
                .map(|t| JsString::from(format!("0x{}", hex::encode(t.0))))
                .collect::<Vec<_>>()
        }

        #[wasm_bindgen]
        pub fn get_network_registry_topics(&self) -> Vec<JsString> {
            self.w
                .get_network_registry_topics()
                .iter()
                .map(|t| JsString::from(format!("0x{}", hex::encode(t.0))))
                .collect::<Vec<_>>()
        }

        #[wasm_bindgen]
        pub fn get_node_safe_registry_topics(&self) -> Vec<JsString> {
            self.w
                .get_node_safe_registry_topics()
                .iter()
                .map(|t| JsString::from(format!("0x{}", hex::encode(t.0))))
                .collect::<Vec<_>>()
        }

        #[wasm_bindgen]
        pub fn get_ticket_price_oracle_topics(&self) -> Vec<JsString> {
            self.w
                .get_ticket_price_oracle_topics()
                .iter()
                .map(|t| JsString::from(format!("0x{}", hex::encode(t.0))))
                .collect::<Vec<_>>()
        }

        #[wasm_bindgen]
        pub fn init(
            address_to_monitor: &str,
            chain_key: &str,
            contract_addresses_js: JsValue,
            callbacks: IndexerCallbacks,
        ) -> Handlers {
            let contract_addresses =
                serde_wasm_bindgen::from_value::<ContractAddresses>(contract_addresses_js).unwrap();
            Self {
                w: super::ContractEventHandlers {
                    address_to_monitor: Address::from_str(address_to_monitor).unwrap(),
                    chain_key: Address::from_str(chain_key).unwrap(),
                    addresses: (&contract_addresses).into(),
                    cbs: callbacks,
                },
            }
        }

        #[wasm_bindgen]
        pub async fn on_event(
            &self,
            db: &Database,
            address: &str,
            topics: Array,
            data: &str,
            block_number: &str,
            snapshot: &Snapshot,
        ) -> JsResult<()> {
            let contract_address = Address::from_str(address).unwrap();
            let u32_block_number = u32::from_str(block_number).unwrap();

            let decoded_data = ok_or_jserr!(decode(data))?;

            let mut decoded_topics: Vec<H256> = vec![];

            for topic in topics.iter() {
                let decoded_topic = ok_or_jserr!(decode(String::from(JsString::from(topic))))?;
                decoded_topics.push(H256::from_slice(&decoded_topic));
            }

           let r = {
                debug!(">>> WRITE on_event");
                let val = db.as_ref_counted();
                let mut g = val.write().await;

                self.w
                    .on_event(
                        &mut *g,
                        &contract_address,
                        u32_block_number,
                        &RawLog {
                            topics: decoded_topics,
                            data: decoded_data,
                        },
                        snapshot,
                    )
                    .await
                    .map_err(|e| {
                        error!("on_event error - {:?}", e.to_string());
                        JsValue::from(e.to_string())
                    })
            };
            debug!("<<< WRITE on_event");
            r
        }
    }
}
