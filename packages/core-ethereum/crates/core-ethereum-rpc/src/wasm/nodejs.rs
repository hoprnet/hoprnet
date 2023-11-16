use crate::wasm::{HttpPostRequestor, HttpRequestError};
use async_trait::async_trait;

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use wasm_bindgen::{prelude::wasm_bindgen, JsCast};

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen(module = "@hoprnet/hopr-utils")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn http_post(url: &str, json_data: &str) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
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
                    Some(s) => Ok(format!("{}", s)), // cannot call to_string() here
                    None => Err(HttpRequestError::InterfaceError("cannot cast result to string".into())),
                }
            }
            Err(_) => Err(HttpRequestError::InterfaceError("...".into())), // TODO: properly distinguish errors
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn http_post(&self, _url: &str, _json_data: &str) -> Result<String, HttpRequestError> {
        unimplemented!("not implemented on non-wasm or non-Node.js targets")
    }
}
