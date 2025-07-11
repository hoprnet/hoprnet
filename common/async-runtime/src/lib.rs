//! Executor API for HOPR which exposes the necessary async functions depending on the enabled
//! runtime.

pub use futures::future::AbortHandle;

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
    pub use futures::future::{AbortHandle, abortable};
    pub use tokio::{
        task::{JoinHandle, spawn, spawn_blocking, spawn_local},
        time::{sleep, timeout as timeout_fut},
    };
}

#[macro_export]
macro_rules! spawn_as_abortable {
    ($($expr:expr),*) => {{
        let (proc, abort_handle) = $crate::prelude::abortable($($expr),*);
        let _jh = $crate::prelude::spawn(proc);
        abort_handle
    }}
}

// If no runtime is enabled, fail compilation
#[cfg(all(not(feature = "runtime-tokio"), not(feature = "runtime-async-std")))]
compile_error!("No runtime enabled");
