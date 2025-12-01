use blokli_client::api::{BlokliQueryClient, BlokliTransactionClient};
use futures::{FutureExt, SinkExt, StreamExt, TryFutureExt};
use hopr_chain_types::prelude::{GasEstimation, SignableTransaction};
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

const TX_QUEUE_CAPACITY: usize = 2048;

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

        let (sender, receiver) = futures::channel::mpsc::channel(TX_QUEUE_CAPACITY);
        self.sender = Some(sender);

        tracing::debug!(tx_count, signer = %self.signer.public().to_address(), "starting transaction sequencer");

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
                        // We currently use fixed gas estimation.
                        tx.sign_and_encode_to_eip2718(nonce, GasEstimation::default().into(), &signer)
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
                        let prev_nonce = nonce_inc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        tracing::debug!(prev_nonce, ?res, "nonce incremented due to tx success or rejection");
                    } else {
                        tracing::debug!(?res, "nonce not incremented due to tx failure other than rejection");
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
        let mut sender = self
            .sender
            .clone()
            .ok_or(ConnectorError::InvalidState("transaction sender not started"))?;

        let (notifier_tx, notifier_rx) = futures::channel::oneshot::channel();

        sender
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
        if let Some(mut sender) = self.sender.take() {
            // Causes the internally spawned task that drives the queue to terminate
            sender.close_channel();
        }
    }
}

#[cfg(test)]
mod tests {
    use hopr_primitive_types::traits::BytesRepresentable;
    use hopr_chain_types::ContractAddresses;
    use hopr_chain_types::payload::SafePayloadGenerator;
    use hopr_chain_types::prelude::PayloadGenerator;
    use hopr_primitive_types::prelude::*;

    use crate::connector::tests::{MODULE_ADDR, PRIVATE_KEY_1};
    use crate::testing::BlokliTestStateBuilder;
    use super::*;

    #[tokio::test]
    async fn test_sequencer_should_increase_nonce_after_successful_tx() -> anyhow::Result<()> {
        let blokli_client = BlokliTestStateBuilder::default()
            .with_balances([([1u8; Address::SIZE].into(), HoprBalance::zero())])
            .with_balances([([1u8; Address::SIZE].into(), XDaiBalance::zero())])
            .with_balances([(ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(), XDaiBalance::new_base(10))])
            .with_balances([(ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(), HoprBalance::new_base(1000))])
            .with_hopr_network_chain_info(1, "rotsee")
            .build_dynamic_client(MODULE_ADDR.into());

        let blokli_client = std::sync::Arc::new(blokli_client);

        let mut tx_sequencer = TransactionSequencer::<_, <SafePayloadGenerator as PayloadGenerator>::TxRequest>::new(ChainKeypair::from_secret(&PRIVATE_KEY_1)?, blokli_client.clone());
        tx_sequencer.start().await?;

        let safe_tx_gen = SafePayloadGenerator::new(&ChainKeypair::from_secret(&PRIVATE_KEY_1)?, ContractAddresses::for_network("rotsee").unwrap(), MODULE_ADDR.into());

        let nonce_before = tx_sequencer.nonce.load(std::sync::atomic::Ordering::SeqCst);

        tx_sequencer.enqueue_transaction(
            safe_tx_gen.transfer([1u8; Address::SIZE].into(), HoprBalance::new_base(100))?,
            std::time::Duration::from_secs(2)
        ).await?.await?;

        assert!(nonce_before < tx_sequencer.nonce.load(std::sync::atomic::Ordering::SeqCst), "nonce should be incremented after successful tx");

        insta::assert_yaml_snapshot!(*blokli_client.snapshot());

        Ok(())
    }

    #[tokio::test]
    async fn test_sequencer_should_increase_nonce_after_rejected_tx() -> anyhow::Result<()> {
        let blokli_client = BlokliTestStateBuilder::default()
            .with_balances([([1u8; Address::SIZE].into(), HoprBalance::zero())])
            .with_balances([([1u8; Address::SIZE].into(), XDaiBalance::zero())])
            .with_balances([(ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(), XDaiBalance::new_base(10))])
            .with_balances([(ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(), HoprBalance::new_base(1000))])
            .with_hopr_network_chain_info(1, "rotsee")
            .build_dynamic_client(MODULE_ADDR.into());

        let blokli_client = std::sync::Arc::new(blokli_client);

        let mut tx_sequencer = TransactionSequencer::<_, <SafePayloadGenerator as PayloadGenerator>::TxRequest>::new(ChainKeypair::from_secret(&PRIVATE_KEY_1)?, blokli_client.clone());
        tx_sequencer.start().await?;

        let safe_tx_gen = SafePayloadGenerator::new(&ChainKeypair::from_secret(&PRIVATE_KEY_1)?, ContractAddresses::for_network("rotsee").unwrap(), MODULE_ADDR.into());

        let nonce_before = tx_sequencer.nonce.load(std::sync::atomic::Ordering::SeqCst);

        assert!(tx_sequencer.enqueue_transaction(
            safe_tx_gen.transfer([1u8; Address::SIZE].into(), HoprBalance::new_base(100000))?,
            std::time::Duration::from_secs(2)
        ).await?.await.is_err(), "rejected tx should fail");

        assert!(nonce_before < tx_sequencer.nonce.load(std::sync::atomic::Ordering::SeqCst), "nonce should be incremented after rejected tx");

        insta::assert_yaml_snapshot!(*blokli_client.snapshot());

        Ok(())
    }
}
