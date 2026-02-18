use std::{collections::HashSet, net::SocketAddr, time::Duration};

use hopr_lib::{
    HoprProtocolConfig, SafeModule, WinningProbability,
    config::{
        HoprLibConfig, HoprPacketPipelineConfig, HostConfig, HostType, ProbeConfig, SessionGlobalConfig,
        TransportConfig,
    },
    exports::transport::config::HoprCodecConfig,
};
use hoprd_api::config::{Api, Auth};
use proc_macro_regex::regex;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use validator::{Validate, ValidationError, ValidationErrors};

pub const DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 9091;

// Validate that the path is a valid UTF-8 path.
//
// Also used to perform the identity file existence check on the
// specified path, which is now circumvented but could
// return in the future workflows of setting up a node.
fn validate_file_path(_s: &str) -> Result<(), ValidationError> {
    Ok(())

    // if std::path::Path::new(_s).is_file() {
    //     Ok(())
    // } else {
    //     Err(ValidationError::new(
    //         "Invalid file path specified, the file does not exist or is not a file",
    //     ))
    // }
}

fn validate_password(s: &str) -> Result<(), ValidationError> {
    if !s.is_empty() {
        Ok(())
    } else {
        Err(ValidationError::new("No password could be found"))
    }
}

regex!(is_private_key "^(0[xX])?[a-fA-F0-9]{128}$");

pub(crate) fn validate_private_key(s: &str) -> Result<(), ValidationError> {
    if is_private_key(s) {
        Ok(())
    } else {
        Err(ValidationError::new("No valid private key could be found"))
    }
}

fn validate_optional_private_key(s: &str) -> Result<(), ValidationError> {
    validate_private_key(s)
}

#[derive(Default, Serialize, Deserialize, Validate, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    #[validate(custom(function = "validate_file_path"))]
    #[serde(default)]
    pub file: String,
    #[validate(custom(function = "validate_password"))]
    #[serde(default)]
    pub password: String,
    #[validate(custom(function = "validate_optional_private_key"))]
    #[serde(default)]
    pub private_key: Option<String>,
}

impl std::fmt::Debug for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let obfuscated: String = "<REDACTED>".into();

        f.debug_struct("Identity")
            .field("file", &self.file)
            .field("password", &obfuscated)
            .field("private_key", &obfuscated)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct Db {
    /// Path to the directory containing the database
    #[serde(default)]
    pub data: String,
    /// Determines whether the database should be initialized upon startup.
    #[serde(default = "just_true")]
    #[default = true]
    pub initialize: bool,
    /// Determines whether the database should be forcibly-initialized if it exists upon startup.
    #[serde(default)]
    pub force_initialize: bool,
}

fn default_session_idle_timeout() -> Duration {
    HoprLibConfig::default().protocol.session.idle_timeout
}

fn default_max_sessions() -> usize {
    HoprLibConfig::default().protocol.session.maximum_sessions as usize
}

fn default_session_establish_max_retries() -> usize {
    HoprLibConfig::default().protocol.session.establish_max_retries as usize
}

fn default_probe_recheck_threshold() -> Duration {
    HoprLibConfig::default().protocol.probe.recheck_threshold
}

fn default_probe_interval() -> Duration {
    HoprLibConfig::default().protocol.probe.interval
}

fn default_outgoing_ticket_winning_prob() -> Option<f64> {
    HoprLibConfig::default()
        .protocol
        .packet
        .codec
        .outgoing_win_prob
        .map(|p| p.as_f64())
}

/// Subset of various selected HOPR library network-related configuration options.
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserHoprNetworkConfig {
    /// How long it takes before HOPR Session is considered idle and is closed automatically
    #[default(default_session_idle_timeout())]
    #[serde(default = "default_session_idle_timeout", with = "humantime_serde")]
    pub session_idle_timeout: Duration,
    /// Maximum number of outgoing or incoming Sessions allowed by the Session manager
    #[default(default_max_sessions())]
    #[serde(default = "default_max_sessions")]
    pub maximum_sessions: usize,
    /// How many retries are made to establish an outgoing HOPR Session
    #[default(default_session_establish_max_retries())]
    #[serde(default = "default_session_establish_max_retries")]
    pub session_establish_max_retries: usize,
    /// The time interval for which to consider peer re-probing in seconds
    #[default(default_probe_recheck_threshold())]
    #[serde(default = "default_probe_recheck_threshold", with = "humantime_serde")]
    pub probe_recheck_threshold: Duration,
    /// The delay between individual probing rounds for neighbor discovery
    #[default(default_probe_interval())]
    #[serde(default = "default_probe_interval", with = "humantime_serde")]
    pub probe_interval: Duration,
    /// Should local addresses be announced on-chain?
    #[serde(default)]
    pub announce_local_addresses: bool,
    /// Should local addresses be preferred when dialing a peer?
    #[serde(default)]
    pub prefer_local_addresses: bool,
    /// Outgoing ticket winning probability.
    #[default(default_outgoing_ticket_winning_prob())]
    #[serde(default = "default_outgoing_ticket_winning_prob")]
    pub outgoing_ticket_winning_prob: Option<f64>,
}

/// Subset of the [`HoprLibConfig`] that is tuned to be user-facing and more user-friendly.
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserHoprLibConfig {
    /// Determines whether the node should be advertised publicly on-chain.
    #[default(just_true())]
    #[serde(default = "just_true")]
    pub announce: bool,
    /// Configuration related to host specifics
    #[default(default_host())]
    #[serde(default = "default_host")]
    pub host: HostConfig,
    /// Safe and Module configuration
    #[serde(default)]
    pub safe_module: SafeModule,
    /// Various HOPR-network and transport-related configuration options.
    #[serde(default)]
    pub network: UserHoprNetworkConfig,
}

// NOTE: this intentionally does not validate (0.0.0.0) to force user to specify
// their external IP.
#[inline]
fn default_host() -> HostConfig {
    HostConfig {
        address: HostType::IPv4(hopr_lib::config::DEFAULT_HOST.to_owned()),
        port: hopr_lib::config::DEFAULT_PORT,
    }
}

impl From<UserHoprLibConfig> for HoprLibConfig {
    fn from(value: UserHoprLibConfig) -> Self {
        HoprLibConfig {
            host: value.host,
            publish: value.announce,
            safe_module: value.safe_module,
            protocol: HoprProtocolConfig {
                transport: TransportConfig {
                    announce_local_addresses: value.network.announce_local_addresses,
                    prefer_local_addresses: value.network.prefer_local_addresses,
                },
                packet: HoprPacketPipelineConfig {
                    codec: HoprCodecConfig {
                        outgoing_win_prob: value
                            .network
                            .outgoing_ticket_winning_prob
                            .and_then(|v| WinningProbability::try_from_f64(v).ok()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                probe: ProbeConfig {
                    interval: value.network.probe_interval,
                    recheck_threshold: value.network.probe_recheck_threshold,
                    ..Default::default()
                },
                session: SessionGlobalConfig {
                    idle_timeout: value.network.session_idle_timeout,
                    maximum_sessions: value.network.maximum_sessions as u32,
                    establish_max_retries: value.network.session_establish_max_retries as u32,
                    ..Default::default()
                },
            },
        }
    }
}

impl Validate for UserHoprLibConfig {
    fn validate(&self) -> Result<(), ValidationErrors> {
        HoprLibConfig::from(self.clone()).validate()
    }
}

/// The main configuration object of the entire node.
///
/// The configuration is composed of individual configurations of corresponding
/// component configuration objects.
///
/// An always up-to-date config YAML example can be found in [example_cfg.yaml](https://github.com/hoprnet/hoprnet/tree/master/hoprd/hoprd/example_cfg.yaml)
/// which is always in the root of this crate.
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq, smart_default::SmartDefault)]
#[serde(deny_unknown_fields)]
pub struct HoprdConfig {
    /// Configuration related to hopr-lib functionality
    #[validate(nested)]
    #[serde(default)]
    pub hopr: UserHoprLibConfig,
    /// Configuration regarding the identity of the node
    #[validate(nested)]
    #[serde(default)]
    pub identity: Identity,
    /// Configuration of the underlying database engine
    #[validate(nested)]
    #[serde(default)]
    pub db: Db,
    /// Configuration relevant for the API of the node
    #[validate(nested)]
    #[serde(default)]
    pub api: Api,
    /// Configuration of the Session entry/exit node IP protocol forwarding.
    #[validate(nested)]
    #[serde(default)]
    pub session_ip_forwarding: SessionIpForwardingConfig,
    /// Blokli provider URL to connect to.
    #[validate(url)]
    pub blokli_url: Option<String>,
    /// Configuration of underlying node behavior in the form strategies
    ///
    /// Strategies represent automatically executable behavior performed by
    /// the node given pre-configured triggers.
    #[validate(nested)]
    #[serde(default = "hopr_strategy::hopr_default_strategies")]
    #[default(hopr_strategy::hopr_default_strategies())]
    pub strategy: hopr_strategy::StrategyConfig,
}

impl HoprdConfig {
    pub fn as_redacted(&self) -> Self {
        let mut ret = self.clone();
        // redacting sensitive information
        match ret.api.auth {
            Auth::None => {}
            Auth::Token(_) => ret.api.auth = Auth::Token("<REDACTED>".to_owned()),
        }

        if ret.identity.private_key.is_some() {
            ret.identity.private_key = Some("<REDACTED>".to_owned());
        }

        "<REDACTED>".clone_into(&mut ret.identity.password);

        ret
    }

    pub fn as_redacted_string(&self) -> crate::errors::Result<String> {
        let redacted_cfg = self.as_redacted();
        serde_json::to_string(&redacted_cfg).map_err(|e| crate::errors::HoprdError::SerializationError(e.to_string()))
    }
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

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        str::FromStr,
    };

    use anyhow::Context;
    use clap::{Args, Command, FromArgMatches};
    use hopr_lib::Address;
    use tempfile::NamedTempFile;

    use super::*;

    pub fn example_cfg() -> anyhow::Result<HoprdConfig> {
        let safe_module = hopr_lib::config::SafeModule {
            safe_address: Address::from_str("0x0000000000000000000000000000000000000000")?,
            module_address: Address::from_str("0x0000000000000000000000000000000000000000")?,
        };

        let identity = Identity {
            file: "path/to/identity.file".to_string(),
            password: "change_me".to_owned(),
            private_key: None,
        };

        let host = HostConfig {
            address: HostType::IPv4("1.2.3.4".into()),
            port: 9091,
        };

        Ok(HoprdConfig {
            hopr: UserHoprLibConfig {
                host,
                safe_module,
                ..Default::default()
            },
            db: Db {
                data: "/app/db".to_owned(),
                ..Default::default()
            },
            identity,
            ..HoprdConfig::default()
        })
    }

    #[test]
    fn test_config_should_be_serializable_into_string() -> anyhow::Result<()> {
        let cfg = example_cfg()?;

        let from_yaml: HoprdConfig = serde_saphyr::from_str(include_str!("../example_cfg.yaml"))?;
        assert_eq!(cfg, from_yaml);

        Ok(())
    }

    #[test]
    fn test_config_should_be_deserializable_from_a_string_in_a_file() -> anyhow::Result<()> {
        let mut config_file = NamedTempFile::new()?;
        let mut prepared_config_file = config_file.reopen()?;

        let cfg = example_cfg()?;
        let yaml = serde_saphyr::to_string(&cfg)?;
        config_file.write_all(yaml.as_bytes())?;

        let mut buf = String::new();
        prepared_config_file.read_to_string(&mut buf)?;
        let deserialized_cfg: HoprdConfig = serde_saphyr::from_str(&buf)?;

        assert_eq!(deserialized_cfg, cfg);

        Ok(())
    }

    /// TODO: This test attempts to deserialize the data structure incorrectly in the native build
    /// (`confirmations`` are an extra field), as well as misses the native implementation for the
    /// version satisfies check
    #[test]
    #[ignore]
    fn test_config_is_extractable_from_the_cli_arguments() -> anyhow::Result<()> {
        let pwnd = "rpc://pawned!";

        let mut config_file = NamedTempFile::new()?;

        let mut cfg = example_cfg()?;
        cfg.blokli_url = Some(pwnd.to_owned());

        let yaml = serde_saphyr::to_string(&cfg)?;
        config_file.write_all(yaml.as_bytes())?;
        let cfg_file_path = config_file
            .path()
            .to_str()
            .context("file path should have a string representation")?
            .to_string();

        let cli_args = vec!["hoprd", "--configurationFilePath", cfg_file_path.as_str()];

        let mut cmd = Command::new("hoprd").version("0.0.0");
        cmd = crate::cli::CliArgs::augment_args(cmd);
        let derived_matches = cmd.try_get_matches_from(cli_args)?;
        let args = crate::cli::CliArgs::from_arg_matches(&derived_matches)?;

        // skipping validation
        let cfg = HoprdConfig::try_from(args)?;

        assert_eq!(cfg.blokli_url, Some(pwnd.to_owned()));

        Ok(())
    }
}
