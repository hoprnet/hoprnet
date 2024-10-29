use futures::{stream, StreamExt};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, error, info, trace, warn};

use chain_rpc::{BlockWithLogs, HoprIndexerRpcOperations, LogFilter};
use chain_types::chain_events::SignificantChainEvent;
use hopr_async_runtime::prelude::{spawn, JoinHandle};
use hopr_crypto_types::types::Hash;
use hopr_db_api::logs::HoprDbLogOperations;
use hopr_db_sql::info::HoprDbInfoOperations;
use hopr_db_sql::HoprDbGeneralModelOperations;
use hopr_primitive_types::prelude::*;

use crate::{
    errors::{CoreEthereumIndexerError, Result},
    traits::ChainLogHandler,
    IndexerConfig,
};

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::SimpleGauge;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_INDEXER_CURRENT_BLOCK: SimpleGauge =
        SimpleGauge::new(
            "hopr_indexer_block_number",
            "Current last processed block number by the indexer",
    ).unwrap();
    static ref METRIC_INDEXER_CHECKSUM: SimpleGauge =
        SimpleGauge::new(
            "hopr_indexer_checksum",
            "Contains an unsigned integer that represents the low 32-bits of the Indexer checksum"
    ).unwrap();
    static ref METRIC_INDEXER_SYNC_PROGRESS: SimpleGauge =
        SimpleGauge::new(
            "hopr_indexer_sync_progress",
            " Sync progress of the historical data by the indexer",
    ).unwrap();
}

/// Indexer
///
/// Accepts the RPC operational functionality [chain_rpc::HoprIndexerRpcOperations]
/// and provides the indexing operation resulting in and output of [chain_types::chain_events::SignificantChainEvent]
/// streamed outside the indexer by the unbounded channel.
///
/// The roles of the indexer:
/// 1. prime the RPC endpoint
/// 2. request an RPC stream of changes to process
/// 3. process block and log stream
/// 4. ensure finalization by postponing processing until the head is far enough
/// 5. store relevant data into the DB
/// 6. pass the processing on to the business logic
#[derive(Debug, Clone)]
pub struct Indexer<T, U, Db>
where
    T: HoprIndexerRpcOperations + Send + 'static,
    U: ChainLogHandler + Clone + Send + 'static,
    Db: HoprDbGeneralModelOperations + Clone + Send + Sync + 'static,
{
    rpc: Option<T>,
    db_processor: Option<U>,
    db: Db,
    cfg: IndexerConfig,
    egress: async_channel::Sender<SignificantChainEvent>,
    panic_on_completion: bool,
}

impl<T, U, Db> Indexer<T, U, Db>
where
    T: HoprIndexerRpcOperations + Sync + Send + 'static,
    U: ChainLogHandler + Clone + Send + Sync + 'static,
    Db: HoprDbGeneralModelOperations + Clone + Send + Sync + 'static,
{
    pub fn new(
        rpc: T,
        db_processor: U,
        db: Db,
        cfg: IndexerConfig,
        egress: async_channel::Sender<SignificantChainEvent>,
    ) -> Self {
        Self {
            rpc: Some(rpc),
            db_processor: Some(db_processor),
            db,
            cfg,
            egress,
            panic_on_completion: true,
        }
    }

    pub fn disable_panic_on_completion(mut self) -> Self {
        self.panic_on_completion = false;
        self
    }

    pub async fn start(&mut self) -> Result<JoinHandle<()>>
    where
        T: HoprIndexerRpcOperations + 'static,
        U: ChainLogHandler + 'static,
        Db: HoprDbGeneralModelOperations + HoprDbInfoOperations + HoprDbLogOperations + 'static,
    {
        if self.rpc.is_none() || self.db_processor.is_none() {
            return Err(CoreEthereumIndexerError::ProcessError(
                "indexer cannot start, missing components".into(),
            ));
        }

        info!("Starting chain indexing");

        let rpc = self.rpc.take().expect("rpc should be present");
        let db_processor = self.db_processor.take().expect("db_processor should be present");
        let db = self.db.clone();
        let tx_significant_events = self.egress.clone();
        let panic_on_completion = self.panic_on_completion;

        // we skip on addresses which have no topics
        let mut addresses = vec![];
        let mut topics = vec![];
        db_processor.contract_addresses().iter().for_each(|address| {
            let contract_topics = db_processor.contract_address_topics(*address);
            if !contract_topics.is_empty() {
                addresses.push(*address);
                topics.extend(contract_topics);
            }
        });

        let log_filter = LogFilter {
            address: addresses,
            topics: topics.into_iter().map(Hash::from).collect(),
        };

        // First check whether fast sync is enabled and can be performed.
        // If so:
        //   1. delete the existing indexed data
        //   2. reset the fast sync progress
        //   3. run the fast sync process until completion
        //   4. finally starting the rpc indexer.
        let fast_sync_configured = self.cfg.fast_sync;
        let index_empty = self.db.index_is_empty().await?;

        match (fast_sync_configured, index_empty) {
            (true, false) => {
                warn!("Fast sync is enabled, but the index database is not empty. In order to use fast-sync again you must stop this node and remove the index database manually.");
            }
            (false, _) => {
                info!("Fast sync is disabled");
            }
            (true, true) => {
                info!("Fast sync is enabled, starting the fast sync process");
                // To ensure a proper state, we reset any auxiliary data in the database
                self.db.clear_index_db(None).await?;
                self.db.set_logs_unprocessed(None, None).await?;

                // Now fast-sync can start
                while let Some(block_number) = self.db.get_logs_block_numbers(None, None).await?.next().await {
                    Self::process_block_by_id(&db, &db_processor, block_number).await?;
                }
            }
        }

        info!("Building rpc indexer background process");
        let (tx, mut rx) = futures::channel::mpsc::channel::<()>(1);

        let next_block_to_process = if let Some(last_log) = self.db.get_last_checksummed_log().await? {
            info!(
                "Loaded indexer state at block #{0} with checksum: {1}",
                last_log.block_number,
                last_log.checksum.unwrap()
            );

            if self.cfg.start_block_number < last_log.block_number {
                // If some prior indexing took place already, avoid reprocessing
                last_log.block_number + 1
            } else {
                self.cfg.start_block_number
            }
        } else {
            self.cfg.start_block_number
        };

        info!("Indexer next block to process #{next_block_to_process}");

        let indexing_proc = spawn(async move {
            let is_synced = Arc::new(AtomicBool::new(false));
            let chain_head = Arc::new(AtomicU64::new(0));

            let event_stream = rpc
                .try_stream_logs(next_block_to_process, log_filter)
                .expect("block stream should be constructible")
                .then(|block| {
                    Self::calculate_sync_process(
                        "rpc",
                        block.clone(),
                        &rpc,
                        chain_head.clone(),
                        is_synced.clone(),
                        next_block_to_process,
                        tx.clone(),
                    )
                })
                .filter_map(|block| {
                    let db = db.clone();

                    async move {
                        debug!("Storing logs from {}", block.clone());
                        let logs = block.clone().logs;
                        let logs_vec = logs.into_iter().map(SerializableLog::from).collect();
                        match db.store_logs(logs_vec).await {
                            Ok(store_results) => {
                                if let Some(err) = store_results
                                    .into_iter()
                                    .filter(|r| r.is_err())
                                    .map(|r| r.unwrap_err())
                                    .next()
                                {
                                    error!("Failed to store logs from {block}: {err}");
                                    None
                                } else {
                                    Some(block)
                                }
                            }
                            Err(e) => {
                                error!("Failed to store logs from {block}: {e}");
                                None
                            }
                        }
                    }
                })
                .filter_map(|block| {
                    let db = db.clone();
                    let db_processor = db_processor.clone();

                    async move {
                        match Self::process_block_by_id(&db, &db_processor, block.block_id).await {
                            Ok(events) => events,
                            Err(e) => {
                                error!("Failed to process logs from {block}: {e}");
                                None
                            }
                        }
                    }
                })
                .flat_map(stream::iter);

            futures::pin_mut!(event_stream);
            while let Some(event) = event_stream.next().await {
                trace!(event=%event, "Processing onchain event");
                // Pass the events further only once we're fully synced
                if is_synced.load(Ordering::Relaxed) {
                    if let Err(e) = tx_significant_events.try_send(event) {
                        error!(error = %e,"failed to pass a significant chain event further");
                    }
                }
            }

            if panic_on_completion {
                panic!(
                    "Indexer event stream has been terminated, cannot proceed further!\n\
                    This error indicates that an issue has occurred at the RPC provider!\n\
                    The node cannot function without a good RPC connection."
                );
            }
        });

        if rx.next().await.is_some() {
            Ok(indexing_proc)
        } else {
            Err(crate::errors::CoreEthereumIndexerError::ProcessError(
                "Error during indexing start".into(),
            ))
        }
    }

    /// Processes a block by its ID.
    ///
    /// This function retrieves logs for the given block ID and processes them using the database
    /// and log handler.
    ///
    /// # Arguments
    ///
    /// * `db` - The database operations handler.
    /// * `db_processor` - The database log handler.
    /// * `block_id` - The ID of the block to process.
    ///
    /// # Returns
    ///
    /// A `Result` containing an optional vector of significant chain events if the operation succeeds or an error if it fails.
    async fn process_block_by_id(
        db: &Db,
        db_processor: &U,
        block_id: u64,
    ) -> crate::errors::Result<Option<Vec<SignificantChainEvent>>>
    where
        U: ChainLogHandler + 'static,
        Db: HoprDbLogOperations + 'static,
    {
        let mut log_stream = db.get_logs(Some(block_id), Some(0)).await?;
        let mut block = BlockWithLogs {
            block_id,
            ..Default::default()
        };

        while let Some(log) = log_stream.next().await {
            if log.block_number == block_id {
                block.logs.insert(log);
            } else {
                error!(
                    "block number mismatch in logs stream, expected {block_id} but got {}",
                    log.block_number
                )
            }
        }

        Ok(Self::process_block(db, db_processor, block).await)
    }

    /// Processes a block and its logs.
    ///
    /// This function collects events from the block logs and updates the database with the processed logs.
    ///
    /// # Arguments
    ///
    /// * `db` - The database operations handler.
    /// * `db_processor` - The database log handler.
    /// * `block` - The block with logs to process.
    ///
    /// # Returns
    ///
    /// An optional vector of significant chain events if the operation succeeds.
    async fn process_block(db: &Db, db_processor: &U, block: BlockWithLogs) -> Option<Vec<SignificantChainEvent>>
    where
        U: ChainLogHandler + 'static,
        Db: HoprDbLogOperations + 'static,
    {
        let block_id = block.block_id;
        debug!("Processing events from block #{block_id}");

        match db_processor.collect_block_events(block.clone()).await {
            Ok(events) => {
                match db.set_logs_processed(Some(block_id), Some(0)).await {
                    Ok(_) => match db.update_logs_checksums().await {
                        Ok(_) => Self::print_indexer_state(db).await,
                        Err(e) => error!("Failed to update checksums for logs from block #{block_id}: {e}"),
                    },
                    Err(e) => error!("Failed to mark logs from block #{block_id} as processed: {e}"),
                }
                info!(
                    "Processed {} significant chain events from block #{}",
                    events.len(),
                    block_id
                );
                Some(events)
            }
            Err(e) => {
                error!("Failed to process logs from block #{block_id} into events: {e}");
                None
            }
        }
    }

    /// Prints the current state of the indexer.
    ///
    /// This function retrieves and logs the last checksummed log entry and the count of logs.
    ///
    /// # Arguments
    ///
    /// * `db` - The database operations handler.
    async fn print_indexer_state(db: &Db)
    where
        Db: HoprDbLogOperations + 'static,
    {
        match db.get_last_checksummed_log().await {
            Ok(Some(log)) => match db.get_logs_count(Some(log.block_number), Some(0)).await {
                Ok(count) => {
                    let checksum = log.checksum.unwrap();
                    info!(
                        block_number = log.block_number,
                        log_count = count,
                        last_log_checksum = checksum,
                        "Indexer state update",
                    );

                    #[cfg(all(feature = "prometheus", not(test)))]
                    {
                        let checksum_hash = Hash::from_hex(checksum.as_str()).expect("Invalid checksum");
                        let low_4_bytes =
                            hopr_primitive_types::prelude::U256::from_big_endian(checksum_hash.as_ref()).low_u32();
                        METRIC_INDEXER_CHECKSUM.set(low_4_bytes.into());
                    }
                }
                Err(e) => error!("Cannot retrieve log count: {e}"),
            },
            Ok(None) => {
                debug!("No logs have been checksummed yet");
            }
            Err(e) => error!("Cannot retrieve last checksummed log: {e}"),
        }
    }

    /// Calculates the synchronization progress.
    ///
    /// This function processes a block and updates synchronization metrics and state.
    ///
    /// # Arguments
    ///
    /// * `prefix` - A string prefix for logging purposes.
    /// * `block` - The block with logs to process.
    /// * `rpc` - The RPC operations handler.
    /// * `chain_head` - The current chain head block number.
    /// * `is_synced` - A boolean indicating whether the indexer is synced.
    /// * `next_block_to_process` - The next block number to process.
    /// * `tx` - A sender channel for synchronization notifications.
    ///
    /// # Returns
    ///
    /// The block which was provided as input.
    async fn calculate_sync_process(
        prefix: &str,
        block: BlockWithLogs,
        rpc: &T,
        chain_head: Arc<AtomicU64>,
        is_synced: Arc<AtomicBool>,
        next_block_to_process: u64,
        mut tx: futures::channel::mpsc::Sender<()>,
    ) -> BlockWithLogs
    where
        T: HoprIndexerRpcOperations + 'static,
    {
        info!("Processing block number: {}", block.block_id);

        let current_block = block.block_id;
        #[cfg(all(feature = "prometheus", not(test)))]
        {
            METRIC_INDEXER_CURRENT_BLOCK.set(current_block as f64);
        }

        match rpc.block_number().await {
            Ok(current_chain_block_number) => chain_head.store(current_chain_block_number, Ordering::Relaxed),
            Err(error) => {
                error!("Failed to fetch block number from RPC, cannot continue indexing due to {error}");
                panic!("Failed to fetch block number from RPC, cannot continue indexing due to {error}")
            }
        };

        let head = chain_head.load(Ordering::Relaxed);

        if !is_synced.load(Ordering::Relaxed) {
            let block_difference = head - next_block_to_process;
            let progress = if block_difference == 0 {
                1_f64
            } else {
                (current_block - next_block_to_process) as f64 / block_difference as f64
            };

            info!(
                indexer = prefix,
                progress = progress * 100_f64,
                block = current_block,
                "Sync progress"
            );

            #[cfg(all(feature = "prometheus", not(test)))]
            METRIC_INDEXER_SYNC_PROGRESS.set(progress);

            if current_block >= head {
                info!("indexer {prefix} sync successfully completed");
                is_synced.store(true, Ordering::Relaxed);
                if let Err(e) = tx.try_send(()) {
                    error!("failed to notify about achieving indexer {prefix} synchronization: {e}")
                }
            }
        }

        block
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use ethers::{
        abi::{encode, Token},
        contract::EthEvent,
    };
    use futures::{join, Stream};
    use hex_literal::hex;
    use mockall::mock;
    use multiaddr::Multiaddr;
    use std::collections::BTreeSet;
    use std::pin::Pin;

    use bindings::hopr_announcements::AddressAnnouncementFilter;
    use chain_rpc::BlockWithLogs;
    use chain_types::chain_events::ChainEventType;
    use hopr_crypto_types::keypairs::{Keypair, OffchainKeypair};
    use hopr_crypto_types::prelude::ChainKeypair;
    use hopr_db_sql::db::HoprDb;
    use hopr_primitive_types::prelude::*;

    use crate::traits::MockChainLogHandler;

    use super::*;

    lazy_static::lazy_static! {
        static ref ALICE_KP: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).expect("lazy static keypair should be constructible");
        static ref ALICE: Address = ALICE_KP.public().to_address();
        static ref BOB: Address = hex!("3798fa65d6326d3813a0d33489ac35377f4496ef").into();
        static ref CHRIS: Address = hex!("250eefb2586ab0873befe90b905126810960ee7c").into();

        static ref RANDOM_ANNOUNCEMENT_CHAIN_EVENT: ChainEventType = ChainEventType::Announcement {
            peer: (*OffchainKeypair::from_secret(&hex!("14d2d952715a51aadbd4cc6bfac9aa9927182040da7b336d37d5bb7247aa7566")).expect("lazy static keypair should be constructible").public()).into(),
            address: hex!("2f4b7662a192b8125bbf51cfbf1bf5cc00b2c8e5").into(),
            multiaddresses: vec![Multiaddr::empty()],
        };
    }

    fn build_announcement_logs(
        address: Address,
        size: usize,
        block_number: u64,
        log_index: U256,
    ) -> anyhow::Result<Vec<SerializableLog>> {
        let mut logs: Vec<SerializableLog> = vec![];
        let block_hash = Hash::create(&[format!("my block hash {block_number}").as_bytes()]).to_hex();

        for i in 0..size {
            let test_multiaddr: Multiaddr = format!("/ip4/1.2.3.4/tcp/{}", 1000 + i).parse()?;
            logs.push(SerializableLog {
                address: address.to_hex(),
                block_hash: block_hash.clone(),
                topics: vec![format!("{:#x}", AddressAnnouncementFilter::signature())],
                data: encode(&[
                    Token::Address(ethers::abi::Address::from_slice(address.as_ref())),
                    Token::String(test_multiaddr.to_string()),
                ])
                .into(),
                tx_hash: Hash::create(&[format!("my tx hash {i}").as_bytes()]).to_hex(),
                tx_index: 0,
                block_number,
                log_index: log_index.as_u64(),
                ..Default::default()
            });
        }

        Ok(logs)
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
    async fn test_indexer_should_check_the_db_for_last_processed_block_and_supply_none_if_none_is_found(
    ) -> anyhow::Result<()> {
        let mut handlers = MockChainLogHandler::new();
        let mut rpc = MockHoprIndexerOps::new();
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

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
            async_channel::unbounded().0,
        )
        .disable_panic_on_completion();

        let (indexing, _) = join!(indexer.start(), async move {
            async_std::task::sleep(std::time::Duration::from_millis(200)).await;
            tx.close_channel()
        });
        assert!(indexing.is_err()); // terminated by the close channel

        Ok(())
    }

    #[test_log::test(async_std::test)]
    async fn test_indexer_should_check_the_db_for_last_processed_block_and_supply_it_when_found() -> anyhow::Result<()>
    {
        let mut handlers = MockChainLogHandler::new();
        let mut rpc = MockHoprIndexerOps::new();
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;
        let head_block = 1000;
        let latest_block = 15u64;

        handlers.expect_contract_addresses().return_const(vec![]);

        rpc.expect_block_number().return_once(move || Ok(head_block));

        let (tx, rx) = futures::channel::mpsc::unbounded::<BlockWithLogs>();
        rpc.expect_try_stream_logs()
            .once()
            .withf(move |x: &u64, _y: &chain_rpc::LogFilter| *x == latest_block + 1)
            .return_once(move |_, _| Ok(Box::pin(rx)));

        // insert and process latest block
        let log_1 = SerializableLog {
            address: Hash::create(&[b"my address"]).to_hex(),
            topics: [Hash::create(&[b"my topic"]).to_hex()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1u64,
            block_number: latest_block,
            block_hash: Hash::create(&[b"my block hash"]).to_hex(),
            tx_hash: Hash::create(&[b"my tx hash"]).to_hex(),
            log_index: 1u64,
            removed: false,
            processed: Some(false),
            ..Default::default()
        };
        assert!(db.store_log(log_1.clone()).await.is_ok());
        assert!(db.set_logs_processed(Some(latest_block), Some(0)).await.is_ok());
        assert!(db.update_logs_checksums().await.is_ok());

        let mut indexer = Indexer::new(
            rpc,
            handlers,
            db.clone(),
            IndexerConfig::default().disable_fast_sync(),
            async_channel::unbounded().0,
        )
        .disable_panic_on_completion();

        let (indexing, _) = join!(indexer.start(), async move {
            async_std::task::sleep(std::time::Duration::from_millis(200)).await;
            tx.close_channel()
        });
        assert!(indexing.is_err()); // terminated by the close channel

        Ok(())
    }

    #[async_std::test]
    async fn test_indexer_should_pass_blocks_that_are_finalized() -> anyhow::Result<()> {
        let mut handlers = MockChainLogHandler::new();
        let mut rpc = MockHoprIndexerOps::new();
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let cfg = IndexerConfig::default();

        handlers.expect_contract_addresses().return_const(vec![]);

        let (mut tx, rx) = futures::channel::mpsc::unbounded::<BlockWithLogs>();
        rpc.expect_try_stream_logs()
            .times(1)
            .withf(move |x: &u64, _y: &chain_rpc::LogFilter| *x == 0)
            .return_once(move |_, _| Ok(Box::pin(rx)));

        let head_block = 1000;
        rpc.expect_block_number().returning(move || Ok(head_block));

        let finalized_block = BlockWithLogs {
            block_id: head_block - 1,
            logs: BTreeSet::from_iter(build_announcement_logs(*BOB, 4, head_block - 1, U256::from(23u8))?),
        };
        let head_allowing_finalization = BlockWithLogs {
            block_id: head_block,
            logs: BTreeSet::new(),
        };

        handlers
            .expect_collect_block_events()
            .times(finalized_block.logs.len())
            .returning(|_| Ok(vec![]));

        assert!(tx.start_send(finalized_block.clone()).is_ok());
        assert!(tx.start_send(head_allowing_finalization.clone()).is_ok());

        let mut indexer =
            Indexer::new(rpc, handlers, db.clone(), cfg, async_channel::unbounded().0).disable_panic_on_completion();
        let _ = join!(indexer.start(), async move {
            async_std::task::sleep(std::time::Duration::from_millis(200)).await;
            tx.close_channel()
        });

        Ok(())
    }

    #[test_log::test(async_std::test)]
    async fn test_indexer_should_yield_back_once_the_past_events_are_indexed() -> anyhow::Result<()> {
        let mut handlers = MockChainLogHandler::new();
        let mut rpc = MockHoprIndexerOps::new();
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let cfg = IndexerConfig::default();

        // Set to be an empty list because we don't want to index anything really
        handlers.expect_contract_addresses().return_const(vec![]);

        let (mut tx, rx) = futures::channel::mpsc::unbounded::<BlockWithLogs>();
        // Expected to be called once starting at 0 and yield the respective blocks
        rpc.expect_try_stream_logs()
            .times(1)
            .withf(move |x: &u64, _y: &chain_rpc::LogFilter| *x == 0)
            .return_once(move |_, _| Ok(Box::pin(rx)));

        let head_block = 1000;
        let block_numbers = vec![head_block - 1, head_block, head_block + 1];

        let blocks: Vec<BlockWithLogs> = block_numbers
            .iter()
            .map(|block_id| BlockWithLogs {
                block_id: *block_id,
                logs: BTreeSet::from_iter(build_announcement_logs(*ALICE, 1, *block_id, U256::from(23u8)).unwrap()),
            })
            .collect();

        for _ in 0..(blocks.len() as u64) {
            rpc.expect_block_number().returning(move || Ok(head_block));
        }

        for block in blocks.iter() {
            assert!(tx.start_send(block.clone()).is_ok());
        }

        handlers.expect_clone().times(blocks.len()).returning(move || {
            let block_numbers = block_numbers.clone();
            let mut handlers2 = MockChainLogHandler::new();

            // Generate the expected events to be able to process the blocks
            handlers2.expect_contract_addresses().return_const(vec![]);
            handlers2
                .expect_collect_block_events()
                .times(1)
                .withf(move |b| block_numbers.contains(&b.block_id))
                .returning(|b| {
                    let block_id = b.block_id;
                    Ok(vec![SignificantChainEvent {
                        tx_hash: Hash::create(&[format!("my tx hash {block_id}").as_bytes()]),
                        event_type: RANDOM_ANNOUNCEMENT_CHAIN_EVENT.clone(),
                    }])
                });

            handlers2
        });

        let (tx_events, rx_events) = async_channel::unbounded();
        let mut indexer = Indexer::new(rpc, handlers, db.clone(), cfg, tx_events).disable_panic_on_completion();
        indexer.start().await?;

        // At this point we expect 2 events to arrive. The third event, which was generated first,
        // should be dropped because it was generated before the indexer was in sync with head.
        let _first = rx_events.recv();
        let _second = rx_events.recv();
        let third = rx_events.try_recv();

        assert!(third.is_err());

        Ok(())
    }

    #[test_log::test(async_std::test)]
    async fn test_indexer_should_not_reprocess_last_processed_block() -> anyhow::Result<()> {
        let last_processed_block = 100_u64;

        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        // insert and process latest block
        let log_1 = SerializableLog {
            address: Hash::create(&[b"my address"]).to_hex(),
            topics: [Hash::create(&[b"my topic"]).to_hex()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1u64,
            block_number: last_processed_block,
            block_hash: Hash::create(&[b"my block hash"]).to_hex(),
            tx_hash: Hash::create(&[b"my tx hash"]).to_hex(),
            log_index: 1u64,
            removed: false,
            processed: Some(false),
            ..Default::default()
        };
        assert!(db.store_log(log_1.clone()).await.is_ok());
        assert!(db.set_logs_processed(Some(last_processed_block), Some(0)).await.is_ok());
        assert!(db.update_logs_checksums().await.is_ok());

        let (mut tx, rx) = futures::channel::mpsc::unbounded::<BlockWithLogs>();

        let mut rpc = MockHoprIndexerOps::new();
        rpc.expect_try_stream_logs()
            .once()
            .withf(move |x: &u64, _y: &chain_rpc::LogFilter| *x == last_processed_block + 1)
            .return_once(move |_, _| Ok(Box::pin(rx)));

        rpc.expect_block_number()
            .once()
            .return_once(move || Ok(last_processed_block + 1));

        let block = BlockWithLogs {
            block_id: last_processed_block + 1,
            logs: BTreeSet::from_iter(build_announcement_logs(
                *ALICE,
                1,
                last_processed_block + 1,
                U256::from(23u8),
            )?),
        };

        tx.start_send(block)?;

        let mut handlers = MockChainLogHandler::new();
        handlers.expect_contract_addresses().return_const(vec![]);

        let (tx_events, _) = async_channel::unbounded();
        let mut indexer =
            Indexer::new(rpc, handlers, db.clone(), IndexerConfig::default(), tx_events).disable_panic_on_completion();
        indexer.start().await?;

        Ok(())
    }
}
