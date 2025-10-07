use std::fmt::Formatter;

use futures::{future::BoxFuture, stream::BoxStream};
use hopr_crypto_types::prelude::{OffchainKeypair, OffchainPublicKey};
pub use hopr_internal_types::prelude::AccountEntry;
use hopr_primitive_types::prelude::Address;
pub use hopr_primitive_types::prelude::{Balance, Currency};
pub use multiaddr::Multiaddr;

use crate::chain::ChainReceipt;

/// Error that can occur when making a node announcement.
///
/// See [`ChainWriteAccountOperations::announce`]
#[derive(Debug, strum::EnumIs, strum::EnumTryAs)]
pub enum AnnouncementError<E> {
    /// Special error when an account is already announced.
    AlreadyAnnounced,
    /// Error that can occur when processing an announcement.
    ProcessingError(E),
}

impl<E: std::fmt::Display> std::fmt::Display for AnnouncementError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AnnouncementError::AlreadyAnnounced => f.write_str("already announced"),
            AnnouncementError::ProcessingError(e) => write!(f, "account processing error: {e}"),
        }
    }
}

impl<E: std::error::Error> std::error::Error for AnnouncementError<E> {}

/// On-chain write operations regarding on-chain node accounts.
#[async_trait::async_trait]
pub trait ChainWriteAccountOperations {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Announces transport key and list of multi addresses.
    async fn announce(
        &self,
        multiaddrs: &[Multiaddr],
        key: &OffchainKeypair,
    ) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, AnnouncementError<Self::Error>>;

    /// Withdraws native or token currency.
    async fn withdraw<C: Currency + Send>(
        &self,
        balance: Balance<C>,
        recipient: &Address,
    ) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error>;

    /// Registers Safe address with the current node.
    async fn register_safe(
        &self,
        safe_address: &Address,
    ) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error>;
}

/// Selector for on-chain node accounts.
///
/// See [`ChainReadAccountOperations::stream_accounts`].
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AccountSelector {
    /// Selects accounts that are announced with multi-addresses.
    pub public_only: bool,
    /// Selects accounts bound with the given chain key.
    pub chain_key: Option<Address>,
    /// Selects accounts bound with the given off-chain key.
    pub offchain_key: Option<OffchainPublicKey>,
}

/// Chain operations that read on-chain node accounts.
#[async_trait::async_trait]
pub trait ChainReadAccountOperations {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Returns the native or token currency balance of the current node's account.
    async fn node_balance<C: Currency>(&self) -> Result<Balance<C>, Self::Error>;

    /// Returns the native or token currency balance of the current node's Safe.
    async fn safe_balance<C: Currency>(&self) -> Result<Balance<C>, Self::Error>;

    /// Returns the native or token currency Safe allowance.
    async fn safe_allowance<C: Currency>(&self) -> Result<Balance<C>, Self::Error>;

    /// Returns account entry given the on-chain `address`.
    async fn find_account_by_address(&self, address: &Address) -> Result<Option<AccountEntry>, Self::Error>;

    /// Returns account entry given the off-chain `packet_key`.
    async fn find_account_by_packet_key(
        &self,
        packet_key: &OffchainPublicKey,
    ) -> Result<Option<AccountEntry>, Self::Error>;

    /// Validates the node's Safe setup.
    async fn check_node_safe_module_status(&self) -> Result<bool, Self::Error>;

    /// Checks if the given safe address can be registered with the current node.
    async fn can_register_with_safe(&self, safe_address: &Address) -> Result<bool, Self::Error>;

    /// Returns on-chain node accounts with the given [`AccountSelector`].
    async fn stream_accounts<'a>(
        &'a self,
        selector: AccountSelector,
    ) -> Result<BoxStream<'a, AccountEntry>, Self::Error>;

    /// Counts the accounts with the given [`AccountSelector`].
    ///
    /// This is potentially done more effectively than counting more elements of
    /// the stream returned by [`ChainReadAccountOperations::stream_accounts`].
    async fn count_accounts(&self, selector: AccountSelector) -> Result<usize, Self::Error>;
}
