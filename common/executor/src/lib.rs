//! Executor API for HOPR which exposes the necessary async functions depending on the enabled
//! runtime.
//!
//!
#[cfg(any(test, feature = "runtime-async-std"))]
pub mod api {
    pub use async_std::future::timeout as timeout_fut;
    pub use async_std::task::{sleep, spawn, spawn_blocking, spawn_local, JoinHandle};

    pub async fn cancel_join_handle<T>(handle: JoinHandle<T>) {
        handle.cancel().await;
    }
}

#[cfg(all(not(test), feature = "runtime-tokio", not(feature = "runtime-async-std")))]
pub mod api {
    pub use tokio::time::timeout as timeout_fut;
    pub use tokio::{
        task::{spawn, spawn_blocking, spawn_local, JoinHandle},
        time::sleep,
    };

    pub async fn cancel_join_handle<T>(handle: JoinHandle<T>) {
        handle.abort()
    }
}
