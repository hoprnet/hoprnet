use hopr_crypto_types::prelude::Hash;
use hopr_primitive_types::prelude::*;

/// Contains various on-chain information collected by Indexer,
/// such as domain separators, ticket price, Network Registry status...etc.
/// All these members change very rarely and therefore can be cached.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IndexerData {
    /// Ledger smart contract domain separator
    pub ledger_dst: Option<Hash>,
    /// Node safe registry smart contract domain separator
    pub safe_registry_dst: Option<Hash>,
    /// Channels smart contract domain separator
    pub channels_dst: Option<Hash>,
    /// Current ticket price
    pub ticket_price: Option<Balance>,
    /// Network registry state
    pub nr_enabled: bool,
}

impl IndexerData {
    /// Convenience method to retrieve domain separator according to the [DomainSeparator] enum.
    pub fn domain_separator(&self, dst_type: DomainSeparator) -> Option<Hash> {
        match dst_type {
            DomainSeparator::Ledger => self.ledger_dst,
            DomainSeparator::SafeRegistry => self.safe_registry_dst,
            DomainSeparator::Channel => self.channels_dst,
        }
    }
}

/// Contains information about node's safe.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SafeInfo {
    /// Safe address
    pub safe_address: Address,
    /// Safe module address.
    pub module_address: Address,
}

/// Enumerates different domain separators
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DomainSeparator {
    /// Ledger smart contract domain separator
    Ledger,
    /// Node safe registry smart contract domain separator
    SafeRegistry,
    /// Channels smart contract domain separator
    Channel,
}
