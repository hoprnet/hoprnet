use std::cell::RefCell;
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
use core::pin::Pin;
use futures::{
    stream::FusedStream,
    task::{Context, Poll},
    Future,
};
use pin_project_lite::pin_project;
use utils_types::traits::BinarySerializable;

/// Configuration of the Heartbeat
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HeartbeatConfig {
    pub max_parallel_heartbeats: usize,
    pub heartbeat_variance: u32,
    pub heartbeat_interval: u32,
    pub heartbeat_threshold: u64,
}

impl HeartbeatConfig {
    pub fn new(
        max_parallel_heartbeats: usize,
        heartbeat_variance: u32,
        heartbeat_interval: u32,
        heartbeat_threshold: u64,
    ) -> Self {
        Self {
            max_parallel_heartbeats,
            heartbeat_interval,
            heartbeat_threshold,
            heartbeat_variance,
        }
    }
}

pub struct Heartbeat {
    ended: RefCell<bool>,
    config: HeartbeatConfig,
}

impl Heartbeat {
    pub fn new(config: HeartbeatConfig) -> Self {
        Self {
            ended: RefCell::new(false),
            config,
        }
    }
    pub fn start(&mut self) {}

    pub fn has_ended(&self) -> bool {
        *self.ended.borrow()
    }

    pub fn set_ended(&self) -> Result<(), String> {
        match self.ended.try_borrow_mut() {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn get_config(&self) -> HeartbeatConfig {
        self.config
    }
}
pin_project! {
    pub struct HeartbeatRequest<St> {
        #[pin]
        stream: St,
        buffered: Option<Box<[u8]>>,
        ended: bool
    }
}

impl From<JsStreamingIterable> for HeartbeatRequest<StreamingIterable> {
    fn from(x: JsStreamingIterable) -> Self {
        Self {
            stream: x.into(),
            buffered: None,
            ended: false,
        }
    }
}
impl<St: DuplexStream> Future for HeartbeatRequest<St> {
    type Output = Result<(), String>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        if let Some(item) = this.buffered.take() {
            return match this.stream.as_mut().poll_ready(cx) {
                Poll::Ready(_) => match this.stream.as_mut().start_send(item) {
                    Ok(()) => Poll::Ready(Ok(())),
                    Err(e) => {
                        *this.ended = true;
                        Poll::Ready(Err(e))
                    }
                },
                Poll::Pending => Poll::Pending,
            };
        }

        if *this.ended || this.stream.is_terminated() {
            return Poll::Ready(Ok(()));
        }

        match this.stream.as_mut().poll_next(cx) {
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
                        Ping(_) => panic!("must not happen"),
                        Pong(x) => {
                            *this.buffered = Some(x.serialize());

                            match this.stream.as_mut().poll_ready(cx) {
                                Poll::Pending => Poll::Pending,
                                Poll::Ready(Err(e)) => {
                                    *this.ended = true;
                                    Poll::Ready(Err(e))
                                }
                                Poll::Ready(Ok(())) => {
                                    match this.stream.as_mut().start_send(this.buffered.take().unwrap()) {
                                        Err(e) => {
                                            *this.ended = true;
                                            Poll::Ready(Err(e))
                                        }
                                        Ok(()) => Poll::Ready(Ok(())),
                                    }
                                }
                            }
                        }
                    },
                },
            },
        }
    }
}
