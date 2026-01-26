use blokli_client::api::{BlokliQueryClient, BlokliTransactionClient};
use futures::{FutureExt, SinkExt, StreamExt, TryFutureExt};
use hopr_chain_types::prelude::{GasEstimation, SignableTransaction};
use hopr_crypto_types::prelude::*;

use crate::errors::{self, ConnectorError};

type TxRequest<T> = (
    T,
    futures::channel::oneshot::Sender<errors::Result<blokli_client::api::TxId>>,
);

/// Takes care of sequencing transactions, nonce management, and submitting the transactions to the blockchain in-order.
///
/// This object is meant to be a singleton and cannot be cloned.
pub struct TransactionSequencer<C, R> {
    sender: futures::channel::mpsc::Sender<TxRequest<R>>,
    client: std::sync::Arc<C>,
}

const TX_QUEUE_CAPACITY: usize = 2048;

impl<C, R> TransactionSequencer<C, R>
where
    C: BlokliQueryClient + BlokliTransactionClient + Send + Sync + 'static,
    R: SignableTransaction + Send + Sync + 'static,
{
    pub fn new(signer: ChainKeypair, client: std::sync::Arc<C>) -> Self {
        tracing::debug!(signer = %signer.public().to_address(), "starting transaction sequencer");

        let client_clone = client.clone();
        let (sender, receiver) = futures::channel::mpsc::channel::<TxRequest<R>>(TX_QUEUE_CAPACITY);
        let chain_info_cache = std::sync::Arc::new(async_lock::OnceCell::new());
        let current_nonce = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
        let current_nonce_clone = current_nonce.clone();
        hopr_async_runtime::prelude::spawn(
            receiver
                .then(move |(tx, notifier): (R, _)| {
                    let client = client_clone.clone();
                    let signer = signer.clone();
                    let chain_info_cache = chain_info_cache.clone();
                    let current_nonce = current_nonce.clone();
                    async move {
                        // The chain ID is fetched only once and cached for the lifetime of the sequencer.
                        let chain_id = match chain_info_cache
                            .get_or_try_init(|| {
                                client
                                    .query_chain_info()
                                    .map_err(ConnectorError::from)
                                    .and_then(|ci| futures::future::ok(ci.chain_id as u64))
                            })
                            .await
                        {
                            Ok(chain_id) => {
                                tracing::debug!(chain_id, "chain id initialized");
                                *chain_id
                            }
                            Err(e) => return (Err(e), notifier),
                        };

                        // Currently use fixed gas estimation.
                        let gas_estimation = GasEstimation::default();
                        tracing::debug!(?gas_estimation, "gas estimation used for tx");

                        // We always query the transaction count for the signer and use
                        // the maximum between this value and the local counter
                        match client
                            .query_transaction_count(&signer.public().to_address().into())
                            .map_err(ConnectorError::from)
                            .await
                        {
                            Ok(tx_count) => {
                                let prev_nonce =
                                    current_nonce.fetch_max(tx_count, std::sync::atomic::Ordering::Relaxed);
                                tracing::debug!(prev_nonce, tx_count, "transaction count retrieved");
                            }
                            Err(e) => return (Err(e), notifier),
                        }

                        let nonce = current_nonce.load(std::sync::atomic::Ordering::Relaxed);
                        tracing::debug!(nonce, "nonce used for the tx");
                        tx.sign_and_encode_to_eip2718(nonce, chain_id, gas_estimation.into(), &signer)
                            .map_err(ConnectorError::from)
                            .and_then(move |tx| {
                                tracing::debug!(nonce, "submitting transaction");
                                let client = client.clone();
                                async move {
                                    client
                                        .submit_and_track_transaction(&tx)
                                        .map_err(ConnectorError::from)
                                        .await
                                }
                            })
                            .map(|res| (res, notifier))
                            .await
                    }
                })
                .for_each(move |(res, notifier)| {
                    // The nonce is incremented when the transaction succeeded or failed due to on-chain
                    // rejection.
                    if res.is_ok()
                        || res
                            .as_ref()
                            .is_err_and(|error| error.as_transaction_rejection_error().is_some())
                    {
                        let prev_nonce = current_nonce_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        tracing::debug!(prev_nonce, ?res, "nonce incremented due to tx success or rejection");
                    } else {
                        tracing::warn!(?res, "nonce not incremented due to tx failure other than rejection");
                    }

                    if notifier.send(res).is_err() {
                        tracing::debug!(
                            "failed to notify transaction result - the caller may not want to await the result \
                             anymore."
                        );
                    }
                    futures::future::ready(())
                })
                .inspect(|_| tracing::warn!("transaction sequencer queue stopped")),
        );

        Self { sender, client }
    }
}

impl<C, R> TransactionSequencer<C, R>
where
    C: BlokliTransactionClient + Send + Sync + 'static,
{
    /// Adds the transaction to the [`TransactionSequencer`] queue.
    ///
    /// The `timeout_until_finalized` is a total time until the TX is submitted and either confirmed or rejected.
    pub async fn enqueue_transaction(
        &self,
        transaction: R,
        timeout_until_finalized: std::time::Duration,
    ) -> errors::Result<impl Future<Output = errors::Result<blokli_client::api::types::Transaction>>> {
        let (notifier_tx, notifier_rx) = futures::channel::oneshot::channel();

        self.sender
            .clone()
            .send((transaction, notifier_tx))
            .await
            .map_err(|_| ConnectorError::InvalidState("transaction queue dropped"))?;

        Ok(notifier_rx
            .inspect_ok(|res| tracing::debug!(?res, "transaction tracking id received"))
            .map(move |result| {
                result
                    .map_err(|_| ConnectorError::InvalidState("transaction notifier dropped"))
                    .and_then(|tx_res| tx_res.map(|id| (id, timeout_until_finalized)))
            })
            .and_then(|(tx_id, timeout)| {
                self.client
                    .track_transaction(tx_id, timeout)
                    .map_err(ConnectorError::from)
                    .inspect(|res| tracing::debug!(?res, "transaction tracking done"))
            }))
    }
}

impl<C, R> Drop for TransactionSequencer<C, R> {
    fn drop(&mut self) {
        // Causes the internally spawned task that drives the queue to terminate
        self.sender.close_channel();
    }
}
