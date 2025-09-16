use std::future::Future;

use futures::stream::BoxStream;
use hopr_crypto_types::prelude::OffchainPublicKey;
pub use hopr_internal_types::prelude::AccountEntry;
use hopr_primitive_types::prelude::Address;
pub use hopr_primitive_types::prelude::{Balance, Currency};
pub use multiaddr::Multiaddr;

use crate::chain::ChainReceipt;

/// On-chain write operations regarding on-chain node accounts.
#[async_trait::async_trait]
pub trait ChainWriteAccountOperations {
    type Error;

    /// Announces transport key and list of multi addresses.
    async fn announce(
        &self,
        multiaddrs: &[Multiaddr],
        key: &OffchainPublicKey,
    ) -> Result<impl Future<Output = Result<ChainReceipt, Self::Error>> + Send + '_, Self::Error>;

    /// Withdraws native or token currency.
    async fn withdraw<C: Currency>(
        &self,
        balance: Balance<C>,
    ) -> Result<impl Future<Output = Result<ChainReceipt, Self::Error>> + Send + '_, Self::Error>;

    /// Registers Safe address with the current node.
    async fn register_safe(
        &self,
        safe_address: Address,
    ) -> Result<impl Future<Output = Result<ChainReceipt, Self::Error>> + Send + '_, Self::Error>;
}

/// Selector for on-chain node accounts.
///
/// See [`ChainReadAccountOperations::stream_accounts`].
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AccountSelector {
    pub public_only: bool,
}

/// Chain operations that read on-chain node accounts.
#[async_trait::async_trait]
pub trait ChainReadAccountOperations {
    type Error;

    /// Returns on-chain node accounts with the given [`AccountSelector`].
    async fn stream_accounts<'a>(
        &'a self,
        selector: AccountSelector,
    ) -> Result<BoxStream<'a, Result<AccountEntry, Self::Error>>, Self::Error>;
}
