use async_trait::async_trait;
use core_crypto::types::Hash;
use core_ethereum_actions::payload::PayloadGenerator;
use core_ethereum_actions::transaction_queue::{TransactionExecutor, TransactionResult};
use core_ethereum_rpc::HoprRpcOperations;
use core_ethereum_types::TypedTransaction;
use core_types::acknowledgement::AcknowledgedTicket;
use core_types::announcement::AnnouncementData;
use std::marker::PhantomData;
use utils_types::primitives::{Address, Balance};

use crate::errors::Result;

/// Represents an abstract client that is capable of submitting
/// an Ethereum transaction-like object to the blockchain.
#[async_trait(? Send)]
pub trait EthereumClient<T: Into<TypedTransaction>> {
    /// Sends transaction to the blockchain and returns its hash.
    /// Does not poll for transaction completion.
    async fn post_transaction(&self, tx: T) -> Result<Hash>;

    /// Sends transaction to the blockchain and awaits the required number
    /// of confirmations by polling the underlying provider periodically.
    /// Then returns the TX hash.
    async fn post_transaction_and_await_confirmation(&self, tx: T) -> Result<Hash>;
}

/// Instantiation of `EthereumClient` using `HoprRpcOperations`.
#[derive(Clone)]
pub struct RpcEthereumClient<Rpc: HoprRpcOperations> {
    rpc: Rpc,
}

impl<Rpc: HoprRpcOperations> RpcEthereumClient<Rpc> {
    pub fn new(rpc: Rpc) -> Self {
        Self { rpc }
    }
}

#[async_trait(? Send)]
impl<Rpc: HoprRpcOperations> EthereumClient<TypedTransaction> for RpcEthereumClient<Rpc> {
    async fn post_transaction(&self, tx: TypedTransaction) -> Result<Hash> {
        let res = self.rpc.send_transaction(tx).await?.tx_hash();
        Ok(res)
    }

    async fn post_transaction_and_await_confirmation(&self, tx: TypedTransaction) -> Result<Hash> {
        let pending_tx = self.rpc.send_transaction(tx).await?;
        let hash = pending_tx.tx_hash();
        pending_tx.await?;
        Ok(hash)
    }
}

/// Implementation of `TransactionExecutor` using the given `EthereumClient` and corresponding
/// `PayloadGenerator`.
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

#[async_trait(? Send)]
impl<T, C, PGen> TransactionExecutor for EthereumTransactionExecutor<T, C, PGen>
where
    T: Into<TypedTransaction>,
    C: EthereumClient<T> + Clone,
    PGen: PayloadGenerator<T> + Clone,
{
    async fn redeem_ticket(&self, acked_ticket: AcknowledgedTicket) -> TransactionResult {
        match self.payload_generator.redeem_ticket(acked_ticket) {
            Ok(tx) => match self.client.post_transaction(tx).await {
                Ok(tx_hash) => TransactionResult::TicketRedeemed { tx_hash },
                Err(e) => TransactionResult::Failure(e.to_string()),
            },
            Err(e) => e.into(),
        }
    }

    async fn fund_channel(&self, destination: Address, balance: Balance) -> TransactionResult {
        match self.payload_generator.fund_channel(destination, balance) {
            Ok(tx) => match self.client.post_transaction(tx).await {
                Ok(tx_hash) => TransactionResult::ChannelFunded { tx_hash },
                Err(e) => TransactionResult::Failure(e.to_string()),
            },
            Err(e) => e.into(),
        }
    }

    async fn initiate_outgoing_channel_closure(&self, dst: Address) -> TransactionResult {
        match self.payload_generator.initiate_outgoing_channel_closure(dst) {
            Ok(tx) => match self.client.post_transaction(tx).await {
                Ok(tx_hash) => TransactionResult::ChannelClosureInitiated { tx_hash },
                Err(e) => TransactionResult::Failure(e.to_string()),
            },
            Err(e) => e.into(),
        }
    }

    async fn finalize_outgoing_channel_closure(&self, dst: Address) -> TransactionResult {
        match self.payload_generator.finalize_outgoing_channel_closure(dst) {
            Ok(tx) => match self.client.post_transaction(tx).await {
                Ok(tx_hash) => TransactionResult::ChannelClosed { tx_hash },
                Err(e) => TransactionResult::Failure(e.to_string()),
            },
            Err(e) => e.into(),
        }
    }

    async fn close_incoming_channel(&self, src: Address) -> TransactionResult {
        match self.payload_generator.close_incoming_channel(src) {
            Ok(tx) => match self.client.post_transaction(tx).await {
                Ok(tx_hash) => TransactionResult::ChannelClosed { tx_hash },
                Err(e) => TransactionResult::Failure(e.to_string()),
            },
            Err(e) => e.into(),
        }
    }

    async fn withdraw(&self, recipient: Address, amount: Balance) -> TransactionResult {
        match self.payload_generator.transfer(recipient, amount) {
            // Withdraw transaction is out-of-band Indexer, so its confirmation
            // is awaited via polling.
            Ok(tx) => match self.client.post_transaction_and_await_confirmation(tx).await {
                Ok(tx_hash) => TransactionResult::Withdrawn { tx_hash },
                Err(e) => TransactionResult::Failure(e.to_string()),
            },
            Err(e) => e.into(),
        }
    }

    async fn announce(&self, data: AnnouncementData) -> TransactionResult {
        match self.payload_generator.announce(data) {
            Ok(tx) => match self.client.post_transaction(tx).await {
                Ok(tx_hash) => TransactionResult::Announced { tx_hash },
                Err(e) => TransactionResult::Failure(e.to_string()),
            },
            Err(e) => e.into(),
        }
    }

    async fn register_safe(&self, safe_address: Address) -> TransactionResult {
        match self.payload_generator.register_safe_by_node(safe_address) {
            Ok(tx) => match self.client.post_transaction(tx).await {
                Ok(tx_hash) => TransactionResult::SafeRegistered { tx_hash },
                Err(e) => TransactionResult::Failure(e.to_string()),
            },
            Err(e) => e.into(),
        }
    }
}
