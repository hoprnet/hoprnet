//! Hoprd configuration utility `hoprd-cfg`
//!
//! This executable offers functionalities associated with configuration management
//! of the HOPRd node configuration.
//!
//! ## Help
//! ```shell
//! ➜   hoprd-cfg --help
//! Usage: hoprd-cfg [OPTIONS]
//!
//! Options:
//!   -d, --default              Print the default YAML config for the hoprd
//!   -v, --validate <VALIDATE>  Validate the config at this path
//!   -h, --help                 Print help
//!   -V, --version              Print version
//! ```
//!
//! ## Dump a default configuration file
//! ```shell
//! ➜   hoprd-cfg -d     
//! hopr:
//! host:
//!   address: !IPv4 0.0.0.0
//!   port: 9091
//!
//! ... <snip>
//!
//! ```
//!
//! ## Validate an existing configuration YAML
//!
//! ```shell
//! ➜   hoprd-cfg -v /tmp/bad-config.yaml
//! Error: ValidationError("The specified network 'anvil-localhost' is not listed as supported ([\"debug-staging\", \"dufour\", \"rotsee\"])")
//!
//! ➜   echo $?
//! 1
//! ```

use std::path::PathBuf;

use clap::Parser;

use hoprd::config::HoprdConfig;
use validator::Validate;

#[derive(Parser, Default)]
#[clap(author, version, about, long_about = None)]
struct CliArgs {
    /// Print the default YAML config for the hoprd
    #[clap(short = 'd', long, conflicts_with = "validate")]
    default: bool,
    /// Validate the config at this path
    #[clap(short, long, conflicts_with = "default")]
    validate: Option<PathBuf>,
}

fn main() -> Result<(), hoprd::errors::HoprdError> {
    let args = CliArgs::parse();

    if args.default {
        println!(
            "{}",
            serde_yaml::to_string(&hoprd::config::HoprdConfig::default())
                .map_err(|e| hoprd::errors::HoprdError::ConfigError(e.to_string()))?
        );
    } else if let Some(cfg_path) = args.validate {
        let cfg_path = cfg_path
            .into_os_string()
            .into_string()
            .map_err(|_| hoprd::errors::HoprdError::ConfigError("file path not convertible".into()))?;

        let yaml_configuration = hopr_platform::file::native::read_to_string(&cfg_path)
            .map_err(|e| hoprd::errors::HoprdError::ConfigError(e.to_string()))?;

        let cfg: HoprdConfig = serde_yaml::from_str(&yaml_configuration)
            .map_err(|e| hoprd::errors::HoprdError::SerializationError(e.to_string()))?;

        if !cfg
            .hopr
            .chain
            .protocols
            .supported_networks(hopr_lib::constants::APP_VERSION_COERCED)
            .iter()
            .any(|network| network == &cfg.hopr.chain.network)
        {
            return Err(hoprd::errors::HoprdError::ValidationError(format!(
                "The specified network '{}' is not listed as supported ({:?})",
                cfg.hopr.chain.network,
                cfg.hopr
                    .chain
                    .protocols
                    .supported_networks(hopr_lib::constants::APP_VERSION_COERCED)
            )));
        };

        if let Err(e) = cfg.validate() {
            return Err(hoprd::errors::HoprdError::ValidationError(e.to_string()));
        };
    }

    Ok(())
}
