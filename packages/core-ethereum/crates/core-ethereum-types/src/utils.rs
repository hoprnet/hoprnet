//! Chain utilities used for testing.
//! This used in unit and integration tests.

use ethers::prelude::*;
use bindings::hopr_channels::HoprChannels;
use bindings::hopr_token::HoprToken;
use utils_types::primitives::Address;
use crate::TypedTransaction;

/// Used for testing. Creates local Anvil instance.
/// When block time is given, new blocks is mined periodically.
/// Otherwise, a new block is mined per transaction.
#[cfg(not(target_arch = "wasm32"))]
pub fn create_anvil(block_time: Option<std::time::Duration>) -> ethers::utils::AnvilInstance {
    let mut anvil = ethers::utils::Anvil::new()
        .path(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../.foundry/bin/anvil"));

    if let Some(bt) = block_time {
        anvil = anvil.block_time(bt.as_secs());
    }

    anvil.spawn()
}

/// Mints specified amount of HOPR tokens to the contract deployer wallet.
/// Assumes that the `hopr_token` contract is associated with a RPC client that also deployed the contract.
/// Returns the block number at which the minting transaction was confirmed.
pub async fn mint_tokens<M: Middleware + 'static>(hopr_token: HoprToken<M>, amount: utils_types::primitives::U256) -> u64 {
    let deployer = hopr_token.client().default_sender().expect("client must have a signer");
    hopr_token
        .grant_role(hopr_token.minter_role().await.unwrap(), deployer)
        .send()
        .await
        .unwrap();
    hopr_token
        .mint(deployer, amount.into(), Bytes::new(), Bytes::new())
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap()
        .block_number
        .unwrap()
        .as_u64()
}

/// Creates a transaction that transfers the given `amount` of native tokens to the
/// given destination.
pub fn create_native_transfer(to: Address, amount: utils_types::primitives::U256) -> TypedTransaction {
    let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());
    tx.set_to(H160::from(to));
    tx.set_value(amount);
    tx
}

/// Funds the given wallet address with specified amount of native tokens and HOPR tokens.
/// These must be present in the client's wallet.
pub async fn fund_node<M: Middleware>(
    node: Address,
    native_token: utils_types::primitives::U256,
    hopr_token: utils_types::primitives::U256,
    hopr_token_contract: HoprToken<M>,
) -> () {
    let native_transfer_tx = Eip1559TransactionRequest::new()
        .to(NameOrAddress::Address(node.into()))
        .value(native_token);

    let client = hopr_token_contract.client();

    client
        .send_transaction(native_transfer_tx, None)
        .await
        .unwrap()
        .await
        .unwrap();

    hopr_token_contract
        .transfer(node.into(), hopr_token.into())
        .send()
        .await
        .unwrap()
        .await
        .unwrap();
}

/// Funds the channel to the counterparty with the given amount of HOPR tokens.
/// The amount must be present in the wallet of the client.
pub async fn fund_channel<M: Middleware>(
    counterparty: Address,
    hopr_token: HoprToken<M>,
    hopr_channels: HoprChannels<M>,
    amount: utils_types::primitives::U256,
) {
    hopr_token
        .approve(hopr_channels.address(), amount.into())
        .send()
        .await
        .unwrap()
        .await
        .unwrap();

    hopr_channels
        .fund_channel(counterparty.into(), amount.as_u128())
        .send()
        .await
        .unwrap()
        .await
        .unwrap();
}