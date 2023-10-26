use async_trait::async_trait;
use ethers_providers::{JsonRpcClient, JsonRpcError, ProviderError};
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use utils_misc::utils::wasm::js_value_to_error_msg;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use serde::{
    de::{self, MapAccess, Unexpected, Visitor},
    Deserialize, Serialize,
};
use serde_json::{value::RawValue};
use std::fmt;
use thiserror::Error;


fn is_zst<T>(_t: &T) -> bool {
    std::mem::size_of::<T>() == 0
}

/// A JSON-RPC request
#[derive(Serialize, Deserialize, Debug)]
pub struct Request<'a, T> {
    id: u64,
    jsonrpc: &'a str,
    method: &'a str,
    #[serde(skip_serializing_if = "is_zst")]
    params: T,
}

impl<'a, T> Request<'a, T> {
    /// Creates a new JSON RPC request
    pub fn new(id: u64, method: &'a str, params: T) -> Self {
        Self { id, jsonrpc: "2.0", method, params }
    }
}

/// A JSON-RPC response
#[derive(Debug)]
pub enum Response<'a> {
    Success { id: u64, result: &'a RawValue },
    Error { id: u64, error: JsonRpcError },
    Notification { method: &'a str, params: Params<'a> },
}

#[derive(Deserialize, Debug)]
pub struct Params<'a> {
    pub subscription: ethers::types::U256,
    #[serde(borrow)]
    pub result: &'a RawValue,
}

// FIXME: ideally, this could be auto-derived as an untagged enum, but due to
// https://github.com/serde-rs/serde/issues/1183 this currently fails
impl<'de: 'a, 'a> Deserialize<'de> for Response<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
    {
        struct ResponseVisitor<'a>(&'a ());
        impl<'de: 'a, 'a> Visitor<'de> for ResponseVisitor<'a> {
            type Value = Response<'a>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid jsonrpc 2.0 response object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: MapAccess<'de>,
            {
                let mut jsonrpc = false;

                // response & error
                let mut id = None;
                // only response
                let mut result = None;
                // only error
                let mut error = None;
                // only notification
                let mut method = None;
                let mut params = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "jsonrpc" => {
                            if jsonrpc {
                                return Err(de::Error::duplicate_field("jsonrpc"))
                            }

                            let value = map.next_value()?;
                            if value != "2.0" {
                                return Err(de::Error::invalid_value(Unexpected::Str(value), &"2.0"))
                            }

                            jsonrpc = true;
                        }
                        "id" => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"))
                            }

                            let value: u64 = map.next_value()?;
                            id = Some(value);
                        }
                        "result" => {
                            if result.is_some() {
                                return Err(de::Error::duplicate_field("result"))
                            }

                            let value: &RawValue = map.next_value()?;
                            result = Some(value);
                        }
                        "error" => {
                            if error.is_some() {
                                return Err(de::Error::duplicate_field("error"))
                            }

                            let value: JsonRpcError = map.next_value()?;
                            error = Some(value);
                        }
                        "method" => {
                            if method.is_some() {
                                return Err(de::Error::duplicate_field("method"))
                            }

                            let value: &str = map.next_value()?;
                            method = Some(value);
                        }
                        "params" => {
                            if params.is_some() {
                                return Err(de::Error::duplicate_field("params"))
                            }

                            let value: Params = map.next_value()?;
                            params = Some(value);
                        }
                        key => {
                            return Err(de::Error::unknown_field(
                                key,
                                &["id", "jsonrpc", "result", "error", "params", "method"],
                            ))
                        }
                    }
                }

                // jsonrpc version must be present in all responses
                if !jsonrpc {
                    return Err(de::Error::missing_field("jsonrpc"))
                }

                match (id, result, error, method, params) {
                    (Some(id), Some(result), None, None, None) => {
                        Ok(Response::Success { id, result })
                    }
                    (Some(id), None, Some(error), None, None) => Ok(Response::Error { id, error }),
                    (None, None, None, Some(method), Some(params)) => {
                        Ok(Response::Notification { method, params })
                    }
                    _ => Err(de::Error::custom(
                        "response must be either a success/error or notification object",
                    )),
                }
            }
        }

        deserializer.deserialize_map(ResponseVisitor(&()))
    }
}

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

    #[error("Deserialization Error: {err}. Response: {text}")]
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

        let json_data = serde_json::to_string(&payload).
            map_err(|err| ClientError::SerdeJson { err, text: "failed to serialized request".into() })?;

        let body = match http_post(self.url.as_str(), &json_data).await {
            Ok(value) => value.as_string().ok_or(ClientError::JsError("not a string response".into()))?,
            Err(err) => return Err(ClientError::JsError(js_value_to_error_msg(err).unwrap_or("missing error message".into()))),
        };

        let raw = match serde_json::from_str(&body) {
            Ok(Response::Success { result, .. }) => result.to_owned(),
            Ok(Response::Error { error, .. }) => return Err(error.into()),
            Ok(_) => {
                let err = ClientError::SerdeJson {
                    err: serde::de::Error::custom("unexpected notification over HTTP transport"),
                    text: body,
                };
                return Err(err)
            }
            Err(err) => {
                return Err(ClientError::SerdeJson {
                    err,
                    text: body,
                })
            }
        };

        let res = serde_json::from_str(raw.get())
            .map_err(|err| ClientError::SerdeJson { err, text: raw.to_string() })?;

        Ok(res)
    }
}
