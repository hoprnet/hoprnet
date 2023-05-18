use async_trait::async_trait;

use utils_types::primitives::{Address, Balance, U256};

use crate::errors::Result;

#[async_trait(? Send)] // not placing the `Send` trait limitations on the trait
pub trait HoprCoreEthereumDbActions {
    async fn abc(&self) -> Result<()>;
}
