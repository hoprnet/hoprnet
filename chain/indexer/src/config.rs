/// Configuration for the chain indexer functionality
#[derive(Debug, Clone, smart_default::SmartDefault)]
pub struct IndexerConfig {
    /// The block at which the indexer should start
    ///
    /// It typically makes little sense to start indexing from the beginning
    /// of the chain; all that is sufficient is to start indexing since the
    /// relevant smart contracts were introduced into the chain.
    ///
    /// This value makes sure that indexing is relevant and as minimal as possible.
    ///
    /// Default is `0`.
    pub start_block_number: u64,

    /// Whether to use fast synchronization during indexing.
    /// When enabled, it allows for quicker indexing of existing logs during node startup.
    ///
    /// Default is `true`.
    #[default(true)]
    pub fast_sync: bool,

    /// Whether to enable log snapshot downloading on startup.
    /// When enabled and fast sync is enabled, the indexer will attempt to download
    /// a pre-built logs database snapshot for faster initial sync.
    ///
    /// Default is `true`.
    #[default(true)]
    pub log_snapshot_enabled: bool,

    /// URL to download log snapshot from.
    /// This should point to a publicly accessible tar.gz file containing
    /// the SQLite logs database files.
    ///
    /// Default is "https://snapshots.hoprnet.org/logs/latest.tar.gz".
    #[default("https://snapshots.hoprnet.org/logs/latest.tar.gz".to_string())]
    pub log_snapshot_url: String,
}

impl IndexerConfig {
    /// Creates a new indexer configuration.
    ///
    /// # Arguments
    ///
    /// * `start_block_number` - The block number from which to start indexing
    /// * `fast_sync` - Whether to enable fast synchronization during startup
    /// * `log_snapshot_enabled` - Whether to enable log snapshot downloading
    /// * `log_snapshot_url` - URL to download log snapshot from
    ///
    /// # Returns
    ///
    /// A new instance of `IndexerConfig`
    pub fn new(
        start_block_number: u64,
        fast_sync: bool,
        log_snapshot_enabled: bool,
        log_snapshot_url: String,
    ) -> Self {
        Self {
            start_block_number,
            fast_sync,
            log_snapshot_enabled,
            log_snapshot_url,
        }
    }
}
