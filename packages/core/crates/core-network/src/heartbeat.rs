use blake2::{Blake2s256, Digest};

/// Takes a ping request message and returns a ping response message
pub fn generate_ping_response(req: Box<[u8]>) -> Box<[u8]> {
    let mut hasher = Blake2s256::new();

    hasher.update(req);

    hasher.finalize().to_vec().into_boxed_slice()
}

/// Configuration of the Heartbeat
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeartbeatConfig {
    pub max_parallel_heartbeats: usize,
    pub heartbeat_variance: f32,
    pub heartbeat_interval: u32,
    pub heartbeat_threshold: u64,
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::heartbeat::HeartbeatConfig;
    use futures::stream::{Stream, StreamExt};
    use js_sys::AsyncIterator;
    use utils_misc::async_iterable::wasm::{to_box_u8_stream, to_jsvalue_stream};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::stream::JsStream;

    #[wasm_bindgen]
    impl HeartbeatConfig {
        #[wasm_bindgen]
        pub fn build(
            max_parallel_heartbeats: usize,
            heartbeat_variance: f32,
            heartbeat_interval: u32,
            heartbeat_threshold: u64,
        ) -> Self {
            Self {
                max_parallel_heartbeats,
                heartbeat_variance,
                heartbeat_interval,
                heartbeat_threshold,
            }
        }
    }

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
    pub fn reply_to_ping(stream: AsyncIterator) -> Result<AsyncIterableHelperCoreHeartbeat, JsValue> {
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
