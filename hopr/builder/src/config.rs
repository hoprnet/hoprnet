use std::{collections::HashSet, net::SocketAddr, time::Duration};

use serde_with::serde_as;

/// Configuration of the Exit node (see [`HoprServerIpForwardingReactor`](crate::exit::HoprServerIpForwardingReactor))
/// and the Entry node.
#[serde_as]
#[derive(
    Clone, Debug, Eq, PartialEq, smart_default::SmartDefault, serde::Deserialize, serde::Serialize, validator::Validate,
)]
pub struct SessionIpForwardingConfig {
    /// Controls whether allowlisting should be done via `target_allow_list`.
    /// If set to `false`, the node will act as an Exit node for any target.
    ///
    /// Defaults to `true`.
    #[serde(default = "just_true")]
    #[default(true)]
    pub use_target_allow_list: bool,

    /// Enforces only the given target addresses (after DNS resolution).
    ///
    /// This is used only if `use_target_allow_list` is set to `true`.
    /// If left empty (and `use_target_allow_list` is `true`), the node will not act as an Exit node.
    ///
    /// Defaults to empty.
    #[serde(default)]
    #[serde_as(as = "HashSet<serde_with::DisplayFromStr>")]
    pub target_allow_list: HashSet<SocketAddr>,

    /// Delay between retries in seconds to reach a TCP target.
    ///
    /// Defaults to 2 seconds.
    #[serde(default = "default_target_retry_delay")]
    #[default(default_target_retry_delay())]
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub tcp_target_retry_delay: Duration,

    /// Maximum number of retries to reach a TCP target before giving up.
    ///
    /// Default is 10.
    #[serde(default = "default_max_tcp_target_retries")]
    #[default(default_max_tcp_target_retries())]
    #[validate(range(min = 1))]
    pub max_tcp_target_retries: u32,

    /// Specifies the default `listen_host` for Session listening sockets
    /// at an Entry node.
    #[serde(default = "default_entry_listen_host")]
    #[default(default_entry_listen_host())]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub default_entry_listen_host: SocketAddr,
}

fn default_target_retry_delay() -> Duration {
    Duration::from_secs(2)
}

fn default_entry_listen_host() -> SocketAddr {
    "127.0.0.1:0".parse().unwrap()
}

fn default_max_tcp_target_retries() -> u32 {
    10
}

fn just_true() -> bool {
    true
}
