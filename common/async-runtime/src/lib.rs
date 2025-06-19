//! Executor API for HOPR which exposes the necessary async functions depending on the enabled
//! runtime.

pub use futures::future::AbortHandle;
use futures::future::abortable;

#[cfg(feature = "runtime-async-std")]
#[deprecated(note = "Use `runtime-tokio` feature, the `async-std` crate is deprecated")]
pub mod prelude {
    pub use async_std::{
        future::timeout as timeout_fut,
        task::{JoinHandle, sleep, spawn, spawn_blocking, spawn_local},
    };
}

// Both features could be enabled during testing; therefore, we only use tokio when it's
// exclusively enabled.
#[cfg(feature = "runtime-tokio")]
pub mod prelude {
    pub use tokio::{
        task::{JoinHandle, spawn, spawn_blocking, spawn_local},
        time::{sleep, timeout as timeout_fut},
    };
}

pub fn spawn_as_abortable<F, T>(f: F) -> AbortHandle
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let (proc, abort_handle) = abortable(f);
    let _jh = prelude::spawn(proc);
    abort_handle
}

// If no runtime is enabled, fail compilation
#[cfg(all(not(feature = "runtime-tokio"), not(feature = "runtime-async-std")))]
compile_error!("No runtime enabled");
