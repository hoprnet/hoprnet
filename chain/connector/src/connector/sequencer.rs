use blokli_client::api::{AccountSelector, BlokliQueryClient, BlokliTransactionClient};
use futures::{FutureExt, SinkExt, StreamExt, TryFutureExt};
use hopr_chain_types::prelude::SignableTransaction;
use hopr_crypto_types::prelude::*;

use crate::errors::ConnectorError;

type TxRequest<T> = (
    T,
    futures::channel::oneshot::Sender<Result<blokli_client::api::TxId, ConnectorError>>,
);

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

    pub async fn start(&mut self) -> Result<(), ConnectorError> {
        let accounts = self
            .client
            .query_accounts(AccountSelector::Address(self.signer.public().to_address().into()))
            .await?;

        if accounts.len() > 1 {
            return Err(ConnectorError::InvalidState("more than one account found".into()));
        }

        if let Some(nonce) = accounts.first().and_then(|a| a.safe_transaction_count.clone()) {
            self.nonce.store(
                nonce
                    .0
                    .parse()
                    .map_err(|_| ConnectorError::InvalidState("invalid network nonce".into()))?,
                std::sync::atomic::Ordering::SeqCst,
            );
        }

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
                            .map_err(ConnectorError::from)
                            .and_then(move |tx| {
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
                    if res.is_ok() {
                        nonce_inc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    }
                    if let Err(_) = notifier.send(res) {
                        tracing::error!("failed to notify transaction result");
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
    pub async fn enqueue_transaction(
        &self,
        transaction: R,
        tx_timeout: std::time::Duration,
    ) -> Result<impl Future<Output = Result<blokli_client::api::types::Transaction, ConnectorError>>, ConnectorError>
    {
        let sender = self
            .sender
            .as_ref()
            .ok_or(ConnectorError::InvalidState("transaction sender not started".into()))?;

        let (notifier_tx, notifier_rx) = futures::channel::oneshot::channel();

        sender
            .clone()
            .send((transaction, notifier_tx))
            .await
            .map_err(|_| ConnectorError::InvalidState("transaction queue dropped".into()))?;

        Ok(notifier_rx
            .map(move |result| {
                result
                    .map_err(|_| ConnectorError::InvalidState("transaction notifier dropped".into()))
                    .and_then(|tx_res| tx_res.map(|id| (id, tx_timeout)))
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
