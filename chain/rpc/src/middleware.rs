use async_trait::async_trait;
use ethers::{
    middleware::{
        gas_oracle::{GasCategory, GasOracleError},
        GasOracle,
    },
    utils::parse_units,
};
use primitive_types::U256;
use reqwest::Client;
use serde::Deserialize;
use url::Url;

const URL: &str = "https://ggnosis.blockscan.com/gasapi.ashx?apikey=key&method=gasoracle";
pub const EIP1559_FEE_ESTIMATION_DEFAULT_MAX_FEE_GNOSIS: u64 = 3_000_000_000;
pub const EIP1559_FEE_ESTIMATION_DEFAULT_PRIORITY_FEE_GNOSIS: u64 = 100_000_000;

/// Use the underlying gas tracker API of GnosisScan to populate the gas price.
/// It returns gas price in gwei.
/// It implements the `GasOracle` trait.
#[derive(Clone, Debug)]
#[must_use]
pub struct GnosisScan {
    client: Client,
    url: Url,
    gas_category: GasCategory,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Response {
    pub status: String,
    pub message: String,
    pub result: ResponseResult,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ResponseResult {
    #[serde(rename = "LastBlock")]
    pub last_block: String,
    #[serde(rename = "SafeGasPrice")]
    pub safe_gas_price: String,
    #[serde(rename = "ProposeGasPrice")]
    pub propose_gas_price: String,
    #[serde(rename = "FastGasPrice")]
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

impl Default for GnosisScan {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GasOracle for GnosisScan {
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

impl GnosisScan {
    /// Creates a new GnosisScan gas price oracle.
    pub fn new() -> Self {
        Self::with_client(Client::new())
    }

    /// Same as [`Self::new`] but with a custom [`Client`].
    pub fn with_client(client: Client) -> Self {
        let url = Url::parse(URL).unwrap();
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
        let response = self
            .client
            .get(self.url.as_ref())
            .send()
            .await
            .map_err(|_| GasOracleError::InvalidResponse)?
            .error_for_status()
            .map_err(|_| GasOracleError::InvalidResponse)?
            .json()
            .await
            .map_err(|_| GasOracleError::InvalidResponse)?;
        Ok(response)
    }
}
