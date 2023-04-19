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
    use crate::errors::NetworkingError;
    use crate::heartbeat::HeartbeatConfig;
    use crate::messaging::{ControlMessage, ControlMessage::Ping, PingMessage};
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
        ok_or_jserr!(PingMessage::deserialize(u8a)
            .map_err(|_| NetworkingError::DecodingError)
            .and_then(|req| ControlMessage::generate_pong_response(&Ping(req)))
            .and_then(|msg| msg.get_ping_message().map(PingMessage::serialize)))
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

use utils_misc::{
    streaming_iterable::{JsStreamingIterable, StreamingIterable},
    traits::DuplexStream,
};

use crate::{
    errors::NetworkingError,
    messaging::{
        ControlMessage::{self, Ping, Pong},
        PingMessage,
    },
};
use futures::{
    stream::FusedStream,
    task::{Context, Poll},
    Future, Sink, SinkExt, Stream,
};
use pin_project_lite::pin_project;
use std::pin::Pin;
use utils_types::traits::BinarySerializable;

pin_project! {
    pub struct HeartbeatRequest<St> {
        #[pin]
        stream: St,
        buffered: Option<Box<[u8]>>,
        ended: bool
    }
}

impl From<JsStreamingIterable> for HeartbeatRequest<StreamingIterable> {
    fn from(streaming_iterable: JsStreamingIterable) -> Self {
        HeartbeatRequest::new(streaming_iterable)
    }
}

impl HeartbeatRequest<StreamingIterable> {
    pub fn new(stream: JsStreamingIterable) -> Self {
        Self {
            stream: StreamingIterable::from(stream),
            ended: false,
            buffered: None,
        }
    }
}

impl<St: DuplexStream> Stream for HeartbeatRequest<St> {
    type Item = Result<Box<[u8]>, String>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        if *this.ended {
            return Poll::Ready(None);
        }

        match this.stream.poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => {
                *this.ended = true;
                Poll::Ready(None)
            }
            Poll::Ready(Some(Err(e))) => {
                *this.ended = true;
                Poll::Ready(Some(Err(e)))
            }
            Poll::Ready(Some(Ok(msg))) => match PingMessage::deserialize(&msg) {
                Err(_) => {
                    *this.ended = true;
                    return Poll::Ready(Some(Err(NetworkingError::DecodingError.to_string())));
                }
                Ok(req) => match ControlMessage::generate_pong_response(&Ping(req)) {
                    Err(e) => {
                        *this.ended = true;
                        return Poll::Ready(Some(Err(e.to_string())));
                    }
                    Ok(res) => match res {
                        Ping(x) => panic!("must not happen"),
                        Pong(x) => Poll::Ready(Some(Ok(x.serialize()))),
                    },
                },
            },
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

impl<St: DuplexStream> FusedStream for HeartbeatRequest<St> {
    fn is_terminated(&self) -> bool {
        self.ended
    }
}

impl<St: DuplexStream> Sink<Box<[u8]>> for HeartbeatRequest<St> {
    type Error = String;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();

        this.stream.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: Box<[u8]>) -> Result<(), Self::Error> {
        let this = self.project();

        this.stream.start_send(item)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();

        this.stream.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();

        this.stream.poll_close(cx)
    }
}

impl<St: DuplexStream> Future for HeartbeatRequest<St> {
    type Output = Result<(), String>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if let Some(item) = this.buffered {
            match this.stream.poll_ready(cx) {
                Poll::Ready(_) => {
                    this.stream.start_send(this.buffered.take().unwrap());
                    return Poll::Ready(Ok(()));
                }
                Poll::Pending => return Poll::Pending,
            }
        }

        if this.stream.is_terminated() {
            return Poll::Ready(Ok(()));
        }

        match this.stream.poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => {
                *this.ended = true;
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Some(Err(e))) => {
                *this.ended = true;
                Poll::Ready(Err(e))
            }
            Poll::Ready(Some(Ok(msg))) => match PingMessage::deserialize(&msg) {
                Err(_) => {
                    *this.ended = true;
                    return Poll::Ready(Err(NetworkingError::DecodingError.to_string()));
                }
                Ok(req) => match ControlMessage::generate_pong_response(&Ping(req)) {
                    Err(e) => {
                        *this.ended = true;
                        Poll::Ready(Err(e.to_string()))
                    }
                    Ok(res) => match res {
                        Ping(x) => panic!("must not happen"),
                        Pong(x) => {
                            *this.buffered = Some(x.serialize());

                            match this.stream.poll_ready(cx) {
                                Poll::Pending => Poll::Pending,
                                Poll::Ready(Err(e)) => {
                                    *this.ended = true;
                                    Poll::Ready(Err(e))
                                }
                                Poll::Ready(Ok(())) => match this.stream.start_send(this.buffered.take().unwrap()) {
                                    Err(e) => {
                                        *this.ended = true;
                                        Poll::Ready(Err(e))
                                    }
                                    Ok(()) => Poll::Ready(Ok(())),
                                },
                            }
                        }
                    },
                },
            },
        }
    }
}
