//! This module contains arguments and functions to interact with the Winning Probability contract for a privileged
//! account. It can set the global minimum winning probability and read the current global minimum winning probability.
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
//!     --provider-url "http://localhost:8545"
//! ```
use clap::Parser;
use hopr_bindings::{
    exports::alloy::primitives::aliases::U56, hopr_winning_probability_oracle::HoprWinningProbabilityOracle,
};
use hopr_internal_types::{prelude::WinningProbability, tickets::EncodedWinProb};
use tracing::{debug, info};

use crate::{
    environment_config::NetworkProviderArgs,
    key_pair::{ArgEnvReader, PrivateKeyArgs},
    utils::{Cmd, HelperErrors, a2h},
};

/// CLI arguments for `hopli win-prob`
#[derive(Clone, Debug, Parser)]
pub enum WinProbSubcommands {
    /// Set the global minimum ticket winning probability as an owner
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
            a2h(contract_addresses.addresses.winning_probability_oracle),
            rpc_provider.clone(),
        );

        // convert the winning probability to the format required by the contract
        let winning_probability_val = WinningProbability::try_from(winning_probability).map_err(|_| {
            HelperErrors::ParseError("Failed to convert winning probability to the required format".into())
        })?;

        info!(
            winning_probability = %winning_probability_val,
            win_prob_uint56 = %winning_probability,
            "Setting the global minimum winning probability"
        );

        hopr_win_prob
            .setWinProb(U56::from_be_slice(&winning_probability_val.as_encoded()))
            .send()
            .await?
            .watch()
            .await?;
        Ok(())
    }

    pub async fn execute_get_win_prob(network_provider: NetworkProviderArgs) -> Result<f64, HelperErrors> {
        // get RPC provider for the given network and environment
        let rpc_provider = network_provider.get_provider_without_signer().await?;
        let contract_addresses = network_provider.get_network_details_from_name()?;

        let hopr_win_prob = HoprWinningProbabilityOracle::new(
            a2h(contract_addresses.addresses.winning_probability_oracle),
            rpc_provider.clone(),
        );

        // get winning probability from the contract
        let current_win_prob =
            hopr_win_prob.currentWinProb().call().await.map_err(|e| {
                HelperErrors::MiddlewareError(format!("Failed to get current winning probability: {e}"))
            })?;

        // convert into f64
        let mut tmp: EncodedWinProb = Default::default();
        tmp.copy_from_slice(&current_win_prob.to_be_bytes::<7>());
        let current_win_prob = WinningProbability::from(tmp);
        let current_win_prob_f64 = current_win_prob.as_f64();
        info!(
            current_win_prob_f64 = %current_win_prob_f64,
            current_win_prob_uint56 = %current_win_prob,
            "Current global minimum winning probability"
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
