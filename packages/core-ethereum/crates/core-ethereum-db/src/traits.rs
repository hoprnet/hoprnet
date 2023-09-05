use crate::errors::Result;
use async_trait::async_trait;

use core_crypto::types::OffchainPublicKey;
use core_crypto::types::{HalfKeyChallenge, Hash};
use core_types::{
    account::AccountEntry,
    acknowledgement::{AcknowledgedTicket, PendingAcknowledgement, UnacknowledgedTicket},
    channels::{ChannelEntry, Ticket},
};
use utils_types::primitives::{Address, AuthorizationToken, Balance, Snapshot, U256};

pub trait HoprCoreEthereumDbActions {
    // core only part
    fn get_current_ticket_index(&self, channel_id: &Hash) -> Result<Option<U256>>;
    fn set_current_ticket_index(&mut self, channel_id: &Hash, index: U256) -> Result<()>;

    fn get_tickets(&self, signer: Option<Address>) -> Result<Vec<Ticket>>;

    fn mark_rejected(&mut self, ticket: &Ticket) -> Result<()>;

    fn get_pending_acknowledgement(
        &self,
        half_key_challenge: &HalfKeyChallenge,
    ) -> Result<Option<PendingAcknowledgement>>;

    fn store_pending_acknowledgment(
        &mut self,
        half_key_challenge: HalfKeyChallenge,
        pending_acknowledgment: PendingAcknowledgement,
    ) -> Result<()>;

    fn replace_unack_with_ack(
        &mut self,
        half_key_challenge: &HalfKeyChallenge,
        ack_ticket: AcknowledgedTicket,
    ) -> Result<()>;

    // core and core-ethereum part
    /// Get all acknowledged tickets within the filter criteria.
    fn get_acknowledged_tickets(&self, filter: Option<ChannelEntry>) -> Result<Vec<AcknowledgedTicket>>;

    /// Get all unacknowledged tickets within the filter criteria.
    fn get_unacknowledged_tickets(&self, filter: Option<ChannelEntry>) -> Result<Vec<UnacknowledgedTicket>>;

    /// Mark the ticket as pending.
    fn mark_pending(&mut self, counterparty: &Address, ticket: &Ticket) -> Result<()>;

    /// Get pending balance to a counter party's address.
    fn get_pending_balance_to(&self, counterparty: &Address) -> Result<Balance>;

    fn get_packet_key(&self, chain_key: &Address) -> Result<Option<OffchainPublicKey>>;

    fn get_chain_key(&self, packet_key: &OffchainPublicKey) -> Result<Option<Address>>;

    fn link_chain_and_packet_keys(
        &mut self,
        chain_key: &Address,
        packet_key: &OffchainPublicKey,
        snapshot: &Snapshot,
    ) -> Result<()>;

    fn get_channel_to(&self, dest: &Address) -> Result<Option<ChannelEntry>>;

    /// Get channel from peer with Ethereum address.
    fn get_channel_from(&self, src: &Address) -> Result<Option<ChannelEntry>>;

    /// Update channel information.
    fn update_channel_and_snapshot(
        &mut self,
        channel_id: &Hash,
        channel: &ChannelEntry,
        snapshot: &Snapshot,
    ) -> Result<()>;

    // core-ethereum only part
    /// Delete acknowledged tickets belonging to a channel
    fn delete_acknowledged_tickets_from(&mut self, source: ChannelEntry) -> Result<()>;

    /// Get the value of the lastest block number.
    fn get_latest_block_number(&self) -> Result<u32>;

    /// Set the latest block number to this value.
    fn update_latest_block_number(&mut self, number: u32) -> Result<()>;

    /// Get the latest confirmed snapshot.
    fn get_latest_confirmed_snapshot(&self) -> Result<Option<Snapshot>>;

    /// Get channel from hash.
    fn get_channel(&self, channel: &Hash) -> Result<Option<ChannelEntry>>;

    /// TODO: unused?
    fn get_channels(&self) -> Result<Vec<ChannelEntry>>;

    /// Get all open channels.
    fn get_channels_open(&self) -> Result<Vec<ChannelEntry>>;

    /// Get account for address.
    fn get_account(&self, address: &Address) -> Result<Option<AccountEntry>>;

    /// Update the account entry.
    fn update_account_and_snapshot(&mut self, account: &AccountEntry, snapshot: &Snapshot) -> Result<()>;

    /// Get all accounts.
    fn get_accounts(&self) -> Result<Vec<AccountEntry>>;

    /// Get all public accounts.
    fn get_public_node_accounts(&self) -> Result<Vec<AccountEntry>>;

    /// Get the total value of redeemed tickets.
    fn get_redeemed_tickets_value(&self) -> Result<Balance>;

    /// Get the total number of redeemed tickets.
    fn get_redeemed_tickets_count(&self) -> Result<usize>;

    /// Get the total number of neglected tickets.
    fn get_neglected_tickets_count(&self) -> Result<usize>;

    /// Get the total number of pending tickets.
    fn get_pending_tickets_count(&self) -> Result<usize>;

    /// Get the total number of losing tickets.
    fn get_losing_tickets_count(&self) -> Result<usize>;

    /// Resolve pending tickets.
    fn resolve_pending(&mut self, ticket: &Address, balance: &Balance, snapshot: &Snapshot) -> Result<()>;

    /// Mark the ticket as redeemed.
    fn mark_redeemed(&mut self, counterparty: &Address, ticket: &AcknowledgedTicket) -> Result<()>;

    /// Mark an acknowledged ticket as losing.
    fn mark_losing_acked_ticket(&mut self, counterparty: &Address, ticket: &AcknowledgedTicket) -> Result<()>;

    /// Get the total value of all rejected tickets.
    fn get_rejected_tickets_value(&self) -> Result<Balance>;

    /// Get the count of all rejected tickets.
    fn get_rejected_tickets_count(&self) -> Result<usize>;

    /// Get channel from source to destination.
    fn get_channel_x(&self, src: &Address, dest: &Address) -> Result<Option<ChannelEntry>>;

    /// Get all channels from a given address.
    fn get_channels_from(&self, address: &Address) -> Result<Vec<ChannelEntry>>;

    /// Get all channels to a given address.
    fn get_channels_to(&self, address: &Address) -> Result<Vec<ChannelEntry>>;

    /// Get the current balance.
    fn get_hopr_balance(&self) -> Result<Balance>;

    /// Set balance as the current balance.
    fn set_hopr_balance(&mut self, balance: &Balance) -> Result<()>;

    /// Get the current ticket price.
    fn get_ticket_price(&self) -> Result<Option<U256>>;

    /// Set new ticket price
    fn set_ticket_price(&mut self, ticket_price: &U256) -> Result<()>;

    /// Get the domain separator of node-safe-registry contract
    fn get_node_safe_registry_domain_separator(&self) -> Result<Option<Hash>>;

    /// Set the domain separator of node-safe-registry contract
    fn set_node_safe_registry_domain_separator(
        &mut self,
        node_safe_registry_domain_separator: &Hash,
        snapshot: &Snapshot,
    ) -> Result<()>;

    /// Get the domain separator of channels contract
    fn get_channels_domain_separator(&self) -> Result<Option<Hash>>;

    /// Set the domain separator of channels contract
    fn set_channels_domain_separator(&mut self, channels_domain_separator: &Hash, snapshot: &Snapshot) -> Result<()>;

    /// Get the ledger domain separator of channels contract
    fn get_channels_ledger_domain_separator(&self) -> Result<Option<Hash>>;

    /// Set the ledger domain separator of channels contract
    fn set_channels_ledger_domain_separator(
        &mut self,
        channels_ledger_domain_separator: &Hash,
        snapshot: &Snapshot,
    ) -> Result<()>;

    /// Add balance to the current balance.
    fn add_hopr_balance(&mut self, balance: &Balance, snapshot: &Snapshot) -> Result<()>;

    /// Subtract balance from the current balance.
    fn sub_hopr_balance(&mut self, balance: &Balance, snapshot: &Snapshot) -> Result<()>;

    /// Get the staking safe address
    fn get_staking_safe_address(&self) -> Result<Option<Address>>;

    /// Sets the staking safe address
    fn set_staking_safe_address(&mut self, safe_address: &Address) -> Result<()>;

    /// Get the staking module address
    fn get_staking_module_address(&self) -> Result<Option<Address>>;

    /// Sets the staking module address
    fn set_staking_module_address(&mut self, module_address: &Address) -> Result<()>;

    /// Get the allowance for HoprChannels contract to transfer tokens on behalf of staking safe address
    fn get_staking_safe_allowance(&self) -> Result<Balance>;

    /// Sets the allowance for HoprChannels contract to transfer tokens on behalf of staking safe address
    fn set_staking_safe_allowance(&mut self, allowance: &Balance, snapshot: &Snapshot) -> Result<()>;

    /// Check whether the Network Registry is enabled.
    fn is_network_registry_enabled(&self) -> Result<bool>;

    /// Enable or disable network registry
    fn set_network_registry(&mut self, enabled: bool, snapshot: &Snapshot) -> Result<()>;

    /// Check whether node is allowed to participate in the network
    fn is_allowed_to_access_network(&self, node: &Address) -> Result<bool>;

    /// Enable or disable access to network
    fn set_allowed_to_access_network(&mut self, node: &Address, allowed: bool, snapshot: &Snapshot) -> Result<()>;

    fn get_from_network_registry(&self, stake_account: &Address) -> Result<Vec<Address>>;

    /// Check if address as eligible to be operating in the network.
    fn is_eligible(&self, account: &Address) -> Result<bool>;

    /// Set address as eligible to be operating in the network.
    /// returns affected node addresses
    fn set_eligible(&mut self, account: &Address, eligible: bool, snapshot: &Snapshot) -> Result<Vec<Address>>;

    /// Check if account is protected by a MFA module (e.g. Gnosis Safe)
    /// returns MFA module address
    fn is_mfa_protected(&self) -> Result<Option<Address>>;

    /// Marks this account as being protected by a MFA module (e.g. Gnosis Safe) or removes it
    /// `Some(Address)` -> MFA present
    /// `None` -> no MFA
    fn set_mfa_protected_and_update_snapshot(
        &mut self,
        maybe_mfa_address: Option<Address>,
        snapshot: &Snapshot,
    ) -> Result<()>;

    /// Stores the REST API token.
    fn store_authorization(&mut self, token: AuthorizationToken) -> Result<()>;

    /// Retrieves the REST API token given its ID.
    fn retrieve_authorization(&self, id: String) -> Result<Option<AuthorizationToken>>;

    /// Deletes the REST API token given its ID.
    fn delete_authorization(&mut self, id: String) -> Result<()>;
}
