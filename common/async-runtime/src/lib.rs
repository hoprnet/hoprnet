//! Executor API for HOPR which exposes the necessary async functions depending on the enabled
//! runtime.
//!
//!
#[cfg(feature = "runtime-async-std")]
pub mod prelude {
    pub use async_std::future::timeout as timeout_fut;
    pub use async_std::task::{sleep, spawn, spawn_blocking, spawn_local, JoinHandle};

    pub async fn cancel_join_handle<T>(handle: JoinHandle<T>) {
        handle.cancel().await;
    }
}

// Both features could be enabled during testing, therefore we only use tokio when its
// exclusively enabled.
#[cfg(all(feature = "runtime-tokio", not(feature = "runtime-async-std")))]
pub mod prelude {
    pub use tokio::time::timeout as timeout_fut;
    pub use tokio::{
        task::{spawn, spawn_blocking, spawn_local, JoinHandle},
        time::sleep,
    };

    pub async fn cancel_join_handle<T>(handle: JoinHandle<T>) {
        handle.abort()
    }
}

// If no runtime is enabled, fail compilation
#[cfg(all(not(feature = "runtime-tokio"), not(feature = "runtime-async-std")))]
compile_error!("No runtime enabled");
