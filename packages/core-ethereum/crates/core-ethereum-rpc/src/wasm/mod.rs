use async_trait::async_trait;
use ethers_providers::{JsonRpcClient, JsonRpcError, ProviderError, RpcError};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use thiserror::Error;

use crate::wasm::helper::{Request, Response};

mod helper;

#[cfg(feature = "wasm")]
mod nodejs;

#[derive(Error, Debug)]
pub enum HttpRequestError {
    #[error("error on js-wasm interface: {0}")]
    InterfaceError(String),

    #[error("http error - status {0}")]
    HttpError(u32),
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait HttpPostRequestor: Send + Sync {
    async fn http_post(&self, url: &str, json_data: &str) -> Result<String, HttpRequestError>;
}

/// Error thrown when sending an HTTP request
#[derive(Error, Debug)]
pub enum ClientError {
    /// Thrown if the request failed
    #[error(transparent)]
    RequestorError(#[from] HttpRequestError),

    /// Thrown if the response could not be parsed
    #[error(transparent)]
    JsonRpcError(#[from] JsonRpcError),

    /// Serde JSON Error
    #[error("deserialization error: {err}, response: {text}")]
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
            ClientError::RequestorError(err) =>
            /*ProviderError::HTTPError(err.into())*/
            {
                unimplemented!()
            }
            _ => ProviderError::JsonRpcClientError(Box::new(src)),
        }
    }
}

impl RpcError for ClientError {
    fn as_error_response(&self) -> Option<&JsonRpcError> {
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
pub struct HttpRequestorRpcClient<Req: HttpPostRequestor + Debug> {
    id: AtomicU64,
    url: String,
    requestor: Req,
}

impl<Req: HttpPostRequestor + Debug> HttpRequestorRpcClient<Req> {
    pub fn new(base_url: &str, requestor: Req) -> Self {
        Self {
            id: AtomicU64::new(1),
            url: base_url.to_owned(),
            requestor,
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<Req: HttpPostRequestor + Debug> JsonRpcClient for HttpRequestorRpcClient<Req> {
    type Error = ClientError;

    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R, ClientError> {
        let next_id = self.id.fetch_add(1, Ordering::SeqCst);
        let payload = Request::new(next_id, method, params);
        let serialized_payload = serde_json::to_string(&payload).map_err(|err| ClientError::SerdeJson {
            err,
            text: "cannot serialize payload".into(),
        })?;

        let body = self.requestor.http_post(self.url.as_ref(), &serialized_payload).await?;

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
