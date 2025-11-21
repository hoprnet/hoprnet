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
pub mod cpu {
    pub use rayon;

    /// Initialize a `rayon` CPU thread pool with the given number of threads.
    pub fn init_thread_pool(num_threads: usize) -> Result<(), rayon::ThreadPoolBuildError> {
        rayon::ThreadPoolBuilder::new().num_threads(num_threads).build_global()
    }

    /// Spawn an awaitable non-blocking execution of the given blocking function on a `rayon` CPU thread pool.
    ///
    /// The current thread pool uses a LIFO (Last In First Out) scheduling policy for the thread's queue, but
    /// FIFO (First In First Out) for stealing tasks from other threads.
    #[cfg(feature = "rayon")]
    pub async fn spawn_blocking<R: Send + 'static>(f: impl FnOnce() -> R + Send + 'static) -> R {
        let (tx, rx) = futures::channel::oneshot::channel();
        rayon::spawn(|| {
            tx.send(std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)))
                .unwrap_or_else(|_| eprintln!("blocking task panicked"))
        });
        rx.await
            .expect("spawned blocking process should be awaitable")
            .unwrap_or_else(|caught_panic| std::panic::resume_unwind(caught_panic))
    }

    /// Spawn an awaitable non-blocking execution of the given blocking function on a `rayon` CPU thread pool.
    ///
    /// Executed tasks are loaded using a FIFO (First In First Out) scheduling policy.
    #[cfg(feature = "rayon")]
    pub async fn spawn_fifo_blocking<R: Send + 'static>(f: impl FnOnce() -> R + Send + 'static) -> R {
        let (tx, rx) = futures::channel::oneshot::channel();
        rayon::spawn_fifo(|| {
            tx.send(std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)))
                .unwrap_or_else(|_| eprintln!("blocking task panicked"))
        });
        rx.await
            .expect("spawned fifo blocking process should be awaitable")
            .unwrap_or_else(|caught_panic| std::panic::resume_unwind(caught_panic))
    }
}
