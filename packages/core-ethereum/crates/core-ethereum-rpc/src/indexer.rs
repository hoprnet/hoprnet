use async_trait::async_trait;
use ethers_providers::{JsonRpcClient, Middleware};
use futures::channel::mpsc::UnboundedReceiver;
use futures::future::poll_fn;
use std::pin::Pin;
use ethers::types::BlockNumber;
use futures::StreamExt;
use utils_log::{debug, error, warn};

use crate::errors::Result;
use crate::rpc::RpcOperations;
use crate::{HoprIndexerRpcOperations, Log, LogFilter};

#[cfg(all(not(feature = "wasm"), not(test)))]
use async_std::task::{sleep, spawn_local};

#[cfg(test)]
use tokio::{task::spawn_local, time::sleep};

#[cfg(all(feature = "wasm", not(test)))]
use gloo_timers::future::sleep;

#[cfg(all(feature = "wasm", not(test)))]
use wasm_bindgen_futures::spawn_local;
use crate::errors::RpcError::FilterIsEmpty;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<P: JsonRpcClient + 'static> HoprIndexerRpcOperations for RpcOperations<P> {
    async fn block_number(&self) -> Result<u64> {
        let r = self.provider.get_block_number().await?;
        Ok(r.as_u64())
    }

    async fn try_stream_logs(
        &self,
        start_block_number: Option<u64>,
        filter: LogFilter,
    ) -> Result<UnboundedReceiver<Log>> {
        if filter.is_empty() {
            return Err(FilterIsEmpty)
        }

        let (mut tx, rx) = futures::channel::mpsc::unbounded::<Log>();

        // The provider internally performs retries on timeouts and errors.
        let provider = self.provider.clone();
        let cfg_clone = self.cfg.clone();

        spawn_local(async move {
            let mut last_block = start_block_number.map(|n| BlockNumber::Number(n.into())).unwrap_or(BlockNumber::Latest);
            'poll: loop {
                let range_filter = ethers::types::Filter::from(filter.clone())
                    .from_block(last_block)
                    .to_block(BlockNumber::Latest);

                debug!("polling logs in from {last_block}");

                let mut retrieved_logs = provider.get_logs_paginated(&range_filter, cfg_clone.logs_page_size);

                while let Some(maybe_log) = retrieved_logs.next().await {
                    match maybe_log {
                        Ok(log) => {
                            match poll_fn(|cx| Pin::new(&tx).poll_ready(cx)).await {
                                Ok(_) => {
                                    let log = Log::from(log);
                                    last_block = BlockNumber::Number(log.block_number.into());
                                    debug!("retrieved {log}");

                                    if let Err(e) = tx.start_send(log) {
                                        error!("failed to pass log to the receiver: {e}");
                                        break 'poll;
                                    }
                                },
                                Err(_) => {
                                    warn!("receiver has been closed");
                                    break 'poll;
                                }
                            }
                        },
                        Err(e) => {
                            error!("failed to retrieve log: {e}")
                        }
                    }
                }

                sleep(cfg_clone.expected_block_time).await;
            }

            tx.close_channel();
            warn!("done streaming logs");
        });

        Ok(rx)
    }
}

#[cfg(test)]
mod test {
    use crate::rpc::tests::mint_tokens;
    use crate::rpc::{RpcOperations, RpcOperationsConfig};
    use crate::{HoprIndexerRpcOperations, Log, LogFilter};
    use bindings::hopr_channels::*;
    use bindings::hopr_token::{ApprovalFilter, HoprToken, TransferFilter};
    use core_crypto::keypairs::{ChainKeypair, Keypair};
    use core_ethereum_misc::{create_anvil, create_rpc_client_to_anvil, ContractAddresses, ContractInstances};
    use ethers::contract::EthEvent;
    use ethers_providers::{Http, Middleware};
    use futures::StreamExt;
    use std::str::FromStr;
    use std::time::Duration;
    use core_crypto::types::Hash;
    use utils_log::debug;
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

    /*#[tokio::test]
    async fn test_try_stream_with_logs_should_not_stream_past_blocks_if_needed() {
        let _ = env_logger::builder().is_test(true).try_init();

        let anvil = create_anvil(Some(Duration::from_secs(1)));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();

        tokio::time::sleep(Duration::from_secs(2)).await; // wait two blocks

        let cfg = RpcOperationsConfig::default();
        let rpc = RpcOperations::new(Http::from_str(&anvil.endpoint()).unwrap(), &chain_key_0, cfg)
            .expect("failed to construct rpc");

        let stream_start = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();

        let local = tokio::task::LocalSet::new();
        let blocks = local
            .run_until(async move {
                rpc.try_stream_logs(None, Default::default())
                    .await
                    .unwrap()
                    .take(2)
                    .collect::<Vec<BlockWithLogs>>()
                    .await
            })
            .await;

        assert!(blocks.iter().all(|b| b.logs.is_empty()), "must not contain logs");
        assert!(
            blocks
                .into_iter()
                .all(|b| stream_start.as_secs() <= b.block.timestamp.as_u64()),
            "must not fetch past blocks"
        );
    }

    #[tokio::test]
    async fn test_try_stream_with_logs_should_not_stream_older_blocks() {
        let _ = env_logger::builder().is_test(true).try_init();

        let anvil = create_anvil(Some(Duration::from_secs(1)));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();

        tokio::time::sleep(Duration::from_secs(2)).await; // wait two blocks

        let cfg = RpcOperationsConfig::default();
        let rpc = RpcOperations::new(Http::from_str(&anvil.endpoint()).unwrap(), &chain_key_0, cfg)
            .expect("failed to construct rpc");

        let local = tokio::task::LocalSet::new();
        let blocks = local
            .run_until(async move {
                rpc.try_block_with_logs_stream(Some(2), Default::default())
                    .await
                    .unwrap()
                    .take(2)
                    .collect::<Vec<BlockWithLogs>>()
                    .await
            })
            .await;

        assert!(blocks.iter().all(|b| b.logs.is_empty()), "must not contain logs");
        assert!(
            blocks.into_iter().all(|b| b.block.number.unwrap() >= 2),
            "must not blocks before #2"
        );
    }*/

    #[tokio::test]
    async fn test_try_stream_with_logs_should_contain_all_logs_when_opening_channel() {
        let _ = env_logger::builder().is_test(true).try_init();

        let anvil = create_anvil(Some(Duration::from_secs(1)));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();
        let chain_key_1 = ChainKeypair::from_secret(anvil.keys()[1].to_bytes().as_ref()).unwrap();

        // Deploy contracts
        let contract_instances = {
            let client = create_rpc_client_to_anvil(&anvil, &chain_key_0);
            ContractInstances::deploy_for_testing(client, &chain_key_0)
                .await
                .expect("could not deploy contracts")
        };

        let contract_addrs = ContractAddresses::from(&contract_instances);

        let cfg = RpcOperationsConfig {
            tx_polling_interval: Duration::from_millis(10),
            contract_addrs: contract_addrs.clone(),
            ..RpcOperationsConfig::default()
        };

        let rpc = RpcOperations::new(Http::from_str(&anvil.endpoint()).unwrap(), &chain_key_0, cfg)
            .expect("failed to construct rpc");

        debug!("contract addrs: {:#?}", contract_addrs);

        let log_filter = LogFilter {
            address: vec![contract_addrs.token, contract_addrs.channels],
            topics: vec![
                TransferFilter::signature().into(),
                ApprovalFilter::signature().into(),
                ChannelOpenedFilter::signature().into(),
                ChannelBalanceIncreasedFilter::signature().into(),
            ],
        };

        let local = tokio::task::LocalSet::new();

        // Spawn channel funding
        local.spawn_local(async move {
            tokio::time::sleep(Duration::from_secs(1)).await;
            fund_channel(
                chain_key_0.public().to_address(),
                chain_key_1.public().to_address(),
                contract_instances.token,
                contract_instances.channels,
            )
            .await;
        });

        let expectations = vec![
            (contract_addrs.channels, Hash::from(ChannelOpenedFilter::signature())),
            (contract_addrs.channels, Hash::from(ChannelBalanceIncreasedFilter::signature())),
            (contract_addrs.token, Hash::from(ApprovalFilter::signature())),
            (contract_addrs.token, Hash::from(TransferFilter::signature())),
        ];

        // Spawn stream
        let filter_clone = log_filter.clone();

        let run = local.run_until(async move {
            rpc
                .try_stream_logs(Some(1), filter_clone)
                .await
                .expect("must create stream")
                .filter(|log| futures::future::ready(
                    expectations
                        .iter()
                        .any(| (addr, topic) | log.address.eq(addr) && log.topics.contains(topic))
                ))
                .take(5)
                .collect::<Vec<Log>>()
                .await
        });

        // Everything must complete within 30 seconds
        let retrieved_logs = tokio::time::timeout(Duration::from_secs(30), run)
            .await
            .expect("timeout");

        for log in retrieved_logs.iter() {
            debug!("- {log}");
        }

        // The logs within the single block must contain all 4 events
        //let block_num = retrieved_logs[0].block_number;

        //assert!(retrieved_logs.iter().all(|log| log.block_number == block_num), "all logs must be within the same block");

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
