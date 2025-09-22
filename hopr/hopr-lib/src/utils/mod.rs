#[cfg(all(feature = "utils_session", not(feature = "runtime-tokio")))]
compile_error!("'utils_session' feature requires the 'runtime-tokio' feature to be enabled");

/// Session-related utilities (available when utils_session feature is enabled)
#[cfg(feature = "utils_session")]
pub mod session;
