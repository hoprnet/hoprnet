use async_trait::async_trait;
use ethers_providers::{JsonRpcClient, Middleware};
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures::future::poll_fn;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use utils_log::{debug, error, info, warn};
use utils_types::sma::{NoSumSMA, SMA};

use crate::errors::Result;
use crate::errors::RpcError::{GeneralError, NoSuchBlock};
use crate::rpc::{HoprMiddleware, RpcOperations};
use crate::{BlockWithLogs, HoprIndexerRpcOperations, Log, LogFilter};

#[cfg(all(not(feature = "wasm"), not(test)))]
use async_std::task::{sleep, spawn_local};

#[cfg(test)]
use tokio::{task::spawn_local, time::sleep};

#[cfg(all(feature = "wasm", not(test)))]
use gloo_timers::future::sleep;

#[cfg(all(feature = "wasm", not(test)))]
use wasm_bindgen_futures::spawn_local;

const BLOCK_TIME_AVG_WINDOW_SIZE: u32 = 10;

async fn send_block_with_logs(tx: &mut UnboundedSender<BlockWithLogs>, block: BlockWithLogs) -> Result<()> {
    match poll_fn(|cx| Pin::new(&tx).poll_ready(cx)).await {
        Ok(_) => match tx.start_send(block) {
            Ok(_) => Ok(()),
            Err(_) => Err(GeneralError("failed to pass block with logs to the receiver".into())),
        },
        Err(_) => Err(GeneralError("receiver has been closed".into())),
    }
}

async fn get_block_with_logs_from_provider<P: JsonRpcClient + 'static>(
    block_number: u64,
    query: &LogFilter,
    provider: Arc<HoprMiddleware<P>>,
) -> Result<BlockWithLogs> {
    debug!("getting block #{block_number} with logs");
    match provider.get_block(block_number).await? {
        Some(block) => {
            let logs = if !query.is_empty() {
                debug!("getting logs from #{block_number} using {query}");
                let filter = ethers::types::Filter::from(query.clone())
                    .at_block_hash(block.hash.expect("block must have block hash"));

                provider.get_logs(&filter).await?.into_iter().map(Log::from).collect()
            } else {
                debug!("empty log filter for block #{block_number}");
                Vec::new()
            };

            Ok(BlockWithLogs {
                block: block.into(),
                logs,
            })
        }
        None => Err(NoSuchBlock),
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<P: JsonRpcClient + 'static> HoprIndexerRpcOperations for RpcOperations<P> {
    async fn block_number(&self) -> Result<u64> {
        let r = self.provider.get_block_number().await?;
        Ok(r.as_u64())
    }

    async fn try_block_with_logs_stream(
        &self,
        start_block_number: Option<u64>,
        filter: LogFilter,
    ) -> Result<UnboundedReceiver<BlockWithLogs>> {
        let (mut tx, rx) = futures::channel::mpsc::unbounded::<BlockWithLogs>();

        // The provider internally performs retries on timeouts and errors.
        let provider = self.provider.clone();
        let latest_block = self.block_number().await?;
        let start_block = start_block_number.unwrap_or(latest_block);
        let initial_poll_backoff = self.cfg.expected_block_time;

        spawn_local(async move {
            let mut current = start_block;
            let mut latest = latest_block;

            let mut block_time_sma = NoSumSMA::<Duration, u32>::new(BLOCK_TIME_AVG_WINDOW_SIZE);
            let mut prev_block = None;

            while current <= latest {
                match get_block_with_logs_from_provider(current, &filter, provider.clone()).await {
                    Ok(block_with_logs) => {
                        let new_current = block_with_logs.block.number.expect("past block must not be pending");
                        let block_ts = Duration::from_secs(block_with_logs.block.timestamp.as_u64());
                        info!("got past {block_with_logs}");

                        if let Err(e) = send_block_with_logs(&mut tx, block_with_logs).await {
                            error!("failed to dispatch past block: {e}");
                            break; // Receiver is closed, terminate the stream
                        }

                        if let Some(prev_block_ts) = prev_block {
                            block_time_sma.add_sample(block_ts - prev_block_ts);
                        }

                        prev_block = Some(block_ts);
                        current = new_current + 1;

                        match provider.get_block_number().await.map(|u| u.as_u64()) {
                            Ok(block_number) => {
                                latest = block_number;
                            }
                            Err(e) => {
                                error!("failed to get latest block number: {e}");
                            }
                        }
                    }
                    Err(e) => {
                        error!("failed to obtain block #{current} with logs: {e}");
                    }
                }
            }
            if current >= latest {
                let mut current_backoff = block_time_sma.get_average().unwrap_or(initial_poll_backoff);
                debug!("done receiving past blocks {start_block}-{latest}, polling for new blocks > {latest} with initial backoff {} ms", current_backoff.as_millis());
                loop {
                    sleep(current_backoff).await;

                    match get_block_with_logs_from_provider(current, &filter, provider.clone()).await {
                        Ok(block_with_logs) => {
                            let new_current = block_with_logs.block.number.expect("new block must not be pending");
                            let block_ts = Duration::from_secs(block_with_logs.block.timestamp.as_u64());
                            info!("got new {block_with_logs}");

                            if let Err(e) = send_block_with_logs(&mut tx, block_with_logs).await {
                                error!("failed to dispatch new block: {e}");
                                break; // Receiver is closed, terminate the stream
                            }

                            if let Some(prev_block_ts) = prev_block {
                                block_time_sma.add_sample(block_ts - prev_block_ts);
                            }

                            prev_block = Some(block_ts);
                            current = new_current + 1;

                            current_backoff = block_time_sma.get_average().unwrap_or(initial_poll_backoff);
                        }
                        Err(NoSuchBlock) => {
                            current_backoff = (current_backoff / 2).max(Duration::from_millis(100));
                            debug!("no block #{current}, waiting {} ms", current_backoff.as_millis());
                        }
                        Err(e) => {
                            error!("failed to obtain block {} with logs: {e}", current + 1);
                        }
                    }
                }
            } else {
                error!("processing past blocks did not get up to the latest block {current} < {latest}");
            }

            warn!("block processing done");
            tx.close_channel();
        });

        Ok(rx)
    }
}

#[cfg(test)]
mod test {
    use crate::rpc::tests::{mint_tokens, mock_config};
    use crate::rpc::RpcOperations;
    use crate::{HoprIndexerRpcOperations, LogFilter};
    use bindings::hopr_channels::*;
    use bindings::hopr_token::{ApprovalFilter, HoprToken, TransferFilter};
    use core_crypto::keypairs::{ChainKeypair, Keypair};
    use ethers::contract::EthEvent;
    use ethers_providers::{Http, Middleware};
    use futures::StreamExt;
    use std::str::FromStr;
    use std::time::Duration;
    use core_ethereum_misc::{ContractAddresses, ContractInstances, create_anvil, create_rpc_client_to_anvil};
    use utils_types::primitives::Address;
    use utils_types::traits::BinarySerializable;

    async fn fund_channel<M: Middleware + 'static>(
        sender: Address,
        counterparty: Address,
        hopr_token: HoprToken<M>,
        hopr_channels: HoprChannels<M>,
    ) {
        mint_tokens(hopr_token.clone(), 1000_u128, sender).await;

        hopr_token
            .approve(hopr_channels.address(), 1u128.into())
            .send()
            .await
            .unwrap()
            .await
            .unwrap();

        hopr_channels
            .fund_channel(ethers::types::Address::from_slice(&counterparty.to_bytes()), 1u128)
            .send()
            .await
            .unwrap()
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_try_stream_with_logs_should_contain_all_logs_when_opening_channel() {
        let _ = env_logger::builder().is_test(true).try_init();

        let anvil = create_anvil(Some(Duration::from_secs(1)));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();
        let chain_key_1 = ChainKeypair::from_secret(anvil.keys()[1].to_bytes().as_ref()).unwrap();

        // Deploy contracts
        let contract_instances = {
            let client = create_rpc_client_to_anvil(&anvil, &chain_key_0);
            ContractInstances::deploy_for_testing(client, &chain_key_0).await.expect("could not deploy contracts")
        };

        let contract_addrs = ContractAddresses::from(&contract_instances);

        let mut cfg = mock_config();
        cfg.contract_addrs = contract_addrs.clone();

        let rpc = RpcOperations::new(Http::from_str(&anvil.endpoint()).unwrap(), &chain_key_0, cfg)
            .expect("failed to construct rpc");

        let log_filter = LogFilter {
            address: vec![contract_addrs.token, contract_addrs.channels],
            topics: vec![
                TransferFilter::signature(),
                ApprovalFilter::signature(),
                ChannelOpenedFilter::signature(),
                ChannelBalanceIncreasedFilter::signature(),
            ],
        };

        let local = tokio::task::LocalSet::new();

        // Spawn channel funding
        local.spawn_local(async move {
            tokio::time::sleep(Duration::from_secs(1)).await;
            fund_channel(chain_key_0.public().to_address(), chain_key_1.public().to_address(), contract_instances.token, contract_instances.channels).await;
        });

        // Spawn stream
        let (tx, rx) = futures::channel::oneshot::channel();
        let filter_clone = log_filter.clone();
        local.spawn_local(async move {
            let mut blocks = rpc
                .try_block_with_logs_stream(Some(1), filter_clone)
                .await
                .unwrap()
                .skip_while(|block| futures::future::ready(block.logs.len() != 4))
                .take(1)
                .collect::<Vec<_>>()
                .await;

            let _ = tx.send(blocks.pop().unwrap().logs);
        });

        // Everything must complete within 30 seconds
        tokio::time::timeout(Duration::from_secs(30), local)
            .await
            .expect("timeout");

        // The logs within the single block must contain all 4 events
        let retrieved_logs = rx.await.expect("failed to retrieve logs");
        assert!(
            retrieved_logs.iter().any(|log| log.address == contract_addrs.channels
                && log.topics.contains(&ChannelOpenedFilter::signature().0.into())),
            "must contain channel open"
        );
        assert!(
            retrieved_logs.iter().any(|log| log.address == contract_addrs.channels
                && log
                    .topics
                    .contains(&ChannelBalanceIncreasedFilter::signature().0.into())),
            "must contain channel balance increase"
        );
        assert!(
            retrieved_logs
                .iter()
                .any(|log| log.address == contract_addrs.token
                    && log.topics.contains(&ApprovalFilter::signature().0.into())),
            "must contain token approval"
        );
        assert!(
            retrieved_logs
                .iter()
                .any(|log| log.address == contract_addrs.token
                    && log.topics.contains(&TransferFilter::signature().0.into())),
            "must contain token transfer"
        );
    }
}
