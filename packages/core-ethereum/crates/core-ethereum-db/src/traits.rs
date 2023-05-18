use async_trait::async_trait;

use core_crypto::iterated_hash::Intermediate;
use core_crypto::{iterated_hash::IteratedHash, types::{HalfKeyChallenge, Hash, PublicKey}};
use core_types::acknowledgement::{AcknowledgedTicket};
use core_types::{channels::{ChannelEntry, Ticket}, account::AccountEntry};
use utils_types::primitives::{Address, Balance, Snapshot};
use utils_db::{db::DB, traits::BinaryAsyncKVStorage};

use crate::errors::Result;

#[async_trait(? Send)] // not placing the `Send` trait limitations on the trait
pub trait HoprCoreEthereumDbActions {
    async fn get_acknowledged_tickets(&self, filter: ChannelEntry) -> Result<Vec<AcknowledgedTicket>>;

    async fn delete_acknowledged_tickets_from(&mut self, source: ChannelEntry) -> Result<()>;

    async fn delete_acknowledged_ticket(&mut self, ticket: AcknowledgedTicket) -> Result<()>;

    async fn store_hash_intermediaries(&mut self, channel: Hash, intermediates: IteratedHash) -> Result<()>;

    async fn get_commitment(&self, channel: Hash, iteration: usize) -> Result<Option<Hash>>;

    async fn get_current_commitment(&self, channel: Hash) -> Result<Option<Hash>>;

    async fn set_current_commitment(&mut self, channel: Hash, commitment: Hash) -> Result<()>;

    async fn get_latest_block_number(&self) -> Result<u32>;

    async fn update_latest_block_number(&mut self, number: u32) -> Result<()>;

    async fn get_latest_confirmed_snapshot(&self) -> Result<Option<Snapshot>>;

    async fn get_channel(&self, channel: &Hash) -> Result<Option<ChannelEntry>>;

    async fn get_channels(&self, filter: ChannelEntry) -> Result<Vec<AcknowledgedTicket>>;

    async fn update_channel_and_snapshot(&mut self, channel_id: &Hash, channel: ChannelEntry, snapshot: Snapshot) -> Result<()>;

    async fn get_account(&self, address: &Address) -> Result<Option<AccountEntry>>;

    async fn update_account_and_snapshot(&mut self, account: AccountEntry, snapshot: Snapshot) -> Result<()>;

    async fn get_accounts(&self, address: Address) -> Result<Vec<AccountEntry>>;

    // getAccountsIterable

    async fn get_redeemed_tickets_value(&self) -> Result<Balance>;

    async fn get_redeemed_tickets_count(&self) -> Result<usize>;

    async fn get_neglected_tickets_count(&self) -> Result<usize>;

    async fn get_pending_tickets_count(&self) -> Result<usize>;

    async fn get_losing_tickets_count(&self) -> Result<usize>;

    async fn get_pendings_balance_to(&self, counterparty: &Address) -> Result<Balance>;

    async fn mark_pending(&mut self, ticket: &Ticket) -> Result<()>;

    async fn resolve_pending(&mut self, ticket: &Ticket, snapshot: Snapshot) -> Result<()>;

    async fn mark_redeemeed(&mut self, ticket: &AcknowledgedTicket) -> Result<()>;

    async fn mark_losing_acked_ticket(&mut self, ticket: &AcknowledgedTicket) -> Result<()>;

    async fn get_rejected_tickets_value(&self) -> Result<Balance>;

    async fn get_rejected_tickets_count(&self) -> Result<usize>;

    async fn get_channel_x(&self, src: &PublicKey, dest: &PublicKey) -> Result<Option<ChannelEntry>>;

    async fn get_channel_to(&self, dest: &PublicKey) -> Result<Option<ChannelEntry>>;

    async fn get_channel_from(&self, src: &PublicKey) -> Result<Option<ChannelEntry>>;

    async fn get_channels_from(&self, address: Address) -> Result<Vec<ChannelEntry>>;

    async fn get_channels_to(&self, address: Address) -> Result<Vec<ChannelEntry>>;

    async fn get_hopr_balance(&self, src: &PublicKey) -> Result<Balance>;

    async fn set_hopr_balance(&mut self, balance: &Balance) -> Result<()>;

    async fn add_hopr_balance(&mut self, balance: Balance, snapshot: Snapshot) -> Result<()>;

    async fn sub_hopr_balance(&mut self, balance: Balance, snapshot: Snapshot) -> Result<()>;

    async fn is_network_registry_enabled(&self, snapshot: Snapshot) -> Result<bool>;

    async fn set_network_registry(&mut self, enabled: bool, snapshot: Snapshot) -> Result<()>;

    async fn add_to_network_registry(&mut self, public_key: &PublicKey, account: Address, snapshot: Snapshot) -> Result<()>;

    async fn remove_from_network_registry(&mut self, public_key: &PublicKey, account: Address, snapshot: Snapshot) -> Result<()>;

    async fn get_account_from_network_registry(&self, public_key: &PublicKey) -> Result<Option<Address>>;

    async fn find_hopr_node_using_account_in_network_registry(&self, account: Address) -> Result<Vec<PublicKey>>;

    async fn is_eligible(&self, account: &Address) -> Result<bool>;

    async fn set_eligible(&mut self, account: &Address, eligible: bool, snapshot: Snapshot) -> Result<()>;
}
