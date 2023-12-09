/// Application version as presented externally using the heartbeat mechanism
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_VERSION_COERCED: &str = const_format::formatcp!(
    "{}.{}.{}",
    env!("CARGO_PKG_VERSION_MAJOR"),
    env!("CARGO_PKG_VERSION_MINOR"),
    env!("CARGO_PKG_VERSION_PATCH")
);

pub const DB_VERSION_TAG: &str = "main_4";

pub const MIN_NATIVE_BALANCE: &str = "1000000000000000"; // 0.001
pub const SUGGESTED_NATIVE_BALANCE: &str = "10000000000000000"; // 0.01
