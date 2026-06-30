use std::{
    sync::atomic::AtomicU64,
    time::{Duration, Instant},
};

use blokli_client::api::{BlokliQueryClient, BlokliTransactionClient};
use futures::{FutureExt, SinkExt, StreamExt, TryFutureExt};
use hopr_api::{
    Address,
    types::{
        chain::prelude::{GasEstimation, SignableTransaction},
        crypto::prelude::*,
    },
};

use crate::{
    errors::{self, ConnectorError},
    utils::model_to_chain_info,
};

type TxRequest<T> = (
    T,
    Option<ChainKeypair>,
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

struct FixedTti {
    fixed: Address,
    tti: std::time::Duration,
}

impl FixedTti {
    #[inline]
    fn duration_for(&self, key: &Address) -> Option<Duration> {
        if key == &self.fixed {
            None // expiration cleared => never expires by time
        } else {
            Some(self.tti) // standard time-to-idle
        }
    }
}

impl moka::Expiry<Address, std::sync::Arc<AtomicU64>> for FixedTti {
    fn expire_after_create(&self, key: &Address, _: &std::sync::Arc<AtomicU64>, _: Instant) -> Option<Duration> {
        self.duration_for(key)
    }

    fn expire_after_read(
        &self,
        key: &Address,
        _: &std::sync::Arc<AtomicU64>,
        _: Instant,
        _: Option<Duration>,
        _: Instant,
    ) -> Option<Duration> {
        self.duration_for(key)
    }

    fn expire_after_update(
        &self,
        key: &Address,
        _: &std::sync::Arc<AtomicU64>,
        _: Instant,
        _: Option<Duration>,
    ) -> Option<Duration> {
        self.duration_for(key)
    }
}

impl<C, R> TransactionSequencer<C, R>
where
    C: BlokliQueryClient + BlokliTransactionClient + Send + Sync + 'static,
    R: SignableTransaction + Send + Sync + 'static,
{
    pub fn new(signer: ChainKeypair, client: std::sync::Arc<C>) -> Self {
        tracing::debug!(signer = %signer.public().to_address(), "starting transaction sequencer");

        let client_clone = client.clone();
        let (sender, receiver) = futures::channel::mpsc::channel::<TxRequest<R>>(TX_QUEUE_CAPACITY);

        // Nonce of our node signer never expires, all other signers expire after 10 minutes
        let current_nonce = moka::sync::CacheBuilder::new(1024)
            .expire_after(FixedTti {
                fixed: signer.public().to_address(),
                tti: Duration::from_mins(10),
            })
            .build();

        let current_nonce_clone = current_nonce.clone();
        hopr_utils::runtime::prelude::spawn(
            receiver
                .then(move |(tx, tx_signer, notifier): (R, _, _)| {
                    let client = client_clone.clone();
                    let signer = tx_signer.unwrap_or_else(|| signer.clone());
                    let signer_addr = signer.public().to_address();
                    let current_nonce = current_nonce.clone();
                    async move {
                        let chain_info = match client.query_chain_info().map_err(ConnectorError::from).await {
                            Ok(chain_info) => {
                                tracing::debug!(chain_id = chain_info.chain_id, "chain info retrieved for tx");
                                chain_info
                            }
                            Err(e) => return (Err(e), signer_addr, notifier),
                        };

                        let parsed_chain_info = match model_to_chain_info(chain_info) {
                            Ok(parsed_chain_info) => parsed_chain_info,
                            Err(error) => return (Err(error), signer_addr, notifier),
                        };
                        let chain_id = parsed_chain_info.info.chain_id;
                        let gas_estimation = GasEstimation::from(parsed_chain_info);
                        tracing::debug!(?gas_estimation, "gas estimation used for tx");

                        // We always query the transaction count for the signer and use
                        // the maximum between this value and the local counter
                        match client
                            .query_transaction_count(&signer_addr.into())
                            .map_err(ConnectorError::from)
                            .await
                        {
                            Ok(tx_count) => {
                                let prev_nonce = current_nonce
                                    .entry(signer_addr)
                                    .or_default()
                                    .value()
                                    .fetch_max(tx_count, std::sync::atomic::Ordering::Relaxed);

                                tracing::debug!(prev_nonce, tx_count, "transaction count retrieved");
                            }
                            Err(e) => return (Err(e), signer_addr, notifier),
                        }

                        // At this point the value must be set, so we can load it
                        let nonce = current_nonce
                            .entry(signer_addr)
                            .or_default()
                            .value()
                            .load(std::sync::atomic::Ordering::Relaxed);
                        tracing::debug!(nonce, signer = %signer_addr, "nonce used for the tx");

                        tx.sign_and_encode_to_eip2718(nonce, chain_id, gas_estimation.into(), &signer)
                            .map_err(ConnectorError::from)
                            .and_then(move |tx| {
                                tracing::debug!(nonce, signer = %signer_addr, "submitting transaction");
                                let client = client.clone();
                                async move {
                                    client
                                        .submit_and_track_transaction(&tx)
                                        .map_err(ConnectorError::from)
                                        .await
                                }
                            })
                            .map(|res| (res, signer_addr, notifier))
                            .await
                    }
                })
                .for_each(move |(res, signer_addr, notifier)| {
                    // The nonce is incremented when the transaction succeeded or failed due to on-chain
                    // rejection.
                    if res.is_ok()
                        || res
                            .as_ref()
                            .is_err_and(|error| error.as_transaction_rejection_error().is_some())
                    {
                        // Increment the nonce
                        let prev_nonce = current_nonce_clone
                            .entry(signer_addr)
                            .or_default()
                            .value()
                            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                        tracing::debug!(prev_nonce, signer = %signer_addr, ?res, "nonce incremented due to tx success or rejection");
                    } else {
                        tracing::warn!(?res, signer = %signer_addr, "nonce not incremented due to tx failure other than rejection");
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
        custom_signer: Option<ChainKeypair>,
    ) -> errors::Result<impl Future<Output = errors::Result<blokli_client::api::types::Transaction>>> {
        let (notifier_tx, notifier_rx) = futures::channel::oneshot::channel();

        self.sender
            .clone()
            .send((transaction, custom_signer, notifier_tx))
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
