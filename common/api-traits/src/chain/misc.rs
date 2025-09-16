use hopr_crypto_types::prelude::Hash;

/// Contains domain separator information.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DomainSeparators {
    /// HOPR Ledger smart contract domain separator.
    pub ledger: Hash,
    /// HOPR Node Safe Registry smart contract domain separator.
    pub safe_registry: Hash,
    /// HOPR Channels smart contract domain separator.
    pub channel: Hash,
}

/// Retrieves various on-chain information.
#[async_trait::async_trait]
pub trait ChainMiscOperations {
    type Error;
    /// Retrieves the domain separators of HOPR smart contracts.
    async fn domain_separators(&self) -> Result<DomainSeparators, Self::Error>;
}
