use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use ethers_providers::{JsonRpcClient, Middleware};
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};

use futures::future::poll_fn;
use utils_log::{debug, error, info, warn};

use crate::{BlockWithLogs, EventsQuery, HoprIndexerRpcOperations, Log};
use crate::rpc::{HoprMiddleware, RpcOperations};
use crate::errors::RpcError::{GeneralError, NoSuchBlock};
use crate::errors::Result;

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::{sleep, spawn_local};

#[cfg(all(feature = "wasm", not(test)))]
use gloo_timers::future::sleep;

#[cfg(all(feature = "wasm", not(test)))]
use wasm_bindgen_futures::spawn_local;
use utils_types::sma::{NoSumSMA, SMA};

async fn send_block_with_logs(tx: &mut UnboundedSender<BlockWithLogs>, block: BlockWithLogs) -> Result<()> {
    match poll_fn(|cx| Pin::new(&tx).poll_ready(cx)).await {
        Ok(_) => {
            match tx.start_send(block) {
                Ok(_) => Ok(()),
                Err(_) => Err(GeneralError("failed to pass block with logs to the receiver".into()))
            }
        }
        Err(_) => Err(GeneralError("receiver has been closed".into()))
    }
}

async fn get_block_with_logs_from_provider<P: JsonRpcClient + 'static>(block_number: u64, filter: EventsQuery, provider: Arc<HoprMiddleware<P>>) -> Result<BlockWithLogs> {
    debug!("getting block {block_number} with logs");
    match provider.get_block(block_number).await? {
        Some(block) => {
            let filter: ethers::types::Filter = filter.into();

            let logs = provider.get_logs(&filter.from_block(block_number).to_block(block_number)).await?;
            Ok(BlockWithLogs {
                block: block.into(),
                logs: logs.into_iter().map(Log::from).collect()
            })
        },
        None => Err(NoSuchBlock)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<P: JsonRpcClient + 'static> HoprIndexerRpcOperations for RpcOperations<P> {
    async fn block_number(&self) -> Result<u64> {
        let r = self.provider.get_block_number().await?;
        Ok(r.as_u64())
    }

    async fn poll_blocks_with_logs(&self, start_block_number: Option<u64>, filter: EventsQuery) -> crate::errors::Result<UnboundedReceiver<BlockWithLogs>> {
        let (mut tx, rx) = futures::channel::mpsc::unbounded::<BlockWithLogs>();

        // The provider internally performs retries on timeouts and errors.
        let provider = self.provider.clone();
        let latest_block = self.block_number().await?;
        let start_block = start_block_number.unwrap_or(latest_block);
        let initial_poll_backoff = Duration::from_secs(4);

        spawn_local(async move {
            let mut current = start_block;
            let mut latest = latest_block;

            let mut block_time_sma = NoSumSMA::<Duration, u32>::new(10);
            let mut prev_block = None;

            while current < latest_block {
                match get_block_with_logs_from_provider(current, filter.clone(), provider.clone()).await {
                    Ok(block_with_logs) => {
                        let new_current = block_with_logs.block.number.expect("past block must not be pending");
                        let block_ts = Duration::from_secs(block_with_logs.block.timestamp.as_u64());
                        info!("got past block {new_current}");

                        if let Err(e) = send_block_with_logs(&mut tx, block_with_logs).await {
                            error!("failed to dispatch past block: {e}");
                            break; // Only fail if the receiver has closed
                        }

                        if let Some(prev_block_ts) = prev_block {
                            block_time_sma.add_sample( block_ts - prev_block_ts);
                        }

                        prev_block = Some(block_ts);
                        current = new_current;

                        match provider.get_block_number().await.map(|u| u.as_u64()) {
                            Ok(block_number) => {
                                latest = block_number;
                            },
                            Err(e) => {
                                error!("failed to get latest block number: {e}");
                            }
                        }
                    },
                    Err(e) => {
                        error!("failed to obtain block {current} with logs: {e}");
                    }
                }
            }
            if current >= latest {
                debug!("done retrieving past blocks, polling for new blocks > {current}");
                let mut current_backoff = block_time_sma.get_average().unwrap_or(initial_poll_backoff);
                loop {
                    match get_block_with_logs_from_provider(current + 1, filter.clone(), provider.clone()).await {
                        Ok(block_with_logs) => {
                            let new_current = block_with_logs.block.number.expect("past block must not be pending");
                            let block_ts = Duration::from_secs(block_with_logs.block.timestamp.as_u64());
                            info!("got new block {new_current}");

                            if let Err(e) = send_block_with_logs(&mut tx, block_with_logs).await {
                                error!("failed to dispatch new block: {e}");
                                break; // Only fail if the receiver has closed
                            }

                            if let Some(prev_block_ts) = prev_block {
                                block_time_sma.add_sample( block_ts - prev_block_ts);
                            }

                            prev_block = Some(block_ts);
                            current = new_current;
                            current_backoff = block_time_sma.get_average().unwrap_or(initial_poll_backoff);
                        }
                        Err(NoSuchBlock) => {
                            sleep(current_backoff.min(Duration::from_millis(100))).await;
                            current_backoff /= 2;
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

}