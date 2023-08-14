use crate::errors::Result;
use async_trait::async_trait;

use core_crypto::types::OffchainPublicKey;
use core_crypto::{
    iterated_hash::IteratedHash,
    types::{HalfKeyChallenge, Hash},
};
use core_types::{
    account::AccountEntry,
    acknowledgement::{AcknowledgedTicket, PendingAcknowledgement, UnacknowledgedTicket},
    channels::{ChannelEntry, Ticket},
};
use utils_types::primitives::{Address, AuthorizationToken, Balance, Snapshot, U256};

#[async_trait(? Send)] // not placing the `Send` trait limitations on the trait
pub trait HoprCoreEthereumDbActions {
    // core only part
    async fn get_current_ticket_index(&self, channel_id: &Hash) -> Result<Option<U256>>;
    async fn set_current_ticket_index(&mut self, channel_id: &Hash, index: U256) -> Result<()>;

    async fn get_tickets(&self, signer: Option<Address>) -> Result<Vec<Ticket>>;

    async fn mark_rejected(&mut self, ticket: &Ticket) -> Result<()>;

    async fn check_and_set_packet_tag(&mut self, tag: &[u8]) -> Result<bool>;

    async fn get_pending_acknowledgement(
        &self,
        half_key_challenge: &HalfKeyChallenge,
    ) -> Result<Option<PendingAcknowledgement>>;

    async fn store_pending_acknowledgment(
        &mut self,
        half_key_challenge: HalfKeyChallenge,
        pending_acknowledgment: PendingAcknowledgement,
    ) -> Result<()>;

    async fn replace_unack_with_ack(
        &mut self,
        half_key_challenge: &HalfKeyChallenge,
        ack_ticket: AcknowledgedTicket,
    ) -> Result<()>;

    // core and core-ethereum part
    /// Get all acknowledged tickets within the filter criteria.
    async fn get_acknowledged_tickets(&self, filter: Option<ChannelEntry>) -> Result<Vec<AcknowledgedTicket>>;

    /// Get all unacknowledged tickets within the filter criteria.
    async fn get_unacknowledged_tickets(&self, filter: Option<ChannelEntry>) -> Result<Vec<UnacknowledgedTicket>>;

    /// Mark the ticket as pending.
    async fn mark_pending(&mut self, ticket: &Ticket) -> Result<()>;

    /// Get pending balance to a counter party's address.
    async fn get_pending_balance_to(&self, counterparty: &Address) -> Result<Balance>;

    async fn get_packet_key(&self, chain_key: &Address) -> Result<Option<OffchainPublicKey>>;

    async fn get_chain_key(&self, packet_key: &OffchainPublicKey) -> Result<Option<Address>>;

    async fn link_chain_and_packet_keys(
        &mut self,
        chain_key: &Address,
        packet_key: &OffchainPublicKey,
        snapshot: &Snapshot,
    ) -> Result<()>;

    async fn get_channel_to(&self, dest: &Address) -> Result<Option<ChannelEntry>>;

    /// Get channel from peer with Ethereum address.
    async fn get_channel_from(&self, src: &Address) -> Result<Option<ChannelEntry>>;

    /// Update channel information.
    async fn update_channel_and_snapshot(
        &mut self,
        channel_id: &Hash,
        channel: &ChannelEntry,
        snapshot: &Snapshot,
    ) -> Result<()>;

    // core-ethereum only part
    /// Delete acknowledged tickets belonging to a channel
    async fn delete_acknowledged_tickets_from(&mut self, source: ChannelEntry) -> Result<()>;

    /// Store intermediary hash values.
    async fn store_hash_intermediaries(&mut self, channel: &Hash, intermediates: &IteratedHash) -> Result<()>;

    /// Get the value of the commitment for channel with iteration.
    async fn get_commitment(&self, channel: &Hash, iteration: usize) -> Result<Option<Hash>>;

    /// Get the value of the current commitment hash for a specific channel.
    async fn get_current_commitment(&self, channel: &Hash) -> Result<Option<Hash>>;

    /// Set the value of the current commitment for a specific channel.
    async fn set_current_commitment(&mut self, channel: &Hash, commitment: &Hash) -> Result<()>;

    /// Get the value of the lastest block number.
    async fn get_latest_block_number(&self) -> Result<u32>;

    /// Set the latest block number to this value.
    async fn update_latest_block_number(&mut self, number: u32) -> Result<()>;

    /// Get the latest confirmed snapshot.
    async fn get_latest_confirmed_snapshot(&self) -> Result<Option<Snapshot>>;

    /// Get channel from hash.
    async fn get_channel(&self, channel: &Hash) -> Result<Option<ChannelEntry>>;

    /// TODO: unused?
    async fn get_channels(&self) -> Result<Vec<ChannelEntry>>;

    /// Get all open channels.
    async fn get_channels_open(&self) -> Result<Vec<ChannelEntry>>;

    /// Get account for address.
    async fn get_account(&self, address: &Address) -> Result<Option<AccountEntry>>;

    /// Update the account entry.
    async fn update_account_and_snapshot(&mut self, account: &AccountEntry, snapshot: &Snapshot) -> Result<()>;

    /// Get all accounts.
    async fn get_accounts(&self) -> Result<Vec<AccountEntry>>;

    /// Get all public accounts.
    async fn get_public_node_accounts(&self) -> Result<Vec<AccountEntry>>;

    /// Get the total value of redeemed tickets.
    async fn get_redeemed_tickets_value(&self) -> Result<Balance>;

    /// Get the total number of redeemed tickets.
    async fn get_redeemed_tickets_count(&self) -> Result<usize>;

    /// Get the total number of neglected tickets.
    async fn get_neglected_tickets_count(&self) -> Result<usize>;

    /// Get the total number of pending tickets.
    async fn get_pending_tickets_count(&self) -> Result<usize>;

    /// Get the total number of losing tickets.
    async fn get_losing_tickets_count(&self) -> Result<usize>;

    /// Resolve pending tickets.
    async fn resolve_pending(&mut self, ticket: &Address, balance: &Balance, snapshot: &Snapshot) -> Result<()>;

    /// Mark the ticket as redeemed.
    async fn mark_redeemed(&mut self, ticket: &AcknowledgedTicket) -> Result<()>;

    /// Mark an acknowledged ticket as losing.
    async fn mark_losing_acked_ticket(&mut self, ticket: &AcknowledgedTicket) -> Result<()>;

    /// Get the total value of all rejected tickets.
    async fn get_rejected_tickets_value(&self) -> Result<Balance>;

    /// Get the count of all rejected tickets.
    async fn get_rejected_tickets_count(&self) -> Result<usize>;

    /// Get channel from source to destination.
    async fn get_channel_x(&self, src: &Address, dest: &Address) -> Result<Option<ChannelEntry>>;

    /// Get all channels from a given address.
    async fn get_channels_from(&self, address: Address) -> Result<Vec<ChannelEntry>>;

    /// Get all channels to a given address.
    async fn get_channels_to(&self, address: Address) -> Result<Vec<ChannelEntry>>;

    /// Get the current balance.
    async fn get_hopr_balance(&self) -> Result<Balance>;

    /// Set balance as the current balance.
    async fn set_hopr_balance(&mut self, balance: &Balance) -> Result<()>;

    /// Add balance to the current balance.
    async fn add_hopr_balance(&mut self, balance: &Balance, snapshot: &Snapshot) -> Result<()>;

    /// Subtract balance from the current balance.
    async fn sub_hopr_balance(&mut self, balance: &Balance, snapshot: &Snapshot) -> Result<()>;

    /// Get the staking safe address
    async fn get_staking_safe_address(&self) -> Result<Option<Address>>;

    /// Sets the staking safe address
    async fn set_staking_safe_address(&mut self, safe_address: &Address) -> Result<()>;
    
    /// Get the staking module address
    async fn get_staking_module_address(&self) -> Result<Option<Address>>;

    /// Sets the staking module address
    async fn set_staking_module_address(&mut self, module_address: &Address) -> Result<()>;

    /// Check whether the Network Registry is enabled.
    async fn is_network_registry_enabled(&self) -> Result<bool>;

    /// Set whether the Network Registry is enabled.
    async fn set_network_registry(&mut self, enabled: bool, snapshot: &Snapshot) -> Result<()>;

    /// Add Hopr public key to an ETH address.
    async fn add_to_network_registry(
        &mut self,
        address: &Address,
        account: &Address,
        snapshot: &Snapshot,
    ) -> Result<()>;

    /// Unlink Hopr public key to an ETH address by removing the entry.
    async fn remove_from_network_registry(
        &mut self,
        node_address: &Address,
        account: &Address,
        snapshot: &Snapshot,
    ) -> Result<()>;

    /// Get address associated with the public key.
    async fn get_account_from_network_registry(&self, public_key: &Address) -> Result<Option<Address>>;

    /// Find HOPR node based on its address.
    async fn find_hopr_node_using_account_in_network_registry(&self, account: &Address) -> Result<Vec<Address>>;

    /// Check if address as eligible to be operating in the network.
    async fn is_eligible(&self, account: &Address) -> Result<bool>;

    /// Set address as eligible to be operating in the network.
    async fn set_eligible(&mut self, account: &Address, eligible: bool, snapshot: &Snapshot) -> Result<()>;

    /// Add Hopr node ETH address to its associated safe address.
    async fn add_to_node_safe_registry(
        &mut self,
        node_address: &Address,
        safe_address: &Address,
        snapshot: &Snapshot,
    ) -> Result<()>;

    /// Unlink Hopr node ETH address and the safe address.
    async fn remove_from_node_safe_registry(
        &mut self,
        node_address: &Address,
        safe_address: &Address,
        snapshot: &Snapshot,
    ) -> Result<()>;

    /// Get safe address associated with the public key.
    async fn get_safe_from_node_safe_registry(&self, node_address: &Address) -> Result<Option<Address>>;

    /// Find HOPR node based on its associated safe address.
    async fn find_hopr_node_using_safe_in_node_safe_registry(&self, account: &Address) -> Result<Vec<Address>>;
    
    /// Stores the REST API token.
    async fn store_authorization(&mut self, token: AuthorizationToken) -> Result<()>;

    /// Retrieves the REST API token given its ID.
    async fn retrieve_authorization(&self, id: String) -> Result<Option<AuthorizationToken>>;

    /// Deletes the REST API token given its ID.
    async fn delete_authorization(&mut self, id: String) -> Result<()>;
}
