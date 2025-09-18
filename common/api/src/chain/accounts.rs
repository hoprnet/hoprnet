use std::error::Error;

use futures::{future::BoxFuture, stream::BoxStream};
use hopr_crypto_types::prelude::OffchainPublicKey;
pub use hopr_internal_types::prelude::AccountEntry;
use hopr_primitive_types::prelude::Address;
pub use hopr_primitive_types::prelude::{Balance, Currency};
pub use multiaddr::Multiaddr;

use crate::chain::ChainReceipt;

/// On-chain write operations regarding on-chain node accounts.
#[async_trait::async_trait]
pub trait ChainWriteAccountOperations {
    type Error: Error + Send + Sync + 'static;

    /// Announces transport key and list of multi addresses.
    async fn announce(
        &self,
        multiaddrs: &[Multiaddr],
        key: &OffchainPublicKey,
    ) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error>;

    /// Withdraws native or token currency.
    async fn withdraw<C: Currency>(
        &self,
        balance: Balance<C>,
    ) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error>;

    /// Registers Safe address with the current node.
    async fn register_safe(
        &self,
        safe_address: Address,
    ) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error>;

    /// Checks if the given safe address can be registered with the current node.
    async fn can_register_with_safe(safe_address: Address) -> Result<bool, Self::Error>;
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
}

/// Chain operations that read on-chain node accounts.
#[async_trait::async_trait]
pub trait ChainReadAccountOperations {
    type Error: Error + Send + Sync + 'static;

    /// Returns the native or token currency balance of the current node's account.
    async fn node_balance<C: Currency>(&self) -> Result<Balance<C>, Self::Error>;

    /// Returns the native or token currency balance of the current node's Safe.
    async fn safe_balance<C: Currency>(&self) -> Result<Balance<C>, Self::Error>;

    /// Returns the native or token currency Safe allowance.
    async fn safe_allowance<C: Currency>(&self) -> Result<Balance<C>, Self::Error>;

    /// Returns on-chain node accounts with the given [`AccountSelector`].
    async fn stream_accounts<'a>(
        &'a self,
        selector: AccountSelector,
    ) -> Result<BoxStream<'a, Result<AccountEntry, Self::Error>>, Self::Error>;
}
