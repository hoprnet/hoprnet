//! Parallelization utilities for different types of workloads.
//!
//! ## Blocking and non-blocking code
//! Using `async` executor carries the need to consider different types of operations happening during the
//! code execution. Primarily, the task in async code can be divided into *blocking* and *non-blocking*,
//! where a code execution is considered blocking, if it does not allow the executor to swap the current task
//! (i.e. to jump to a different code execution block). A good rule of thumb for efficient asynchronous code
//! is to not have more than 10 to 100 microseconds between each .await.
//!
//! ## What if blocking is needed
//! Sometimes it is necessary to block a thread, e.g. when performing a CPU intensive task or waiting for a
//! synchronous IO operation. Because these blocking operations would prevent the async executor to jump to a
//! different task, effectively blocking it, one of the 3 possible strategies must be used to offload the
//! blocking task from the executor's thread:
//! 1. use executor native `spawn_blocking` to spawn the blocking task to a dedicated pool of blocking tasks running
//!    alongside the executor threads
//!    - this solution allows to offload tasks onto typically hundreds of threads
//!    - because there are typically too many threads, such a scenario is ideal for synchronous blocking IO
//! 2. use a dedicated parallelization mechanism with its own thread pool
//!    - solution most typically used for the CPU heavy tasks
//!    - allows execution of the task over a smaller thread pool fully optimizing each CPU
//! 3. use a `thread`
//!    - used typically when a blocking operation keeps running forever
//!
//! More information about parallelization, execution and executors can be found in an excellent blog post [here](https://ryhl.io/blog/async-what-is-blocking/).

/// Module for real thread pool-based parallelization of CPU heavy blocking workloads.
///
/// ## Zombie task prevention
///
/// The Rayon thread pool is intentionally small (sized to CPU cores) because it handles
/// CPU-heavy crypto operations (EC multiplication, ECDSA signing, MAC verification).
/// Callers wrap these tasks with timeouts (e.g., 150ms for packet decoding). When a
/// timeout fires, the async receiver side (`rx`) is dropped, but Rayon has no native
/// cancellation—the closure continues running to completion as a "zombie" task whose
/// result is silently discarded.
///
/// Under sustained load, zombie accumulation can starve the pool: timed-out tasks
/// continue occupying threads, causing subsequent tasks to also time out, creating
/// more zombies in a feedback loop. The pool becomes permanently saturated even after
/// load drops, because each arriving packet (including protocol heartbeats) creates
/// zombies faster than the pool can drain them.
///
/// To break this cycle, each spawned closure checks `tx.is_canceled()` before executing.
/// If the receiver was already dropped (timeout fired while the task waited in the queue),
/// the closure returns immediately instead of burning CPU time.
///
/// ## Queue depth limiting
///
/// To prevent unbounded queue growth under sustained load, the module tracks the number
/// of tasks currently queued. Callers can use [`try_spawn_blocking`] or [`try_spawn_fifo_blocking`]
/// to get a fallible spawn that returns [`SpawnError::QueueFull`] when the queue limit is reached.
/// This allows callers to implement backpressure by dropping packets gracefully instead of
/// accumulating unbounded work.
///
/// The queue limit defaults to `10 * rayon::current_num_threads()` but can be overridden
/// via the `HOPR_CPU_TASK_QUEUE_LIMIT` environment variable.
///
/// ## Observability
///
/// Prometheus metrics (behind the `prometheus` feature) track:
/// - **submitted**: total tasks entering the Rayon queue
/// - **completed**: tasks that delivered results to a live receiver
/// - **cancelled**: tasks skipped via cooperative cancellation (receiver dropped before execution)
/// - **orphaned**: tasks that ran to completion but whose receiver was dropped during execution
/// - **queue_wait**: histogram of time between submission and execution start
/// - **execution_time**: histogram of actual task execution duration
/// - **queue_depth**: current number of tasks in the queue
/// - **queue_limit**: configured maximum queue depth (for comparison with queue_depth)
/// - **rejected**: total tasks rejected due to queue being full
///
/// A rising cancelled or orphaned count relative to submitted indicates pool saturation.
/// Sustained queue_wait times approaching the caller's timeout threshold signal imminent
/// cascading failures.
pub mod cpu {
    pub use rayon;

    #[cfg(all(feature = "prometheus", not(test)))]
    static METRIC_RAYON_TASKS_SUBMITTED: std::sync::LazyLock<hopr_metrics::SimpleCounter> =
        std::sync::LazyLock::new(|| {
            hopr_metrics::SimpleCounter::new(
                "hopr_rayon_tasks_submitted_total",
                "Total number of tasks submitted to the Rayon thread pool",
            )
            .unwrap()
        });

    #[cfg(all(feature = "prometheus", not(test)))]
    static METRIC_RAYON_TASKS_COMPLETED: std::sync::LazyLock<hopr_metrics::SimpleCounter> =
        std::sync::LazyLock::new(|| {
            hopr_metrics::SimpleCounter::new(
                "hopr_rayon_tasks_completed_total",
                "Total number of Rayon tasks that completed and delivered results",
            )
            .unwrap()
        });

    #[cfg(all(feature = "prometheus", not(test)))]
    static METRIC_RAYON_TASKS_CANCELLED: std::sync::LazyLock<hopr_metrics::SimpleCounter> =
        std::sync::LazyLock::new(|| {
            hopr_metrics::SimpleCounter::new(
                "hopr_rayon_tasks_cancelled_total",
                "Total number of Rayon tasks skipped because receiver was already dropped",
            )
            .unwrap()
        });

    #[cfg(all(feature = "prometheus", not(test)))]
    static METRIC_RAYON_TASKS_ORPHANED: std::sync::LazyLock<hopr_metrics::SimpleCounter> =
        std::sync::LazyLock::new(|| {
            hopr_metrics::SimpleCounter::new(
                "hopr_rayon_tasks_orphaned_total",
                "Total number of Rayon tasks whose results were discarded after completion",
            )
            .unwrap()
        });

    #[cfg(all(feature = "prometheus", not(test)))]
    static METRIC_RAYON_QUEUE_WAIT: std::sync::LazyLock<hopr_metrics::SimpleHistogram> =
        std::sync::LazyLock::new(|| {
            hopr_metrics::SimpleHistogram::new(
                "hopr_rayon_queue_wait_seconds",
                "Time tasks spend waiting in the Rayon queue before execution starts",
                vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.15, 0.25, 0.5, 1.0],
            )
            .unwrap()
        });

    #[cfg(all(feature = "prometheus", not(test)))]
    static METRIC_RAYON_EXECUTION_TIME: std::sync::LazyLock<hopr_metrics::MultiHistogram> =
        std::sync::LazyLock::new(|| {
            hopr_metrics::MultiHistogram::new(
                "hopr_rayon_execution_seconds",
                "Time tasks spend executing in the Rayon thread pool",
                vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.15, 0.25, 0.5, 1.0],
                &["operation"],
            )
            .unwrap()
        });

    #[cfg(all(feature = "prometheus", not(test)))]
    static METRIC_RAYON_QUEUE_DEPTH: std::sync::LazyLock<hopr_metrics::SimpleGauge> = std::sync::LazyLock::new(|| {
        hopr_metrics::SimpleGauge::new(
            "hopr_rayon_queue_depth",
            "Current number of tasks waiting in the Rayon queue",
        )
        .unwrap()
    });

    #[cfg(all(feature = "prometheus", not(test)))]
    static METRIC_RAYON_QUEUE_LIMIT: std::sync::LazyLock<hopr_metrics::SimpleGauge> = std::sync::LazyLock::new(|| {
        hopr_metrics::SimpleGauge::new(
            "hopr_rayon_queue_limit",
            "Configured maximum queue depth for the Rayon thread pool",
        )
        .unwrap()
    });

    #[cfg(all(feature = "prometheus", not(test)))]
    static METRIC_RAYON_TASKS_REJECTED: std::sync::LazyLock<hopr_metrics::SimpleCounter> =
        std::sync::LazyLock::new(|| {
            hopr_metrics::SimpleCounter::new(
                "hopr_rayon_tasks_rejected_total",
                "Total number of tasks rejected due to queue being full",
            )
            .unwrap()
        });

    use std::sync::{
        OnceLock,
        atomic::{AtomicUsize, Ordering},
    };

    /// Current number of tasks in the queue (submitted but not yet completed/cancelled/orphaned).
    static QUEUE_DEPTH: AtomicUsize = AtomicUsize::new(0);

    /// Lazily initialized queue limit.
    static QUEUE_LIMIT: OnceLock<usize> = OnceLock::new();

    /// Default multiplier for queue limit relative to thread count.
    const DEFAULT_QUEUE_LIMIT_MULTIPLIER: usize = 10;

    /// Environment variable name for overriding queue limit.
    const QUEUE_LIMIT_ENV_VAR: &str = "HOPR_CPU_TASK_QUEUE_LIMIT";

    /// Error type for fallible spawn operations.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SpawnError {
        /// The queue is full and cannot accept more tasks.
        QueueFull {
            /// Current queue depth when rejection occurred.
            current: usize,
            /// Configured queue limit.
            limit: usize,
        },
    }

    impl std::fmt::Display for SpawnError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                SpawnError::QueueFull { current, limit } => {
                    write!(f, "rayon queue full: {current}/{limit} tasks queued")
                }
            }
        }
    }

    impl std::error::Error for SpawnError {}

    /// Returns the configured queue limit, initializing it on first call.
    fn get_queue_limit() -> usize {
        *QUEUE_LIMIT.get_or_init(|| {
            let limit = std::env::var(QUEUE_LIMIT_ENV_VAR)
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .filter(|&v| v > 0)
                .unwrap_or_else(|| rayon::current_num_threads() * DEFAULT_QUEUE_LIMIT_MULTIPLIER);

            #[cfg(all(feature = "prometheus", not(test)))]
            METRIC_RAYON_QUEUE_LIMIT.set(limit as f64);

            limit
        })
    }

    /// Attempts to acquire a queue slot using compare-and-swap.
    /// Returns the new queue depth on success, or `SpawnError::QueueFull` if the limit is reached.
    fn try_acquire_queue_slot() -> Result<usize, SpawnError> {
        let limit = get_queue_limit();
        loop {
            let current = QUEUE_DEPTH.load(Ordering::Relaxed);
            if current >= limit {
                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_RAYON_TASKS_REJECTED.increment();
                return Err(SpawnError::QueueFull { current, limit });
            }
            match QUEUE_DEPTH.compare_exchange_weak(current, current + 1, Ordering::AcqRel, Ordering::Relaxed) {
                Ok(_) => {
                    let new_depth = current + 1;
                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_RAYON_QUEUE_DEPTH.set(new_depth as f64);
                    return Ok(new_depth);
                }
                Err(_) => continue, // CAS failed, retry
            }
        }
    }

    /// Releases a queue slot, decrementing the depth counter.
    fn release_queue_slot() {
        let prev = QUEUE_DEPTH.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(prev > 0, "queue depth underflow");
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_RAYON_QUEUE_DEPTH.set((prev - 1) as f64);
    }

    /// Returns the current queue depth (tasks submitted but not yet completed).
    pub fn current_queue_depth() -> usize {
        QUEUE_DEPTH.load(Ordering::Relaxed)
    }

    /// Returns the configured queue limit.
    pub fn queue_limit() -> usize {
        get_queue_limit()
    }

    /// Initialize a `rayon` CPU thread pool with the given number of threads.
    ///
    /// This also initializes the queue limit metric so it's available immediately
    /// after thread pool initialization.
    pub fn init_thread_pool(num_threads: usize) -> Result<(), rayon::ThreadPoolBuildError> {
        let result = rayon::ThreadPoolBuilder::new().num_threads(num_threads).build_global();
        // Initialize queue limit (and its metric) now that the thread pool is ready
        let _ = get_queue_limit();
        result
    }

    /// Builds a cancellable task closure and its corresponding oneshot receiver.
    ///
    /// The returned closure wraps `f` with cooperative cancellation, panic catching,
    /// queue-wait timing, and Prometheus metric updates. It is suitable for passing
    /// directly to [`rayon::spawn`] or [`rayon::spawn_fifo`].
    ///
    /// If `track_queue_depth` is true, the task will release a queue slot on completion.
    /// This should be true when the task was submitted via `try_spawn_*` functions.
    ///
    /// See the [module-level documentation](cpu) for why cooperative cancellation
    /// is necessary.
    #[cfg(feature = "rayon")]
    fn cancellable_task<R: Send + 'static>(
        f: impl FnOnce() -> R + Send + 'static,
        track_queue_depth: bool,
        operation: &'static str,
    ) -> (
        impl FnOnce() + Send + 'static,
        futures::channel::oneshot::Receiver<std::thread::Result<R>>,
    ) {
        let (tx, rx) = futures::channel::oneshot::channel();
        let submitted_at = std::time::Instant::now();

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_RAYON_TASKS_SUBMITTED.increment();

        let task = move || {
            // Cooperative cancellation: if the caller already timed out and dropped
            // the receiver, skip the (potentially expensive) closure entirely.
            // This is a best-effort check—there's an inherent race between this check
            // and the timeout firing, but it catches the common case where a task sat
            // in the queue past its deadline, preventing zombie accumulation.
            if tx.is_canceled() {
                tracing::debug!(
                    queue_wait_ms = submitted_at.elapsed().as_millis() as u64,
                    "skipping cancelled task (receiver dropped before execution)"
                );
                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_RAYON_TASKS_CANCELLED.increment();
                if track_queue_depth {
                    release_queue_slot();
                }
                return;
            }

            let wait_duration = submitted_at.elapsed();
            #[cfg(all(feature = "prometheus", not(test)))]
            METRIC_RAYON_QUEUE_WAIT.observe(wait_duration.as_secs_f64());

            #[cfg(all(feature = "prometheus", not(test)))]
            let execution_start = std::time::Instant::now();

            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));

            #[cfg(all(feature = "prometheus", not(test)))]
            METRIC_RAYON_EXECUTION_TIME.observe(&[operation], execution_start.elapsed().as_secs_f64());

            match tx.send(result) {
                Ok(()) => {
                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_RAYON_TASKS_COMPLETED.increment();
                }
                // The receiver was dropped between the is_canceled() check above and
                // task completion (the timeout fired while the closure was running).
                // The work is wasted but unavoidable without mid-closure cancellation.
                Err(_) => {
                    tracing::debug!(
                        queue_wait_ms = wait_duration.as_millis() as u64,
                        "receiver dropped during execution, result discarded"
                    );
                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_RAYON_TASKS_ORPHANED.increment();
                }
            }
            if track_queue_depth {
                release_queue_slot();
            }
        };

        (task, rx)
    }

    /// Spawn an awaitable non-blocking execution of the given blocking function on a `rayon` CPU thread pool.
    ///
    /// The current thread pool uses a LIFO (Last In First Out) scheduling policy for the thread's queue, but
    /// FIFO (First In First Out) for stealing tasks from other threads.
    ///
    /// The queue limit defaults to `10 * rayon::current_num_threads()` but can be overridden
    /// via the `HOPR_CPU_TASK_QUEUE_LIMIT` environment variable.
    ///
    /// Includes cooperative cancellation: if the async receiver is dropped (e.g., due to timeout)
    /// before the Rayon task starts executing, the task is skipped without performing any work.
    /// See the [module-level documentation](cpu) for details on why this is necessary.
    ///
    /// # Errors
    ///
    /// Returns [`SpawnError::QueueFull`] if the queue depth has reached the configured limit.
    #[cfg(feature = "rayon")]
    pub async fn spawn_blocking<R: Send + 'static>(
        f: impl FnOnce() -> R + Send + 'static,
        operation: &'static str,
    ) -> Result<R, SpawnError> {
        try_acquire_queue_slot()?;
        let (task, rx) = cancellable_task(f, true, operation);
        rayon::spawn(task);
        Ok(rx
            .await
            .expect("rayon task channel closed unexpectedly")
            .unwrap_or_else(|caught_panic| std::panic::resume_unwind(caught_panic)))
    }

    /// Spawn an awaitable non-blocking execution of the given blocking function on a `rayon` CPU thread pool.
    ///
    /// Executed tasks are loaded using a FIFO (First In First Out) scheduling policy.
    /// This is the primary variant used by packet decoding, where FIFO ordering prevents
    /// starvation of older tasks by newer arrivals.
    ///
    /// The queue limit defaults to `10 * rayon::current_num_threads()` but can be overridden
    /// via the `HOPR_CPU_TASK_QUEUE_LIMIT` environment variable.
    ///
    /// Includes cooperative cancellation: if the async receiver is dropped (e.g., due to timeout)
    /// before the Rayon task starts executing, the task is skipped without performing any work.
    /// See the [module-level documentation](cpu) for details on why this is necessary.
    ///
    /// # Errors
    ///
    /// Returns [`SpawnError::QueueFull`] if the queue depth has reached the configured limit.
    #[cfg(feature = "rayon")]
    pub async fn spawn_fifo_blocking<R: Send + 'static>(
        f: impl FnOnce() -> R + Send + 'static,
        operation: &'static str,
    ) -> Result<R, SpawnError> {
        try_acquire_queue_slot()?;
        let (task, rx) = cancellable_task(f, true, operation);
        rayon::spawn_fifo(task);
        Ok(rx
            .await
            .expect("rayon task channel closed unexpectedly")
            .unwrap_or_else(|caught_panic| std::panic::resume_unwind(caught_panic)))
    }

    /// Alias for [`spawn_blocking`] for backwards compatibility.
    #[cfg(feature = "rayon")]
    #[inline]
    pub async fn try_spawn_blocking<R: Send + 'static>(
        f: impl FnOnce() -> R + Send + 'static,
        operation: &'static str,
    ) -> Result<R, SpawnError> {
        spawn_blocking(f, operation).await
    }

    /// Alias for [`spawn_fifo_blocking`] for backwards compatibility.
    #[cfg(feature = "rayon")]
    #[inline]
    pub async fn try_spawn_fifo_blocking<R: Send + 'static>(
        f: impl FnOnce() -> R + Send + 'static,
        operation: &'static str,
    ) -> Result<R, SpawnError> {
        spawn_fifo_blocking(f, operation).await
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

    /// Verifies that cooperative cancellation skips tasks whose receivers have been
    /// dropped (simulating a timeout). Without cancellation, each task would run its
    /// full 50ms sleep, causing massive delays. With cancellation, skipped tasks complete
    /// in nanoseconds, so the final task should return promptly.
    #[tokio::test]
    #[serial]
    async fn cancelled_tasks_are_skipped_via_cooperative_cancellation() {
        let initial_depth = cpu::current_queue_depth();
        let executed_count = Arc::new(AtomicU32::new(0));

        // Submit many slow tasks and immediately cancel them by dropping the future.
        // We poll once via now_or_never() so the async fn body executes up to the rx.await,
        // which submits the Rayon task. Then dropping the future drops rx, marking the task
        // as cancelled.
        for _ in 0..100 {
            let count = executed_count.clone();
            let fut = cpu::spawn_fifo_blocking(
                move || {
                    count.fetch_add(1, Ordering::SeqCst);
                    std::thread::sleep(Duration::from_millis(50));
                },
                "test",
            );
            // Poll once to submit the Rayon task, then drop the future (drops receiver)
            let _ = fut.now_or_never();
        }

        // Submit a task that should complete promptly because cancelled tasks are skipped
        let start = std::time::Instant::now();
        let result = cpu::spawn_fifo_blocking(|| 42, "test").await.unwrap();
        let elapsed = start.elapsed();

        assert_eq!(result, 42);

        // Without cancellation: 100 tasks * 50ms / num_threads ≈ seconds of delay
        // With cancellation: cancelled tasks are skipped in nanoseconds
        assert!(
            elapsed < Duration::from_secs(2),
            "Task took {elapsed:?} - cancelled tasks may not be getting skipped"
        );

        // Allow remaining Rayon tasks to drain and wait for queue depth to return to initial
        for _ in 0..50 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if cpu::current_queue_depth() == initial_depth {
                break;
            }
        }

        let executed = executed_count.load(Ordering::SeqCst);

        // Most tasks should have been skipped due to cancellation.
        // A few may have started before their receiver was dropped.
        assert!(
            executed < 50,
            "Expected most tasks to be skipped by cancellation, but {executed}/100 executed"
        );
    }

    /// Verifies that after a burst of timed-out tasks, the pool recovers and new tasks
    /// complete normally without lingering starvation.
    #[tokio::test]
    #[serial]
    async fn pool_recovers_after_cancelled_burst() {
        let initial_depth = cpu::current_queue_depth();

        // Saturate with tasks that will be cancelled
        for _ in 0..50 {
            let fut = cpu::spawn_fifo_blocking(
                || {
                    std::thread::sleep(Duration::from_millis(100));
                },
                "test",
            );
            let _ = fut.now_or_never();
        }

        // Wait for any in-flight tasks to complete
        tokio::time::sleep(Duration::from_millis(300)).await;

        // Now verify the pool is healthy: multiple sequential tasks should all complete promptly
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

        // Wait for queue depth to return to initial value
        for _ in 0..50 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if cpu::current_queue_depth() == initial_depth {
                break;
            }
        }
    }

    #[tokio::test]
    #[serial]
    async fn try_spawn_blocking_returns_result() {
        let result = cpu::try_spawn_blocking(|| 42, "test").await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    #[serial]
    async fn try_spawn_fifo_blocking_returns_result() {
        let result = cpu::try_spawn_fifo_blocking(|| "hello", "test").await;
        assert_eq!(result.unwrap(), "hello");
    }

    #[tokio::test]
    #[serial]
    async fn queue_depth_tracking() {
        // This test verifies that queue depth increases when a task is submitted
        // and decreases when it completes.
        let initial_depth = cpu::current_queue_depth();

        // Submit a slow task via try_spawn using a barrier to control execution
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let barrier_clone = barrier.clone();

        let task_handle = tokio::spawn(async move {
            cpu::try_spawn_fifo_blocking(
                move || {
                    barrier_clone.wait();
                    42
                },
                "test",
            )
            .await
        });

        // Give the task time to be submitted to rayon
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Queue depth should have increased by 1
        let depth_during = cpu::current_queue_depth();
        assert!(
            depth_during > initial_depth,
            "Queue depth should increase after submitting task: initial={initial_depth}, during={depth_during}"
        );

        // Release the barrier to let the task complete
        barrier.wait();

        // Wait for the task to complete and verify it returned correctly
        let result = task_handle.await.unwrap();
        assert_eq!(result.unwrap(), 42);

        // Give time for queue depth to decrement
        tokio::time::sleep(Duration::from_millis(50)).await;

        let final_depth = cpu::current_queue_depth();
        assert_eq!(
            final_depth, initial_depth,
            "Queue depth should return to initial value after task completion"
        );
    }

    #[tokio::test]
    #[serial]
    async fn queue_full_returns_error() {
        // Temporarily set a very low queue limit by saturating the queue
        // We need to submit more tasks than the limit allows
        let limit = cpu::queue_limit();

        // Use barriers to hold tasks in the queue
        let barriers: Vec<_> = (0..limit + 5).map(|_| Arc::new(std::sync::Barrier::new(2))).collect();

        let mut handles = Vec::new();

        // Submit tasks up to and beyond the limit
        for (i, barrier) in barriers.iter().enumerate() {
            let barrier_clone = barrier.clone();
            let handle = tokio::spawn(async move {
                cpu::try_spawn_fifo_blocking(
                    move || {
                        barrier_clone.wait();
                        i
                    },
                    "test",
                )
                .await
            });
            handles.push(handle);

            // Small delay to ensure sequential submission
            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        // Check that at least one task was rejected due to queue being full
        // Release all barriers so tasks can complete
        for barrier in &barriers {
            std::thread::spawn({
                let b = barrier.clone();
                move || b.wait()
            });
        }

        let mut rejected_count = 0;
        for handle in handles {
            let result = handle.await.unwrap();
            if result.is_err() {
                rejected_count += 1;
                if let Err(cpu::SpawnError::QueueFull { current, limit }) = result {
                    assert!(current >= limit, "QueueFull should report current >= limit");
                }
            }
        }

        // At least some tasks should have been rejected
        // (we submitted limit + 5 tasks)
        assert!(
            rejected_count > 0,
            "Expected at least one task to be rejected due to queue being full, but all succeeded. limit={limit}"
        );

        // Wait for tasks to drain
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    #[tokio::test]
    #[serial]
    async fn queue_depth_decrements_on_cancellation() {
        // This test verifies that cancelled tasks properly release their queue slots.
        let initial_depth = cpu::current_queue_depth();

        // Submit tasks that will be cancelled via cooperative cancellation
        for _ in 0..10 {
            let fut = cpu::try_spawn_fifo_blocking(
                || {
                    std::thread::sleep(Duration::from_millis(100));
                },
                "test",
            );
            // Poll once to submit the Rayon task, then drop the future (drops receiver)
            let _ = fut.now_or_never();
        }

        // Wait for cancelled tasks to be processed and release their slots
        tokio::time::sleep(Duration::from_millis(500)).await;

        let final_depth = cpu::current_queue_depth();
        assert_eq!(
            final_depth, initial_depth,
            "Queue depth should return to initial value after cancelled tasks are processed"
        );
    }
}
