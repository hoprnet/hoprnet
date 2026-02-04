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
/// ## Observability
///
/// Prometheus metrics (behind the `prometheus` feature) track:
/// - **submitted**: total tasks entering the Rayon queue
/// - **completed**: tasks that delivered results to a live receiver
/// - **cancelled**: tasks skipped via cooperative cancellation (receiver dropped before execution)
/// - **orphaned**: tasks that ran to completion but whose receiver was dropped during execution
/// - **queue_wait**: histogram of time between submission and execution start
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

    /// Initialize a `rayon` CPU thread pool with the given number of threads.
    pub fn init_thread_pool(num_threads: usize) -> Result<(), rayon::ThreadPoolBuildError> {
        rayon::ThreadPoolBuilder::new().num_threads(num_threads).build_global()
    }

    /// Builds a cancellable task closure and its corresponding oneshot receiver.
    ///
    /// The returned closure wraps `f` with cooperative cancellation, panic catching,
    /// queue-wait timing, and Prometheus metric updates. It is suitable for passing
    /// directly to [`rayon::spawn`] or [`rayon::spawn_fifo`].
    ///
    /// See the [module-level documentation](cpu) for why cooperative cancellation
    /// is necessary.
    #[cfg(feature = "rayon")]
    fn cancellable_task<R: Send + 'static>(
        f: impl FnOnce() -> R + Send + 'static,
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
                return;
            }

            let wait_duration = submitted_at.elapsed();
            #[cfg(all(feature = "prometheus", not(test)))]
            METRIC_RAYON_QUEUE_WAIT.observe(wait_duration.as_secs_f64());

            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
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
        };

        (task, rx)
    }

    /// Spawn an awaitable non-blocking execution of the given blocking function on a `rayon` CPU thread pool.
    ///
    /// The current thread pool uses a LIFO (Last In First Out) scheduling policy for the thread's queue, but
    /// FIFO (First In First Out) for stealing tasks from other threads.
    ///
    /// Includes cooperative cancellation: if the async receiver is dropped (e.g., due to timeout)
    /// before the Rayon task starts executing, the task is skipped without performing any work.
    /// See the [module-level documentation](cpu) for details on why this is necessary.
    #[cfg(feature = "rayon")]
    pub async fn spawn_blocking<R: Send + 'static>(f: impl FnOnce() -> R + Send + 'static) -> R {
        let (task, rx) = cancellable_task(f);
        rayon::spawn(task);
        rx.await
            .expect("rayon task channel closed unexpectedly")
            .unwrap_or_else(|caught_panic| std::panic::resume_unwind(caught_panic))
    }

    /// Spawn an awaitable non-blocking execution of the given blocking function on a `rayon` CPU thread pool.
    ///
    /// Executed tasks are loaded using a FIFO (First In First Out) scheduling policy.
    /// This is the primary variant used by packet decoding, where FIFO ordering prevents
    /// starvation of older tasks by newer arrivals.
    ///
    /// Includes cooperative cancellation: if the async receiver is dropped (e.g., due to timeout)
    /// before the Rayon task starts executing, the task is skipped without performing any work.
    /// See the [module-level documentation](cpu) for details on why this is necessary.
    #[cfg(feature = "rayon")]
    pub async fn spawn_fifo_blocking<R: Send + 'static>(f: impl FnOnce() -> R + Send + 'static) -> R {
        let (task, rx) = cancellable_task(f);
        rayon::spawn_fifo(task);
        rx.await
            .expect("rayon task channel closed unexpectedly")
            .unwrap_or_else(|caught_panic| std::panic::resume_unwind(caught_panic))
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

    use super::cpu;

    #[tokio::test]
    async fn spawn_blocking_returns_result() {
        let result = cpu::spawn_blocking(|| 42).await;
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn spawn_fifo_blocking_returns_result() {
        let result = cpu::spawn_fifo_blocking(|| "hello").await;
        assert_eq!(result, "hello");
    }

    #[tokio::test]
    async fn spawn_blocking_propagates_panic() {
        let result = std::panic::AssertUnwindSafe(cpu::spawn_blocking(|| {
            panic!("test panic");
        }))
        .catch_unwind()
        .await;
        assert!(result.is_err(), "should propagate panic from Rayon task");
    }

    /// Verifies that cooperative cancellation skips tasks whose receivers have been
    /// dropped (simulating a timeout). Without cancellation, each task would run its
    /// full 50ms sleep, causing massive delays. With cancellation, skipped tasks complete
    /// in nanoseconds, so the final task should return promptly.
    #[tokio::test]
    async fn cancelled_tasks_are_skipped_via_cooperative_cancellation() {
        let executed_count = Arc::new(AtomicU32::new(0));

        // Submit many slow tasks and immediately cancel them by dropping the future.
        // We poll once via now_or_never() so the async fn body executes up to the rx.await,
        // which submits the Rayon task. Then dropping the future drops rx, marking the task
        // as cancelled.
        for _ in 0..100 {
            let count = executed_count.clone();
            let fut = cpu::spawn_fifo_blocking(move || {
                count.fetch_add(1, Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(50));
            });
            // Poll once to submit the Rayon task, then drop the future (drops receiver)
            let _ = fut.now_or_never();
        }

        // Submit a task that should complete promptly because cancelled tasks are skipped
        let start = std::time::Instant::now();
        let result = cpu::spawn_fifo_blocking(|| 42).await;
        let elapsed = start.elapsed();

        assert_eq!(result, 42);

        // Without cancellation: 100 tasks * 50ms / num_threads ≈ seconds of delay
        // With cancellation: cancelled tasks are skipped in nanoseconds
        assert!(
            elapsed < Duration::from_secs(2),
            "Task took {elapsed:?} - cancelled tasks may not be getting skipped"
        );

        // Allow remaining Rayon tasks to drain
        tokio::time::sleep(Duration::from_millis(500)).await;
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
    async fn pool_recovers_after_cancelled_burst() {
        // Saturate with tasks that will be cancelled
        for _ in 0..50 {
            let fut = cpu::spawn_fifo_blocking(|| {
                std::thread::sleep(Duration::from_millis(100));
            });
            let _ = fut.now_or_never();
        }

        // Wait for any in-flight tasks to complete
        tokio::time::sleep(Duration::from_millis(300)).await;

        // Now verify the pool is healthy: multiple sequential tasks should all complete promptly
        for i in 0..10 {
            let start = std::time::Instant::now();
            let result = cpu::spawn_fifo_blocking(move || i * 2).await;
            let elapsed = start.elapsed();

            assert_eq!(result, i * 2);
            assert!(
                elapsed < Duration::from_millis(500),
                "Recovery task {i} took {elapsed:?} - pool may still be starved"
            );
        }
    }
}
