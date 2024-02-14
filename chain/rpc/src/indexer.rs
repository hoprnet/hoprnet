//! Extends the [RpcOperations] type with functionality needed by the Indexer component.
//!
//! The functionality required functionality is defined in the [HoprIndexerRpcOperations] trait,
//! which is implemented for [RpcOperations] hereof.
//! The primary goal is to provide a stream of [BlockWithLogs] filtered by the given [LogFilter]
//! as the new matching blocks are mined in the underlying blockchain. The stream also allows to collect
//! historical blockchain data.
//!
//! For details on the Indexer see the `chain-indexer` crate.
use async_stream::stream;
use async_trait::async_trait;
use ethers::providers::{JsonRpcClient, Middleware};
use ethers::types::BlockNumber;
use futures::{Stream, StreamExt, TryStreamExt};
use log::error;
use log::{debug, warn};
use std::pin::Pin;

use crate::errors::{Result, RpcError::FilterIsEmpty};
use crate::rpc::RpcOperations;
use crate::{BlockWithLogs, HoprIndexerRpcOperations, Log, LogFilter};

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::SimpleGauge;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_RPC_CHAIN_HEAD: SimpleGauge =
        SimpleGauge::new(
            "hopr_chain_head_block_number",
            "Current block number of chain head",
    ).unwrap();
}

#[async_trait]
impl<P: JsonRpcClient + 'static> HoprIndexerRpcOperations for RpcOperations<P> {
    async fn block_number(&self) -> Result<u64> {
        self.get_block_number().await
    }

    fn try_stream_logs<'a>(
        &'a self,
        start_block_number: u64,
        filter: LogFilter,
    ) -> Result<Pin<Box<dyn Stream<Item = BlockWithLogs> + Send + 'a>>> {
        if filter.is_empty() {
            return Err(FilterIsEmpty);
        }

        Ok(Box::pin(stream! {
            // On first iteration use the given block number as start
            let mut from_block = start_block_number;

            loop {
                match self.block_number().await {
                    Ok(latest_block) => {
                        if from_block > latest_block {
                            if from_block == start_block_number {
                                // If on first iteration the start block is in the future, just set it to latest
                                from_block = latest_block;
                            } else if from_block - 1 == latest_block {
                                // If we came here early (we tolerate only off-by one), wait some more
                                futures_timer::Delay::new(self.cfg.expected_block_time / 3).await;
                                continue;
                            } else {
                                // This is a hard-failure on subsequent iterations which is unrecoverable
                                panic!("indexer start block number {from_block} is greater than the chain latest block number {latest_block} =>
                                possible causes: chain reorg, RPC provider out of sync, corrupted DB =>
                                possible solutions: change the RPC provider, reinitialize the DB");
                            }
                        }

                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_RPC_CHAIN_HEAD.set(latest_block as f64);

                        // Range is inclusive
                        let range_filter = ethers::types::Filter::from(filter.clone())
                            .from_block(BlockNumber::Number(from_block.into()))
                            .to_block(BlockNumber::Number(latest_block.into()));

                        // Range of blocks to fetch is always bounded
                        let range_size = self.cfg.max_block_range_fetch_size.min(latest_block - from_block);
                        debug!("polling logs from blocks #{from_block} - #{latest_block} (range size {range_size})");

                        // If we're fetching logs from wide block range, we'll use the pagination log query.
                        let mut retrieved_logs = if range_size >= self.cfg.min_block_range_fetch_size {
                            self.provider.get_logs_paginated(&range_filter, range_size).boxed()
                        } else {
                            // For smaller block ranges, we use the ordinary getLogs call which minimizes RPC calls
                            futures::stream::iter(self.provider.get_logs(&range_filter)
                                .await
                                .unwrap_or_else(|e| {
                                    error!("polling logs from #{from_block} - #{latest_block} failed: {e}");
                                    Vec::new()
                                })
                                .into_iter()
                                .map(Ok)
                            ).boxed()
                        };

                        let mut current_block_log = BlockWithLogs { block_id: from_block, ..Default::default()};
                        while let Ok(Some(log)) = retrieved_logs.try_next().await {
                            let log = Log::from(log);

                            // This in general should not happen, but handle such case to be safe
                            if log.block_number > latest_block {
                                warn!("got {log} that has not yet reached the finalized tip at {latest_block}");
                                break;
                            }

                            // This assumes the logs are arriving ordered by blocks when fetching a range
                            if current_block_log.block_id != log.block_number {
                                debug!("completed {current_block_log}");
                                yield current_block_log;

                                current_block_log = BlockWithLogs::default();
                                current_block_log.block_id = log.block_number;
                            }

                            debug!("retrieved {log}");
                            current_block_log.logs.push(log);
                        }

                        // Yield everything we've collected until this point
                        debug!("completed {current_block_log}");
                        yield current_block_log;
                        from_block = latest_block + 1;
                    }
                    Err(e) => error!("failed to obtain current block number from chain: {e}")
                }

                futures_timer::Delay::new(self.cfg.expected_block_time).await;
            }
        }))
    }
}

#[cfg(test)]
mod test {
    use async_std::prelude::FutureExt;
    use ethers::contract::EthEvent;
    use futures::StreamExt;
    use std::time::Duration;

    use bindings::hopr_channels::*;
    use bindings::hopr_token::{ApprovalFilter, TransferFilter};
    use chain_types::{ContractAddresses, ContractInstances};
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
    use log::debug;

    use crate::client::native::SurfRequestor;
    use crate::client::{create_rpc_client_to_anvil, JsonRpcProviderClient, SimpleJsonRpcRetryPolicy};
    use crate::rpc::{RpcOperations, RpcOperationsConfig};
    use crate::{BlockWithLogs, HoprIndexerRpcOperations, LogFilter};

    #[async_std::test]
    async fn test_should_get_block_number() {
        let expected_block_time = Duration::from_secs(1);
        let anvil = chain_types::utils::create_anvil(Some(expected_block_time));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();

        let client = JsonRpcProviderClient::new(
            &anvil.endpoint(),
            SurfRequestor::default(),
            SimpleJsonRpcRetryPolicy::default(),
        );

        let cfg = RpcOperationsConfig {
            finality: 2,
            expected_block_time,
            ..RpcOperationsConfig::default()
        };

        // Wait until contracts deployments are final
        async_std::task::sleep((1 + cfg.finality) * expected_block_time).await;

        let rpc = RpcOperations::new(client, &chain_key_0, cfg).expect("failed to construct rpc");

        let b1 = rpc.block_number().await.expect("should get block number");

        async_std::task::sleep(expected_block_time * 2).await;

        let b2 = rpc.block_number().await.expect("should get block number");

        assert!(b2 > b1, "block number should increase");
    }

    #[async_std::test]
    async fn test_try_stream_logs_should_contain_all_logs_when_opening_channel() {
        let _ = env_logger::builder().is_test(true).try_init();

        let expected_block_time = Duration::from_secs(1);

        let anvil = chain_types::utils::create_anvil(Some(expected_block_time));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();
        let chain_key_1 = ChainKeypair::from_secret(anvil.keys()[1].to_bytes().as_ref()).unwrap();

        // Deploy contracts
        let contract_instances = {
            let client = create_rpc_client_to_anvil(SurfRequestor::default(), &anvil, &chain_key_0);
            ContractInstances::deploy_for_testing(client, &chain_key_0)
                .await
                .expect("could not deploy contracts")
        };

        let tokens_minted_at =
            chain_types::utils::mint_tokens(contract_instances.token.clone(), 1000_u128.into()).await;
        debug!("tokens were minted at block {tokens_minted_at}");

        let contract_addrs = ContractAddresses::from(&contract_instances);

        let cfg = RpcOperationsConfig {
            tx_polling_interval: Duration::from_millis(10),
            contract_addrs,
            expected_block_time,
            ..RpcOperationsConfig::default()
        };

        let client = JsonRpcProviderClient::new(
            &anvil.endpoint(),
            SurfRequestor::default(),
            SimpleJsonRpcRetryPolicy::default(),
        );

        // Wait until contracts deployments are final
        async_std::task::sleep((1 + cfg.finality) * expected_block_time).await;

        let rpc = RpcOperations::new(client, &chain_key_0, cfg).expect("failed to construct rpc");

        let log_filter = LogFilter {
            address: vec![contract_addrs.token, contract_addrs.channels],
            topics: vec![
                TransferFilter::signature().into(),
                ApprovalFilter::signature().into(),
                ChannelOpenedFilter::signature().into(),
                ChannelBalanceIncreasedFilter::signature().into(),
            ],
        };

        debug!("{:#?}", contract_addrs);
        debug!("{:#?}", log_filter);

        // Spawn channel funding
        async_std::task::spawn(async move {
            chain_types::utils::fund_channel(
                chain_key_1.public().to_address(),
                contract_instances.token,
                contract_instances.channels,
                1_u128.into(),
            )
            .delay(expected_block_time * 2)
            .await;
        });

        // Spawn stream
        let count_filtered_topics = log_filter.topics.len();
        let retrieved_logs = rpc
            .try_stream_logs(1, log_filter)
            .expect("must create stream")
            .skip_while(|b| futures::future::ready(b.len() != count_filtered_topics))
            .take(1)
            .collect::<Vec<BlockWithLogs>>()
            .timeout(Duration::from_secs(30))
            .await
            .expect("timeout"); // Everything must complete within 30 seconds

        // The last block must contain all 4 events
        let last_block_logs = retrieved_logs.last().unwrap().clone().logs;

        assert!(
            last_block_logs.iter().any(|log| log.address == contract_addrs.channels
                && log.topics.contains(&ChannelOpenedFilter::signature().0.into())),
            "must contain channel open"
        );
        assert!(
            last_block_logs.iter().any(|log| log.address == contract_addrs.channels
                && log
                    .topics
                    .contains(&ChannelBalanceIncreasedFilter::signature().0.into())),
            "must contain channel balance increase"
        );
        assert!(
            last_block_logs
                .iter()
                .any(|log| log.address == contract_addrs.token
                    && log.topics.contains(&ApprovalFilter::signature().0.into())),
            "must contain token approval"
        );
        assert!(
            last_block_logs
                .iter()
                .any(|log| log.address == contract_addrs.token
                    && log.topics.contains(&TransferFilter::signature().0.into())),
            "must contain token transfer"
        );
    }

    #[async_std::test]
    async fn test_try_stream_logs_should_contain_only_channel_logs_when_filtered_on_funding_channel() {
        let _ = env_logger::builder().is_test(true).try_init();

        let expected_block_time = Duration::from_secs(1);

        let anvil = chain_types::utils::create_anvil(Some(expected_block_time));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();
        let chain_key_1 = ChainKeypair::from_secret(anvil.keys()[1].to_bytes().as_ref()).unwrap();

        // Deploy contracts
        let contract_instances = {
            let client = create_rpc_client_to_anvil(SurfRequestor::default(), &anvil, &chain_key_0);
            ContractInstances::deploy_for_testing(client, &chain_key_0)
                .await
                .expect("could not deploy contracts")
        };

        let tokens_minted_at =
            chain_types::utils::mint_tokens(contract_instances.token.clone(), 1000_u128.into()).await;
        debug!("tokens were minted at block {tokens_minted_at}");

        let contract_addrs = ContractAddresses::from(&contract_instances);

        let cfg = RpcOperationsConfig {
            tx_polling_interval: Duration::from_millis(10),
            contract_addrs,
            expected_block_time,
            finality: 2,
            ..RpcOperationsConfig::default()
        };

        let client = JsonRpcProviderClient::new(
            &anvil.endpoint(),
            SurfRequestor::default(),
            SimpleJsonRpcRetryPolicy::default(),
        );

        // Wait until contracts deployments are final
        async_std::task::sleep((1 + cfg.finality) * expected_block_time).await;

        let rpc = RpcOperations::new(client, &chain_key_0, cfg).expect("failed to construct rpc");

        let log_filter = LogFilter {
            address: vec![contract_addrs.channels],
            topics: vec![
                ChannelOpenedFilter::signature().into(),
                ChannelBalanceIncreasedFilter::signature().into(),
            ],
        };

        debug!("{:#?}", contract_addrs);
        debug!("{:#?}", log_filter);

        // Spawn channel funding
        async_std::task::spawn(async move {
            chain_types::utils::fund_channel(
                chain_key_1.public().to_address(),
                contract_instances.token,
                contract_instances.channels,
                1_u128.into(),
            )
            .delay(expected_block_time * 2)
            .await;
        });

        // Spawn stream
        let count_filtered_topics = log_filter.topics.len();
        let retrieved_logs = rpc
            .try_stream_logs(1, log_filter)
            .expect("must create stream")
            .skip_while(|b| futures::future::ready(b.len() != count_filtered_topics))
            .take(1)
            .collect::<Vec<BlockWithLogs>>()
            .timeout(Duration::from_secs(30))
            .await
            .expect("timeout"); // Everything must complete within 30 seconds

        // The last block must contain all 2 events
        let last_block_logs = retrieved_logs.first().unwrap().clone().logs;

        assert!(
            last_block_logs.iter().any(|log| log.address == contract_addrs.channels
                && log.topics.contains(&ChannelOpenedFilter::signature().0.into())),
            "must contain channel open"
        );
        assert!(
            last_block_logs.iter().any(|log| log.address == contract_addrs.channels
                && log
                    .topics
                    .contains(&ChannelBalanceIncreasedFilter::signature().0.into())),
            "must contain channel balance increase"
        );
    }
}
