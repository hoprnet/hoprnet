use async_trait::async_trait;
use ethers_providers::{JsonRpcClient, JsonRpcError, RetryPolicy};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::errors::{HttpRequestError, JsonRpcProviderClientError};
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
    /// Creates the client given the `HttpPostRequestor`
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
        Self {
            id: AtomicU64::new(1),
            url: self.url.clone(),
            requestor: self.requestor.clone(),
        }
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
        // Create and serialize the Request object
        let next_id = self.id.fetch_add(1, Ordering::SeqCst);
        let payload = Request::new(next_id, method, params);
        let serialized_payload = serde_json::to_string(&payload).map_err(|err| Self::Error::SerdeJson {
            err,
            text: "cannot serialize payload".into(),
        })?;

        // Perform the actual request
        let body = self.requestor.http_post(self.url.as_ref(), &serialized_payload).await?;

        // First deserialize the Response object
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

        // Next, deserialize the data out of the Response object
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
        // There are 3 error cases:
        // - serialization error of request/response
        // - JSON RPC error
        // - HTTP error
        match error {
            JsonRpcProviderClientError::SerdeJson { text, .. } => {
                // some providers send invalid JSON RPC in the error case (no `id:u64`), but the
                // text should be a `JsonRpcError`
                #[derive(Deserialize)]
                struct Resp {
                    error: JsonRpcError,
                }

                if let Ok(resp) = serde_json::from_str::<Resp>(text) {
                    return self.should_retry_on_json_rpc_error(&resp.error);
                }
                false
            }
            JsonRpcProviderClientError::JsonRpcError(JsonRpcError { code, message, .. }) => {
                // Alchemy throws it this way
                if *code == 429 {
                    return true;
                }

                // This is an Infura error code for `exceeded project rate limit`
                if *code == -32005 {
                    return true;
                }

                // Alternative alchemy error for specific IPs
                if *code == -32016 && message.contains("rate limit") {
                    return true;
                }

                match message.as_str() {
                    // This is commonly thrown by infura and is apparently a load balancer issue, see also <https://github.com/MetaMask/metamask-extension/issues/7234>
                    "header not found" => true,
                    // also thrown by Infura if out of budget for the day and rate-limited
                    "daily request count exceeded, request rate limited" => true,
                    _ => false,
                }
            }
            JsonRpcProviderClientError::BackendError(err) => {
                err == HttpRequestError::HttpError(429) ||  // too many requests should be retried
                err == HttpRequestError::Timeout // timeouts are retried as well
            }
        }
    }

    fn backoff_hint(&self, error: &JsonRpcProviderClientError) -> Option<Duration> {
        if let JsonRpcProviderClientError::JsonRpcError(JsonRpcError { data, .. }) = error {
            let data = data.as_ref()?;

            // if daily rate limit exceeded, infura returns the requested backoff in the error
            // response
            let backoff_seconds = &data["rate"]["backoff_seconds"];
            // infura rate limit error
            if let Some(seconds) = backoff_seconds.as_u64() {
                return Some(Duration::from_secs(seconds));
            }
            if let Some(seconds) = backoff_seconds.as_f64() {
                return Some(Duration::from_secs(seconds as u64 + 1));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {}
