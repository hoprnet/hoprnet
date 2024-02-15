/// Configuration for the chain indexer functionality
#[derive(Debug, Clone, Copy, smart_default::SmartDefault)]
pub struct IndexerConfig {
    /// The block at which the indexer should start
    ///
    /// It typically makes little sense to start indexing from the beginning
    /// of the chain, all that is sufficient is to start indexing since the
    /// relevant smart contracts were introduced into the chain.
    ///
    /// This value makes sure that indexing is relevant and as minimal as possible.
    ///
    /// Default is `0`.
    pub start_block_number: u64,
    /// Fetch token transactions
    ///
    /// Whether the token transaction topics should also be fetched.
    ///
    /// Default is `true`.
    #[default = true]
    pub fetch_token_transactions: bool,
}
