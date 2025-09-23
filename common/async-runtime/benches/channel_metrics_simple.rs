use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hopr_async_runtime::monitored_channel;
use std::time::Duration;
use std::hint::black_box;

// Import necessary traits for both backends
use futures::StreamExt;

// For futures backend when using Sink trait
#[cfg(not(feature = "runtime-tokio"))]
use futures::SinkExt;

// For tokio backend when using Sink trait (no prometheus case)
#[cfg(all(feature = "runtime-tokio", not(feature = "prometheus")))]
use futures::SinkExt;

// Benchmark constants
const CHANNEL_SIZES: &[usize] = &[10, 100, 1000];
const MESSAGE_COUNTS: &[usize] = &[100, 1000];

/// Simple channel send/receive benchmark
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

/// Native channel benchmark for comparison
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

/// Native futures channel benchmark for comparison
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

/// Channel creation benchmark
fn bench_channel_creation(buffer_size: usize) {
    let (_sender, _receiver) = monitored_channel::<usize>(buffer_size, "creation_bench");
}

/// Native channel creation benchmark
#[cfg(feature = "runtime-tokio")]
fn bench_native_creation_tokio(buffer_size: usize) {
    let (_sender, _receiver) = tokio::sync::mpsc::channel::<usize>(buffer_size);
}

#[cfg(not(feature = "runtime-tokio"))]
fn bench_native_creation_futures(buffer_size: usize) {
    let (_sender, _receiver) = futures::channel::mpsc::channel::<usize>(buffer_size);
}

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
                }
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
                }
            );
        }
    }
    
    group.finish();
}

fn bench_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_channel_creation");
    
    for &buffer_size in CHANNEL_SIZES {
        // Benchmark monitored channel creation
        group.bench_with_input(
            BenchmarkId::new("monitored", buffer_size),
            &buffer_size,
            |b, &buf| {
                b.iter(|| bench_channel_creation(buf));
            }
        );
        
        // Benchmark native channel creation
        group.bench_with_input(
            BenchmarkId::new("native", buffer_size),
            &buffer_size,
            |b, &buf| {
                b.iter(|| {
                    #[cfg(feature = "runtime-tokio")]
                    bench_native_creation_tokio(buf);
                    #[cfg(not(feature = "runtime-tokio"))]
                    bench_native_creation_futures(buf);
                });
            }
        );
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