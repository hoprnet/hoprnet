use std::cell::RefCell;
use utils_misc::{
    streaming_iterable::{JsStreamingIterable, StreamingIterable},
    traits::DuplexStream,
};

use crate::messaging::{
    ControlMessage::{self, Ping},
    PingMessage,
};
use futures::{SinkExt, StreamExt};
use pin_project_lite::pin_project;
use utils_log::error;
use utils_types::traits::BinarySerializable;

/// Configuration of the Heartbeat
#[derive(Clone, Debug, PartialEq)]
pub struct HeartbeatConfig {
    pub max_parallel_heartbeats: usize,
    pub heartbeat_variance: u32,
    pub heartbeat_interval: u32,
    pub heartbeat_threshold: u64,
    pub environment_id: String,
    pub normalized_version: String,
}

impl HeartbeatConfig {
    pub fn new(
        max_parallel_heartbeats: usize,
        heartbeat_variance: u32,
        heartbeat_interval: u32,
        heartbeat_threshold: u64,
        environment_id: String,
        normalized_version: String,
    ) -> Self {
        Self {
            max_parallel_heartbeats,
            heartbeat_interval,
            heartbeat_threshold,
            heartbeat_variance,
            environment_id,
            normalized_version,
        }
    }
}

pub struct Heartbeat {
    ended: RefCell<bool>,
    config: HeartbeatConfig,
    protocols: [String; 2],
}

impl Heartbeat {
    pub fn new(config: HeartbeatConfig) -> Self {
        Self {
            ended: RefCell::new(false),
            protocols: [
                // new
                format!(
                    "/hopr/{}/heartbeat/{}",
                    &config.environment_id, &config.normalized_version
                ),
                // deprecated
                format!("/hopr/{}/heartbeat", &config.environment_id),
            ],
            config,
        }
    }

    pub fn has_ended(&self) -> bool {
        *self.ended.borrow()
    }

    pub fn set_ended(&self) -> Result<(), String> {
        match self.ended.try_borrow_mut() {
            Ok(mut x) => {
                *x = true;
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn get_config(&self) -> HeartbeatConfig {
        self.config.to_owned()
    }

    pub fn get_protocols(&self) -> Vec<String> {
        Vec::from(self.protocols.to_owned())
    }
}

pin_project! {
    pub struct HeartbeatRequest<St> {
        #[pin]
        stream: St,
    }
}

impl From<JsStreamingIterable> for HeartbeatRequest<StreamingIterable> {
    fn from(x: JsStreamingIterable) -> Self {
        Self { stream: x.into() }
    }
}

impl<St: DuplexStream> HeartbeatRequest<St> {
    pub async fn handle(&mut self) -> () {
        while let Some(msg) = self.stream.next().await {
            match msg {
                Ok(msg) => match PingMessage::deserialize(&msg) {
                    Ok(valid_incoming) => match ControlMessage::generate_pong_response(&Ping(valid_incoming)) {
                        Ok(response) => match response {
                            ControlMessage::Ping(_) => continue,
                            ControlMessage::Pong(x) => match self.stream.send(x.serialize()).await {
                                Ok(()) => continue,
                                Err(e) => {
                                    error!("{}", e);
                                    break;
                                }
                            },
                        },
                        Err(e) => {
                            error!("{}", e);
                            continue;
                        }
                    },
                    Err(e) => {
                        error!("{}", e);
                        continue;
                    }
                },
                Err(e) => {
                    error!("{}", e);
                    continue;
                }
            };
        }

        match self.stream.close().await {
            Ok(()) => (),
            Err(e) => error!("{}", e),
        };
    }
}
