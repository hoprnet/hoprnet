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

/// Retry policy to use with the RPC Provider
#[derive(Debug)]
pub struct SimpleJsonRpcRetryPolicy;

impl SimpleJsonRpcRetryPolicy {
    fn should_retry_on_json_rpc_error(&self, err: &JsonRpcError) -> bool {
        let JsonRpcError { code, message, .. } = err;

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
}

impl RetryPolicy<JsonRpcProviderClientError> for SimpleJsonRpcRetryPolicy {
    fn should_retry(&self, error: &JsonRpcProviderClientError) -> bool {
        // There are 3 error cases:
        // - serialization errors of request/response
        // - JSON RPC errors
        // - HTTP & transport errors
        match error {
            // Serialization error
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

            // JSON-RPC error
            JsonRpcProviderClientError::JsonRpcError(err) => self.should_retry_on_json_rpc_error(err),

            // HTTP & transport errors: Only retry HTTP Too Many Requests and Timeouts
            JsonRpcProviderClientError::BackendError(HttpRequestError::Timeout)
            | JsonRpcProviderClientError::BackendError(HttpRequestError::HttpError(429)) => true,

            // Everything else is not retried and immediately considered an error
            _ => false,
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
pub mod tests {
    use async_trait::async_trait;
    use core_ethereum_types::create_anvil;
    use ethers_providers::JsonRpcClient;
    use futures::FutureExt;
    use reqwest::header::{HeaderValue, CONTENT_TYPE};
    use std::time::Duration;

    use crate::client::JsonRpcProviderClient;
    use crate::errors::{HttpRequestError, JsonRpcProviderClientError};
    use crate::{HttpPostRequestor, MockHttpPostRequestor};

    #[derive(Debug)]
    pub struct ReqwestRequestor(reqwest::Client);

    impl Default for ReqwestRequestor {
        fn default() -> Self {
            Self(
                reqwest::Client::builder()
                    .timeout(Duration::from_secs(5))
                    .build()
                    .expect("failed to build Reqwest client"),
            )
        }
    }

    #[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
    #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
    impl HttpPostRequestor for ReqwestRequestor {
        async fn http_post(&self, url: &str, json_data: &str) -> Result<String, HttpRequestError> {
            self.0
                .post(url)
                .header(CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap())
                .body(Vec::from(json_data.as_bytes()))
                .send()
                .await
                .map_err(|e| {
                    if e.is_status() {
                        HttpRequestError::HttpError(e.status().unwrap().as_u16())
                    } else if e.is_timeout() {
                        HttpRequestError::Timeout
                    } else {
                        HttpRequestError::UnknownError(e.to_string())
                    }
                })?
                .text()
                .await
                .map_err(|e| HttpRequestError::InterfaceError(format!("body: {}", e.to_string())))
        }
    }

    #[tokio::test]
    async fn test_client_should_get_block_number() {
        let block_time = Duration::from_secs(1);

        let anvil = create_anvil(Some(block_time));
        let client = JsonRpcProviderClient::new(&anvil.endpoint(), ReqwestRequestor::default());

        let mut last_number = 0;

        for _ in 0..3 {
            tokio::time::sleep(block_time).await;

            let number: ethers::types::U64 = client
                .request("eth_blockNumber", ())
                .await
                .expect("should get block number");

            assert!(number.as_u64() > last_number, "next block number must be greater");
            last_number = number.as_u64();
        }
    }

    #[tokio::test]
    async fn test_client_should_fail_on_malformed_request() {
        let anvil = create_anvil(None);
        let client = JsonRpcProviderClient::new(&anvil.endpoint(), ReqwestRequestor::default());

        let err = client
            .request::<_, ethers::types::U64>("eth_blockNumber_bla", ())
            .await
            .expect_err("expected error");
        assert!(matches!(err, JsonRpcProviderClientError::JsonRpcError(..)));
    }

    #[tokio::test]
    async fn test_client_should_fail_on_malformed_response() {
        let mut mock_requestor = MockHttpPostRequestor::new();

        mock_requestor
            .expect_http_post()
            .once()
            .withf(|url, _| url == "localhost")
            .returning(|_, _| futures::future::ok("}{malformed".to_string()).boxed());

        let client = JsonRpcProviderClient::new("localhost".into(), mock_requestor);

        let err = client
            .request::<_, ethers::types::U64>("eth_blockNumber", ())
            .await
            .expect_err("expected error");
        assert!(matches!(err, JsonRpcProviderClientError::SerdeJson { .. }));
    }
}
