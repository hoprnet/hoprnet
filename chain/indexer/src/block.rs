use std::sync::Arc;

use async_lock::RwLock;
use async_std::task::spawn;
use futures::{stream, StreamExt};
use tracing::{debug, error, info, trace};

use chain_db::traits::HoprCoreEthereumDbActions;
use chain_rpc::{HoprIndexerRpcOperations, Log, LogFilter};
use chain_types::chain_events::SignificantChainEvent;
use hopr_crypto_types::types::Hash;
use hopr_primitive_types::prelude::*;

use crate::{errors::CoreEthereumIndexerError, traits::ChainLogHandler, IndexerConfig};

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::SimpleGauge;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_INDEXER_CURRENT_BLOCK: SimpleGauge =
        SimpleGauge::new(
            "hopr_indexer_block_number",
            "Current last processed block number by the indexer",
    ).unwrap();
    static ref METRIC_INDEXER_SYNC_PROGRESS: SimpleGauge =
        SimpleGauge::new(
            "hopr_indexer_sync_progress",
            " Sync progress of the historical data by the indexer",
    ).unwrap();
}

fn log_comparator(left: &Log, right: &Log) -> std::cmp::Ordering {
    let blocks = left.block_number.cmp(&right.block_number);
    if blocks == std::cmp::Ordering::Equal {
        let tx_indices = left.tx_index.cmp(&right.tx_index);
        if tx_indices == std::cmp::Ordering::Equal {
            left.log_index.cmp(&right.log_index)
        } else {
            tx_indices
        }
    } else {
        blocks
    }
}

/// Indexer
///
/// Accepts the RPC operational functionality [chain_rpc::HoprIndexerRpcOperations]
/// and provides the indexing operation resulting in and output of [chain_types::chain_events::SignificantChainEvent]
/// streamed outside of the indexer by the unbounded channel.
///
/// The roles of the indexer:
/// 1. prime the RPC endpoinnt
/// 2. request an RPC stream of changes to process
/// 3. process block and log stream
/// 4. ensure finalization by postponing processing until the head is far enough
/// 5. store relevant data into the DB
/// 6. pass the processing on to the business logic
#[derive(Debug, Clone)]
pub struct Indexer<T, U, V>
where
    T: HoprIndexerRpcOperations + Send + 'static,
    U: ChainLogHandler + Send + 'static,
    V: HoprCoreEthereumDbActions + Send + Sync + 'static,
{
    rpc: Option<T>,
    db_processor: Option<U>,
    db: Arc<RwLock<V>>,
    cfg: IndexerConfig,
    egress: futures::channel::mpsc::UnboundedSender<SignificantChainEvent>,
}

impl<T, U, V> Indexer<T, U, V>
where
    T: HoprIndexerRpcOperations + Sync + Send + 'static,
    U: ChainLogHandler + Send + Sync + 'static,
    V: HoprCoreEthereumDbActions + Send + Sync + 'static,
{
    pub fn new(
        rpc: T,
        db_processor: U,
        db: Arc<RwLock<V>>,
        cfg: IndexerConfig,
        egress: futures::channel::mpsc::UnboundedSender<SignificantChainEvent>,
    ) -> Self {
        Self {
            rpc: Some(rpc),
            db_processor: Some(db_processor),
            db,
            cfg,
            egress,
        }
    }

    pub async fn start(&mut self) -> crate::errors::Result<()>
    where
        T: HoprIndexerRpcOperations + 'static,
        U: ChainLogHandler + 'static,
        V: HoprCoreEthereumDbActions + 'static,
    {
        if self.rpc.is_none() || self.db_processor.is_none() {
            return Err(CoreEthereumIndexerError::ProcessError(
                "indexer is already started".into(),
            ));
        }

        info!("Starting indexer...");

        let rpc = self.rpc.take().expect("rpc should be present");
        let db_processor = self.db_processor.take().expect("db_processor should be present");
        let db = self.db.clone();
        let tx_significant_events = self.egress.clone();

        let db_latest_block = self.db.read().await.get_latest_block_number().await?.map(|v| v as u64);

        let latest_block_in_db = db_latest_block.unwrap_or(self.cfg.start_block_number);

        info!(
            "DB latest block: {:?}, Latest block {:?}",
            db_latest_block, latest_block_in_db
        );

        let mut topics = vec![];
        topics.extend(crate::constants::topics::announcement());
        topics.extend(crate::constants::topics::channel());
        topics.extend(crate::constants::topics::node_safe_registry());
        topics.extend(crate::constants::topics::network_registry());
        topics.extend(crate::constants::topics::ticket_price_oracle());
        if self.cfg.fetch_token_transactions {
            topics.extend(crate::constants::topics::token());
        }

        let log_filter = LogFilter {
            address: db_processor.contract_addresses(),
            topics: topics.into_iter().map(Hash::from).collect(),
        };

        info!("Building indexer background process");
        let (tx, mut rx) = futures::channel::mpsc::channel::<()>(1);

        spawn(async move {
            let is_synced = Arc::new(std::sync::atomic::AtomicBool::new(false));

            let mut chain_head = 0;

            let block_stream = rpc
                .try_stream_logs(latest_block_in_db, log_filter)
                .expect("block stream should be constructible")
                .then(|block_with_logs| {
                    let rpc = &rpc;
                    let mut tx = tx.clone();
                    let is_synced = is_synced.clone();

                    async move {
                        info!("Processed block number: {}", block_with_logs.block_id);

                        let current_block = block_with_logs.block_id;
                        #[cfg(all(feature = "prometheus", not(test)))]
                        {
                            METRIC_INDEXER_CURRENT_BLOCK.set(block_with_logs.block_id as f64);
                        }

                        match rpc.block_number().await {
                            Ok(current_chain_block_number) => {
                                chain_head = current_chain_block_number;
                            }
                            Err(error) => {
                                error!("failed to fetch block number from RPC: {error}");
                                chain_head = chain_head.max(current_block);
                            }
                        }

                        if !is_synced.load(std::sync::atomic::Ordering::Relaxed) {
                            let progress =
                                (current_block - latest_block_in_db) as f64 / (chain_head - latest_block_in_db) as f64;
                            info!("Sync progress {:.2}% @ block {}", progress * 100_f64, current_block);

                            #[cfg(all(feature = "prometheus", not(test)))]
                            METRIC_INDEXER_SYNC_PROGRESS.set(progress);

                            if current_block >= chain_head {
                                info!("Indexer sync successfully completed");
                                is_synced.store(true, std::sync::atomic::Ordering::Relaxed);
                                if let Err(e) = tx.try_send(()) {
                                    error!("failed to notify about achieving index synchronization: {e}")
                                }
                            }
                        }

                        block_with_logs
                    }
                })
                .then(|block_with_logs| async {
                    if let Err(error) = db
                        .write()
                        .await
                        .update_latest_block_number(block_with_logs.block_id as u32)
                        .await
                    {
                        error!("failed to write the latest block number into the database: {error}");
                    }

                    block_with_logs
                })
                .flat_map(|mut block_with_logs| {
                    // Assuming sorted and properly organized blocks,
                    // the following lines are just a sanity safety mechanism
                    block_with_logs.logs.sort_by(log_comparator);
                    stream::iter(block_with_logs.logs)
                })
                .then(|log| async {
                    let snapshot = Snapshot::new(
                        U256::from(log.block_number),
                        U256::from(log.tx_index), // TODO: unused, kept for ABI compatibility of DB
                        log.log_index,
                    );

                    let tx_hash = log.tx_hash;

                    db_processor
                        .on_event(log.address, log.block_number as u32, log.into(), snapshot)
                        .await
                        .map(|v| {
                            v.map(|event_type| {
                                // Pair the event type with the TX hash here
                                let significant_event = SignificantChainEvent { tx_hash, event_type };
                                debug!("indexer got {significant_event}");
                                significant_event
                            })
                        })
                        .map_err(|e| {
                            error!("failed to process logs: {e}");
                            e
                        })
                });

            futures::pin_mut!(block_stream);
            while let Some(event) = block_stream.next().await {
                trace!("Processing an onchain event: {event:?}");
                if is_synced.load(std::sync::atomic::Ordering::Relaxed) {
                    match event {
                        Ok(Some(e)) => {
                            if let Err(e) = tx_significant_events.unbounded_send(e) {
                                error!("failed to pass a significant chain event further: {e}");
                            }
                        }
                        Ok(None) => {}
                        Err(e) => {
                            error!("failed to process on-chain event into a recognizable chain event: {e}")
                        }
                    }
                }
            }
        });

        if std::future::poll_fn(|cx| futures::Stream::poll_next(std::pin::Pin::new(&mut rx), cx))
            .await
            .is_some()
        {
            Ok(())
        } else {
            Err(crate::errors::CoreEthereumIndexerError::ProcessError(
                "Error during indexing start".into(),
            ))
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::pin::Pin;

    use async_trait::async_trait;
    use bindings::hopr_announcements::AddressAnnouncementFilter;
    use chain_db::db::CoreEthereumDb;
    use chain_rpc::BlockWithLogs;
    use chain_types::chain_events::ChainEventType;
    use ethers::{
        abi::{encode, Token},
        contract::EthEvent,
    };
    use futures::{join, Stream};
    use hex_literal::hex;
    use hopr_crypto_types::keypairs::{Keypair, OffchainKeypair};
    use hopr_primitive_types::prelude::*;
    use mockall::mock;
    use multiaddr::Multiaddr;
    use utils_db::{db::DB, CurrentDbShim};

    use crate::traits::MockChainLogHandler;

    use super::*;

    lazy_static::lazy_static! {
        static ref ALICE: Address = hex!("bcc0c23fb7f4cdbdd9ff68b59456ab5613b858f8").into();
        static ref BOB: Address = hex!("3798fa65d6326d3813a0d33489ac35377f4496ef").into();
        static ref CHRIS: Address = hex!("250eefb2586ab0873befe90b905126810960ee7c").into();

        static ref RANDOM_ANNOUNCEMENT_CHAIN_EVENT: ChainEventType = ChainEventType::Announcement {
            peer: (*OffchainKeypair::from_secret(&hex!("14d2d952715a51aadbd4cc6bfac9aa9927182040da7b336d37d5bb7247aa7566")).unwrap().public()).into(),
            address: hex!("2f4b7662a192b8125bbf51cfbf1bf5cc00b2c8e5").into(),
            multiaddresses: vec![Multiaddr::empty()],
        };
    }

    async fn create_stub_db() -> Arc<RwLock<CoreEthereumDb<CurrentDbShim>>> {
        Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
            *ALICE,
        )))
    }

    fn build_announcement_logs(address: Address, size: usize, block_number: u64, log_index: U256) -> Vec<Log> {
        let mut logs: Vec<Log> = vec![];

        for i in 0..size {
            let test_multiaddr: Multiaddr = format!("/ip4/1.2.3.4/tcp/{}", 1000 + i).parse().unwrap();
            logs.push(Log {
                address,
                topics: vec![AddressAnnouncementFilter::signature().into()],
                data: encode(&[
                    Token::Address(ethers::abi::Address::from_slice(&address.to_bytes())),
                    Token::String(test_multiaddr.to_string()),
                ])
                .into(),
                tx_hash: Default::default(),
                tx_index: 0,
                block_number,
                log_index,
            });
        }

        logs
    }

    mock! {
        HoprIndexerOps {}     // Name of the mock struct, less the "Mock" prefix

        #[async_trait]
        impl HoprIndexerRpcOperations for HoprIndexerOps {
            async fn block_number(&self) -> chain_rpc::errors::Result<u64>;

            fn try_stream_logs<'a>(
                &'a self,
                start_block_number: u64,
                filter: LogFilter,
            ) -> chain_rpc::errors::Result<Pin<Box<dyn Stream<Item = BlockWithLogs> + Send + 'a>>>;
        }
    }

    #[async_std::test]
    async fn test_indexer_should_check_the_db_for_last_processed_block_and_supply_none_if_none_is_found() {
        let mut handlers = MockChainLogHandler::new();
        let mut rpc = MockHoprIndexerOps::new();
        let db = create_stub_db().await;

        handlers.expect_contract_addresses().return_const(vec![]);

        let head_block = 1000;
        rpc.expect_block_number().return_once(move || Ok(head_block));

        let (tx, rx) = futures::channel::mpsc::unbounded::<BlockWithLogs>();
        rpc.expect_try_stream_logs()
            .withf(move |x: &u64, _y: &chain_rpc::LogFilter| *x == 0)
            .return_once(move |_, _| Ok(Box::pin(rx)));

        let mut indexer = Indexer::new(
            rpc,
            handlers,
            db.clone(),
            IndexerConfig::default(),
            futures::channel::mpsc::unbounded().0,
        );
        let (indexing, _) = join!(indexer.start(), async move {
            async_std::task::sleep(std::time::Duration::from_millis(200)).await;
            tx.close_channel()
        });
        assert!(indexing.is_err()) // terminated by the close channel
    }

    #[async_std::test]
    async fn test_indexer_should_check_the_db_for_last_processed_block_and_supply_it_when_found() {
        let mut handlers = MockChainLogHandler::new();
        let mut rpc = MockHoprIndexerOps::new();
        let db = create_stub_db().await;

        handlers.expect_contract_addresses().return_const(vec![]);

        let head_block = 1000;
        let latest_block = 15u64;
        assert!(db
            .write()
            .await
            .update_latest_block_number(latest_block as u32)
            .await
            .is_ok());
        rpc.expect_block_number().return_once(move || Ok(head_block));

        let (tx, rx) = futures::channel::mpsc::unbounded::<BlockWithLogs>();
        rpc.expect_try_stream_logs()
            .withf(move |x: &u64, _y: &chain_rpc::LogFilter| *x == latest_block)
            .return_once(move |_, _| Ok(Box::pin(rx)));

        let mut indexer = Indexer::new(
            rpc,
            handlers,
            db.clone(),
            IndexerConfig::default(),
            futures::channel::mpsc::unbounded().0,
        );
        let (indexing, _) = join!(indexer.start(), async move {
            async_std::task::sleep(std::time::Duration::from_millis(200)).await;
            tx.close_channel()
        });
        assert!(indexing.is_err()) // terminated by the close channel
    }

    #[async_std::test]
    async fn test_indexer_should_not_pass_blocks_unless_finalized() {
        let mut handlers = MockChainLogHandler::new();
        let mut rpc = MockHoprIndexerOps::new();
        let db = create_stub_db().await;

        handlers.expect_contract_addresses().return_const(vec![]);

        let head_block = 1000;
        rpc.expect_block_number().return_once(move || Ok(head_block));

        let (mut tx, rx) = futures::channel::mpsc::unbounded::<BlockWithLogs>();
        rpc.expect_try_stream_logs()
            .times(1)
            .withf(move |x: &u64, _y: &chain_rpc::LogFilter| *x == 0)
            .return_once(move |_, _| Ok(Box::pin(rx)));

        let expected = BlockWithLogs {
            block_id: head_block - 1,
            logs: vec![],
        };

        handlers.expect_on_event().times(0);

        assert!(tx.start_send(expected.clone()).is_ok());

        let mut indexer = Indexer::new(
            rpc,
            handlers,
            db.clone(),
            IndexerConfig::default(),
            futures::channel::mpsc::unbounded().0,
        );
        let _ = join!(indexer.start(), async move {
            async_std::task::sleep(std::time::Duration::from_millis(200)).await;
            tx.close_channel()
        });
    }

    #[async_std::test]
    async fn test_indexer_should_pass_blocks_that_are_finalized() {
        let mut handlers = MockChainLogHandler::new();
        let mut rpc = MockHoprIndexerOps::new();
        let db = create_stub_db().await;

        let cfg = IndexerConfig::default();

        handlers.expect_contract_addresses().return_const(vec![]);

        let (mut tx, rx) = futures::channel::mpsc::unbounded::<BlockWithLogs>();
        rpc.expect_try_stream_logs()
            .times(1)
            .withf(move |x: &u64, _y: &chain_rpc::LogFilter| *x == 0)
            .return_once(move |_, _| Ok(Box::pin(rx)));

        let head_block = 1000;
        rpc.expect_block_number().returning(move || Ok(head_block));
        rpc.expect_block_number().returning(move || Ok(head_block));

        let finalized_block = BlockWithLogs {
            block_id: head_block - 1,
            logs: build_announcement_logs(*BOB, 4, head_block - 1, U256::from(23u8)),
        };
        let head_allowing_finalization = BlockWithLogs {
            block_id: head_block,
            logs: vec![],
        };

        handlers
            .expect_on_event()
            .times(finalized_block.logs.len())
            .returning(|_, _, _, _| Ok(None));

        assert!(tx.start_send(finalized_block.clone()).is_ok());
        assert!(tx.start_send(head_allowing_finalization.clone()).is_ok());

        let mut indexer = Indexer::new(rpc, handlers, db.clone(), cfg, futures::channel::mpsc::unbounded().0);
        let _ = join!(indexer.start(), async move {
            async_std::task::sleep(std::time::Duration::from_millis(200)).await;
            tx.close_channel()
        });
    }

    #[async_std::test]
    async fn test_indexer_should_yield_back_once_the_past_events_are_indexed() {
        let mut handlers = MockChainLogHandler::new();
        let mut rpc = MockHoprIndexerOps::new();
        let db = create_stub_db().await;

        let cfg = IndexerConfig::default();

        handlers.expect_contract_addresses().return_const(vec![]);

        let (mut tx, rx) = futures::channel::mpsc::unbounded::<BlockWithLogs>();
        rpc.expect_try_stream_logs()
            .times(1)
            .withf(move |x: &u64, _y: &chain_rpc::LogFilter| *x == 0)
            .return_once(move |_, _| Ok(Box::pin(rx)));

        let head_block = 1000;

        let blocks = vec![
            // head - 1 sync block
            BlockWithLogs {
                block_id: head_block - 1,
                logs: build_announcement_logs(*ALICE, 1, head_block - 1, U256::from(23u8)),
            },
            // head sync block
            BlockWithLogs {
                block_id: head_block,
                logs: build_announcement_logs(*BOB, 1, head_block, U256::from(23u8)),
            },
            // post-sync block
            BlockWithLogs {
                block_id: head_block,
                logs: build_announcement_logs(*CHRIS, 1, head_block, U256::from(23u8)),
            },
        ];

        for _ in 0..(blocks.len() as u64) {
            rpc.expect_block_number().returning(move || Ok(head_block));
        }

        handlers
            .expect_on_event()
            .times(blocks.len())
            .returning(|_, _, _, _| Ok(Some(RANDOM_ANNOUNCEMENT_CHAIN_EVENT.clone())));

        for block in blocks.iter() {
            assert!(tx.start_send(block.clone()).is_ok());
        }

        let (tx_events, rx_events) = futures::channel::mpsc::unbounded();
        let mut indexer = Indexer::new(rpc, handlers, db.clone(), cfg, tx_events);
        assert!(indexer.start().await.is_ok());

        // tx.close_channel();

        let received = async_std::future::timeout(
            std::time::Duration::from_millis(500),
            rx_events.take(1).collect::<Vec<_>>(),
        )
        .await;

        assert!(received.is_ok());
        assert_eq!(received.unwrap().len(), 1)
    }
}
