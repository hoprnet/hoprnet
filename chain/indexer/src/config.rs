/// Configuration for the chain indexer functionality
#[derive(Debug, Clone, Copy, smart_default::SmartDefault)]
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
    pub fast_sync: bool,
}

impl IndexerConfig {
    pub fn new(start_block_number: u64, fast_sync: bool) -> Self {
        Self {
            start_block_number,
            fast_sync,
        }
    }

    pub fn disable_fast_sync(mut self) -> Self {
        self.fast_sync = false;
        self
    }

    pub fn enable_fast_sync(mut self) -> Self {
        self.fast_sync = true;
        self
    }
}
