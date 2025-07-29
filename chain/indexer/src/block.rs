use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

use alloy::sol_types::SolEvent;
use futures::{
    StreamExt,
    future::AbortHandle,
    stream::{self},
};
use hopr_bindings::hoprtoken::HoprToken::{Approval, Transfer};
use hopr_chain_rpc::{BlockWithLogs, FilterSet, HoprIndexerRpcOperations};
use hopr_chain_types::chain_events::SignificantChainEvent;
use hopr_crypto_types::types::Hash;
use hopr_db_api::logs::HoprDbLogOperations;
use hopr_db_sql::{HoprDbGeneralModelOperations, info::HoprDbInfoOperations};
use hopr_primitive_types::prelude::*;
use tracing::{debug, error, info, trace};

use crate::{
    IndexerConfig,
    errors::{CoreEthereumIndexerError, Result},
    snapshot::{SnapshotInfo, SnapshotManager},
    traits::ChainLogHandler,
};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_INDEXER_CURRENT_BLOCK: hopr_metrics::metrics::SimpleGauge =
        hopr_metrics::metrics::SimpleGauge::new(
            "hopr_indexer_block_number",
            "Current last processed block number by the indexer",
    ).unwrap();
    static ref METRIC_INDEXER_CHECKSUM: hopr_metrics::metrics::SimpleGauge =
        hopr_metrics::metrics::SimpleGauge::new(
            "hopr_indexer_checksum",
            "Contains an unsigned integer that represents the low 32-bits of the Indexer checksum"
    ).unwrap();
    static ref METRIC_INDEXER_SYNC_PROGRESS: hopr_metrics::metrics::SimpleGauge =
        hopr_metrics::metrics::SimpleGauge::new(
            "hopr_indexer_sync_progress",
            "Sync progress of the historical data by the indexer",
    ).unwrap();
    static ref METRIC_INDEXER_SYNC_SOURCE: hopr_metrics::metrics::MultiGauge =
        hopr_metrics::metrics::MultiGauge::new(
            "hopr_indexer_data_source",
            "Current data source of the Indexer",
            &["source"],
    ).unwrap();

}

/// Indexer
///
/// Accepts the RPC operational functionality [hopr_chain_rpc::HoprIndexerRpcOperations]
/// and provides the indexing operation resulting in and output of
/// [hopr_chain_types::chain_events::SignificantChainEvent] streamed outside the indexer by the unbounded channel.
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
    U: ChainLogHandler + Send + 'static,
    Db: HoprDbGeneralModelOperations + HoprDbInfoOperations + HoprDbLogOperations + Clone + Send + Sync + 'static,
{
    rpc: Option<T>,
    db_processor: Option<U>,
    db: Db,
    cfg: IndexerConfig,
    egress: async_channel::Sender<SignificantChainEvent>,
    // If true (default), the indexer will panic if the event stream is terminated.
    // Setting it to false is useful for testing.
    panic_on_completion: bool,
}

impl<T, U, Db> Indexer<T, U, Db>
where
    T: HoprIndexerRpcOperations + Sync + Send + 'static,
    U: ChainLogHandler + Send + Sync + 'static,
    Db: HoprDbGeneralModelOperations + HoprDbInfoOperations + HoprDbLogOperations + Clone + Send + Sync + 'static,
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

    /// Disables the panic on completion.
    pub fn without_panic_on_completion(mut self) -> Self {
        self.panic_on_completion = false;
        self
    }

    pub async fn start(mut self) -> Result<AbortHandle>
    where
        T: HoprIndexerRpcOperations + 'static,
        U: ChainLogHandler + 'static,
        Db: HoprDbGeneralModelOperations + HoprDbInfoOperations + HoprDbLogOperations + Clone + Send + Sync + 'static,
    {
        if self.rpc.is_none() || self.db_processor.is_none() {
            return Err(CoreEthereumIndexerError::ProcessError(
                "indexer cannot start, missing components".into(),
            ));
        }

        info!("Starting chain indexing");

        let rpc = self.rpc.take().expect("rpc should be present");
        let logs_handler = Arc::new(self.db_processor.take().expect("db_processor should be present"));
        let db = self.db.clone();
        let tx_significant_events = self.egress.clone();
        let panic_on_completion = self.panic_on_completion;

        let (log_filters, address_topics) = Self::generate_log_filters(&logs_handler);

        // Check that the contract addresses and topics are consistent with what is in the logs DB,
        // or if the DB is empty, prime it with the given addresses and topics.
        db.ensure_logs_origin(address_topics).await?;

        let is_synced = Arc::new(AtomicBool::new(false));
        let chain_head = Arc::new(AtomicU64::new(0));

        // update the chain head once at startup to get a reference for initial syncing
        // progress calculation
        debug!("Updating chain head at indexer startup");
        Self::update_chain_head(&rpc, chain_head.clone()).await;

        // First, check whether fast sync is enabled and can be performed.
        // If so:
        //   1. Download the snapshot if the logs database is empty and the snapshot is enabled
        //   2. Delete the existing indexed data
        //   3. Reset fast sync progress
        //   4. Run the fast sync process until completion
        //   5. Finally, starting the rpc indexer.
        let fast_sync_configured = self.cfg.fast_sync;
        let index_empty = self.db.index_is_empty().await?;

        // Pre-start operations to ensure the indexer is ready, including snapshot fetching
        self.pre_start().await?;

        #[derive(PartialEq, Eq)]
        enum FastSyncMode {
            None,
            FromScratch,
            Continue,
        }

        let will_perform_fast_sync = match (fast_sync_configured, index_empty) {
            (true, false) => {
                info!(
                    "Fast sync is enabled, but the index database is not empty. Fast sync will continue on existing \
                     unprocessed logs."
                );
                FastSyncMode::Continue
            }
            (false, true) => {
                info!("Fast sync is disabled, but the index database is empty. Doing a full re-sync.");
                // Clean the last processed log from the Log DB, to allow full resync
                self.db.clear_index_db(None).await?;
                self.db.set_logs_unprocessed(None, None).await?;
                FastSyncMode::None
            }
            (false, false) => {
                info!("Fast sync is disabled and the index database is not empty. Continuing normal sync.");
                FastSyncMode::None
            }
            (true, true) => {
                info!("Fast sync is enabled, starting the fast sync process");
                // To ensure a proper state, reset any auxiliary data in the database
                self.db.clear_index_db(None).await?;
                self.db.set_logs_unprocessed(None, None).await?;
                FastSyncMode::FromScratch
            }
        };

        let (tx, mut rx) = futures::channel::mpsc::channel::<()>(1);

        // Perform the fast-sync if requested
        if FastSyncMode::None != will_perform_fast_sync {
            let processed = match will_perform_fast_sync {
                FastSyncMode::FromScratch => None,
                FastSyncMode::Continue => Some(false),
                _ => unreachable!(),
            };

            #[cfg(all(feature = "prometheus", not(test)))]
            {
                METRIC_INDEXER_SYNC_SOURCE.set(&["fast-sync"], 1.0);
                METRIC_INDEXER_SYNC_SOURCE.set(&["rpc"], 0.0);
            }

            let log_block_numbers = self.db.get_logs_block_numbers(None, None, processed).await?;
            let _first_log_block_number = log_block_numbers.first().copied().unwrap_or(0);
            let _head = chain_head.load(Ordering::Relaxed);
            for block_number in log_block_numbers {
                debug!(
                    block_number,
                    first_log_block_number = _first_log_block_number,
                    head = _head,
                    "computing processed logs"
                );
                // Do not pollute the logs with the fast-sync progress
                Self::process_block_by_id(&db, &logs_handler, block_number, is_synced.load(Ordering::Relaxed)).await?;

                #[cfg(all(feature = "prometheus", not(test)))]
                {
                    let progress =
                        (block_number - _first_log_block_number) as f64 / (_head - _first_log_block_number) as f64;
                    METRIC_INDEXER_SYNC_PROGRESS.set(progress);
                }
            }
        }

        info!("Building rpc indexer background process");

        let next_block_to_process = if let Some(last_log) = self.db.get_last_checksummed_log().await? {
            info!(
                start_block = last_log.block_number,
                start_checksum = last_log.checksum.unwrap(),
                "Loaded indexer state",
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

        info!(next_block_to_process, "Indexer start point");

        let indexing_abort_handle = hopr_async_runtime::spawn_as_abortable!(async move {
            // Update the chain head once again
            debug!("Updating chain head at indexer startup");
            Self::update_chain_head(&rpc, chain_head.clone()).await;

            #[cfg(all(feature = "prometheus", not(test)))]
            {
                METRIC_INDEXER_SYNC_SOURCE.set(&["fast-sync"], 0.0);
                METRIC_INDEXER_SYNC_SOURCE.set(&["rpc"], 1.0);
            }

            let rpc_ref = &rpc;

            let event_stream = rpc
                .try_stream_logs(next_block_to_process, log_filters, is_synced.load(Ordering::Relaxed))
                .expect("block stream should be constructible")
                .then(|block| {
                    let db = db.clone();
                    let chain_head = chain_head.clone();
                    let is_synced = is_synced.clone();
                    let tx = tx.clone();
                    let logs_handler = logs_handler.clone();

                    async move {
                        Self::calculate_sync_process(
                            block.block_id,
                            rpc_ref,
                            db,
                            chain_head.clone(),
                            is_synced.clone(),
                            next_block_to_process,
                            tx.clone(),
                            logs_handler.safe_address().into(),
                            logs_handler.contract_addresses_map().channels.into(),
                        )
                        .await;

                        block
                    }
                })
                .filter_map(|block| {
                    let db = db.clone();
                    let logs_handler = logs_handler.clone();

                    async move {
                        debug!(%block, "storing logs from block");
                        let logs = block.logs.clone();

                        // Filter out the token contract logs because we do not need to store these
                        // in the database.
                        let logs_vec = logs
                            .into_iter()
                            .filter(|log| log.address != logs_handler.contract_addresses_map().token)
                            .collect();

                        match db.store_logs(logs_vec).await {
                            Ok(store_results) => {
                                if let Some(error) = store_results
                                    .into_iter()
                                    .filter(|r| r.is_err())
                                    .map(|r| r.unwrap_err())
                                    .next()
                                {
                                    error!(%block, %error, "failed to processed stored logs from block");
                                    None
                                } else {
                                    Some(block)
                                }
                            }
                            Err(error) => {
                                error!(%block, %error, "failed to store logs from block");
                                None
                            }
                        }
                    }
                })
                .filter_map(|block| {
                    let db = db.clone();
                    let logs_handler = logs_handler.clone();
                    let is_synced = is_synced.clone();
                    async move {
                        Self::process_block(&db, &logs_handler, block, false, is_synced.load(Ordering::Relaxed)).await
                    }
                })
                .flat_map(stream::iter);

            futures::pin_mut!(event_stream);
            while let Some(event) = event_stream.next().await {
                trace!(%event, "processing on-chain event");
                // Pass the events further only once we're fully synced
                if is_synced.load(Ordering::Relaxed) {
                    if let Err(error) = tx_significant_events.try_send(event) {
                        error!(%error, "failed to pass a significant chain event further");
                    }
                }
            }

            if panic_on_completion {
                panic!(
                    "Indexer event stream has been terminated. This error may be caused by a failed RPC connection."
                );
            }
        });

        if rx.next().await.is_some() {
            Ok(indexing_abort_handle)
        } else {
            Err(crate::errors::CoreEthereumIndexerError::ProcessError(
                "Error during indexing start".into(),
            ))
        }
    }

    pub async fn pre_start(&self) -> Result<()> {
        let fast_sync_configured = self.cfg.fast_sync;
        let index_empty = self.db.index_is_empty().await?;

        // Check if we need to download snapshot before fast sync
        let logs_db_has_data = self.has_logs_data().await?;

        if fast_sync_configured && index_empty && !logs_db_has_data && !self.cfg.logs_snapshot_url.is_empty() {
            info!("Logs database is empty, attempting to download logs snapshot...");

            match self.download_snapshot().await {
                Ok(snapshot_info) => {
                    info!("Logs snapshot downloaded successfully: {:?}", snapshot_info);
                }
                Err(e) => {
                    error!("Failed to download logs snapshot: {}. Continuing with regular sync.", e);
                }
            }
        }
        Ok(())
    }

    /// Generates specialized log filters for efficient blockchain event processing.
    ///
    /// This function creates a comprehensive set of log filters that optimize
    /// indexer performance by categorizing filters based on contract types and
    /// event relevance to the specific node.
    ///
    /// # Arguments
    /// * `logs_handler` - Handler containing contract addresses and safe address
    ///
    /// # Returns
    /// * `(FilterSet, Vec<(Address, Hash)>)` - A tuple containing:
    ///   - `FilterSet`: Categorized filters for blockchain event processing.
    ///   - `Vec<(Address, Hash)>`: A vector of address-topic pairs for logs origin validation.
    ///
    /// # Filter Categories
    /// * `all` - Complete set of filters for normal operation
    /// * `token` - Token-specific filters (Transfer, Approval events for safe)
    /// * `no_token` - Non-token contract filters for initial sync optimization
    fn generate_log_filters(logs_handler: &U) -> (FilterSet, Vec<(Address, Hash)>) {
        let safe_address = logs_handler.safe_address();
        let addresses_no_token = logs_handler
            .contract_addresses()
            .into_iter()
            .filter(|a| *a != logs_handler.contract_addresses_map().token)
            .collect::<Vec<_>>();
        let mut filter_base_addresses = vec![];
        let mut filter_base_topics = vec![];
        let mut address_topics = vec![];

        addresses_no_token.iter().for_each(|address| {
            let topics = logs_handler.contract_address_topics(*address);
            if !topics.is_empty() {
                filter_base_addresses.push(alloy::primitives::Address::from(*address));
                filter_base_topics.extend(topics.clone());
                for topic in topics.iter() {
                    address_topics.push((*address, Hash::from(topic.0)))
                }
            }
        });

        let filter_base = alloy::rpc::types::Filter::new()
            .address(filter_base_addresses)
            .event_signature(filter_base_topics);
        let filter_token = alloy::rpc::types::Filter::new().address(alloy::primitives::Address::from(
            logs_handler.contract_addresses_map().token,
        ));

        let filter_transfer_to = filter_token
            .clone()
            .event_signature(Transfer::SIGNATURE_HASH)
            .topic2(alloy::primitives::B256::from_slice(safe_address.to_bytes32().as_ref()));

        let filter_transfer_from = filter_token
            .clone()
            .event_signature(Transfer::SIGNATURE_HASH)
            .topic1(alloy::primitives::B256::from_slice(safe_address.to_bytes32().as_ref()));

        let filter_approval = filter_token
            .event_signature(Approval::SIGNATURE_HASH)
            .topic1(alloy::primitives::B256::from_slice(safe_address.to_bytes32().as_ref()))
            .topic2(alloy::primitives::B256::from_slice(
                logs_handler.contract_addresses_map().channels.to_bytes32().as_ref(),
            ));

        let set = FilterSet {
            all: vec![
                filter_base.clone(),
                filter_transfer_from.clone(),
                filter_transfer_to.clone(),
                filter_approval.clone(),
            ],
            token: vec![filter_transfer_from, filter_transfer_to, filter_approval],
            no_token: vec![filter_base],
        };

        (set, address_topics)
    }

    /// Processes a block by its ID.
    ///
    /// This function retrieves logs for the given block ID and processes them using the database
    /// and log handler.
    ///
    /// # Arguments
    ///
    /// * `db` - The database operations handler.
    /// * `logs_handler` - The database log handler.
    /// * `block_id` - The ID of the block to process.
    ///
    /// # Returns
    ///
    /// A `Result` containing an optional vector of significant chain events if the operation succeeds or an error if it
    /// fails.
    async fn process_block_by_id(
        db: &Db,
        logs_handler: &U,
        block_id: u64,
        is_synced: bool,
    ) -> crate::errors::Result<Option<Vec<SignificantChainEvent>>>
    where
        U: ChainLogHandler + 'static,
        Db: HoprDbLogOperations + 'static,
    {
        let logs = db.get_logs(Some(block_id), Some(0)).await?;
        let mut block = BlockWithLogs {
            block_id,
            ..Default::default()
        };

        for log in logs {
            if log.block_number == block_id {
                block.logs.insert(log);
            } else {
                error!(
                    expected = block_id,
                    actual = log.block_number,
                    "block number mismatch in logs from database"
                );
                panic!("block number mismatch in logs from database")
            }
        }

        Ok(Self::process_block(db, logs_handler, block, true, is_synced).await)
    }

    /// Processes a block and its logs.
    ///
    /// This function collects events from the block logs and updates the database with the processed logs.
    ///
    /// # Arguments
    ///
    /// * `db` - The database operations handler.
    /// * `logs_handler` - The database log handler.
    /// * `block` - The block with logs to process.
    /// * `fetch_checksum_from_db` - A boolean indicating whether to fetch the checksum from the database.
    ///
    /// # Returns
    ///
    /// An optional vector of significant chain events if the operation succeeds.
    async fn process_block(
        db: &Db,
        logs_handler: &U,
        block: BlockWithLogs,
        fetch_checksum_from_db: bool,
        is_synced: bool,
    ) -> Option<Vec<SignificantChainEvent>>
    where
        U: ChainLogHandler + 'static,
        Db: HoprDbLogOperations + 'static,
    {
        let block_id = block.block_id;
        let log_count = block.logs.len();
        debug!(block_id, "processing events");

        // FIXME: The block indexing and marking as processed should be done in a single
        // transaction. This is difficult since currently this would be across databases.
        let events = stream::iter(block.logs.clone())
            .filter_map(|log| async move {
                match logs_handler.collect_log_event(log.clone(), is_synced).await {
                    Ok(data) => match db.set_log_processed(log).await {
                        Ok(_) => data,
                        Err(error) => {
                            error!(block_id, %error, "failed to mark log as processed, panicking to prevent data loss");
                            panic!("failed to mark log as processed, panicking to prevent data loss")
                        }
                    },
                    Err(error) => {
                        error!(block_id, %error, "failed to process log into event, panicking to prevent data loss");
                        panic!("failed to process log into event, panicking to prevent data loss")
                    }
                }
            })
            .collect::<Vec<SignificantChainEvent>>()
            .await;

        // if we made it this far, no errors occurred and we can update checksums and indexer state
        match db.update_logs_checksums().await {
            Ok(last_log_checksum) => {
                let checksum = if fetch_checksum_from_db {
                    let last_log = block.logs.into_iter().next_back()?;
                    let log = db.get_log(block_id, last_log.tx_index, last_log.log_index).await.ok()?;

                    log.checksum?
                } else {
                    last_log_checksum.to_string()
                };

                if log_count != 0 {
                    info!(
                        block_number = block_id,
                        log_count, last_log_checksum = ?checksum, "Indexer state update",
                    );

                    #[cfg(all(feature = "prometheus", not(test)))]
                    {
                        if let Ok(checksum_hash) = Hash::from_hex(checksum.as_str()) {
                            let low_4_bytes =
                                hopr_primitive_types::prelude::U256::from_big_endian(checksum_hash.as_ref()).low_u32();
                            METRIC_INDEXER_CHECKSUM.set(low_4_bytes.into());
                        } else {
                            error!("Invalid checksum generated from logs");
                        }
                    }
                }

                // finally update the block number in the database to the last processed block
                match db.set_indexer_state_info(None, block_id as u32).await {
                    Ok(_) => {
                        trace!(block_id, "updated indexer state info");
                    }
                    Err(error) => error!(block_id, %error, "failed to update indexer state info"),
                }
            }
            Err(error) => error!(block_id, %error, "failed to update checksums for logs from block"),
        }

        debug!(
            block_id,
            num_events = events.len(),
            "processed significant chain events from block",
        );

        Some(events)
    }

    async fn update_chain_head(rpc: &T, chain_head: Arc<AtomicU64>) -> u64
    where
        T: HoprIndexerRpcOperations + 'static,
    {
        match rpc.block_number().await {
            Ok(head) => {
                chain_head.store(head, Ordering::Relaxed);
                debug!(head, "Updated chain head");
                head
            }
            Err(error) => {
                error!(%error, "Failed to fetch block number from RPC");
                panic!("Failed to fetch block number from RPC, cannot continue indexing due to {error}")
            }
        }
    }

    /// Calculates the synchronization progress.
    ///
    /// This function processes a block and updates synchronization metrics and state.
    ///
    /// # Arguments
    ///
    /// * `block` - The block with logs to process.
    /// * `rpc` - The RPC operations handler.
    /// * `chain_head` - The current chain head block number.
    /// * `is_synced` - A boolean indicating whether the indexer is synced.
    /// * `start_block` - The first block number to process.
    /// * `tx` - A sender channel for synchronization notifications.
    ///
    /// # Returns
    ///
    /// The block which was provided as input.
    #[allow(clippy::too_many_arguments)]
    async fn calculate_sync_process(
        current_block: u64,
        rpc: &T,
        db: Db,
        chain_head: Arc<AtomicU64>,
        is_synced: Arc<AtomicBool>,
        next_block_to_process: u64,
        mut tx: futures::channel::mpsc::Sender<()>,
        safe_address: Option<Address>,
        channels_address: Option<Address>,
    ) where
        T: HoprIndexerRpcOperations + 'static,
        Db: HoprDbInfoOperations + Clone + Send + Sync + 'static,
    {
        #[cfg(all(feature = "prometheus", not(test)))]
        {
            METRIC_INDEXER_CURRENT_BLOCK.set(current_block as f64);
        }

        let mut head = chain_head.load(Ordering::Relaxed);

        // We only print out sync progress if we are not yet synced.
        // Once synced, we don't print out progress anymore.
        if !is_synced.load(Ordering::Relaxed) {
            let mut block_difference = head.saturating_sub(next_block_to_process);

            let progress = if block_difference == 0 {
                // Before we call the sync complete, we check the chain again.
                head = Self::update_chain_head(rpc, chain_head.clone()).await;
                block_difference = head.saturating_sub(next_block_to_process);

                if block_difference == 0 {
                    1_f64
                } else {
                    (current_block - next_block_to_process) as f64 / block_difference as f64
                }
            } else {
                (current_block - next_block_to_process) as f64 / block_difference as f64
            };

            info!(
                progress = progress * 100_f64,
                block = current_block,
                head,
                "Sync progress to last known head"
            );

            #[cfg(all(feature = "prometheus", not(test)))]
            METRIC_INDEXER_SYNC_PROGRESS.set(progress);

            if current_block >= head {
                info!("indexer sync completed successfully");
                is_synced.store(true, Ordering::Relaxed);

                if let Some(safe_address) = safe_address {
                    info!("updating safe balance from chain after indexer sync completed");
                    match rpc.get_hopr_balance(safe_address).await {
                        Ok(balance) => {
                            if let Err(error) = db.set_safe_hopr_balance(None, balance).await {
                                error!(%error, "failed to update safe balance from chain after indexer sync completed");
                            }
                        }
                        Err(error) => {
                            error!(%error, "failed to fetch safe balance from chain after indexer sync completed");
                        }
                    }
                }

                if let Some((channels_address, safe_address)) = channels_address.zip(safe_address) {
                    info!("updating safe allowance from chain after indexer sync completed");
                    match rpc.get_hopr_allowance(safe_address, channels_address).await {
                        Ok(allowance) => {
                            if let Err(error) = db.set_safe_hopr_allowance(None, allowance).await {
                                error!(%error, "failed to update safe allowance from chain after indexer sync completed");
                            }
                        }
                        Err(error) => {
                            error!(%error, "failed to fetch safe allowance from chain after indexer sync completed");
                        }
                    }
                }

                if let Err(error) = tx.try_send(()) {
                    error!(%error, "failed to notify about achieving indexer synchronization")
                }
            }
        }
    }

    /// Checks if the logs database has any existing data.
    ///
    /// This method determines whether the database already contains logs, which helps
    /// decide whether to download a snapshot for faster synchronization. It queries
    /// the database for the total log count and returns an error if the query fails
    /// (e.g., when the database doesn't exist yet).
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if the database contains one or more logs
    /// - `Ok(false)` if the database is empty
    /// - `Err(e)` if the database cannot be queried
    async fn has_logs_data(&self) -> Result<bool> {
        self.db
            .get_logs_count(None, None)
            .await
            .map(|count| count > 0)
            .map_err(|e| CoreEthereumIndexerError::SnapshotError(e.to_string()))
    }

    /// Downloads and installs a database snapshot for faster initial synchronization.
    ///
    /// This method coordinates the snapshot download process by:
    /// 1. Validating the indexer configuration
    /// 2. Creating a snapshot manager instance
    /// 3. Downloading and extracting the snapshot to the data directory
    ///
    /// Snapshots allow new nodes to quickly synchronize with the network by downloading
    /// pre-built database files instead of fetching all historical logs from scratch.
    ///
    /// # Returns
    ///
    /// - `Ok(SnapshotInfo)` containing details about the downloaded snapshot
    /// - `Err(CoreEthereumIndexerError::SnapshotError)` if validation or download fails
    ///
    /// # Prerequisites
    ///
    /// - Configuration must be valid (proper URL format, data directory set)
    /// - Sufficient disk space must be available
    /// - Network connectivity to the snapshot URL
    pub async fn download_snapshot(&self) -> Result<SnapshotInfo> {
        // Validate config before proceeding
        if let Err(e) = self.cfg.validate() {
            return Err(CoreEthereumIndexerError::SnapshotError(e.to_string()));
        }

        let snapshot_manager = SnapshotManager::with_db(self.db.clone())
            .map_err(|e| CoreEthereumIndexerError::SnapshotError(e.to_string()))?;

        let data_dir = Path::new(&self.cfg.data_directory);

        snapshot_manager
            .download_and_setup_snapshot(&self.cfg.logs_snapshot_url, data_dir)
            .await
            .map_err(|e| CoreEthereumIndexerError::SnapshotError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, pin::Pin};

    use alloy::{
        dyn_abi::DynSolValue,
        primitives::{Address as AlloyAddress, B256},
        sol_types::SolEvent,
    };
    use async_trait::async_trait;
    use futures::{Stream, join};
    use hex_literal::hex;
    use hopr_chain_rpc::BlockWithLogs;
    use hopr_chain_types::{ContractAddresses, chain_events::ChainEventType};
    use hopr_crypto_types::{
        keypairs::{Keypair, OffchainKeypair},
        prelude::ChainKeypair,
    };
    use hopr_db_sql::{accounts::HoprDbAccountOperations, db::HoprDb};
    use hopr_internal_types::account::{AccountEntry, AccountType};
    use hopr_primitive_types::prelude::*;
    use mockall::mock;
    use multiaddr::Multiaddr;

    use super::*;
    use crate::traits::MockChainLogHandler;

    lazy_static::lazy_static! {
        static ref ALICE_OKP: OffchainKeypair = OffchainKeypair::random();
        static ref ALICE_KP: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).expect("lazy static keypair should be constructible");
        static ref ALICE: Address = ALICE_KP.public().to_address();
        static ref BOB_OKP: OffchainKeypair = OffchainKeypair::random();
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
        starting_log_index: u64,
    ) -> anyhow::Result<Vec<SerializableLog>> {
        let mut logs: Vec<SerializableLog> = vec![];
        let block_hash = Hash::create(&[format!("my block hash {block_number}").as_bytes()]);

        for i in 0..size {
            let test_multiaddr: Multiaddr = format!("/ip4/1.2.3.4/tcp/{}", 1000 + i).parse()?;
            let tx_index: u64 = i as u64;
            let log_index: u64 = starting_log_index + tx_index;

            logs.push(SerializableLog {
                address,
                block_hash: block_hash.into(),
                topics: vec![hopr_bindings::hoprannouncementsevents::HoprAnnouncementsEvents::AddressAnnouncement::SIGNATURE_HASH.into()],
                data: DynSolValue::Tuple(vec![
                    DynSolValue::Address(AlloyAddress::from_slice(address.as_ref())),
                    DynSolValue::String(test_multiaddr.to_string()),
                ])
                .abi_encode(),
                tx_hash: Hash::create(&[format!("my tx hash {i}").as_bytes()]).into(),
                tx_index,
                block_number,
                log_index,
                ..Default::default()
            });
        }

        Ok(logs)
    }

    mock! {
        HoprIndexerOps {}     // Name of the mock struct, less the "Mock" prefix

        #[async_trait]
        impl HoprIndexerRpcOperations for HoprIndexerOps {
            async fn block_number(&self) -> hopr_chain_rpc::errors::Result<u64>;
            async fn get_hopr_allowance(&self, owner: Address, spender: Address) -> hopr_chain_rpc::errors::Result<HoprBalance>;
            async fn get_xdai_balance(&self, address: Address) -> hopr_chain_rpc::errors::Result<XDaiBalance>;
            async fn get_hopr_balance(&self, address: Address) -> hopr_chain_rpc::errors::Result<HoprBalance>;

            fn try_stream_logs<'a>(
                &'a self,
                start_block_number: u64,
                filters: FilterSet,
                is_synced: bool,
            ) -> hopr_chain_rpc::errors::Result<Pin<Box<dyn Stream<Item = BlockWithLogs> + Send + 'a>>>;
        }
    }

    #[tokio::test]
    async fn test_indexer_should_check_the_db_for_last_processed_block_and_supply_none_if_none_is_found()
    -> anyhow::Result<()> {
        let mut handlers = MockChainLogHandler::new();
        let mut rpc = MockHoprIndexerOps::new();
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let addr = Address::new(b"my address 123456789");
        let topic = Hash::create(&[b"my topic"]);
        db.ensure_logs_origin(vec![(addr, topic)]).await?;

        handlers.expect_contract_addresses().return_const(vec![addr]);
        handlers
            .expect_contract_address_topics()
            .withf(move |x| x == &addr)
            .return_const(vec![B256::from_slice(topic.as_ref())]);
        handlers
            .expect_contract_address_topics()
            .withf(move |x| x == &addr)
            .return_const(vec![B256::from_slice(Hash::create(&[b"my topic"]).as_ref())]);
        handlers
            .expect_safe_address()
            .return_const(Address::new(b"my safe address 1234"));
        handlers
            .expect_contract_addresses_map()
            .return_const(ContractAddresses::default());

        let head_block = 1000;
        rpc.expect_block_number().times(2).returning(move || Ok(head_block));

        let (tx, rx) = futures::channel::mpsc::unbounded::<BlockWithLogs>();
        rpc.expect_try_stream_logs()
            .withf(move |x: &u64, _y: &FilterSet, _: &bool| *x == 0)
            .return_once(move |_, _, _| Ok(Box::pin(rx)));

        let indexer = Indexer::new(
            rpc,
            handlers,
            db.clone(),
            IndexerConfig::default(),
            async_channel::unbounded().0,
        )
        .without_panic_on_completion();

        let (indexing, _) = join!(indexer.start(), async move {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            tx.close_channel()
        });
        assert!(indexing.is_err()); // terminated by the close channel

        Ok(())
    }

    #[tokio::test]
    async fn test_indexer_should_check_the_db_for_last_processed_block_and_supply_it_when_found() -> anyhow::Result<()>
    {
        let mut handlers = MockChainLogHandler::new();
        let mut rpc = MockHoprIndexerOps::new();
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;
        let head_block = 1000;
        let latest_block = 15u64;

        let addr = Address::new(b"my address 123456789");
        let topic = Hash::create(&[b"my topic"]);

        handlers.expect_contract_addresses().return_const(vec![addr]);
        handlers
            .expect_contract_address_topics()
            .withf(move |x| x == &addr)
            .return_const(vec![B256::from_slice(topic.as_ref())]);
        handlers
            .expect_contract_address_topics()
            .withf(move |x| x == &addr)
            .return_const(vec![B256::from_slice(Hash::create(&[b"my topic"]).as_ref())]);
        handlers
            .expect_safe_address()
            .return_const(Address::new(b"my safe address 1234"));
        handlers
            .expect_contract_addresses_map()
            .return_const(ContractAddresses::default());

        db.ensure_logs_origin(vec![(addr, topic)]).await?;

        rpc.expect_block_number().times(2).returning(move || Ok(head_block));

        let (tx, rx) = futures::channel::mpsc::unbounded::<BlockWithLogs>();
        rpc.expect_try_stream_logs()
            .once()
            .withf(move |x: &u64, _y: &FilterSet, _: &bool| *x == latest_block + 1)
            .return_once(move |_, _, _| Ok(Box::pin(rx)));

        // insert and process latest block
        let log_1 = SerializableLog {
            address: Address::new(b"my address 123456789"),
            topics: [Hash::create(&[b"my topic"]).into()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1u64,
            block_number: latest_block,
            block_hash: Hash::create(&[b"my block hash"]).into(),
            tx_hash: Hash::create(&[b"my tx hash"]).into(),
            log_index: 1u64,
            removed: false,
            processed: Some(false),
            ..Default::default()
        };
        assert!(db.store_log(log_1.clone()).await.is_ok());
        assert!(db.set_logs_processed(Some(latest_block), Some(0)).await.is_ok());
        assert!(db.update_logs_checksums().await.is_ok());

        let indexer = Indexer::new(
            rpc,
            handlers,
            db.clone(),
            IndexerConfig {
                fast_sync: false,
                ..Default::default()
            },
            async_channel::unbounded().0,
        )
        .without_panic_on_completion();

        let (indexing, _) = join!(indexer.start(), async move {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            tx.close_channel()
        });
        assert!(indexing.is_err()); // terminated by the close channel

        Ok(())
    }

    #[tokio::test]
    async fn test_indexer_should_pass_blocks_that_are_finalized() -> anyhow::Result<()> {
        let mut handlers = MockChainLogHandler::new();
        let mut rpc = MockHoprIndexerOps::new();
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let cfg = IndexerConfig::default();

        let addr = Address::new(b"my address 123456789");
        handlers.expect_contract_addresses().return_const(vec![addr]);
        handlers
            .expect_contract_address_topics()
            .withf(move |x| x == &addr)
            .return_const(vec![B256::from_slice(Hash::create(&[b"my topic"]).as_ref())]);
        handlers
            .expect_safe_address()
            .return_const(Address::new(b"my safe address 1234"));
        handlers
            .expect_contract_addresses_map()
            .return_const(ContractAddresses::default());

        let (mut tx, rx) = futures::channel::mpsc::unbounded::<BlockWithLogs>();
        rpc.expect_try_stream_logs()
            .times(1)
            .withf(move |x: &u64, _y: &FilterSet, _: &bool| *x == 0)
            .return_once(move |_, _, _| Ok(Box::pin(rx)));

        let head_block = 1000;
        rpc.expect_block_number().returning(move || Ok(head_block));

        rpc.expect_get_hopr_balance()
            .withf(move |x| x == &Address::new(b"my safe address 1234"))
            .returning(move |_| Ok(HoprBalance::default()));

        rpc.expect_get_hopr_allowance()
            .withf(move |x, y| x == &Address::new(b"my safe address 1234") && y == &Address::from([0; 20]))
            .returning(move |_, _| Ok(HoprBalance::default()));

        let finalized_block = BlockWithLogs {
            block_id: head_block - 1,
            logs: BTreeSet::from_iter(build_announcement_logs(*BOB, 4, head_block - 1, 23)?),
        };
        let head_allowing_finalization = BlockWithLogs {
            block_id: head_block,
            logs: BTreeSet::new(),
        };

        // called once per block which is finalizable
        handlers
            .expect_collect_log_event()
            // .times(2)
            .times(finalized_block.logs.len())
            .returning(|_, _| Ok(None));

        assert!(tx.start_send(finalized_block.clone()).is_ok());
        assert!(tx.start_send(head_allowing_finalization.clone()).is_ok());

        let indexer =
            Indexer::new(rpc, handlers, db.clone(), cfg, async_channel::unbounded().0).without_panic_on_completion();
        let _ = join!(indexer.start(), async move {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            tx.close_channel()
        });

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_indexer_fast_sync_full_with_resume() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let addr = Address::new(b"my address 123456789");
        let topic = Hash::create(&[b"my topic"]);

        // Run 1: Fast sync enabled, index empty
        {
            let logs = vec![
                build_announcement_logs(*BOB, 1, 1, 1)?,
                build_announcement_logs(*BOB, 1, 2, 1)?,
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

            assert!(db.ensure_logs_origin(vec![(addr, topic)]).await.is_ok());

            for log in logs {
                assert!(db.store_log(log).await.is_ok());
            }
            assert!(db.update_logs_checksums().await.is_ok());
            assert_eq!(db.get_logs_block_numbers(None, None, Some(true)).await?.len(), 0);
            assert_eq!(db.get_logs_block_numbers(None, None, Some(false)).await?.len(), 2);

            let (tx, rx) = futures::channel::mpsc::unbounded::<BlockWithLogs>();
            let (tx_events, _) = async_channel::unbounded();

            let head_block = 5;
            let mut rpc = MockHoprIndexerOps::new();
            rpc.expect_block_number().returning(move || Ok(head_block));
            rpc.expect_try_stream_logs()
                .times(1)
                .withf(move |x: &u64, _y: &FilterSet, _: &bool| *x == 3)
                .return_once(move |_, _, _| Ok(Box::pin(rx)));

            let mut handlers = MockChainLogHandler::new();
            handlers.expect_contract_addresses().return_const(vec![addr]);
            handlers
                .expect_contract_address_topics()
                .withf(move |x| x == &addr)
                .return_const(vec![B256::from_slice(topic.as_ref())]);
            handlers
                .expect_collect_log_event()
                .times(2)
                .withf(move |l, _| [1, 2].contains(&l.block_number))
                .returning(|_, _| Ok(None));
            handlers
                .expect_contract_address_topics()
                .withf(move |x| x == &addr)
                .return_const(vec![B256::from_slice(Hash::create(&[b"my topic"]).as_ref())]);
            handlers
                .expect_safe_address()
                .return_const(Address::new(b"my safe address 1234"));
            handlers
                .expect_contract_addresses_map()
                .return_const(ContractAddresses::default());

            let indexer_cfg = IndexerConfig {
                start_block_number: 0,
                fast_sync: true,
                logs_snapshot_url: "".to_string(),
                data_directory: "/tmp/test_data".to_string(),
            };
            let indexer = Indexer::new(rpc, handlers, db.clone(), indexer_cfg, tx_events).without_panic_on_completion();
            let (indexing, _) = join!(indexer.start(), async move {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                tx.close_channel()
            });
            assert!(indexing.is_err()); // terminated by the close channel

            assert_eq!(db.get_logs_block_numbers(None, None, Some(true)).await?.len(), 2);
            assert_eq!(db.get_logs_block_numbers(None, None, Some(false)).await?.len(), 0);

            // At the end we need to simulate that the index is not empty,
            // thus storing some data.
            db.insert_account(
                None,
                AccountEntry {
                    public_key: *ALICE_OKP.public(),
                    chain_addr: *ALICE,
                    entry_type: AccountType::NotAnnounced,
                    published_at: 1,
                },
            )
            .await?;
            db.insert_account(
                None,
                AccountEntry {
                    public_key: *BOB_OKP.public(),
                    chain_addr: *BOB,
                    entry_type: AccountType::NotAnnounced,
                    published_at: 1,
                },
            )
            .await?;
        }

        // Run 2: Fast sync enabled, index not empty, resume after 2 logs
        {
            let logs = vec![
                build_announcement_logs(*BOB, 1, 3, 1)?,
                build_announcement_logs(*BOB, 1, 4, 1)?,
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

            assert!(db.ensure_logs_origin(vec![(addr, topic)]).await.is_ok());

            for log in logs {
                assert!(db.store_log(log).await.is_ok());
            }
            assert!(db.update_logs_checksums().await.is_ok());
            assert_eq!(db.get_logs_block_numbers(None, None, Some(true)).await?.len(), 2);
            assert_eq!(db.get_logs_block_numbers(None, None, Some(false)).await?.len(), 2);

            let (tx, rx) = futures::channel::mpsc::unbounded::<BlockWithLogs>();
            let (tx_events, _) = async_channel::unbounded();

            let head_block = 5;
            let mut rpc = MockHoprIndexerOps::new();
            rpc.expect_block_number().returning(move || Ok(head_block));
            rpc.expect_try_stream_logs()
                .times(1)
                .withf(move |x: &u64, _y: &FilterSet, _: &bool| *x == 5)
                .return_once(move |_, _, _| Ok(Box::pin(rx)));

            let mut handlers = MockChainLogHandler::new();
            handlers.expect_contract_addresses().return_const(vec![addr]);
            handlers
                .expect_contract_address_topics()
                .withf(move |x| x == &addr)
                .return_const(vec![B256::from_slice(topic.as_ref())]);

            handlers
                .expect_collect_log_event()
                .times(2)
                .withf(move |l, _| [3, 4].contains(&l.block_number))
                .returning(|_, _| Ok(None));
            handlers
                .expect_contract_address_topics()
                .withf(move |x| x == &addr)
                .return_const(vec![B256::from_slice(Hash::create(&[b"my topic"]).as_ref())]);
            handlers
                .expect_safe_address()
                .return_const(Address::new(b"my safe address 1234"));
            handlers
                .expect_contract_addresses_map()
                .return_const(ContractAddresses::default());

            let indexer_cfg = IndexerConfig {
                start_block_number: 0,
                fast_sync: true,
                logs_snapshot_url: "".to_string(),
                data_directory: "/tmp/test_data".to_string(),
            };
            let indexer = Indexer::new(rpc, handlers, db.clone(), indexer_cfg, tx_events).without_panic_on_completion();
            let (indexing, _) = join!(indexer.start(), async move {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                tx.close_channel()
            });
            assert!(indexing.is_err()); // terminated by the close channel

            assert_eq!(db.get_logs_block_numbers(None, None, Some(true)).await?.len(), 4);
            assert_eq!(db.get_logs_block_numbers(None, None, Some(false)).await?.len(), 0);
        }

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_indexer_should_yield_back_once_the_past_events_are_indexed() -> anyhow::Result<()> {
        let mut handlers = MockChainLogHandler::new();
        let mut rpc = MockHoprIndexerOps::new();
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let cfg = IndexerConfig::default();

        // We don't want to index anything really
        let addr = Address::new(b"my address 123456789");
        handlers.expect_contract_addresses().return_const(vec![addr]);
        handlers
            .expect_contract_address_topics()
            .withf(move |x| x == &addr)
            .return_const(vec![B256::from_slice(Hash::create(&[b"my topic"]).as_ref())]);
        handlers
            .expect_contract_address_topics()
            .withf(move |x| x == &addr)
            .return_const(vec![B256::from_slice(Hash::create(&[b"my topic"]).as_ref())]);
        handlers
            .expect_safe_address()
            .return_const(Address::new(b"my safe address 1234"));
        handlers
            .expect_contract_addresses_map()
            .return_const(ContractAddresses::default());

        let (mut tx, rx) = futures::channel::mpsc::unbounded::<BlockWithLogs>();
        // Expected to be called once starting at 0 and yield the respective blocks
        rpc.expect_try_stream_logs()
            .times(1)
            .withf(move |x: &u64, _y: &FilterSet, _: &bool| *x == 0)
            .return_once(move |_, _, _| Ok(Box::pin(rx)));
        rpc.expect_get_hopr_balance()
            .once()
            .return_once(move |_| Ok(HoprBalance::zero()));
        rpc.expect_get_hopr_allowance()
            .once()
            .return_once(move |_, _| Ok(HoprBalance::zero()));

        let head_block = 1000;
        let block_numbers = [head_block - 1, head_block, head_block + 1];

        let blocks: Vec<BlockWithLogs> = block_numbers
            .iter()
            .map(|block_id| BlockWithLogs {
                block_id: *block_id,
                logs: BTreeSet::from_iter(build_announcement_logs(*ALICE, 1, *block_id, 23).unwrap()),
            })
            .collect();

        for _ in 0..(blocks.len() as u64) {
            rpc.expect_block_number().returning(move || Ok(head_block));
        }

        for block in blocks.iter() {
            assert!(tx.start_send(block.clone()).is_ok());
        }

        // Generate the expected events to be able to process the blocks
        handlers
            .expect_collect_log_event()
            .times(1)
            .withf(move |l, _| block_numbers.contains(&l.block_number))
            .returning(|l, _| {
                let block_number = l.block_number;
                Ok(Some(SignificantChainEvent {
                    tx_hash: Hash::create(&[format!("my tx hash {block_number}").as_bytes()]),
                    event_type: RANDOM_ANNOUNCEMENT_CHAIN_EVENT.clone(),
                }))
            });

        let (tx_events, rx_events) = async_channel::unbounded();
        let indexer = Indexer::new(rpc, handlers, db.clone(), cfg, tx_events).without_panic_on_completion();
        indexer.start().await?;

        // At this point we expect 2 events to arrive. The third event, which was generated first,
        // should be dropped because it was generated before the indexer was in sync with head.
        let _first = rx_events.recv();
        let _second = rx_events.recv();
        let third = rx_events.try_recv();

        assert!(third.is_err());

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_indexer_should_not_reprocess_last_processed_block() -> anyhow::Result<()> {
        let last_processed_block = 100_u64;

        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let addr = Address::new(b"my address 123456789");
        let topic = Hash::create(&[b"my topic"]);
        assert!(db.ensure_logs_origin(vec![(addr, topic)]).await.is_ok());

        // insert and process latest block
        let log_1 = SerializableLog {
            address: Address::new(b"my address 123456789"),
            topics: [Hash::create(&[b"my topic"]).into()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1u64,
            block_number: last_processed_block,
            block_hash: Hash::create(&[b"my block hash"]).into(),
            tx_hash: Hash::create(&[b"my tx hash"]).into(),
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
            .withf(move |x: &u64, _y: &FilterSet, _: &bool| *x == last_processed_block + 1)
            .return_once(move |_, _, _| Ok(Box::pin(rx)));

        rpc.expect_block_number()
            .times(3)
            .returning(move || Ok(last_processed_block + 1));

        rpc.expect_get_hopr_balance()
            .once()
            .return_once(move |_| Ok(HoprBalance::zero()));

        rpc.expect_get_hopr_allowance()
            .once()
            .return_once(move |_, _| Ok(HoprBalance::zero()));

        let block = BlockWithLogs {
            block_id: last_processed_block + 1,
            logs: BTreeSet::from_iter(build_announcement_logs(*ALICE, 1, last_processed_block + 1, 23)?),
        };

        tx.start_send(block)?;

        let mut handlers = MockChainLogHandler::new();
        handlers.expect_contract_addresses().return_const(vec![addr]);
        handlers
            .expect_contract_address_topics()
            .withf(move |x| x == &addr)
            .return_const(vec![B256::from_slice(topic.as_ref())]);
        handlers
            .expect_contract_address_topics()
            .withf(move |x| x == &addr)
            .return_const(vec![B256::from_slice(Hash::create(&[b"my topic"]).as_ref())]);
        handlers
            .expect_safe_address()
            .return_const(Address::new(b"my safe address 1234"));
        handlers
            .expect_contract_addresses_map()
            .return_const(ContractAddresses::default());

        let indexer_cfg = IndexerConfig {
            start_block_number: 0,
            fast_sync: false,
            logs_snapshot_url: "".to_string(),
            data_directory: "/tmp/test_data".to_string(),
        };

        let (tx_events, _) = async_channel::unbounded();
        let indexer = Indexer::new(rpc, handlers, db.clone(), indexer_cfg, tx_events).without_panic_on_completion();
        indexer.start().await?;

        Ok(())
    }
}
