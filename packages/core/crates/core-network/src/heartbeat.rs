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
    use crate::messaging::ControlMessage;
    use futures::stream::{Stream, StreamExt};
    use js_sys::AsyncIterator;
    use utils_misc::async_iterable::wasm::{to_box_u8_stream, to_jsvalue_stream};
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::traits::BinarySerializable;
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
        pub async fn next(&mut self) -> JsResult<JsValue> {
            to_jsvalue_stream(self.stream.as_mut().next().await)
        }
    }

    /// Used to pre-compute ping responses
    #[wasm_bindgen]
    pub fn generate_ping_response(u8a: &[u8]) -> JsResult<Box<[u8]>> {
        ok_or_jserr!(ControlMessage::deserialize(u8a))
            .and_then(|req| ok_or_jserr!(ControlMessage::generate_pong_response(&req)))
            .map(|msg| msg.serialize())
    }

    #[wasm_bindgen]
    pub fn reply_to_ping(stream: AsyncIterator) -> JsResult<AsyncIterableHelperCoreHeartbeat> {
        let stream = JsStream::from(stream).map(to_box_u8_stream).map(|res| {
            res.and_then(|rq| {
                generate_ping_response(rq.as_ref()).map_err(|e| e.as_string().unwrap_or("not a string".into()))
            })
        });

        Ok(AsyncIterableHelperCoreHeartbeat {
            stream: Box::new(stream),
        })
    }
}
