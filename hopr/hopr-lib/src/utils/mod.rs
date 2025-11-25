#[cfg(all(feature = "utils_futures", not(feature = "runtime-tokio")))]
compile_error!("'utils_futures' feature requires the 'runtime-tokio' feature to be enabled");
#[cfg(feature = "utils_futures")]
pub use hopr_network_types::utils as futures;

#[cfg(all(feature = "utils_parallel", not(feature = "runtime-tokio")))]
compile_error!("'utils_parallel' feature requires the 'runtime-tokio' feature to be enabled");
#[cfg(feature = "utils_parallel")]
pub use hopr_parallelize as parallelize;
