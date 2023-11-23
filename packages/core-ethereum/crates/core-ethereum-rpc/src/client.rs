use crate::errors::JsonRpcProviderClientError;
use async_trait::async_trait;
use ethers_providers::{JsonRpcClient, RetryPolicy};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::helper::{Request, Response};
use crate::HttpPostRequestor;

/// Modified implementation of `ethers::providers::Http` so that it can
/// operate with any `HttpPostRequestor`.
#[derive(Debug)]
pub struct JsonRpcProviderClient<Req: HttpPostRequestor + Debug> {
    id: AtomicU64,
    url: String,
    requestor: Req,
}

impl<Req: HttpPostRequestor + Debug> JsonRpcProviderClient<Req> {
    pub fn new(base_url: &str, requestor: Req) -> Self {
        Self {
            id: AtomicU64::new(1),
            url: base_url.to_owned(),
            requestor,
        }
    }
}

impl<Req: HttpPostRequestor + Debug + Clone> Clone for JsonRpcProviderClient<Req> {
    fn clone(&self) -> Self {
        Self { id: AtomicU64::new(1), url: self.url.clone(), requestor: self.requestor.clone() }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<Req: HttpPostRequestor + Debug> JsonRpcClient for JsonRpcProviderClient<Req> {
    type Error = JsonRpcProviderClientError;

    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R, Self::Error> {
        let next_id = self.id.fetch_add(1, Ordering::SeqCst);
        let payload = Request::new(next_id, method, params);
        let serialized_payload = serde_json::to_string(&payload).map_err(|err| Self::Error::SerdeJson {
            err,
            text: "cannot serialize payload".into(),
        })?;

        let body = self.requestor.http_post(self.url.as_ref(), &serialized_payload).await?;

        let raw = match serde_json::from_str(&body) {
            Ok(Response::Success { result, .. }) => result.to_owned(),
            Ok(Response::Error { error, .. }) => return Err(error.into()),
            Ok(_) => {
                let err = Self::Error::SerdeJson {
                    err: serde::de::Error::custom("unexpected notification over HTTP transport"),
                    text: body,
                };
                return Err(err);
            }
            Err(err) => return Err(Self::Error::SerdeJson { err, text: body }),
        };

        let res = serde_json::from_str(raw.get()).map_err(|err| Self::Error::SerdeJson {
            err,
            text: raw.to_string(),
        })?;

        Ok(res)
    }
}

#[derive(Debug)]
pub struct SimpleJsonRpcRetryPolicy;

impl RetryPolicy<JsonRpcProviderClientError> for SimpleJsonRpcRetryPolicy {
    fn should_retry(&self, error: &JsonRpcProviderClientError) -> bool {
        todo!()
    }

    fn backoff_hint(&self, error: &JsonRpcProviderClientError) -> Option<Duration> {
        todo!()
    }
}
