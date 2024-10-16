// Copyright (c) 2023 Protocol Labs.
//
// Licensed under the Apache License, Version 2.0 or MIT license, at your option.
//
// A copy of the Apache License, Version 2.0 is included in the software as
// LICENSE-APACHE and a copy of the MIT license is included in the software
// as LICENSE-MIT. You may also obtain a copy of the Apache License, Version 2.0
// at https://www.apache.org/licenses/LICENSE-2.0 and a copy of the MIT license
// at https://opensource.org/licenses/MIT.

//! Connection round-trip time measurement

use std::sync::Arc;

use instant::{Duration, Instant};
use parking_lot::Mutex;

use crate::connection::Action;
use crate::frame::{header::Ping, Frame};

const PING_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Clone, Debug)]
pub(crate) struct Rtt(Arc<Mutex<RttInner>>);

impl Rtt {
    pub(crate) fn new() -> Self {
        Self(Arc::new(Mutex::new(RttInner {
            rtt: None,
            state: RttState::Waiting {
                next: Instant::now(),
            },
        })))
    }

    pub(crate) fn next_ping(&mut self) -> Option<Frame<Ping>> {
        let state = &mut self.0.lock().state;

        match state {
            RttState::AwaitingPong { .. } => return None,
            RttState::Waiting { next } => {
                if *next > Instant::now() {
                    return None;
                }
            }
        }

        let nonce = rand::random();
        *state = RttState::AwaitingPong {
            sent_at: Instant::now(),
            nonce,
        };
        log::debug!("sending ping {nonce}");
        Some(Frame::ping(nonce))
    }

    pub(crate) fn handle_pong(&mut self, received_nonce: u32) -> Action {
        let inner = &mut self.0.lock();

        let (sent_at, expected_nonce) = match inner.state {
            RttState::Waiting { .. } => {
                log::error!("received unexpected pong {received_nonce}");
                return Action::Terminate(Frame::protocol_error());
            }
            RttState::AwaitingPong { sent_at, nonce } => (sent_at, nonce),
        };

        if received_nonce != expected_nonce {
            log::error!("received pong with {received_nonce} but expected {expected_nonce}");
            return Action::Terminate(Frame::protocol_error());
        }

        let rtt = sent_at.elapsed();
        inner.rtt = Some(rtt);
        log::debug!("received pong {received_nonce}, estimated round-trip-time {rtt:?}");

        inner.state = RttState::Waiting {
            next: Instant::now() + PING_INTERVAL,
        };

        return Action::None;
    }

    pub(crate) fn get(&self) -> Option<Duration> {
        self.0.lock().rtt
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Rtt {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self(Arc::new(Mutex::new(RttInner::arbitrary(g))))
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(Clone))]
struct RttInner {
    state: RttState,
    rtt: Option<Duration>,
}

#[cfg(test)]
impl quickcheck::Arbitrary for RttInner {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self {
            state: RttState::arbitrary(g),
            rtt: if bool::arbitrary(g) {
                Some(Duration::arbitrary(g))
            } else {
                None
            },
        }
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(Clone))]
enum RttState {
    AwaitingPong { sent_at: Instant, nonce: u32 },
    Waiting { next: Instant },
}

#[cfg(test)]
impl quickcheck::Arbitrary for RttState {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        if bool::arbitrary(g) {
            RttState::AwaitingPong {
                sent_at: Instant::now(),
                nonce: u32::arbitrary(g),
            }
        } else {
            RttState::Waiting {
                next: Instant::now(),
            }
        }
    }
}
