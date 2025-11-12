use blokli_client::api::{BlokliQueryClient, BlokliTransactionClient};
use futures::{FutureExt, SinkExt, StreamExt, TryFutureExt};
use hopr_chain_types::prelude::SignableTransaction;
use hopr_crypto_types::prelude::*;

use crate::errors::{self, ConnectorError};

type TxRequest<T> = (
    T,
    futures::channel::oneshot::Sender<errors::Result<blokli_client::api::TxId>>,
);

/// Takes care of sequencing transactions, nonce management and submitting the transactions to the blockchain in-order.
pub struct TransactionSequencer<C, R> {
    sender: Option<futures::channel::mpsc::Sender<TxRequest<R>>>,
    nonce: std::sync::Arc<std::sync::atomic::AtomicU64>,
    client: std::sync::Arc<C>,
    signer: ChainKeypair,
}

impl<C, R> TransactionSequencer<C, R>
where
    C: BlokliQueryClient + BlokliTransactionClient + Send + Sync + 'static,
    R: SignableTransaction + Send + Sync + 'static,
{
    pub fn new(signer: ChainKeypair, client: std::sync::Arc<C>) -> Self {
        Self {
            sender: None,
            nonce: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1)),
            client,
            signer,
        }
    }

    pub async fn start(&mut self) -> errors::Result<()> {
        let tx_count = self
            .client
            .query_transaction_count(&self.signer.public().to_address().into())
            .await?;
        self.nonce.store(tx_count, std::sync::atomic::Ordering::SeqCst);

        let (sender, receiver) = futures::channel::mpsc::channel(1024);
        self.sender = Some(sender);

        let nonce_load = self.nonce.clone();
        let nonce_inc = self.nonce.clone();
        let client = self.client.clone();
        let signer = self.signer.clone();
        hopr_async_runtime::prelude::spawn(
            receiver
                .then(move |(tx, notifier)| {
                    let client = client.clone();
                    let signer = signer.clone();
                    let nonce = nonce_load.load(std::sync::atomic::Ordering::SeqCst);
                    async move {
                        tx.sign_and_encode_to_eip2718(nonce, None, &signer)
                            .map_err(errors::ConnectorError::from)
                            .and_then(move |tx| {
                                tracing::debug!("submitting transaction");
                                let client = client.clone();
                                async move {
                                    client
                                        .submit_and_track_transaction(&tx)
                                        .map_err(errors::ConnectorError::from)
                                        .await
                                }
                            })
                            .map(|res| (res, notifier))
                            .await
                    }
                })
                .for_each(move |(res, notifier)| {
                    // The nonce is incremented when the transaction succeeded or failed for other reasons than on-chain
                    // rejection.
                    if res.is_ok()
                        || res
                            .as_ref()
                            .is_err_and(|error| error.as_transaction_rejection_error().is_none())
                    {
                        nonce_inc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    }
                    if notifier.send(res).is_err() {
                        tracing::debug!(
                            "failed to notify transaction result - the caller may not want to await the result \
                             anymore."
                        );
                    }
                    futures::future::ready(())
                }),
        );

        Ok(())
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
        let sender = self
            .sender
            .as_ref()
            .ok_or(ConnectorError::InvalidState("transaction sender not started"))?;

        let (notifier_tx, notifier_rx) = futures::channel::oneshot::channel();

        sender
            .clone()
            .send((transaction, notifier_tx))
            .await
            .map_err(|_| ConnectorError::InvalidState("transaction queue dropped"))?;

        Ok(notifier_rx
            .map(move |result| {
                result
                    .map_err(|_| ConnectorError::InvalidState("transaction notifier dropped"))
                    .and_then(|tx_res| tx_res.map(|id| (id, timeout_until_finalized)))
            })
            .and_then(|(tx_id, timeout)| {
                self.client
                    .track_transaction(tx_id, timeout)
                    .map_err(ConnectorError::from)
            }))
    }
}

impl<C, R> Drop for TransactionSequencer<C, R> {
    fn drop(&mut self) {
        if let Some(mut sender) = self.sender.take() {
            sender.close_channel();
        }
    }
}
