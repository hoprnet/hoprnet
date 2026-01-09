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
#[derive(Debug, strum::EnumIs, strum::EnumTryAs, thiserror::Error)]
pub enum AnnouncementError<E> {
    /// Special error when an account is already announced.
    #[error("already announced")]
    AlreadyAnnounced,
    /// Error that can occur when processing an announcement.
    #[error("account announcement error: {0}")]
    ProcessingError(E),
}

impl<E> AnnouncementError<E> {
    /// Constructs a [`AnnouncementError::ProcessingError`].
    pub fn processing<F: Into<E>>(error: F) -> Self {
        Self::ProcessingError(error.into())
    }
}

/// Error that can occur when registering a node with a Safe.
#[derive(Debug, strum::EnumIs, strum::EnumTryAs, thiserror::Error)]
pub enum SafeRegistrationError<E> {
    /// Special error when a Safe is already registered.
    #[error("node is already registered with safe {0}")]
    AlreadyRegistered(Address),
    /// Error that can occur when processing a Safe registration.
    #[error("safe registration error: {0}")]
    ProcessingError(E),
}

impl<E> SafeRegistrationError<E> {
    /// Constructs a [`SafeRegistrationError::ProcessingError`].
    pub fn processing<F: Into<E>>(error: F) -> Self {
        Self::ProcessingError(error.into())
    }
}

/// On-chain write operations regarding on-chain node accounts.
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait ChainWriteAccountOperations {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Announces transport key and list of multi addresses.
    async fn announce(
        &self,
        multiaddrs: &[Multiaddr],
        key: &OffchainKeypair,
    ) -> Result<BoxFuture<'life0, Result<ChainReceipt, Self::Error>>, AnnouncementError<Self::Error>>;

    /// Withdraws native or token currency from the Safe or node account (depends on the used [`PayloadGenerator`]).
    async fn withdraw<C: Currency + Send>(
        &self,
        balance: Balance<C>,
        recipient: &Address,
    ) -> Result<BoxFuture<'life0, Result<ChainReceipt, Self::Error>>, Self::Error>;

    /// Registers Safe address with the current node.
    async fn register_safe(
        &self,
        safe_address: &Address,
    ) -> Result<BoxFuture<'life0, Result<ChainReceipt, Self::Error>>, SafeRegistrationError<Self::Error>>;
}

/// Selector for on-chain node accounts.
///
/// See [`ChainReadAccountOperations::stream_accounts`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct AccountSelector {
    /// Selects accounts that announced with publicly routable multi-addresses.
    pub public_only: bool,
    /// Selects accounts bound with the given chain key.
    pub chain_key: Option<Address>,
    /// Selects accounts bound with the given off-chain key.
    pub offchain_key: Option<OffchainPublicKey>,
}

impl AccountSelector {
    /// Selects only accounts that announced with publicly routable multi-addresses.
    #[must_use]
    pub fn with_public_only(mut self, public_only: bool) -> Self {
        self.public_only = public_only;
        self
    }

    /// Selects accounts bound with the given chain key.
    #[must_use]
    pub fn with_chain_key(mut self, chain_key: Address) -> Self {
        self.chain_key = Some(chain_key);
        self
    }

    /// Selects accounts bound with the given off-chain key.
    #[must_use]
    pub fn with_offchain_key(mut self, offchain_key: OffchainPublicKey) -> Self {
        self.offchain_key = Some(offchain_key);
        self
    }

    /// Checks if the given [`account`](AccountEntry) satisfies the selector.
    pub fn satisfies(&self, account: &AccountEntry) -> bool {
        if self.public_only && !account.has_announced_with_routing_info() {
            return false;
        }

        if let Some(chain_key) = &self.chain_key {
            if &account.chain_addr != chain_key {
                return false;
            }
        }

        if let Some(packet_key) = &self.offchain_key {
            if &account.public_key != packet_key {
                return false;
            }
        }

        true
    }
}

/// Chain operations that read on-chain node accounts.
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait ChainReadAccountOperations {
    type Error: std::error::Error + Send + Sync + 'static;

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

    /// Waits for the account with the key-binding on the `offchain_key` to appear on-chain or the `timeout` expires.
    async fn await_key_binding(
        &self,
        offchain_key: &OffchainPublicKey,
        timeout: std::time::Duration,
    ) -> Result<AccountEntry, Self::Error>;
}
