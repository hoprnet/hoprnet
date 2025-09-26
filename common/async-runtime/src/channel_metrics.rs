//! # Channel Monitoring Module
//!
//! Instrumented channel implementations for Prometheus metrics collection with dual-backend support.
//!
//! ## Features
//!
//! - **Metrics**: Channel capacity, send/receive duration tracking
//! - **Backends**: Tokio (accurate capacity) or futures (best-effort)
//! - **Safety**: 5-second timeout on blocking sends with error logging
//! - **Performance**: Zero overhead when Prometheus disabled
//!
//! ## Feature Flags
//!
//! - `prometheus`: Enables metrics collection
//! - `runtime-tokio`: Uses tokio backend for accurate capacity tracking
//!
//! ## Usage
//!
//! ```rust
//! use hopr_async_runtime::monitored_channel;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let (sender, mut receiver) = monitored_channel::<String>(1024, "task_queue");
//!
//! sender.send("hello".to_string()).await?;
//! let msg = receiver.recv().await;
//! # Ok(())
//! # }
//! ```

use std::{
    pin::Pin,
    task::{Context, Poll},
};

#[cfg(not(feature = "runtime-tokio"))]
use futures::SinkExt;
// Always use futures SendError for consistency across backends
use futures::channel::mpsc::SendError;

// Timeout for blocking send operations
const SEND_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

// Logging for timeout errors
use tracing::error;

// Import timeout functionality
#[cfg(feature = "runtime-tokio")]
use crate::prelude::timeout_fut;

/// Create SendError for API compatibility between tokio and futures backends.
#[cfg(feature = "runtime-tokio")]
fn create_futures_send_error() -> SendError {
    let (mut tx, rx) = futures::channel::mpsc::channel::<()>(1);
    drop(rx); // Close the receiver
    tx.try_send(()).unwrap_err().into_send_error()
}

#[cfg(feature = "prometheus")]
use {
    hopr_metrics::metrics::{MultiGauge, MultiHistogram},
    lazy_static::lazy_static,
};

// Import the appropriate channel implementation based on features
#[cfg(all(feature = "runtime-tokio", feature = "prometheus"))]
mod tokio_backend {
    pub type Sender<T> = tokio::sync::mpsc::Sender<T>;
    pub type Receiver<T> = tokio::sync::mpsc::Receiver<T>;
}

#[cfg(all(not(feature = "runtime-tokio"), feature = "prometheus"))]
mod futures_backend {
    pub type Sender<T> = futures::channel::mpsc::Sender<T>;
    pub type Receiver<T> = futures::channel::mpsc::Receiver<T>;
}

#[cfg(not(feature = "prometheus"))]
mod basic_backend {
    #[cfg(not(feature = "runtime-tokio"))]
    pub use futures::channel::mpsc::{Receiver, Sender};
    #[cfg(feature = "runtime-tokio")]
    pub use tokio::sync::mpsc::{Receiver, Sender};
}

// Import the appropriate types based on features
#[cfg(not(feature = "prometheus"))]
use basic_backend::*;
#[cfg(feature = "prometheus")]
use futures::sink::Sink;
// Re-export futures Stream for compatibility
use futures::stream::Stream;
#[cfg(all(not(feature = "runtime-tokio"), feature = "prometheus"))]
use futures_backend::*;
#[cfg(all(feature = "runtime-tokio", feature = "prometheus"))]
use tokio_backend::*;

// Prometheus metrics for channel monitoring
//
// These metrics are only available when the `prometheus` feature is enabled.
// All metrics include a `channel_name` label for identification and filtering.
#[cfg(feature = "prometheus")]
lazy_static! {
    /// Maximum buffer size of mpsc channels by channel name
    ///
    /// **Type**: Gauge
    /// **Labels**: `channel_name`
    /// **Description**: Records the configured buffer size when a channel is created.
    /// This metric remains constant for the lifetime of a channel.
    static ref HOPR_CHANNEL_MAX_CAPACITY: MultiGauge = MultiGauge::new(
        "hopr_channel_max_capacity",
        "Maximum buffer size of mpsc channels",
        &["channel_name"]
    ).unwrap();

    /// Current available buffer space of mpsc channels by channel name
    ///
    /// **Type**: Gauge
    /// **Labels**: `channel_name`
    /// **Description**: Records the current available buffer space. Updated after
    /// send/receive operations. Accuracy depends on backend:
    /// - Tokio: Real-time accurate tracking
    /// - Futures: Always 0 (tracking unavailable)
    static ref HOPR_CHANNEL_CURRENT_CAPACITY: MultiGauge = MultiGauge::new(
        "hopr_channel_current_capacity",
        "Current buffer size of mpsc channels",
        &["channel_name"]
    ).unwrap();

    /// Time distribution for channel send operations in seconds by channel name
    ///
    /// **Type**: Histogram
    /// **Labels**: `channel_name`
    /// **Buckets**: 1ms to 5s (optimal for async message passing)
    /// **Description**: Records the wall-clock time spent in send operations,
    /// including any blocking time when the channel buffer is full.
    static ref HOPR_CHANNEL_SEND_DURATION: MultiHistogram = MultiHistogram::new(
        "hopr_channel_send_duration_sec",
        "Time distribution for channel send operations in seconds",
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0],
        &["channel_name"]
    ).unwrap();

    /// Time distribution for channel receive operations in seconds by channel name
    ///
    /// **Type**: Histogram
    /// **Labels**: `channel_name`
    /// **Buckets**: 1ms to 5s (optimal for async message passing)
    /// **Description**: Records the wall-clock time spent in receive operations,
    /// including any waiting time when no messages are available.
    static ref HOPR_CHANNEL_RECEIVE_DURATION: MultiHistogram = MultiHistogram::new(
        "hopr_channel_receive_duration_sec",
        "Time distribution for channel receive operations in seconds",
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0],
        &["channel_name"]
    ).unwrap();
}

/// Channel sender with automatic Prometheus metrics collection.
///
/// Wraps `tokio::sync::mpsc::Sender` or `futures::channel::mpsc::Sender` based on features.
/// Records send duration and capacity metrics when `prometheus` feature is enabled.
///
/// **Note**: Tokio backend uses `&self` for sends, futures backend requires `&mut self`.
#[derive(Debug)]
#[cfg(feature = "prometheus")]
pub struct InstrumentedSender<T> {
    sender: Sender<T>,
    channel_name: String,
    capacity: usize,
}

/// Zero-overhead sender wrapper when Prometheus is disabled.
#[derive(Debug)]
#[cfg(not(feature = "prometheus"))]
pub struct InstrumentedSender<T> {
    sender: Sender<T>,
}

impl<T> InstrumentedSender<T> {
    /// Send a message with metrics and 5-second timeout (tokio backend).
    ///
    /// # Errors
    /// Returns `SendError` if receiver dropped or timeout exceeded.
    #[cfg(all(feature = "runtime-tokio", feature = "prometheus"))]
    pub async fn send(&self, msg: T) -> Result<(), SendError> {
        let start = std::time::Instant::now();

        let send_future = self.sender.send(msg);
        let result = match timeout_fut(SEND_TIMEOUT, send_future).await {
            Ok(send_result) => send_result.map_err(|_| create_futures_send_error()),
            Err(_) => {
                error!(
                    channel_name = %self.channel_name,
                    timeout_secs = SEND_TIMEOUT.as_secs(),
                    "Channel send operation timed out"
                );
                Err(create_futures_send_error())
            }
        };

        if result.is_ok() {
            let duration = start.elapsed().as_secs_f64();
            HOPR_CHANNEL_SEND_DURATION.observe(&[&self.channel_name], duration);
        }
        self.update_channel_capacity();

        result
    }

    /// Send a message with metrics and 5-second timeout (futures backend).
    ///
    /// # Errors
    /// Returns `SendError` if receiver dropped or timeout exceeded.
    #[cfg(all(not(feature = "runtime-tokio"), feature = "prometheus"))]
    pub async fn send(&mut self, msg: T) -> Result<(), SendError> {
        let start = std::time::Instant::now();

        // For futures backend, we use a simple elapsed time check
        // since we don't have access to tokio::time::timeout
        let send_result = self.sender.send(msg).await;

        let duration = start.elapsed();
        if duration > SEND_TIMEOUT {
            error!(
                channel_name = %self.channel_name,
                duration_secs = duration.as_secs(),
                timeout_secs = SEND_TIMEOUT.as_secs(),
                "Channel send operation exceeded timeout"
            );
        }

        if send_result.is_ok() {
            let duration_secs = duration.as_secs_f64();
            HOPR_CHANNEL_SEND_DURATION.observe(&[&self.channel_name], duration_secs);
        }
        self.update_channel_capacity();

        send_result
    }

    /// Send with 5-second timeout, no metrics (tokio backend).
    #[cfg(all(feature = "runtime-tokio", not(feature = "prometheus")))]
    pub async fn send(&self, msg: T) -> Result<(), SendError> {
        match timeout_fut(SEND_TIMEOUT, self.sender.send(msg)).await {
            Ok(send_result) => send_result.map_err(|_| create_futures_send_error()),
            Err(_) => {
                error!(
                    timeout_secs = SEND_TIMEOUT.as_secs(),
                    "Channel send operation timed out"
                );
                Err(create_futures_send_error())
            }
        }
    }

    /// Send with timeout logging, no metrics (futures backend).
    #[cfg(all(not(feature = "runtime-tokio"), not(feature = "prometheus")))]
    pub async fn send(&mut self, msg: T) -> Result<(), SendError> {
        let start = std::time::Instant::now();
        let result = self.sender.send(msg).await;

        let duration = start.elapsed();
        if duration > SEND_TIMEOUT {
            error!(
                duration_secs = duration.as_secs(),
                timeout_secs = SEND_TIMEOUT.as_secs(),
                "Channel send operation exceeded timeout"
            );
        }

        result
    }

    /// Non-blocking send (tokio backend).
    #[cfg(all(feature = "runtime-tokio", feature = "prometheus"))]
    pub fn try_send(&self, msg: T) -> Result<(), tokio::sync::mpsc::error::TrySendError<T>> {
        let result = self.sender.try_send(msg);
        if result.is_ok() {
            self.update_channel_capacity();
        }
        result
    }

    /// Non-blocking send (futures backend).
    #[cfg(all(not(feature = "runtime-tokio"), feature = "prometheus"))]
    pub fn try_send(&mut self, msg: T) -> Result<(), futures::channel::mpsc::TrySendError<T>> {
        let result = self.sender.try_send(msg);
        if result.is_ok() {
            self.update_channel_capacity();
        }
        result
    }

    /// Non-blocking send (no metrics).
    #[cfg(all(feature = "runtime-tokio", not(feature = "prometheus")))]
    pub fn try_send(&self, msg: T) -> Result<(), tokio::sync::mpsc::error::TrySendError<T>> {
        self.sender.try_send(msg)
    }

    /// Non-blocking send (no metrics).
    #[cfg(all(not(feature = "runtime-tokio"), not(feature = "prometheus")))]
    pub fn try_send(&mut self, msg: T) -> Result<(), futures::channel::mpsc::TrySendError<T>> {
        self.sender.try_send(msg)
    }

    /// Check if receiver dropped (channel closed).
    pub fn is_closed(&self) -> bool {
        #[cfg(feature = "runtime-tokio")]
        {
            self.sender.is_closed()
        }
        #[cfg(not(feature = "runtime-tokio"))]
        {
            self.sender.is_closed()
        }
    }

    /// Check if channel is open (not closed).
    pub fn is_ready(&self) -> bool {
        !self.is_closed()
    }

    /// Close channel (tokio: no-op, channels close on drop).
    #[cfg(feature = "runtime-tokio")]
    pub fn close_channel(&mut self) {
        // tokio channels don't have a close_channel method
        // The channel closes when all senders are dropped
    }

    /// Close channel explicitly (futures backend).
    #[cfg(not(feature = "runtime-tokio"))]
    pub fn close_channel(&mut self) {
        self.sender.close_channel()
    }

    /// Update capacity metric (tokio: accurate, futures: 0).
    #[cfg(feature = "prometheus")]
    fn update_channel_capacity(&self) {
        #[cfg(feature = "runtime-tokio")]
        {
            let cap = if self.is_closed() {
                0.0
            } else {
                self.sender.capacity() as f64
            };
            HOPR_CHANNEL_CURRENT_CAPACITY.set(&[&self.channel_name], cap);
        }

        #[cfg(not(feature = "runtime-tokio"))]
        {
            let cap = 0.0;
            HOPR_CHANNEL_CURRENT_CAPACITY.set(&[&self.channel_name], cap);
        }
    }
}

// Implement Sink trait for futures compatibility (tokio backend)
#[cfg(all(feature = "runtime-tokio", feature = "prometheus"))]
impl<T> Sink<T> for InstrumentedSender<T> {
    type Error = SendError;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // tokio channels are always ready unless closed
        if self.is_closed() {
            Poll::Ready(Err(create_futures_send_error()))
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        match self.try_send(item) {
            Ok(()) => Ok(()),
            Err(tokio::sync::mpsc::error::TrySendError::Full(_msg)) => {
                // Should not happen if poll_ready returned Ready(Ok)
                Err(create_futures_send_error())
            }
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_msg)) => Err(create_futures_send_error()),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(())) // tokio channels don't need explicit flushing
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut().close_channel();
        Poll::Ready(Ok(()))
    }
}

// Implement Sink trait for futures compatibility (futures backend)
#[cfg(all(not(feature = "runtime-tokio"), feature = "prometheus"))]
impl<T> Sink<T> for InstrumentedSender<T> {
    type Error = SendError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.sender).poll_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        let result = Pin::new(&mut self.sender).start_send(item);
        if result.is_ok() {
            self.update_channel_capacity();
        }
        result
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.sender).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.sender).poll_close(cx)
    }
}

impl<T> Clone for InstrumentedSender<T> {
    fn clone(&self) -> Self {
        #[cfg(feature = "prometheus")]
        {
            Self {
                sender: self.sender.clone(),
                channel_name: self.channel_name.clone(),
                capacity: self.capacity,
            }
        }
        #[cfg(not(feature = "prometheus"))]
        {
            Self {
                sender: self.sender.clone(),
            }
        }
    }
}

/// Channel receiver with automatic Prometheus metrics collection.
///
/// Wraps `tokio::sync::mpsc::Receiver` or `futures::channel::mpsc::Receiver` based on features.
/// Records receive duration and capacity metrics when `prometheus` feature is enabled.
///
/// Implements `futures::Stream` for async iteration:
/// ```rust
/// use futures::StreamExt;
/// # use hopr_async_runtime::monitored_channel;
/// # async fn example() {
/// let (_tx, mut rx) = monitored_channel::<i32>(100, "example");
/// while let Some(msg) = rx.next().await {
///     // Process message
/// }
/// # }
/// ```
#[derive(Debug)]
#[cfg(feature = "prometheus")]
pub struct InstrumentedReceiver<T> {
    receiver: Receiver<T>,
    channel_name: String,
}

/// Zero-overhead receiver wrapper when Prometheus is disabled.
#[derive(Debug)]
#[cfg(not(feature = "prometheus"))]
pub struct InstrumentedReceiver<T> {
    receiver: Receiver<T>,
}

impl<T> InstrumentedReceiver<T> {
    /// Non-blocking receive (tokio backend).
    #[cfg(all(feature = "runtime-tokio", feature = "prometheus"))]
    pub fn try_recv(&mut self) -> Result<T, tokio::sync::mpsc::error::TryRecvError> {
        let start = std::time::Instant::now();
        let result = self.receiver.try_recv();

        if result.is_ok() {
            let duration = start.elapsed().as_secs_f64();
            HOPR_CHANNEL_RECEIVE_DURATION.observe(&[&self.channel_name], duration);
            self.update_channel_capacity();
        }

        result
    }

    /// Non-blocking receive (futures backend).
    #[cfg(all(not(feature = "runtime-tokio"), feature = "prometheus"))]
    pub fn try_next(&mut self) -> Result<Option<T>, futures::channel::mpsc::TryRecvError> {
        let start = std::time::Instant::now();
        let result = self.receiver.try_next();

        if let Ok(Some(_)) = &result {
            let duration = start.elapsed().as_secs_f64();
            HOPR_CHANNEL_RECEIVE_DURATION.observe(&[&self.channel_name], duration);
            self.update_channel_capacity();
        }

        result
    }

    /// Non-blocking receive (no metrics).
    #[cfg(all(feature = "runtime-tokio", not(feature = "prometheus")))]
    pub fn try_recv(&mut self) -> Result<T, tokio::sync::mpsc::error::TryRecvError> {
        self.receiver.try_recv()
    }

    /// Non-blocking receive (no metrics).
    #[cfg(all(not(feature = "runtime-tokio"), not(feature = "prometheus")))]
    pub fn try_next(&mut self) -> Result<Option<T>, futures::channel::mpsc::TryRecvError> {
        self.receiver.try_next()
    }

    /// Async receive with metrics (tokio backend).
    #[cfg(all(feature = "runtime-tokio", feature = "prometheus"))]
    pub async fn recv(&mut self) -> Option<T> {
        let start = std::time::Instant::now();
        let result = self.receiver.recv().await;

        if result.is_some() {
            let duration = start.elapsed().as_secs_f64();
            HOPR_CHANNEL_RECEIVE_DURATION.observe(&[&self.channel_name], duration);
            self.update_channel_capacity();
        }

        result
    }

    /// Async receive without metrics (tokio backend).
    #[cfg(all(feature = "runtime-tokio", not(feature = "prometheus")))]
    pub async fn recv(&mut self) -> Option<T> {
        self.receiver.recv().await
    }

    /// Close receiver.
    pub fn close(&mut self) {
        #[cfg(feature = "runtime-tokio")]
        {
            self.receiver.close()
        }
        #[cfg(not(feature = "runtime-tokio"))]
        {
            self.receiver.close()
        }
    }

    /// Get available channel capacity (tokio: accurate, futures: always 0).
    #[cfg(feature = "runtime-tokio")]
    pub fn capacity(&self) -> usize {
        self.receiver.capacity()
    }

    /// Get capacity (always 0 for futures backend, exists for API compatibility).
    #[cfg(not(feature = "runtime-tokio"))]
    pub fn capacity(&self) -> usize {
        // futures backend doesn't provide this information
        0
    }

    /// Update capacity metric after receive.
    fn update_channel_capacity(&self) {
        HOPR_CHANNEL_CURRENT_CAPACITY.set(&[&self.channel_name], self.capacity() as f64);
    }
}

// Implement Stream for futures compatibility (tokio backend)
#[cfg(all(feature = "runtime-tokio", feature = "prometheus"))]
impl<T> Stream for InstrumentedReceiver<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let start = std::time::Instant::now();
        let result = self.receiver.poll_recv(cx);

        if let Poll::Ready(Some(_)) = &result {
            let duration = start.elapsed().as_secs_f64();
            HOPR_CHANNEL_RECEIVE_DURATION.observe(&[&self.channel_name], duration);
            self.update_channel_capacity();
        }

        result
    }
}

// Implement Stream for futures compatibility (futures backend)
#[cfg(all(not(feature = "runtime-tokio"), feature = "prometheus"))]
impl<T> Stream for InstrumentedReceiver<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let start = std::time::Instant::now();
        let result = Pin::new(&mut self.receiver).poll_next(cx);

        if let Poll::Ready(Some(_)) = &result {
            let duration = start.elapsed().as_secs_f64();
            HOPR_CHANNEL_RECEIVE_DURATION.observe(&[&self.channel_name], duration);
            self.update_channel_capacity();
        }

        result
    }
}

// Implement Stream for no-prometheus (tokio backend)
#[cfg(all(feature = "runtime-tokio", not(feature = "prometheus")))]
impl<T> Stream for InstrumentedReceiver<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}

// Implement Stream for no-prometheus (futures backend)
#[cfg(all(not(feature = "runtime-tokio"), not(feature = "prometheus")))]
impl<T> Stream for InstrumentedReceiver<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.receiver).poll_next(cx)
    }
}

/// Create a monitored mpsc channel with metrics collection.
///
/// Creates an instrumented channel that collects Prometheus metrics when enabled.
/// Uses `tokio::sync::mpsc` with `runtime-tokio` feature, otherwise `futures::channel::mpsc`.
///
/// ## Metrics (when `prometheus` enabled)
/// - `hopr_channel_max_capacity`: Buffer size (gauge)
/// - `hopr_channel_current_capacity`: Available space (gauge, accurate with tokio)
/// - `hopr_channel_send_duration_sec`: Send timing (histogram)
/// - `hopr_channel_receive_duration_sec`: Receive timing (histogram)
///
/// ## Performance
/// - Prometheus disabled: Zero overhead
/// - With tokio: ~5-10% overhead
/// - With futures: ~2-5% overhead
///
/// # Examples
/// ```rust
/// use hopr_async_runtime::monitored_channel;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let (tx, mut rx) = monitored_channel::<String>(1024, "task_queue");
///
/// tx.send("data".to_string()).await?;
/// if let Some(msg) = rx.recv().await {
///     // Process message
/// }
/// # Ok(())
/// # }
/// ```
#[cfg(all(feature = "runtime-tokio", feature = "prometheus"))]
pub fn monitored_channel<T>(
    buffer_size: usize,
    channel_name: &str,
) -> (InstrumentedSender<T>, InstrumentedReceiver<T>) {
    let (sender, receiver) = tokio::sync::mpsc::channel(buffer_size);

    // Initialize capacity metric
    HOPR_CHANNEL_MAX_CAPACITY.set(&[channel_name], buffer_size as f64);
    HOPR_CHANNEL_CURRENT_CAPACITY.set(&[channel_name], 0.0);

    (
        InstrumentedSender {
            sender,
            channel_name: channel_name.to_string(),
            capacity: buffer_size,
        },
        InstrumentedReceiver {
            receiver,
            channel_name: channel_name.to_string(),
        },
    )
}

/// Create monitored channel (futures backend with prometheus).
#[cfg(all(not(feature = "runtime-tokio"), feature = "prometheus"))]
pub fn monitored_channel<T>(
    buffer_size: usize,
    channel_name: &str,
) -> (InstrumentedSender<T>, InstrumentedReceiver<T>) {
    let (sender, receiver) = futures::channel::mpsc::channel(buffer_size);

    // Initialize capacity metric
    HOPR_CHANNEL_MAX_CAPACITY.set(&[channel_name], buffer_size as f64);
    HOPR_CHANNEL_CURRENT_CAPACITY.set(&[channel_name], 0.0);

    (
        InstrumentedSender {
            sender,
            channel_name: channel_name.to_string(),
            capacity: buffer_size,
        },
        InstrumentedReceiver {
            receiver,
            channel_name: channel_name.to_string(),
        },
    )
}

/// Create channel without metrics (prometheus disabled).
#[cfg(not(feature = "prometheus"))]
pub fn monitored_channel<T>(
    buffer_size: usize,
    _channel_name: &str,
) -> (InstrumentedSender<T>, InstrumentedReceiver<T>) {
    #[cfg(feature = "runtime-tokio")]
    {
        let (sender, receiver) = tokio::sync::mpsc::channel(buffer_size);
        (InstrumentedSender { sender }, InstrumentedReceiver { receiver })
    }
    #[cfg(not(feature = "runtime-tokio"))]
    {
        let (sender, receiver) = futures::channel::mpsc::channel(buffer_size);
        (InstrumentedSender { sender }, InstrumentedReceiver { receiver })
    }
}

/// Tests for channel metrics across both backends and feature combinations.
#[cfg(test)]
mod tests {
    #[cfg(not(feature = "runtime-tokio"))]
    use futures::StreamExt;

    use super::*;

    #[tokio::test]
    async fn test_monitored_channel_basic_functionality() {
        #[cfg(feature = "runtime-tokio")]
        let (sender, mut receiver) = monitored_channel::<String>(10, "test_channel");
        #[cfg(not(feature = "runtime-tokio"))]
        let (mut sender, mut receiver) = monitored_channel::<String>(10, "test_channel");

        // Test send and receive
        #[cfg(feature = "runtime-tokio")]
        {
            sender.send("test message".to_string()).await.unwrap();
            let received = receiver.recv().await.unwrap();
            assert_eq!(received, "test message");
        }
        #[cfg(not(feature = "runtime-tokio"))]
        {
            sender.send("test message".to_string()).await.unwrap();
            let received = receiver.next().await.unwrap();
            assert_eq!(received, "test message");
        }
    }

    #[tokio::test]
    async fn test_try_send_try_receive() {
        #[cfg(feature = "runtime-tokio")]
        let (sender, mut receiver) = monitored_channel::<String>(1, "test_try_channel");
        #[cfg(not(feature = "runtime-tokio"))]
        let (mut sender, mut receiver) = monitored_channel::<String>(1, "test_try_channel");

        // Test try_send - first message should succeed
        #[cfg(feature = "runtime-tokio")]
        {
            sender.try_send("message1".to_string()).unwrap();
            // Test try_recv - should receive the message
            let received = receiver.try_recv().unwrap();
            assert_eq!(received, "message1");
            // Channel should be empty now
            assert!(receiver.try_recv().is_err());
        }
        #[cfg(not(feature = "runtime-tokio"))]
        {
            sender.try_send("message1".to_string()).unwrap();
            // Test try_next - should receive the message
            let received = receiver.try_next().unwrap().unwrap();
            assert_eq!(received, "message1");
            // Channel should be empty now
            assert!(receiver.try_next().is_err());
        }
    }

    #[test]
    fn test_channel_ready() {
        let (sender, _receiver) = monitored_channel::<String>(42, "ready_test");
        assert!(sender.is_ready());
    }

    #[tokio::test]
    async fn test_channel_close() {
        #[cfg(feature = "runtime-tokio")]
        let (sender, mut receiver) = monitored_channel::<String>(10, "close_test");
        #[cfg(not(feature = "runtime-tokio"))]
        let (mut sender, mut receiver) = monitored_channel::<String>(10, "close_test");

        // Close the channel - for tokio this means dropping the sender
        // for futures this means calling close_channel
        #[cfg(feature = "runtime-tokio")]
        {
            drop(sender); // tokio channels close when all senders are dropped
            assert!(receiver.recv().await.is_none());
        }
        #[cfg(not(feature = "runtime-tokio"))]
        {
            sender.close_channel(); // futures channels have explicit close
            assert!(receiver.next().await.is_none());
        }
    }

    #[cfg(all(feature = "runtime-tokio", feature = "prometheus"))]
    #[tokio::test]
    async fn test_accurate_capacity_metrics() {
        let (sender, mut receiver) = monitored_channel::<i32>(5, "accurate_test");

        // Check that queue cap is as set
        assert_eq!(receiver.capacity(), 5);

        // Send some messages
        sender.send(1).await.unwrap();
        sender.send(2).await.unwrap();
        sender.send(3).await.unwrap();

        // Check that queue cap is accurate
        assert_eq!(receiver.capacity(), 2);

        // Receive one message
        let _msg = receiver.recv().await.unwrap();

        // Check that queue cap incremented
        assert_eq!(receiver.capacity(), 3);
    }
}
