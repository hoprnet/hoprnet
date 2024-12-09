/// The default HTTP request engine
///
/// TODO: Should be an internal type, `hopr_lib::chain` must be moved to this package
#[cfg(feature = "runtime-async-std")]
pub type DefaultHttpPostRequestor = crate::client::surf_client::SurfRequestor;

// Both features could be enabled during testing, therefore we only use tokio when its
// exclusively enabled.
#[cfg(all(feature = "runtime-tokio", not(feature = "runtime-async-std")))]
pub type DefaultHttpPostRequestor = crate::client::reqwest_client::ReqwestRequestor;

/// The default JSON RPC provider client
pub type JsonRpcClient =
    crate::client::JsonRpcProviderClient<DefaultHttpPostRequestor, crate::client::SimpleJsonRpcRetryPolicy>;
