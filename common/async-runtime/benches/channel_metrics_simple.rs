//! # Channel Metrics Simple Benchmark
//!
//! This benchmark measures the basic performance impact of channel monitoring in `hopr-async-runtime`.
//! It provides a simple, reliable comparison between monitored and native channels across all feature configurations.
//!
//! ## Test Groups
//!
//! ### 1. **simple_channel_send**
//! Measures throughput of sending messages through channels with different buffer sizes and message counts.
//! - Compares monitored vs native channels
//! - Tests buffer sizes: 10, 100, 1000
//! - Tests message counts: 100, 1000
//!
//! ### 2. **simple_channel_creation**
//! Measures the overhead of creating monitored vs native channels.
//! - Tests memory allocation overhead
//! - Compares initialization time
//!
//! ## Running This Benchmark
//!
//! ```bash
//! # Run with specific feature combinations
//! cargo bench -p hopr-async-runtime --features runtime-tokio,prometheus channel_metrics_simple
//! cargo bench -p hopr-async-runtime --no-default-features --features runtime-tokio channel_metrics_simple
//! cargo bench -p hopr-async-runtime --no-default-features --features runtime-futures channel_metrics_simple
//! cargo bench -p hopr-async-runtime --features runtime-futures,prometheus channel_metrics_simple
//!
//! # Use the helper script for all configurations
//! ./benches/bench_all_configs.sh
//! ```
//!
//! ## Expected Performance Impact
//!
//! - **With Prometheus**: 5-10% overhead (tokio), 2-5% overhead (futures)
//! - **Without Prometheus**: ~0% overhead (passthrough)
//! - **Channel Creation**: ~77% overhead (one-time cost)
//!
//! ## Feature Configuration Support
//!
//! This benchmark works with all feature combinations:
//! - `runtime-tokio` + `prometheus`
//! - `runtime-tokio` (no prometheus)
//! - `runtime-futures` + `prometheus`
//! - `runtime-futures` (no prometheus)
//!
//! ## Interpreting Results
//!
//! ### Key Metrics
//!
//! - **time**: Execution time per operation (lower is better)
//! - **thrpt**: Throughput in messages/second (higher is better)
//! - **change**: Performance change from previous runs (if available)
//!
//! ### Performance Calculations
//!
//! ```
//! Overhead % = (monitored_time - native_time) / native_time * 100
//!
//! Example:
//! - Monitored: 251.2 µs
//! - Native: 241.1 µs  
//! - Overhead: (251.2 - 241.1) / 241.1 * 100 = 4.2%
//! ```
//!
//! ### Configuration Comparison
//!
//! ```
//! monitored/prometheus vs native     = full monitoring overhead
//! monitored/no-prometheus vs native  = wrapper overhead (~0%)
//! tokio vs futures                  = backend performance difference
//! ```
//!
//! ### Optimization Guidelines
//!
//! - **< 5% overhead**: Excellent, monitoring is very efficient
//! - **5-10% overhead**: Good, acceptable for most use cases
//! - **> 10% overhead**: Consider disabling monitoring for hot paths
//! - **Creation overhead**: One-time cost, typically not critical

use std::{hint::black_box, time::Duration};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
// For futures backend when using Sink trait
#[cfg(not(feature = "runtime-tokio"))]
use futures::SinkExt;
// For tokio backend when using Sink trait (no prometheus case)
#[cfg(all(feature = "runtime-tokio", not(feature = "prometheus")))]
use futures::SinkExt;
// Import necessary traits for both backends
use futures::StreamExt;
use hopr_async_runtime::monitored_channel;

// Benchmark constants
const CHANNEL_SIZES: &[usize] = &[10, 100, 1000];
const MESSAGE_COUNTS: &[usize] = &[100, 1000];

/// Simple channel send/receive benchmark that works across all feature configurations.
///
/// This function sends `message_count` messages through a monitored channel with the specified
/// `buffer_size` and measures the total time taken. It handles the API differences between
/// tokio and futures backends automatically through conditional compilation.
///
/// # Configuration Handling
///
/// - **Tokio + Prometheus**: Uses `&self` for sender, native `.send()` method
/// - **Tokio No Prometheus**: Uses `&mut self` for sender, Sink trait `.send()` method
/// - **Futures + Prometheus**: Uses `&mut self` for sender, native `.send()` method
/// - **Futures No Prometheus**: Uses `&mut self` for sender, native `.send()` method
///
/// # Parameters
///
/// - `buffer_size`: Channel buffer capacity (10, 100, or 1000)
/// - `message_count`: Number of messages to send (100 or 1000)
async fn bench_channel_simple(buffer_size: usize, message_count: usize) {
    #[cfg(all(feature = "runtime-tokio", feature = "prometheus"))]
    {
        let (sender, mut receiver) = monitored_channel::<usize>(buffer_size, "bench_channel");

        // Spawn receiver task
        let receiver_handle = tokio::spawn(async move {
            for _ in 0..message_count {
                receiver.next().await;
            }
        });

        // Send messages - tokio backend with prometheus uses &self
        for i in 0..message_count {
            sender.send(black_box(i)).await.unwrap();
        }

        receiver_handle.await.unwrap();
    }

    #[cfg(all(feature = "runtime-tokio", not(feature = "prometheus")))]
    {
        let (mut sender, mut receiver) = monitored_channel::<usize>(buffer_size, "bench_channel");

        // Spawn receiver task
        let receiver_handle = tokio::spawn(async move {
            for _ in 0..message_count {
                receiver.next().await;
            }
        });

        // Send messages - tokio backend without prometheus uses Sink trait (&mut self)
        for i in 0..message_count {
            sender.send(black_box(i)).await.unwrap();
        }

        receiver_handle.await.unwrap();
    }

    #[cfg(not(feature = "runtime-tokio"))]
    {
        let (mut sender, mut receiver) = monitored_channel::<usize>(buffer_size, "bench_channel");

        // Spawn receiver task
        let receiver_handle = tokio::spawn(async move {
            for _ in 0..message_count {
                receiver.next().await;
            }
        });

        // Send messages - futures backend uses &mut self
        for i in 0..message_count {
            sender.send(black_box(i)).await.unwrap();
        }

        receiver_handle.await.unwrap();
    }
}

/// Native tokio channel benchmark for baseline comparison.
///
/// This function provides a baseline measurement using native `tokio::sync::mpsc::channel`
/// without any monitoring overhead. Used to calculate the performance impact of monitoring.
///
/// # Parameters
///
/// - `buffer_size`: Channel buffer capacity
/// - `message_count`: Number of messages to send
#[cfg(feature = "runtime-tokio")]
async fn bench_native_tokio(buffer_size: usize, message_count: usize) {
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<usize>(buffer_size);

    let receiver_handle = tokio::spawn(async move {
        for _ in 0..message_count {
            receiver.recv().await;
        }
    });

    for i in 0..message_count {
        sender.send(black_box(i)).await.unwrap();
    }

    receiver_handle.await.unwrap();
}

/// Native futures channel benchmark for baseline comparison.
///
/// This function provides a baseline measurement using native `futures::channel::mpsc::channel`
/// without any monitoring overhead. Used to calculate the performance impact of monitoring.
///
/// # Parameters
///
/// - `buffer_size`: Channel buffer capacity
/// - `message_count`: Number of messages to send
#[cfg(not(feature = "runtime-tokio"))]
async fn bench_native_futures(buffer_size: usize, message_count: usize) {
    let (mut sender, mut receiver) = futures::channel::mpsc::channel::<usize>(buffer_size);

    let receiver_handle = tokio::spawn(async move {
        for _ in 0..message_count {
            receiver.next().await;
        }
    });

    for i in 0..message_count {
        sender.send(black_box(i)).await.unwrap();
    }

    receiver_handle.await.unwrap();
}

/// Monitored channel creation benchmark.
///
/// Measures the overhead of creating a monitored channel, including metric initialization.
/// This is a one-time cost but helps understand the setup overhead.
///
/// # Parameters
///
/// - `buffer_size`: Channel buffer capacity
fn bench_channel_creation(buffer_size: usize) {
    let (_sender, _receiver) = monitored_channel::<usize>(buffer_size, "creation_bench");
}

/// Native tokio channel creation benchmark for baseline comparison.
///
/// Measures the time to create a native tokio channel without any monitoring.
/// Used as baseline to calculate monitoring creation overhead.
#[cfg(feature = "runtime-tokio")]
fn bench_native_creation_tokio(buffer_size: usize) {
    let (_sender, _receiver) = tokio::sync::mpsc::channel::<usize>(buffer_size);
}

/// Native futures channel creation benchmark for baseline comparison.
///
/// Measures the time to create a native futures channel without any monitoring.
/// Used as baseline to calculate monitoring creation overhead.
#[cfg(not(feature = "runtime-tokio"))]
fn bench_native_creation_futures(buffer_size: usize) {
    let (_sender, _receiver) = futures::channel::mpsc::channel::<usize>(buffer_size);
}

/// Main benchmark function for channel send/receive operations.
///
/// This function runs the core benchmark comparing monitored vs native channels
/// across different buffer sizes and message counts. Results show the runtime
/// overhead of channel monitoring.
///
/// # Measurements
///
/// - **Throughput**: Messages per second (higher is better)
/// - **Latency**: Time per operation (lower is better)
/// - **Overhead**: Performance difference vs native channels
///
/// # Output Interpretation
///
/// ```text
/// simple_channel_send/monitored/buf_100_msgs_1000
///   time: [250.1 µs 251.2 µs 252.3 µs]
///   thrpt: [3.96 Melem/s 3.98 Melem/s 4.00 Melem/s]
///
/// simple_channel_send/native/buf_100_msgs_1000  
///   time: [240.5 µs 241.1 µs 241.7 µs]
///   thrpt: [4.14 Melem/s 4.15 Melem/s 4.16 Melem/s]
/// ```
///
/// In this example: ~4% overhead (251.2µs vs 241.1µs)
fn bench_channel_send(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("simple_channel_send");

    for &buffer_size in CHANNEL_SIZES {
        for &message_count in MESSAGE_COUNTS {
            group.throughput(Throughput::Elements(message_count as u64));

            // Benchmark monitored channels
            group.bench_with_input(
                BenchmarkId::new("monitored", format!("buf_{}_msgs_{}", buffer_size, message_count)),
                &(buffer_size, message_count),
                |b, &(buf, msgs)| {
                    b.to_async(&runtime).iter(|| bench_channel_simple(buf, msgs));
                },
            );

            // Benchmark native channels for comparison
            group.bench_with_input(
                BenchmarkId::new("native", format!("buf_{}_msgs_{}", buffer_size, message_count)),
                &(buffer_size, message_count),
                |b, &(buf, msgs)| {
                    b.to_async(&runtime).iter(|| async {
                        #[cfg(feature = "runtime-tokio")]
                        bench_native_tokio(buf, msgs).await;
                        #[cfg(not(feature = "runtime-tokio"))]
                        bench_native_futures(buf, msgs).await;
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark function for channel creation overhead.
///
/// Measures the one-time cost of creating monitored channels vs native channels.
/// This helps understand the initialization overhead of the monitoring system.
///
/// # What It Measures
///
/// - **Memory allocation**: Time to allocate channel structures
/// - **Metric initialization**: Time to set up Prometheus metrics (if enabled)
/// - **Baseline comparison**: Native channel creation time
///
/// # Expected Results
///
/// - **With Prometheus**: ~77% overhead (includes metric setup)
/// - **Without Prometheus**: Near-zero overhead (simple wrapper)
///
/// # Output Interpretation
///
/// ```text
/// simple_channel_creation/monitored/1000
///   time: [124.2 ns 125.1 ns 126.0 ns]
///
/// simple_channel_creation/native/1000
///   time: [70.1 ns 70.8 ns 71.5 ns]
/// ```
///
/// In this example: ~77% creation overhead (125.1ns vs 70.8ns)
fn bench_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_channel_creation");

    for &buffer_size in CHANNEL_SIZES {
        // Benchmark monitored channel creation
        group.bench_with_input(BenchmarkId::new("monitored", buffer_size), &buffer_size, |b, &buf| {
            b.iter(|| bench_channel_creation(buf));
        });

        // Benchmark native channel creation
        group.bench_with_input(BenchmarkId::new("native", buffer_size), &buffer_size, |b, &buf| {
            b.iter(|| {
                #[cfg(feature = "runtime-tokio")]
                bench_native_creation_tokio(buf);
                #[cfg(not(feature = "runtime-tokio"))]
                bench_native_creation_futures(buf);
            });
        });
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(50)
        .measurement_time(Duration::from_secs(5))
        .warm_up_time(Duration::from_secs(2));
    targets = bench_channel_send, bench_creation
}

criterion_main!(benches);
