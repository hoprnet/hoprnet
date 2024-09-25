//! This module contains arguments and functions to interact with the Winning Probability contract for a privileged account.
//! It can set the global minimum winning probability and read the current global minimum winning probability.
//! Some sample commands:
//! - Set winning probability:
//! ```text
//! hopli win-prob set \
//!     --network anvil-localhost \
//!     --contracts-root "../ethereum/contracts" \
//!     --winning-probability 0.5 \
//!     --private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
//!     --provider-url "http://localhost:8545"
//! ```
//! - Get winning probability:
//! ```text
//! hopli win-prob get \
//!     --network anvil-localhost \
//!     --contracts-root "../ethereum/contracts" \
//!     --private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
//!     --provider-url "http://localhost:8545"
//! ```
use crate::key_pair::ArgEnvReader;
use crate::{
    environment_config::NetworkProviderArgs,
    key_pair::PrivateKeyArgs,
    utils::{Cmd, HelperErrors},
};
use bindings::hopr_winning_probability_oracle::HoprWinningProbabilityOracle;
use clap::Parser;
use hopr_lib::{f64_to_win_prob, win_prob_to_f64};
use tracing::{debug, info};

/// CLI arguments for `hopli win-prob`
#[derive(Clone, Debug, Parser)]
pub enum WinProbSubcommands {
    // Set the global minimum ticket winning probability as an owner
    #[command(visible_alias = "s")]
    Set {
        /// Network name, contracts config file root, and customized provider, if available
        #[command(flatten)]
        network_provider: NetworkProviderArgs,

        /// New winning probability
        #[clap(help = "New winning probability", short = 'w', long, default_value_t = 1.0f64)]
        winning_probability: f64,

        /// Access to the private key of a manager of Network Registry contract
        #[command(flatten)]
        private_key: PrivateKeyArgs,
    },

    /// Read the current global minimum winning probability
    #[command(visible_alias = "g")]
    Get {
        /// Network name, contracts config file root, and customized provider, if available
        #[command(flatten)]
        network_provider: NetworkProviderArgs,
    },
}

impl WinProbSubcommands {
    pub async fn execute_set_win_prob(
        network_provider: NetworkProviderArgs,
        winning_probability: f64,
        private_key: PrivateKeyArgs,
    ) -> Result<(), HelperErrors> {
        // Read the private key from arguments or the "PRIVATE_KEY" environment variable
        let signer_private_key = private_key.read("PRIVATE_KEY")?;

        // get RPC provider for the given network and environment
        let rpc_provider = network_provider.get_provider_with_signer(&signer_private_key).await?;
        let contract_addresses = network_provider.get_network_details_from_name()?;

        let hopr_win_prob = HoprWinningProbabilityOracle::new(
            contract_addresses.addresses.winning_probability_oracle,
            rpc_provider.clone(),
        );

        // convert the winning probability to the format required by the contract
        let winning_probability = f64_to_win_prob(winning_probability).map_err(|_| {
            HelperErrors::ParseError("Failed to convert winning probability to the required format".into())
        })?;

        // convert the new winning probability
        let mut win_prob_param = [0u8; 8];
        win_prob_param[1..].copy_from_slice(&winning_probability);
        let win_prob_param = u64::from_be_bytes(win_prob_param);

        info!(
            "Setting the global minimum winning probability to {:?} ({:?} in uint56 format)",
            winning_probability, win_prob_param
        );

        hopr_win_prob
            .set_win_prob(win_prob_param)
            .send()
            .await
            .map_err(|e| HelperErrors::MiddlewareError(format!("Failed in broadcasting transactions {:?}", e)))?
            .await
            .map_err(|e| HelperErrors::MiddlewareError(format!("Failed in getting receipt {:?}", e)))?;
        Ok(())
    }

    pub async fn execute_get_win_prob(network_provider: NetworkProviderArgs) -> Result<f64, HelperErrors> {
        // get RPC provider for the given network and environment
        let rpc_provider = network_provider.get_provider_without_signer().await?;
        let contract_addresses = network_provider.get_network_details_from_name()?;

        let hopr_win_prob = HoprWinningProbabilityOracle::new(
            contract_addresses.addresses.winning_probability_oracle,
            rpc_provider.clone(),
        );

        // get winning probability from the contract
        let current_win_prob = hopr_win_prob
            .current_win_prob()
            .await
            .map_err(|e| HelperErrors::MiddlewareError(format!("Failed to get current winning probability: {}", e)))?;

        // convert into f64
        let mut tmp = [0u8; 7];
        tmp.copy_from_slice(&current_win_prob.to_be_bytes()[1..]);
        let current_win_prob_f64 = win_prob_to_f64(&tmp);
        info!(
            "Current global minimum winning probability is {:?} ({:?} in uint56 format)",
            current_win_prob_f64, current_win_prob
        );
        Ok(current_win_prob_f64)
    }
}

impl Cmd for WinProbSubcommands {
    /// Run the execute_register function.
    /// By default, registration is done by manager wallet
    fn run(self) -> Result<(), HelperErrors> {
        Ok(())
    }

    async fn async_run(self) -> Result<(), HelperErrors> {
        match self {
            WinProbSubcommands::Set {
                network_provider,
                winning_probability,
                private_key,
            } => {
                WinProbSubcommands::execute_set_win_prob(network_provider, winning_probability, private_key).await?;
            }
            WinProbSubcommands::Get { network_provider } => {
                let win_prob = WinProbSubcommands::execute_get_win_prob(network_provider).await?;
                debug!("Current global minimum winning probability is: {}", win_prob);
            }
        }
        Ok(())
    }
}
