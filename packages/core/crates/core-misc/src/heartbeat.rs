use blake2::{Blake2s256, Digest};

pub fn reply_to_ping(req: Result<Vec<u8>, String>) -> Result<Vec<u8>, String> {
    let mut hasher = Blake2s256::new();

    hasher.update(req.unwrap().as_slice());

    Ok(hasher.finalize().to_vec())
}

pub mod wasm {
    // use super::reply_to_ping;
    use futures::stream::{Stream, StreamExt};
    use js_sys::AsyncIterator;
    use utils_misc::async_iterable::wasm::{to_jsvalue_stream, to_vec_u8_stream};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::stream::JsStream;

    // Copied because wasm_bindgen does not support lifetimes yet
    #[wasm_bindgen]
    pub struct AsyncIterableHelperCoreHeartbeat {
        stream: Box<dyn Stream<Item = Result<Vec<u8>, String>> + Unpin>,
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
            .map(to_vec_u8_stream)
            .map(super::reply_to_ping);

        Ok(AsyncIterableHelperCoreHeartbeat {
            stream: Box::new(stream),
        })
    }
}
