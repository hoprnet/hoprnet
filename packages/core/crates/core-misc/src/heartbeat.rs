use blake2::{Blake2s256, Digest};

/// Takes a ping request message and returns a ping response message
pub fn generate_ping_response(req: Box<[u8]>) -> Box<[u8]> {
    let mut hasher = Blake2s256::new();

    hasher.update(req);

    hasher.finalize().to_vec().into_boxed_slice()
}

#[cfg(feature = "wasm")]
pub mod wasm {
    // use super::reply_to_ping;
    use futures::stream::{Stream, StreamExt};
    use js_sys::AsyncIterator;
    use utils_misc::async_iterable::wasm::{to_box_u8_stream, to_jsvalue_stream};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::stream::JsStream;

    // Copied because wasm_bindgen does not support lifetimes yet
    #[wasm_bindgen]
    pub struct AsyncIterableHelperCoreHeartbeat {
        stream: Box<dyn Stream<Item = Result<Box<[u8]>, String>> + Unpin>,
    }

    #[wasm_bindgen]
    impl AsyncIterableHelperCoreHeartbeat {
        pub async fn next(&mut self) -> Result<JsValue, JsValue> {
            to_jsvalue_stream(self.stream.as_mut().next().await)
        }
    }

    #[wasm_bindgen]
    pub fn reply_to_ping(
        stream: AsyncIterator,
    ) -> Result<AsyncIterableHelperCoreHeartbeat, JsValue> {
        let stream = JsStream::from(stream)
            .map(to_box_u8_stream)
            .map(|req| req.map(super::generate_ping_response));

        Ok(AsyncIterableHelperCoreHeartbeat {
            stream: Box::new(stream),
        })
    }

    /// Used to pre-compute ping responses
    #[wasm_bindgen]
    pub fn generate_ping_response(u8a: &[u8]) -> Box<[u8]> {
        super::generate_ping_response(u8a.into())
    }
}
