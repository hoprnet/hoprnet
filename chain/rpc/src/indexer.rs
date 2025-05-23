//! Extends the [RpcOperations] type with functionality needed by the Indexer component.
//!
//! The functionality required functionality is defined in the [HoprIndexerRpcOperations] trait,
//! which is implemented for [RpcOperations] hereof.
//! The primary goal is to provide a stream of [BlockWithLogs] filtered by the given [LogFilter]
//! as the new matching blocks are mined in the underlying blockchain. The stream also allows to collect
//! historical blockchain data.
//!
//! For details on the Indexer see the `chain-indexer` crate.
use std::pin::Pin;

use alloy::{providers::Provider, rpc::types::Filter};
use async_stream::stream;
use async_trait::async_trait;
use futures::{Stream, StreamExt, stream::BoxStream};
#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::SimpleGauge;
use tracing::{debug, error, trace, warn};

use crate::{
    BlockWithLogs, HoprIndexerRpcOperations, Log, LogFilter,
    errors::{Result, RpcError, RpcError::FilterIsEmpty},
    rpc::RpcOperations,
    transport::HttpRequestor,
};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_RPC_CHAIN_HEAD: SimpleGauge =
        SimpleGauge::new(
            "hopr_chain_head_block_number",
            "Current block number of chain head",
    ).unwrap();
}

/// Splits the range between `from_block` and `to_block` (inclusive)
/// to chunks of maximum size `max_chunk_size` and creates [alloy::rpc::types::Filter] for each chunk
/// using the given [LogFilter].
fn split_range<'a>(filter: LogFilter, from_block: u64, to_block: u64, max_chunk_size: u64) -> BoxStream<'a, Filter> {
    assert!(from_block <= to_block, "invalid block range");
    assert!(max_chunk_size > 0, "chunk size must be greater than 0");

    futures::stream::unfold((from_block, to_block), move |(start, to)| {
        if start <= to {
            let end = to_block.min(start + max_chunk_size - 1);
            let filter = Filter::from(filter.clone()).from_block(start).to_block(end);
            futures::future::ready(Some((filter, (end + 1, to))))
        } else {
            futures::future::ready(None)
        }
    })
    .boxed()
}

// impl<P: JsonRpcClient + 'static, R: HttpRequestor + 'static> RpcOperations<P, R> {
impl<R: HttpRequestor + 'static + Clone> RpcOperations<R> {
    /// Retrieves logs in the given range (`from_block` and `to_block` are inclusive).
    fn stream_logs(&self, filter: LogFilter, from_block: u64, to_block: u64) -> BoxStream<Result<Log>> {
        let fetch_ranges = split_range(filter, from_block, to_block, self.cfg.max_block_range_fetch_size);

        debug!(
            "polling logs from blocks #{from_block} - #{to_block} (via {:?} chunks)",
            (to_block - from_block) / self.cfg.max_block_range_fetch_size + 1
        );

        fetch_ranges
            .then(|subrange| {
                let prov_clone = self.provider.clone();

                async move {
                    trace!(
                        from = ?subrange.get_from_block(),
                        to = ?subrange.get_to_block(),
                        "fetching logs in block subrange"
                    );
                    match prov_clone.get_logs(&subrange).await {
                        Ok(logs) => Ok(logs),
                        Err(e) => {
                            error!(
                                from = ?subrange.get_from_block(),
                                to = ?subrange.get_to_block(),
                                error = %e,
                                "failed to fetch logs in block subrange"
                            );
                            Err(e)
                        }
                    }
                }
            })
            .flat_map(|result| {
                futures::stream::iter(match result {
                    Ok(logs) => logs.into_iter().map(|log| Ok(Log::try_from(log)?)).collect::<Vec<_>>(),
                    Err(e) => vec![Err(RpcError::from(e))],
                })
            })
            .boxed()
    }
}

#[async_trait]
impl<R: HttpRequestor + 'static + Clone> HoprIndexerRpcOperations for RpcOperations<R> {
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
                                debug!(last_block = latest_block, start_block = start_block_number, blocks_diff = past_diff, "Indexer premature request. Block not found yet in RPC provider.");
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

                        trace!(from_block, to_block = latest_block, "processing batch");

                        let mut current_block_log = BlockWithLogs { block_id: from_block, ..Default::default()};

                        loop {
                            match retrieved_logs.next().await {
                                Some(Ok(log)) => {
                                    // This in general should not happen, but handle such a case to be safe
                                    if log.block_number > latest_block {
                                        warn!(%log, latest_block, "got log that has not yet reached the finalized tip");
                                        break;
                                    }

                                    // This should not happen, thus panic.
                                    if current_block_log.block_id > log.block_number {
                                        error!(log_block_id = log.block_number, current_block_log.block_id, "received log from a previous block");
                                        panic!("The on-chain logs are not ordered by block number. This is a critical error.");
                                    }

                                    // This assumes the logs are arriving ordered by blocks when fetching a range
                                    if current_block_log.block_id < log.block_number {
                                        debug!(block = %current_block_log, "completed block, moving to next");
                                        yield current_block_log;

                                        current_block_log = BlockWithLogs::default();
                                        current_block_log.block_id = log.block_number;
                                    }

                                    debug!("retrieved {log}");
                                    current_block_log.logs.insert(log.into());
                                },
                                None => {
                                    break;
                                },
                                Some(Err(e)) => {
                                    error!(error=%e, "failed to process blocks");
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
                        debug!(block = %current_block_log, "completed block, processing batch finished");
                        yield current_block_log;
                        from_block = latest_block + 1;
                        count_failures = 0;
                    }

                    Err(e) => error!(error = %e, "failed to obtain current block number from chain")
                }

                futures_timer::Delay::new(self.cfg.expected_block_time).await;
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use alloy::{
        primitives::U256,
        rpc::{client::ClientBuilder, types::Filter},
        sol_types::SolEvent,
        transports::{http::ReqwestTransport, layers::RetryBackoffLayer},
    };
    use anyhow::Context;
    use futures::StreamExt;
    use hopr_async_runtime::prelude::{sleep, spawn};
    use hopr_bindings::{
        hoprchannelsevents::HoprChannelsEvents::{ChannelBalanceIncreased, ChannelOpened},
        hoprtoken::HoprToken::{Approval, Transfer},
    };
    use hopr_chain_types::{ContractAddresses, ContractInstances};
    use hopr_crypto_types::{
        keypairs::{ChainKeypair, Keypair},
        types::Hash,
    };
    use tokio::time::timeout;
    use tracing::debug;

    use crate::{
        BlockWithLogs, HoprIndexerRpcOperations, LogFilter,
        client::create_rpc_client_to_anvil,
        errors::RpcError,
        indexer::split_range,
        rpc::{RpcOperations, RpcOperationsConfig},
    };

    fn filter_bounds(filter: &Filter) -> anyhow::Result<(u64, u64)> {
        Ok((
            filter
                .block_option
                .get_from_block()
                .context("a value should be present")?
                .as_number()
                .context("a value should be convertible")?,
            filter
                .block_option
                .get_to_block()
                .context("a value should be present")?
                .as_number()
                .context("a value should be convertible")?,
        ))
    }

    #[tokio::test]
    async fn test_split_range() -> anyhow::Result<()> {
        let ranges = split_range(LogFilter::default(), 0, 10, 2).collect::<Vec<_>>().await;

        assert_eq!(6, ranges.len());
        assert_eq!((0, 1), filter_bounds(&ranges[0])?);
        assert_eq!((2, 3), filter_bounds(&ranges[1])?);
        assert_eq!((4, 5), filter_bounds(&ranges[2])?);
        assert_eq!((6, 7), filter_bounds(&ranges[3])?);
        assert_eq!((8, 9), filter_bounds(&ranges[4])?);
        assert_eq!((10, 10), filter_bounds(&ranges[5])?);

        let ranges = split_range(LogFilter::default(), 0, 0, 2).collect::<Vec<_>>().await;
        assert_eq!(1, ranges.len());
        assert_eq!((0, 0), filter_bounds(&ranges[0])?);

        let ranges = split_range(LogFilter::default(), 0, 0, 1).collect::<Vec<_>>().await;
        assert_eq!(1, ranges.len());
        assert_eq!((0, 0), filter_bounds(&ranges[0])?);

        let ranges = split_range(LogFilter::default(), 0, 3, 1).collect::<Vec<_>>().await;
        assert_eq!(4, ranges.len());
        assert_eq!((0, 0), filter_bounds(&ranges[0])?);
        assert_eq!((1, 1), filter_bounds(&ranges[1])?);
        assert_eq!((2, 2), filter_bounds(&ranges[2])?);
        assert_eq!((3, 3), filter_bounds(&ranges[3])?);

        let ranges = split_range(LogFilter::default(), 0, 3, 10).collect::<Vec<_>>().await;
        assert_eq!(1, ranges.len());
        assert_eq!((0, 3), filter_bounds(&ranges[0])?);

        Ok(())
    }

    #[tokio::test]
    async fn test_should_get_block_number() -> anyhow::Result<()> {
        let expected_block_time = Duration::from_secs(1);
        let anvil = hopr_chain_types::utils::create_anvil(Some(expected_block_time));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;

        let transport_client = ReqwestTransport::new(anvil.endpoint_url());

        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new(2, 100, 100))
            .transport(transport_client.clone(), transport_client.guess_local());

        let cfg = RpcOperationsConfig {
            finality: 2,
            expected_block_time,
            gas_oracle_url: None,
            ..RpcOperationsConfig::default()
        };

        // Wait until contracts deployments are final
        sleep((1 + cfg.finality) * expected_block_time).await;

        let rpc = RpcOperations::new(rpc_client, transport_client.client().clone(), &chain_key_0, cfg)?;

        let b1 = rpc.block_number().await?;

        sleep(expected_block_time * 2).await;

        let b2 = rpc.block_number().await?;

        assert!(b2 > b1, "block number should increase");

        Ok(())
    }

    #[tokio::test]
    async fn test_try_stream_logs_should_contain_all_logs_when_opening_channel() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let expected_block_time = Duration::from_secs(1);

        let anvil = hopr_chain_types::utils::create_anvil(Some(expected_block_time));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let chain_key_1 = ChainKeypair::from_secret(anvil.keys()[1].to_bytes().as_ref())?;

        // Deploy contracts
        let contract_instances = {
            let client = create_rpc_client_to_anvil(&anvil, &chain_key_0);
            ContractInstances::deploy_for_testing(client, &chain_key_0).await?
        };

        let tokens_minted_at =
            hopr_chain_types::utils::mint_tokens(contract_instances.token.clone(), U256::from(1000_u128))
                .await?
                .unwrap();
        debug!("tokens were minted at block {tokens_minted_at}");

        let contract_addrs = ContractAddresses::from(&contract_instances);

        let transport_client = ReqwestTransport::new(anvil.endpoint_url());

        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new(2, 100, 100))
            .transport(transport_client.clone(), transport_client.guess_local());

        let cfg = RpcOperationsConfig {
            tx_polling_interval: Duration::from_millis(10),
            contract_addrs,
            expected_block_time,
            gas_oracle_url: None,
            ..RpcOperationsConfig::default()
        };

        // Wait until contracts deployments are final
        sleep((1 + cfg.finality) * expected_block_time).await;

        let rpc = RpcOperations::new(rpc_client, transport_client.client().clone(), &chain_key_0, cfg)?;

        let log_filter = LogFilter {
            address: vec![contract_addrs.token, contract_addrs.channels],
            topics: vec![
                Hash::from(Approval::SIGNATURE_HASH.0),
                Hash::from(Transfer::SIGNATURE_HASH.0),
                Hash::from(ChannelOpened::SIGNATURE_HASH.0),
                Hash::from(ChannelBalanceIncreased::SIGNATURE_HASH.0),
            ],
        };

        debug!("{:#?}", contract_addrs);
        debug!("{:#?}", log_filter);

        // Spawn stream
        let count_filtered_topics = log_filter.topics.len();
        let retrieved_logs = spawn(async move {
            Ok::<_, RpcError>(
                rpc.try_stream_logs(1, log_filter)?
                    .skip_while(|b| futures::future::ready(b.len() != count_filtered_topics))
                    .next()
                    .await,
            )
        });

        // Spawn channel funding
        let _ = hopr_chain_types::utils::fund_channel(
            chain_key_1.public().to_address(),
            contract_instances.token,
            contract_instances.channels,
            U256::from(1_u128),
        )
        .await;

        let retrieved_logs = timeout(Duration::from_secs(30), retrieved_logs) // Give up after 30 seconds
            .await???;

        // The last block must contain all 4 events
        let last_block_logs = retrieved_logs
            .into_iter()
            .next_back()
            .context("a log should be present")?
            .clone()
            .logs;

        let channel_open_filter = ChannelOpened::SIGNATURE_HASH;
        let channel_balance_filter = ChannelBalanceIncreased::SIGNATURE_HASH;
        let approval_filter = Approval::SIGNATURE_HASH;
        let transfer_filter = Transfer::SIGNATURE_HASH;

        debug!(
            "channel_open_filter: {:?} - {:?}",
            channel_open_filter,
            channel_open_filter.0.to_vec()
        );
        debug!(
            "channel_balance_filter: {:?} - {:?}",
            channel_balance_filter,
            channel_balance_filter.0.to_vec()
        );
        debug!(
            "approval_filter: {:?} - {:?}",
            approval_filter,
            approval_filter.0.to_vec()
        );
        debug!(
            "transfer_filter: {:?} - {:?}",
            transfer_filter,
            transfer_filter.0.to_vec()
        );
        debug!("logs: {:#?}", last_block_logs);

        assert!(
            last_block_logs
                .iter()
                .any(|log| log.address == contract_addrs.channels && log.topics.contains(&channel_open_filter.into())),
            "must contain channel open"
        );
        assert!(
            last_block_logs.iter().any(
                |log| log.address == contract_addrs.channels && log.topics.contains(&channel_balance_filter.into())
            ),
            "must contain channel balance increase"
        );
        assert!(
            last_block_logs
                .iter()
                .any(|log| log.address == contract_addrs.token && log.topics.contains(&approval_filter.into())),
            "must contain token approval"
        );
        assert!(
            last_block_logs
                .iter()
                .any(|log| log.address == contract_addrs.token && log.topics.contains(&transfer_filter.into())),
            "must contain token transfer"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_try_stream_logs_should_contain_only_channel_logs_when_filtered_on_funding_channel()
    -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let expected_block_time = Duration::from_secs(1);

        let anvil = hopr_chain_types::utils::create_anvil(Some(expected_block_time));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let chain_key_1 = ChainKeypair::from_secret(anvil.keys()[1].to_bytes().as_ref())?;

        // Deploy contracts
        let contract_instances = {
            let client = create_rpc_client_to_anvil(&anvil, &chain_key_0);
            ContractInstances::deploy_for_testing(client, &chain_key_0).await?
        };

        let tokens_minted_at =
            hopr_chain_types::utils::mint_tokens(contract_instances.token.clone(), U256::from(1000_u128))
                .await?
                .unwrap();
        debug!("tokens were minted at block {tokens_minted_at}");

        let contract_addrs = ContractAddresses::from(&contract_instances);

        let cfg = RpcOperationsConfig {
            tx_polling_interval: Duration::from_millis(10),
            contract_addrs,
            expected_block_time,
            finality: 2,
            gas_oracle_url: None,
            ..RpcOperationsConfig::default()
        };

        let transport_client = ReqwestTransport::new(anvil.endpoint_url());

        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new(2, 100, 100))
            .transport(transport_client.clone(), transport_client.guess_local());

        // Wait until contracts deployments are final
        sleep((1 + cfg.finality) * expected_block_time).await;

        let rpc = RpcOperations::new(rpc_client, transport_client.client().clone(), &chain_key_0, cfg)?;

        let log_filter = LogFilter {
            address: vec![contract_addrs.channels],
            topics: vec![
                Hash::from(ChannelOpened::SIGNATURE_HASH.0),
                Hash::from(ChannelBalanceIncreased::SIGNATURE_HASH.0),
            ],
        };

        debug!("{:#?}", contract_addrs);
        debug!("{:#?}", log_filter);

        // Spawn stream
        let count_filtered_topics = log_filter.topics.len();
        let retrieved_logs = spawn(async move {
            Ok::<_, RpcError>(
                rpc.try_stream_logs(1, log_filter)?
                    .skip_while(|b| futures::future::ready(b.len() != count_filtered_topics))
                    // .next()
                    .take(1)
                    .collect::<Vec<BlockWithLogs>>()
                    .await,
            )
        });

        // Spawn channel funding
        let _ = hopr_chain_types::utils::fund_channel(
            chain_key_1.public().to_address(),
            contract_instances.token,
            contract_instances.channels,
            U256::from(1_u128),
        )
        .await;

        let retrieved_logs = timeout(Duration::from_secs(30), retrieved_logs) // Give up after 30 seconds
            .await???;

        // The last block must contain all 2 events
        let last_block_logs = retrieved_logs
            .first()
            .context("a value should be present")?
            .clone()
            .logs;

        let channel_open_filter: [u8; 32] = ChannelOpened::SIGNATURE_HASH.0;
        let channel_balance_filter: [u8; 32] = ChannelBalanceIncreased::SIGNATURE_HASH.0;

        assert!(
            last_block_logs
                .iter()
                .any(|log| log.address == contract_addrs.channels && log.topics.contains(&channel_open_filter)),
            "must contain channel open"
        );
        assert!(
            last_block_logs
                .iter()
                .any(|log| log.address == contract_addrs.channels && log.topics.contains(&channel_balance_filter)),
            "must contain channel balance increase"
        );

        Ok(())
    }
}
