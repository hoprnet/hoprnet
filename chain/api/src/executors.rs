use async_trait::async_trait;
use chain_actions::action_queue::TransactionExecutor;
use chain_actions::payload::PayloadGenerator;
use chain_rpc::errors::RpcError;
use chain_rpc::{HoprRpcOperations, PendingTransaction};
use chain_types::TypedTransaction;
use futures::future::Either;
use futures::{pin_mut, FutureExt};
use hopr_crypto_types::types::Hash;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::time::Duration;

use async_std::task::sleep;

/// Represents an abstract client that is capable of submitting
/// an Ethereum transaction-like object to the blockchain.
#[async_trait]
pub trait EthereumClient<T: Into<TypedTransaction>> {
    /// Sends transaction to the blockchain and returns its hash.
    ///
    /// Does not poll for transaction completion.
    async fn post_transaction(&self, tx: T) -> chain_rpc::errors::Result<Hash>;

    /// Sends transaction to the blockchain and awaits the required number
    /// of confirmations by polling the underlying provider periodically.
    ///
    /// Returns the TX hash.
    async fn post_transaction_and_await_confirmation(&self, tx: T) -> chain_rpc::errors::Result<Hash>;
}

#[derive(Clone, Debug, PartialEq, smart_default::SmartDefault, Serialize, Deserialize)]
pub struct RpcEthereumClientConfig {
    /// Maximum time to wait for the TX to get submitted.
    ///
    /// This must be strictly greater than any timeouts in the underlying `HoprRpcOperations`
    ///
    /// Defaults to 30 seconds.
    #[default(Duration::from_secs(30))]
    pub max_tx_submission_wait: Duration,
}

/// Instantiation of `EthereumClient` using `HoprRpcOperations`.
#[derive(Clone)]
pub struct RpcEthereumClient<Rpc: HoprRpcOperations> {
    rpc: Rpc,
    cfg: RpcEthereumClientConfig,
}

impl<Rpc: HoprRpcOperations> RpcEthereumClient<Rpc> {
    pub fn new(rpc: Rpc, cfg: RpcEthereumClientConfig) -> Self {
        Self { rpc, cfg }
    }

    /// Post a transaction with a specified timeout.
    ///
    /// If the transaction yields a result before the timeout, the result value is returned.
    /// Otherwise, an [RpcError::Timeout] is returned and the transaction sending is aborted.
    async fn post_tx_with_timeout(&self, tx: TypedTransaction) -> chain_rpc::errors::Result<PendingTransaction> {
        let submit_tx = self.rpc.send_transaction(tx).fuse();
        let timeout = sleep(self.cfg.max_tx_submission_wait).fuse();
        pin_mut!(submit_tx, timeout);

        match futures::future::select(submit_tx, timeout).await {
            Either::Left((res, _)) => res,
            Either::Right(_) => Err(RpcError::Timeout),
        }
    }
}

#[async_trait]
impl<Rpc: HoprRpcOperations + Send + Sync> EthereumClient<TypedTransaction> for RpcEthereumClient<Rpc> {
    async fn post_transaction(&self, tx: TypedTransaction) -> chain_rpc::errors::Result<Hash> {
        self.post_tx_with_timeout(tx).await.map(|t| t.tx_hash())
    }

    /// Post a transaction and wait for its completion.
    ///
    /// The mechanism uses an internal timeout and retry mechanism (set to `3`)
    async fn post_transaction_and_await_confirmation(&self, tx: TypedTransaction) -> chain_rpc::errors::Result<Hash> {
        Ok(self.post_tx_with_timeout(tx).await?.await?.tx_hash)
    }
}

/// Implementation of [`TransactionExecutor`] using the given [`EthereumClient`] and corresponding
/// [`PayloadGenerator`].
#[derive(Clone, Debug)]
pub struct EthereumTransactionExecutor<T, C, PGen>
where
    T: Into<TypedTransaction>,
    C: EthereumClient<T> + Clone,
    PGen: PayloadGenerator<T> + Clone,
{
    client: C,
    payload_generator: PGen,
    _data: PhantomData<T>,
}

impl<T, C, PGen> EthereumTransactionExecutor<T, C, PGen>
where
    T: Into<TypedTransaction>,
    C: EthereumClient<T> + Clone,
    PGen: PayloadGenerator<T> + Clone,
{
    pub fn new(client: C, payload_generator: PGen) -> Self {
        Self {
            client,
            payload_generator,
            _data: PhantomData,
        }
    }
}

#[async_trait]
impl<T, C, PGen> TransactionExecutor for EthereumTransactionExecutor<T, C, PGen>
where
    T: Into<TypedTransaction> + Sync + Send,
    C: EthereumClient<T> + Clone + Sync + Send,
    PGen: PayloadGenerator<T> + Clone + Sync + Send,
{
    async fn redeem_ticket(
        &self,
        winning_ticket: ProvableWinningTicket,
        domain_separator: &Hash,
    ) -> chain_actions::errors::Result<Hash> {
        let payload = self.payload_generator.redeem_ticket(winning_ticket, domain_separator)?;
        Ok(self.client.post_transaction(payload).await?)
    }

    async fn fund_channel(&self, destination: Address, balance: Balance) -> chain_actions::errors::Result<Hash> {
        let payload = self.payload_generator.fund_channel(destination, balance)?;
        Ok(self.client.post_transaction(payload).await?)
    }

    async fn initiate_outgoing_channel_closure(&self, dst: Address) -> chain_actions::errors::Result<Hash> {
        let payload = self.payload_generator.initiate_outgoing_channel_closure(dst)?;
        Ok(self.client.post_transaction(payload).await?)
    }

    async fn finalize_outgoing_channel_closure(&self, dst: Address) -> chain_actions::errors::Result<Hash> {
        let payload = self.payload_generator.finalize_outgoing_channel_closure(dst)?;
        Ok(self.client.post_transaction(payload).await?)
    }

    async fn close_incoming_channel(&self, src: Address) -> chain_actions::errors::Result<Hash> {
        let payload = self.payload_generator.close_incoming_channel(src)?;
        Ok(self.client.post_transaction(payload).await?)
    }

    async fn withdraw(&self, recipient: Address, amount: Balance) -> chain_actions::errors::Result<Hash> {
        let payload = self.payload_generator.transfer(recipient, amount)?;

        // Withdraw transaction is out-of-band from Indexer, so its confirmation
        // is awaited via polling.
        Ok(self.client.post_transaction_and_await_confirmation(payload).await?)
    }

    async fn announce(&self, data: AnnouncementData) -> chain_actions::errors::Result<Hash> {
        let payload = self.payload_generator.announce(data)?;
        Ok(self.client.post_transaction(payload).await?)
    }

    async fn register_safe(&self, safe_address: Address) -> chain_actions::errors::Result<Hash> {
        let payload = self.payload_generator.register_safe_by_node(safe_address)?;
        Ok(self.client.post_transaction(payload).await?)
    }
}
