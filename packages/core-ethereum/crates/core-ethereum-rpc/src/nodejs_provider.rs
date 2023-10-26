use async_trait::async_trait;
use ethers::utils::__serde_json::Error;
use ethers_providers::{JsonRpcClient, JsonRpcError, ProviderError, RpcError};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;
use utils_misc::utils::wasm::js_value_to_error_msg;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use crate::nodejs_provider::NodeJsRpcError::JsCallError;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn http_request(method: &str, json_params: &str) -> Result<JsValue, JsValue>;
}

#[derive(Debug, Error)]
pub enum NodeJsRpcError {
    #[error("RPC request execution error in JS: {0}")]
    JsCallError(String),

    #[error(transparent)]
    SerializationError(#[from] serde_json::Error),
}

impl RpcError for NodeJsRpcError {
    fn as_error_response(&self) -> Option<&JsonRpcError> {
        todo!()
    }

    fn as_serde_error(&self) -> Option<&Error> {
        todo!()
    }
}

impl From<NodeJsRpcError> for ProviderError {
    fn from(value: NodeJsRpcError) -> Self {
        match value {
            NodeJsRpcError::SerializationError(e) => ProviderError::SerdeJson(e),
            JsCallError(e) => ProviderError::CustomError(e),
        }
    }
}

#[derive(Debug)]
pub struct NodeJsRpcClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NodeJsHttpResponse {
    pub code: u16,
    pub data: String,
}

#[async_trait(? Send)]
impl JsonRpcClient for NodeJsRpcClient {
    type Error = NodeJsRpcError;

    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R, Self::Error> {
        let json_params = serde_json::to_string(&params)?;
        let http_res = match http_request(method, &json_params).await {
            Ok(s) => s
                .as_string()
                .ok_or(JsCallError("expected string on http output".into()))?,
            Err(err) => {
                return Err(JsCallError(
                    js_value_to_error_msg(err).unwrap_or("unknown error".into()),
                ))
            }
        };

        Ok(serde_json::from_str(&http_res)?)
    }
}
