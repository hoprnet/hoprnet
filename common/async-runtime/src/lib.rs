//! Executor API for HOPR which exposes the necessary async functions depending on the enabled
//! runtime.

pub use futures::future::AbortHandle;

// Both features could be enabled during testing; therefore, we only use tokio when it's
// exclusively enabled.
#[cfg(feature = "runtime-tokio")]
pub mod prelude {
    pub use futures::future::{AbortHandle, abortable};
    pub use tokio::{
        task::{JoinError, JoinHandle, spawn, spawn_blocking, spawn_local},
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
#[cfg(not(feature = "runtime-tokio"))]
compile_error!("No runtime enabled");
