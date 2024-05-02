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
use futures::stream::BoxStream;
use futures::{Stream, StreamExt, TryStreamExt};
use std::pin::Pin;
use tracing::{debug, info, warn};
use tracing::{error, trace};

use crate::errors::{Result, RpcError, RpcError::FilterIsEmpty};
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

/// Splits the range between `from_block` and `to_block` (inclusive)
/// to chunks of maximum size `max_chunk_size` and creates [ethers::types::Filter] for each chunk
/// using the given [LogFilter].
fn split_range<'a>(
    filter: LogFilter,
    from_block: u64,
    to_block: u64,
    max_chunk_size: u64,
) -> BoxStream<'a, ethers::types::Filter> {
    assert!(from_block <= to_block, "invalid block range");
    assert!(max_chunk_size > 0, "chunk size must be greater than 0");

    futures::stream::unfold((from_block, to_block), move |(start, to)| {
        let subfilter = filter.clone();
        async move {
            if start <= to {
                let end = to_block.min(start + max_chunk_size - 1);
                let filter = ethers::types::Filter::from(subfilter).from_block(start).to_block(end);
                Some((filter, (end + 1, to)))
            } else {
                None
            }
        }
    })
    .boxed()
}

impl<P: JsonRpcClient + 'static> RpcOperations<P> {
    /// Retrieves logs in the given range (`from_block` and `to_block` are inclusive).
    fn stream_logs(&self, filter: LogFilter, from_block: u64, to_block: u64) -> BoxStream<Result<Log>> {
        let fetch_ranges = split_range(filter, from_block, to_block, self.cfg.max_block_range_fetch_size);

        info!(
            "polling logs from blocks #{from_block} - #{to_block} (via {:?} chunks)",
            (to_block - from_block) / self.cfg.max_block_range_fetch_size + 1
        );

        fetch_ranges
            .then(|subrange| {
                let prov_clone = self.provider.clone();
                async move {
                    match prov_clone.get_logs(&subrange).await {
                        Ok(logs) => Ok(logs),
                        Err(e) => {
                            error!(
                                "failed to fetch logs in block subrange {:?}-{:?}: {e}",
                                subrange.get_from_block(),
                                subrange.get_to_block()
                            );
                            Err(e)
                        }
                    }
                }
            })
            .flat_map(|result| {
                futures::stream::iter(match result {
                    Ok(logs) => logs.into_iter().map(|log| Ok(Log::from(log))).collect::<Vec<_>>(),
                    Err(e) => vec![Err(RpcError::from(e))],
                })
            })
            .boxed()
    }
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

            const MAX_LOOP_FAILURES: usize = 5;
            const MAX_RPC_PAST_BLOCKS: usize = 50;
            let mut count_failures = 0;

            'outer: loop {
                match self.block_number().await {
                    Ok(latest_block) => {
                        if from_block > latest_block {
                            let past_diff = from_block - latest_block;
                            if from_block == start_block_number {
                                // If on first iteration the start block is in the future, just set
                                // it to the latest
                                from_block = latest_block;
                            } else if past_diff <= MAX_RPC_PAST_BLOCKS as u64 {
                                // If we came here early (we tolerate only off-by MAX_RPC_PAST_BLOCKS), wait some more
                                warn!("too early query, RPC provider still at block {latest_block}, diff to DB: {past_diff}");
                                futures_timer::Delay::new(past_diff as u32 * self.cfg.expected_block_time / 3).await;
                                continue;
                            } else {
                                // This is a hard-failure on later iterations which is unrecoverable
                                panic!("indexer start block number {from_block} is greater than the chain latest block number {latest_block} (diff {past_diff}) =>
                                possible causes: chain reorg, RPC provider out of sync, corrupted DB =>
                                possible solutions: change the RPC provider, reinitialize the DB");
                            }
                        }

                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_RPC_CHAIN_HEAD.set(latest_block as f64);

                        let mut retrieved_logs = self.stream_logs(filter.clone(), from_block, latest_block);

                        let mut current_block_log = BlockWithLogs { block_id: from_block, ..Default::default()};
                        trace!("begin processing batch #{from_block} - #{latest_block}");
                        loop {
                            match retrieved_logs.try_next().await {
                                Ok(Some(log)) => {
                                    // This in general should not happen, but handle such a case to be safe
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
                                    current_block_log.logs.insert(log);
                                },
                                Ok(None) => {
                                    trace!("done processing batch #{from_block} - #{latest_block}");
                                    break;
                                },
                                Err(e) => {
                                    error!("failure when processing blocks from RPC: {e}");
                                    count_failures += 1;

                                    if count_failures < MAX_LOOP_FAILURES {
                                        // Continue the outer loop, which throws away the current block
                                        // that may be incomplete due to this error.
                                        // We will start at this block again to re-query it.
                                        from_block = current_block_log.block_id;
                                        continue 'outer;
                                    } else {
                                        panic!("!!! Cannot advance the chain indexing due to unrecoverable RPC errors.

                                        The RPC provider does not seem to be working correctly. 
                                        
                                        The last encountered error was: {e}");
                                    }
                                }
                            }
                        }

                        // Yield everything we've collected until this point
                        debug!("completed {current_block_log}");
                        yield current_block_log;
                        from_block = latest_block + 1;
                        count_failures = 0;
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
    use tracing::debug;

    use crate::client::native::SurfRequestor;
    use crate::client::{create_rpc_client_to_anvil, JsonRpcProviderClient, SimpleJsonRpcRetryPolicy};
    use crate::indexer::split_range;
    use crate::rpc::{RpcOperations, RpcOperationsConfig};
    use crate::{BlockWithLogs, HoprIndexerRpcOperations, LogFilter};

    fn filter_bounds(filter: &ethers::types::Filter) -> (u64, u64) {
        (
            filter
                .block_option
                .get_from_block()
                .unwrap()
                .as_number()
                .unwrap()
                .as_u64(),
            filter
                .block_option
                .get_to_block()
                .unwrap()
                .as_number()
                .unwrap()
                .as_u64(),
        )
    }

    #[async_std::test]
    async fn test_split_range() {
        let ranges = split_range(LogFilter::default(), 0, 10, 2).collect::<Vec<_>>().await;

        assert_eq!(6, ranges.len());
        assert_eq!((0, 1), filter_bounds(&ranges[0]));
        assert_eq!((2, 3), filter_bounds(&ranges[1]));
        assert_eq!((4, 5), filter_bounds(&ranges[2]));
        assert_eq!((6, 7), filter_bounds(&ranges[3]));
        assert_eq!((8, 9), filter_bounds(&ranges[4]));
        assert_eq!((10, 10), filter_bounds(&ranges[5]));

        let ranges = split_range(LogFilter::default(), 0, 0, 2).collect::<Vec<_>>().await;
        assert_eq!(1, ranges.len());
        assert_eq!((0, 0), filter_bounds(&ranges[0]));

        let ranges = split_range(LogFilter::default(), 0, 0, 1).collect::<Vec<_>>().await;
        assert_eq!(1, ranges.len());
        assert_eq!((0, 0), filter_bounds(&ranges[0]));

        let ranges = split_range(LogFilter::default(), 0, 3, 1).collect::<Vec<_>>().await;
        assert_eq!(4, ranges.len());
        assert_eq!((0, 0), filter_bounds(&ranges[0]));
        assert_eq!((1, 1), filter_bounds(&ranges[1]));
        assert_eq!((2, 2), filter_bounds(&ranges[2]));
        assert_eq!((3, 3), filter_bounds(&ranges[3]));

        let ranges = split_range(LogFilter::default(), 0, 3, 10).collect::<Vec<_>>().await;
        assert_eq!(1, ranges.len());
        assert_eq!((0, 3), filter_bounds(&ranges[0]));
    }

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
