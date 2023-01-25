use blake2::{Blake2s256, Digest};

/// Takes a ping request message and returns a ping response message
pub fn generate_ping_response(req: Vec<u8>) -> Vec<u8> {
    let mut hasher = Blake2s256::new();

    hasher.update(req.as_slice());

    hasher.finalize().to_vec()
}

pub mod wasm {
    // use super::reply_to_ping;
    use futures::stream::{Stream, StreamExt};
    use js_sys::{AsyncIterator, Uint8Array};
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
            .map(|req| req.map(super::generate_ping_response));

        Ok(AsyncIterableHelperCoreHeartbeat {
            stream: Box::new(stream),
        })
    }

    /// Used to pre-compute ping responses
    #[wasm_bindgen]
    pub fn generate_ping_response(u8a: Uint8Array) -> Uint8Array {
        let ping_reponse = super::generate_ping_response(u8a.to_vec());

        let res = Uint8Array::new_with_length(32);
        res.copy_from(ping_reponse.as_slice());

        res
    }
}
