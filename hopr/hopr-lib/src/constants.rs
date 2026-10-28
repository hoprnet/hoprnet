use hopr_api::types::primitive::balance::XDaiBalance;

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

/// Recommended minimum kernel UDP socket buffer size (`net.core.rmem_default` /
/// `net.core.wmem_default`), matching the value that resolved the throughput regression
/// described in <https://github.com/hoprnet/hoprnet/issues/8503>.
pub const RECOMMENDED_MIN_UDP_BUFFER_BYTES: u64 = 4 * 1024 * 1024;

/// TCP congestion-control algorithm recommended for better throughput on lossy/high-latency
/// links (`net.ipv4.tcp_congestion_control`).
pub const RECOMMENDED_CONGESTION_CONTROL: &str = "bbr";
