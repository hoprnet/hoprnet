use blake2::{Blake2s256, Digest};

/// Takes a ping request message and returns a ping response message
pub fn generate_ping_response(req: Box<[u8]>) -> Box<[u8]> {
    let mut hasher = Blake2s256::new();

    hasher.update(req);

    hasher.finalize().to_vec().into_boxed_slice()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeartbeatConfig {
    pub max_parallel_heartbeats: usize,
    pub heartbeat_variance: f32,
    pub heartbeat_interval: u32,
    pub heartbeat_threshold: u64,
    pub network_quality_threshold: f32,
}

#[derive(Debug, Copy, Clone)]
struct Heartbeat {
    peers: i32
}

#[cfg(feature = "wasm")]
pub mod wasm {
    // use super::reply_to_ping;
    use futures::stream::{Stream, StreamExt};
    use js_sys::AsyncIterator;
    use utils_misc::async_iterable::wasm::{to_box_u8_stream, to_jsvalue_stream};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::stream::JsStream;

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

    // #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
    // extern "C" {
    //     #[wasm_bindgen(catch)]
    //     pub async fn send_message(msg: Box<[u8]>, recipient: &str) -> Result<js_sys::JsValue, js_sys::JsValue>;
    // }


    //
    // impl Heartbeat {
    //     pub fn start(&self) {
    //         // TODO: set WASM based interval trigger for heartbeat
    //     }
    //
    //     async fn check(&self) {
    //         let _threshold = current_timestamp();    // FIX: self.config.heartbeat_threshold;
    //         // TODO: log(`Checking nodes since ${thresholdTime} (${new Date(thresholdTime).toLocaleString()})`)
    //     }
    // }
}
