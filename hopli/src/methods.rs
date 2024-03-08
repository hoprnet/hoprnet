use crate::utils::HelperErrors;
use bindings::{hopr_network_registry::HoprNetworkRegistry, hopr_token::HoprToken};
use chain_rpc::TypedTransaction;
use ethers::{
    abi::AbiEncode,
    contract::{
        multicall_contract::{Aggregate3ValueCall, Call3Value},
        ContractError, Multicall, MulticallError, MULTICALL_ADDRESS,
    },
    providers::Middleware,
    types::{Address, Bytes, Eip1559TransactionRequest, H160, U256},
    utils::keccak256,
};
use hex_literal::hex;
use log::info;
use std::sync::Arc;
use std::{ops::Add, str::FromStr};

/// Deploy a MULTICALL contract into Anvil local chain for testing
pub async fn deploy_multicall3_for_testing<M>(provider: Arc<M>) -> Result<(), ContractError<M>>
where
    M: Middleware,
{
    // Fund Multicall3 deployer and deploy ERC1820Registry
    let mut tx = Eip1559TransactionRequest::new();
    tx = tx.to(H160::from_str(crate::utils::MULTICALL3_DEPLOYER).unwrap());
    tx = tx.value(crate::utils::ETH_VALUE_FOR_MULTICALL3_DEPLOYER);

    provider
        .send_transaction(tx, None)
        .await
        .map_err(|e| ContractError::MiddlewareError { e })?
        .await?;

    provider.send_raw_transaction(
        hex!("f90f538085174876e800830f42408080b90f00608060405234801561001057600080fd5b50610ee0806100206000396000f3fe6080604052600436106100f35760003560e01c80634d2301cc1161008a578063a8b0574e11610059578063a8b0574e1461025a578063bce38bd714610275578063c3077fa914610288578063ee82ac5e1461029b57600080fd5b80634d2301cc146101ec57806372425d9d1461022157806382ad56cb1461023457806386d516e81461024757600080fd5b80633408e470116100c65780633408e47014610191578063399542e9146101a45780633e64a696146101c657806342cbb15c146101d957600080fd5b80630f28c97d146100f8578063174dea711461011a578063252dba421461013a57806327e86d6e1461015b575b600080fd5b34801561010457600080fd5b50425b6040519081526020015b60405180910390f35b61012d610128366004610a85565b6102ba565b6040516101119190610bbe565b61014d610148366004610a85565b6104ef565b604051610111929190610bd8565b34801561016757600080fd5b50437fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0140610107565b34801561019d57600080fd5b5046610107565b6101b76101b2366004610c60565b610690565b60405161011193929190610cba565b3480156101d257600080fd5b5048610107565b3480156101e557600080fd5b5043610107565b3480156101f857600080fd5b50610107610207366004610ce2565b73ffffffffffffffffffffffffffffffffffffffff163190565b34801561022d57600080fd5b5044610107565b61012d610242366004610a85565b6106ab565b34801561025357600080fd5b5045610107565b34801561026657600080fd5b50604051418152602001610111565b61012d610283366004610c60565b61085a565b6101b7610296366004610a85565b610a1a565b3480156102a757600080fd5b506101076102b6366004610d18565b4090565b60606000828067ffffffffffffffff8111156102d8576102d8610d31565b60405190808252806020026020018201604052801561031e57816020015b6040805180820190915260008152606060208201528152602001906001900390816102f65790505b5092503660005b8281101561047757600085828151811061034157610341610d60565b6020026020010151905087878381811061035d5761035d610d60565b905060200281019061036f9190610d8f565b6040810135958601959093506103886020850185610ce2565b73ffffffffffffffffffffffffffffffffffffffff16816103ac6060870187610dcd565b6040516103ba929190610e32565b60006040518083038185875af1925050503d80600081146103f7576040519150601f19603f3d011682016040523d82523d6000602084013e6103fc565b606091505b50602080850191909152901515808452908501351761046d577f08c379a000000000000000000000000000000000000000000000000000000000600052602060045260176024527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060445260846000fd5b5050600101610325565b508234146104e6576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601a60248201527f4d756c746963616c6c333a2076616c7565206d69736d6174636800000000000060448201526064015b60405180910390fd5b50505092915050565b436060828067ffffffffffffffff81111561050c5761050c610d31565b60405190808252806020026020018201604052801561053f57816020015b606081526020019060019003908161052a5790505b5091503660005b8281101561068657600087878381811061056257610562610d60565b90506020028101906105749190610e42565b92506105836020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff166105a66020850185610dcd565b6040516105b4929190610e32565b6000604051808303816000865af19150503d80600081146105f1576040519150601f19603f3d011682016040523d82523d6000602084013e6105f6565b606091505b5086848151811061060957610609610d60565b602090810291909101015290508061067d576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601760248201527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060448201526064016104dd565b50600101610546565b5050509250929050565b43804060606106a086868661085a565b905093509350939050565b6060818067ffffffffffffffff8111156106c7576106c7610d31565b60405190808252806020026020018201604052801561070d57816020015b6040805180820190915260008152606060208201528152602001906001900390816106e55790505b5091503660005b828110156104e657600084828151811061073057610730610d60565b6020026020010151905086868381811061074c5761074c610d60565b905060200281019061075e9190610e76565b925061076d6020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff166107906040850185610dcd565b60405161079e929190610e32565b6000604051808303816000865af19150503d80600081146107db576040519150601f19603f3d011682016040523d82523d6000602084013e6107e0565b606091505b506020808401919091529015158083529084013517610851577f08c379a000000000000000000000000000000000000000000000000000000000600052602060045260176024527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060445260646000fd5b50600101610714565b6060818067ffffffffffffffff81111561087657610876610d31565b6040519080825280602002602001820160405280156108bc57816020015b6040805180820190915260008152606060208201528152602001906001900390816108945790505b5091503660005b82811015610a105760008482815181106108df576108df610d60565b602002602001015190508686838181106108fb576108fb610d60565b905060200281019061090d9190610e42565b925061091c6020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff1661093f6020850185610dcd565b60405161094d929190610e32565b6000604051808303816000865af19150503d806000811461098a576040519150601f19603f3d011682016040523d82523d6000602084013e61098f565b606091505b506020830152151581528715610a07578051610a07576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601760248201527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060448201526064016104dd565b506001016108c3565b5050509392505050565b6000806060610a2b60018686610690565b919790965090945092505050565b60008083601f840112610a4b57600080fd5b50813567ffffffffffffffff811115610a6357600080fd5b6020830191508360208260051b8501011115610a7e57600080fd5b9250929050565b60008060208385031215610a9857600080fd5b823567ffffffffffffffff811115610aaf57600080fd5b610abb85828601610a39565b90969095509350505050565b6000815180845260005b81811015610aed57602081850181015186830182015201610ad1565b81811115610aff576000602083870101525b50601f017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0169290920160200192915050565b600082825180855260208086019550808260051b84010181860160005b84811015610bb1578583037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe001895281518051151584528401516040858501819052610b9d81860183610ac7565b9a86019a9450505090830190600101610b4f565b5090979650505050505050565b602081526000610bd16020830184610b32565b9392505050565b600060408201848352602060408185015281855180845260608601915060608160051b870101935082870160005b82811015610c52577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa0888703018452610c40868351610ac7565b95509284019290840190600101610c06565b509398975050505050505050565b600080600060408486031215610c7557600080fd5b83358015158114610c8557600080fd5b9250602084013567ffffffffffffffff811115610ca157600080fd5b610cad86828701610a39565b9497909650939450505050565b838152826020820152606060408201526000610cd96060830184610b32565b95945050505050565b600060208284031215610cf457600080fd5b813573ffffffffffffffffffffffffffffffffffffffff81168114610bd157600080fd5b600060208284031215610d2a57600080fd5b5035919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81833603018112610dc357600080fd5b9190910192915050565b60008083357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe1843603018112610e0257600080fd5b83018035915067ffffffffffffffff821115610e1d57600080fd5b602001915036819003821315610a7e57600080fd5b8183823760009101908152919050565b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc1833603018112610dc357600080fd5b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa1833603018112610dc357600080fdfea2646970667358221220bb2b5c71a328032f97c676ae39a1ec2148d3e5d6f73d95e9b17910152d61f16264736f6c634300080c00331ca0edce47092c0f398cebf3ffc267f05c8e7076e3b89445e0fe50f6332273d4569ba01b0b9d000e19b24c5869b0fc3b22b0d6fa47cd63316875cbbd577d76e6fde086")
            .into()).await.map_err(|e| ContractError::MiddlewareError {e})?.await?;
    Ok(())
}

/// Get native balance and hopr token balance for given addresses
pub async fn get_native_and_token_balances<M>(
    hopr_token: HoprToken<M>,
    addresses: Vec<Address>,
) -> Result<(Vec<U256>, Vec<U256>), MulticallError<M>>
where
    M: Middleware,
{
    let provider = hopr_token.client();
    let mut multicall = Multicall::new(provider.clone(), Some(MULTICALL_ADDRESS)).await?;

    for address in addresses {
        multicall.add_get_eth_balance(address, false).add_call(
            hopr_token
                .method::<_, U256>("balanceOf", address)
                .map_err(|e| MulticallError::ContractError(ContractError::AbiError(e)))?,
            false,
        );
    }

    let response: Vec<U256> = multicall.call_array().await?;
    let mut native_balance: Vec<U256> = vec![];
    let mut token_balance: Vec<U256> = vec![];

    for (i, &balance) in response.iter().enumerate() {
        if i % 2 == 0 {
            native_balance.push(balance);
        } else {
            token_balance.push(balance);
        }
    }

    Ok((native_balance, token_balance))
}

/// Transfer some HOPR tokens from the caller to the list of addresses
/// Address_i receives amounts_i HOPR tokens.
/// When there's not enough token in caller's balance, if the caller is
/// a minter, mint the missing tokens. If not, returns error
pub async fn transfer_or_mint_tokens<M>(
    hopr_token: HoprToken<M>,
    addresses: Vec<Address>,
    amounts: Vec<U256>,
) -> Result<U256, HelperErrors>
where
    M: Middleware,
{
    let caller = hopr_token.client().default_sender().expect("client must have a sender");
    let provider = hopr_token.client();
    let mut multicall = Multicall::new(provider.clone(), Some(MULTICALL_ADDRESS))
        .await
        .expect("cannot create multicall");

    // check if two vectors have the same length
    assert_eq!(
        addresses.len(),
        amounts.len(),
        "addresses and amounts are of different lengths in transfer_or_mint_tokens"
    );
    // calculate the sum of tokens to be sent
    let total = amounts.iter().fold(U256::zero(), |acc, cur| acc.add(cur));
    info!("total amount of HOPR tokens to be transferred {:?}", total.to_string());

    // get caller balance and its role
    let encoded_minter_role: [u8; 32] = keccak256(b"MINTER_ROLE");
    multicall
        .add_call(
            hopr_token
                .method::<_, U256>("balanceOf", caller)
                .map_err(|e| HelperErrors::MulticallError(e.to_string()))?,
            false,
        )
        .add_call(
            hopr_token
                .method::<_, bool>("hasRole", (encoded_minter_role, caller))
                .map_err(|e| HelperErrors::MulticallError(e.to_string()))?,
            false,
        );
    let result: (U256, bool) = multicall.call().await.map_err(|_| {
        HelperErrors::MulticallError(
            "failed in getting caller balance and its role in transfer_or_mint_tokens".to_string(),
        )
    })?;

    // compare the total with caller's current balance. If caller doens't have enough balance, try to mint some. Otherwise, revert
    if total.gt(&result.0) {
        info!("caller does not have enough balance to transfer tokens to recipients.");
        if result.1 {
            info!("caller tries to mint tokens");
            hopr_token
                .mint(caller, total, Bytes::default(), Bytes::default())
                .send()
                .await
                .unwrap()
                .await
                .unwrap();
        } else {
            return Err(HelperErrors::NotAMinter);
        }
    }

    // approve the multicall to be able to transfer from caller's wallet
    hopr_token
        .approve(MULTICALL_ADDRESS, total)
        .send()
        .await
        .unwrap()
        .await
        .unwrap();

    // transfer token to multicall contract and transfer to multiple recipients
    for (i, address) in addresses.iter().enumerate() {
        // skip transferring zero amount of tokens
        if amounts[i].gt(&U256::zero()) {
            multicall.add_call(
                hopr_token
                    .method::<_, bool>("transferFrom", (caller.clone(), address.clone(), amounts[i]))
                    .map_err(|e| HelperErrors::MulticallError(e.to_string()))?,
                false,
            );
        }
    }

    multicall.send().await.unwrap().await.unwrap();

    Ok(total)
}

/// Transfer some native tokens from the caller to the list of addresses
/// Address_i receives amounts_i native tokens.
/// TODO: improve this to use `aggregate3Value` if possible
pub async fn transfer_native_tokens<M: Middleware>(
    provider: Arc<M>,
    addresses: Vec<Address>,
    amounts: Vec<U256>,
) -> Result<U256, HelperErrors> {
    // let caller = provider.default_sender().expect("client must have a sender");

    // check if two vectors have the same length
    assert_eq!(
        addresses.len(),
        amounts.len(),
        "addresses and amounts are of different lengths in transfer_native_tokens"
    );
    // calculate the sum of tokens to be sent
    let total = amounts.iter().fold(U256::zero(), |acc, cur| acc.add(cur));
    info!(
        "total amount of native tokens to be transferred {:?}",
        total.to_string()
    );

    let call3values: Vec<Call3Value> = addresses
        .iter()
        .enumerate()
        .map(|(i, addr)| Call3Value {
            target: addr.clone(),
            allow_failure: false,
            value: amounts[i].into(),
            call_data: Bytes::default(),
        })
        .collect();
    // transfer native tokens to multicall contract and transfer to multiple recipients
    let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());
    tx.set_to(MULTICALL_ADDRESS);
    tx.set_data(Aggregate3ValueCall { calls: call3values }.encode().into());
    tx.set_value(total);

    provider
        .send_transaction(tx, None)
        .await
        .map_err(|e| HelperErrors::MiddlewareError(e.to_string()))?
        .await
        .unwrap();

    Ok(total)
}

/// Get registered safes for given nodes on the network registry
pub async fn get_registered_safes_for_nodes_on_network_registry<M>(
    network_registry: HoprNetworkRegistry<M>,
    node_addresses: Vec<H160>,
) -> Result<Vec<H160>, MulticallError<M>>
where
    M: Middleware,
{
    let provider = network_registry.client();
    let mut multicall = Multicall::new(provider.clone(), Some(MULTICALL_ADDRESS))
        .await
        .expect("cannot create multicall");

    for node in node_addresses {
        multicall.add_call(
            network_registry
                .method::<_, Address>("nodeRegisterdToAccount", node)
                .map_err(|e| MulticallError::ContractError(ContractError::AbiError(e)))?,
            false,
        );
    }

    let response: Vec<Address> = multicall.call_array().await?;

    Ok(response)
}

/// Register safes and nodes to the network registry, and force-sync the eligibility to true.
/// It returns the number of removed nodes and nodes being added.
/// - If nodes have been registered to a different safe, overwrite it (remove the old safe and regsiter with the new safe)
/// - If ndoes have been registered to the same safe, no op
/// - If nodes have not been registered to any safe, register it
/// After all the nodes have been added to the network registry, force-sync the eligibility of all the added safes to true
pub async fn register_safes_and_nodes_on_network_registry<M>(
    network_registry: HoprNetworkRegistry<M>,
    safe_addresses: Vec<H160>,
    node_addresses: Vec<H160>,
) -> Result<(usize, usize), HelperErrors>
where
    M: Middleware,
{
    assert_eq!(
        safe_addresses.len(),
        node_addresses.len(),
        "unmatched lengths of safes and nodes"
    );

    // check registered safes of given node addresses
    let registered_safes =
        get_registered_safes_for_nodes_on_network_registry(network_registry.clone(), node_addresses.clone())
            .await
            .unwrap();

    let mut nodes_to_remove: Vec<H160> = Vec::new();
    let mut safes_to_add: Vec<H160> = Vec::new();
    let mut nodes_to_add: Vec<H160> = Vec::new();

    for (i, registered_safe) in registered_safes.iter().enumerate() {
        if registered_safe.eq(&H160::zero()) {
            // no entry, add to network registry
            safes_to_add.push(safe_addresses[i]);
            nodes_to_add.push(node_addresses[i]);
        } else if registered_safe.ne(&safe_addresses[i]) {
            // remove first then add
            nodes_to_remove.push(node_addresses[i]);
            safes_to_add.push(safe_addresses[i]);
            nodes_to_add.push(node_addresses[i]);
        } else {
            // no-op
        }
    }

    if !nodes_to_remove.is_empty() {
        // need to remove some nodes
        network_registry
            .manager_deregister(nodes_to_remove.clone())
            .send()
            .await
            .unwrap()
            .await
            .unwrap();
    }

    network_registry
        .manager_register(safes_to_add.clone(), nodes_to_add.clone())
        .send()
        .await
        .unwrap()
        .await
        .unwrap();

    // force sync their eligibility
    network_registry
        .manager_force_sync(safes_to_add.clone(), vec![true; safes_to_add.len()])
        .send()
        .await
        .unwrap()
        .await
        .unwrap();

    Ok((nodes_to_remove.len(), nodes_to_add.len()))
}

/// Deregister safes and nodes to the network registry. Does not do any action on the eligibility.
/// It returns the number of removed nodes
/// - If nodes have been registered to a safe, remove the node
/// - If nodes have not been registered to any safe, no op
pub async fn deregister_nodes_from_network_registry<M>(
    network_registry: HoprNetworkRegistry<M>,
    node_addresses: Vec<H160>,
) -> Result<usize, HelperErrors>
where
    M: Middleware,
{
    // check registered safes of given node addresses
    let registered_safes =
        get_registered_safes_for_nodes_on_network_registry(network_registry.clone(), node_addresses.clone())
            .await
            .unwrap();

    let mut nodes_to_remove: Vec<H160> = Vec::new();

    for (i, registered_safe) in registered_safes.iter().enumerate() {
        if registered_safe.ne(&H160::zero()) {
            // remove the node
            nodes_to_remove.push(node_addresses[i]);
        }
    }

    if !nodes_to_remove.is_empty() {
        // need to remove some nodes
        network_registry
            .manager_deregister(nodes_to_remove.clone())
            .send()
            .await
            .unwrap()
            .await
            .unwrap();
    }
    Ok(nodes_to_remove.len())
}

/// Force-sync the eligibility to given values. This can only be called with a manager account
pub async fn force_sync_safes_on_network_registry<M>(
    network_registry: HoprNetworkRegistry<M>,
    safe_addresses: Vec<H160>,
    eligibilities: Vec<bool>,
) -> Result<(), HelperErrors>
where
    M: Middleware,
{
    assert_eq!(
        safe_addresses.len(),
        eligibilities.len(),
        "unmatched lengths of safes and eligibilities"
    );

    // force sync their eligibility
    network_registry
        .manager_force_sync(safe_addresses, eligibilities)
        .send()
        .await
        .unwrap()
        .await
        .unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chain_rpc::client::{create_rpc_client_to_anvil, native::SurfRequestor};
    use chain_types::ContractInstances;
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
    use hopr_primitive_types::primitives::Address;

    #[async_std::test]
    async fn test_native_and_token_balances_in_anvil_with_multicall() {
        // create a keypair
        let kp = ChainKeypair::random();
        let kp_address = Address::from(&kp);

        // launch local anvil instance
        let anvil = chain_types::utils::create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();
        let client = create_rpc_client_to_anvil(SurfRequestor::default(), &anvil, &contract_deployer);
        // deploy hopr contracts
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");
        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await.unwrap();

        // get native and token balances
        let (native_balance, token_balance) = get_native_and_token_balances(instances.token, vec![kp_address.into()])
            .await
            .unwrap();
        assert_eq!(native_balance.len(), 1, "invalid native balance lens");
        assert_eq!(token_balance.len(), 1, "invalid token balance lens");
        assert_eq!(native_balance[0].as_u64(), 0u64, "wrong native balance");
        assert_eq!(token_balance[0].as_u64(), 0u64, "wrong token balance");
        drop(anvil);
    }

    #[async_std::test]
    async fn test_transfer_or_mint_tokens_in_anvil_with_multicall() {
        let mut addresses: Vec<ethers::types::Address> = Vec::new();
        for _ in 0..4 {
            addresses.push(Address::random().into());
        }
        let desired_amount = vec![U256::from(1), U256::from(2), U256::from(3), U256::from(4)];

        // launch local anvil instance
        let anvil = chain_types::utils::create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();
        let client = create_rpc_client_to_anvil(SurfRequestor::default(), &anvil, &contract_deployer);
        // deploy hopr contracts
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");

        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await.unwrap();
        // grant deployer token minter role
        let encoded_minter_role: [u8; 32] = keccak256(b"MINTER_ROLE");
        instances
            .token
            .grant_role(
                encoded_minter_role.clone(),
                contract_deployer.public().to_address().into(),
            )
            .send()
            .await
            .unwrap()
            .await
            .unwrap();

        // test the deployer has minter role now
        let check_minter_role = instances
            .token
            .has_role(
                encoded_minter_role.clone(),
                contract_deployer.public().to_address().into(),
            )
            .call()
            .await
            .unwrap();
        assert!(check_minter_role, "deployer does not have minter role yet");

        // transfer or mint tokens to addresses
        let total_transferred_amount =
            transfer_or_mint_tokens(instances.token.clone(), addresses.clone(), desired_amount.clone())
                .await
                .unwrap();

        // get native and token balances
        let (native_balance, token_balance) = get_native_and_token_balances(instances.token, addresses.clone().into())
            .await
            .unwrap();
        assert_eq!(native_balance.len(), 4, "invalid native balance lens");
        assert_eq!(token_balance.len(), 4, "invalid token balance lens");
        for (i, amount) in desired_amount.iter().enumerate() {
            assert_eq!(token_balance[i].as_u64(), amount.as_u64(), "token balance unmatch");
        }

        assert_eq!(
            total_transferred_amount,
            U256::from(10),
            "amount transferred does not equal to the desired amount"
        );
    }

    #[async_std::test]
    async fn test_transfer_native_tokens_in_anvil_with_multicall() {
        let mut addresses: Vec<ethers::types::Address> = Vec::new();
        for _ in 0..4 {
            addresses.push(Address::random().into());
        }
        let desired_amount = vec![U256::from(1), U256::from(2), U256::from(3), U256::from(4)];

        // launch local anvil instance
        let anvil = chain_types::utils::create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();
        let client = create_rpc_client_to_anvil(SurfRequestor::default(), &anvil, &contract_deployer);
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");

        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await.unwrap();

        // transfer native tokens to addresses
        let total_transferred_amount =
            transfer_native_tokens(client.clone(), addresses.clone(), desired_amount.clone())
                .await
                .unwrap();

        // get native and token balances
        let (native_balance, token_balance) = get_native_and_token_balances(instances.token, addresses.clone().into())
            .await
            .unwrap();
        assert_eq!(native_balance.len(), 4, "invalid native balance lens");
        assert_eq!(token_balance.len(), 4, "invalid token balance lens");
        for (i, amount) in desired_amount.iter().enumerate() {
            assert_eq!(native_balance[i].as_u64(), amount.as_u64(), "native balance unmatch");
        }

        assert_eq!(
            total_transferred_amount,
            U256::from(10),
            "amount transferred does not equal to the desired amount"
        );
    }

    #[async_std::test]
    async fn test_register_safes_and_nodes_then_deregister_nodes_in_anvil_with_multicall() {
        let mut safe_addresses: Vec<ethers::types::Address> = Vec::new();
        let mut node_addresses: Vec<ethers::types::Address> = Vec::new();
        for _ in 0..4 {
            safe_addresses.push(Address::random().into());
            node_addresses.push(Address::random().into());
        }

        // launch local anvil instance
        let anvil = chain_types::utils::create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();
        let client = create_rpc_client_to_anvil(SurfRequestor::default(), &anvil, &contract_deployer);
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");
        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await.unwrap();

        // register some nodes
        let (removed_amt, added_amt) = register_safes_and_nodes_on_network_registry(
            instances.network_registry.clone(),
            safe_addresses.clone(),
            node_addresses.clone(),
        )
        .await
        .unwrap();

        assert_eq!(removed_amt, 0, "should not remove any safe");
        assert_eq!(added_amt, 4, "there should be 4 additions");

        // get registered safes from nodes
        let registered_safes = get_registered_safes_for_nodes_on_network_registry(
            instances.network_registry.clone(),
            node_addresses.clone(),
        )
        .await
        .unwrap();

        assert_eq!(safe_addresses.len(), registered_safes.len(), "returned length unmatch");
        for (i, safe_addr) in safe_addresses.iter().enumerate() {
            assert_eq!(safe_addr, &registered_safes[i], "registered safe addresses unmatch");
        }

        // deregister 3 of them
        let deregisterd_nodes = deregister_nodes_from_network_registry(
            instances.network_registry.clone(),
            node_addresses.split_at(3).0.to_vec(),
        )
        .await
        .unwrap();
        assert_eq!(deregisterd_nodes, 3, "cannot deregister all the nodes");

        // re-register 4 of them
        let (removed_amt_2, added_amt_2) = register_safes_and_nodes_on_network_registry(
            instances.network_registry.clone(),
            safe_addresses.clone(),
            node_addresses.clone(),
        )
        .await
        .unwrap();

        assert_eq!(removed_amt_2, 0, "should not remove any safe");
        assert_eq!(added_amt_2, 3, "there should be 3 additions");
    }
}
