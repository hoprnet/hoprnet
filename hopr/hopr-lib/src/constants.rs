use hopr_primitive_types::balance::XDaiBalance;

/// Application version as presented externally using the heartbeat mechanism
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application version coerced into the `major`.`minor`.`patch` form
pub const APP_VERSION_COERCED: &str = const_format::formatcp!(
    "{}.{}.{}",
    env!("CARGO_PKG_VERSION_MAJOR"),
    env!("CARGO_PKG_VERSION_MINOR"),
    env!("CARGO_PKG_VERSION_PATCH")
);

lazy_static::lazy_static! {
    /// Minimum native token balance to start the node.
    pub static ref  MIN_NATIVE_BALANCE: XDaiBalance = "0.001 xdai".parse().unwrap();
    /// Balance that is suggested for funding the node the first time to make it start.
    pub static ref  SUGGESTED_NATIVE_BALANCE: XDaiBalance = "0.01 xdai".parse().unwrap();
}

pub const ONBOARDING_INFORMATION_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);

/// Default minimum quality of stored peer entries to re-sync from the persistent storage on
/// node's startup.
pub const DEFAULT_MIN_QUALITY_TO_SYNC: f64 = 0.9;
