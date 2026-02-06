//! Parallelization utilities for CPU-heavy blocking workloads.
//!
//! This crate provides async-friendly wrappers around Rayon's thread pool for offloading
//! CPU-intensive operations (EC multiplication, ECDSA signing, MAC verification) from
//! async executor threads.
//!
//! For background on async executors and blocking, see
//! [Async: What is blocking?](https://ryhl.io/blog/async-what-is-blocking/).
//!
//! See the [`cpu`] module for the primary API.

/// Module for thread pool-based parallelization of CPU-heavy blocking workloads.
///
/// ## Zombie Task Prevention
///
/// The Rayon thread pool is sized to CPU cores for crypto operations. Callers wrap
/// tasks with timeouts (e.g., 150ms for packet decoding). When a timeout fires, the
/// async receiver is dropped, but Rayon has no native cancellation—the closure
/// continues as a "zombie" task whose result is discarded.
///
/// Under sustained load, zombie accumulation can starve the pool: timed-out tasks
/// continue occupying threads, causing subsequent tasks to also time out. To break
/// this cycle, each spawned closure checks `tx.is_canceled()` before executing.
/// If the receiver was dropped while queued, the closure returns immediately.
///
/// ## Queue Depth Limiting
///
/// To prevent unbounded queue growth, the module tracks outstanding tasks (queued +
/// running). Use [`spawn_blocking`] or [`spawn_fifo_blocking`] which return
/// [`SpawnError::QueueFull`] when the configured limit is reached.
///
/// Set `HOPR_CPU_TASK_QUEUE_LIMIT` environment variable to enable limiting.
///
/// ## Observability
///
/// Prometheus metrics (behind the `prometheus` feature) track:
/// - **submitted**: total tasks entering the queue
/// - **completed**: tasks that delivered results to a live receiver
/// - **cancelled**: tasks skipped via cooperative cancellation
/// - **orphaned**: tasks that ran but whose receiver was dropped during execution
/// - **rejected**: tasks rejected due to queue being full
/// - **queue_wait**: histogram of queue wait time
/// - **execution_time**: histogram of task execution duration
/// - **outstanding_tasks**: current queued + running tasks
/// - **queue_limit**: configured maximum (for comparison)
#[cfg(feature = "rayon")]
pub mod cpu {
    pub use rayon;

    use std::sync::atomic::{AtomicUsize, Ordering};

    use futures::channel::oneshot;

    /// Histogram buckets for timing metrics (seconds).
    #[cfg(all(feature = "prometheus", not(test)))]
    const TIMING_BUCKETS: &[f64] = &[0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.15, 0.25, 0.5, 1.0];

    mod metrics {
        #[cfg(all(feature = "prometheus", not(test)))]
        pub use real::*;

        #[cfg(any(not(feature = "prometheus"), test))]
        pub use noop::*;

        #[cfg(all(feature = "prometheus", not(test)))]
        mod real {
            use lazy_static::lazy_static;

            lazy_static! {
                static ref TASKS_SUBMITTED: hopr_metrics::SimpleCounter = hopr_metrics::SimpleCounter::new(
                    "hopr_rayon_tasks_submitted_total",
                    "Total number of tasks submitted to the Rayon thread pool",
                )
                .unwrap();
                static ref TASKS_COMPLETED: hopr_metrics::SimpleCounter = hopr_metrics::SimpleCounter::new(
                    "hopr_rayon_tasks_completed_total",
                    "Total number of Rayon tasks that completed and delivered results",
                )
                .unwrap();
                static ref TASKS_CANCELLED: hopr_metrics::SimpleCounter = hopr_metrics::SimpleCounter::new(
                    "hopr_rayon_tasks_cancelled_total",
                    "Total number of Rayon tasks skipped because receiver was already dropped",
                )
                .unwrap();
                static ref TASKS_ORPHANED: hopr_metrics::SimpleCounter = hopr_metrics::SimpleCounter::new(
                    "hopr_rayon_tasks_orphaned_total",
                    "Total number of Rayon tasks whose results were discarded after completion",
                )
                .unwrap();
                static ref TASKS_REJECTED: hopr_metrics::SimpleCounter = hopr_metrics::SimpleCounter::new(
                    "hopr_rayon_tasks_rejected_total",
                    "Total number of tasks rejected due to queue being full",
                )
                .unwrap();
                static ref QUEUE_WAIT: hopr_metrics::SimpleHistogram = hopr_metrics::SimpleHistogram::new(
                    "hopr_rayon_queue_wait_seconds",
                    "Time tasks spend waiting in the Rayon queue before execution starts",
                    super::super::TIMING_BUCKETS.to_vec(),
                )
                .unwrap();
                static ref EXECUTION_TIME: hopr_metrics::MultiHistogram = hopr_metrics::MultiHistogram::new(
                    "hopr_rayon_execution_seconds",
                    "Time tasks spend executing in the Rayon thread pool",
                    super::super::TIMING_BUCKETS.to_vec(),
                    &["operation"],
                )
                .unwrap();
                static ref OUTSTANDING_TASKS: hopr_metrics::SimpleGauge = hopr_metrics::SimpleGauge::new(
                    "hopr_rayon_outstanding_tasks",
                    "Current number of tasks queued or running in the Rayon pool",
                )
                .unwrap();
                static ref QUEUE_LIMIT: hopr_metrics::SimpleGauge = hopr_metrics::SimpleGauge::new(
                    "hopr_rayon_queue_limit",
                    "Configured maximum outstanding tasks for the Rayon thread pool",
                )
                .unwrap();
            }

            #[inline]
            pub fn submitted() {
                TASKS_SUBMITTED.increment();
            }

            #[inline]
            pub fn completed() {
                TASKS_COMPLETED.increment();
            }

            #[inline]
            pub fn cancelled() {
                TASKS_CANCELLED.increment();
            }

            #[inline]
            pub fn orphaned() {
                TASKS_ORPHANED.increment();
            }

            #[inline]
            pub fn rejected() {
                TASKS_REJECTED.increment();
            }

            #[inline]
            pub fn observe_queue_wait(seconds: f64) {
                QUEUE_WAIT.observe(seconds);
            }

            #[inline]
            pub fn observe_execution(operation: &str, seconds: f64) {
                EXECUTION_TIME.observe(&[operation], seconds);
            }

            #[inline]
            pub fn outstanding_inc() {
                OUTSTANDING_TASKS.increment(1.0);
            }

            #[inline]
            pub fn outstanding_dec() {
                OUTSTANDING_TASKS.decrement(1.0);
            }

            #[inline]
            pub fn set_queue_limit(limit: usize) {
                QUEUE_LIMIT.set(limit as f64);
            }
        }

        #[cfg(any(not(feature = "prometheus"), test))]
        mod noop {
            #[inline]
            pub fn submitted() {}
            #[inline]
            pub fn completed() {}
            #[inline]
            pub fn cancelled() {}
            #[inline]
            pub fn orphaned() {}
            #[inline]
            pub fn rejected() {}
            #[inline]
            pub fn observe_queue_wait(_: f64) {}
            #[inline]
            pub fn observe_execution(_: &str, _: f64) {}
            #[inline]
            pub fn outstanding_inc() {}
            #[inline]
            pub fn outstanding_dec() {}
            #[inline]
            pub fn set_queue_limit(_: usize) {}
        }
    }

    /// Current number of outstanding tasks (queued + running).
    static OUTSTANDING: AtomicUsize = AtomicUsize::new(0);

    lazy_static::lazy_static! {
        /// Queue limit from environment. `None` means no limit.
        static ref QUEUE_LIMIT: Option<usize> = {
            let limit = std::env::var("HOPR_CPU_TASK_QUEUE_LIMIT")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .filter(|&v| v > 0);

            if let Some(l) = limit {
                metrics::set_queue_limit(l);
            }

            limit
        };
    }

    /// Error type for spawn operations.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SpawnError {
        /// The queue is full and cannot accept more tasks.
        QueueFull {
            /// Current outstanding task count when rejection occurred.
            current: usize,
            /// Configured queue limit.
            limit: usize,
        },
    }

    impl std::fmt::Display for SpawnError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                SpawnError::QueueFull { current, limit } => {
                    write!(f, "rayon queue full: {current}/{limit} tasks outstanding")
                }
            }
        }
    }

    impl std::error::Error for SpawnError {}

    /// Returns the current outstanding task count (queued + running).
    #[inline]
    pub fn outstanding_tasks() -> usize {
        OUTSTANDING.load(Ordering::Relaxed)
    }

    /// Returns the configured queue limit, or `None` if unlimited.
    #[inline]
    pub fn queue_limit() -> Option<usize> {
        *QUEUE_LIMIT
    }

    /// Attempts to acquire a slot for a new task.
    ///
    /// Returns `Ok(())` if no limit or slot acquired, `Err(QueueFull)` if at limit.
    fn try_acquire_slot() -> Result<(), SpawnError> {
        OUTSTANDING.fetch_add(1, Ordering::AcqRel);
        metrics::outstanding_inc();

        if let Some(limit) = *QUEUE_LIMIT {
            let current = OUTSTANDING.load(Ordering::Relaxed);
            if current > limit {
                release_slot();
                metrics::rejected();
                return Err(SpawnError::QueueFull { current: current - 1, limit });
            }
        }
        Ok(())
    }

    /// Releases a slot when a task completes, is cancelled, or orphaned.
    #[inline]
    fn release_slot() {
        let prev = OUTSTANDING.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(prev > 0, "outstanding task count underflow");
        metrics::outstanding_dec();
    }

    /// Initialize the Rayon thread pool with the given number of threads.
    ///
    /// Also initializes the queue limit metric.
    pub fn init_thread_pool(num_threads: usize) -> Result<(), rayon::ThreadPoolBuildError> {
        let result = rayon::ThreadPoolBuilder::new().num_threads(num_threads).build_global();
        let _ = *QUEUE_LIMIT; // Initialize limit metric
        result
    }

    /// Builds a cancellable task closure and its receiver.
    ///
    /// The closure wraps `f` with cooperative cancellation, panic catching,
    /// timing metrics, and slot tracking.
    fn cancellable_task<R: Send + 'static>(
        f: impl FnOnce() -> R + Send + 'static,
        operation: &'static str,
    ) -> (impl FnOnce() + Send + 'static, oneshot::Receiver<std::thread::Result<R>>) {
        let (tx, rx) = oneshot::channel();
        let submitted_at = std::time::Instant::now();

        metrics::submitted();

        let task = move || {
            if tx.is_canceled() {
                tracing::debug!(
                    queue_wait_ms = submitted_at.elapsed().as_millis() as u64,
                    "skipping cancelled task (receiver dropped before execution)"
                );
                metrics::cancelled();
                release_slot();
                return;
            }

            let wait_duration = submitted_at.elapsed();
            metrics::observe_queue_wait(wait_duration.as_secs_f64());

            let execution_start = std::time::Instant::now();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
            metrics::observe_execution(operation, execution_start.elapsed().as_secs_f64());

            match tx.send(result) {
                Ok(()) => metrics::completed(),
                Err(_) => {
                    tracing::debug!(
                        queue_wait_ms = wait_duration.as_millis() as u64,
                        "receiver dropped during execution, result discarded"
                    );
                    metrics::orphaned();
                }
            }
            release_slot();
        };

        (task, rx)
    }

    /// Spawn a blocking function on the Rayon thread pool (LIFO scheduling).
    ///
    /// Uses Rayon's default LIFO scheduling for the thread's local queue.
    ///
    /// Includes cooperative cancellation: if the receiver is dropped before the
    /// task starts (e.g., timeout), the task is skipped without executing.
    ///
    /// # Errors
    ///
    /// Returns [`SpawnError::QueueFull`] if the outstanding task count exceeds the limit.
    pub async fn spawn_blocking<R: Send + 'static>(
        f: impl FnOnce() -> R + Send + 'static,
        operation: &'static str,
    ) -> Result<R, SpawnError> {
        try_acquire_slot()?;
        let (task, rx) = cancellable_task(f, operation);
        rayon::spawn(task);
        Ok(rx
            .await
            .expect("rayon task channel closed unexpectedly")
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic)))
    }

    /// Spawn a blocking function on the Rayon thread pool (FIFO scheduling).
    ///
    /// Uses FIFO scheduling which prevents starvation of older tasks. This is the
    /// preferred variant for packet decoding and similar ordered workloads.
    ///
    /// Includes cooperative cancellation: if the receiver is dropped before the
    /// task starts (e.g., timeout), the task is skipped without executing.
    ///
    /// # Errors
    ///
    /// Returns [`SpawnError::QueueFull`] if the outstanding task count exceeds the limit.
    pub async fn spawn_fifo_blocking<R: Send + 'static>(
        f: impl FnOnce() -> R + Send + 'static,
        operation: &'static str,
    ) -> Result<R, SpawnError> {
        try_acquire_slot()?;
        let (task, rx) = cancellable_task(f, operation);
        rayon::spawn_fifo(task);
        Ok(rx
            .await
            .expect("rayon task channel closed unexpectedly")
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic)))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            Arc,
            atomic::{AtomicU32, Ordering},
        },
        time::Duration,
    };

    use futures::FutureExt;
    use serial_test::serial;

    use super::cpu;

    #[tokio::test]
    #[serial]
    async fn spawn_blocking_returns_result() {
        let result = cpu::spawn_blocking(|| 42, "test").await.unwrap();
        assert_eq!(result, 42);
    }

    #[tokio::test]
    #[serial]
    async fn spawn_fifo_blocking_returns_result() {
        let result = cpu::spawn_fifo_blocking(|| "hello", "test").await.unwrap();
        assert_eq!(result, "hello");
    }

    #[tokio::test]
    #[serial]
    async fn spawn_blocking_propagates_panic() {
        let result = std::panic::AssertUnwindSafe(async {
            cpu::spawn_blocking(
                || {
                    panic!("test panic");
                },
                "test",
            )
            .await
            .unwrap()
        })
        .catch_unwind()
        .await;
        assert!(result.is_err(), "should propagate panic from Rayon task");
    }

    #[tokio::test]
    #[serial]
    async fn cancelled_tasks_are_skipped_via_cooperative_cancellation() {
        let initial_outstanding = cpu::outstanding_tasks();
        let executed_count = Arc::new(AtomicU32::new(0));

        for _ in 0..100 {
            let count = executed_count.clone();
            let fut = cpu::spawn_fifo_blocking(
                move || {
                    count.fetch_add(1, Ordering::SeqCst);
                    std::thread::sleep(Duration::from_millis(50));
                },
                "test",
            );
            let _ = fut.now_or_never();
        }

        let start = std::time::Instant::now();
        let result = cpu::spawn_fifo_blocking(|| 42, "test").await.unwrap();
        let elapsed = start.elapsed();

        assert_eq!(result, 42);
        assert!(
            elapsed < Duration::from_secs(2),
            "Task took {elapsed:?} - cancelled tasks may not be getting skipped"
        );

        for _ in 0..50 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if cpu::outstanding_tasks() == initial_outstanding {
                break;
            }
        }

        let executed = executed_count.load(Ordering::SeqCst);
        assert!(
            executed < 50,
            "Expected most tasks to be skipped by cancellation, but {executed}/100 executed"
        );
    }

    #[tokio::test]
    #[serial]
    async fn pool_recovers_after_cancelled_burst() {
        let initial_outstanding = cpu::outstanding_tasks();

        for _ in 0..50 {
            let fut = cpu::spawn_fifo_blocking(
                || {
                    std::thread::sleep(Duration::from_millis(100));
                },
                "test",
            );
            let _ = fut.now_or_never();
        }

        tokio::time::sleep(Duration::from_millis(300)).await;

        for i in 0..10 {
            let start = std::time::Instant::now();
            let result = cpu::spawn_fifo_blocking(move || i * 2, "test").await.unwrap();
            let elapsed = start.elapsed();

            assert_eq!(result, i * 2);
            assert!(
                elapsed < Duration::from_millis(500),
                "Recovery task {i} took {elapsed:?} - pool may still be starved"
            );
        }

        for _ in 0..50 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if cpu::outstanding_tasks() == initial_outstanding {
                break;
            }
        }
    }

    #[tokio::test]
    #[serial]
    async fn outstanding_tasks_tracking() {
        let initial = cpu::outstanding_tasks();

        let barrier = Arc::new(std::sync::Barrier::new(2));
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            cpu::spawn_fifo_blocking(
                move || {
                    barrier_clone.wait();
                    42
                },
                "test",
            )
            .await
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        let during = cpu::outstanding_tasks();
        assert!(
            during > initial,
            "Outstanding should increase: initial={initial}, during={during}"
        );

        barrier.wait();

        let result = handle.await.unwrap();
        assert_eq!(result.unwrap(), 42);

        tokio::time::sleep(Duration::from_millis(50)).await;

        let after = cpu::outstanding_tasks();
        assert_eq!(after, initial, "Outstanding should return to initial after completion");
    }

    #[tokio::test]
    #[serial]
    async fn outstanding_decrements_on_cancellation() {
        let initial = cpu::outstanding_tasks();

        for _ in 0..10 {
            let fut = cpu::spawn_fifo_blocking(
                || {
                    std::thread::sleep(Duration::from_millis(100));
                },
                "test",
            );
            let _ = fut.now_or_never();
        }

        tokio::time::sleep(Duration::from_millis(500)).await;

        let after = cpu::outstanding_tasks();
        assert_eq!(after, initial, "Outstanding should return to initial after cancelled tasks drain");
    }
}
