use async_trait::async_trait;
use ethers::{
    middleware::{
        gas_oracle::{GasCategory, GasOracleError},
        GasOracle,
    },
    utils::parse_units,
};
use primitive_types::U256;
use serde::Deserialize;
use url::Url;

use crate::HttpRequestor;

pub const EIP1559_FEE_ESTIMATION_DEFAULT_MAX_FEE_GNOSIS: u64 = 3_000_000_000;
pub const EIP1559_FEE_ESTIMATION_DEFAULT_PRIORITY_FEE_GNOSIS: u64 = 100_000_000;

/// Use the underlying gas tracker API of GnosisScan to populate the gas price.
/// It returns gas price in gwei.
/// It implements the `GasOracle` trait.
/// If no Oracle URL is given, it returns no values.
#[derive(Clone, Debug)]
#[must_use]
pub struct GnosisScan<C> {
    client: C,
    url: Option<Url>,
    gas_category: GasCategory,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Response {
    pub status: String,
    pub message: String,
    pub result: ResponseResult,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct ResponseResult {
    pub last_block: String,
    pub safe_gas_price: String,
    pub propose_gas_price: String,
    pub fast_gas_price: String,
}

impl Response {
    #[inline]
    pub fn gas_from_category(&self, gas_category: GasCategory) -> String {
        self.result.gas_from_category(gas_category)
    }
}

impl ResponseResult {
    fn gas_from_category(&self, gas_category: GasCategory) -> String {
        match gas_category {
            GasCategory::SafeLow => self.safe_gas_price.clone(),
            GasCategory::Standard => self.propose_gas_price.clone(),
            GasCategory::Fast => self.fast_gas_price.clone(),
            GasCategory::Fastest => self.fast_gas_price.clone(),
        }
    }
}

#[async_trait]
impl<C: HttpRequestor + std::fmt::Debug> GasOracle for GnosisScan<C> {
    async fn fetch(&self) -> Result<U256, GasOracleError> {
        let res: Response = self.query().await?;
        let gas_price_in_gwei = res.gas_from_category(self.gas_category);
        let gas_price = parse_units(gas_price_in_gwei, "gwei")?.into();
        Ok(gas_price)
    }

    // returns hardcoded (max_fee_per_gas, max_priority_fee_per_gas)
    // Due to foundry is unable to estimate EIP-1559 fees for L2s https://github.com/foundry-rs/foundry/issues/5709,
    // a hardcoded value of (3 gwei, 0.1 gwei) for Gnosischain is returned.
    async fn estimate_eip1559_fees(&self) -> Result<(U256, U256), GasOracleError> {
        Ok((
            U256::from(EIP1559_FEE_ESTIMATION_DEFAULT_MAX_FEE_GNOSIS),
            U256::from(EIP1559_FEE_ESTIMATION_DEFAULT_PRIORITY_FEE_GNOSIS),
        ))
    }
}

impl<C: HttpRequestor> GnosisScan<C> {
    /// Same as [`Self::new`] but with a custom [`Client`].
    pub fn with_client(client: C, url: Option<Url>) -> Self {
        Self {
            client,
            url,
            gas_category: GasCategory::Standard,
        }
    }

    /// Sets the gas price category to be used when fetching the gas price.
    pub fn category(mut self, gas_category: GasCategory) -> Self {
        self.gas_category = gas_category;
        self
    }

    /// Perform a request to the gas price API and deserialize the response.
    pub async fn query(&self) -> Result<Response, GasOracleError> {
        self.client
            .http_get(self.url.as_ref().ok_or(GasOracleError::NoValues)?.as_str())
            .await
            .map_err(|error| {
                tracing::error!(%error, "failed to query gas price API");
                GasOracleError::InvalidResponse
            })
            .and_then(|response| {
                serde_json::from_slice(response.as_ref()).map_err(|error| {
                    tracing::error!(%error, "failed to deserialize gas price API response");
                    GasOracleError::InvalidResponse
                })
            })
    }
}
