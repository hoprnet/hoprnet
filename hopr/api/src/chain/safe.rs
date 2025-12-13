use hopr_primitive_types::{
    balance::{Balance, Currency},
    prelude::Address,
};

/// Information about a deployed Safe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeployedSafe {
    /// Safe address.
    pub address: Address,
    /// Address of the Safe owner (typically the node chain key).
    pub owner: Address,
    /// Address of the Safe module.
    pub module: Address,
}

/// Selector for [deployed Safes](DeployedSafe).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SafeSelector {
    /// Selects Safes owned by the given address.
    Owner(Address),
    /// Selects Safes with the given address.
    Address(Address),
}

impl SafeSelector {
    pub fn satisfies(&self, safe: &DeployedSafe) -> bool {
        match self {
            SafeSelector::Owner(owner) => &safe.owner == owner,
            SafeSelector::Address(address) => &safe.address == address,
        }
    }
}

/// Operations for reading Safe information.
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait ChainReadSafeOperations {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Returns the native or token currency Safe allowance.
    async fn safe_allowance<C: Currency, A: Into<Address> + Send>(
        &self,
        safe_address: A,
    ) -> Result<Balance<C>, Self::Error>;
    /// Retrieves [`DeployedSafe`] information using the given [`selector`](SafeSelector).
    ///
    /// Returns `None` if no deployed Safe matched the given `selector`.
    async fn safe_info(&self, selector: SafeSelector) -> Result<Option<DeployedSafe>, Self::Error>;
    /// Waits for a Safe matching the given [`selector`](SafeSelector) to be deployed up to the given `timeout`.
    ///
    /// Returns immediately if the matching Safe is already deployed.
    async fn await_safe_deployment(
        &self,
        selector: SafeSelector,
        timeout: std::time::Duration,
    ) -> Result<DeployedSafe, Self::Error>;
    /// Predicts the Module address based on the given `nonce`, `owner` and `safe_address` of the Safe.
    async fn predict_module_address(
        &self,
        nonce: u64,
        owner: &Address,
        safe_address: &Address,
    ) -> Result<Address, Self::Error>;
}
