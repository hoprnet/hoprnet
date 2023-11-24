use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::errors::HttpRequestError;
use crate::HttpPostRequestor;

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use wasm_bindgen::{prelude::wasm_bindgen, JsCast};

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen(module = "@hoprnet/hopr-utils")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn http_post(url: &str, json_data: &str) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NodeJsHttpError {
    pub msg: String,
    pub status: i16,
}

impl From<NodeJsHttpError> for HttpRequestError {
    fn from(value: NodeJsHttpError) -> Self {
        if value.status >= 400 {
            HttpRequestError::HttpError(value.status as u16)
        } else if value.status > 0 {
            HttpRequestError::UnknownError(format!("code: {}, {}", value.status, value.msg))
        } else {
            let msg = value.msg.to_lowercase();
            if msg.contains("timeout") || msg.contains("timed out") {
                HttpRequestError::Timeout
            } else {
                HttpRequestError::UnknownError(msg)
            }
        }
    }
}

pub struct NodeJsHttpPostRequestor;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl HttpPostRequestor for NodeJsHttpPostRequestor {
    #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
    async fn http_post(&self, url: &str, json_data: &str) -> Result<String, HttpRequestError> {
        match http_post(url, json_data).await {
            Ok(s) => {
                match s.dyn_ref::<js_sys::JsString>() {
                    Some(s) => Ok(format!("{s}")), // must call to_string like this due to name clash
                    None => Err(HttpRequestError::InterfaceError("cannot cast result to string".into())),
                }
            }
            Err(err) => {
                let error = serde_wasm_bindgen::from_value::<NodeJsHttpError>(err)
                    .map_err(|e| HttpRequestError::InterfaceError(format!("failed to deserialize error object: {e}")))?;
                Err(error.into())
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn http_post(&self, _url: &str, _json_data: &str) -> Result<String, HttpRequestError> {
        unimplemented!("not implemented on non-wasm or non-Node.js targets")
    }
}
