//! Test helpers for `hopr-transport-session`.
//!
//! This module is compiled unconditionally so integration tests can always use it
//! without requiring a test build of the rlib.

#![allow(clippy::type_complexity)]

use futures::{StreamExt, future::BoxFuture};
use hopr_api::types::internal::routing::DestinationRouting;
use hopr_protocol_app::prelude::*;

use crate::{errors, types::HoprStartProtocol};

#[async_trait::async_trait]
pub trait SendMsg: Send + Sync {
    async fn send_message(&self, routing: DestinationRouting, data: ApplicationDataOut) -> errors::Result<()>;
}

// ---------------------------------------------------------------------------
// Mock
// ---------------------------------------------------------------------------

/// Call-count expectation.
#[derive(Clone, Debug)]
pub enum Times {
    /// Exactly N calls.
    Exact(usize),
    /// At least N calls (from `RangeFrom`).
    AtLeast(usize),
}

impl Times {
    fn remaining(&self) -> usize {
        match self {
            Times::Exact(n) => *n,
            Times::AtLeast(_) => 0,
        }
    }
}

impl From<usize> for Times {
    fn from(n: usize) -> Self {
        Times::Exact(n)
    }
}

impl From<std::ops::RangeFrom<usize>> for Times {
    fn from(r: std::ops::RangeFrom<usize>) -> Self {
        Times::AtLeast(r.start)
    }
}

impl From<std::ops::RangeFrom<i32>> for Times {
    fn from(r: std::ops::RangeFrom<i32>) -> Self {
        Times::AtLeast(r.start as usize)
    }
}

/// A recorded expectation.
struct Expectation {
    matcher: std::sync::Arc<dyn Fn(&DestinationRouting, &ApplicationDataOut) -> bool + Send + Sync>,
    /// Invoked at each matching call to `send_message` to produce the response future.
    factory: std::sync::Arc<
        dyn Fn(DestinationRouting, ApplicationDataOut) -> BoxFuture<'static, Result<(), errors::TransportSessionError>>
            + Send
            + Sync,
    >,
    times: Times,
    /// Only used when `times` is `Exact` — counts remaining calls.
    times_remaining: usize,
    /// Tracks actual matched calls for `AtLeast` expectations; validated on Drop.
    matched_calls: usize,
}

impl Expectation {
    fn new(
        matcher: std::sync::Arc<dyn Fn(&DestinationRouting, &ApplicationDataOut) -> bool + Send + Sync>,
        factory: std::sync::Arc<
            dyn Fn(
                    DestinationRouting,
                    ApplicationDataOut,
                ) -> BoxFuture<'static, Result<(), errors::TransportSessionError>>
                + Send
                + Sync,
        >,
        times: Times,
        times_remaining: usize,
    ) -> Self {
        Self {
            matcher,
            factory,
            times,
            times_remaining,
            matched_calls: 0,
        }
    }
}

struct MsgSenderInner {
    expectations: Vec<Expectation>,
    call_count: usize,
}

/// An expectation-based mock of [`SendMsg`].
///
/// Clones share the same mutable state — expectations and call counts
/// are kept in an `Arc` so that all clones (e.g. the instance passed
/// into a spawned task) observe the same calls and consume the same
/// expectation slots.
pub struct MsgSender {
    inner: std::sync::Arc<parking_lot::Mutex<MsgSenderInner>>,
}

impl Clone for MsgSender {
    fn clone(&self) -> Self {
        Self {
            inner: std::sync::Arc::clone(&self.inner),
        }
    }
}

impl Clone for Expectation {
    fn clone(&self) -> Self {
        Self {
            matcher: std::sync::Arc::clone(&self.matcher),
            factory: std::sync::Arc::clone(&self.factory),
            times: self.times.clone(),
            times_remaining: self.times_remaining,
            matched_calls: self.matched_calls,
        }
    }
}

impl std::fmt::Debug for MsgSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MsgSender").finish()
    }
}

impl MsgSender {
    pub fn new() -> Self {
        Self {
            inner: std::sync::Arc::new(parking_lot::Mutex::new(MsgSenderInner {
                expectations: Vec::new(),
                call_count: 0,
            })),
        }
    }

    /// Set up an expectation.  Use the returned struct to configure `.withf()`, `.returning()`, etc.
    pub fn expect_send_message(&mut self) -> Expect<'_> {
        Expect {
            sender: self,
            matcher: None,
            times: Times::Exact(1),
        }
    }

    fn add_expectation(&self, expectation: Expectation) {
        self.inner.lock().expectations.push(expectation);
    }
}

impl Default for MsgSender {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MsgSender {
    fn drop(&mut self) {
        // Only the last reference to the inner state validates expectations
        // so that all clones share the same counter.
        let inner = match std::sync::Arc::try_unwrap(self.inner.clone()) {
            Ok(inner) => inner.into_inner(),
            Err(_) => return,
        };
        let mut errors = Vec::new();
        for (i, e) in inner.expectations.iter().enumerate() {
            match &e.times {
                Times::Exact(n) if e.times_remaining > 0 => {
                    errors.push(format!(
                        "expectation[{}]: expected {} more call(s)",
                        i, e.times_remaining
                    ));
                }
                Times::AtLeast(n) if e.matched_calls < *n => {
                    errors.push(format!(
                        "expectation[{}]: expected at least {} call(s), got {}",
                        i, n, e.matched_calls
                    ));
                }
                _ => {}
            }
        }
        if !errors.is_empty() {
            panic!("MsgSender expectations not fulfilled:\n  {}", errors.join("\n  "));
        }
    }
}

#[async_trait::async_trait]
impl SendMsg for MsgSender {
    async fn send_message(&self, routing: DestinationRouting, data: ApplicationDataOut) -> errors::Result<()> {
        let response = {
            let mut guard = self.inner.lock();
            guard.call_count += 1;
            let call_count = guard.call_count;

            // Find the expectation to handle this call:
            // 1. rposition finds the LAST Exact expectation that still has remaining calls. This ensures Exact
            //    expectations are consumed in registration order while AtLeast expectations silently absorb any
            //    overflow calls.
            // 2. If no Exact matches, position finds the first AtLeast whose minimum is met.
            // 3. If neither finds anything, the call is unexpected.
            let idx = guard
                .expectations
                .iter()
                .rposition(|e| matches!(&e.times, Times::Exact(_)) && e.times_remaining > 0)
                .or_else(|| {
                    guard.expectations.iter().position(|e| match &e.times {
                        Times::Exact(_) => false,
                        Times::AtLeast(n) => call_count >= *n,
                    })
                })
                .expect("send_message called but no expectations match this call");
            let exp = &mut guard.expectations[idx];
            match &exp.times {
                Times::Exact(_) => {
                    if exp.times_remaining == 0 {
                        panic!(
                            "send_message called more times than expected for expectation at index {}",
                            idx
                        );
                    }
                    // Check matcher BEFORE mutating state so a failing matcher
                    // does not consume the call slot (Thread 38).
                    if !(exp.matcher)(&routing, &data) {
                        panic!("send_message called with arguments that do not match the expectation");
                    }
                    exp.times_remaining -= 1;
                }
                Times::AtLeast(n) => {
                    if call_count < *n {
                        panic!("send_message called {} times but at least {} expected", call_count, n);
                    }
                    if !(exp.matcher)(&routing, &data) {
                        panic!("send_message called with arguments that do not match the expectation");
                    }
                    exp.matched_calls += 1;
                }
            };
            (exp.factory)(routing, data)
        };

        (response).await
    }
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Builder for a [`send_message`](SendMsg::send_message) expectation.
pub struct Expect<'a> {
    sender: &'a mut MsgSender,
    matcher: Option<std::sync::Arc<dyn Fn(&DestinationRouting, &ApplicationDataOut) -> bool + Send + Sync>>,
    times: Times,
}

impl<'a> Expect<'a> {
    /// Restrict which arguments this expectation matches.
    pub fn withf<M>(mut self, matcher: M) -> Self
    where
        M: Fn(&DestinationRouting, &ApplicationDataOut) -> bool + Send + Sync + 'static,
    {
        self.matcher = Some(std::sync::Arc::new(matcher));
        self
    }

    /// Expect this to be called exactly once.
    pub fn once(self) -> Self {
        self.times(1)
    }

    /// Expect this to be called exactly `n` times, or at least `n` times when passed
    /// a `RangeFrom` (e.g. `.times(5..)`).
    pub fn times<T: Into<Times>>(mut self, n: T) -> Self {
        self.times = n.into();
        self
    }

    /// Add this expectation to a [`mockall::Sequence`].
    ///
    /// No-op for this manual mock — the expectation ordering is enforced by
    /// the order in which expectations are registered.
    pub fn in_sequence(self, _seq: &mut mockall::Sequence) -> Self {
        self
    }

    /// Set the response returned when this expectation matches.
    /// The `Fn` factory is invoked at each matching call to `send_message`, producing
    /// a fresh future.  Test closures should wrap non-`Copy` captures (e.g. `SessionManager`)
    /// in `Arc` so the `move` async block inside the factory can clone them freely.
    pub fn returning<F>(self, f: F) -> &'a mut MsgSender
    where
        F: Fn(DestinationRouting, ApplicationDataOut) -> BoxFuture<'static, Result<(), errors::TransportSessionError>>
            + Send
            + Sync
            + 'static,
    {
        let times_remaining = self.times.remaining();
        self.sender.add_expectation(Expectation::new(
            self.matcher.unwrap_or_else(|| std::sync::Arc::new(|_, _| true)),
            std::sync::Arc::new(f),
            self.times,
            times_remaining,
        ));
        self.sender
    }
}

/// Drains messages from `sender` into the provided [`MsgSender`] mock.
///
/// Returns the tx half (to pass to [`crate::manager::SessionManager::start`]) and a [`tokio::task::JoinHandle`]
/// that must be awaited after the test to ensure all messages are dispatched.
pub fn mock_packet_planning(
    sender: MsgSender,
) -> (
    futures::channel::mpsc::UnboundedSender<(DestinationRouting, ApplicationDataOut)>,
    tokio::task::JoinHandle<()>,
) {
    let (tx, mut rx) = futures::channel::mpsc::unbounded();
    let handle = tokio::task::spawn(async move {
        while let Some((routing, data)) = rx.next().await {
            sender
                .send_message(routing, data)
                .await
                .expect("send message must not fail in mock");
        }
    });
    (tx, handle)
}

pub fn msg_type(data: &ApplicationDataOut, expected: hopr_protocol_start::StartProtocolDiscriminants) -> bool {
    HoprStartProtocol::decode(data.data.application_tag, &data.data.plain_text)
        .map(|d| hopr_protocol_start::StartProtocolDiscriminants::from(d) == expected)
        .unwrap_or(false)
}

pub fn start_msg_match(data: &ApplicationDataOut, msg: impl Fn(HoprStartProtocol) -> bool) -> bool {
    HoprStartProtocol::decode(data.data.application_tag, &data.data.plain_text)
        .map(msg)
        .unwrap_or(false)
}
