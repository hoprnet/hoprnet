//! Parallelization utilities for CPU and IO heavy workloads.
//!
//! More information about parallization, execution and executors can be found [here](https://ryhl.io/blog/async-what-is-blocking/)

/// Module for real thread pool based parallelization of CPU heavy blocking workloads.
pub mod cpu {
    pub use rayon;

    /// Spawn an awaitable non-blocking execution of the given blocking function on a `rayon` CPU thread pool.
    ///
    /// The current thread pool uses a LIFO (Last In First Out) scheduling policy for the thread's queue, but
    /// FIFO (First In First Out) for stealing tasks from other threads.
    #[cfg(feature = "rayon")]
    pub async fn spawn_blocking<R: Send + 'static>(f: impl FnOnce() -> R + Send + 'static) -> R {
        let (tx, rx) = futures::channel::oneshot::channel();
        rayon::spawn(|| {
            tx.send(std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)))
                .unwrap_or_else(|_| unreachable!())
        });
        rx.await
            .unwrap()
            .unwrap_or_else(|caught_panic| std::panic::resume_unwind(caught_panic))
    }

    /// Spawn an awaitable non-blocking execution of the given blocking function on a `rayon` CPU thread pool.
    ///
    /// Executed tasks are loaded using a FIFO (First In First Out) (Last In First Out) scheduling policy.
    #[cfg(feature = "rayon")]
    pub async fn spawn_fifo_blocking<R: Send + 'static>(f: impl FnOnce() -> R + Send + 'static) -> R {
        let (tx, rx) = futures::channel::oneshot::channel();
        rayon::spawn_fifo(|| {
            tx.send(std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)))
                .unwrap_or_else(|_| unreachable!())
        });
        rx.await
            .unwrap()
            .unwrap_or_else(|caught_panic| std::panic::resume_unwind(caught_panic))
    }
}

/// Module for executor based parallelization of blocking IO heavy workloads.
pub mod sync_io {
    #[cfg(feature = "runtime-async-std")]
    pub use async_std::task::spawn_blocking;

    #[cfg(feature = "runtime-tokio")]
    pub use tokio::task::spawn_blocking;
}
