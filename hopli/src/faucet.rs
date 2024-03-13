//! This module contains arguments and functions to fund some Ethereum wallets
//! with native tokens and HOPR tokens
//!
//! Despite HOPR contracts are mainly deployed on Gnosis chain,
//! HOPR token contract addresses vary on the network.
//!
//! Attention! Do not use this function to distribute large amount of tokens
///
/// Note that to save gas in batch funding, multicall is used to facilitate token distribution, via `transferFrom`
/// To use this functionality, caller must grant Multicall3 contract the exact allowance equal to the sum of tokens
/// to be transferred. As it's a separate function, there is a window between granting the allowance and executing
/// the transactin. Attacker may take advantage of this window and steal tokens from the caller's account.
use crate::{
    environment_config::NetworkProviderArgs,
    key_pair::{IdentityFileArgs, PrivateKeyArgs, PrivateKeyReader},
    methods::{get_native_and_token_balances, transfer_native_tokens, transfer_or_mint_tokens},
    utils::{Cmd, HelperErrors},
};
use bindings::hopr_token::HoprToken;
use clap::Parser;
use ethers::{
    types::{H160, U256},
    utils::parse_units,
};
// use ethers::types::Address;
use std::{ops::Sub, str::FromStr};
use tracing::info;

/// CLI arguments for `hopli faucet`
#[derive(Parser, Default, Debug)]
pub struct FaucetArgs {
    /// Network name, contracts config file root, and customized provider, if available
    #[clap(flatten)]
    pub network_provider: NetworkProviderArgs,

    /// Additional addresses (comma-separated) to receive funds.
    #[clap(
        help = "Comma-separated Ethereum addresses of nodes that will receive funds",
        long,
        short,
        default_value = None
    )]
    address: Option<String>,

    /// Argument to locate identity file(s)
    #[clap(flatten)]
    local_identity: IdentityFileArgs,

    /// The amount of HOPR tokens (in floating number) to be funded per wallet
    #[clap(
        help = "Hopr amount in ether, e.g. 10",
        long,
        short = 't',
        value_parser = clap::value_parser!(f64),
        default_value_t = 2000.0
    )]
    hopr_amount: f64,

    /// The amount of native tokens (in floating number) to be funded per wallet
    #[clap(
        help = "Native token amount in ether, e.g. 1",
        long,
        short = 'g',
        value_parser = clap::value_parser!(f64),
        default_value_t = 10.0
    )]
    native_amount: f64,

    /// Access to the private key, of which the wallet either contains sufficient assets
    /// as the source of funds or it can mint necessary tokens
    #[clap(flatten)]
    pub private_key: PrivateKeyArgs,
}

impl FaucetArgs {
    /// Execute the faucet command, which funds addresses with required amount of tokens
    async fn execute_faucet(self) -> Result<(), HelperErrors> {
        let FaucetArgs {
            network_provider,
            address,
            local_identity,
            hopr_amount,
            native_amount,
            private_key,
        } = self;

        // Include provided address
        let mut eth_addresses_all: Vec<H160> = Vec::new();
        if let Some(addresses) = address {
            eth_addresses_all.extend(addresses.split(',').map(|addr| H160::from_str(addr).unwrap()));
        }
        // if local identity dirs/path is provided, read addresses from identity files
        eth_addresses_all.extend(local_identity.to_addresses().unwrap().into_iter().map(H160::from));
        info!("All the addresses: {:?}", eth_addresses_all);

        // `PRIVATE_KEY` - Private key is required to send on-chain transactions
        let signer_private_key = private_key.read_default()?;

        // get RPC provider for the given network and environment
        let rpc_provider = network_provider.get_provider_with_signer(&signer_private_key).await?;
        let contract_addresses = network_provider.get_network_details_from_name()?;

        let hopr_token = HoprToken::new(contract_addresses.addresses.token, rpc_provider.clone());

        // complete actions as defined in `transferOrMintHoprAndSendNativeToAmount` in `SingleActions.s.sol`
        // get token and native balances for addresses
        let (native_balances, token_balances) =
            get_native_and_token_balances(hopr_token.clone(), eth_addresses_all.clone())
                .await
                .unwrap();
        // Get the amount of HOPR tokens that addresses need to receive to reach the desired amount
        let hopr_token_amounts: Vec<U256> =
            vec![U256::from(parse_units(hopr_amount, "ether").unwrap()); eth_addresses_all.len()]
                .into_iter()
                .enumerate()
                .map(|(i, h)| {
                    if h.gt(&token_balances[i]) {
                        h.sub(token_balances[i])
                    } else {
                        U256::zero()
                    }
                })
                .collect();

        // Get the amount of native tokens that addresses need to receive to reach the desired amount
        let native_token_amounts: Vec<U256> =
            vec![U256::from(parse_units(native_amount, "ether").unwrap()); eth_addresses_all.len()]
                .into_iter()
                .enumerate()
                .map(|(i, n)| {
                    if n.gt(&native_balances[i]) {
                        n.sub(native_balances[i])
                    } else {
                        U256::zero()
                    }
                })
                .collect();

        // transfer of mint HOPR tokens
        let total_transferred_hopr_token =
            transfer_or_mint_tokens(hopr_token, eth_addresses_all.clone(), hopr_token_amounts).await?;
        info!("total transferred hopr-token is {:?}", total_transferred_hopr_token);
        // send native tokens
        let total_transferred_native_token =
            transfer_native_tokens(rpc_provider, eth_addresses_all, native_token_amounts).await?;
        info!("total transferred native-token is {:?}", total_transferred_native_token);
        Ok(())
    }
}

impl Cmd for FaucetArgs {
    /// Run the execute_faucet function
    fn run(self) -> Result<(), HelperErrors> {
        Ok(())
    }

    async fn async_run(self) -> Result<(), HelperErrors> {
        self.execute_faucet().await
    }
}
