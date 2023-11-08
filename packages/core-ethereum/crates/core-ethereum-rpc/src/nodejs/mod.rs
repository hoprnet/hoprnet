use async_trait::async_trait;
use ethers_providers::{JsonRpcClient, JsonRpcError, ProviderError, PubsubClient};
use std::fmt::Debug;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use utils_misc::utils::wasm::js_value_to_error_msg;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use serde::{
    de::{DeserializeOwned, MapAccess, Visitor},
    Deserialize, Serialize,
};
use std::future::Future;
use thiserror::Error;

use crate::nodejs::helper::{Request, Response};

mod helper;

#[wasm_bindgen(module = "@hoprnet/hopr-utils")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn http_post(url: &str, json_data: &str) -> Result<JsValue, JsValue>;
}

/// Error thrown when sending an HTTP request
#[derive(Error, Debug)]
pub enum ClientError {
    /// Thrown if the request failed
    #[error("js error: {0}")]
    JsError(String),

    #[error(transparent)]
    /// Thrown if the response could not be parsed
    JsonRpcError(#[from] JsonRpcError),

    #[error("deserialization error: {err}, response: {text}")]
    /// Serde JSON Error
    SerdeJson {
        /// Underlying error
        err: serde_json::Error,
        /// The contents of the HTTP response that could not be deserialized
        text: String,
    },
}

impl From<ClientError> for ProviderError {
    fn from(src: ClientError) -> Self {
        match src {
            ClientError::JsError(err) => ProviderError::CustomError(err),
            _ => ProviderError::JsonRpcClientError(Box::new(src)),
        }
    }
}

impl ethers_providers::RpcError for ClientError {
    fn as_error_response(&self) -> Option<&ethers_providers::JsonRpcError> {
        if let ClientError::JsonRpcError(err) = self {
            Some(err)
        } else {
            None
        }
    }

    fn as_serde_error(&self) -> Option<&serde_json::Error> {
        match self {
            ClientError::SerdeJson { err, .. } => Some(err),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct NodeJsRpcClient {
    id: AtomicU64,
    url: String,
}

impl NodeJsRpcClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            id: AtomicU64::new(1),
            url: base_url.to_owned(),
        }
    }
}

#[async_trait(? Send)]
impl JsonRpcClient for NodeJsRpcClient {
    type Error = ClientError;

    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R, Self::Error> {
        let next_id = self.id.fetch_add(1, Ordering::SeqCst);
        let payload = Request::new(next_id, method, params);

        let json_data = serde_json::to_string(&payload).map_err(|err| ClientError::SerdeJson {
            err,
            text: "failed to serialized request".into(),
        })?;

        let body = match http_post(self.url.as_str(), &json_data).await {
            Ok(value) => value
                .as_string()
                .ok_or(ClientError::JsError("not a string response".into()))?,
            Err(err) => {
                return Err(ClientError::JsError(
                    js_value_to_error_msg(err).unwrap_or("missing error message".into()),
                ))
            }
        };

        let raw = match serde_json::from_str(&body) {
            Ok(Response::Success { result, .. }) => result.to_owned(),
            Ok(Response::Error { error, .. }) => return Err(error.into()),
            Ok(_) => {
                let err = ClientError::SerdeJson {
                    err: serde::de::Error::custom("unexpected notification over HTTP transport"),
                    text: body,
                };
                return Err(err);
            }
            Err(err) => return Err(ClientError::SerdeJson { err, text: body }),
        };

        let res = serde_json::from_str(raw.get()).map_err(|err| ClientError::SerdeJson {
            err,
            text: raw.to_string(),
        })?;

        Ok(res)
    }
}
