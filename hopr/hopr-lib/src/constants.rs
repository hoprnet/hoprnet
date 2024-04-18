/// Application version as presented externally using the heartbeat mechanism
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application version coerced into the `major`.`minor`.`patch` form
pub const APP_VERSION_COERCED: &str = const_format::formatcp!(
    "{}.{}.{}",
    env!("CARGO_PKG_VERSION_MAJOR"),
    env!("CARGO_PKG_VERSION_MINOR"),
    env!("CARGO_PKG_VERSION_PATCH")
);

/// Minimum native token balance to start the node.
pub const MIN_NATIVE_BALANCE: &str = "1000000000000000"; // 0.001
/// Balance that is suggested for funding the node the first time in order to make it start.
pub const SUGGESTED_NATIVE_BALANCE: &str = "10000000000000000"; // 0.01
